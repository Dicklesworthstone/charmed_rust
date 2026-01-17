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
use lipgloss::Style;

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
        }
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
            let item_width = sep.chars().count() + help.key.chars().count() + 1 + help.desc.chars().count();

            // Check width limit
            if self.width > 0 {
                let ellipsis_width = 1 + self.ellipsis.chars().count();
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

            // Build column
            let keys_col = self.styles.full_key.render(&keys.join("\n"));
            let descs_col = self.styles.full_desc.render(&descs.join("\n"));
            let column = format!("{}{} {}", sep, keys_col, descs_col);

            // Approximate width
            let max_key_width = keys.iter().map(|k| k.chars().count()).max().unwrap_or(0);
            let max_desc_width = descs.iter().map(|d| d.chars().count()).max().unwrap_or(0);
            let col_width = self.full_separator.chars().count() + max_key_width + 1 + max_desc_width;

            // Check width limit
            if self.width > 0 && total_width + col_width > self.width {
                break;
            }

            total_width += col_width;
            columns.push(column);
        }

        columns.join("")
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
        let disabled = Binding::new().keys(&["q"]).help("q", "quit").set_enabled(false);

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
        let disabled = Binding::new().keys(&["x"]).help("x", "exit").set_enabled(false);

        assert!(should_render_column(&[&enabled]));
        assert!(!should_render_column(&[&disabled]));
        assert!(should_render_column(&[&disabled, &enabled]));
    }
}
