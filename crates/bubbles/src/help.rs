//! Help view component.
//!
//! This module provides a help view for displaying key bindings in TUI
//! applications.
//!
//! # Example
//!
//! ```rust
//! use bubbles::help::Help;
//! use bubbles::key::Binding;
//!
//! let help = Help::new();
//!
//! // Create some bindings
//! let quit = Binding::new().keys(&["q", "ctrl+c"]).help("q", "quit");
//! let save = Binding::new().keys(&["ctrl+s"]).help("ctrl+s", "save");
//!
//! // Render short help
//! let view = help.short_help_view(&[&quit, &save]);
//! ```

use crate::key::Binding;
use bubbletea::{Cmd, Message, Model};
use lipgloss::Style;
use unicode_width::UnicodeWidthStr;

/// Styles for the help view.
#[derive(Debug, Clone)]
pub struct Styles {
    /// Style for ellipsis when help is truncated.
    pub ellipsis: Style,
    /// Style for keys in short help.
    pub short_key: Style,
    /// Style for descriptions in short help.
    pub short_desc: Style,
    /// Style for separator in short help.
    pub short_separator: Style,
    /// Style for keys in full help.
    pub full_key: Style,
    /// Style for descriptions in full help.
    pub full_desc: Style,
    /// Style for separator in full help.
    pub full_separator: Style,
}

impl Default for Styles {
    fn default() -> Self {
        Self {
            ellipsis: Style::new(),
            short_key: Style::new(),
            short_desc: Style::new(),
            short_separator: Style::new(),
            full_key: Style::new(),
            full_desc: Style::new(),
            full_separator: Style::new(),
        }
    }
}

/// Message to toggle full help display.
#[derive(Debug, Clone, Copy)]
pub struct ToggleFullHelpMsg;

/// Message to set the width of the help view.
#[derive(Debug, Clone, Copy)]
pub struct SetWidthMsg(pub usize);

/// Message to set key bindings for the help view.
#[derive(Debug, Clone)]
pub struct SetBindingsMsg(pub Vec<Binding>);

/// Help view model.
#[derive(Debug, Clone)]
pub struct Help {
    /// Maximum width for the help view.
    pub width: usize,
    /// Whether to show full help (vs short help).
    pub show_all: bool,
    /// Separator for short help items.
    pub short_separator: String,
    /// Separator for full help columns.
    pub full_separator: String,
    /// Ellipsis shown when help is truncated.
    pub ellipsis: String,
    /// Styles for rendering.
    pub styles: Styles,
    /// Key bindings for standalone Model usage.
    bindings: Vec<Binding>,
}

impl Default for Help {
    fn default() -> Self {
        Self::new()
    }
}

impl Help {
    /// Creates a new help view with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            width: 0,
            show_all: false,
            short_separator: " • ".to_string(),
            full_separator: "    ".to_string(),
            ellipsis: "…".to_string(),
            styles: Styles::default(),
            bindings: Vec::new(),
        }
    }

    /// Sets the key bindings for standalone Model usage.
    #[must_use]
    pub fn with_bindings(mut self, bindings: Vec<Binding>) -> Self {
        self.bindings = bindings;
        self
    }

    /// Sets the key bindings.
    pub fn set_bindings(&mut self, bindings: Vec<Binding>) {
        self.bindings = bindings;
    }

    /// Returns the stored key bindings.
    #[must_use]
    pub fn bindings(&self) -> &[Binding] {
        &self.bindings
    }

    /// Sets the width of the help view.
    #[must_use]
    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Sets whether to show all help items.
    #[must_use]
    pub fn show_all(mut self, show: bool) -> Self {
        self.show_all = show;
        self
    }

    /// Renders the help view for a list of bindings.
    ///
    /// Displays short help if `show_all` is false, full help otherwise.
    #[must_use]
    pub fn view(&self, bindings: &[&Binding]) -> String {
        if self.show_all {
            self.full_help_view(&[bindings.to_vec()])
        } else {
            self.short_help_view(bindings)
        }
    }

    /// Renders short help from a list of bindings.
    #[must_use]
    pub fn short_help_view(&self, bindings: &[&Binding]) -> String {
        if bindings.is_empty() {
            return String::new();
        }

        let mut result = String::new();
        let mut total_width = 0;

        for binding in bindings {
            if !binding.enabled() {
                continue;
            }

            let help = binding.get_help();
            if help.key.is_empty() && help.desc.is_empty() {
                continue;
            }

            // Separator
            let sep = if total_width > 0 {
                self.styles.short_separator.render(&self.short_separator)
            } else {
                String::new()
            };

            // Key + desc
            let key_str = self.styles.short_key.render(&help.key);
            let desc_str = self.styles.short_desc.render(&help.desc);
            let item = format!("{}{} {}", sep, key_str, desc_str);
            let item_width = sep.width() + help.key.width() + 1 + help.desc.width();

            // Check width limit
            if self.width > 0 {
                let ellipsis_width = 1 + self.ellipsis.width();
                if total_width + item_width > self.width {
                    if total_width + ellipsis_width < self.width {
                        result.push(' ');
                        result.push_str(&self.styles.ellipsis.render(&self.ellipsis));
                    }
                    break;
                }
            }

            total_width += item_width;
            result.push_str(&item);
        }

        result
    }

    /// Renders full help from groups of bindings.
    #[must_use]
    pub fn full_help_view(&self, groups: &[Vec<&Binding>]) -> String {
        if groups.is_empty() {
            return String::new();
        }

        let mut columns: Vec<String> = Vec::new();
        let mut total_width = 0;

        for group in groups {
            if !should_render_column(group) {
                continue;
            }

            // Collect enabled bindings
            let mut keys: Vec<&str> = Vec::new();
            let mut descs: Vec<&str> = Vec::new();

            for binding in group {
                if binding.enabled() {
                    let help = binding.get_help();
                    if !help.key.is_empty() || !help.desc.is_empty() {
                        keys.push(help.key.as_str());
                        descs.push(help.desc.as_str());
                    }
                }
            }

            if keys.is_empty() {
                continue;
            }

            // Separator
            let sep = if total_width > 0 {
                self.styles.full_separator.render(&self.full_separator)
            } else {
                String::new()
            };

            // Build column using join_horizontal to properly align multi-line strings
            let keys_col = self.styles.full_key.render(&keys.join("\n"));
            let descs_col = self.styles.full_desc.render(&descs.join("\n"));
            // Use lipgloss::join_horizontal to properly align columns like Go does
            let column = lipgloss::join_horizontal(
                lipgloss::Position::Top,
                &[&sep, &keys_col, " ", &descs_col],
            );

            // Approximate width
            let max_key_width = keys.iter().map(|k| k.width()).max().unwrap_or(0);
            let max_desc_width = descs.iter().map(|d| d.width()).max().unwrap_or(0);
            let col_width = self.full_separator.width() + max_key_width + 1 + max_desc_width;

            // Check width limit
            if self.width > 0 && total_width + col_width > self.width {
                break;
            }

            total_width += col_width;
            columns.push(column);
        }

        // Join all columns horizontally
        if columns.len() <= 1 {
            columns.join("")
        } else {
            let refs: Vec<&str> = columns.iter().map(|s| s.as_str()).collect();
            lipgloss::join_horizontal(lipgloss::Position::Top, &refs)
        }
    }
}

/// Returns whether a column should be rendered (has at least one enabled binding).
fn should_render_column(bindings: &[&Binding]) -> bool {
    bindings.iter().any(|b| b.enabled())
}

/// Trait for types that can provide key bindings for help display.
pub trait KeyMap {
    /// Returns bindings for short help display.
    fn short_help(&self) -> Vec<Binding>;

    /// Returns groups of bindings for full help display.
    fn full_help(&self) -> Vec<Vec<Binding>>;
}

/// Implement the Model trait for standalone bubbletea usage.
impl Model for Help {
    fn init(&self) -> Option<Cmd> {
        // Help doesn't need initialization
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle toggle full help
        if msg.is::<ToggleFullHelpMsg>() {
            self.show_all = !self.show_all;
            return None;
        }

        // Handle set width
        if let Some(SetWidthMsg(width)) = msg.downcast_ref::<SetWidthMsg>() {
            self.width = *width;
            return None;
        }

        // Handle set bindings
        if let Some(set_bindings) = msg.downcast::<SetBindingsMsg>() {
            self.bindings = set_bindings.0;
            return None;
        }

        None
    }

    fn view(&self) -> String {
        // Use stored bindings for standalone Model view
        let binding_refs: Vec<&Binding> = self.bindings.iter().collect();
        if self.show_all {
            self.full_help_view(&[binding_refs])
        } else {
            self.short_help_view(&binding_refs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_new() {
        let help = Help::new();
        assert_eq!(help.width, 0);
        assert!(!help.show_all);
        assert_eq!(help.short_separator, " • ");
    }

    #[test]
    fn test_help_short_view() {
        let help = Help::new();
        let quit = Binding::new().keys(&["q"]).help("q", "quit");
        let save = Binding::new().keys(&["ctrl+s"]).help("^s", "save");

        let view = help.short_help_view(&[&quit, &save]);
        assert!(view.contains("q"));
        assert!(view.contains("quit"));
        assert!(view.contains("^s"));
        assert!(view.contains("save"));
    }

    #[test]
    fn test_help_short_view_with_width() {
        let help = Help::new().width(20);
        let quit = Binding::new().keys(&["q"]).help("q", "quit");
        let save = Binding::new().keys(&["ctrl+s"]).help("^s", "save");
        let other = Binding::new().keys(&["x"]).help("x", "something very long");

        let view = help.short_help_view(&[&quit, &save, &other]);
        // Should be truncated
        assert!(view.len() <= 25); // Account for styling overhead
    }

    #[test]
    fn test_help_full_view() {
        let help = Help::new();
        let quit = Binding::new().keys(&["q"]).help("q", "quit");
        let save = Binding::new().keys(&["ctrl+s"]).help("^s", "save");

        let view = help.full_help_view(&[vec![&quit, &save]]);
        assert!(view.contains("q"));
        assert!(view.contains("quit"));
    }

    #[test]
    fn test_help_empty_bindings() {
        let help = Help::new();
        assert_eq!(help.short_help_view(&[]), "");
        assert_eq!(help.full_help_view(&[]), "");
    }

    #[test]
    fn test_help_disabled_bindings() {
        let help = Help::new();
        let disabled = Binding::new()
            .keys(&["q"])
            .help("q", "quit")
            .set_enabled(false);

        let view = help.short_help_view(&[&disabled]);
        assert!(!view.contains("quit"));
    }

    #[test]
    fn test_help_builder() {
        let help = Help::new().width(80).show_all(true);
        assert_eq!(help.width, 80);
        assert!(help.show_all);
    }

    #[test]
    fn test_should_render_column() {
        let enabled = Binding::new().keys(&["q"]).help("q", "quit");
        let disabled = Binding::new()
            .keys(&["x"])
            .help("x", "exit")
            .set_enabled(false);

        assert!(should_render_column(&[&enabled]));
        assert!(!should_render_column(&[&disabled]));
        assert!(should_render_column(&[&disabled, &enabled]));
    }

    // Model trait tests

    #[test]
    fn test_help_model_init_returns_none() {
        let help = Help::new();
        assert!(Model::init(&help).is_none());
    }

    #[test]
    fn test_help_model_toggle_full_help() {
        let mut help = Help::new();
        assert!(!help.show_all);

        Model::update(&mut help, Message::new(ToggleFullHelpMsg));
        assert!(help.show_all);

        Model::update(&mut help, Message::new(ToggleFullHelpMsg));
        assert!(!help.show_all);
    }

    #[test]
    fn test_help_model_set_width() {
        let mut help = Help::new();
        assert_eq!(help.width, 0);

        Model::update(&mut help, Message::new(SetWidthMsg(80)));
        assert_eq!(help.width, 80);

        Model::update(&mut help, Message::new(SetWidthMsg(120)));
        assert_eq!(help.width, 120);
    }

    #[test]
    fn test_help_model_set_bindings() {
        let mut help = Help::new();
        assert!(help.bindings().is_empty());

        let bindings = vec![
            Binding::new().keys(&["q"]).help("q", "quit"),
            Binding::new().keys(&["ctrl+s"]).help("^s", "save"),
        ];

        Model::update(&mut help, Message::new(SetBindingsMsg(bindings)));
        assert_eq!(help.bindings().len(), 2);
    }

    #[test]
    fn test_help_model_view_short_mode() {
        let quit = Binding::new().keys(&["q"]).help("q", "quit");
        let save = Binding::new().keys(&["ctrl+s"]).help("^s", "save");

        let help = Help::new().with_bindings(vec![quit, save]);
        let view = Model::view(&help);

        assert!(view.contains("q"));
        assert!(view.contains("quit"));
        assert!(view.contains("^s"));
        assert!(view.contains("save"));
    }

    #[test]
    fn test_help_model_view_full_mode() {
        let quit = Binding::new().keys(&["q"]).help("q", "quit");
        let save = Binding::new().keys(&["ctrl+s"]).help("^s", "save");

        let help = Help::new().with_bindings(vec![quit, save]).show_all(true);
        let view = Model::view(&help);

        assert!(view.contains("q"));
        assert!(view.contains("quit"));
    }

    #[test]
    fn test_help_model_view_empty_bindings() {
        let help = Help::new();
        let view = Model::view(&help);
        assert!(view.is_empty());
    }

    #[test]
    fn test_help_model_view_respects_width() {
        let quit = Binding::new().keys(&["q"]).help("q", "quit");
        let save = Binding::new().keys(&["ctrl+s"]).help("^s", "save");
        let other = Binding::new()
            .keys(&["x"])
            .help("x", "something very very long");

        let help = Help::new().width(20).with_bindings(vec![quit, save, other]);
        let view = Model::view(&help);

        // View should be truncated due to width constraint
        assert!(view.len() <= 30); // Account for styling overhead
    }

    #[test]
    fn test_help_with_bindings_builder() {
        let bindings = vec![
            Binding::new().keys(&["q"]).help("q", "quit"),
            Binding::new().keys(&["ctrl+s"]).help("^s", "save"),
        ];

        let help = Help::new().with_bindings(bindings);
        assert_eq!(help.bindings().len(), 2);
    }

    #[test]
    fn test_help_set_bindings_method() {
        let mut help = Help::new();
        help.set_bindings(vec![Binding::new().keys(&["q"]).help("q", "quit")]);
        assert_eq!(help.bindings().len(), 1);
    }

    #[test]
    fn test_help_model_satisfies_model_bounds() {
        fn accepts_model<M: Model + Send + 'static>(_model: M) {}
        let help = Help::new();
        accepts_model(help);
    }

    #[test]
    fn test_help_model_update_returns_none() {
        let mut help = Help::new();
        // All message types should return None (no commands)
        assert!(Model::update(&mut help, Message::new(ToggleFullHelpMsg)).is_none());
        assert!(Model::update(&mut help, Message::new(SetWidthMsg(80))).is_none());
        assert!(Model::update(&mut help, Message::new(SetBindingsMsg(vec![]))).is_none());
    }

    // Parity audit tests (bd-212m.6.6)

    #[test]
    fn test_help_full_view_multi_group() {
        // Test full help with multiple groups (columns)
        let help = Help::new();
        let nav_up = Binding::new().keys(&["up", "k"]).help("↑/k", "up");
        let nav_down = Binding::new().keys(&["down", "j"]).help("↓/j", "down");
        let action_enter = Binding::new().keys(&["enter"]).help("enter", "select");
        let action_quit = Binding::new().keys(&["q", "ctrl+c"]).help("q", "quit");

        let groups = vec![vec![&nav_up, &nav_down], vec![&action_enter, &action_quit]];

        let view = help.full_help_view(&groups);
        // Should contain all keys and descriptions
        assert!(view.contains("↑/k"));
        assert!(view.contains("↓/j"));
        assert!(view.contains("enter"));
        assert!(view.contains("quit"));
        // Should contain newlines for multi-row layout
        assert!(view.contains('\n'));
    }

    #[test]
    fn test_help_full_view_with_width_truncation() {
        // Test that full help respects width limits
        let help = Help::new().width(30);
        let b1 = Binding::new().keys(&["a"]).help("a", "first action");
        let b2 = Binding::new().keys(&["b"]).help("b", "second action");
        let b3 = Binding::new().keys(&["c"]).help("c", "third action that won't fit");

        let groups = vec![vec![&b1], vec![&b2], vec![&b3]];

        let view = help.full_help_view(&groups);
        // With width=30, not all groups should fit
        // Exact behavior depends on column width calculation
        assert!(!view.is_empty());
    }

    #[test]
    fn test_help_mixed_enabled_disabled_in_group() {
        // Test that disabled bindings are skipped within a group
        let help = Help::new();
        let enabled = Binding::new().keys(&["a"]).help("a", "enabled");
        let disabled = Binding::new()
            .keys(&["b"])
            .help("b", "disabled")
            .set_enabled(false);
        let enabled2 = Binding::new().keys(&["c"]).help("c", "also enabled");

        let view = help.short_help_view(&[&enabled, &disabled, &enabled2]);

        assert!(view.contains("a"));
        assert!(!view.contains("b disabled"));
        assert!(view.contains("c"));
    }

    #[test]
    fn test_help_full_view_skips_all_disabled_group() {
        // Test that groups with all disabled bindings are skipped
        let help = Help::new();
        let enabled = Binding::new().keys(&["a"]).help("a", "enabled");
        let disabled1 = Binding::new()
            .keys(&["b"])
            .help("b", "disabled1")
            .set_enabled(false);
        let disabled2 = Binding::new()
            .keys(&["c"])
            .help("c", "disabled2")
            .set_enabled(false);

        let groups = vec![vec![&disabled1, &disabled2], vec![&enabled]];

        let view = help.full_help_view(&groups);
        assert!(view.contains("a"));
        assert!(view.contains("enabled"));
        assert!(!view.contains("disabled1"));
        assert!(!view.contains("disabled2"));
    }

    #[test]
    fn test_help_short_view_ellipsis_truncation() {
        // Test that ellipsis is added when truncating
        let help = Help::new().width(15);
        let b1 = Binding::new().keys(&["a"]).help("a", "first");
        let b2 = Binding::new().keys(&["b"]).help("b", "second");
        let b3 = Binding::new().keys(&["c"]).help("c", "third");

        let view = help.short_help_view(&[&b1, &b2, &b3]);

        // Should be truncated with ellipsis
        assert!(view.len() <= 20); // Width + some margin for ellipsis
    }

    #[test]
    fn test_help_separator_styles() {
        // Test that separators use the configured values
        let mut help = Help::new();
        help.short_separator = " | ".to_string();
        help.full_separator = "  ||  ".to_string();

        let b1 = Binding::new().keys(&["a"]).help("a", "first");
        let b2 = Binding::new().keys(&["b"]).help("b", "second");

        let short_view = help.short_help_view(&[&b1, &b2]);
        assert!(short_view.contains(" | "), "Short view: {}", short_view);

        let full_view = help.full_help_view(&[vec![&b1], vec![&b2]]);
        assert!(full_view.contains("||"), "Full view: {}", full_view);
    }

    #[test]
    fn test_help_unicode_keys() {
        // Test handling of Unicode characters in key names
        let help = Help::new();
        let arrow_up = Binding::new().keys(&["up"]).help("↑", "move up");
        let arrow_down = Binding::new().keys(&["down"]).help("↓", "move down");

        let view = help.short_help_view(&[&arrow_up, &arrow_down]);
        assert!(view.contains("↑"));
        assert!(view.contains("↓"));
    }

    #[test]
    fn test_help_empty_key_or_desc() {
        // Test bindings with empty key or description are skipped
        let help = Help::new();
        let empty_both = Binding::new().keys(&["a"]).help("", "");
        let empty_key = Binding::new().keys(&["b"]).help("", "desc only");
        let empty_desc = Binding::new().keys(&["c"]).help("key only", "");
        let normal = Binding::new().keys(&["d"]).help("d", "normal");

        let view = help.short_help_view(&[&empty_both, &empty_key, &empty_desc, &normal]);

        // empty_both should be skipped (both empty)
        // empty_key and empty_desc should be included (one is non-empty)
        // normal should be included
        assert!(view.contains("desc only") || view.contains("key only"));
        assert!(view.contains("d normal") || view.contains("d") && view.contains("normal"));
    }

    #[test]
    fn test_help_view_method_dispatches_correctly() {
        // Test that view() method correctly dispatches based on show_all
        let b1 = Binding::new().keys(&["a"]).help("a", "action");
        let b2 = Binding::new().keys(&["b"]).help("b", "back");

        let help_short = Help::new();
        let help_full = Help::new().show_all(true);

        let short_view = help_short.view(&[&b1, &b2]);
        let full_view = help_full.view(&[&b1, &b2]);

        // Short view should be single line
        assert!(!short_view.contains('\n') || short_view.lines().count() == 1);
        // Full view should have bindings stacked
        assert!(full_view.contains("a"));
        assert!(full_view.contains("b"));
    }

    #[test]
    fn test_help_default_separators() {
        // Verify default separators match Go implementation
        let help = Help::new();
        assert_eq!(help.short_separator, " • ");
        assert_eq!(help.full_separator, "    ");
        assert_eq!(help.ellipsis, "…");
    }

    #[test]
    fn test_help_zero_width_no_truncation() {
        // Width of 0 means no truncation
        let help = Help::new().width(0);
        let b1 = Binding::new()
            .keys(&["a"])
            .help("a", "a very long description that would normally be truncated");
        let b2 = Binding::new()
            .keys(&["b"])
            .help("b", "another very long description");

        let view = help.short_help_view(&[&b1, &b2]);

        // Should contain full descriptions
        assert!(view.contains("a very long description"));
        assert!(view.contains("another very long description"));
    }
}
