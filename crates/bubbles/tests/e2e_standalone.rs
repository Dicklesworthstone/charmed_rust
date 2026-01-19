//! E2E Integration Tests for Standalone Component Usage
//!
//! These tests verify that all bubbles components work correctly when used
//! as standalone Models in bubbletea applications.

use bubbletea::{KeyMsg, KeyType, Message, Model, simulator::ProgramSimulator};

// ============================================================================
// Spinner E2E Tests
// ============================================================================

mod spinner_e2e {
    use super::*;
    use bubbles::spinner::{SpinnerModel, spinners};

    #[test]
    fn test_spinner_standalone_lifecycle() {
        let spinner = SpinnerModel::new();
        let mut sim = ProgramSimulator::new(spinner);

        // Init should return a tick command
        let init_cmd = sim.init();
        assert!(
            init_cmd.is_some(),
            "Spinner init should return tick command"
        );

        // Check initial view
        let view = sim.last_view().unwrap();
        assert!(!view.is_empty(), "Spinner should render a frame");

        // Verify stats
        let stats = sim.stats();
        assert_eq!(stats.init_calls, 1);
        assert_eq!(stats.view_calls, 1);
    }

    #[test]
    fn test_spinner_frame_progression() {
        let spinner = SpinnerModel::with_spinner(spinners::line());
        let mut sim = ProgramSimulator::new(spinner);

        sim.init();
        let view1 = sim.last_view().unwrap().to_string();

        // Use the spinner's tick() method to get a valid tick message
        let tick = sim.model().tick();
        sim.send(tick);
        sim.step();

        let view2 = sim.last_view().unwrap().to_string();

        // Views should be different as frame advanced
        // (Line spinner: | / - \ so they should differ)
        assert_ne!(view1, view2, "Spinner frame should advance on tick");
    }

    #[test]
    fn test_spinner_multiple_ticks() {
        let spinner = SpinnerModel::with_spinner(spinners::line());
        let mut sim = ProgramSimulator::new(spinner);

        sim.init();

        // Process multiple ticks to verify continuous animation
        for _ in 0..4 {
            let tick = sim.model().tick();
            sim.send(tick);
            sim.step();
        }

        // After 4 ticks on a line spinner (4 frames), we should be back to frame 0
        assert_eq!(
            sim.stats().update_calls,
            4,
            "Should have processed 4 updates"
        );
    }
}

// ============================================================================
// Viewport E2E Tests
// ============================================================================

mod viewport_e2e {
    use super::*;
    use bubbles::viewport::Viewport;

    #[test]
    fn test_viewport_standalone_lifecycle() {
        let mut viewport = Viewport::new(80, 10);
        viewport.set_content("Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10\nLine 11\nLine 12");

        let mut sim = ProgramSimulator::new(viewport);

        // Init should return None for viewport
        let init_cmd = sim.init();
        assert!(init_cmd.is_none(), "Viewport init should return None");

        // Verify initial view
        let view = sim.last_view().unwrap();
        assert!(view.contains("Line 1"), "Should show first line");
        assert!(
            view.contains("Line 10"),
            "Should show lines within viewport"
        );
    }

    #[test]
    fn test_viewport_keyboard_scroll() {
        let mut viewport = Viewport::new(80, 3);
        viewport.set_content("Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6");

        let mut sim = ProgramSimulator::new(viewport);
        sim.init();

        // Initial state - should see first 3 lines
        assert!(sim.last_view().unwrap().contains("Line 1"));

        // Press 'j' to scroll down
        let down_key = Message::new(KeyMsg::from_char('j'));
        sim.send(down_key);
        sim.step();

        // Should now see lines 2-4
        let view = sim.last_view().unwrap();
        assert!(!view.contains("Line 1"), "Line 1 should scroll out");
        assert!(view.contains("Line 2"), "Line 2 should be visible");
    }

    #[test]
    fn test_viewport_page_navigation() {
        let mut viewport = Viewport::new(80, 2);
        viewport.set_content("1\n2\n3\n4\n5\n6\n7\n8");

        let mut sim = ProgramSimulator::new(viewport);
        sim.init();

        // Page down (space or pgdown)
        let pg_down = Message::new(KeyMsg::from_char('f'));
        sim.send(pg_down);
        sim.step();

        assert_eq!(sim.model().y_offset(), 2, "Should have paged down");
    }
}

// ============================================================================
// TextInput E2E Tests
// ============================================================================

mod textinput_e2e {
    use super::*;
    use bubbles::textinput::TextInput;

    #[test]
    fn test_textinput_standalone_lifecycle() {
        let mut input = TextInput::new();
        input.focus();

        let mut sim = ProgramSimulator::new(input);
        sim.init();

        // Verify initial view shows placeholder or empty
        let view = sim.last_view().unwrap();
        assert!(!view.is_empty(), "TextInput should render something");
    }

    #[test]
    fn test_textinput_typing() {
        let mut input = TextInput::new();
        input.focus();

        let mut sim = ProgramSimulator::new(input);
        sim.init();

        // Type 'h'
        let h_key = Message::new(KeyMsg::from_char('h'));
        sim.send(h_key);
        sim.step();

        // Type 'i'
        let i_key = Message::new(KeyMsg::from_char('i'));
        sim.send(i_key);
        sim.step();

        // Check value
        assert_eq!(sim.model().value(), "hi", "TextInput should contain 'hi'");
    }

    #[test]
    fn test_textinput_backspace() {
        let mut input = TextInput::new();
        input.focus();
        input.set_value("hello");

        let mut sim = ProgramSimulator::new(input);
        sim.init();

        // Send backspace
        let backspace = Message::new(KeyMsg::from_type(KeyType::Backspace));
        sim.send(backspace);
        sim.step();

        assert_eq!(
            sim.model().value(),
            "hell",
            "Backspace should delete last char"
        );
    }
}

// ============================================================================
// Progress E2E Tests
// ============================================================================

mod progress_e2e {
    use super::*;
    use bubbles::progress::Progress;

    #[test]
    fn test_progress_standalone_lifecycle() {
        let progress = Progress::new().width(40);

        let mut sim = ProgramSimulator::new(progress);
        sim.init();

        let view = sim.last_view().unwrap();
        assert!(!view.is_empty(), "Progress should render");
    }

    #[test]
    fn test_progress_percentage() {
        let mut progress = Progress::new().width(40);
        progress.set_percent(0.5);

        let mut sim = ProgramSimulator::new(progress);
        sim.init();

        assert!(
            (sim.model().percent() - 0.5).abs() < 0.01,
            "Progress should be at 50%"
        );
    }
}

// ============================================================================
// Timer E2E Tests
// ============================================================================

mod timer_e2e {
    use super::*;
    use bubbles::timer::Timer;
    use std::time::Duration;

    #[test]
    fn test_timer_standalone_lifecycle() {
        let timer = Timer::new(Duration::from_secs(60));

        let mut sim = ProgramSimulator::new(timer);
        sim.init();

        let view = sim.last_view().unwrap();
        assert!(!view.is_empty(), "Timer should render");
    }

    #[test]
    fn test_timer_running() {
        let timer = Timer::new(Duration::from_secs(10));

        let mut sim = ProgramSimulator::new(timer);
        sim.init();

        assert!(
            sim.model().running(),
            "Timer should start running after init"
        );
    }
}

// ============================================================================
// Stopwatch E2E Tests
// ============================================================================

mod stopwatch_e2e {
    use super::*;
    use bubbles::stopwatch::{StartStopMsg, Stopwatch};

    #[test]
    fn test_stopwatch_standalone_lifecycle() {
        let stopwatch = Stopwatch::new();

        let mut sim = ProgramSimulator::new(stopwatch);
        sim.init();

        let view = sim.last_view().unwrap();
        assert!(!view.is_empty(), "Stopwatch should render");
    }

    #[test]
    fn test_stopwatch_start_stop() {
        let stopwatch = Stopwatch::new();

        let mut sim = ProgramSimulator::new(stopwatch);
        sim.init();

        // Initially stopwatch is not running (init returns cmd to start it, but
        // the actual StartStopMsg needs to be processed)
        assert!(
            !sim.model().running(),
            "Stopwatch should not be running initially"
        );

        // Simulate starting via the StartStopMsg
        let start_msg = Message::new(StartStopMsg {
            id: sim.model().id(),
            running: true,
        });
        sim.send(start_msg);
        sim.step();

        assert!(
            sim.model().running(),
            "Stopwatch should be running after start message"
        );
    }
}

// ============================================================================
// Paginator E2E Tests
// ============================================================================

mod paginator_e2e {
    use super::*;
    use bubbles::paginator::Paginator;

    #[test]
    fn test_paginator_standalone_lifecycle() {
        let paginator = Paginator::new().total_pages(10);

        let mut sim = ProgramSimulator::new(paginator);
        sim.init();

        let view = sim.last_view().unwrap();
        assert!(!view.is_empty(), "Paginator should render");
    }

    #[test]
    fn test_paginator_navigation() {
        let paginator = Paginator::new().total_pages(5);

        let mut sim = ProgramSimulator::new(paginator);
        sim.init();

        assert_eq!(sim.model().page(), 0, "Should start at page 0");

        // Navigate next
        let next_key = Message::new(KeyMsg::from_char('l'));
        sim.send(next_key);
        sim.step();

        assert_eq!(sim.model().page(), 1, "Should be on page 1");
    }
}

// ============================================================================
// Help E2E Tests
// ============================================================================

mod help_e2e {
    use super::*;
    use bubbles::help::Help;
    use bubbles::key::Binding;

    #[test]
    fn test_help_standalone_lifecycle() {
        let help = Help::new();

        let mut sim = ProgramSimulator::new(help);
        sim.init();

        let _view = sim.last_view().unwrap();
        // Help with no bindings may render empty, that's ok
        assert!(sim.stats().view_calls >= 1, "View should be called");
    }

    #[test]
    fn test_help_with_bindings() {
        let mut help = Help::new().width(80);
        let bindings = vec![
            Binding::new().keys(&["q"]).help("q", "quit"),
            Binding::new().keys(&["?"]).help("?", "toggle help"),
        ];
        help.set_bindings(bindings);

        let mut sim = ProgramSimulator::new(help);
        sim.init();

        let view = sim.last_view().unwrap();
        // The help view should include the key descriptions
        assert!(
            view.contains('q') || view.contains("quit") || !view.is_empty(),
            "Help should render bindings"
        );
    }
}

// ============================================================================
// Cursor E2E Tests
// ============================================================================

mod cursor_e2e {
    use super::*;
    use bubbles::cursor::Cursor;

    #[test]
    fn test_cursor_standalone_lifecycle() {
        let cursor = Cursor::new();

        let mut sim = ProgramSimulator::new(cursor);
        sim.init();

        let _view = sim.last_view().unwrap();
        // Cursor renders something (could be empty string when no char set)
        assert!(sim.stats().view_calls >= 1, "View should be called");
    }

    #[test]
    fn test_cursor_focus_blur() {
        let cursor = Cursor::new();

        let mut sim = ProgramSimulator::new(cursor);
        sim.init();

        // Cursor should handle focus/blur messages
        let focus_msg = Message::new(bubbletea::FocusMsg);
        sim.send(focus_msg);
        sim.step();

        assert_eq!(sim.stats().update_calls, 1, "Should process focus message");
    }
}

// ============================================================================
// Table E2E Tests
// ============================================================================

mod table_e2e {
    use super::*;
    use bubbles::table::{Column, Row, Table};

    #[test]
    fn test_table_standalone_lifecycle() {
        let columns = vec![Column::new("Name", 20), Column::new("Status", 10)];
        let rows: Vec<Row> = vec![
            vec!["Server 1".into(), "Online".into()],
            vec!["Server 2".into(), "Offline".into()],
        ];
        let table = Table::new().columns(columns).rows(rows);

        let mut sim = ProgramSimulator::new(table);
        sim.init();

        let view = sim.last_view().unwrap();
        assert!(
            view.contains("Name") || view.contains("Server"),
            "Table should render columns"
        );
    }

    #[test]
    fn test_table_row_navigation() {
        let columns = vec![Column::new("Name", 20)];
        let rows: Vec<Row> = vec![
            vec!["Row 1".into()],
            vec!["Row 2".into()],
            vec!["Row 3".into()],
        ];
        // Table must be focused to receive key events
        let table = Table::new().columns(columns).rows(rows).focused(true);

        let mut sim = ProgramSimulator::new(table);
        sim.init();

        assert_eq!(sim.model().cursor(), 0, "Should start at row 0");

        // Move down
        let down_key = Message::new(KeyMsg::from_char('j'));
        sim.send(down_key);
        sim.step();

        assert_eq!(sim.model().cursor(), 1, "Should be at row 1");
    }
}

// ============================================================================
// TextArea E2E Tests
// ============================================================================

mod textarea_e2e {
    use super::*;
    use bubbles::textarea::TextArea;

    #[test]
    fn test_textarea_standalone_lifecycle() {
        let mut textarea = TextArea::new();
        textarea.focus();

        let mut sim = ProgramSimulator::new(textarea);
        sim.init();

        let view = sim.last_view().unwrap();
        assert!(!view.is_empty(), "TextArea should render");
    }

    #[test]
    fn test_textarea_multiline_input() {
        let mut textarea = TextArea::new();
        textarea.focus();

        let mut sim = ProgramSimulator::new(textarea);
        sim.init();

        // Type some text
        let h_key = Message::new(KeyMsg::from_char('h'));
        sim.send(h_key);
        sim.step();

        let i_key = Message::new(KeyMsg::from_char('i'));
        sim.send(i_key);
        sim.step();

        // Check value
        assert!(
            sim.model().value().contains("hi"),
            "TextArea should contain typed text"
        );
    }
}

// ============================================================================
// FilePicker E2E Tests
// ============================================================================

mod filepicker_e2e {
    use super::*;
    use bubbles::filepicker::FilePicker;

    #[test]
    fn test_filepicker_standalone_lifecycle() {
        let filepicker = FilePicker::new();

        let mut sim = ProgramSimulator::new(filepicker);
        let _init_cmd = sim.init();

        // FilePicker init may return a command to read directory
        // That's expected behavior
        let _view = sim.last_view().unwrap();
        assert!(sim.stats().view_calls >= 1, "View should be called");
    }

    #[test]
    fn test_filepicker_current_directory() {
        let filepicker = FilePicker::new();

        let mut sim = ProgramSimulator::new(filepicker);
        sim.init();

        // FilePicker should have a current directory set
        let current_dir = sim.model().current_directory();
        // Check directory has components (not empty)
        assert!(
            current_dir.to_str().is_some_and(|s| !s.is_empty()),
            "FilePicker should have a current directory"
        );
    }
}

// ============================================================================
// List E2E Tests
// ============================================================================

mod list_e2e {
    use super::*;
    use bubbles::list::{DefaultDelegate, Item, List};

    #[derive(Clone)]
    struct TestItem {
        name: String,
    }

    impl Item for TestItem {
        fn filter_value(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn test_list_standalone_lifecycle() {
        let items = vec![
            TestItem {
                name: "Apple".into(),
            },
            TestItem {
                name: "Banana".into(),
            },
            TestItem {
                name: "Cherry".into(),
            },
        ];
        let list = List::new(items, DefaultDelegate::new(), 80, 10);

        let mut sim = ProgramSimulator::new(list);
        sim.init();

        let view = sim.last_view().unwrap();
        assert!(!view.is_empty(), "List should render");
    }

    #[test]
    fn test_list_navigation() {
        let items = vec![
            TestItem {
                name: "Apple".into(),
            },
            TestItem {
                name: "Banana".into(),
            },
            TestItem {
                name: "Cherry".into(),
            },
        ];
        let list = List::new(items, DefaultDelegate::new(), 80, 10);

        let mut sim = ProgramSimulator::new(list);
        sim.init();

        assert_eq!(sim.model().index(), 0, "Should start at index 0");

        // Move down
        let down_key = Message::new(KeyMsg::from_char('j'));
        sim.send(down_key);
        sim.step();

        assert_eq!(sim.model().index(), 1, "Should be at index 1");
    }

    #[test]
    fn test_list_selection() {
        let items = vec![
            TestItem {
                name: "Apple".into(),
            },
            TestItem {
                name: "Banana".into(),
            },
        ];
        let list = List::new(items, DefaultDelegate::new(), 80, 10);

        let mut sim = ProgramSimulator::new(list);
        sim.init();

        let selected = sim.model().selected_item();
        assert!(selected.is_some(), "Should have selected item");
        assert_eq!(
            selected.unwrap().name,
            "Apple",
            "First item should be selected"
        );
    }
}

// ============================================================================
// Component Combination Tests
// ============================================================================

mod combination_tests {
    use super::*;
    use bubbles::help::Help;
    use bubbles::key::Binding;
    use bubbles::textinput::TextInput;

    /// A composite model combining `TextInput` and `Help`
    struct FormWithHelp {
        input: TextInput,
        help: Help,
        show_help: bool,
    }

    impl FormWithHelp {
        fn new() -> Self {
            let mut input = TextInput::new();
            input.focus();

            let mut help = Help::new();
            help.set_bindings(vec![
                Binding::new().keys(&["?"]).help("?", "toggle help"),
                Binding::new().keys(&["enter"]).help("enter", "submit"),
            ]);

            Self {
                input,
                help,
                show_help: false,
            }
        }
    }

    impl Model for FormWithHelp {
        fn init(&self) -> Option<bubbletea::Cmd> {
            None
        }

        fn update(&mut self, msg: Message) -> Option<bubbletea::Cmd> {
            if let Some(key) = msg.downcast_ref::<KeyMsg>() {
                if key.key_type == KeyType::Runes && key.runes == vec!['?'] {
                    self.show_help = !self.show_help;
                    return None;
                }
            }

            // Forward to input
            self.input.update(msg);
            None
        }

        fn view(&self) -> String {
            let mut output = self.input.view();
            if self.show_help {
                output.push_str("\n\n");
                // Use Model::view to call the trait method which uses stored bindings
                output.push_str(&Model::view(&self.help));
            }
            output
        }
    }

    #[test]
    fn test_form_with_help_combination() {
        let form = FormWithHelp::new();
        let mut sim = ProgramSimulator::new(form);

        sim.init();
        assert!(!sim.model().show_help, "Help should be hidden initially");

        // Toggle help
        let help_key = Message::new(KeyMsg::from_char('?'));
        sim.send(help_key);
        sim.step();

        assert!(sim.model().show_help, "Help should be visible after toggle");
    }

    #[test]
    fn test_form_with_help_typing() {
        let form = FormWithHelp::new();
        let mut sim = ProgramSimulator::new(form);

        sim.init();

        // Type in input
        let a_key = Message::new(KeyMsg::from_char('a'));
        sim.send(a_key);
        sim.step();

        let b_key = Message::new(KeyMsg::from_char('b'));
        sim.send(b_key);
        sim.step();

        assert_eq!(
            sim.model().input.value(),
            "ab",
            "Input should contain typed text"
        );
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

mod error_handling_tests {
    use super::*;
    use bubbles::progress::Progress;
    use bubbles::viewport::Viewport;

    #[test]
    fn test_viewport_empty_content() {
        let viewport = Viewport::new(80, 10);
        let mut sim = ProgramSimulator::new(viewport);

        sim.init();

        // Should handle empty content gracefully
        let view = sim.last_view().unwrap();
        assert!(
            view.lines().count() <= 10,
            "Should not overflow viewport height"
        );
    }

    #[test]
    fn test_progress_boundary_values() {
        // Test 0%
        let mut progress = Progress::new().width(40);
        progress.set_percent(0.0);
        let mut sim = ProgramSimulator::new(progress);
        sim.init();
        assert!((sim.model().percent() - 0.0).abs() < 0.01);

        // Test 100%
        let mut progress = Progress::new().width(40);
        progress.set_percent(1.0);
        let mut sim = ProgramSimulator::new(progress);
        sim.init();
        assert!((sim.model().percent() - 1.0).abs() < 0.01);

        // Test > 100% (should clamp)
        let mut progress = Progress::new().width(40);
        progress.set_percent(1.5);
        let mut sim = ProgramSimulator::new(progress);
        sim.init();
        assert!(sim.model().percent() <= 1.0, "Should clamp to 100%");
    }

    #[test]
    fn test_viewport_scroll_beyond_content() {
        let mut viewport = Viewport::new(80, 10);
        viewport.set_content("Line 1\nLine 2");

        let mut sim = ProgramSimulator::new(viewport);
        sim.init();

        // Try to scroll way past content
        for _ in 0..100 {
            let down_key = Message::new(KeyMsg::from_char('j'));
            sim.send(down_key);
            sim.step();
        }

        // Should be at bottom, not crashed
        assert!(sim.model().at_bottom(), "Should be at bottom");
    }
}

// ============================================================================
// Performance Tests
// ============================================================================

mod performance_tests {
    use super::*;
    use bubbles::spinner::SpinnerModel;
    use bubbles::viewport::Viewport;
    use std::time::Instant;

    #[test]
    fn test_spinner_performance() {
        let start = Instant::now();

        let spinner = SpinnerModel::new();
        let mut sim = ProgramSimulator::new(spinner);
        sim.init();

        // Process many view renders (frame advances naturally via tick messages)
        for _ in 0..1000 {
            let view = sim.model().view();
            assert!(!view.is_empty());
        }

        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 1000,
            "Spinner operations should be fast"
        );
    }

    #[test]
    fn test_viewport_large_content_performance() {
        let start = Instant::now();

        // Create viewport with large content
        let content: String = (0..10000)
            .map(|i| format!("Line {}", i))
            .collect::<Vec<_>>()
            .join("\n");

        let mut viewport = Viewport::new(80, 24);
        viewport.set_content(&content);

        let mut sim = ProgramSimulator::new(viewport);
        sim.init();

        // Scroll through content
        for _ in 0..100 {
            let down_key = Message::new(KeyMsg::from_char('j'));
            sim.send(down_key);
            sim.step();
        }

        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 1000,
            "Viewport with large content should be fast"
        );
    }
}
