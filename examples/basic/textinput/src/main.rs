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
