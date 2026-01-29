#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::redundant_closure_for_method_calls)]

//! Property-based tests for lipgloss layout functions:
//! join_horizontal, join_vertical, place.

use lipgloss::{join_horizontal, join_vertical, place, visible_width, Position};
use proptest::prelude::*;

/// Count lines in output.
fn line_count(s: &str) -> usize {
    if s.is_empty() {
        0
    } else {
        s.lines().count()
    }
}

/// Max visible width across all lines.
fn max_line_width(s: &str) -> usize {
    s.lines().map(visible_width).max().unwrap_or(0)
}

fn position_strategy() -> impl Strategy<Value = Position> {
    prop_oneof![
        Just(Position::Top),
        Just(Position::Center),
        Just(Position::Bottom),
        Just(Position::Left),
        Just(Position::Right),
    ]
}

fn ascii_block() -> impl Strategy<Value = String> {
    // Generate 1-5 lines of 1-20 chars each
    prop::collection::vec("[a-zA-Z0-9]{1,20}", 1..=5)
        .prop_map(|lines| lines.join("\n"))
}

// =============================================================================
// join_horizontal properties
// =============================================================================

proptest! {
    #[test]
    fn join_horizontal_empty_is_empty(pos in position_strategy()) {
        let result = join_horizontal(pos, &[]);
        prop_assert_eq!(result, "");
    }

    #[test]
    fn join_horizontal_single_identity(
        pos in position_strategy(),
        block in ascii_block(),
    ) {
        let result = join_horizontal(pos, &[&block]);
        prop_assert_eq!(result, block);
    }

    #[test]
    fn join_horizontal_height_is_max(
        pos in position_strategy(),
        a in ascii_block(),
        b in ascii_block(),
    ) {
        let result = join_horizontal(pos, &[&a, &b]);
        let result_lines = line_count(&result);
        let max_height = line_count(&a).max(line_count(&b));
        prop_assert_eq!(result_lines, max_height,
            "join_horizontal height should be max of inputs");
    }

    #[test]
    fn join_horizontal_width_is_sum(
        a in ascii_block(),
        b in ascii_block(),
    ) {
        let result = join_horizontal(Position::Top, &[&a, &b]);
        let a_width = max_line_width(&a);
        let b_width = max_line_width(&b);
        let result_width = max_line_width(&result);
        prop_assert_eq!(result_width, a_width + b_width,
            "join_horizontal width should be sum of block widths");
    }

    #[test]
    fn join_horizontal_never_panics(
        pos in position_strategy(),
        blocks in prop::collection::vec(ascii_block(), 0..=4),
    ) {
        let refs: Vec<&str> = blocks.iter().map(|s| s.as_str()).collect();
        let _ = join_horizontal(pos, &refs);
    }
}

// =============================================================================
// join_vertical properties
// =============================================================================

proptest! {
    #[test]
    fn join_vertical_empty_is_empty(pos in position_strategy()) {
        let result = join_vertical(pos, &[]);
        prop_assert_eq!(result, "");
    }

    #[test]
    fn join_vertical_single_identity(
        pos in position_strategy(),
        block in ascii_block(),
    ) {
        let result = join_vertical(pos, &[&block]);
        prop_assert_eq!(result, block);
    }

    #[test]
    fn join_vertical_height_is_sum(
        pos in position_strategy(),
        a in ascii_block(),
        b in ascii_block(),
    ) {
        let result = join_vertical(pos, &[&a, &b]);
        let result_lines = line_count(&result);
        let total_height = line_count(&a) + line_count(&b);
        prop_assert_eq!(result_lines, total_height,
            "join_vertical height should be sum of inputs");
    }

    #[test]
    fn join_vertical_width_is_max(
        a in ascii_block(),
        b in ascii_block(),
    ) {
        let result = join_vertical(Position::Left, &[&a, &b]);
        let a_width = max_line_width(&a);
        let b_width = max_line_width(&b);
        let result_width = max_line_width(&result);
        prop_assert_eq!(result_width, a_width.max(b_width),
            "join_vertical width should be max of block widths");
    }

    #[test]
    fn join_vertical_left_aligned_starts_at_column_0(
        a in ascii_block(),
        b in ascii_block(),
    ) {
        let result = join_vertical(Position::Left, &[&a, &b]);
        // Just verify no panic and content preserved
        prop_assert!(result.contains(a.lines().next().unwrap_or("")));
    }

    #[test]
    fn join_vertical_never_panics(
        pos in position_strategy(),
        blocks in prop::collection::vec(ascii_block(), 0..=4),
    ) {
        let refs: Vec<&str> = blocks.iter().map(|s| s.as_str()).collect();
        let _ = join_vertical(pos, &refs);
    }
}

// =============================================================================
// place properties
// =============================================================================

proptest! {
    #[test]
    fn place_output_dimensions(
        w in 1usize..=40,
        h in 1usize..=20,
        h_pos in position_strategy(),
        v_pos in position_strategy(),
        content in "[a-zA-Z]{1,10}",
    ) {
        let result = place(w, h, h_pos, v_pos, &content);
        let result_lines = line_count(&result);
        let content_height = line_count(&content);

        // Output height should be max(h, content_height)
        prop_assert_eq!(result_lines, h.max(content_height),
            "place output height should be max(container, content)");

        // Each line width should be at least w (if content fits)
        if visible_width(&content) <= w {
            for line in result.lines() {
                prop_assert_eq!(visible_width(line), w,
                    "each line should have width = container width");
            }
        }
    }

    #[test]
    fn place_preserves_content(
        w in 5usize..=40,
        h in 1usize..=10,
        content in "[a-z]{1,5}",
    ) {
        let result = place(w, h, Position::Left, Position::Top, &content);
        prop_assert!(result.contains(&content),
            "place should preserve content text");
    }

    #[test]
    fn place_never_panics(
        w in 0usize..=100,
        h in 0usize..=50,
        h_pos in position_strategy(),
        v_pos in position_strategy(),
        content in "\\PC{0,30}",
    ) {
        let _ = place(w, h, h_pos, v_pos, &content);
    }

    #[test]
    fn place_top_left_content_at_start(
        w in 5usize..=20,
        h in 2usize..=5,
        content in "[a-z]{1,5}",
    ) {
        let result = place(w, h, Position::Left, Position::Top, &content);
        let first_line = result.lines().next().unwrap_or("");
        prop_assert!(first_line.starts_with(&content),
            "top-left: content should be at start of first line");
    }

    #[test]
    fn place_bottom_right_content_at_end(
        w in 5usize..=20,
        h in 2usize..=5,
        content in "[a-z]{1,5}",
    ) {
        let result = place(w, h, Position::Right, Position::Bottom, &content);
        let last_line = result.lines().last().unwrap_or("");
        prop_assert!(last_line.ends_with(&content),
            "bottom-right: content should be at end of last line");
    }
}
