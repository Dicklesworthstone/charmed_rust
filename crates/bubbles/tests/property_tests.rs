use bubbles::paginator::Paginator;
use bubbles::progress::Progress;
use bubbles::textinput::TextInput;
use proptest::prelude::*;

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
}
