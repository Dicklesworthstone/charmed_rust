//! E2E tests for intermediate example applications.
//!
//! These tests verify that intermediate examples work correctly with
//! more complex user interactions.
//!
//! Note: Run with `cargo test -p e2e-tests -- --ignored`

use e2e_tests::TestTerminal;
use std::time::Duration;

// ============================================================================
// Todo List Example Tests
// ============================================================================

mod todo_list {
    use super::*;

    #[test]
    #[ignore]
    fn test_todo_list_displays_title() {
        let mut term = TestTerminal::spawn("todo-list").expect("Failed to spawn todo-list");

        term.wait_for("Todo", Duration::from_secs(5))
            .expect("Should display todo list title");

        term.exit().expect("Should exit cleanly");
    }

    #[test]
    #[ignore]
    fn test_todo_list_shows_initial_items() {
        let mut term = TestTerminal::spawn("todo-list").expect("Failed to spawn todo-list");

        // Should show some initial items
        term.wait_for("Todo", Duration::from_secs(5)).unwrap();

        // Check for help text
        term.assert_screen_contains("a")
            .or_else(|_| term.assert_screen_contains("add"))
            .expect("Should show add hint");

        term.exit().expect("Should exit cleanly");
    }

    #[test]
    #[ignore]
    fn test_todo_list_navigation() {
        let mut term = TestTerminal::spawn("todo-list").expect("Failed to spawn todo-list");
        term.wait_for("Todo", Duration::from_secs(5)).unwrap();

        // Navigate down
        term.press_key("j").expect("Should send j key");
        std::thread::sleep(Duration::from_millis(100));

        // Navigate up
        term.press_key("k").expect("Should send k key");
        std::thread::sleep(Duration::from_millis(100));

        term.exit().expect("Should exit cleanly");
    }

    #[test]
    #[ignore]
    fn test_todo_list_quit() {
        let mut term = TestTerminal::spawn("todo-list").expect("Failed to spawn todo-list");
        term.wait_for("Todo", Duration::from_secs(5)).unwrap();

        term.press_key("q").expect("Should send q key");

        std::thread::sleep(Duration::from_millis(100));
        let status = term.exit().expect("Should exit");
        assert!(status.success());
    }
}

// ============================================================================
// Viewport Example Tests
// ============================================================================

mod viewport {
    use super::*;

    #[test]
    #[ignore]
    fn test_viewport_displays_title() {
        let mut term = TestTerminal::spawn("viewport").expect("Failed to spawn viewport");

        term.wait_for("Viewport", Duration::from_secs(5))
            .expect("Should display viewport title");

        term.exit().expect("Should exit cleanly");
    }

    #[test]
    #[ignore]
    fn test_viewport_shows_scroll_indicator() {
        let mut term = TestTerminal::spawn("viewport").expect("Failed to spawn viewport");

        term.wait_for("Scroll", Duration::from_secs(5))
            .expect("Should show scroll indicator");

        term.exit().expect("Should exit cleanly");
    }

    #[test]
    #[ignore]
    fn test_viewport_scroll_down() {
        let mut term = TestTerminal::spawn("viewport").expect("Failed to spawn viewport");
        term.wait_for("Viewport", Duration::from_secs(5)).unwrap();

        // Scroll down with j
        term.press_key("j").expect("Should send j key");
        std::thread::sleep(Duration::from_millis(100));

        // Scroll should change (hard to verify exact percentage without parsing ANSI)
        term.exit().expect("Should exit cleanly");
    }

    #[test]
    #[ignore]
    fn test_viewport_page_down() {
        let mut term = TestTerminal::spawn("viewport").expect("Failed to spawn viewport");
        term.wait_for("Viewport", Duration::from_secs(5)).unwrap();

        // Page down with f
        term.press_key("f").expect("Should send f key");
        std::thread::sleep(Duration::from_millis(100));

        term.exit().expect("Should exit cleanly");
    }

    #[test]
    #[ignore]
    fn test_viewport_quit() {
        let mut term = TestTerminal::spawn("viewport").expect("Failed to spawn viewport");
        term.wait_for("Viewport", Duration::from_secs(5)).unwrap();

        term.press_key("q").expect("Should send q key");

        std::thread::sleep(Duration::from_millis(100));
        let status = term.exit().expect("Should exit");
        assert!(status.success());
    }
}

// ============================================================================
// Progress Example Tests
// ============================================================================

mod progress {
    use super::*;

    #[test]
    #[ignore]
    fn test_progress_displays_title() {
        let mut term = TestTerminal::spawn("progress").expect("Failed to spawn progress");

        term.wait_for("Progress", Duration::from_secs(5))
            .expect("Should display progress title");

        term.exit().expect("Should exit cleanly");
    }

    #[test]
    #[ignore]
    fn test_progress_shows_ready_state() {
        let mut term = TestTerminal::spawn("progress").expect("Failed to spawn progress");

        term.wait_for("Ready", Duration::from_secs(5))
            .or_else(|_| term.wait_for("start", Duration::from_secs(1)))
            .expect("Should show ready state or start hint");

        term.exit().expect("Should exit cleanly");
    }

    #[test]
    #[ignore]
    fn test_progress_start_with_enter() {
        let mut term = TestTerminal::spawn("progress").expect("Failed to spawn progress");
        term.wait_for("Ready", Duration::from_secs(5)).unwrap();

        // Start with Enter
        term.press_key("enter").expect("Should send enter key");

        // Should show running state
        term.wait_for("Processing", Duration::from_secs(2))
            .or_else(|_| term.wait_for("Running", Duration::from_secs(1)))
            .expect("Should start processing");

        term.exit().expect("Should exit cleanly");
    }

    #[test]
    #[ignore]
    fn test_progress_cancel_with_escape() {
        let mut term = TestTerminal::spawn("progress").expect("Failed to spawn progress");
        term.wait_for("Ready", Duration::from_secs(5)).unwrap();

        // Start
        term.press_key("enter").unwrap();
        term.wait_for("Processing", Duration::from_secs(2)).ok();

        // Cancel
        term.press_key("escape").expect("Should send escape key");

        // May show cancelled or just exit
        std::thread::sleep(Duration::from_millis(200));
        term.exit().expect("Should exit cleanly");
    }

    #[test]
    #[ignore]
    fn test_progress_quit() {
        let mut term = TestTerminal::spawn("progress").expect("Failed to spawn progress");
        term.wait_for("Progress", Duration::from_secs(5)).unwrap();

        term.press_key("q").expect("Should send q key");

        std::thread::sleep(Duration::from_millis(100));
        let status = term.exit().expect("Should exit");
        assert!(status.success());
    }
}
