//! Snapshot tests for key views (bd-2mvl)
//!
//! Uses insta for snapshot testing to catch UI regressions.
//! Snapshots are stored in `tests/snapshots/` and should be committed.
//!
//! # Running
//!
//! ```bash
//! cargo test -p demo_showcase --test snapshot_tests
//!
//! # Update snapshots:
//! cargo insta test -p demo_showcase --test snapshot_tests
//! cargo insta review
//! ```
//!
//! # Strategy
//!
//! - Use ASCII/no-color mode for deterministic output
//! - Fixed terminal size (80x24 or 120x40) for consistency
//! - Strip ANSI codes for cleaner diffs
//! - Focus on structure, not colors

use demo_showcase::config::{AnimationMode, ColorMode, Config};
use demo_showcase::messages::Page;
use demo_showcase::test_support::E2ERunner;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Strip ANSI escape codes from a string for cleaner snapshots.
fn strip_ansi(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut in_escape = false;

    for c in input.chars() {
        if c == '\x1b' {
            in_escape = true;
            continue;
        }
        if in_escape {
            if c == 'm' {
                in_escape = false;
            }
            continue;
        }
        result.push(c);
    }
    result
}

/// Create a test runner with fixed size and no-color mode.
fn create_snapshot_runner(name: &str, width: u16, height: u16) -> E2ERunner {
    let mut config = Config::default();
    config.color_mode = ColorMode::Never;
    config.animations = AnimationMode::Disabled;
    config.alt_screen = false;
    // Use fixed seed for deterministic output
    config.seed = Some(12345);

    let mut runner = E2ERunner::with_config(name, config);
    runner.resize(width, height);
    runner
}

/// Redact dynamic time-based values that would cause snapshot flakiness.
/// Replaces Duration patterns like "19371h 34m" with "[DURATION]".
fn redact_dynamic_times(input: &str) -> String {
    // Pattern: Duration field shows elapsed time like "19371h 34m" or "2m 30s"
    // We redact the hours+minutes or minutes+seconds pattern after "Duration:"
    let re = regex::Regex::new(r"Duration:\s*\d+h\s*\d+m").unwrap();
    let result = re.replace_all(input, "Duration: [REDACTED]");

    // Also handle shorter durations (minutes only)
    let re2 = regex::Regex::new(r"Duration:\s*\d+m\s*\d*s?").unwrap();
    re2.replace_all(&result, "Duration: [REDACTED]").to_string()
}

/// Get a clean snapshot-ready view from the runner.
fn snapshot_view(runner: &E2ERunner) -> String {
    redact_dynamic_times(&strip_ansi(&runner.view()))
}

// =============================================================================
// DASHBOARD SNAPSHOTS
// =============================================================================

#[test]
fn snapshot_dashboard_80x24() {
    let mut runner = create_snapshot_runner("snapshot_dashboard_80x24", 80, 24);

    runner.step("Navigate to Dashboard");
    runner.press_key('1');
    runner.assert_page(Page::Dashboard);

    let view = snapshot_view(&runner);
    insta::assert_snapshot!("dashboard_80x24", view);

    runner.finish().expect("Dashboard snapshot test");
}

#[test]
fn snapshot_dashboard_120x40() {
    let mut runner = create_snapshot_runner("snapshot_dashboard_120x40", 120, 40);

    runner.step("Navigate to Dashboard");
    runner.press_key('1');
    runner.assert_page(Page::Dashboard);

    let view = snapshot_view(&runner);
    insta::assert_snapshot!("dashboard_120x40", view);

    runner.finish().expect("Dashboard snapshot test");
}

// =============================================================================
// JOBS PAGE SNAPSHOTS
// =============================================================================

#[test]
fn snapshot_jobs_80x24() {
    let mut runner = create_snapshot_runner("snapshot_jobs_80x24", 80, 24);

    runner.step("Navigate to Jobs page");
    runner.press_key('3');
    runner.assert_page(Page::Jobs);

    let view = snapshot_view(&runner);
    insta::assert_snapshot!("jobs_80x24", view);

    runner.finish().expect("Jobs snapshot test");
}

#[test]
fn snapshot_jobs_120x40() {
    let mut runner = create_snapshot_runner("snapshot_jobs_120x40", 120, 40);

    runner.step("Navigate to Jobs page");
    runner.press_key('3');
    runner.assert_page(Page::Jobs);

    let view = snapshot_view(&runner);
    insta::assert_snapshot!("jobs_120x40", view);

    runner.finish().expect("Jobs snapshot test");
}

// =============================================================================
// LOGS PAGE SNAPSHOTS
// =============================================================================
//
// NOTE: Logs page snapshots are NOT included because log entries contain
// dynamic timestamps that change between test runs. The logs page is tested
// via E2E tests (bd-1s7t) which verify functionality without snapshotting
// the exact rendered output.

// =============================================================================
// DOCS PAGE SNAPSHOTS
// =============================================================================

#[test]
fn snapshot_docs_80x24() {
    let mut runner = create_snapshot_runner("snapshot_docs_80x24", 80, 24);

    runner.step("Navigate to Docs page");
    runner.press_key('5');
    runner.assert_page(Page::Docs);

    let view = snapshot_view(&runner);
    insta::assert_snapshot!("docs_80x24", view);

    runner.finish().expect("Docs snapshot test");
}

#[test]
fn snapshot_docs_120x40() {
    let mut runner = create_snapshot_runner("snapshot_docs_120x40", 120, 40);

    runner.step("Navigate to Docs page");
    runner.press_key('5');
    runner.assert_page(Page::Docs);

    let view = snapshot_view(&runner);
    insta::assert_snapshot!("docs_120x40", view);

    runner.finish().expect("Docs snapshot test");
}

/// Test docs at narrow width to verify reflow.
#[test]
fn snapshot_docs_narrow_60x24() {
    let mut runner = create_snapshot_runner("snapshot_docs_narrow_60x24", 60, 24);

    runner.step("Navigate to Docs page");
    runner.press_key('5');
    runner.assert_page(Page::Docs);

    let view = snapshot_view(&runner);
    insta::assert_snapshot!("docs_narrow_60x24", view);

    runner.finish().expect("Docs narrow snapshot test");
}

// =============================================================================
// SETTINGS PAGE SNAPSHOTS
// =============================================================================

#[test]
fn snapshot_settings_80x24() {
    let mut runner = create_snapshot_runner("snapshot_settings_80x24", 80, 24);

    runner.step("Navigate to Settings page");
    runner.press_key('8');
    runner.assert_page(Page::Settings);

    let view = snapshot_view(&runner);
    insta::assert_snapshot!("settings_80x24", view);

    runner.finish().expect("Settings snapshot test");
}

#[test]
fn snapshot_settings_120x40() {
    let mut runner = create_snapshot_runner("snapshot_settings_120x40", 120, 40);

    runner.step("Navigate to Settings page");
    runner.press_key('8');
    runner.assert_page(Page::Settings);

    let view = snapshot_view(&runner);
    insta::assert_snapshot!("settings_120x40", view);

    runner.finish().expect("Settings snapshot test");
}

// =============================================================================
// SERVICES PAGE SNAPSHOTS
// =============================================================================

#[test]
fn snapshot_services_80x24() {
    let mut runner = create_snapshot_runner("snapshot_services_80x24", 80, 24);

    runner.step("Navigate to Services page");
    runner.press_key('2');
    runner.assert_page(Page::Services);

    let view = snapshot_view(&runner);
    insta::assert_snapshot!("services_80x24", view);

    runner.finish().expect("Services snapshot test");
}

// =============================================================================
// FILES PAGE SNAPSHOTS
// =============================================================================

#[test]
fn snapshot_files_80x24() {
    let mut runner = create_snapshot_runner("snapshot_files_80x24", 80, 24);

    runner.step("Navigate to Files page");
    runner.press_key('6');
    runner.assert_page(Page::Files);

    let view = snapshot_view(&runner);
    insta::assert_snapshot!("files_80x24", view);

    runner.finish().expect("Files snapshot test");
}

// =============================================================================
// WIZARD PAGE SNAPSHOTS
// =============================================================================

#[test]
fn snapshot_wizard_80x24() {
    let mut runner = create_snapshot_runner("snapshot_wizard_80x24", 80, 24);

    runner.step("Navigate to Wizard page");
    runner.press_key('7');
    runner.assert_page(Page::Wizard);

    let view = snapshot_view(&runner);
    insta::assert_snapshot!("wizard_80x24", view);

    runner.finish().expect("Wizard snapshot test");
}

// =============================================================================
// HELP OVERLAY SNAPSHOT
// =============================================================================

#[test]
fn snapshot_help_overlay() {
    let mut runner = create_snapshot_runner("snapshot_help_overlay", 100, 30);

    runner.step("Open help overlay");
    runner.press_key('?');

    let view = snapshot_view(&runner);
    insta::assert_snapshot!("help_overlay", view);

    runner.finish().expect("Help overlay snapshot test");
}

// =============================================================================
// SIDEBAR STATES
// =============================================================================

#[test]
fn snapshot_sidebar_collapsed() {
    let mut runner = create_snapshot_runner("snapshot_sidebar_collapsed", 80, 24);

    runner.step("Navigate to Dashboard");
    runner.press_key('1');
    runner.assert_page(Page::Dashboard);

    runner.step("Collapse sidebar");
    runner.press_key('[');

    let view = snapshot_view(&runner);
    insta::assert_snapshot!("sidebar_collapsed", view);

    runner.finish().expect("Sidebar collapsed snapshot test");
}

// =============================================================================
// RESPONSIVE LAYOUT TESTS
// =============================================================================

#[test]
fn snapshot_responsive_very_narrow() {
    let mut runner = create_snapshot_runner("snapshot_responsive_narrow", 40, 20);

    runner.step("Navigate to Dashboard");
    runner.press_key('1');
    runner.assert_page(Page::Dashboard);

    let view = snapshot_view(&runner);
    insta::assert_snapshot!("responsive_narrow_40x20", view);

    runner.finish().expect("Narrow responsive snapshot test");
}

#[test]
fn snapshot_responsive_wide() {
    let mut runner = create_snapshot_runner("snapshot_responsive_wide", 160, 50);

    runner.step("Navigate to Dashboard");
    runner.press_key('1');
    runner.assert_page(Page::Dashboard);

    let view = snapshot_view(&runner);
    insta::assert_snapshot!("responsive_wide_160x50", view);

    runner.finish().expect("Wide responsive snapshot test");
}
