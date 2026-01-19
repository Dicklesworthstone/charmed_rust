#![forbid(unsafe_code)]
// Allow these clippy lints for API ergonomics and terminal UI code
#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::use_self)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::enum_glob_use)]
#![allow(clippy::match_like_matches_macro)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::single_match_else)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::new_without_default)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::missing_fields_in_debug)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::manual_repeat_n)]
#![allow(clippy::if_not_else)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::same_item_push)]
#![allow(clippy::bool_to_int_with_if)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::branches_sharing_code)]
#![allow(clippy::items_after_test_module)]

//! # Lipgloss
//!
//! A powerful terminal styling library for creating beautiful CLI applications.
//!
//! Lipgloss provides a declarative, CSS-like approach to terminal styling with support for:
//! - **Colors**: ANSI, 256-color, true color, and adaptive colors
//! - **Text formatting**: Bold, italic, underline, strikethrough, and more
//! - **Layout**: Padding, margins, borders, and alignment
//! - **Word wrapping** and **text truncation**
//!
//! ## Quick Start
//!
//! ```rust
//! use lipgloss::{Style, Border, Position};
//!
//! // Create a styled box
//! let style = Style::new()
//!     .bold()
//!     .foreground("#ff00ff")
//!     .background("#1a1a1a")
//!     .padding((1, 2))
//!     .border(Border::rounded())
//!     .align(Position::Center);
//!
//! println!("{}", style.render("Hello, Lipgloss!"));
//! ```
//!
//! ## Style Builder
//!
//! Styles are built using a fluent API where each method returns a new style:
//!
//! ```rust
//! use lipgloss::Style;
//!
//! let base = Style::new().bold();
//! let red = base.clone().foreground("#ff0000");
//! let blue = base.clone().foreground("#0000ff");
//! ```
//!
//! ## Colors
//!
//! Multiple color formats are supported:
//!
//! ```rust
//! use lipgloss::{Style, AdaptiveColor, Color};
//!
//! // Hex colors
//! let style = Style::new().foreground("#ff00ff");
//!
//! // ANSI 256 colors
//! let style = Style::new().foreground("196");
//!
//! // Adaptive colors (light/dark themes)
//! let adaptive = AdaptiveColor {
//!     light: Color::from("#000000"),
//!     dark: Color::from("#ffffff"),
//! };
//! let style = Style::new().foreground_color(adaptive);
//! ```
//!
//! ## Borders
//!
//! Several preset borders are available:
//!
//! ```rust
//! use lipgloss::{Style, Border};
//!
//! let style = Style::new()
//!     .border(Border::rounded())
//!     .padding(1);
//!
//! // Available borders:
//! // Border::normal()    ┌───┐
//! // Border::rounded()   ╭───╮
//! // Border::thick()     ┏━━━┓
//! // Border::double()    ╔═══╗
//! // Border::hidden()    (spaces)
//! // Border::ascii()     +---+
//! ```
//!
//! ## Layout
//!
//! CSS-like padding and margin with shorthand notation:
//!
//! ```rust
//! use lipgloss::Style;
//!
//! // All sides
//! let style = Style::new().padding(2);
//!
//! // Vertical, horizontal
//! let style = Style::new().padding((1, 2));
//!
//! // Top, horizontal, bottom
//! let style = Style::new().padding((1, 2, 3));
//!
//! // Top, right, bottom, left (clockwise)
//! let style = Style::new().padding((1, 2, 3, 4));
//! ```

pub mod border;
pub mod color;
pub mod position;
pub mod renderer;
pub mod style;
pub mod theme;

// Re-exports
pub use border::{Border, BorderEdges};
pub use color::{
    AdaptiveColor, AnsiColor, Color, ColorProfile, CompleteAdaptiveColor, CompleteColor, NoColor,
    RgbColor, TerminalColor,
};
pub use position::{Position, Sides};
pub use renderer::{Renderer, color_profile, default_renderer, has_dark_background};
pub use style::Style;
pub use theme::{Theme, ThemeColors};

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::border::Border;
    pub use crate::color::{AdaptiveColor, Color, ColorProfile, NoColor};
    pub use crate::position::{Position, Sides};
    pub use crate::renderer::Renderer;
    pub use crate::style::Style;
    pub use crate::theme::{Theme, ThemeColors};
}

// Convenience constructors

/// Create a new empty style.
///
/// This is equivalent to `Style::new()`.
pub fn new_style() -> Style {
    Style::new()
}

// Join utilities

/// Horizontally joins multi-line strings along a vertical axis.
///
/// The `pos` parameter controls vertical alignment of blocks:
/// - `Position::Top` (0.0): Align to top
/// - `Position::Center` (0.5): Center vertically
/// - `Position::Bottom` (1.0): Align to bottom
///
/// # Example
///
/// ```rust
/// use lipgloss::{join_horizontal, Position};
///
/// let left = "Line 1\nLine 2\nLine 3";
/// let right = "A\nB";
/// let combined = join_horizontal(Position::Top, &[left, right]);
/// ```
pub fn join_horizontal(pos: Position, strs: &[&str]) -> String {
    if strs.is_empty() {
        return String::new();
    }
    if strs.len() == 1 {
        return strs[0].to_string();
    }

    // Split each string into lines and calculate dimensions
    let blocks: Vec<Vec<&str>> = strs.iter().map(|s| s.lines().collect()).collect();
    let widths: Vec<usize> = blocks
        .iter()
        .map(|lines| lines.iter().map(|l| visible_width(l)).max().unwrap_or(0))
        .collect();
    let max_height = blocks.iter().map(|lines| lines.len()).max().unwrap_or(0);

    // Build result lines
    let mut result = Vec::with_capacity(max_height);
    for row in 0..max_height {
        let mut line = String::new();
        for (block_idx, block) in blocks.iter().enumerate() {
            let block_height = block.len();
            let width = widths[block_idx];

            // Calculate vertical offset based on alignment
            let extra_rows = max_height.saturating_sub(block_height);
            let top_offset = (extra_rows as f64 * pos.factor()).round() as usize;

            // Determine which line from this block to use
            let block_row = row.checked_sub(top_offset);
            let content = if let Some(br) = block_row {
                if br < block_height { block[br] } else { "" }
            } else {
                ""
            };

            // Pad to block width
            let content_width = visible_width(content);
            let padding = width.saturating_sub(content_width);
            line.push_str(content);
            line.push_str(&" ".repeat(padding));
        }
        // Preserve trailing spaces to maintain column alignment (like Go)
        result.push(line);
    }

    result.join("\n")
}

/// Vertically joins multi-line strings along a horizontal axis.
///
/// The `pos` parameter controls horizontal alignment:
/// - `Position::Left` (0.0): Align to left
/// - `Position::Center` (0.5): Center horizontally
/// - `Position::Right` (1.0): Align to right
///
/// # Example
///
/// ```rust
/// use lipgloss::{join_vertical, Position};
///
/// let top = "Short";
/// let bottom = "A longer line";
/// let combined = join_vertical(Position::Center, &[top, bottom]);
/// ```
pub fn join_vertical(pos: Position, strs: &[&str]) -> String {
    if strs.is_empty() {
        return String::new();
    }
    if strs.len() == 1 {
        return strs[0].to_string();
    }

    // Find the maximum width across all lines
    let max_width = strs
        .iter()
        .flat_map(|s| s.lines())
        .map(|l| visible_width(l))
        .max()
        .unwrap_or(0);

    // Pad each line to max width based on position
    let mut result = Vec::new();
    for s in strs {
        for line in s.lines() {
            let line_width = visible_width(line);
            let extra = max_width.saturating_sub(line_width);
            let left_pad = (extra as f64 * pos.factor()).round() as usize;
            let right_pad = extra.saturating_sub(left_pad);

            // For left alignment (pos=0), keep trailing spaces to maintain width
            // For right alignment (pos=1), we can trim trailing (no right_pad)
            // For center alignment, keep trailing spaces to maintain width
            let padded = format!("{}{}{}", " ".repeat(left_pad), line, " ".repeat(right_pad));
            // Only trim trailing for right alignment where right_pad would be 0
            if pos.factor() >= 1.0 {
                result.push(padded.trim_end().to_string());
            } else {
                result.push(padded);
            }
        }
    }

    result.join("\n")
}

/// Calculate the visible width of a string (excluding ANSI escapes).
fn visible_width(s: &str) -> usize {
    let mut width = 0;
    let mut in_escape = false;

    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
            continue;
        }
        if in_escape {
            if c == 'm' {
                in_escape = false;
            }
            continue;
        }
        width += unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
    }

    width
}

/// Get the width of the widest line in a string.
pub fn width(s: &str) -> usize {
    s.lines().map(|l| visible_width(l)).max().unwrap_or(0)
}

/// Get the number of lines in a string.
pub fn height(s: &str) -> usize {
    s.lines().count().max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_vertical_left_alignment() {
        let result = join_vertical(Position::Left, &["Short", "LongerText"]);
        println!("Result bytes: {:?}", result.as_bytes());
        println!("Result repr: {:?}", result);
        // Expected: "Short     \nLongerText" (Short with 5 trailing spaces)
        assert_eq!(result, "Short     \nLongerText");
    }

    #[test]
    fn test_join_vertical_center_alignment() {
        let result = join_vertical(Position::Center, &["Short", "LongerText"]);
        println!("Result bytes: {:?}", result.as_bytes());
        println!("Result repr: {:?}", result);
        // Expected: "   Short  \nLongerText" (3 left, 2 right for center)
        // Note: Go uses int truncation which gives left=2, but fixture shows left=3
        // This suggests Go might use rounding, let's check both
        let expected_go = "   Short  \nLongerText"; // 3 left, 2 right
        assert_eq!(result, expected_go);
    }
}

/// Place a string at a position within a given width and height.
///
/// # Example
///
/// ```rust
/// use lipgloss::{place, Position};
///
/// let text = "Hello";
/// let placed = place(20, 5, Position::Center, Position::Center, text);
/// ```
pub fn place(width: usize, height: usize, h_pos: Position, v_pos: Position, s: &str) -> String {
    let content_width = self::width(s);
    let content_height = self::height(s);

    // Horizontal padding - use floor for Go compatibility
    let h_extra = width.saturating_sub(content_width);
    let left_pad = (h_extra as f64 * h_pos.factor()).floor() as usize;
    let _right_pad = h_extra.saturating_sub(left_pad);

    // Vertical padding - use floor for Go compatibility
    let v_extra = height.saturating_sub(content_height);
    let top_pad = (v_extra as f64 * v_pos.factor()).floor() as usize;
    let bottom_pad = v_extra.saturating_sub(top_pad);

    let mut result = Vec::with_capacity(height);

    // Top padding
    for _ in 0..top_pad {
        result.push(" ".repeat(width));
    }

    // Content with horizontal padding
    for line in s.lines() {
        let line_width = visible_width(line);
        let line_extra = width.saturating_sub(line_width);
        let line_left = (line_extra as f64 * h_pos.factor()).floor() as usize;
        let line_right = line_extra.saturating_sub(line_left);
        result.push(format!(
            "{}{}{}",
            " ".repeat(line_left),
            line,
            " ".repeat(line_right)
        ));
    }

    // Bottom padding
    for _ in 0..bottom_pad {
        result.push(" ".repeat(width));
    }

    result.join("\n")
}
