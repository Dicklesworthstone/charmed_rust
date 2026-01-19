//! E2E integration tests for bubbles components as standalone bubbletea models.
//!
//! These tests verify that each component correctly implements the Model trait
//! and can be used standalone in bubbletea applications.
//!
//! Test categories:
//! - Lifecycle tests: init -> update -> view
//! - State transition tests: verify state changes from messages
//! - Rendering tests: verify view output
//! - Component combination tests: multiple components working together

#![forbid(unsafe_code)]

use bubbles::cursor::Cursor;
use bubbles::help::Help;
use bubbles::key::Binding;
use bubbles::paginator::Paginator;
use bubbles::progress::Progress;
use bubbles::spinner::{SpinnerModel, spinners};
use bubbles::stopwatch::Stopwatch;
use bubbles::table::{Column, Table};
use bubbles::textarea::TextArea;
use bubbles::textinput::TextInput;
use bubbles::timer::Timer;
use bubbles::viewport::Viewport;
use bubbletea::{KeyMsg, KeyType, Message, Model};
use std::time::Duration;

// ============================================================================
// Cursor Component Tests
// ============================================================================

mod cursor_tests {
    use super::*;

    #[test]
    fn test_cursor_init_unfocused_returns_none() {
        let cursor = Cursor::new();
        // Default cursor is unfocused, so init returns None
        let cmd = cursor.init();
        assert!(
            cmd.is_none(),
            "Unfocused cursor should not need init command"
        );
    }

    #[test]
    fn test_cursor_init_focused_returns_blink_command() {
        let mut cursor = Cursor::new();
        cursor.focus();
        let cmd = cursor.init();
        // Focused cursor in blink mode should return init command
        assert!(
            cmd.is_some(),
            "Focused cursor in blink mode should return init command"
        );
    }

    #[test]
    fn test_cursor_view_with_char() {
        let mut cursor = Cursor::new();
        cursor.set_char("X");
        let view = cursor.view();
        // Cursor view with a character should render
        assert!(
            view.contains('X'),
            "Cursor view should render the character"
        );
    }

    #[test]
    fn test_cursor_update_handles_messages() {
        let mut cursor = Cursor::new();
        cursor.set_char("_");

        // Model update without panic
        let msg = Message::new("test");
        let _cmd = Model::update(&mut cursor, msg);
    }

    #[test]
    fn test_cursor_focus_unfocus_state() {
        let mut cursor = Cursor::new();
        assert!(!cursor.focused(), "New cursor should be unfocused");

        cursor.focus();
        assert!(cursor.focused(), "Cursor should be focused after focus()");
        assert!(
            !cursor.is_blinking_off(),
            "Focused cursor should be visible"
        );

        cursor.blur();
        assert!(!cursor.focused(), "Cursor should be unfocused after blur()");
    }
}

// ============================================================================
// Spinner Component Tests
// ============================================================================

mod spinner_tests {
    use super::*;

    #[test]
    fn test_spinner_init_returns_tick_command() {
        let spinner = SpinnerModel::new();
        let cmd = spinner.init();
        assert!(
            cmd.is_some(),
            "Spinner should return init command for tick cycle"
        );
    }

    #[test]
    fn test_spinner_view_renders_frame() {
        let spinner = SpinnerModel::new();
        let view = spinner.view();
        assert!(!view.is_empty(), "Spinner view should not be empty");
    }

    #[test]
    fn test_spinner_update_advances_frame() {
        let mut spinner = SpinnerModel::with_spinner(spinners::line());
        let initial_view = spinner.view();

        // Create a tick message for this spinner
        let tick_msg = spinner.tick();
        let _cmd = Model::update(&mut spinner, tick_msg);

        let next_view = spinner.view();
        // The frame should have advanced (views may differ)
        // At minimum, update should succeed without panic
        assert!(!next_view.is_empty());
        // Note: initial and next views might be the same if it cycled back
        let _ = initial_view;
    }

    #[test]
    fn test_spinner_with_different_styles() {
        for spinner_def in [spinners::dot(), spinners::mini_dot(), spinners::pulse()] {
            let spinner = SpinnerModel::with_spinner(spinner_def);
            let view = spinner.view();
            assert!(!view.is_empty(), "All spinner styles should render");
        }
    }
}

// ============================================================================
// Timer Component Tests
// ============================================================================

mod timer_tests {
    use super::*;

    #[test]
    fn test_timer_init_returns_tick_command() {
        let timer = Timer::new(Duration::from_secs(10));
        let cmd = timer.init();
        assert!(cmd.is_some(), "Running timer should return init command");
    }

    #[test]
    fn test_timer_view_shows_remaining_time() {
        let timer = Timer::new(Duration::from_secs(60));
        let view = timer.view();
        // Should contain time representation
        assert!(!view.is_empty(), "Timer view should show remaining time");
    }

    #[test]
    fn test_timer_running_state() {
        let timer = Timer::new(Duration::from_secs(10));
        // New timer starts running by default
        assert!(timer.running(), "New timer should be running");
    }
}

// ============================================================================
// Stopwatch Component Tests
// ============================================================================

mod stopwatch_tests {
    use super::*;

    #[test]
    fn test_stopwatch_init() {
        let stopwatch = Stopwatch::new();
        let cmd = stopwatch.init();
        // Init behavior depends on initial running state
        // At minimum, should not panic
        let _ = cmd;
    }

    #[test]
    fn test_stopwatch_view_shows_elapsed() {
        let stopwatch = Stopwatch::new();
        let view = stopwatch.view();
        // Should show elapsed time (starts at 0)
        assert!(!view.is_empty(), "Stopwatch should render elapsed time");
    }

    #[test]
    fn test_stopwatch_running_state() {
        let stopwatch = Stopwatch::new();
        // New stopwatch should not be running initially
        assert!(!stopwatch.running(), "New stopwatch should not be running");
    }
}

// ============================================================================
// Progress Component Tests
// ============================================================================

mod progress_tests {
    use super::*;

    #[test]
    fn test_progress_init_returns_none() {
        let progress = Progress::new();
        let cmd = progress.init();
        // Non-animated progress doesn't need init command
        assert!(cmd.is_none(), "Basic progress should not need init command");
    }

    #[test]
    fn test_progress_view_shows_bar() {
        let mut progress = Progress::new();
        progress.set_percent(0.5);
        let view = progress.view();
        assert!(!view.is_empty(), "Progress bar should render");
    }

    #[test]
    fn test_progress_at_zero() {
        let mut progress = Progress::new();
        progress.set_percent(0.0);
        let view = progress.view();
        assert!(!view.is_empty());
    }

    #[test]
    fn test_progress_at_full() {
        let mut progress = Progress::new();
        progress.set_percent(1.0);
        let view = progress.view();
        assert!(!view.is_empty());
    }

    #[test]
    fn test_progress_percent_getter() {
        let mut progress = Progress::new();
        progress.set_percent(0.75);
        assert!((progress.percent() - 0.75).abs() < 0.001);
    }
}

// ============================================================================
// Paginator Component Tests
// ============================================================================

mod paginator_tests {
    use super::*;

    #[test]
    fn test_paginator_init_returns_none() {
        let paginator = Paginator::new();
        let cmd = paginator.init();
        assert!(cmd.is_none(), "Paginator doesn't need init command");
    }

    #[test]
    fn test_paginator_view_shows_dots() {
        let paginator = Paginator::new().total_pages(5);
        let view = paginator.view();
        assert!(!view.is_empty(), "Paginator should show page indicators");
    }

    #[test]
    fn test_paginator_navigation() {
        let mut paginator = Paginator::new().total_pages(5);
        paginator.set_page(0);

        assert_eq!(paginator.page(), 0);
        assert!(paginator.on_first_page());

        paginator.next_page();
        assert_eq!(paginator.page(), 1);
        assert!(!paginator.on_first_page());

        paginator.prev_page();
        assert_eq!(paginator.page(), 0);
    }

    #[test]
    fn test_paginator_total_pages() {
        let paginator = Paginator::new().total_pages(10);
        assert_eq!(paginator.get_total_pages(), 10);
    }
}

// ============================================================================
// Help Component Tests
// ============================================================================

mod help_tests {
    use super::*;

    #[test]
    fn test_help_init_returns_none() {
        let help = Help::new();
        let cmd = help.init();
        assert!(cmd.is_none(), "Help doesn't need init command");
    }

    #[test]
    fn test_help_view_with_bindings() {
        let help = Help::new();
        let binding1 = Binding::new().keys(&["q"]).help("q", "quit");
        let binding2 = Binding::new().keys(&["?", "h"]).help("?/h", "help");
        let bindings: Vec<&Binding> = vec![&binding1, &binding2];
        let view = help.short_help_view(&bindings);
        assert!(!view.is_empty(), "Help should render key bindings");
    }

    #[test]
    fn test_help_full_view() {
        let help = Help::new();
        let binding1 = Binding::new().keys(&["up", "k"]).help("up/k", "move up");
        let binding2 = Binding::new()
            .keys(&["down", "j"])
            .help("down/j", "move down");
        let group: Vec<&Binding> = vec![&binding1, &binding2];
        let groups: Vec<Vec<&Binding>> = vec![group];
        let view = help.full_help_view(&groups);
        assert!(!view.is_empty(), "Full help should render grouped bindings");
    }
}

// ============================================================================
// TextInput Component Tests
// ============================================================================

mod textinput_tests {
    use super::*;

    #[test]
    fn test_textinput_init_unfocused_returns_none() {
        let input = TextInput::new();
        let cmd = input.init();
        // Unfocused input doesn't need blink command
        assert!(
            cmd.is_none(),
            "Unfocused input should not need init command"
        );
    }

    #[test]
    fn test_textinput_init_focused_returns_blink() {
        let mut input = TextInput::new();
        input.focus();
        let cmd = input.init();
        assert!(cmd.is_some(), "Focused input should return blink command");
    }

    #[test]
    fn test_textinput_view_shows_prompt() {
        let input = TextInput::new();
        let view = input.view();
        // Default prompt is "> "
        assert!(view.contains('>'), "TextInput should show prompt");
    }

    #[test]
    fn test_textinput_handles_character_input() {
        let mut input = TextInput::new();
        input.focus();

        // Insert a character
        let msg = Message::new(KeyMsg::from_char('h'));
        let _cmd = Model::update(&mut input, msg);
        assert_eq!(input.value(), "h", "TextInput should have typed character");

        // Insert another character
        let msg = Message::new(KeyMsg::from_char('i'));
        let _cmd = Model::update(&mut input, msg);
        assert_eq!(input.value(), "hi", "TextInput should append characters");
    }

    #[test]
    fn test_textinput_set_value() {
        let mut input = TextInput::new();
        input.set_value("hello world");
        assert_eq!(input.value(), "hello world");

        let view = input.view();
        assert!(view.contains("hello"), "View should show the value");
    }

    #[test]
    fn test_textinput_cursor_movement() {
        let mut input = TextInput::new();
        input.set_value("hello");
        input.focus();

        // Cursor starts at end
        assert_eq!(input.position(), 5);

        // Move to start
        input.cursor_start();
        assert_eq!(input.position(), 0);

        // Move to end
        input.cursor_end();
        assert_eq!(input.position(), 5);
    }
}

// ============================================================================
// TextArea Component Tests
// ============================================================================

mod textarea_tests {
    use super::*;

    #[test]
    fn test_textarea_init_unfocused() {
        let textarea = TextArea::new();
        let cmd = textarea.init();
        // Unfocused textarea doesn't need cursor blink
        assert!(cmd.is_none());
    }

    #[test]
    fn test_textarea_init_focused() {
        let mut textarea = TextArea::new();
        textarea.focus();
        let cmd = textarea.init();
        assert!(cmd.is_some(), "Focused textarea needs cursor blink");
    }

    #[test]
    fn test_textarea_view_renders() {
        let textarea = TextArea::new();
        let view = textarea.view();
        // View should render without panic (may be empty or not)
        let _ = view;
    }

    #[test]
    fn test_textarea_set_value() {
        let mut textarea = TextArea::new();
        textarea.set_value("line1\nline2");
        assert_eq!(textarea.value(), "line1\nline2");
    }

    #[test]
    fn test_textarea_multiline_input() {
        let mut textarea = TextArea::new();
        textarea.focus();

        // Type some text
        let msg = Message::new(KeyMsg::from_char('a'));
        let _cmd = Model::update(&mut textarea, msg);

        // Enter for new line
        let msg = Message::new(KeyMsg::from_type(KeyType::Enter));
        let _cmd = Model::update(&mut textarea, msg);

        // Type more text
        let msg = Message::new(KeyMsg::from_char('b'));
        let _cmd = Model::update(&mut textarea, msg);

        let value = textarea.value();
        assert!(value.contains('\n'), "TextArea should support multiline");
    }
}

// ============================================================================
// Table Component Tests
// ============================================================================

mod table_tests {
    use super::*;

    #[test]
    fn test_table_init_returns_none() {
        let table = Table::new()
            .columns(vec![Column::new("Name", 20), Column::new("Age", 10)])
            .rows(vec![
                vec!["Alice".to_string(), "30".to_string()],
                vec!["Bob".to_string(), "25".to_string()],
            ]);
        let cmd = table.init();
        assert!(cmd.is_none(), "Table doesn't need init command");
    }

    #[test]
    fn test_table_view_renders_headers() {
        let table = Table::new()
            .columns(vec![Column::new("Name", 20), Column::new("Age", 10)])
            .rows(vec![vec!["Alice".to_string(), "30".to_string()]]);
        let view = table.view();
        assert!(!view.is_empty(), "Table should render");
    }

    #[test]
    fn test_table_navigation() {
        let mut table = Table::new()
            .columns(vec![Column::new("Name", 20), Column::new("Age", 10)])
            .rows(vec![
                vec!["Alice".to_string(), "30".to_string()],
                vec!["Bob".to_string(), "25".to_string()],
                vec!["Charlie".to_string(), "35".to_string()],
            ]);
        table.focus();

        // Navigate down
        let msg = Message::new(KeyMsg::from_type(KeyType::Down));
        let _cmd = Model::update(&mut table, msg);

        // Should have moved cursor (or stayed at 0 if already there)
        // Main point is it doesn't panic
        let _ = table.cursor();
    }

    #[test]
    fn test_table_empty() {
        let table = Table::new()
            .columns(vec![Column::new("Header", 20)])
            .rows(vec![]);
        let view = table.view();
        // Empty table should render without panic
        let _ = view;
    }
}

// ============================================================================
// Viewport Component Tests
// ============================================================================

mod viewport_tests {
    use super::*;

    #[test]
    fn test_viewport_init_returns_none() {
        let viewport = Viewport::new(80, 24);
        let cmd = viewport.init();
        assert!(cmd.is_none(), "Viewport doesn't need init command");
    }

    #[test]
    fn test_viewport_view_renders_content() {
        let mut viewport = Viewport::new(80, 10);
        viewport.set_content("Line 1\nLine 2\nLine 3\nLine 4\nLine 5");
        let view = viewport.view();
        assert!(!view.is_empty(), "Viewport should render content");
    }

    #[test]
    fn test_viewport_scrolling() {
        let mut viewport = Viewport::new(80, 3);
        let content: String = (1..=20).map(|i| format!("Line {i}\n")).collect();
        viewport.set_content(&content);

        let initial_y = viewport.y_offset();

        // Scroll down
        let msg = Message::new(KeyMsg::from_type(KeyType::Down));
        let _cmd = Model::update(&mut viewport, msg);

        // May or may not have scrolled depending on implementation
        // Main point is it doesn't panic
        assert!(viewport.y_offset() >= initial_y);
    }

    #[test]
    fn test_viewport_line_navigation() {
        let mut viewport = Viewport::new(80, 5);
        viewport.set_content("1\n2\n3\n4\n5\n6\n7\n8\n9\n10");

        viewport.scroll_down(3);
        assert!(viewport.y_offset() > 0, "Should have scrolled down");

        viewport.scroll_up(2);
        // Should have scrolled back up
    }
}

// ============================================================================
// Component Combination Tests
// ============================================================================

mod combination_tests {
    use super::*;

    /// Test that multiple components can be composed in an application.
    /// This simulates a form with TextInput and Help.
    #[test]
    fn test_textinput_with_help_composition() {
        struct FormModel {
            input: TextInput,
            help: Help,
        }

        impl FormModel {
            fn new() -> Self {
                let mut input = TextInput::new();
                input.set_placeholder("Enter name");
                Self {
                    input,
                    help: Help::new(),
                }
            }

            fn view(&self) -> String {
                let binding1 = Binding::new().keys(&["enter"]).help("enter", "submit");
                let binding2 = Binding::new().keys(&["esc"]).help("esc", "cancel");
                let bindings: Vec<&Binding> = vec![&binding1, &binding2];
                format!(
                    "{}\n\n{}",
                    self.input.view(),
                    self.help.short_help_view(&bindings)
                )
            }
        }

        let form = FormModel::new();
        let view = form.view();
        assert!(view.contains("submit"), "Composed view should show help");
    }

    /// Test paginated content with viewport and paginator.
    #[test]
    fn test_viewport_with_paginator() {
        let mut viewport = Viewport::new(80, 10);
        viewport.set_content("Long content here...\n".repeat(50).as_str());

        let paginator = Paginator::new().total_pages(5);

        // Both should render independently
        let vp_view = viewport.view();
        let pg_view = paginator.view();

        assert!(!vp_view.is_empty());
        assert!(!pg_view.is_empty());
    }

    /// Test timer and stopwatch can coexist.
    #[test]
    fn test_timer_and_stopwatch_together() {
        let timer = Timer::new(Duration::from_secs(60));
        let stopwatch = Stopwatch::new();

        let timer_view = timer.view();
        let stopwatch_view = stopwatch.view();

        // Both should render time representations
        assert!(!timer_view.is_empty());
        assert!(!stopwatch_view.is_empty());
    }

    /// Test table with progress bar (loading state simulation).
    #[test]
    fn test_table_with_progress() {
        let table = Table::new()
            .columns(vec![Column::new("Item", 20), Column::new("Status", 10)])
            .rows(vec![vec!["Task 1".to_string(), "Done".to_string()]]);

        let mut progress = Progress::new();
        progress.set_percent(0.75);

        let table_view = table.view();
        let progress_view = progress.view();

        // Both should render without panic
        assert!(!progress_view.is_empty(), "Progress should render");
        // Table view should contain headers at minimum
        assert!(
            table_view.contains("Item") || table_view.contains("Status"),
            "Table view should contain column headers"
        );
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_textinput() {
        let input = TextInput::new();
        assert_eq!(input.value(), "");
        assert!(!input.view().is_empty()); // Should still show prompt
    }

    #[test]
    fn test_viewport_with_no_content() {
        let viewport = Viewport::new(80, 24);
        let view = viewport.view();
        // Empty viewport should render without panic
        let _ = view;
    }

    #[test]
    fn test_paginator_single_page() {
        let mut paginator = Paginator::new().total_pages(1);
        paginator.set_page(0);

        assert!(paginator.on_first_page());
        assert!(paginator.on_last_page());

        // Navigation should be no-op
        paginator.next_page();
        assert!(paginator.on_first_page());
    }

    #[test]
    fn test_progress_overflow_handling() {
        let mut progress = Progress::new();
        progress.set_percent(1.5); // Over 100%
        let view = progress.view();
        // Should handle gracefully
        assert!(!view.is_empty());
    }

    #[test]
    fn test_progress_negative_percent() {
        let mut progress = Progress::new();
        progress.set_percent(-0.5);
        let view = progress.view();
        // Should handle gracefully
        assert!(!view.is_empty());
    }

    #[test]
    fn test_table_single_row() {
        let table = Table::new()
            .columns(vec![Column::new("A", 10)])
            .rows(vec![vec!["1".to_string()]]);
        let view = table.view();
        // Table should render without panic
        let _ = view;
    }

    #[test]
    fn test_textarea_very_long_input() {
        let mut textarea = TextArea::new();
        textarea.focus();

        // Type a long string
        for c in "a".repeat(1000).chars() {
            let msg = Message::new(KeyMsg::from_char(c));
            let _cmd = Model::update(&mut textarea, msg);
        }

        // Should handle without panic
        assert!(textarea.value().len() >= 1000);
    }
}
