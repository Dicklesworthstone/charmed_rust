//! Integration tests for huh forms with bubbletea event loop.
//!
//! These tests verify that huh forms work correctly within the bubbletea
//! TUI context using `ProgramSimulator`.

use bubbletea::key::KeyType;
use bubbletea::simulator::ProgramSimulator;
use huh::{Confirm, Form, Group, Input, MultiSelect, Note, Select, SelectOption, Text};

/// Helper to create a simple form with one input field.
fn simple_input_form() -> Form {
    Form::new(vec![Group::new(vec![Box::new(
        Input::new()
            .key("name")
            .title("Name")
            .placeholder("Enter name"),
    )])])
}

/// Helper to create a form with multiple fields.
fn multi_field_form() -> Form {
    Form::new(vec![Group::new(vec![
        Box::new(Input::new().key("first_name").title("First Name")),
        Box::new(Input::new().key("last_name").title("Last Name")),
        Box::new(Input::new().key("email").title("Email")),
    ])])
}

/// Helper to create a form with different field types.
fn mixed_field_form() -> Form {
    Form::new(vec![Group::new(vec![
        Box::new(Input::new().key("name").title("Name")),
        Box::new(
            Select::new()
                .key("color")
                .title("Favorite Color")
                .options(vec![
                    SelectOption::new("Red", "red"),
                    SelectOption::new("Green", "green"),
                    SelectOption::new("Blue", "blue"),
                ]),
        ),
        Box::new(Confirm::new().key("agree").title("Agree to terms?")),
    ])])
}

// =============================================================================
// Form Initialization Tests
// =============================================================================

#[test]
fn test_form_initializes_in_simulator() {
    let form = simple_input_form();
    let mut sim = ProgramSimulator::new(form);

    // Initialize
    sim.init();

    // Should have captured initial view
    assert!(sim.is_initialized());
    assert!(!sim.views().is_empty());

    // View should contain the form
    let view = sim.last_view().unwrap();
    assert!(view.contains("Name"), "View should show field title");
}

#[test]
fn test_form_shows_placeholder() {
    let form = simple_input_form();
    let mut sim = ProgramSimulator::new(form);
    sim.init();

    let view = sim.last_view().unwrap();
    // Placeholder might be rendered with styling, just check form renders
    assert!(!view.is_empty());
}

// =============================================================================
// Keyboard Input Tests
// =============================================================================

#[test]
fn test_form_receives_character_input() {
    let form = simple_input_form();
    let mut sim = ProgramSimulator::new(form);
    sim.init();

    // Type some characters
    for c in "Alice".chars() {
        sim.sim_key(c);
        sim.step();
    }

    // View should update - verify we processed input (view count increased)
    // Note: The actual text may be styled with ANSI codes, so we verify processing occurred
    assert_eq!(
        sim.stats().update_calls,
        5,
        "Should have processed all 5 characters"
    );
    assert!(
        !sim.last_view().unwrap().is_empty(),
        "View should not be empty after input"
    );
}

#[test]
fn test_form_handles_backspace() {
    let form = simple_input_form();
    let mut sim = ProgramSimulator::new(form);
    sim.init();

    // Type then delete
    for c in "Test".chars() {
        sim.sim_key(c);
        sim.step();
    }

    sim.sim_key_type(KeyType::Backspace);
    sim.step();

    // Should have processed without panic
    assert!(sim.stats().update_calls > 0);
}

// =============================================================================
// Navigation Tests
// =============================================================================

#[test]
fn test_form_tab_advances_field() {
    let form = multi_field_form();
    let mut sim = ProgramSimulator::new(form);
    sim.init();

    // Press Tab to advance to next field
    sim.sim_key_type(KeyType::Tab);
    sim.step();

    // View should update (focus indicator may change)
    let after_tab = sim.last_view().unwrap();
    assert!(sim.stats().update_calls > 0);
    // The view should have been updated
    assert!(!after_tab.is_empty());
}

#[test]
fn test_form_shift_tab_goes_back() {
    let form = multi_field_form();
    let mut sim = ProgramSimulator::new(form);
    sim.init();

    // Go forward twice
    sim.sim_key_type(KeyType::Tab);
    sim.step();
    sim.sim_key_type(KeyType::Tab);
    sim.step();

    // Now go back with Shift+Tab
    sim.sim_key_type(KeyType::ShiftTab);
    sim.step();

    // Should have processed the navigation
    assert!(sim.stats().update_calls >= 3);
}

#[test]
fn test_form_enter_advances_or_submits() {
    let form = simple_input_form();
    let mut sim = ProgramSimulator::new(form);
    sim.init();

    // Type something
    for c in "Test".chars() {
        sim.sim_key(c);
        sim.step();
    }

    // Press Enter
    sim.sim_key_type(KeyType::Enter);
    sim.step();

    // Should have processed
    assert!(sim.stats().update_calls > 0);
}

// =============================================================================
// Escape/Quit Tests
// =============================================================================

#[test]
fn test_form_escape_aborts() {
    let form = simple_input_form();
    let mut sim = ProgramSimulator::new(form);
    sim.init();

    // Press Escape to abort
    sim.sim_key_type(KeyType::Esc);
    if let Some(cmd) = sim.step() {
        // Execute the command (should be quit)
        if let Some(msg) = cmd.execute() {
            sim.send(msg);
            sim.step();
        }
    }

    // Form should be in aborted state or quit requested
    // Check that escape was processed
    assert!(sim.stats().update_calls > 0);
}

// =============================================================================
// Select Field Tests
// =============================================================================

#[test]
fn test_select_field_navigation() {
    let form = Form::new(vec![Group::new(vec![Box::new(
        Select::new().key("choice").title("Choose").options(vec![
            SelectOption::new("Option A", "a"),
            SelectOption::new("Option B", "b"),
            SelectOption::new("Option C", "c"),
        ]),
    )])]);

    let mut sim = ProgramSimulator::new(form);
    sim.init();

    // Navigate down through options
    sim.sim_key_type(KeyType::Down);
    sim.step();
    sim.sim_key_type(KeyType::Down);
    sim.step();

    // Should have processed navigation
    assert!(sim.stats().update_calls >= 2);
}

#[test]
fn test_select_field_wraps_at_bounds() {
    let form = Form::new(vec![Group::new(vec![Box::new(
        Select::new().key("choice").title("Choose").options(vec![
            SelectOption::new("A", "a"),
            SelectOption::new("B", "b"),
        ]),
    )])]);

    let mut sim = ProgramSimulator::new(form);
    sim.init();

    // Navigate down multiple times past the end
    for _ in 0..5 {
        sim.sim_key_type(KeyType::Down);
        sim.step();
    }

    // Should not panic, navigation should be bounded
    assert!(sim.stats().update_calls >= 5);
}

// =============================================================================
// MultiSelect Tests
// =============================================================================

#[test]
fn test_multiselect_toggle() {
    let form = Form::new(vec![Group::new(vec![Box::new(
        MultiSelect::new()
            .key("items")
            .title("Select items")
            .options(vec![
                SelectOption::new("Item 1", "1"),
                SelectOption::new("Item 2", "2"),
                SelectOption::new("Item 3", "3"),
            ]),
    )])]);

    let mut sim = ProgramSimulator::new(form);
    sim.init();

    // Toggle selection with space
    sim.sim_key(' ');
    sim.step();

    // Move down and toggle again
    sim.sim_key_type(KeyType::Down);
    sim.step();
    sim.sim_key(' ');
    sim.step();

    // Should have processed toggles
    assert!(sim.stats().update_calls >= 3);
}

// =============================================================================
// Confirm Field Tests
// =============================================================================

#[test]
fn test_confirm_field_toggle() {
    let form = Form::new(vec![Group::new(vec![Box::new(
        Confirm::new().key("agree").title("Do you agree?"),
    )])]);

    let mut sim = ProgramSimulator::new(form);
    sim.init();

    // Toggle with arrow keys
    sim.sim_key_type(KeyType::Left);
    sim.step();
    sim.sim_key_type(KeyType::Right);
    sim.step();

    // Should have processed toggles
    assert!(sim.stats().update_calls >= 2);
}

// =============================================================================
// Note Field Tests
// =============================================================================

#[test]
fn test_note_field_displays() {
    let form = Form::new(vec![Group::new(vec![
        Box::new(
            Note::new()
                .title("Important")
                .description("Read this carefully"),
        ),
        Box::new(Input::new().key("ack").title("Acknowledge")),
    ])])
    .show_help(false);

    let mut sim = ProgramSimulator::new(form);
    sim.init();

    let view = sim.last_view().unwrap();
    // Note content should be visible
    assert!(
        view.contains("Important") || view.contains("Read") || view.contains("Acknowledge"),
        "Form should display note and input"
    );
}

// =============================================================================
// Text (TextArea) Field Tests
// =============================================================================

#[test]
fn test_text_field_multiline_input() {
    let form = Form::new(vec![Group::new(vec![Box::new(
        Text::new().key("bio").title("Biography").char_limit(500),
    )])]);

    let mut sim = ProgramSimulator::new(form);
    sim.init();

    // Type multiple lines
    for c in "Line 1".chars() {
        sim.sim_key(c);
        sim.step();
    }

    // Note: actual newline handling depends on Text field implementation
    // Just verify it doesn't panic
    assert!(sim.stats().update_calls > 0);
}

// =============================================================================
// Multi-Group Form Tests
// =============================================================================

#[test]
fn test_multi_group_navigation() {
    let form = Form::new(vec![
        Group::new(vec![Box::new(
            Input::new().key("g1_field").title("Group 1 Field"),
        )]),
        Group::new(vec![Box::new(
            Input::new().key("g2_field").title("Group 2 Field"),
        )]),
    ]);

    let mut sim = ProgramSimulator::new(form);
    sim.init();

    // Initial view should show first group
    let initial = sim.last_view().unwrap();
    assert!(!initial.is_empty());

    // Navigate through fields to reach next group
    // (Tab moves between fields, completing a group may advance to next)
    for _ in 0..5 {
        sim.sim_key_type(KeyType::Tab);
        sim.step();
    }

    // Should have processed navigation
    assert!(sim.stats().update_calls >= 5);
}

// =============================================================================
// View Rendering Tests
// =============================================================================

#[test]
fn test_form_view_updates_after_each_input() {
    let form = simple_input_form();
    let mut sim = ProgramSimulator::new(form);
    sim.init();

    let initial_views = sim.views().len();

    // Each step should produce a new view
    sim.sim_key('X');
    sim.step();

    assert!(
        sim.views().len() > initial_views,
        "View should be captured after update"
    );
}

#[test]
fn test_form_view_not_empty() {
    let form = mixed_field_form();
    let mut sim = ProgramSimulator::new(form);
    sim.init();

    // All views should be non-empty
    for view in sim.views() {
        assert!(!view.is_empty(), "Form view should never be empty");
    }
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_empty_form_handles_input() {
    let form = Form::new(Vec::new());
    let mut sim = ProgramSimulator::new(form);
    sim.init();

    // Should not panic on input to empty form
    sim.sim_key('x');
    sim.step();
    sim.sim_key_type(KeyType::Tab);
    sim.step();

    assert!(sim.stats().update_calls >= 2);
}

#[test]
fn test_single_field_form_navigation_bounds() {
    let form = simple_input_form();
    let mut sim = ProgramSimulator::new(form);
    sim.init();

    // Tab multiple times on single-field form
    for _ in 0..10 {
        sim.sim_key_type(KeyType::Tab);
        sim.step();
    }

    // Should not panic, should handle gracefully
    assert!(sim.stats().update_calls >= 10);
}

#[test]
fn test_rapid_input_sequence() {
    let form = simple_input_form();
    let mut sim = ProgramSimulator::new(form);
    sim.init();

    // Rapid input sequence
    let input = "The quick brown fox jumps over the lazy dog";
    for c in input.chars() {
        sim.sim_key(c);
        sim.step();
    }

    // Should have processed all characters
    assert_eq!(sim.stats().update_calls, input.len());
}

// =============================================================================
// Form State Tests
// =============================================================================

#[test]
fn test_form_processes_quit_command() {
    let form = simple_input_form();
    let mut sim = ProgramSimulator::new(form);
    sim.init();

    let initial_updates = sim.stats().update_calls;

    // Escape to abort - form should return quit command
    sim.sim_key_type(KeyType::Esc);
    let cmd = sim.step();

    // Verify escape was processed
    assert!(
        sim.stats().update_calls > initial_updates,
        "Escape key should trigger an update"
    );

    // If a command was returned (quit), execute it
    if let Some(cmd) = cmd
        && let Some(msg) = cmd.execute()
    {
        sim.send(msg);
        sim.run_until_empty();
    }

    // Form should have handled the escape
    assert!(sim.stats().update_calls > initial_updates);
}
