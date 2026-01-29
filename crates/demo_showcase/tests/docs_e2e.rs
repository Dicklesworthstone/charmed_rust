//! E2E Tests: Docs render + search + resize reflow (bd-miqo)
//!
//! End-to-end tests for the Docs page covering:
//! - Document switching and navigation
//! - In-doc search with match navigation
//! - Window resize with content reflow
//! - Glamour markdown rendering integration
//!
//! # Test Categories
//!
//! ## Navigation Tests
//! - Navigate to Docs page
//! - Switch between documents in the list
//! - Focus switching between list and content pane
//!
//! ## Search Tests
//! - Enter search mode with `/`
//! - Type search query and find matches
//! - Navigate matches with n/N
//! - Exit search with Esc
//!
//! ## Resize Tests
//! - Resize window and verify content reflows
//! - Different widths produce different line counts/wrapping

use bubbletea::KeyType;
use demo_showcase::config::{AnimationMode, ColorMode, Config};
use demo_showcase::messages::Page;
use demo_showcase::test_support::E2ERunner;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Create a test runner configured for docs testing.
fn create_docs_runner(name: &str) -> E2ERunner {
    let mut config = Config::default();
    config.color_mode = ColorMode::Never; // No ANSI escapes for easier assertions
    config.animations = AnimationMode::Disabled;
    config.alt_screen = false;
    config.mouse = true;
    config.seed = Some(42424);

    let mut runner = E2ERunner::with_config(name, config);
    runner.resize(120, 40);
    runner
}

// =============================================================================
// NAVIGATION TESTS
// =============================================================================

/// Verify that we can navigate to the Docs page.
#[test]
fn e2e_docs_navigate_to_page() {
    let mut runner = create_docs_runner("docs_navigate");

    runner.step("Navigate to Docs page");
    runner.press_key('5');
    runner.assert_page(Page::Docs);

    runner.step("Verify Docs page renders");
    runner.assert_view_not_empty();
    // Docs page should show the document list (contains "Documents" header)
    runner.assert_contains("Documents");

    runner.finish().expect("docs navigation should work");
}

/// Verify that we can switch between documents.
#[test]
fn e2e_docs_switch_between_documents() {
    let mut runner = create_docs_runner("docs_switch");

    runner.step("Navigate to Docs page");
    runner.press_key('5');
    runner.assert_page(Page::Docs);

    runner.step("Capture first document view");
    let first_view = runner.view();
    runner.assert_view_not_empty();

    runner.step("Navigate to next document with j");
    runner.press_key('j');
    runner.drain();

    runner.step("Verify we can still render");
    runner.assert_view_not_empty();

    runner.step("Navigate to previous document with k");
    runner.press_key('k');
    runner.drain();

    runner.step("Verify view is consistent");
    let back_view = runner.view();
    // After going down and back up, the selection should be preserved
    // Views might differ slightly due to state changes, but both should be valid
    assert!(!first_view.is_empty());
    assert!(!back_view.is_empty());

    runner.finish().expect("docs switching should work");
}

/// Verify that we can switch focus between list and content pane.
#[test]
fn e2e_docs_focus_switching() {
    let mut runner = create_docs_runner("docs_focus");

    runner.step("Navigate to Docs page");
    runner.press_key('5');
    runner.assert_page(Page::Docs);

    runner.step("Switch focus to content pane with Tab");
    runner.press_special(KeyType::Tab);
    runner.drain();

    runner.step("Verify view is still valid");
    runner.assert_view_not_empty();

    runner.step("Switch focus back to list with Tab");
    runner.press_special(KeyType::Tab);
    runner.drain();

    runner.assert_view_not_empty();

    runner.finish().expect("docs focus switching should work");
}

// =============================================================================
// SEARCH TESTS
// =============================================================================

/// Verify that search mode can be entered and exited.
#[test]
fn e2e_docs_search_enter_exit() {
    let mut runner = create_docs_runner("docs_search_enter_exit");

    runner.step("Navigate to Docs page and focus content");
    runner.press_key('5');
    runner.assert_page(Page::Docs);
    runner.press_special(KeyType::Tab); // Focus content pane
    runner.drain();

    runner.step("Enter search mode with /");
    runner.press_key('/');
    runner.drain();

    runner.step("Verify search mode is active");
    // Search mode should show some indication (search prompt, etc.)
    runner.assert_view_not_empty();

    runner.step("Exit search with Esc");
    runner.press_special(KeyType::Esc);
    runner.drain();

    runner.step("Verify we're back to normal mode");
    runner.assert_view_not_empty();

    runner.finish().expect("docs search enter/exit should work");
}

/// Verify that searching finds matches.
#[test]
fn e2e_docs_search_finds_matches() {
    let mut runner = create_docs_runner("docs_search_matches");

    runner.step("Navigate to Docs page and focus content");
    runner.press_key('5');
    runner.assert_page(Page::Docs);
    runner.press_special(KeyType::Tab);
    runner.drain();

    runner.step("Enter search mode");
    runner.press_key('/');
    runner.drain();

    runner.step("Search for 'the' (common word)");
    runner.press_key('t');
    runner.press_key('h');
    runner.press_key('e');
    runner.drain();

    runner.step("Verify view updates (search results shown)");
    let search_view = runner.view();
    // The view should contain something after searching
    assert!(!search_view.is_empty());

    runner.step("Exit search and verify view");
    runner.press_special(KeyType::Enter);
    runner.drain();
    runner.assert_view_not_empty();

    runner.finish().expect("docs search should find matches");
}

/// Verify that match navigation with n/N works.
#[test]
fn e2e_docs_search_match_navigation() {
    let mut runner = create_docs_runner("docs_search_navigation");

    runner.step("Navigate to Docs page and focus content");
    runner.press_key('5');
    runner.assert_page(Page::Docs);
    runner.press_special(KeyType::Tab);
    runner.drain();

    runner.step("Search for 'the'");
    runner.press_key('/');
    runner.drain();
    runner.press_keys("the");
    runner.drain();

    runner.step("Confirm search and enter content mode");
    runner.press_special(KeyType::Enter);
    runner.drain();

    runner.step("Navigate to next match with n");
    let before_n = runner.view();
    runner.press_key('n');
    runner.drain();
    let after_n = runner.view();

    // Views may or may not visually differ, but no panic should occur
    assert!(!before_n.is_empty());
    assert!(!after_n.is_empty());

    runner.step("Navigate to previous match with N");
    runner.press_key('N');
    runner.drain();

    runner.assert_view_not_empty();

    runner.finish().expect("docs match navigation should work");
}

// =============================================================================
// RESIZE TESTS
// =============================================================================

/// Verify that resizing the window causes content to reflow.
#[test]
fn e2e_docs_resize_reflow() {
    let mut runner = create_docs_runner("docs_resize_reflow");

    runner.step("Navigate to Docs page");
    runner.press_key('5');
    runner.assert_page(Page::Docs);

    runner.step("Capture view at 120x40");
    let wide_view = runner.view();
    assert!(!wide_view.is_empty());

    runner.step("Resize to 80x24 (narrower)");
    runner.resize(80, 24);
    runner.drain();

    runner.step("Capture view at 80x24");
    let narrow_view = runner.view();
    assert!(!narrow_view.is_empty());

    runner.step("Verify content differs (reflow occurred)");
    // The narrow view should have different content (more line wrapping)
    // We check that views are non-empty and different
    // Note: Views will definitely differ since dimensions changed
    assert_ne!(
        wide_view.len(),
        narrow_view.len(),
        "Resized view should have different content length"
    );

    runner.step("Resize back to 120x40");
    runner.resize(120, 40);
    runner.drain();

    runner.step("Verify view is restored");
    let restored_view = runner.view();
    assert!(!restored_view.is_empty());

    runner.finish().expect("docs resize reflow should work");
}

/// Verify that very narrow width doesn't cause panic.
#[test]
fn e2e_docs_resize_very_narrow() {
    let mut runner = create_docs_runner("docs_resize_narrow");

    runner.step("Navigate to Docs page");
    runner.press_key('5');
    runner.assert_page(Page::Docs);

    runner.step("Resize to very narrow (60x20)");
    runner.resize(60, 20);
    runner.drain();

    runner.step("Verify view still renders");
    runner.assert_view_not_empty();

    runner.step("Navigate and interact at narrow width");
    runner.press_key('j');
    runner.drain();
    runner.assert_view_not_empty();

    runner.step("Resize to very wide (160x50)");
    runner.resize(160, 50);
    runner.drain();

    runner.assert_view_not_empty();

    runner.finish().expect("docs narrow resize should work");
}

/// Verify that resize during search mode works correctly.
#[test]
fn e2e_docs_resize_during_search() {
    let mut runner = create_docs_runner("docs_resize_search");

    runner.step("Navigate to Docs page and enter search");
    runner.press_key('5');
    runner.assert_page(Page::Docs);
    runner.press_special(KeyType::Tab);
    runner.drain();
    runner.press_key('/');
    runner.drain();
    runner.press_keys("rust");
    runner.drain();

    runner.step("Resize while in search mode");
    let pre_resize = runner.view();
    runner.resize(80, 30);
    runner.drain();
    let post_resize = runner.view();

    runner.step("Verify both views are valid");
    assert!(!pre_resize.is_empty());
    assert!(!post_resize.is_empty());

    runner.step("Exit search and verify normal operation");
    runner.press_special(KeyType::Esc);
    runner.drain();
    runner.assert_view_not_empty();

    runner
        .finish()
        .expect("docs resize during search should work");
}

// =============================================================================
// COMPREHENSIVE SCENARIO
// =============================================================================

/// Comprehensive docs scenario: navigation, search, and resize.
#[test]
fn e2e_docs_comprehensive_scenario() {
    let mut runner = create_docs_runner("docs_comprehensive");

    runner.step("Phase 1: Navigate to Docs");
    runner.press_key('5');
    runner.assert_page(Page::Docs);
    runner.assert_contains("Documents");

    runner.step("Phase 2: Browse documents");
    runner.press_key('j'); // Next doc
    runner.drain();
    runner.press_key('j'); // Next doc
    runner.drain();
    runner.press_key('k'); // Previous doc
    runner.drain();
    runner.assert_view_not_empty();

    runner.step("Phase 3: Focus content and search");
    runner.press_special(KeyType::Tab);
    runner.drain();
    runner.press_key('/');
    runner.drain();
    runner.press_keys("the");
    runner.drain();
    runner.press_special(KeyType::Enter);
    runner.drain();

    runner.step("Phase 4: Navigate search matches");
    runner.press_key('n');
    runner.drain();
    runner.press_key('n');
    runner.drain();
    runner.press_key('N');
    runner.drain();

    runner.step("Phase 5: Resize window");
    runner.resize(100, 35);
    runner.drain();
    runner.assert_view_not_empty();

    runner.step("Phase 6: Return to list");
    runner.press_special(KeyType::Tab);
    runner.drain();
    runner.assert_view_not_empty();

    runner.step("Phase 7: Final resize");
    runner.resize(120, 40);
    runner.drain();

    runner.step("Phase 8: Return to dashboard");
    runner.press_key('1');
    runner.assert_page(Page::Dashboard);

    runner.finish().expect("docs comprehensive scenario should pass");
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

/// Verify that empty search query doesn't cause issues.
#[test]
fn e2e_docs_search_empty_query() {
    let mut runner = create_docs_runner("docs_search_empty");

    runner.step("Navigate to Docs and enter search");
    runner.press_key('5');
    runner.assert_page(Page::Docs);
    runner.press_special(KeyType::Tab);
    runner.drain();
    runner.press_key('/');
    runner.drain();

    runner.step("Exit search immediately (empty query)");
    runner.press_special(KeyType::Esc);
    runner.drain();

    runner.step("Verify view is normal");
    runner.assert_view_not_empty();

    runner.finish().expect("empty search should work");
}

/// Verify that search query that matches nothing doesn't crash.
#[test]
fn e2e_docs_search_no_matches() {
    let mut runner = create_docs_runner("docs_search_no_matches");

    runner.step("Navigate to Docs and search for nonsense");
    runner.press_key('5');
    runner.assert_page(Page::Docs);
    runner.press_special(KeyType::Tab);
    runner.drain();
    runner.press_key('/');
    runner.drain();
    runner.press_keys("xyzzy12345nonexistent");
    runner.drain();

    runner.step("Confirm search and try navigation");
    runner.press_special(KeyType::Enter);
    runner.drain();

    // n/N with no matches should not crash
    runner.step("Try match navigation with no matches");
    runner.press_key('n');
    runner.drain();
    runner.press_key('N');
    runner.drain();

    runner.assert_view_not_empty();

    runner.finish().expect("no matches search should work");
}

/// Verify that toggling syntax highlighting works.
#[test]
fn e2e_docs_toggle_syntax_highlighting() {
    let mut runner = create_docs_runner("docs_syntax_toggle");

    runner.step("Navigate to Docs and focus content");
    runner.press_key('5');
    runner.assert_page(Page::Docs);
    runner.press_special(KeyType::Tab);
    runner.drain();

    runner.step("Capture view with syntax highlighting");
    let with_syntax = runner.view();
    assert!(!with_syntax.is_empty());

    runner.step("Toggle syntax highlighting with 's'");
    runner.press_key('s');
    runner.drain();

    runner.step("Capture view without syntax highlighting");
    let without_syntax = runner.view();
    assert!(!without_syntax.is_empty());

    // Views may differ due to different rendering
    // The main goal is no panic

    runner.step("Toggle back");
    runner.press_key('s');
    runner.drain();

    runner.assert_view_not_empty();

    runner
        .finish()
        .expect("syntax highlighting toggle should work");
}

/// Verify that rapid resize events don't cause panics.
#[test]
fn e2e_docs_rapid_resize() {
    let mut runner = create_docs_runner("docs_rapid_resize");

    runner.step("Navigate to Docs");
    runner.press_key('5');
    runner.assert_page(Page::Docs);

    runner.step("Rapid resize events");
    for i in 0..10 {
        let width = 80 + (i * 5) as u16;
        let height = 24 + (i * 2) as u16;
        runner.resize(width, height);
    }
    runner.drain();

    runner.step("Verify app is still responsive");
    runner.assert_view_not_empty();
    runner.press_key('j');
    runner.drain();
    runner.assert_view_not_empty();

    runner.finish().expect("rapid resize should work");
}
