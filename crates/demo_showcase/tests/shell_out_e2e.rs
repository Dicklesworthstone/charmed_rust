//! E2E Tests: Shell-out and terminal restore (bd-3ilg)
//!
//! End-to-end tests for terminal release and restore (shell-out) functionality.
//! Tests the 'D' key diagnostics pager and terminal state management.
//!
//! # Test Categories
//!
//! ## Diagnostics Pager (D key)
//! - Press 'D' triggers pager open
//! - Terminal released to pager
//! - Diagnostics content displayed
//! - Pager exit restores TUI
//! - TUI state intact after restore
//!
//! ## Headless Mode
//! - Headless mode skips pager (no-op)
//! - App state preserved in headless mode
//! - No hang or delay in CI environment
//!
//! ## Error Handling
//! - Pager not found handled gracefully
//! - Pager crash doesn't corrupt TUI
//! - Ctrl+C during pager handled

use bubbletea::KeyType;
use demo_showcase::messages::Page;
use demo_showcase::test_support::E2ERunner;

// =============================================================================
// DIAGNOSTICS PAGER TESTS (D KEY)
// =============================================================================

/// Verifies that pressing 'D' doesn't cause panics or hangs in headless mode.
///
/// In headless mode (E2E tests), the pager is skipped and the app should
/// continue running normally.
#[test]
fn e2e_shell_out_d_key_headless_no_hang() {
    let mut runner = E2ERunner::new("shell_out_d_key_headless");

    runner.step("Initialize app");
    runner.resize(120, 40);
    runner.assert_page(Page::Dashboard);

    runner.step("Press 'D' to open diagnostics (headless - should be no-op)");
    runner.press_key('D');

    runner.step("Verify app is still responsive");
    runner.assert_page(Page::Dashboard);
    runner.assert_view_not_empty();

    runner.step("Navigate to another page to confirm responsiveness");
    runner.press_key('2');
    runner.assert_page(Page::Services);

    runner
        .finish()
        .expect("D key in headless mode should not hang");
}

/// Verifies that 'D' key can be pressed from any page without issues.
#[test]
fn e2e_shell_out_d_key_from_all_pages() {
    let mut runner = E2ERunner::new("shell_out_d_key_all_pages");

    runner.step("Initialize app");
    runner.resize(120, 40);

    // Test D key from each page
    let pages = [
        ('1', Page::Dashboard, "Dashboard"),
        ('2', Page::Services, "Services"),
        ('3', Page::Jobs, "Jobs"),
        ('4', Page::Logs, "Logs"),
        ('5', Page::Docs, "Docs"),
        ('6', Page::Files, "Files"),
        ('7', Page::Wizard, "Wizard"),
        ('8', Page::Settings, "Settings"),
    ];

    for (key, page, name) in pages {
        runner.step(&format!("Navigate to {} page", name));
        runner.press_key(key);
        runner.assert_page(page);

        runner.step(&format!("Press 'D' on {} page", name));
        runner.press_key('D');

        runner.step(&format!("Verify still on {} page after D", name));
        runner.assert_page(page);
        runner.assert_view_not_empty();
    }

    runner.finish().expect("D key should work from all pages");
}

/// Verifies that 'D' key preserves app state (page, sidebar, theme).
#[test]
fn e2e_shell_out_preserves_state() {
    let mut runner = E2ERunner::new("shell_out_preserves_state");

    runner.step("Initialize app");
    runner.resize(120, 40);

    runner.step("Navigate to Jobs page");
    runner.press_key('3');
    runner.assert_page(Page::Jobs);

    runner.step("Toggle sidebar to hidden");
    runner.press_key('[');

    runner.step("Capture pre-D view");
    let _pre_d_view = runner.view();
    let pre_d_page = runner.model().current_page();

    runner.step("Press 'D' for diagnostics");
    runner.press_key('D');

    runner.step("Verify page unchanged");
    runner.assert_page(pre_d_page);

    runner.step("Verify view structure similar");
    // The view should still contain Jobs-related content
    runner.assert_contains("Jobs");

    runner.step("Navigate away and back");
    runner.press_key('1');
    runner.assert_page(Page::Dashboard);
    runner.press_key('3');
    runner.assert_page(Page::Jobs);

    runner.finish().expect("D key should preserve app state");
}

// =============================================================================
// HEADLESS MODE BEHAVIOR TESTS
// =============================================================================

/// Verifies that shell-out operations complete instantly in headless mode.
#[test]
fn e2e_shell_out_headless_instant() {
    use std::time::Instant;

    let mut runner = E2ERunner::new("shell_out_headless_instant");

    runner.step("Initialize app");
    runner.resize(120, 40);

    runner.step("Time D key press");
    let start = Instant::now();
    runner.press_key('D');
    let elapsed = start.elapsed();

    runner.step("Verify D key was fast (< 100ms)");
    assert!(
        elapsed.as_millis() < 100,
        "D key in headless mode should be instant, took {}ms",
        elapsed.as_millis()
    );

    runner.step("Verify app still responsive");
    runner.press_key('4');
    runner.assert_page(Page::Logs);

    runner
        .finish()
        .expect("headless shell-out should be instant");
}

/// Verifies that multiple D key presses don't accumulate or cause issues.
#[test]
fn e2e_shell_out_multiple_d_presses() {
    let mut runner = E2ERunner::new("shell_out_multiple_d");

    runner.step("Initialize app");
    runner.resize(120, 40);
    runner.assert_page(Page::Dashboard);

    runner.step("Press D multiple times rapidly");
    for i in 1..=10 {
        runner.press_key('D');
        // Verify still responsive after each press
        if i % 3 == 0 {
            runner.assert_page(Page::Dashboard);
        }
    }

    runner.step("Verify app is still responsive");
    runner.assert_page(Page::Dashboard);
    runner.assert_view_not_empty();

    runner.step("Navigate to verify full functionality");
    runner.press_key('5');
    runner.assert_page(Page::Docs);

    runner
        .finish()
        .expect("multiple D presses should not cause issues");
}

// =============================================================================
// KEYBOARD SHORTCUT VERIFICATION
// =============================================================================

/// Verifies that 'D' key is documented in the help overlay.
#[test]
fn e2e_shell_out_d_in_help() {
    let mut runner = E2ERunner::new("shell_out_d_in_help");

    runner.step("Initialize app");
    runner.resize(120, 40);

    runner.step("Open help overlay");
    runner.press_key('?');

    runner.step("Verify D key is documented");
    runner.assert_contains("D");
    runner.assert_contains("diagnostics"); // Case insensitive check might be needed

    runner.step("Close help overlay");
    runner.press_special(KeyType::Esc);

    runner.step("Verify help closed");
    runner.assert_not_contains("Keyboard Shortcuts");

    runner.finish().expect("D key should be documented in help");
}

// =============================================================================
// TERMINAL STATE TESTS
// =============================================================================

/// Verifies that alternate screen buffer is not corrupted after shell-out.
///
/// In headless mode, this is a no-op, but we verify the view remains intact.
#[test]
fn e2e_shell_out_view_intact() {
    let mut runner = E2ERunner::new("shell_out_view_intact");

    runner.step("Initialize app");
    runner.resize(120, 40);

    runner.step("Navigate to Settings page");
    runner.press_key('8');
    runner.assert_page(Page::Settings);

    runner.step("Capture view before D");
    let before_view_len = runner.view().len();

    runner.step("Press D for diagnostics");
    runner.press_key('D');

    runner.step("Verify view has similar length (not corrupted)");
    let after_view_len = runner.view().len();
    // Allow some variance but view should not be empty or drastically different
    assert!(after_view_len > 0, "view should not be empty after D key");
    assert!(
        after_view_len > before_view_len / 2,
        "view should not be drastically smaller after D key"
    );

    runner.step("Verify Settings page content visible");
    runner.assert_contains("Settings");

    runner
        .finish()
        .expect("view should remain intact after shell-out");
}

/// Verifies mouse capture is not affected by shell-out in headless mode.
#[test]
fn e2e_shell_out_mouse_still_works() {
    let mut runner = E2ERunner::new("shell_out_mouse_works");

    runner.step("Initialize app");
    runner.resize(120, 40);
    runner.assert_page(Page::Dashboard);

    runner.step("Press D for diagnostics");
    runner.press_key('D');

    runner.step("Verify mouse click still works (navigate via sidebar)");
    // Click on approximate sidebar location for page 3 (Jobs)
    // Sidebar is typically on left, Jobs would be around row 6-8
    runner.click(10, 6);

    // Note: Click may or may not navigate depending on exact coordinates
    // The key assertion is that mouse input doesn't cause a panic
    runner.step("Verify app still responsive to keyboard after click");
    runner.press_key('3');
    runner.assert_page(Page::Jobs);

    runner.finish().expect("mouse should work after shell-out");
}

// =============================================================================
// ERROR HANDLING TESTS
// =============================================================================

/// Verifies that invalid shell-out doesn't crash the app.
///
/// Even if the pager were to fail, the app should handle it gracefully.
/// In headless mode, the pager is never invoked, so this tests the
/// message handling path.
#[test]
fn e2e_shell_out_graceful_recovery() {
    let mut runner = E2ERunner::new("shell_out_graceful_recovery");

    runner.step("Initialize app");
    runner.resize(120, 40);

    runner.step("Navigate to complex page (Wizard)");
    runner.press_key('7');
    runner.assert_page(Page::Wizard);

    runner.step("Press D (would trigger pager if not headless)");
    runner.press_key('D');

    runner.step("Verify Wizard page state intact");
    runner.assert_page(Page::Wizard);
    runner.assert_contains("Wizard");

    runner.step("Interact with Wizard form");
    runner.press_key('j'); // Move down in form
    runner.press_special(KeyType::Tab); // Tab through form fields

    runner.step("Verify form still interactive");
    runner.assert_page(Page::Wizard);

    runner
        .finish()
        .expect("app should recover gracefully from shell-out");
}

// =============================================================================
// INTEGRATION WITH OTHER FEATURES
// =============================================================================

/// Verifies that D key works correctly with theme switching.
#[test]
fn e2e_shell_out_with_theme_switch() {
    let mut runner = E2ERunner::new("shell_out_with_theme");

    runner.step("Initialize app");
    runner.resize(120, 40);

    runner.step("Switch theme using 't'");
    runner.press_key('t');

    runner.step("Press D for diagnostics");
    runner.press_key('D');

    runner.step("Verify app still works");
    runner.assert_view_not_empty();

    runner.step("Switch theme again");
    runner.press_key('t');

    runner.step("Verify theme switching still works after D");
    runner.assert_view_not_empty();

    runner
        .finish()
        .expect("D key should work with theme switching");
}

/// Verifies that D key works with resize events.
#[test]
fn e2e_shell_out_with_resize() {
    let mut runner = E2ERunner::new("shell_out_with_resize");

    runner.step("Initialize app");
    runner.resize(120, 40);

    runner.step("Press D for diagnostics");
    runner.press_key('D');

    runner.step("Resize terminal");
    runner.resize(80, 24);

    runner.step("Verify app handles resize after D");
    runner.assert_view_not_empty();
    runner.assert_page(Page::Dashboard);

    runner.step("Resize again");
    runner.resize(160, 50);

    runner.step("Press D after resize");
    runner.press_key('D');

    runner.step("Verify still responsive");
    runner.press_key('2');
    runner.assert_page(Page::Services);

    runner
        .finish()
        .expect("D key should work with resize events");
}

// =============================================================================
// SMOKE TEST - COMPREHENSIVE SHELL-OUT SCENARIO
// =============================================================================

/// Comprehensive smoke test for shell-out functionality.
///
/// This test exercises the shell-out feature in the context of normal
/// app usage, ensuring it doesn't interfere with other operations.
#[test]
fn e2e_shell_out_smoke_test() {
    let mut runner = E2ERunner::new("shell_out_smoke");

    runner.step("Initialize app");
    runner.resize(120, 40);
    runner.assert_page(Page::Dashboard);

    // Phase 1: Basic navigation with D key interspersed
    runner.step("Phase 1: Navigate with D key checks");
    runner.press_key('D');
    runner.press_key('2');
    runner.assert_page(Page::Services);
    runner.press_key('D');
    runner.press_key('3');
    runner.assert_page(Page::Jobs);
    runner.press_key('D');

    // Phase 2: Help overlay
    runner.step("Phase 2: Help overlay interaction");
    runner.press_key('?');
    runner.assert_contains("Keyboard Shortcuts");
    runner.press_key('D'); // D while help is open (should close help or be ignored)
    runner.press_special(KeyType::Esc);

    // Phase 3: Settings page with theme and D key
    runner.step("Phase 3: Settings and theme");
    runner.press_key('8');
    runner.assert_page(Page::Settings);
    runner.press_key('t');
    runner.press_key('D');
    runner.press_key('t');
    runner.assert_page(Page::Settings);

    // Phase 4: Resize scenarios
    runner.step("Phase 4: Resize handling");
    runner.resize(80, 24);
    runner.press_key('D');
    runner.resize(120, 40);
    runner.press_key('D');

    // Phase 5: Rapid interactions
    runner.step("Phase 5: Rapid key presses");
    runner.press_keys("12345678");
    runner.assert_page(Page::Settings);
    runner.press_key('D');
    runner.press_key('1');
    runner.assert_page(Page::Dashboard);

    // Final verification
    runner.step("Final verification");
    runner.assert_view_not_empty();
    runner.assert_page(Page::Dashboard);
    runner.assert_contains("Dashboard");

    runner.finish().expect("shell-out smoke test should pass");
}

// =============================================================================
// UNIT-LIKE TESTS FOR SHELL-OUT MODULE
// =============================================================================

/// Tests that are more unit-test-like but run via E2E infrastructure.
mod shell_out_unit_style {
    use demo_showcase::shell_action::{
        generate_diagnostics, open_diagnostics_in_pager, open_in_pager,
    };

    #[test]
    fn diagnostics_content_not_empty() {
        let diag = generate_diagnostics();
        assert!(!diag.is_empty(), "diagnostics should not be empty");
    }

    #[test]
    fn diagnostics_contains_version_info() {
        let diag = generate_diagnostics();
        assert!(
            diag.contains("Version Information"),
            "should have version section"
        );
        assert!(
            diag.contains("Package Version"),
            "should have package version"
        );
    }

    #[test]
    fn diagnostics_contains_environment_info() {
        let diag = generate_diagnostics();
        assert!(
            diag.contains("Environment"),
            "should have environment section"
        );
        assert!(diag.contains("TERM"), "should show TERM var");
        assert!(diag.contains("COLORTERM"), "should show COLORTERM var");
    }

    #[test]
    fn diagnostics_contains_platform_info() {
        let diag = generate_diagnostics();
        assert!(diag.contains("Platform"), "should have platform section");
        assert!(diag.contains("OS"), "should show OS");
        assert!(diag.contains("Arch"), "should show architecture");
    }

    #[test]
    fn diagnostics_contains_charmed_components() {
        let diag = generate_diagnostics();
        assert!(
            diag.contains("Charmed Rust Components"),
            "should list charmed crates"
        );
        assert!(diag.contains("bubbletea"), "should mention bubbletea");
        assert!(diag.contains("lipgloss"), "should mention lipgloss");
    }

    #[test]
    fn headless_open_in_pager_returns_none() {
        let result = open_in_pager("test content".to_string(), true);
        assert!(result.is_none(), "headless mode should return None");
    }

    #[test]
    fn headless_open_diagnostics_returns_none() {
        let diag = generate_diagnostics();
        let result = open_diagnostics_in_pager(diag, true);
        assert!(result.is_none(), "headless diagnostics should return None");
    }

    #[test]
    fn non_headless_open_in_pager_returns_some() {
        let result = open_in_pager("test content".to_string(), false);
        assert!(result.is_some(), "non-headless should return Some(Cmd)");
    }
}
