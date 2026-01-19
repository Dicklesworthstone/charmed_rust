//! E2E tests for basic example applications.
//!
//! These tests verify that basic examples work correctly when run as
//! actual processes with simulated user input.
//!
//! Note: These tests spawn real processes and may be slower than unit tests.
//! They are marked #[ignore] by default for CI. Run with:
//! `cargo test -p e2e-tests -- --ignored`

use e2e_tests::TestTerminal;
use std::time::Duration;

// ============================================================================
// Counter Example Tests
// ============================================================================

mod counter {
    use super::*;

    #[test]
    #[ignore] // Requires TTY, run with --ignored
    fn test_counter_displays_initial_state() {
        let mut term = TestTerminal::spawn("counter").expect("Failed to spawn counter");

        // Wait for initial render
        term.wait_for("Count:", Duration::from_secs(5))
            .expect("Should display counter");

        let status = term.exit().expect("Should exit cleanly");
        assert!(status.success(), "Counter should exit successfully");
    }

    #[test]
    #[ignore]
    fn test_counter_increment() {
        let mut term = TestTerminal::spawn("counter").expect("Failed to spawn counter");

        // Wait for initial state
        term.wait_for("0", Duration::from_secs(5))
            .expect("Should show initial count");

        // Increment
        term.press_key("+").expect("Should send + key");
        term.wait_for("1", Duration::from_secs(2))
            .expect("Count should increment to 1");

        // Increment again
        term.press_key("+").expect("Should send + key");
        term.wait_for("2", Duration::from_secs(2))
            .expect("Count should increment to 2");

        term.exit().expect("Should exit cleanly");
    }

    #[test]
    #[ignore]
    fn test_counter_decrement() {
        let mut term = TestTerminal::spawn("counter").expect("Failed to spawn counter");

        // Wait for initial state and increment first
        term.wait_for("0", Duration::from_secs(5)).unwrap();
        term.press_key("+").unwrap();
        term.press_key("+").unwrap();
        term.wait_for("2", Duration::from_secs(2)).unwrap();

        // Decrement
        term.press_key("-").expect("Should send - key");
        term.wait_for("1", Duration::from_secs(2))
            .expect("Count should decrement to 1");

        term.exit().expect("Should exit cleanly");
    }

    #[test]
    #[ignore]
    fn test_counter_quit_with_q() {
        let mut term = TestTerminal::spawn("counter").expect("Failed to spawn counter");
        term.wait_for("Count", Duration::from_secs(5)).unwrap();

        term.press_key("q").expect("Should send q key");

        // Wait a bit then check exit
        std::thread::sleep(Duration::from_millis(100));
        let status = term.exit().expect("Should exit");
        assert!(status.success(), "Should exit with success status");
    }

    #[test]
    #[ignore]
    fn test_counter_quit_with_escape() {
        let mut term = TestTerminal::spawn("counter").expect("Failed to spawn counter");
        term.wait_for("Count", Duration::from_secs(5)).unwrap();

        term.press_key("escape").expect("Should send escape key");

        std::thread::sleep(Duration::from_millis(100));
        let status = term.exit().expect("Should exit");
        assert!(status.success());
    }
}

// ============================================================================
// Spinner Example Tests
// ============================================================================

mod spinner {
    use super::*;

    #[test]
    #[ignore]
    fn test_spinner_displays_loading() {
        let mut term = TestTerminal::spawn("spinner").expect("Failed to spawn spinner");

        // Wait for loading message
        term.wait_for("Loading", Duration::from_secs(5))
            .expect("Should display loading message");

        term.exit().expect("Should exit cleanly");
    }

    #[test]
    #[ignore]
    fn test_spinner_quit_with_q() {
        let mut term = TestTerminal::spawn("spinner").expect("Failed to spawn spinner");
        term.wait_for("Loading", Duration::from_secs(5)).unwrap();

        term.press_key("q").expect("Should send q key");

        std::thread::sleep(Duration::from_millis(100));
        let status = term.exit().expect("Should exit");
        assert!(status.success());
    }

    #[test]
    #[ignore]
    fn test_spinner_shows_quit_hint() {
        let mut term = TestTerminal::spawn("spinner").expect("Failed to spawn spinner");

        // Should show how to quit
        term.wait_for("q", Duration::from_secs(5))
            .or_else(|_| term.wait_for("quit", Duration::from_secs(1)))
            .expect("Should show quit hint");

        term.exit().expect("Should exit cleanly");
    }
}

// ============================================================================
// TextInput Example Tests
// ============================================================================

mod textinput {
    use super::*;

    #[test]
    #[ignore]
    fn test_textinput_displays_prompt() {
        let mut term = TestTerminal::spawn("textinput").expect("Failed to spawn textinput");

        // Wait for name prompt
        term.wait_for("name", Duration::from_secs(5))
            .expect("Should ask for name");

        term.exit().expect("Should exit cleanly");
    }

    #[test]
    #[ignore]
    fn test_textinput_shows_submit_hint() {
        let mut term = TestTerminal::spawn("textinput").expect("Failed to spawn textinput");

        term.wait_for("Enter", Duration::from_secs(5))
            .expect("Should show Enter hint");

        term.exit().expect("Should exit cleanly");
    }

    #[test]
    #[ignore]
    fn test_textinput_quit_with_escape() {
        let mut term = TestTerminal::spawn("textinput").expect("Failed to spawn textinput");
        term.wait_for("name", Duration::from_secs(5)).unwrap();

        term.press_key("escape").expect("Should send escape key");

        std::thread::sleep(Duration::from_millis(100));
        let status = term.exit().expect("Should exit");
        assert!(status.success());
    }
}
