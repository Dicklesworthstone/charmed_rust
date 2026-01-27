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

pub mod backend;
pub mod border;
pub mod color;
pub mod position;
pub mod renderer;
pub mod style;
pub mod theme;

#[cfg(feature = "wasm")]
pub mod wasm;

// Re-exports
pub use backend::{
    AnsiBackend, DefaultBackend, HtmlBackend, OutputBackend, PlainBackend, default_backend,
};
pub use border::{Border, BorderEdges};
pub use color::{
    AdaptiveColor, AnsiColor, Color, ColorProfile, CompleteAdaptiveColor, CompleteColor, NoColor,
    RgbColor, TerminalColor,
};
pub use position::{Position, Sides};
pub use renderer::{Renderer, color_profile, default_renderer, has_dark_background};
pub use style::Style;
#[cfg(feature = "tokio")]
pub use theme::AsyncThemeContext;
pub use theme::{
    CachedThemedStyle, CatppuccinFlavor, ColorSlot, ColorTransform, ListenerId, Theme,
    ThemeChangeListener, ThemeColors, ThemeContext, ThemePreset, ThemeRole, ThemedColor,
    ThemedStyle, global_theme, set_global_preset, set_global_theme,
};

// WASM bindings (only available with the "wasm" feature)
#[cfg(feature = "wasm")]
pub use wasm::{
    JsColor, JsStyle, join_horizontal as wasm_join_horizontal, join_vertical as wasm_join_vertical,
    new_style as wasm_new_style, place as wasm_place,
};

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::backend::{
        AnsiBackend, DefaultBackend, HtmlBackend, OutputBackend, PlainBackend,
    };
    pub use crate::border::Border;
    pub use crate::color::{AdaptiveColor, Color, ColorProfile, NoColor};
    pub use crate::position::{Position, Sides};
    pub use crate::renderer::Renderer;
    pub use crate::style::Style;
    #[cfg(feature = "tokio")]
    pub use crate::theme::AsyncThemeContext;
    pub use crate::theme::{
        CachedThemedStyle, CatppuccinFlavor, ColorSlot, ColorTransform, ListenerId, Theme,
        ThemeChangeListener, ThemeColors, ThemeContext, ThemePreset, ThemeRole, ThemedColor,
        ThemedStyle, global_theme, set_global_preset, set_global_theme,
    };
    #[cfg(feature = "wasm")]
    pub use crate::wasm::{JsColor, JsStyle};
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

    // Pre-compute alignment factor once
    let factor = pos.factor();

    // Pre-compute vertical offsets for each block (avoid per-row calculation)
    let offsets: Vec<usize> = blocks
        .iter()
        .map(|block| {
            let extra = max_height.saturating_sub(block.len());
            (extra as f64 * factor).round() as usize
        })
        .collect();

    // Estimate total capacity: sum of widths * max_height + newlines
    let total_width: usize = widths.iter().sum();
    let estimated_capacity = max_height * (total_width + 1);
    let mut result = String::with_capacity(estimated_capacity);

    // Build result directly without intermediate Vec<String>
    for row in 0..max_height {
        if row > 0 {
            result.push('\n');
        }

        for (block_idx, block) in blocks.iter().enumerate() {
            let block_height = block.len();
            let width = widths[block_idx];
            let top_offset = offsets[block_idx];

            // Determine which line from this block to use
            let content = row
                .checked_sub(top_offset)
                .filter(|&br| br < block_height)
                .map_or("", |br| block[br]);

            // Pad to block width (avoid " ".repeat() allocation)
            let content_width = visible_width(content);
            let padding = width.saturating_sub(content_width);
            result.push_str(content);
            for _ in 0..padding {
                result.push(' ');
            }
        }
    }

    result
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

    // Pre-compute alignment factor once
    let factor = pos.factor();
    let is_right_aligned = factor >= 1.0;

    // Count total lines for capacity estimation (newlines + 1 per string, avoiding double iteration)
    let line_count: usize = strs
        .iter()
        .map(|s| s.bytes().filter(|&b| b == b'\n').count() + 1)
        .sum();
    let estimated_capacity = line_count * (max_width + 1);
    let mut result = String::with_capacity(estimated_capacity);

    // Pad each line to max width based on position - single pass, no Vec<String>
    let mut first = true;
    for s in strs {
        for line in s.lines() {
            if !first {
                result.push('\n');
            }
            first = false;

            let line_width = visible_width(line);
            let extra = max_width.saturating_sub(line_width);
            let left_pad = (extra as f64 * factor).round() as usize;
            let right_pad = extra.saturating_sub(left_pad);

            // Add left padding (avoid " ".repeat() allocation)
            for _ in 0..left_pad {
                result.push(' ');
            }
            result.push_str(line);

            // Add right padding only if not right-aligned
            if !is_right_aligned {
                for _ in 0..right_pad {
                    result.push(' ');
                }
            }
        }
    }

    result
}

/// Calculate the visible width of a string (excluding ANSI escapes).
fn visible_width(s: &str) -> usize {
    let mut width = 0;
    #[derive(Clone, Copy, PartialEq)]
    enum State {
        Normal,
        Esc,
        Csi,
        Osc,
    }
    let mut state = State::Normal;

    for c in s.chars() {
        match state {
            State::Normal => {
                if c == '\x1b' {
                    state = State::Esc;
                } else {
                    width += unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
                }
            }
            State::Esc => {
                if c == '[' {
                    state = State::Csi;
                } else if c == ']' {
                    state = State::Osc;
                } else {
                    // Handle simple escapes like \x1b7 (save cursor) or \x1b> (keypad)
                    // They are single char after ESC.
                    state = State::Normal;
                }
            }
            State::Csi => {
                // CSI sequence: [params] [intermediate] final
                // Final byte is 0x40-0x7E (@ to ~)
                if ('@'..='~').contains(&c) {
                    state = State::Normal;
                }
            }
            State::Osc => {
                // OSC sequence: ] [params] ; [text] BEL/ST
                // Handle BEL (\x07)
                if c == '\x07' {
                    state = State::Normal;
                } else if c == '\x1b' {
                    // Handle ST (ESC \) - we see ESC, transition to Esc to handle the backslash
                    state = State::Esc;
                }
            }
        }
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

    // Pre-compute alignment factor once for content lines
    let h_factor = h_pos.factor();

    // Pre-allocate blank line once for reuse (avoids allocation per blank line)
    let blank_line = " ".repeat(width);

    // Pre-allocate result with estimated capacity: height lines * (width + newline)
    let estimated_capacity = height * (width + 1);
    let mut result = String::with_capacity(estimated_capacity);

    // Top padding - reuse blank_line
    for i in 0..top_pad {
        if i > 0 {
            result.push('\n');
        }
        result.push_str(&blank_line);
    }

    // Content with horizontal padding - single-pass, avoid format!
    for (i, line) in s.lines().enumerate() {
        if top_pad > 0 || i > 0 {
            result.push('\n');
        }

        let line_width = visible_width(line);
        let line_extra = width.saturating_sub(line_width);
        let line_left = (line_extra as f64 * h_factor).floor() as usize;
        let line_right = line_extra.saturating_sub(line_left);

        // Use slices of blank_line for padding (no allocation)
        result.push_str(&blank_line[..line_left]);
        result.push_str(line);
        result.push_str(&blank_line[..line_right]);
    }

    // Bottom padding - reuse blank_line
    for _ in 0..bottom_pad {
        result.push('\n');
        result.push_str(&blank_line);
    }

    result
}

// =============================================================================
// StyleRanges and Range
// =============================================================================

/// Range specifies a section of text with a start index, end index, and the Style to apply.
///
/// Used with [`style_ranges`] to apply different styles to different parts of a string.
///
/// # Example
///
/// ```rust
/// use lipgloss::{Range, Style, style_ranges};
///
/// let style = Style::new().bold();
/// let range = Range {
///     start: 0,
///     end: 5,
///     style,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct Range {
    /// The starting index (inclusive, in bytes).
    pub start: usize,
    /// The ending index (exclusive, in bytes).
    pub end: usize,
    /// The Style to apply to this range.
    pub style: Style,
}

impl Range {
    /// Creates a new Range.
    pub fn new(start: usize, end: usize, style: Style) -> Self {
        Self { start, end, style }
    }
}

/// Creates a new Range that can be used with [`style_ranges`].
///
/// # Arguments
///
/// * `start` - The starting index of the range (inclusive, in bytes)
/// * `end` - The ending index of the range (exclusive, in bytes)
/// * `style` - The Style to apply to this range
///
/// # Example
///
/// ```rust
/// use lipgloss::{new_range, Style, style_ranges};
///
/// let styled = style_ranges(
///     "Hello, World!",
///     &[
///         new_range(0, 5, Style::new().bold()),
///         new_range(7, 12, Style::new().italic()),
///     ],
/// );
/// ```
pub fn new_range(start: usize, end: usize, style: Style) -> Range {
    Range::new(start, end, style)
}

/// Applies styles to ranges in a string. Existing ANSI styles will be taken into account.
/// Ranges should not overlap.
///
/// # Arguments
///
/// * `s` - The input string to style
/// * `ranges` - A slice of Range objects specifying which parts of the string to style
///
/// # Returns
///
/// The styled string with each range having its specified style applied.
///
/// # Example
///
/// ```rust
/// use lipgloss::{style_ranges, new_range, Style};
///
/// let styled = style_ranges(
///     "Hello, World!",
///     &[
///         new_range(0, 5, Style::new().bold()),
///         new_range(7, 12, Style::new().italic()),
///     ],
/// );
/// ```
pub fn style_ranges(s: &str, ranges: &[Range]) -> String {
    if ranges.is_empty() {
        return s.to_string();
    }

    // Sort ranges by start position
    let mut sorted_ranges: Vec<_> = ranges.iter().collect();
    sorted_ranges.sort_by_key(|r| r.start);

    let bytes = s.as_bytes();
    let mut result = String::new();
    let mut current_pos = 0;

    for range in sorted_ranges {
        let start = range.start.min(bytes.len());
        let end = range.end.min(bytes.len());

        if start > current_pos {
            // Add unstyled text between ranges
            if let Ok(text) = std::str::from_utf8(&bytes[current_pos..start]) {
                result.push_str(text);
            }
        }

        if end > start {
            // Apply style to this range
            if let Ok(text) = std::str::from_utf8(&bytes[start..end]) {
                result.push_str(&range.style.render(text));
            }
        }

        current_pos = end.max(current_pos);
    }

    // Add remaining text after last range
    if current_pos < bytes.len() {
        if let Ok(text) = std::str::from_utf8(&bytes[current_pos..]) {
            result.push_str(text);
        }
    }

    result
}

/// Applies styles to runes at the given indices in the string.
///
/// You must provide styling options for both matched and unmatched runes.
/// Indices out of bounds will be ignored.
///
/// # Arguments
///
/// * `s` - The input string to style
/// * `indices` - Array of character indices indicating which runes to style
/// * `matched` - The Style to apply to runes at the specified indices
/// * `unmatched` - The Style to apply to all other runes
///
/// # Example
///
/// ```rust
/// use lipgloss::{style_runes, Style};
///
/// let styled = style_runes(
///     "Hello",
///     &[0, 1, 2],
///     Style::new().bold(),
///     Style::new().faint(),
/// );
/// ```
pub fn style_runes(s: &str, indices: &[usize], matched: Style, unmatched: Style) -> String {
    use std::collections::HashSet;
    let indices_set: HashSet<_> = indices.iter().copied().collect();

    let mut result = String::new();

    for (i, c) in s.chars().enumerate() {
        let char_str = c.to_string();
        if indices_set.contains(&i) {
            result.push_str(&matched.render(&char_str));
        } else {
            result.push_str(&unmatched.render(&char_str));
        }
    }

    result
}
