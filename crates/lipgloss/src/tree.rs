//! Tree rendering.
//!
//! Port of Go lipgloss's `tree` subpackage. A [`Tree`] is a root value plus an
//! ordered list of child [`Node`]s, where each child is either a [`Leaf`] or a
//! nested [`Tree`]. Rendering draws branch connectors (the *enumerator*) in
//! front of every child and continuation lines (the *indenter*) in front of
//! every nested level.
//!
//! # Example
//!
//! ```rust
//! use lipgloss::tree::Tree;
//!
//! let t = Tree::new()
//!     .root(".")
//!     .child("macOS")
//!     .child(Tree::new().root("Linux").child("NixOS").child("Arch Linux"))
//!     .child("BSD");
//!
//! assert_eq!(
//!     t.to_string(),
//!     ".\n├── macOS\n├── Linux\n│   ├── NixOS\n│   └── Arch Linux\n└── BSD"
//! );
//! ```

use std::fmt;
use std::sync::Arc;

use crate::style::Style;
use crate::{Position, height, join_horizontal, join_vertical, visible_width};

/// Function producing the enumerator (branch connector) or indenter for the
/// child at `index` within `children`.
pub type Enumerator = Arc<dyn Fn(&[Node], usize) -> String + Send + Sync>;

/// Function producing the style for the child at `index` within `children`.
pub type StyleFunc = Arc<dyn Fn(&[Node], usize) -> Style + Send + Sync>;

/// Default tree enumerator: `├──` for inner children and `└──` for the last.
pub fn default_enumerator(children: &[Node], index: usize) -> String {
    if index + 1 == children.len() {
        "└──".to_string()
    } else {
        "├──".to_string()
    }
}

/// Rounded tree enumerator: `├──` for inner children and `╰──` for the last.
pub fn rounded_enumerator(children: &[Node], index: usize) -> String {
    if index + 1 == children.len() {
        "╰──".to_string()
    } else {
        "├──".to_string()
    }
}

/// Default tree indenter: `│  ` below inner children and blanks below the last.
pub fn default_indenter(children: &[Node], index: usize) -> String {
    if index + 1 == children.len() {
        "   ".to_string()
    } else {
        "│  ".to_string()
    }
}

/// A terminal node in a tree.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Leaf {
    value: String,
    hidden: bool,
}

impl Leaf {
    /// Creates a new visible leaf with the given value.
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            hidden: false,
        }
    }

    /// Returns a copy of this leaf with its hidden flag set.
    #[must_use]
    pub fn hide(mut self, hide: bool) -> Self {
        self.hidden = hide;
        self
    }

    /// Returns the leaf's value.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Returns whether the leaf is hidden.
    pub fn hidden(&self) -> bool {
        self.hidden
    }

    /// Sets the leaf's value.
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
    }
}

impl fmt::Display for Leaf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value)
    }
}

/// A node in a tree: either a leaf or a nested tree.
#[derive(Clone)]
pub enum Node {
    /// A terminal node.
    Leaf(Leaf),
    /// A nested tree whose root value is rendered as the item.
    Tree(Tree),
}

impl Node {
    /// Returns the node's value (the root value for trees).
    pub fn value(&self) -> &str {
        match self {
            Self::Leaf(l) => l.value(),
            Self::Tree(t) => t.value(),
        }
    }

    /// Returns whether the node is hidden.
    pub fn hidden(&self) -> bool {
        match self {
            Self::Leaf(l) => l.hidden(),
            Self::Tree(t) => t.hidden(),
        }
    }

    /// Returns the node's visible children window (empty for leaves).
    pub fn children(&self) -> &[Node] {
        match self {
            Self::Leaf(_) => &[],
            Self::Tree(t) => t.children_window(),
        }
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Leaf(l) => f.debug_tuple("Leaf").field(l).finish(),
            Self::Tree(t) => f.debug_tuple("Tree").field(t).finish(),
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Leaf(l) => l.fmt(f),
            Self::Tree(t) => t.fmt(f),
        }
    }
}

impl From<Leaf> for Node {
    fn from(l: Leaf) -> Self {
        Self::Leaf(l)
    }
}

impl From<Tree> for Node {
    fn from(t: Tree) -> Self {
        Self::Tree(t)
    }
}

impl From<&str> for Node {
    fn from(s: &str) -> Self {
        Self::Leaf(Leaf::new(s))
    }
}

impl From<String> for Node {
    fn from(s: String) -> Self {
        Self::Leaf(Leaf::new(s))
    }
}

impl From<&String> for Node {
    fn from(s: &String) -> Self {
        Self::Leaf(Leaf::new(s.as_str()))
    }
}

/// Rendering configuration for a tree level.
#[derive(Clone)]
pub(crate) struct TreeRenderer {
    enumerator: Enumerator,
    indenter: Enumerator,
    enumerator_style: StyleFunc,
    indenter_style: StyleFunc,
    item_style: StyleFunc,
    root_style: Style,
}

impl Default for TreeRenderer {
    fn default() -> Self {
        Self {
            enumerator: Arc::new(default_enumerator),
            indenter: Arc::new(default_indenter),
            enumerator_style: Arc::new(|_, _| Style::new().padding_right(1)),
            indenter_style: Arc::new(|_, _| Style::new().padding_right(1)),
            item_style: Arc::new(|_, _| Style::new()),
            root_style: Style::new(),
        }
    }
}

impl TreeRenderer {
    fn render(&self, node: &Tree, root: bool, prefix: &str) -> String {
        if node.hidden {
            return String::new();
        }
        let mut lines: Vec<String> = Vec::new();

        if root && !node.value.is_empty() {
            lines.push(self.root_style.render(&node.value));
        }

        // Hidden children are skipped entirely so the last *visible* child
        // gets the closing connector.
        let children: Vec<Node> = node
            .children_window()
            .iter()
            .filter(|c| !c.hidden())
            .cloned()
            .collect();

        let max_len = (0..children.len())
            .map(|i| {
                let e = (self.enumerator)(&children, i);
                visible_width(&(self.enumerator_style)(&children, i).render(&e))
            })
            .max()
            .unwrap_or(0);

        for (i, child) in children.iter().enumerate() {
            let indent_style = (self.indenter_style)(&children, i);
            let enum_style = (self.enumerator_style)(&children, i);
            let item_style = (self.item_style)(&children, i);

            let indent = (self.indenter)(&children, i);
            let mut node_prefix = enum_style.render(&(self.enumerator)(&children, i));
            let pad = max_len.saturating_sub(visible_width(&node_prefix));
            if pad > 0 {
                node_prefix = format!("{}{node_prefix}", " ".repeat(pad));
            }

            let item = item_style.render(child.value());
            let mut multiline_prefix = prefix.to_string();

            // Multi-line items extend the connector column with the indenter
            // and the inherited prefix so every line stays aligned.
            let item_height = height(&item);
            if item_height > height(&node_prefix) {
                let filler = enum_style.render(&indent);
                while height(&item) > height(&node_prefix) {
                    node_prefix = join_vertical(Position::Left, &[&node_prefix, &filler]);
                }
            }
            if !prefix.is_empty() {
                while height(&node_prefix) > height(&multiline_prefix) {
                    multiline_prefix = join_vertical(Position::Left, &[&multiline_prefix, prefix]);
                }
            }

            lines.push(join_horizontal(
                Position::Top,
                &[&multiline_prefix, &node_prefix, &item],
            ));

            if let Node::Tree(sub) = child {
                let renderer = sub.renderer.as_deref().unwrap_or(self);
                let child_prefix = format!("{prefix}{}", indent_style.render(&indent));
                let s = renderer.render(sub, false, &child_prefix);
                if !s.is_empty() {
                    lines.push(s);
                }
            }
        }

        lines.join("\n")
    }
}

/// A renderable tree.
#[derive(Clone, Default)]
pub struct Tree {
    value: String,
    hidden: bool,
    offset: (usize, usize),
    children: Vec<Node>,
    renderer: Option<Arc<TreeRenderer>>,
}

impl fmt::Debug for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tree")
            .field("value", &self.value)
            .field("hidden", &self.hidden)
            .field("offset", &self.offset)
            .field("children", &self.children)
            .finish_non_exhaustive()
    }
}

impl Tree {
    /// Creates an empty tree with no root value.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a tree with the given root value and children.
    pub fn with_root<I, N>(root: impl Into<String>, children: I) -> Self
    where
        I: IntoIterator<Item = N>,
        N: Into<Node>,
    {
        Self::new().root(root).children(children)
    }

    /// Sets the root value.
    #[must_use]
    pub fn root(mut self, root: impl Into<String>) -> Self {
        self.value = root.into();
        self
    }

    /// Returns the root value.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Sets whether the whole tree is hidden.
    #[must_use]
    pub fn hide(mut self, hide: bool) -> Self {
        self.hidden = hide;
        self
    }

    /// Returns whether the tree is hidden.
    pub fn hidden(&self) -> bool {
        self.hidden
    }

    /// Appends a child.
    ///
    /// Strings and [`Leaf`]s become leaves. A [`Tree`] without a root value
    /// is attached to the preceding sibling (matching Go lipgloss): if that
    /// sibling is a leaf it becomes the subtree's root, and if it is a tree
    /// the new children are appended to it.
    #[must_use]
    pub fn child(mut self, child: impl Into<Node>) -> Self {
        self.push_child(child.into());
        self
    }

    /// Appends several children in order.
    #[must_use]
    pub fn children<I, N>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = N>,
        N: Into<Node>,
    {
        for c in children {
            self.push_child(c.into());
        }
        self
    }

    /// Appends a child in place (see [`Tree::child`]).
    pub fn push_child(&mut self, child: Node) {
        match child {
            Node::Tree(sub) if sub.value.is_empty() && !self.children.is_empty() => {
                let last = self.children.pop().expect("non-empty children");
                match last {
                    Node::Tree(mut parent) => {
                        for c in sub.children {
                            parent.push_child(c);
                        }
                        self.children.push(Node::Tree(parent));
                    }
                    Node::Leaf(leaf) => {
                        let mut sub = sub;
                        sub.value = leaf.value;
                        sub.hidden = leaf.hidden;
                        self.children.push(Node::Tree(sub));
                    }
                }
            }
            other => self.children.push(other),
        }
    }

    /// Returns all children, ignoring the offset window.
    pub fn all_children(&self) -> &[Node] {
        &self.children
    }

    /// Returns the children inside the offset window (what gets rendered).
    pub fn children_window(&self) -> &[Node] {
        let len = self.children.len();
        let start = self.offset.0.min(len);
        let end = len.saturating_sub(self.offset.1).max(start);
        &self.children[start..end]
    }

    /// Skips `start` children from the front and `end` children from the back
    /// when rendering.
    #[must_use]
    pub fn offset(mut self, start: usize, end: usize) -> Self {
        self.offset = (start, end);
        self
    }

    fn renderer_mut(&mut self) -> &mut TreeRenderer {
        Arc::make_mut(self.renderer.get_or_insert_with(Arc::default))
    }

    /// Sets the enumerator (branch connector) function.
    #[must_use]
    pub fn enumerator<F>(mut self, f: F) -> Self
    where
        F: Fn(&[Node], usize) -> String + Send + Sync + 'static,
    {
        self.renderer_mut().enumerator = Arc::new(f);
        self
    }

    /// Sets the indenter function used for nested levels.
    #[must_use]
    pub fn indenter<F>(mut self, f: F) -> Self
    where
        F: Fn(&[Node], usize) -> String + Send + Sync + 'static,
    {
        self.renderer_mut().indenter = Arc::new(f);
        self
    }

    /// Sets a single style for all enumerators.
    #[must_use]
    pub fn enumerator_style(self, style: Style) -> Self {
        self.enumerator_style_func(move |_, _| style.clone())
    }

    /// Sets a per-child enumerator style function.
    #[must_use]
    pub fn enumerator_style_func<F>(mut self, f: F) -> Self
    where
        F: Fn(&[Node], usize) -> Style + Send + Sync + 'static,
    {
        self.renderer_mut().enumerator_style = Arc::new(f);
        self
    }

    /// Sets a single style for all indenters.
    #[must_use]
    pub fn indenter_style(self, style: Style) -> Self {
        self.indenter_style_func(move |_, _| style.clone())
    }

    /// Sets a per-child indenter style function.
    #[must_use]
    pub fn indenter_style_func<F>(mut self, f: F) -> Self
    where
        F: Fn(&[Node], usize) -> Style + Send + Sync + 'static,
    {
        self.renderer_mut().indenter_style = Arc::new(f);
        self
    }

    /// Sets a single style for all items.
    #[must_use]
    pub fn item_style(self, style: Style) -> Self {
        self.item_style_func(move |_, _| style.clone())
    }

    /// Sets a per-child item style function.
    #[must_use]
    pub fn item_style_func<F>(mut self, f: F) -> Self
    where
        F: Fn(&[Node], usize) -> Style + Send + Sync + 'static,
    {
        self.renderer_mut().item_style = Arc::new(f);
        self
    }

    /// Sets the style of the root value.
    #[must_use]
    pub fn root_style(mut self, style: Style) -> Self {
        self.renderer_mut().root_style = style;
        self
    }

    /// Renders the tree.
    pub fn render(&self) -> String {
        match &self.renderer {
            Some(r) => r.render(self, true, ""),
            None => TreeRenderer::default().render(self, true, ""),
        }
    }
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_tree_renders_empty() {
        assert_eq!(Tree::new().to_string(), "");
    }

    #[test]
    fn root_only() {
        assert_eq!(Tree::new().root("root").to_string(), "root");
    }

    #[test]
    fn flat_children_without_root() {
        let t = Tree::new().child("Foo").child("Bar");
        assert_eq!(t.to_string(), "├── Foo\n└── Bar");
    }

    #[test]
    fn nested_tree() {
        let t = Tree::new()
            .child("Foo")
            .child(
                Tree::new()
                    .root("Bar")
                    .child("Qux")
                    .child(Tree::new().root("Quux").child("Foo").child("Bar"))
                    .child("Quuux"),
            )
            .child("Baz");
        let want = "\
├── Foo
├── Bar
│   ├── Qux
│   ├── Quux
│   │   ├── Foo
│   │   └── Bar
│   └── Quuux
└── Baz";
        assert_eq!(t.to_string(), want);
    }

    #[test]
    fn rootless_subtree_attaches_to_previous_leaf() {
        let t = Tree::new()
            .root("Root")
            .child("Foo")
            .child("Bar")
            .child(Tree::new().child("Qux").child("Quux"))
            .child("Baz");
        let want = "\
Root
├── Foo
├── Bar
│   ├── Qux
│   └── Quux
└── Baz";
        assert_eq!(t.to_string(), want);
    }

    #[test]
    fn rootless_subtree_merges_into_previous_tree() {
        let t = Tree::new()
            .child(Tree::new().root("A").child("a1"))
            .child(Tree::new().child("a2"));
        assert_eq!(t.to_string(), "└── A\n    ├── a1\n    └── a2");
    }

    #[test]
    fn rootless_subtree_first_child_is_inlined_as_tree() {
        let t = Tree::new().child(Tree::new().child("x"));
        // No previous sibling: the rootless tree stays as an empty-valued node.
        assert_eq!(t.all_children().len(), 1);
    }

    #[test]
    fn rounded_enumerator_last_child() {
        let t = Tree::new()
            .child("a")
            .child("b")
            .enumerator(rounded_enumerator);
        assert_eq!(t.to_string(), "├── a\n╰── b");
    }

    #[test]
    fn hidden_children_are_skipped_and_last_visible_closes() {
        let t = Tree::new()
            .child("a")
            .child("b")
            .child(Leaf::new("c").hide(true));
        assert_eq!(t.to_string(), "├── a\n└── b");
    }

    #[test]
    fn hidden_tree_renders_empty() {
        let t = Tree::new().root("r").child("a").hide(true);
        assert_eq!(t.to_string(), "");
    }

    #[test]
    fn offset_window() {
        let t = Tree::new()
            .child("a")
            .child("b")
            .child("c")
            .child("d")
            .offset(1, 1);
        assert_eq!(t.to_string(), "├── b\n└── c");
    }

    #[test]
    fn offset_saturates() {
        let t = Tree::new().child("a").offset(5, 5);
        assert_eq!(t.to_string(), "");
    }

    #[test]
    fn multiline_item_extends_connector() {
        let t = Tree::new().child("line1\nline2").child("b");
        assert_eq!(t.to_string(), "├── line1\n│   line2\n└── b");
    }

    #[test]
    fn custom_enumerator_alignment() {
        let t = Tree::new()
            .children(["a", "b", "c"])
            .enumerator(|_, i| if i == 1 { "->".into() } else { "-".into() });
        // Shorter enumerators are right-aligned to the widest one.
        assert_eq!(t.to_string(), " - a\n-> b\n - c");
    }

    #[test]
    fn subtree_keeps_its_own_renderer() {
        let sub = Tree::new()
            .root("sub")
            .child("x")
            .child("y")
            .enumerator(rounded_enumerator);
        let t = Tree::new().child(sub).child("z");
        assert_eq!(t.to_string(), "├── sub\n│   ├── x\n│   ╰── y\n└── z");
    }

    #[test]
    fn with_root_constructor() {
        let t = Tree::with_root("r", ["a", "b"]);
        assert_eq!(t.to_string(), "r\n├── a\n└── b");
    }

    #[test]
    fn node_accessors() {
        let n: Node = Tree::new().root("r").child("a").into();
        assert_eq!(n.value(), "r");
        assert_eq!(n.children().len(), 1);
        assert!(!n.hidden());
        let l: Node = "leaf".into();
        assert!(l.children().is_empty());
        assert_eq!(l.to_string(), "leaf");
    }
}
