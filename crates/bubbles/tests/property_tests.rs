use bubbles::list::{DefaultDelegate, Item, List};
use bubbles::paginator::Paginator;
use bubbles::progress::Progress;
use bubbles::spinner::{SpinnerModel, spinners};
use bubbles::textinput::TextInput;
use bubbles::viewport::Viewport;
use proptest::prelude::*;

/// Simple item type for List tests.
#[derive(Clone, Debug)]
struct TestItem(String);

impl Item for TestItem {
    fn filter_value(&self) -> &str {
        &self.0
    }
}

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

    // =========================================================================
    // TextInput: additional invariants (bd-ygt5)
    // =========================================================================

    #[test]
    fn test_textinput_cursor_bounds_after_set_cursor(
        s in "\\PC{0,50}",
        cursor_positions in prop::collection::vec(0usize..100, 1..10),
    ) {
        let mut input = TextInput::new();
        input.set_value(&s);

        let char_count = input.value().chars().count();

        for pos in cursor_positions {
            input.set_cursor(pos);

            // Invariant: position is always clamped to valid range
            prop_assert!(input.position() <= char_count,
                "Position {} > char_count {} after set_cursor({})",
                input.position(), char_count, pos);
        }
    }

    #[test]
    fn test_textinput_cursor_start_end_idempotent(
        s in "\\PC{0,50}",
    ) {
        let mut input = TextInput::new();
        input.set_value(&s);

        // cursor_start should be idempotent
        input.cursor_start();
        let pos1 = input.position();
        input.cursor_start();
        let pos2 = input.position();
        prop_assert_eq!(pos1, pos2);
        prop_assert_eq!(pos1, 0);

        // cursor_end should be idempotent
        input.cursor_end();
        let pos3 = input.position();
        input.cursor_end();
        let pos4 = input.position();
        prop_assert_eq!(pos3, pos4);
        prop_assert_eq!(pos3, input.value().chars().count());
    }

    #[test]
    fn test_textinput_set_value_respects_char_limit(
        limit in 1usize..20,
        s in "\\PC{0,50}",
    ) {
        let mut input = TextInput::new();
        input.char_limit = limit;
        input.set_value(&s);

        // Value should be truncated to char_limit
        let char_count = input.value().chars().count();
        prop_assert!(char_count <= limit,
            "char_count {} > limit {}", char_count, limit);
    }

    #[test]
    fn test_textinput_focus_blur_state(
        s in "\\PC{0,30}",
    ) {
        let mut input = TextInput::new();
        input.set_value(&s);

        // Initially not focused
        prop_assert!(!input.focused());

        // After focus
        input.focus();
        prop_assert!(input.focused());

        // After blur
        input.blur();
        prop_assert!(!input.focused());
    }

    // =========================================================================
    // Spinner: state invariants (bd-ygt5)
    // =========================================================================

    #[test]
    fn test_spinner_view_non_empty_after_ticks(
        tick_count in 0usize..500,
    ) {
        let spinner = spinners::dot();
        let mut model = SpinnerModel::with_spinner(spinner);

        // Advance frames by calling update() with tick messages
        for _ in 0..tick_count {
            let tick_msg = model.tick();
            let _ = model.update(tick_msg);
        }

        // Invariant: view() always returns a non-empty string regardless of tick count
        let view = model.view();
        prop_assert!(!view.is_empty(), "View should not be empty after {} ticks", tick_count);
    }

    #[test]
    fn test_spinner_various_types_valid(
        spinner_idx in 0usize..10,
        tick_count in 0usize..50,
    ) {
        // Test different spinner types
        let spinner = match spinner_idx % 10 {
            0 => spinners::dot(),
            1 => spinners::line(),
            2 => spinners::pulse(),
            3 => spinners::points(),
            4 => spinners::globe(),
            5 => spinners::moon(),
            6 => spinners::monkey(),
            7 => spinners::meter(),
            8 => spinners::hamburger(),
            _ => spinners::jump(),
        };

        let frame_count = spinner.frames.len();
        prop_assert!(frame_count > 0, "Spinner should have at least one frame");

        let mut model = SpinnerModel::with_spinner(spinner);

        for _ in 0..tick_count {
            let tick_msg = model.tick();
            let _ = model.update(tick_msg);
        }

        let view = model.view();
        prop_assert!(!view.is_empty());
    }

    // =========================================================================
    // List: state invariants (bd-ygt5)
    // =========================================================================

    #[test]
    fn test_list_selection_bounded(
        item_count in 0usize..50,
        select_idx in 0usize..100,
    ) {
        let items: Vec<TestItem> = (0..item_count)
            .map(|i| TestItem(format!("Item {i}")))
            .collect();

        let mut list = List::new(items, DefaultDelegate::new(), 80, 24);

        // Select an arbitrary index
        list.select(select_idx);

        // Invariant: index is always bounded to valid range
        if item_count > 0 {
            prop_assert!(list.index() < item_count,
                "Index {} >= item_count {}", list.index(), item_count);
        }

        // selected_item should return Some if items exist
        if item_count > 0 {
            prop_assert!(list.selected_item().is_some());
        } else {
            prop_assert!(list.selected_item().is_none());
        }
    }

    #[test]
    fn test_list_cursor_movement_bounded(
        item_count in 1usize..50,
        moves in prop::collection::vec(0u8..4, 0..30),
    ) {
        let items: Vec<TestItem> = (0..item_count)
            .map(|i| TestItem(format!("Item {i}")))
            .collect();

        let mut list = List::new(items, DefaultDelegate::new(), 80, 24);

        for mov in moves {
            match mov % 4 {
                0 => { list.cursor_up(); }
                1 => { list.cursor_down(); }
                2 => { list.cursor_up(); list.cursor_up(); } // Multiple moves
                3 => { list.cursor_down(); list.cursor_down(); }
                _ => {}
            }

            // Invariant: index is always valid after movement
            let idx = list.index();
            prop_assert!(idx < item_count,
                "Index {} >= item_count {} after move", idx, item_count);
        }
    }

    #[test]
    fn test_list_filter_preserves_bounds(
        item_count in 1usize..30,
        filter in "[a-z]{0,3}",
    ) {
        let items: Vec<TestItem> = (0..item_count)
            .map(|i| TestItem(format!("item_{i}")))
            .collect();

        let mut list = List::new(items, DefaultDelegate::new(), 80, 24);
        list.set_filter_value(&filter);

        // Invariant: visible items after filter <= total items
        let visible = list.visible_items();
        prop_assert!(visible.len() <= item_count,
            "Filtered items {} > total items {}", visible.len(), item_count);

        // Index should still be valid
        if !visible.is_empty() {
            prop_assert!(list.index() < visible.len(),
                "Index {} >= visible_count {}", list.index(), visible.len());
        }
    }

    // =========================================================================
    // Viewport: additional invariants (bd-ygt5)
    // =========================================================================

    #[test]
    fn test_viewport_top_plus_visible_bounded(
        width in 1usize..100,
        height in 1usize..30,
        line_count in 0usize..100,
        scroll in 0usize..200,
    ) {
        let content = make_content(line_count);
        let mut vp = Viewport::new(width, height);
        vp.set_content(&content);
        vp.scroll_down(scroll);

        // Invariant: viewport_top + visible_lines <= content_lines
        let top = vp.y_offset();
        let visible = vp.visible_line_count();
        let total = vp.total_line_count();

        prop_assert!(top + visible <= total.max(1),
            "top({}) + visible({}) = {} > total({})",
            top, visible, top + visible, total);
    }

    #[test]
    fn test_viewport_horizontal_scroll_bounded(
        width in 1usize..50,
        height in 1usize..20,
        h_step in 1usize..10,
        scroll_ops in prop::collection::vec(0u8..4, 0..20),
    ) {
        let long_content = "This is a very long line that definitely exceeds most viewport widths for testing horizontal scroll";
        let mut vp = Viewport::new(width, height);
        vp.set_horizontal_step(h_step);
        vp.set_content(long_content);

        for op in scroll_ops {
            match op % 4 {
                0 => { vp.scroll_left(h_step); }
                1 => { vp.scroll_right(h_step); }
                2 => { vp.scroll_left(h_step * 2); }
                3 => { vp.scroll_right(h_step * 2); }
                _ => {}
            }

            // View should never panic
            let _view = vp.view();
        }
    }
}
