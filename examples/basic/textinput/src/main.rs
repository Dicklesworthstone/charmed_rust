//! Text Input Example
//!
//! This example demonstrates:
//! - Using the bubbles TextInput component
//! - Handling focus and user input
//! - Form submission with styling
//!
//! Run with: `cargo run -p example-textinput`

#![forbid(unsafe_code)]

use bubbles::textinput::TextInput;
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, Program, quit};
use lipgloss::Style;

/// Application model that wraps the text input component.
///
/// In the Elm Architecture, your Model contains all application state.
/// Here, we compose a TextInput from the bubbles crate.
#[derive(bubbletea::Model)]
struct App {
    input: TextInput,
    submitted: bool,
    name: String,
}

impl App {
    /// Create a new app with a focused text input.
    fn new() -> Self {
        let mut input = TextInput::new();
        input.set_placeholder("Enter your name...");
        input.focus();

        Self {
            input,
            submitted: false,
            name: String::new(),
        }
    }

    /// Initialize - delegate to text input's init for cursor blinking.
    fn init(&self) -> Option<Cmd> {
        self.input.init()
    }

    /// Handle messages - keyboard input and text input events.
    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle keyboard input
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::Enter => {
                    if !self.submitted {
                        self.name = self.input.value();
                        self.submitted = true;
                    } else {
                        return Some(quit());
                    }
                }
                KeyType::CtrlC | KeyType::Esc => return Some(quit()),
                _ => {}
            }
        }

        // Pass messages to text input (handles character input, cursor, etc.)
        if !self.submitted {
            return self.input.update(msg);
        }

        None
    }

    /// Render the view with text input or greeting.
    fn view(&self) -> String {
        if self.submitted {
            let style = Style::new().foreground("212");
            format!(
                "\n  Hello, {}!\n\n  Press Enter to quit.\n",
                style.render(&self.name)
            )
        } else {
            format!(
                "\n  What's your name?\n\n  {}\n\n  Press Enter to submit, Esc to quit.\n",
                self.input.view()
            )
        }
    }
}

fn main() -> anyhow::Result<()> {
    // Create and run the program
    Program::new(App::new()).with_alt_screen().run()?;

    println!("Goodbye!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a key message for a character
    fn key_char(ch: char) -> Message {
        Message::new(KeyMsg {
            key_type: KeyType::Runes,
            runes: vec![ch],
            alt: false,
            paste: false,
        })
    }

    /// Create a key message for a special key
    fn key_type(kt: KeyType) -> Message {
        Message::new(KeyMsg {
            key_type: kt,
            runes: vec![],
            alt: false,
            paste: false,
        })
    }

    #[test]
    fn test_app_initial_state() {
        let app = App::new();
        assert!(!app.submitted, "App should not be submitted initially");
        assert!(app.name.is_empty(), "Name should be empty initially");
    }

    #[test]
    fn test_input_is_focused() {
        let app = App::new();
        assert!(app.input.focused(), "Input should be focused on start");
    }

    #[test]
    fn test_view_shows_prompt() {
        let app = App::new();
        let view = app.view();
        assert!(view.contains("name"), "View should ask for name");
    }

    #[test]
    fn test_view_shows_help() {
        let app = App::new();
        let view = app.view();
        assert!(view.contains("Enter"), "View should mention Enter key");
    }

    #[test]
    fn test_enter_submits() {
        let mut app = App::new();
        // Type some text first (simulated by setting input value directly)
        app.input.set_value("Alice");

        // Press Enter
        app.update(key_type(KeyType::Enter));

        assert!(app.submitted, "App should be submitted after Enter");
        assert_eq!(app.name, "Alice", "Name should be captured");
    }

    #[test]
    fn test_submitted_view_shows_greeting() {
        let mut app = App::new();
        app.input.set_value("Bob");
        app.update(key_type(KeyType::Enter));

        let view = app.view();
        assert!(view.contains("Hello"), "Submitted view should show greeting");
        assert!(view.contains("Bob"), "Submitted view should show name");
    }

    #[test]
    fn test_enter_after_submit_quits() {
        let mut app = App::new();
        app.input.set_value("Test");
        app.update(key_type(KeyType::Enter)); // Submit

        let cmd = app.update(key_type(KeyType::Enter)); // Quit
        assert!(cmd.is_some(), "Enter after submit should return quit");
    }

    #[test]
    fn test_quit_esc() {
        let mut app = App::new();
        let cmd = app.update(key_type(KeyType::Esc));
        assert!(cmd.is_some(), "Esc should return quit command");
    }

    #[test]
    fn test_quit_ctrl_c() {
        let mut app = App::new();
        let cmd = app.update(key_type(KeyType::CtrlC));
        assert!(cmd.is_some(), "Ctrl+C should return quit command");
    }

    #[test]
    fn test_text_input_receives_chars() {
        let mut app = App::new();
        // The actual character handling is delegated to TextInput
        // We just verify the update doesn't panic
        app.update(key_char('H'));
        app.update(key_char('i'));
    }

    #[test]
    fn test_init_returns_command() {
        let app = App::new();
        // TextInput init returns cursor blink command
        let cmd = app.init();
        assert!(cmd.is_some(), "Init should return cursor blink command");
    }

    #[test]
    fn test_input_blocked_after_submit() {
        let mut app = App::new();
        app.input.set_value("Test");
        app.update(key_type(KeyType::Enter)); // Submit

        // After submit, input should not process chars
        let cmd = app.update(key_char('x'));
        assert!(cmd.is_none(), "Char input after submit should return None");
    }
}
