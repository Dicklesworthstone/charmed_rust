#![forbid(unsafe_code)]

//! Text input example demonstrating user input handling.
//!
//! This example shows how to:
//! - Use the bubbles TextInput component
//! - Handle focus and submission
//! - Display user input with styling
//!
//! Run with: cargo run -p bubbletea --example textinput

use bubbles::textinput::TextInput;
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, Program, quit};
use lipgloss::Style;

/// Application state tracking input and submission.
struct App {
    input: TextInput,
    submitted: bool,
    name: String,
}

impl App {
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
}

impl Model for App {
    fn init(&self) -> Option<Cmd> {
        // Initialize cursor blinking for the text input
        self.input.init()
    }

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

    fn view(&self) -> String {
        if self.submitted {
            let style = Style::new().foreground("212");
            format!(
                "Hello, {}!\n\nPress Enter to quit.",
                style.render(&self.name)
            )
        } else {
            format!(
                "What's your name?\n\n{}\n\nPress Enter to submit, Esc to quit.",
                self.input.view()
            )
        }
    }
}

fn main() -> Result<(), bubbletea::Error> {
    let model = App::new();
    Program::new(model).run()?;
    Ok(())
}
