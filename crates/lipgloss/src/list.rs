//! List rendering.
//!
//! Port of Go lipgloss's `list` subpackage. A [`List`] is a flat sequence of
//! items rendered with an enumerator (bullets, numbers, letters, roman
//! numerals, ...). Lists nest: adding a list as an item attaches it as a
//! sublist of the preceding item.
//!
//! Internally a list is a [`Tree`] without a root, so the same enumerator and
//! style hooks are available.
//!
//! # Example
//!
//! ```rust
//! use lipgloss::list::{self, List};
//!
//! let l = List::new()
//!     .item("Foo")
//!     .item("Bar")
//!     .item(List::new().items(["Hi", "Hello", "Halo"]).enumerator(list::roman))
//!     .item("Qux");
//!
//! assert_eq!(
//!     l.to_string(),
//!     "• Foo\n• Bar\n    I. Hi\n   II. Hello\n  III. Halo\n• Qux"
//! );
//! ```

use std::fmt;

use crate::style::Style;
use crate::tree::{Leaf, Node, Tree};

/// Items of a list, as seen by enumerator and style functions.
pub type Items = [Node];

const ABC_LEN: usize = 26;

fn letter(n: usize) -> char {
    char::from(b'A' + u8::try_from(n % ABC_LEN).unwrap_or(0))
}

/// Alphabetical enumerator: `A.`, `B.`, ..., `Z.`, `AA.`, `AB.`, ...
pub fn alphabet(_items: &Items, index: usize) -> String {
    if index >= ABC_LEN * ABC_LEN + ABC_LEN {
        format!(
            "{}{}{}.",
            letter(index / ABC_LEN / ABC_LEN - 1),
            letter((index / ABC_LEN) % ABC_LEN + ABC_LEN - 1),
            letter(index)
        )
    } else if index >= ABC_LEN {
        format!("{}{}.", letter(index / ABC_LEN - 1), letter(index))
    } else {
        format!("{}.", letter(index))
    }
}

/// Arabic numeral enumerator: `1.`, `2.`, `3.`, ...
pub fn arabic(_items: &Items, index: usize) -> String {
    format!("{}.", index + 1)
}

/// Roman numeral enumerator: `I.`, `II.`, `III.`, `IV.`, ...
pub fn roman(_items: &Items, index: usize) -> String {
    const TABLE: [(usize, &str); 13] = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut n = index + 1;
    let mut out = String::new();
    for (value, numeral) in TABLE {
        while n >= value {
            n -= value;
            out.push_str(numeral);
        }
    }
    out.push('.');
    out
}

/// Bullet enumerator: `•`.
pub fn bullet(_items: &Items, _index: usize) -> String {
    "•".to_string()
}

/// Asterisk enumerator: `*`.
pub fn asterisk(_items: &Items, _index: usize) -> String {
    "*".to_string()
}

/// Dash enumerator: `-`.
pub fn dash(_items: &Items, _index: usize) -> String {
    "-".to_string()
}

/// An item that can be added to a [`List`].
pub enum ListItem {
    /// A plain item.
    Node(Node),
    /// A nested list.
    List(List),
}

impl From<&str> for ListItem {
    fn from(s: &str) -> Self {
        Self::Node(s.into())
    }
}

impl From<String> for ListItem {
    fn from(s: String) -> Self {
        Self::Node(s.into())
    }
}

impl From<&String> for ListItem {
    fn from(s: &String) -> Self {
        Self::Node(s.into())
    }
}

impl From<Leaf> for ListItem {
    fn from(l: Leaf) -> Self {
        Self::Node(l.into())
    }
}

impl From<Tree> for ListItem {
    fn from(t: Tree) -> Self {
        Self::Node(t.into())
    }
}

impl From<List> for ListItem {
    fn from(l: List) -> Self {
        Self::List(l)
    }
}

/// A renderable, optionally nested, enumerated list.
#[derive(Clone, Debug)]
pub struct List {
    tree: Tree,
}

impl Default for List {
    fn default() -> Self {
        Self::new()
    }
}

impl List {
    /// Creates an empty bulleted list.
    pub fn new() -> Self {
        Self {
            tree: Tree::new()
                .enumerator(bullet)
                .indenter(|_, _| " ".to_string()),
        }
    }

    /// Appends an item. Adding a [`List`] nests it under the previous item.
    #[must_use]
    pub fn item(mut self, item: impl Into<ListItem>) -> Self {
        let node = match item.into() {
            ListItem::Node(n) => n,
            ListItem::List(l) => Node::Tree(l.tree),
        };
        self.tree.push_child(node);
        self
    }

    /// Appends several items in order.
    #[must_use]
    pub fn items<I, T>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<ListItem>,
    {
        for item in items {
            self = self.item(item);
        }
        self
    }

    /// Sets whether the list is hidden.
    #[must_use]
    pub fn hide(mut self, hide: bool) -> Self {
        self.tree = self.tree.hide(hide);
        self
    }

    /// Returns whether the list is hidden.
    pub fn hidden(&self) -> bool {
        self.tree.hidden()
    }

    /// Renders only the items between `start` (from the front) and `end`
    /// (from the back).
    #[must_use]
    pub fn offset(mut self, start: usize, end: usize) -> Self {
        self.tree = self.tree.offset(start, end);
        self
    }

    /// Returns the list's items.
    pub fn get_items(&self) -> &Items {
        self.tree.all_children()
    }

    /// Sets the enumerator function.
    #[must_use]
    pub fn enumerator<F>(mut self, f: F) -> Self
    where
        F: Fn(&Items, usize) -> String + Send + Sync + 'static,
    {
        self.tree = self.tree.enumerator(f);
        self
    }

    /// Sets the indenter used for nested lists.
    #[must_use]
    pub fn indenter<F>(mut self, f: F) -> Self
    where
        F: Fn(&Items, usize) -> String + Send + Sync + 'static,
    {
        self.tree = self.tree.indenter(f);
        self
    }

    /// Sets a single style for all enumerators.
    #[must_use]
    pub fn enumerator_style(mut self, style: Style) -> Self {
        self.tree = self.tree.enumerator_style(style);
        self
    }

    /// Sets a per-item enumerator style function.
    #[must_use]
    pub fn enumerator_style_func<F>(mut self, f: F) -> Self
    where
        F: Fn(&Items, usize) -> Style + Send + Sync + 'static,
    {
        self.tree = self.tree.enumerator_style_func(f);
        self
    }

    /// Sets a single style for all indenters.
    #[must_use]
    pub fn indenter_style(mut self, style: Style) -> Self {
        self.tree = self.tree.indenter_style(style);
        self
    }

    /// Sets a per-item indenter style function.
    #[must_use]
    pub fn indenter_style_func<F>(mut self, f: F) -> Self
    where
        F: Fn(&Items, usize) -> Style + Send + Sync + 'static,
    {
        self.tree = self.tree.indenter_style_func(f);
        self
    }

    /// Sets a single style for all items.
    #[must_use]
    pub fn item_style(mut self, style: Style) -> Self {
        self.tree = self.tree.item_style(style);
        self
    }

    /// Sets a per-item style function.
    #[must_use]
    pub fn item_style_func<F>(mut self, f: F) -> Self
    where
        F: Fn(&Items, usize) -> Style + Send + Sync + 'static,
    {
        self.tree = self.tree.item_style_func(f);
        self
    }

    /// Converts the list into its underlying tree.
    pub fn into_tree(self) -> Tree {
        self.tree
    }

    /// Renders the list.
    pub fn render(&self) -> String {
        self.tree.render()
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enumerate(f: fn(&Items, usize) -> String, idx: usize) -> String {
        f(&[], idx)
    }

    #[test]
    fn bullet_list() {
        let l = List::new().items(["Foo", "Bar", "Baz"]);
        assert_eq!(l.to_string(), "• Foo\n• Bar\n• Baz");
    }

    #[test]
    fn empty_list() {
        assert_eq!(List::new().to_string(), "");
    }

    #[test]
    fn arabic_alignment() {
        let items: Vec<String> = (1..=10).map(|i| format!("i{i}")).collect();
        let out = List::new().items(&items).enumerator(arabic).to_string();
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[0], " 1. i1");
        assert_eq!(lines[9], "10. i10");
    }

    #[test]
    fn sublist() {
        let l = List::new()
            .item("Foo")
            .item("Bar")
            .item(List::new().items(["Hi", "Hello", "Halo"]).enumerator(roman))
            .item("Qux");
        assert_eq!(
            l.to_string(),
            "• Foo\n• Bar\n    I. Hi\n   II. Hello\n  III. Halo\n• Qux"
        );
    }

    #[test]
    fn deeply_nested() {
        let l = List::new()
            .item("a")
            .item(List::new().item("b").item(List::new().item("c")));
        assert_eq!(l.to_string(), "• a\n  • b\n    • c");
    }

    #[test]
    fn roman_numerals() {
        assert_eq!(enumerate(roman, 0), "I.");
        assert_eq!(enumerate(roman, 3), "IV.");
        assert_eq!(enumerate(roman, 8), "IX.");
        assert_eq!(enumerate(roman, 13), "XIV.");
        assert_eq!(enumerate(roman, 1993), "MCMXCIV.");
    }

    #[test]
    fn alphabet_enumeration() {
        assert_eq!(enumerate(alphabet, 0), "A.");
        assert_eq!(enumerate(alphabet, 25), "Z.");
        assert_eq!(enumerate(alphabet, 26), "AA.");
        assert_eq!(enumerate(alphabet, 27), "AB.");
        assert_eq!(enumerate(alphabet, 51), "AZ.");
        assert_eq!(enumerate(alphabet, 52), "BA.");
        assert_eq!(enumerate(alphabet, 701), "ZZ.");
        assert_eq!(enumerate(alphabet, 702), "AAA.");
    }

    #[test]
    fn simple_enumerators() {
        assert_eq!(enumerate(arabic, 0), "1.");
        assert_eq!(enumerate(bullet, 3), "•");
        assert_eq!(enumerate(asterisk, 3), "*");
        assert_eq!(enumerate(dash, 3), "-");
    }

    #[test]
    fn hidden_items_and_offset() {
        let l = List::new()
            .item("a")
            .item(Leaf::new("b").hide(true))
            .item("c")
            .item("d")
            .offset(0, 1);
        assert_eq!(l.to_string(), "• a\n• c");
    }

    #[test]
    fn hidden_list() {
        assert_eq!(List::new().item("a").hide(true).to_string(), "");
    }

    #[test]
    fn custom_enumerator_and_styles() {
        let l = List::new()
            .items(["x", "y"])
            .enumerator(|_, i| if i == 0 { "→".into() } else { " ".into() })
            .enumerator_style(Style::new().padding_right(2));
        assert_eq!(l.to_string(), "→  x\n   y");
    }

    #[test]
    fn get_items_reports_children() {
        let l = List::new().items(["a", "b"]);
        assert_eq!(l.get_items().len(), 2);
        assert_eq!(l.get_items()[1].value(), "b");
    }
}
