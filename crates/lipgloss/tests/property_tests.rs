#![allow(clippy::doc_markdown)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::redundant_closure_for_method_calls)]

use lipgloss::{Border, Position, Style, visible_width};
use proptest::prelude::*;

/// Measure lines in rendered output, matching `lipgloss::height()` semantics.
fn output_lines(s: &str) -> usize {
    s.lines().count().max(1)
}

/// Measure the max visible width across all lines.
fn output_width(s: &str) -> usize {
    s.lines().map(visible_width).max().unwrap_or(0)
}

// =============================================================================
// visible_width invariants
// =============================================================================

proptest! {
    #[test]
    fn visible_width_never_panics(s in "\\PC{0,200}") {
        // Should never panic for any printable input
        let _ = visible_width(&s);
    }

    #[test]
    fn visible_width_ascii_equals_len(s in "[a-zA-Z0-9 ]{0,100}") {
        // Pure ASCII without escape sequences: width == byte length
        prop_assert_eq!(visible_width(&s), s.len());
    }

    #[test]
    fn visible_width_ignores_sgr(
        text in "[a-z]{1,20}",
        code in 0u8..108,
    ) {
        let styled = format!("\x1b[{code}m{text}\x1b[0m");
        prop_assert_eq!(visible_width(&styled), text.len());
    }

    #[test]
    fn visible_width_ignores_csi(
        text in "[a-z]{1,20}",
        param in 0u16..100,
        final_byte in prop::sample::select(vec!['A', 'B', 'C', 'D', 'H', 'J', 'K', 'm']),
    ) {
        let csi = format!("\x1b[{param}{final_byte}");
        let input = format!("{csi}{text}");
        prop_assert_eq!(visible_width(&input), text.len());
    }

    #[test]
    fn visible_width_ignores_osc_bel(
        text in "[a-z]{1,20}",
        payload in "[a-z]{0,30}",
    ) {
        let osc = format!("\x1b]{payload}\x07");
        let input = format!("{osc}{text}");
        prop_assert_eq!(visible_width(&input), text.len());
    }

    #[test]
    fn visible_width_ignores_osc_st(
        text in "[a-z]{1,20}",
        payload in "[a-z]{0,30}",
    ) {
        let osc = format!("\x1b]{payload}\x1b\\");
        let input = format!("{osc}{text}");
        prop_assert_eq!(visible_width(&input), text.len());
    }

    #[test]
    fn visible_width_cjk_double_width(
        count in 1usize..20,
    ) {
        // Each CJK character is width 2
        let cjk: String = std::iter::repeat_n('中', count).collect();
        prop_assert_eq!(visible_width(&cjk), count * 2);
    }

    #[test]
    fn visible_width_combining_chars_zero(
        base in "[a-z]",
    ) {
        // Combining acute accent has zero width
        let combined = format!("{base}\u{0301}");
        prop_assert_eq!(visible_width(&combined), 1);
    }
}

// =============================================================================
// Width invariants
// =============================================================================

proptest! {
    #[test]
    fn width_output_matches_target(
        content in "[a-zA-Z0-9 ]{1,30}",
        target_width in 1u16..100,
    ) {
        let content_w = visible_width(&content);
        let style = Style::new().width(target_width);
        let rendered = style.render(&content);

        let rendered_w = output_width(&rendered);

        if content_w <= target_width as usize {
            // When target >= content, output should be exactly target width
            prop_assert_eq!(
                rendered_w, target_width as usize,
                "Expected width {} but got {} for content '{}' (content_w={})",
                target_width, rendered_w, content, content_w
            );
        } else {
            // When content > target, output is at least content width
            // (word wrap may produce different results depending on content)
            prop_assert!(
                rendered_w >= target_width as usize,
                "Expected width >= {} but got {} for content '{}'",
                target_width, rendered_w, content
            );
        }
    }

    #[test]
    fn width_never_less_than_content_single_word(
        content in "[a-zA-Z]{1,15}",
        target_width in 1u16..200,
    ) {
        let style = Style::new().width(target_width);
        let rendered = style.render(&content);
        let rendered_w = output_width(&rendered);
        let content_w = visible_width(&content);

        // Output width should be at least content width (single word can't wrap)
        prop_assert!(
            rendered_w >= content_w,
            "Output width {} < content width {} for '{}'",
            rendered_w, content_w, content
        );
    }
}

// =============================================================================
// Padding invariants
// =============================================================================

proptest! {
    #[test]
    fn padding_horizontal_adds_exactly(
        content in "[a-z]{1,20}",
        pad_left in 0u16..10,
        pad_right in 0u16..10,
    ) {
        let content_w = visible_width(&content);
        let style = Style::new()
            .padding_left(pad_left)
            .padding_right(pad_right);
        let rendered = style.render(&content);
        let rendered_w = output_width(&rendered);

        let expected = content_w + pad_left as usize + pad_right as usize;
        prop_assert_eq!(
            rendered_w, expected,
            "Expected {} + {} + {} = {} but got {}",
            pad_left, content_w, pad_right, expected, rendered_w
        );
    }

    #[test]
    fn padding_vertical_adds_lines(
        content in "[a-z]{1,20}",
        pad_top in 0u16..5,
        pad_bottom in 0u16..5,
    ) {
        let content_lines = output_lines(&content);
        let style = Style::new()
            .padding_top(pad_top)
            .padding_bottom(pad_bottom);
        let rendered = style.render(&content);
        let rendered_lines = output_lines(&rendered);

        let expected = content_lines + pad_top as usize + pad_bottom as usize;
        prop_assert_eq!(
            rendered_lines, expected,
            "Expected {} + {} + {} = {} lines but got {}",
            pad_top, content_lines, pad_bottom, expected, rendered_lines
        );
    }

    #[test]
    fn padding_shorthand_symmetric(
        content in "[a-z]{1,10}",
        v_pad in 0u16..5,
        h_pad in 0u16..10,
    ) {
        let style = Style::new().padding((v_pad, h_pad));
        let rendered = style.render(&content);

        let content_w = visible_width(&content);
        let rendered_w = output_width(&rendered);
        let rendered_h = output_lines(&rendered);

        let expected_w = content_w + 2 * h_pad as usize;
        let expected_h = 1 + 2 * v_pad as usize;

        prop_assert_eq!(rendered_w, expected_w, "Width mismatch");
        prop_assert_eq!(rendered_h, expected_h, "Height mismatch");
    }
}

// =============================================================================
// Margin invariants
// =============================================================================

proptest! {
    #[test]
    fn margin_horizontal_adds_exactly(
        content in "[a-z]{1,20}",
        margin_left in 0u16..10,
        margin_right in 0u16..10,
    ) {
        let content_w = visible_width(&content);
        let style = Style::new()
            .margin_left(margin_left)
            .margin_right(margin_right);
        let rendered = style.render(&content);
        let rendered_w = output_width(&rendered);

        let expected = content_w + margin_left as usize + margin_right as usize;
        prop_assert_eq!(
            rendered_w, expected,
            "Expected margin_l({}) + content({}) + margin_r({}) = {} but got {}",
            margin_left, content_w, margin_right, expected, rendered_w
        );
    }

    #[test]
    fn margin_vertical_adds_lines(
        content in "[a-z]{1,20}",
        margin_top in 0u16..5,
        margin_bottom in 0u16..5,
    ) {
        let content_lines = output_lines(&content);
        let style = Style::new()
            .margin_top(margin_top)
            .margin_bottom(margin_bottom);
        let rendered = style.render(&content);
        let rendered_lines = output_lines(&rendered);

        let expected = content_lines + margin_top as usize + margin_bottom as usize;
        prop_assert_eq!(
            rendered_lines, expected,
            "Expected {} + {} + {} = {} lines but got {}",
            margin_top, content_lines, margin_bottom, expected, rendered_lines
        );
    }
}

// =============================================================================
// Height invariants
// =============================================================================

proptest! {
    #[test]
    fn height_grows_to_target(
        content in "[a-z]{1,20}",
        target_height in 1u16..20,
    ) {
        let content_lines = output_lines(&content);
        let style = Style::new().height(target_height);
        let rendered = style.render(&content);
        let rendered_lines = output_lines(&rendered);

        if content_lines <= target_height as usize {
            prop_assert_eq!(
                rendered_lines, target_height as usize,
                "Expected {} lines but got {} for target {}",
                target_height, rendered_lines, target_height
            );
        } else {
            // When content is taller, height doesn't shrink
            prop_assert_eq!(rendered_lines, content_lines);
        }
    }

    #[test]
    fn height_never_shrinks_content(
        lines in prop::collection::vec("[a-z]{1,10}", 1..10),
        target_height in 1u16..20,
    ) {
        let content = lines.join("\n");
        let content_lines = output_lines(&content);
        let style = Style::new().height(target_height);
        let rendered = style.render(&content);
        let rendered_lines = output_lines(&rendered);

        prop_assert!(
            rendered_lines >= content_lines,
            "Rendered {} lines < content {} lines",
            rendered_lines, content_lines
        );
    }
}

// =============================================================================
// Border invariants
// =============================================================================

fn all_borders() -> Vec<(&'static str, Border)> {
    vec![
        ("normal", Border::normal()),
        ("rounded", Border::rounded()),
        ("thick", Border::thick()),
        ("double", Border::double()),
        ("ascii", Border::ascii()),
        ("hidden", Border::hidden()),
    ]
}

proptest! {
    #[test]
    fn border_adds_two_lines(
        content in "[a-z]{1,20}",
        border_idx in 0usize..6,
    ) {
        let borders = all_borders();
        let (name, border) = &borders[border_idx];
        let content_lines = output_lines(&content);
        let style = Style::new()
            .border(border.clone())
            .border_top(true)
            .border_bottom(true)
            .border_left(true)
            .border_right(true);
        let rendered = style.render(&content);
        let rendered_lines = output_lines(&rendered);

        // Full border adds top + bottom = 2 lines
        prop_assert_eq!(
            rendered_lines, content_lines + 2,
            "Border '{}' should add 2 lines: expected {} but got {}",
            name, content_lines + 2, rendered_lines
        );
    }

    #[test]
    fn border_top_only_adds_one_line(
        content in "[a-z]{1,20}",
        border_idx in 0usize..6,
    ) {
        let borders = all_borders();
        let (name, border) = &borders[border_idx];
        let content_lines = output_lines(&content);
        let style = Style::new()
            .border(border.clone())
            .border_top(true)
            .border_bottom(false)
            .border_left(false)
            .border_right(false);
        let rendered = style.render(&content);
        let rendered_lines = output_lines(&rendered);

        prop_assert_eq!(
            rendered_lines, content_lines + 1,
            "Border '{}' top-only should add 1 line: expected {} but got {}",
            name, content_lines + 1, rendered_lines
        );
    }

    #[test]
    fn border_no_edges_no_change(
        content in "[a-z]{1,20}",
    ) {
        let style = Style::new()
            .border(Border::rounded())
            .border_top(false)
            .border_bottom(false)
            .border_left(false)
            .border_right(false);
        let rendered = style.render(&content);

        // No edges enabled means border is effectively not applied
        prop_assert_eq!(rendered, content);
    }
}

// =============================================================================
// Combined dimension invariants (padding + margin + border)
// =============================================================================

proptest! {
    #[test]
    fn padding_plus_margin_horizontal(
        content in "[a-z]{1,15}",
        pad_left in 0u16..5,
        pad_right in 0u16..5,
        margin_left in 0u16..5,
        margin_right in 0u16..5,
    ) {
        let content_w = visible_width(&content);
        let style = Style::new()
            .padding_left(pad_left)
            .padding_right(pad_right)
            .margin_left(margin_left)
            .margin_right(margin_right);
        let rendered = style.render(&content);
        let rendered_w = output_width(&rendered);

        let expected = content_w
            + pad_left as usize + pad_right as usize
            + margin_left as usize + margin_right as usize;
        prop_assert_eq!(
            rendered_w, expected,
            "Expected total width {} but got {}",
            expected, rendered_w
        );
    }

    #[test]
    fn padding_plus_margin_vertical(
        content in "[a-z]{1,10}",
        pad_top in 0u16..3,
        pad_bottom in 0u16..3,
        margin_top in 0u16..3,
        margin_bottom in 0u16..3,
    ) {
        let content_lines = output_lines(&content);
        let style = Style::new()
            .padding_top(pad_top)
            .padding_bottom(pad_bottom)
            .margin_top(margin_top)
            .margin_bottom(margin_bottom);
        let rendered = style.render(&content);
        let rendered_lines = output_lines(&rendered);

        let expected = content_lines
            + pad_top as usize + pad_bottom as usize
            + margin_top as usize + margin_bottom as usize;
        prop_assert_eq!(
            rendered_lines, expected,
            "Expected total lines {} but got {}",
            expected, rendered_lines
        );
    }
}

// =============================================================================
// Max width / max height invariants
// =============================================================================

proptest! {
    #[test]
    fn max_width_truncates(
        content in "[a-z]{1,50}",
        max_w in 5u16..60,
    ) {
        let style = Style::new().max_width(max_w);
        let rendered = style.render(&content);
        let rendered_w = output_width(&rendered);

        prop_assert!(
            rendered_w <= max_w as usize,
            "max_width({}) but output width is {}",
            max_w, rendered_w
        );
    }

    #[test]
    fn max_height_truncates(
        lines in prop::collection::vec("[a-z]{1,10}", 1..20),
        max_h in 1u16..25,
    ) {
        let content = lines.join("\n");
        let style = Style::new().max_height(max_h);
        let rendered = style.render(&content);
        let rendered_lines = output_lines(&rendered);

        prop_assert!(
            rendered_lines <= max_h as usize,
            "max_height({}) but output has {} lines",
            max_h, rendered_lines
        );
    }
}

// =============================================================================
// Render never panics
// =============================================================================

proptest! {
    #[test]
    fn render_never_panics(
        content in "\\PC{0,50}",
        w in 0u16..100,
        h in 0u16..20,
        pad in 0u16..5,
        margin in 0u16..5,
        use_border in prop::bool::ANY,
        use_bold in prop::bool::ANY,
    ) {
        let mut style = Style::new()
            .width(w)
            .height(h)
            .padding(pad)
            .margin(margin);

        if use_border {
            style = style
                .border(Border::rounded())
                .border_top(true)
                .border_bottom(true)
                .border_left(true)
                .border_right(true);
        }
        if use_bold {
            style = style.bold();
        }

        // Should never panic regardless of input combination
        let _ = style.render(&content);
    }

    #[test]
    fn render_empty_content_never_panics(
        w in 0u16..50,
        h in 0u16..10,
        pad in 0u16..5,
    ) {
        let style = Style::new()
            .width(w)
            .height(h)
            .padding(pad);
        let _ = style.render("");
    }
}

// =============================================================================
// Alignment invariants
// =============================================================================

proptest! {
    #[test]
    fn alignment_preserves_width(
        content in "[a-z]{1,20}",
        target_width in 20u16..60,
        align_idx in 0usize..3,
    ) {
        let positions = [Position::Left, Position::Center, Position::Right];
        let pos = positions[align_idx];
        let style = Style::new()
            .width(target_width)
            .align(pos);
        let rendered = style.render(&content);
        let rendered_w = output_width(&rendered);

        let content_w = visible_width(&content);
        if content_w <= target_width as usize {
            prop_assert_eq!(
                rendered_w, target_width as usize,
                "Alignment should not change target width"
            );
        }
    }
}

// =============================================================================
// place() function invariants
// =============================================================================

proptest! {
    #[test]
    fn place_output_dimensions(
        content in "[a-z]{1,10}",
        place_w in 10usize..50,
        place_h in 3usize..15,
    ) {
        let placed = lipgloss::place(
            place_w, place_h,
            Position::Center, Position::Center,
            &content,
        );
        let out_w = output_width(&placed);
        let out_h = output_lines(&placed);

        let content_w = visible_width(&content);
        let content_h = output_lines(&content);

        // Output dimensions should be at least the requested size
        if content_w <= place_w {
            prop_assert_eq!(out_w, place_w, "place width mismatch");
        }
        if content_h <= place_h {
            prop_assert_eq!(out_h, place_h, "place height mismatch");
        }
    }

    #[test]
    fn place_never_panics(
        content in "\\PC{0,30}",
        w in 0usize..100,
        h in 0usize..30,
    ) {
        let _ = lipgloss::place(w, h, Position::Center, Position::Center, &content);
    }
}

// =============================================================================
// join_horizontal / join_vertical invariants
// =============================================================================

proptest! {
    #[test]
    fn join_horizontal_width_is_sum(
        left in "[a-z]{1,15}",
        right in "[a-z]{1,15}",
    ) {
        let joined = lipgloss::join_horizontal(Position::Top, &[&left, &right]);
        let joined_w = output_width(&joined);
        let expected = visible_width(&left) + visible_width(&right);
        prop_assert_eq!(
            joined_w, expected,
            "Horizontal join: expected width {} but got {}",
            expected, joined_w
        );
    }

    #[test]
    fn join_vertical_height_is_sum(
        top in "[a-z]{1,15}",
        bottom in "[a-z]{1,15}",
    ) {
        let joined = lipgloss::join_vertical(Position::Left, &[&top, &bottom]);
        let joined_h = output_lines(&joined);
        let expected = output_lines(&top) + output_lines(&bottom);
        prop_assert_eq!(
            joined_h, expected,
            "Vertical join: expected {} lines but got {}",
            expected, joined_h
        );
    }

    #[test]
    fn join_vertical_width_is_max(
        top in "[a-z]{1,15}",
        bottom in "[a-z]{1,15}",
    ) {
        let joined = lipgloss::join_vertical(Position::Left, &[&top, &bottom]);
        let joined_w = output_width(&joined);
        let expected = visible_width(&top).max(visible_width(&bottom));
        prop_assert_eq!(
            joined_w, expected,
            "Vertical join: expected max width {} but got {}",
            expected, joined_w
        );
    }

    #[test]
    fn join_horizontal_height_is_max(
        left_lines in prop::collection::vec("[a-z]{1,10}", 1..5),
        right_lines in prop::collection::vec("[a-z]{1,10}", 1..5),
    ) {
        let left = left_lines.join("\n");
        let right = right_lines.join("\n");
        let joined = lipgloss::join_horizontal(Position::Top, &[&left, &right]);
        let joined_h = output_lines(&joined);
        let expected = output_lines(&left).max(output_lines(&right));
        prop_assert_eq!(
            joined_h, expected,
            "Horizontal join: expected max height {} but got {}",
            expected, joined_h
        );
    }
}
