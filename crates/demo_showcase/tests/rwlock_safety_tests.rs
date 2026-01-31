//! Unit Tests: `RwLock` Poison Safety (bd-xnq2)
//!
//! These tests verify that the `demo_showcase` pages are safe from `RwLock` poisoning.
//! Since we use `parking_lot::RwLock` (which never poisons), these tests confirm
//! that behavior and ensure pages survive panic scenarios.
//!
//! # Test Categories
//!
//! ## `parking_lot` Never Poisons
//! - Verify that `parking_lot::RwLock` remains accessible after a panic while holding lock
//!
//! ## Page Survival After Panic
//! - Verify that pages using `RwLock` continue to work after `catch_unwind`
//!
//! ## Comparison with `std::sync::RwLock`
//! - Demonstrate that `std::sync::RwLock` DOES poison (for documentation)

use std::panic::{AssertUnwindSafe, catch_unwind};

// =============================================================================
// TEST: parking_lot::RwLock Never Poisons
// =============================================================================

/// Verify that `parking_lot::RwLock` does not poison when a panic occurs
/// while holding a write lock.
///
/// This is the core property we rely on for SSH session safety.
#[test]
fn test_parking_lot_never_poisons_on_write_panic() {
    use parking_lot::RwLock;

    let lock = RwLock::new(42);

    // Cause a panic while holding the write lock
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let _guard = lock.write();
        panic!("intentional panic to test poisoning");
    }));

    // With parking_lot, the lock is NOT poisoned - read should work
    assert_eq!(
        *lock.read(),
        42,
        "parking_lot lock should be readable after panic"
    );
}

/// Verify that `parking_lot::RwLock` allows writes after a panic.
#[test]
fn test_parking_lot_write_after_panic() {
    use parking_lot::RwLock;

    let lock = RwLock::new(0);

    // Panic while holding write lock
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let _guard = lock.write();
        panic!("intentional panic");
    }));

    // Writing should still work
    {
        let mut guard = lock.write();
        *guard = 100;
    }

    // Verify the write succeeded
    assert_eq!(*lock.read(), 100);
}

/// Verify multiple panic-recovery cycles don't corrupt data.
#[test]
fn test_parking_lot_multiple_panic_cycles() {
    use parking_lot::RwLock;

    let lock = RwLock::new(0_i32);

    for i in 1..=5 {
        // Increment the value
        {
            let mut guard = lock.write();
            *guard += 1;
        }

        // Panic while holding the lock
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = lock.write();
            panic!("panic cycle {i}");
        }));

        // Verify value is still correct
        assert_eq!(
            *lock.read(),
            i,
            "value should persist after panic cycle {i}"
        );
    }
}

// =============================================================================
// TEST: std::sync::RwLock DOES Poison (Documentation/Comparison)
// =============================================================================

/// Demonstrate that `std::sync::RwLock` DOES poison on panic.
/// This test documents WHY we use `parking_lot` instead.
#[test]
fn test_std_rwlock_does_poison() {
    use std::sync::RwLock;

    let lock = RwLock::new(42);

    // Panic while holding write lock
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let _guard = lock.write().unwrap();
        panic!("intentional panic");
    }));

    // `std::sync::RwLock` IS poisoned - read returns Err.
    //
    // The error is a PoisonError. We can recover by calling `into_inner()`, but this is
    // exactly what we want to avoid having to do throughout the codebase.
    let err = lock
        .read()
        .expect_err("std::sync::RwLock should be poisoned after panic");
    assert_eq!(
        *err.into_inner(),
        42,
        "poison recovery should still have the data"
    );
}

// =============================================================================
// TEST: Page Components with RwLock
// =============================================================================

/// Test that a `Viewport` wrapped in `parking_lot::RwLock` survives panics.
/// This simulates the pattern used in `LogsPage`, `DocsPage`, `FilesPage`.
#[test]
fn test_viewport_rwlock_survives_panic() {
    use bubbles::viewport::Viewport;
    use parking_lot::RwLock;

    // Create a viewport similar to how pages do
    let viewport = RwLock::new(Viewport::new(80, 24));

    // Set some content
    viewport.write().set_content("Line 1\nLine 2\nLine 3");

    // Simulate a panic during an update operation
    let _ = catch_unwind(AssertUnwindSafe(|| {
        viewport.write().set_content("panic content");
        panic!("simulated panic during update");
    }));

    // The viewport should still be accessible.
    // Note: The content may be "panic content" (if the write completed before panic)
    // or "Line 1\nLine 2\nLine 3" (if panic happened before write).
    // The key point is that we CAN read it - no poison.
    let rendered = viewport.read().view();
    assert!(
        rendered.contains("panic content") || rendered.contains("Line 1"),
        "expected viewport to be readable after panic"
    );
}

/// Test concurrent access pattern: multiple readers, one writer with panic.
#[test]
fn test_concurrent_readers_survive_writer_panic() {
    use parking_lot::RwLock;
    use std::sync::Arc;
    use std::thread;

    let lock = Arc::new(RwLock::new(vec![1, 2, 3]));
    let lock2 = Arc::clone(&lock);

    // Spawn a writer that will panic
    let writer = thread::spawn(move || {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            lock2.write().push(4);
            panic!("writer panic");
        }));
    });

    // Wait for writer to finish (panic or not)
    let _ = writer.join();

    // Readers should still work
    let reader1 = {
        let lock_clone = Arc::clone(&lock);
        thread::spawn(move || {
            let guard = lock_clone.read();
            guard.len()
        })
    };

    let reader2 = {
        let lock_clone = Arc::clone(&lock);
        thread::spawn(move || {
            let guard = lock_clone.read();
            guard.iter().sum::<i32>()
        })
    };

    // Both readers should succeed
    let len = reader1.join().expect("reader1 should complete");
    let sum = reader2.join().expect("reader2 should complete");

    // The data may have 3 or 4 elements depending on when the panic occurred
    assert!((3..=4).contains(&len), "length should be 3 or 4, got {len}");
    assert!(sum >= 6, "sum should be at least 6, got {sum}");
}

// =============================================================================
// TEST: Integration - Page Model Pattern
// =============================================================================

/// Simulate the `LogsPage` pattern: `RwLock<Viewport>` + `RwLock<String>` for cached content.
/// Verify the page can continue rendering after a panic in `update()`.
#[test]
fn test_page_pattern_survives_update_panic() {
    use bubbles::viewport::Viewport;
    use parking_lot::RwLock;

    // Simulate LogsPage state
    struct MockPage {
        viewport: RwLock<Viewport>,
        formatted_content: RwLock<String>,
        needs_reformat: RwLock<bool>,
    }

    let page = MockPage {
        viewport: RwLock::new(Viewport::new(80, 24)),
        formatted_content: RwLock::new(String::from("initial content")),
        needs_reformat: RwLock::new(false),
    };

    // Simulate update() that panics while holding locks
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let new_content = String::from("new content");
        *page.formatted_content.write() = new_content.clone();
        *page.needs_reformat.write() = true;
        page.viewport.write().set_content(&new_content);
        panic!("panic in update()");
    }));

    // Simulate view() - should be able to read all state
    let viewport_rendered = page.viewport.read().view();
    let formatted = page.formatted_content.read().clone();
    let needs_reformat = *page.needs_reformat.read();

    assert_eq!(formatted, "new content");
    assert!(needs_reformat);
    assert!(viewport_rendered.contains("new content"));
}

/// Verify that a page can recover and process new updates after a panic.
#[test]
fn test_page_continues_after_panic() {
    use parking_lot::RwLock;

    struct Counter {
        value: RwLock<i32>,
    }

    let counter = Counter {
        value: RwLock::new(0),
    };

    // First update succeeds
    *counter.value.write() += 1;
    assert_eq!(*counter.value.read(), 1);

    // Second update panics mid-operation
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let mut v = counter.value.write();
        *v += 1;
        drop(v);
        panic!("panic after increment");
    }));

    // Value may be 1 or 2 depending on panic timing
    let after_panic = *counter.value.read();
    assert!((1..=2).contains(&after_panic));

    // Third update should work fine
    *counter.value.write() += 10;
    let final_value = *counter.value.read();

    // Should be either 11 or 12
    assert!(
        (11..=12).contains(&final_value),
        "counter should continue working, got {final_value}"
    );
}

// =============================================================================
// TEST: Performance Characteristics
// =============================================================================

/// Verify that `parking_lot` has no performance degradation after panic recovery.
#[test]
fn test_no_performance_degradation_after_panic() {
    use parking_lot::RwLock;
    use std::time::Instant;

    let lock = RwLock::new(0_i32);

    // Measure time before any panics
    let before_panic_start = Instant::now();
    for _ in 0..1000 {
        *lock.write() += 1;
    }
    let before_panic_duration = before_panic_start.elapsed();

    // Cause several panics
    for _ in 0..10 {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = lock.write();
            panic!("intentional");
        }));
    }

    // Measure time after panics
    let after_panic_start = Instant::now();
    for _ in 0..1000 {
        *lock.write() += 1;
    }
    let after_panic_duration = after_panic_start.elapsed();

    // After-panic operations should not be significantly slower
    // (Allow 3x variance for test stability)
    assert!(
        after_panic_duration < before_panic_duration * 3,
        "performance should not degrade significantly after panic: before={before_panic_duration:?}, after={after_panic_duration:?}"
    );
}
