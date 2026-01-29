use bubbles::paginator::Paginator;
use bubbles::progress::Progress;
use bubbles::textinput::TextInput;
use bubbles::viewport::Viewport;
use proptest::prelude::*;

fn make_content(line_count: usize) -> String {
    (0..line_count)
        .map(|i| format!("Line {i}"))
        .collect::<Vec<_>>()
        .join("\n")
}

proptest! {
    #[test]
    fn test_paginator_invariants(
        total_pages in 1usize..1000,
        per_page in 1usize..100,
        page in 0usize..2000, // deliberately larger than total_pages
        item_count in 0usize..10000
    ) {
        let mut p = Paginator::new()
            .total_pages(total_pages)
            .per_page(per_page);

        // Invariant: page should be clamped to valid range [0, total_pages - 1]
        p.set_page(page);
        if total_pages > 0 {
            prop_assert!(p.page() < total_pages);
        }

        // Invariant: slice bounds should be valid for item count
        // We set total pages from items to ensure consistent state for slice calc
        p.set_total_pages_from_items(item_count);
        let (start, end) = p.get_slice_bounds(item_count);

        prop_assert!(start <= end);
        prop_assert!(end <= item_count);

        // Items on page should match slice difference
        let count = p.items_on_page(item_count);
        prop_assert_eq!(count, end - start);
    }

    #[test]
    fn test_progress_invariants(
        percent in -2.0f64..2.0f64,
        width in 1usize..200
    ) {
        let mut p = Progress::new().width(width);
        p.set_percent(percent);

        // Invariant: percent is clamped between 0.0 and 1.0
        prop_assert!(p.percent() >= 0.0);
        prop_assert!(p.percent() <= 1.0);

        // View generation should not panic and return non-empty string
        let view = p.view();
        prop_assert!(!view.is_empty());

        // Incremental updates should respect bounds
        p.incr_percent(0.1);
        prop_assert!(p.percent() <= 1.0);

        p.decr_percent(0.1);
        prop_assert!(p.percent() >= 0.0);
    }

    #[test]
    fn test_textinput_invariants(
        s in "\\PC*", // printable chars
        cursor_pos in 0usize..100,
        width in 0usize..50,
        char_limit in 0usize..50
    ) {
        let mut input = TextInput::new();
        input.width = width;
        input.char_limit = char_limit;
        input.set_value(&s);

        // Invariant: value length respect char_limit (if > 0)
        let char_count = input.value().chars().count();
        if char_limit > 0 {
            prop_assert!(char_count <= char_limit);
        }

        // Invariant: cursor position is always <= value length
        input.set_cursor(cursor_pos);
        prop_assert!(input.position() <= char_count);

        // View generation should not panic
        let view = input.view();
        prop_assert!(!view.is_empty()); // Should at least contain prompt

        // Cursor movement invariants
        input.cursor_start();
        prop_assert_eq!(input.position(), 0);

        input.cursor_end();
        prop_assert_eq!(input.position(), char_count);
    }

    // =========================================================================
    // Viewport invariants
    // =========================================================================

    #[test]
    fn test_viewport_scroll_bounds(
        width in 1usize..200,
        height in 1usize..50,
        line_count in 0usize..200,
        scroll_amount in 0usize..300,
    ) {
        let content = make_content(line_count);
        let mut vp = Viewport::new(width, height);
        vp.set_content(&content);

        // Scroll down arbitrary amount
        vp.scroll_down(scroll_amount);

        // Invariant: y_offset never exceeds max scroll
        let max_scroll = line_count.saturating_sub(height);
        prop_assert!(vp.y_offset() <= max_scroll,
            "y_offset {} > max_scroll {} (lines={}, height={})",
            vp.y_offset(), max_scroll, line_count, height);

        // Scroll up arbitrary amount
        vp.scroll_up(scroll_amount);
        prop_assert!(vp.y_offset() <= max_scroll);
    }

    #[test]
    fn test_viewport_at_top_bottom_consistency(
        width in 1usize..100,
        height in 1usize..30,
        line_count in 0usize..100,
    ) {
        let content = make_content(line_count);
        let mut vp = Viewport::new(width, height);
        vp.set_content(&content);

        // At top initially
        prop_assert!(vp.at_top());

        vp.goto_bottom();
        if line_count > height {
            prop_assert!(vp.at_bottom());
            prop_assert!(!vp.at_top());
        }

        vp.goto_top();
        prop_assert!(vp.at_top());
        prop_assert_eq!(vp.y_offset(), 0);
    }

    #[test]
    fn test_viewport_scroll_percent_range(
        width in 1usize..100,
        height in 1usize..30,
        line_count in 0usize..100,
        scroll in 0usize..200,
    ) {
        let content = make_content(line_count);
        let mut vp = Viewport::new(width, height);
        vp.set_content(&content);
        vp.scroll_down(scroll);

        let pct = vp.scroll_percent();
        prop_assert!((0.0..=1.0).contains(&pct),
            "scroll_percent {} out of range", pct);
    }

    #[test]
    fn test_viewport_page_down_up_roundtrip(
        width in 1usize..100,
        height in 1usize..30,
        line_count in 0usize..200,
    ) {
        let content = make_content(line_count);
        let mut vp = Viewport::new(width, height);
        vp.set_content(&content);

        // Page down then page up should return to same or close position
        let initial = vp.y_offset();
        vp.page_down();
        vp.page_up();

        // Should be back at initial (or 0 if content fits in viewport)
        if line_count <= height {
            prop_assert_eq!(vp.y_offset(), 0);
        } else {
            prop_assert_eq!(vp.y_offset(), initial);
        }
    }

    #[test]
    fn test_viewport_view_never_panics(
        width in 1usize..100,
        height in 1usize..30,
        line_count in 0usize..100,
        scroll in 0usize..200,
    ) {
        let content = make_content(line_count);
        let mut vp = Viewport::new(width, height);
        vp.set_content(&content);
        vp.scroll_down(scroll);
        let _view = vp.view();
    }

    #[test]
    fn test_viewport_visible_lines_bounded(
        width in 1usize..100,
        height in 1usize..30,
        line_count in 0usize..100,
    ) {
        let content = make_content(line_count);
        let mut vp = Viewport::new(width, height);
        vp.set_content(&content);

        prop_assert!(vp.visible_line_count() <= height);
        prop_assert!(vp.visible_line_count() <= vp.total_line_count());
    }

    // =========================================================================
    // Progress: extreme values
    // =========================================================================

    #[test]
    fn test_progress_extreme_values(
        percent in prop::num::f64::ANY,
        width in 1usize..200,
    ) {
        let mut p = Progress::new().width(width);
        p.set_percent(percent);

        // Should always clamp to [0, 1] even for NaN/Inf
        prop_assert!(p.percent() >= 0.0);
        prop_assert!(p.percent() <= 1.0);
        prop_assert!(p.percent().is_finite());

        // View should never panic
        let _view = p.view();
    }

    // =========================================================================
    // Paginator: navigation sequence
    // =========================================================================

    #[test]
    fn test_paginator_next_prev_bounded(
        total in 1usize..100,
        steps in 0usize..200,
    ) {
        let mut p = Paginator::new().total_pages(total);

        for _ in 0..steps {
            p.next_page();
        }
        prop_assert!(p.page() < total);

        for _ in 0..steps {
            p.prev_page();
        }
        prop_assert_eq!(p.page(), 0);
    }
}
