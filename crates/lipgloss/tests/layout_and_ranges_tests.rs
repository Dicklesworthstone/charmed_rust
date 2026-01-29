//! Tests for layout functions (join_horizontal, join_vertical, place)
//! and text range styling (style_ranges, style_runes).
//! Also covers border preset correctness.

#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_wrap)]

use lipgloss::{
    join_horizontal, join_vertical, new_range, place, style_ranges, style_runes, visible_width,
    Border, Position, Style,
};

// =============================================================================
// join_horizontal
// =============================================================================

#[test]
fn join_horizontal_empty_input() {
    assert_eq!(join_horizontal(Position::Top, &[]), "");
}

#[test]
fn join_horizontal_single_item() {
    assert_eq!(join_horizontal(Position::Top, &["hello"]), "hello");
}

#[test]
fn join_horizontal_single_multiline() {
    let input = "line1\nline2";
    assert_eq!(join_horizontal(Position::Top, &[input]), input);
}

#[test]
fn join_horizontal_two_single_lines() {
    let result = join_horizontal(Position::Top, &["AB", "CD"]);
    assert_eq!(result, "ABCD");
}

#[test]
fn join_horizontal_top_alignment() {
    let left = "A\nB\nC";
    let right = "X";
    let result = join_horizontal(Position::Top, &[left, right]);
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "AX");
    // Right block is padded with spaces on rows 2 and 3
    assert_eq!(lines[1], "B ");
    assert_eq!(lines[2], "C ");
}

#[test]
fn join_horizontal_bottom_alignment() {
    let left = "A\nB\nC";
    let right = "X";
    let result = join_horizontal(Position::Bottom, &[left, right]);
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "A ");
    assert_eq!(lines[1], "B ");
    assert_eq!(lines[2], "CX");
}

#[test]
fn join_horizontal_center_alignment() {
    let left = "A\nB\nC";
    let right = "X";
    let result = join_horizontal(Position::Center, &[left, right]);
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 3);
    // Center alignment: offset = round(2 * 0.5) = 1
    assert_eq!(lines[0], "A ");
    assert_eq!(lines[1], "BX");
    assert_eq!(lines[2], "C ");
}

#[test]
fn join_horizontal_different_widths() {
    let narrow = "A";
    let wide = "WXYZ";
    let result = join_horizontal(Position::Top, &[narrow, wide]);
    assert_eq!(result, "AWXYZ");
}

#[test]
fn join_horizontal_three_blocks() {
    let a = "A\nB";
    let b = "1\n2";
    let c = "X\nY";
    let result = join_horizontal(Position::Top, &[a, b, c]);
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines[0], "A1X");
    assert_eq!(lines[1], "B2Y");
}

#[test]
fn join_horizontal_preserves_width_padding() {
    // When blocks have unequal line widths, shorter lines get padded
    let left = "AB\nA";
    let right = "X";
    let result = join_horizontal(Position::Top, &[left, right]);
    let lines: Vec<&str> = result.lines().collect();
    // Left block width = 2, so "A" gets padded to 2
    assert_eq!(lines[0], "ABX");
    assert_eq!(lines[1], "A  ");
}

// =============================================================================
// join_vertical
// =============================================================================

#[test]
fn join_vertical_empty_input() {
    assert_eq!(join_vertical(Position::Left, &[]), "");
}

#[test]
fn join_vertical_single_item() {
    assert_eq!(join_vertical(Position::Left, &["hello"]), "hello");
}

#[test]
fn join_vertical_left_alignment() {
    let result = join_vertical(Position::Left, &["Short", "A longer line"]);
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(visible_width(lines[0]), visible_width(lines[1]));
    assert!(lines[0].starts_with("Short"));
}

#[test]
fn join_vertical_right_alignment() {
    let result = join_vertical(Position::Right, &["Short", "A longer line"]);
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 2);
    // "Short" should be right-aligned (padded on left)
    assert!(lines[0].ends_with("Short"));
    let leading_spaces = lines[0].len() - lines[0].trim_start().len();
    assert!(leading_spaces > 0);
}

#[test]
fn join_vertical_center_alignment() {
    let result = join_vertical(Position::Center, &["Hi", "A longer line"]);
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 2);
    // "Hi" should be roughly centered within the width of "A longer line"
    let leading = lines[0].len() - lines[0].trim_start().len();
    let trailing = lines[0].len() - lines[0].trim_end().len();
    // Centering means leading ≈ trailing (within 1)
    assert!((leading as i64 - trailing as i64).unsigned_abs() <= 1);
}

#[test]
fn join_vertical_multiline_blocks() {
    let top = "A\nB";
    let bottom = "CCC";
    let result = join_vertical(Position::Left, &[top, bottom]);
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 3);
    // All lines should be padded to width 3
    assert_eq!(lines[0], "A  ");
    assert_eq!(lines[1], "B  ");
    assert_eq!(lines[2], "CCC");
}

#[test]
fn join_vertical_equal_width() {
    let result = join_vertical(Position::Left, &["ABC", "DEF"]);
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines[0], "ABC");
    assert_eq!(lines[1], "DEF");
}

// =============================================================================
// place
// =============================================================================

#[test]
fn place_center_center() {
    let result = place(10, 5, Position::Center, Position::Center, "Hi");
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 5);
    // Content should be roughly in the middle row
    // 5 rows, content = 1 row => top_pad = floor((4) * 0.5) = 2
    // So row index 2 should contain "Hi"
    assert!(lines[2].contains("Hi"));
    // Each line should be width 10
    for line in &lines {
        assert_eq!(visible_width(line), 10);
    }
}

#[test]
fn place_top_left() {
    let result = place(10, 3, Position::Left, Position::Top, "Hi");
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with("Hi"));
    assert_eq!(visible_width(lines[0]), 10);
}

#[test]
fn place_bottom_right() {
    let result = place(10, 3, Position::Right, Position::Bottom, "Hi");
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[2].ends_with("Hi"));
}

#[test]
fn place_content_fills_space() {
    // Content exactly matches width/height
    let result = place(5, 1, Position::Left, Position::Top, "Hello");
    assert_eq!(result, "Hello");
}

#[test]
fn place_multiline_content() {
    let content = "AB\nCD";
    let result = place(6, 4, Position::Center, Position::Center, content);
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 4);
    for line in &lines {
        assert_eq!(visible_width(line), 6);
    }
}

#[test]
fn place_content_wider_than_width() {
    // Content wider than container should not panic
    let result = place(3, 1, Position::Left, Position::Top, "Hello");
    // Content doesn't get truncated, just no padding
    assert!(result.contains("Hello"));
}

// =============================================================================
// style_ranges
// =============================================================================

#[test]
fn style_ranges_empty_ranges() {
    let result = style_ranges("Hello, World!", &[]);
    assert_eq!(result, "Hello, World!");
}

#[test]
fn style_ranges_single_range() {
    let bold = Style::new().bold();
    let result = style_ranges("Hello, World!", &[new_range(0, 5, bold)]);
    // The styled portion should contain ANSI codes
    assert!(result.contains("Hello"));
    assert!(result.contains("World"));
    // Result should have ANSI escape sequences for the bold part
    assert!(result.contains('\x1b'));
}

#[test]
fn style_ranges_multiple_ranges() {
    let bold = Style::new().bold();
    let italic = Style::new().italic();
    let result = style_ranges(
        "Hello, World!",
        &[new_range(0, 5, bold), new_range(7, 12, italic)],
    );
    assert!(result.contains('\x1b'));
    // Unstyled comma+space should be present as-is
    assert!(result.contains(", "));
}

#[test]
fn style_ranges_non_overlapping_with_gap() {
    let s = "ABCDEFGHIJ";
    let bold = Style::new().bold();
    let result = style_ranges(s, &[new_range(0, 3, bold.clone()), new_range(7, 10, bold)]);
    // Middle "DEFG" should be unstyled
    // visible_width should still equal 10
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, "ABCDEFGHIJ");
}

#[test]
fn style_ranges_out_of_bounds() {
    let bold = Style::new().bold();
    // End beyond string length - should clamp
    let result = style_ranges("Hi", &[new_range(0, 100, bold)]);
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, "Hi");
}

#[test]
fn style_ranges_unsorted_input() {
    let bold = Style::new().bold();
    let italic = Style::new().italic();
    // Ranges given in reverse order - should still work (sorted internally)
    let result = style_ranges(
        "ABCDEF",
        &[new_range(3, 6, italic), new_range(0, 3, bold)],
    );
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, "ABCDEF");
}

// =============================================================================
// style_runes
// =============================================================================

#[test]
fn style_runes_basic() {
    let matched = Style::new().bold();
    let unmatched = Style::new().faint();
    let result = style_runes("Hello", &[0, 1, 2], matched, unmatched);
    // Each character gets individually styled
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, "Hello");
}

#[test]
fn style_runes_empty_indices() {
    let matched = Style::new().bold();
    let unmatched = Style::new().faint();
    let result = style_runes("Hello", &[], matched, unmatched);
    // All characters get unmatched style
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, "Hello");
}

#[test]
fn style_runes_all_indices() {
    let matched = Style::new().bold();
    let unmatched = Style::new().faint();
    let result = style_runes("Hi", &[0, 1], matched, unmatched);
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, "Hi");
}

#[test]
fn style_runes_out_of_bounds_indices() {
    let matched = Style::new().bold();
    let unmatched = Style::new().faint();
    // Index 99 is beyond string length - should be ignored
    let result = style_runes("Hi", &[0, 99], matched, unmatched);
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, "Hi");
}

// =============================================================================
// Border presets
// =============================================================================

#[test]
fn border_normal_characters() {
    let b = Border::normal();
    assert_eq!(b.top_left, "┌");
    assert_eq!(b.top_right, "┐");
    assert_eq!(b.bottom_left, "└");
    assert_eq!(b.bottom_right, "┘");
    assert_eq!(b.top, "─");
    assert_eq!(b.bottom, "─");
    assert_eq!(b.left, "│");
    assert_eq!(b.right, "│");
}

#[test]
fn border_rounded_characters() {
    let b = Border::rounded();
    assert_eq!(b.top_left, "╭");
    assert_eq!(b.top_right, "╮");
    assert_eq!(b.bottom_left, "╰");
    assert_eq!(b.bottom_right, "╯");
}

#[test]
fn border_thick_characters() {
    let b = Border::thick();
    assert_eq!(b.top_left, "┏");
    assert_eq!(b.top_right, "┓");
    assert_eq!(b.bottom_left, "┗");
    assert_eq!(b.bottom_right, "┛");
    assert_eq!(b.top, "━");
    assert_eq!(b.left, "┃");
}

#[test]
fn border_double_characters() {
    let b = Border::double();
    assert_eq!(b.top_left, "╔");
    assert_eq!(b.top_right, "╗");
    assert_eq!(b.bottom_left, "╚");
    assert_eq!(b.bottom_right, "╝");
    assert_eq!(b.top, "═");
    assert_eq!(b.left, "║");
}

#[test]
fn border_ascii_characters() {
    let b = Border::ascii();
    assert_eq!(b.top_left, "+");
    assert_eq!(b.top_right, "+");
    assert_eq!(b.bottom_left, "+");
    assert_eq!(b.bottom_right, "+");
    assert_eq!(b.top, "-");
    assert_eq!(b.left, "|");
}

#[test]
fn border_block_all_same() {
    let b = Border::block();
    assert_eq!(b.top, "█");
    assert_eq!(b.bottom, "█");
    assert_eq!(b.left, "█");
    assert_eq!(b.right, "█");
    assert_eq!(b.top_left, "█");
    assert_eq!(b.top_right, "█");
    assert_eq!(b.bottom_left, "█");
    assert_eq!(b.bottom_right, "█");
}

#[test]
fn border_hidden_all_spaces() {
    let b = Border::hidden();
    assert_eq!(b.top, " ");
    assert_eq!(b.bottom, " ");
    assert_eq!(b.left, " ");
    assert_eq!(b.right, " ");
    assert_eq!(b.top_left, " ");
}

#[test]
fn border_none_is_empty() {
    let b = Border::none();
    assert!(b.is_empty());
}

#[test]
fn border_normal_is_not_empty() {
    assert!(!Border::normal().is_empty());
}

#[test]
fn border_hidden_is_not_empty() {
    // Hidden has space chars, not empty strings
    assert!(!Border::hidden().is_empty());
}

#[test]
fn border_table_connectors() {
    let b = Border::normal();
    assert_eq!(b.middle_left, "├");
    assert_eq!(b.middle_right, "┤");
    assert_eq!(b.middle, "┼");
    assert_eq!(b.middle_top, "┬");
    assert_eq!(b.middle_bottom, "┴");
}

#[test]
fn border_markdown_preset() {
    let b = Border::markdown();
    assert_eq!(b.top, "-");
    assert_eq!(b.left, "|");
    assert_eq!(b.top_left, "|");
    assert_eq!(b.middle, "|");
}

#[test]
fn border_size_methods() {
    let b = Border::normal();
    assert_eq!(b.top_size(), 1);
    assert_eq!(b.right_size(), 1);
    assert_eq!(b.bottom_size(), 1);
    assert_eq!(b.left_size(), 1);
}

#[test]
fn border_none_sizes_zero() {
    let b = Border::none();
    assert_eq!(b.top_size(), 0);
    assert_eq!(b.right_size(), 0);
    assert_eq!(b.bottom_size(), 0);
    assert_eq!(b.left_size(), 0);
}

// =============================================================================
// Border rendering through Style
// =============================================================================

#[test]
fn style_with_border_renders_box() {
    let s = Style::new().border(Border::ascii());
    let result = s.render("Hi");
    // Should contain border characters
    assert!(result.contains('+'));
    assert!(result.contains('-'));
    assert!(result.contains('|'));
    assert!(result.contains("Hi"));
}

#[test]
fn style_with_rounded_border() {
    let s = Style::new().border(Border::rounded());
    let result = s.render("X");
    assert!(result.contains('╭'));
    assert!(result.contains('╯'));
    assert!(result.contains('X'));
}

// =============================================================================
// Helpers
// =============================================================================

/// Strip ANSI escape sequences from a string for content verification.
fn strip_ansi(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if let Some(&next) = chars.peek()
                && next == '['
            {
                // CSI sequence - skip until letter
                chars.next();
                while let Some(&nc) = chars.peek() {
                    chars.next();
                    if nc.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        result.push(c);
    }
    result
}
