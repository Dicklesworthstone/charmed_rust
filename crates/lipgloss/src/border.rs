//! Border styles for terminal boxes.
//!
//! Lipgloss provides several preset border styles, and you can create custom ones.
//!
//! # Preset Borders
//!
//! - [`Border::normal()`] - Standard border (┌─┐)
//! - [`Border::rounded()`] - Rounded corners (╭─╮)
//! - [`Border::block()`] - Full block (█)
//! - [`Border::thick()`] - Thick lines (┏━┓)
//! - [`Border::double()`] - Double lines (╔═╗)
//! - [`Border::hidden()`] - Invisible (spaces)
//! - [`Border::ascii()`] - ASCII characters (+-|)
//!
//! # Example
//!
//! ```rust
//! use lipgloss::Border;
//!
//! let border = Border::rounded();
//! assert_eq!(border.top_left, "╭");
//! ```

use unicode_width::UnicodeWidthStr;

/// Border characters for all edges and corners.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Border {
    /// Top edge character(s).
    pub top: String,
    /// Bottom edge character(s).
    pub bottom: String,
    /// Left edge character(s).
    pub left: String,
    /// Right edge character(s).
    pub right: String,
    /// Top-left corner.
    pub top_left: String,
    /// Top-right corner.
    pub top_right: String,
    /// Bottom-left corner.
    pub bottom_left: String,
    /// Bottom-right corner.
    pub bottom_right: String,
    /// Middle-left connector (for tables).
    pub middle_left: String,
    /// Middle-right connector (for tables).
    pub middle_right: String,
    /// Middle cross (for tables).
    pub middle: String,
    /// Middle-top connector (for tables).
    pub middle_top: String,
    /// Middle-bottom connector (for tables).
    pub middle_bottom: String,
}

impl Border {
    /// Creates an empty border (no characters).
    pub const fn none() -> Self {
        Self {
            top: String::new(),
            bottom: String::new(),
            left: String::new(),
            right: String::new(),
            top_left: String::new(),
            top_right: String::new(),
            bottom_left: String::new(),
            bottom_right: String::new(),
            middle_left: String::new(),
            middle_right: String::new(),
            middle: String::new(),
            middle_top: String::new(),
            middle_bottom: String::new(),
        }
    }

    /// Standard border with 90-degree corners.
    ///
    /// ```text
    /// ┌───┐
    /// │   │
    /// └───┘
    /// ```
    pub fn normal() -> Self {
        Self {
            top: "─".into(),
            bottom: "─".into(),
            left: "│".into(),
            right: "│".into(),
            top_left: "┌".into(),
            top_right: "┐".into(),
            bottom_left: "└".into(),
            bottom_right: "┘".into(),
            middle_left: "├".into(),
            middle_right: "┤".into(),
            middle: "┼".into(),
            middle_top: "┬".into(),
            middle_bottom: "┴".into(),
        }
    }

    /// Border with rounded corners.
    ///
    /// ```text
    /// ╭───╮
    /// │   │
    /// ╰───╯
    /// ```
    pub fn rounded() -> Self {
        Self {
            top: "─".into(),
            bottom: "─".into(),
            left: "│".into(),
            right: "│".into(),
            top_left: "╭".into(),
            top_right: "╮".into(),
            bottom_left: "╰".into(),
            bottom_right: "╯".into(),
            middle_left: "├".into(),
            middle_right: "┤".into(),
            middle: "┼".into(),
            middle_top: "┬".into(),
            middle_bottom: "┴".into(),
        }
    }

    /// Full block border.
    ///
    /// ```text
    /// █████
    /// █   █
    /// █████
    /// ```
    pub fn block() -> Self {
        Self {
            top: "█".into(),
            bottom: "█".into(),
            left: "█".into(),
            right: "█".into(),
            top_left: "█".into(),
            top_right: "█".into(),
            bottom_left: "█".into(),
            bottom_right: "█".into(),
            middle_left: "█".into(),
            middle_right: "█".into(),
            middle: "█".into(),
            middle_top: "█".into(),
            middle_bottom: "█".into(),
        }
    }

    /// Half-block border (outer).
    pub fn outer_half_block() -> Self {
        Self {
            top: "▀".into(),
            bottom: "▄".into(),
            left: "▌".into(),
            right: "▐".into(),
            top_left: "▛".into(),
            top_right: "▜".into(),
            bottom_left: "▙".into(),
            bottom_right: "▟".into(),
            middle_left: String::new(),
            middle_right: String::new(),
            middle: String::new(),
            middle_top: String::new(),
            middle_bottom: String::new(),
        }
    }

    /// Half-block border (inner).
    pub fn inner_half_block() -> Self {
        Self {
            top: "▄".into(),
            bottom: "▀".into(),
            left: "▐".into(),
            right: "▌".into(),
            top_left: "▗".into(),
            top_right: "▖".into(),
            bottom_left: "▝".into(),
            bottom_right: "▘".into(),
            middle_left: String::new(),
            middle_right: String::new(),
            middle: String::new(),
            middle_top: String::new(),
            middle_bottom: String::new(),
        }
    }

    /// Thick border.
    ///
    /// ```text
    /// ┏━━━┓
    /// ┃   ┃
    /// ┗━━━┛
    /// ```
    pub fn thick() -> Self {
        Self {
            top: "━".into(),
            bottom: "━".into(),
            left: "┃".into(),
            right: "┃".into(),
            top_left: "┏".into(),
            top_right: "┓".into(),
            bottom_left: "┗".into(),
            bottom_right: "┛".into(),
            middle_left: "┣".into(),
            middle_right: "┫".into(),
            middle: "╋".into(),
            middle_top: "┳".into(),
            middle_bottom: "┻".into(),
        }
    }

    /// Double-line border.
    ///
    /// ```text
    /// ╔═══╗
    /// ║   ║
    /// ╚═══╝
    /// ```
    pub fn double() -> Self {
        Self {
            top: "═".into(),
            bottom: "═".into(),
            left: "║".into(),
            right: "║".into(),
            top_left: "╔".into(),
            top_right: "╗".into(),
            bottom_left: "╚".into(),
            bottom_right: "╝".into(),
            middle_left: "╠".into(),
            middle_right: "╣".into(),
            middle: "╬".into(),
            middle_top: "╦".into(),
            middle_bottom: "╩".into(),
        }
    }

    /// Hidden border (spaces for layout without visible border).
    pub fn hidden() -> Self {
        Self {
            top: " ".into(),
            bottom: " ".into(),
            left: " ".into(),
            right: " ".into(),
            top_left: " ".into(),
            top_right: " ".into(),
            bottom_left: " ".into(),
            bottom_right: " ".into(),
            middle_left: " ".into(),
            middle_right: " ".into(),
            middle: " ".into(),
            middle_top: " ".into(),
            middle_bottom: " ".into(),
        }
    }

    /// ASCII-only border.
    ///
    /// ```text
    /// +---+
    /// |   |
    /// +---+
    /// ```
    pub fn ascii() -> Self {
        Self {
            top: "-".into(),
            bottom: "-".into(),
            left: "|".into(),
            right: "|".into(),
            top_left: "+".into(),
            top_right: "+".into(),
            bottom_left: "+".into(),
            bottom_right: "+".into(),
            middle_left: "+".into(),
            middle_right: "+".into(),
            middle: "+".into(),
            middle_top: "+".into(),
            middle_bottom: "+".into(),
        }
    }

    /// Markdown table border style.
    pub fn markdown() -> Self {
        Self {
            top: "-".into(),
            bottom: "-".into(),
            left: "|".into(),
            right: "|".into(),
            top_left: "|".into(),
            top_right: "|".into(),
            bottom_left: "|".into(),
            bottom_right: "|".into(),
            middle_left: "|".into(),
            middle_right: "|".into(),
            middle: "|".into(),
            middle_top: "|".into(),
            middle_bottom: "|".into(),
        }
    }

    /// Returns true if this border has no visible characters.
    pub fn is_empty(&self) -> bool {
        self.top.is_empty()
            && self.bottom.is_empty()
            && self.left.is_empty()
            && self.right.is_empty()
            && self.top_left.is_empty()
            && self.top_right.is_empty()
            && self.bottom_left.is_empty()
            && self.bottom_right.is_empty()
    }

    /// Get the width of the top border edge.
    pub fn top_size(&self) -> usize {
        max_rune_width(&self.top_left)
            .max(max_rune_width(&self.top))
            .max(max_rune_width(&self.top_right))
    }

    /// Get the width of the right border edge.
    pub fn right_size(&self) -> usize {
        max_rune_width(&self.top_right)
            .max(max_rune_width(&self.right))
            .max(max_rune_width(&self.bottom_right))
    }

    /// Get the width of the bottom border edge.
    pub fn bottom_size(&self) -> usize {
        max_rune_width(&self.bottom_left)
            .max(max_rune_width(&self.bottom))
            .max(max_rune_width(&self.bottom_right))
    }

    /// Get the width of the left border edge.
    pub fn left_size(&self) -> usize {
        max_rune_width(&self.top_left)
            .max(max_rune_width(&self.left))
            .max(max_rune_width(&self.bottom_left))
    }
}

/// Get the maximum width of any character in a string.
fn max_rune_width(s: &str) -> usize {
    if s.is_empty() {
        return 0;
    }
    s.chars().map(|c| c.to_string().width()).max().unwrap_or(0)
}

/// Which border edges should be rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BorderEdges {
    pub top: bool,
    pub right: bool,
    pub bottom: bool,
    pub left: bool,
}

impl BorderEdges {
    /// All edges enabled.
    pub const fn all() -> Self {
        Self {
            top: true,
            right: true,
            bottom: true,
            left: true,
        }
    }

    /// No edges enabled.
    pub const fn none() -> Self {
        Self {
            top: false,
            right: false,
            bottom: false,
            left: false,
        }
    }

    /// Returns true if any edge is enabled.
    pub const fn any(&self) -> bool {
        self.top || self.right || self.bottom || self.left
    }

    /// Returns true if all edges are enabled.
    pub const fn is_all(&self) -> bool {
        self.top && self.right && self.bottom && self.left
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_border_presets() {
        let normal = Border::normal();
        assert_eq!(normal.top_left, "┌");
        assert_eq!(normal.top, "─");

        let rounded = Border::rounded();
        assert_eq!(rounded.top_left, "╭");

        let ascii = Border::ascii();
        assert_eq!(ascii.top_left, "+");
    }

    #[test]
    fn test_border_sizes() {
        let normal = Border::normal();
        assert_eq!(normal.top_size(), 1);
        assert_eq!(normal.left_size(), 1);

        let empty = Border::none();
        assert_eq!(empty.top_size(), 0);
    }

    #[test]
    fn test_border_edges() {
        let all = BorderEdges::all();
        assert!(all.any());
        assert!(all.is_all());

        let none = BorderEdges::none();
        assert!(!none.any());
    }
}
