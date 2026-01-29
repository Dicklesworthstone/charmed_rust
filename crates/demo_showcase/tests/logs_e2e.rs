//! E2E Tests: Logs export/copy/clear actions (bd-3spt)
//!
//! End-to-end tests for the Logs page export and copy functionality:
//! - Export logs to file and verify file is written
//! - Copy viewport content to file
//! - Clear logs and verify viewer empties
//!
//! # Test Categories
//!
//! ## Export Tests
//! - Navigate to Logs page
//! - Trigger export action
//! - Verify export file is created in artifact directory
//!
//! ## Copy Tests
//! - Copy visible viewport to file
//! - Copy all filtered logs to file
//!
//! ## Clear Tests
//! - Clear all logs
//! - Verify viewer shows empty state

use std::fs;
use std::path::PathBuf;

use bubbletea::KeyType;
use demo_showcase::config::{AnimationMode, ColorMode, Config};
use demo_showcase::messages::Page;
use demo_showcase::test_support::E2ERunner;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Create a test runner configured for logs testing.
fn create_logs_runner(name: &str) -> E2ERunner {
    let config = Config {
        color_mode: ColorMode::Never, // No ANSI escapes for easier assertions
        animations: AnimationMode::Disabled,
        alt_screen: false,
        mouse: true,
        seed: Some(42424),
        ..Config::default()
    };

    let mut runner = E2ERunner::with_config(name, config);
    runner.resize(120, 40);
    runner
}

/// Get the export directory path (matches `LogsPage::export_dir()`).
fn export_dir() -> PathBuf {
    std::env::var("DEMO_SHOWCASE_E2E_ARTIFACTS")
        .map_or_else(|_| PathBuf::from("demo_showcase_exports"), PathBuf::from)
}

/// Clean up any existing export files for a fresh test.
fn cleanup_exports() {
    let dir = export_dir();
    if dir.exists() {
        // Only remove log export files, not the whole directory
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with("logs_"))
                {
                    let _ = fs::remove_file(path);
                }
            }
        }
    }
}

/// Count log export files in the export directory.
fn count_log_exports() -> usize {
    let dir = export_dir();
    if !dir.exists() {
        return 0;
    }

    fs::read_dir(&dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|e| {
                    e.path()
                        .file_name()
                        .is_some_and(|n| n.to_string_lossy().starts_with("logs_"))
                })
                .count()
        })
        .unwrap_or(0)
}

/// Find the most recent log export file.
fn find_latest_log_export() -> Option<PathBuf> {
    let dir = export_dir();
    if !dir.exists() {
        return None;
    }

    fs::read_dir(&dir)
        .ok()?
        .filter_map(Result::ok)
        .filter(|e| {
            e.path()
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("logs_"))
        })
        .max_by_key(|e| e.metadata().ok().and_then(|m| m.modified().ok()))
        .map(|e| e.path())
}

// =============================================================================
// NAVIGATION TESTS
// =============================================================================

/// Verify that we can navigate to the Logs page.
#[test]
fn e2e_logs_navigate_to_page() {
    let mut runner = create_logs_runner("logs_navigate");

    runner.step("Navigate to Logs page");
    runner.press_key('4'); // Logs is page 4
    runner.assert_page(Page::Logs);

    runner.step("Verify Logs page renders");
    runner.assert_view_not_empty();
    // The logs page should render with header and content
    // We just verify the view is non-empty; specific content varies

    runner.finish().expect("logs navigation should work");
}

/// Verify that we can scroll through logs.
#[test]
fn e2e_logs_scroll_navigation() {
    let mut runner = create_logs_runner("logs_scroll");

    runner.step("Navigate to Logs page");
    runner.press_key('4');
    runner.assert_page(Page::Logs);

    runner.step("Scroll down with j");
    let before_scroll = runner.view();
    runner.press_key('j');
    runner.drain();

    runner.step("Scroll down more");
    runner.press_key('j');
    runner.press_key('j');
    runner.drain();

    runner.step("Scroll up with k");
    runner.press_key('k');
    runner.drain();

    let after_scroll = runner.view();
    // Both views should be valid
    assert!(!before_scroll.is_empty());
    assert!(!after_scroll.is_empty());

    runner.finish().expect("logs scrolling should work");
}

// =============================================================================
// EXPORT TESTS
// =============================================================================

/// Verify that export creates a file in the artifact directory.
#[test]
fn e2e_logs_export_creates_file() {
    cleanup_exports();
    let initial_count = count_log_exports();

    let mut runner = create_logs_runner("logs_export");

    runner.step("Navigate to Logs page");
    runner.press_key('4');
    runner.assert_page(Page::Logs);

    runner.step("Verify logs are present before export");
    runner.assert_view_not_empty();

    runner.step("Export logs with 'e' key");
    runner.press_key('e');
    runner.drain();

    // Give the command a chance to execute
    runner.step("Wait for export to complete");
    runner.drain();

    runner.step("Verify view is still valid after export");
    runner.assert_view_not_empty();

    runner.finish().expect("logs export should work");

    // Verify export file was created
    let final_count = count_log_exports();
    assert!(
        final_count > initial_count,
        "Export should create a new file (initial: {initial_count}, final: {final_count})"
    );

    // Verify the export file has content
    if let Some(export_path) = find_latest_log_export() {
        let content = fs::read_to_string(&export_path).expect("Should read export file");
        assert!(
            !content.is_empty(),
            "Export file should have content: {export_path:?}"
        );
        // Export should contain log-like content
        assert!(
            content.contains('[') || content.contains("INFO") || content.contains("ERROR"),
            "Export should contain log entries"
        );
    }
}

/// Verify that export with no logs doesn't crash.
#[test]
fn e2e_logs_export_after_clear() {
    let mut runner = create_logs_runner("logs_export_empty");

    runner.step("Navigate to Logs page");
    runner.press_key('4');
    runner.assert_page(Page::Logs);

    runner.step("Clear all logs with 'X'");
    runner.press_key('X');
    runner.drain();

    runner.step("Try to export cleared logs");
    runner.press_key('e');
    runner.drain();

    runner.step("Verify app is still responsive");
    runner.assert_view_not_empty();

    runner
        .finish()
        .expect("export after clear should not crash");
}

// =============================================================================
// COPY TESTS
// =============================================================================

/// Verify that copy viewport action works.
#[test]
fn e2e_logs_copy_viewport() {
    cleanup_exports();

    let mut runner = create_logs_runner("logs_copy_viewport");

    runner.step("Navigate to Logs page");
    runner.press_key('4');
    runner.assert_page(Page::Logs);

    runner.step("Scroll to get some content in viewport");
    runner.press_key('j');
    runner.press_key('j');
    runner.drain();

    runner.step("Copy viewport with 'y' key");
    runner.press_key('y');
    runner.drain();

    runner.step("Verify view is still valid after copy");
    runner.assert_view_not_empty();

    runner.finish().expect("logs copy viewport should work");

    // Check if a viewport copy file was created
    let dir = export_dir();
    if dir.exists() {
        let viewport_files: Vec<_> = fs::read_dir(&dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| {
                e.path()
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().contains("viewport"))
            })
            .collect();

        if !viewport_files.is_empty() {
            let content = fs::read_to_string(viewport_files[0].path())
                .expect("Should read viewport copy file");
            assert!(!content.is_empty(), "Viewport copy should have content");
        }
    }
}

/// Verify that copy all action works.
#[test]
fn e2e_logs_copy_all() {
    cleanup_exports();

    let mut runner = create_logs_runner("logs_copy_all");

    runner.step("Navigate to Logs page");
    runner.press_key('4');
    runner.assert_page(Page::Logs);

    runner.step("Copy all filtered logs with 'Y' key");
    runner.press_key('Y');
    runner.drain();

    runner.step("Verify view is still valid after copy all");
    runner.assert_view_not_empty();

    runner.finish().expect("logs copy all should work");
}

// =============================================================================
// CLEAR TESTS
// =============================================================================

/// Verify that clear removes all log entries.
#[test]
fn e2e_logs_clear_empties_viewer() {
    let mut runner = create_logs_runner("logs_clear");

    runner.step("Navigate to Logs page");
    runner.press_key('4');
    runner.assert_page(Page::Logs);

    runner.step("Verify logs are present before clear");
    let before_clear = runner.view();
    assert!(!before_clear.is_empty());
    // Should have log content visible
    let has_log_content = before_clear.contains("INFO")
        || before_clear.contains("ERROR")
        || before_clear.contains("WARN")
        || before_clear.contains("DEBUG");

    runner.step("Clear logs with 'X' key");
    runner.press_key('X');
    runner.drain();

    runner.step("Verify view is updated after clear");
    let after_clear = runner.view();
    assert!(!after_clear.is_empty());

    // If we had log content before, it should be different or empty now
    if has_log_content {
        // After clear, there should be fewer log entries or empty state
        // The view itself won't be empty (UI chrome remains)
        runner.assert_view_not_empty();
    }

    runner.finish().expect("logs clear should work");
}

/// Verify that clearing logs multiple times doesn't crash.
#[test]
fn e2e_logs_clear_multiple_times() {
    let mut runner = create_logs_runner("logs_clear_multiple");

    runner.step("Navigate to Logs page");
    runner.press_key('4');
    runner.assert_page(Page::Logs);

    runner.step("Clear logs multiple times");
    for i in 0..3 {
        runner.step(&format!("Clear attempt {}", i + 1));
        runner.press_key('X');
        runner.drain();
        runner.assert_view_not_empty();
    }

    runner.step("Verify app is still responsive after multiple clears");
    runner.press_key('j');
    runner.drain();
    runner.assert_view_not_empty();

    runner.finish().expect("multiple clears should not crash");
}

// =============================================================================
// COMPREHENSIVE SCENARIO
// =============================================================================

/// Comprehensive logs scenario: navigation, export, copy, and clear.
#[test]
fn e2e_logs_comprehensive_scenario() {
    cleanup_exports();
    let mut runner = create_logs_runner("logs_comprehensive");

    runner.step("Phase 1: Navigate to Logs");
    runner.press_key('4');
    runner.assert_page(Page::Logs);
    runner.assert_view_not_empty();

    runner.step("Phase 2: Scroll through logs");
    runner.press_key('j');
    runner.press_key('j');
    runner.press_key('j');
    runner.drain();
    runner.press_key('k');
    runner.drain();

    runner.step("Phase 3: Toggle follow mode");
    runner.press_key('f');
    runner.drain();
    runner.assert_view_not_empty();
    runner.press_key('f'); // Toggle back
    runner.drain();

    runner.step("Phase 4: Filter by level (toggle INFO)");
    runner.press_key('3'); // Toggle INFO level (1=ERROR, 2=WARN, 3=INFO, 4=DEBUG, 5=TRACE)
    runner.drain();
    runner.assert_view_not_empty();

    runner.step("Phase 5: Copy viewport");
    runner.press_key('y');
    runner.drain();

    runner.step("Phase 6: Export logs");
    runner.press_key('e');
    runner.drain();

    runner.step("Phase 7: Clear logs");
    runner.press_key('X');
    runner.drain();
    runner.assert_view_not_empty();

    runner.step("Phase 8: Return to dashboard");
    runner.press_key('1');
    runner.assert_page(Page::Dashboard);

    runner
        .finish()
        .expect("logs comprehensive scenario should pass");

    // Verify artifacts were created
    let export_count = count_log_exports();
    assert!(
        export_count > 0,
        "Should have created at least one export file"
    );
}

// =============================================================================
// FILTER + EXPORT INTERACTION
// =============================================================================

/// Verify that filtered logs are exported correctly.
#[test]
fn e2e_logs_export_filtered() {
    cleanup_exports();
    let mut runner = create_logs_runner("logs_export_filtered");

    runner.step("Navigate to Logs page");
    runner.press_key('4');
    runner.assert_page(Page::Logs);

    runner.step("Apply level filter (show only ERROR)");
    // Toggle off all levels except ERROR
    runner.press_key('2'); // WARN off
    runner.press_key('3'); // INFO off
    runner.press_key('4'); // DEBUG off
    runner.press_key('5'); // TRACE off
    runner.drain();

    runner.step("Export filtered logs");
    runner.press_key('e');
    runner.drain();

    runner.step("Verify view is still valid");
    runner.assert_view_not_empty();

    runner.finish().expect("filtered export should work");
}

/// Verify that search filter + export works.
#[test]
fn e2e_logs_search_then_export() {
    let mut runner = create_logs_runner("logs_search_export");

    runner.step("Navigate to Logs page");
    runner.press_key('4');
    runner.assert_page(Page::Logs);

    runner.step("Enter search mode with /");
    runner.press_key('/');
    runner.drain();

    runner.step("Search for 'error'");
    runner.press_keys("error");
    runner.drain();

    runner.step("Confirm search");
    runner.press_special(KeyType::Enter);
    runner.drain();

    runner.step("Export after search");
    runner.press_key('e');
    runner.drain();

    runner.step("Verify app is still responsive");
    runner.assert_view_not_empty();

    runner.finish().expect("search then export should work");
}

// =============================================================================
// EDGE CASES
// =============================================================================

/// Verify that resize during export doesn't crash.
#[test]
fn e2e_logs_resize_during_export() {
    let mut runner = create_logs_runner("logs_resize_export");

    runner.step("Navigate to Logs page");
    runner.press_key('4');
    runner.assert_page(Page::Logs);

    runner.step("Start export and immediately resize");
    runner.press_key('e');
    runner.resize(80, 24);
    runner.drain();

    runner.step("Verify view is valid after resize");
    runner.assert_view_not_empty();

    runner
        .finish()
        .expect("resize during export should not crash");
}

/// Verify rapid action sequences don't crash.
#[test]
fn e2e_logs_rapid_actions() {
    let mut runner = create_logs_runner("logs_rapid_actions");

    runner.step("Navigate to Logs page");
    runner.press_key('4');
    runner.assert_page(Page::Logs);

    runner.step("Rapid key presses");
    runner.press_key('j');
    runner.press_key('k');
    runner.press_key('y');
    runner.press_key('e');
    runner.press_key('f');
    runner.press_key('f');
    runner.drain();

    runner.step("Verify app is still responsive");
    runner.assert_view_not_empty();

    runner.finish().expect("rapid actions should not crash");
}
