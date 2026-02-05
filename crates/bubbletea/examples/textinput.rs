#![forbid(unsafe_code)]

//! Text input example demonstrating `#[derive(Model)]` with component composition.
//!
//! This example shows how to:
//! - Use the bubbles `TextInput` component
//! - Handle focus and submission
//! - Display user input with styling
//!
//! Run with: cargo run -p bubbletea --example textinput

mod bubbles {
    pub mod textinput {
        #![allow(
            clippy::must_use_candidate,
            clippy::unused_self,
            clippy::needless_pass_by_ref_mut,
            clippy::missing_const_for_fn
        )]

        use bubbletea::{Cmd, Message};

        #[derive(Clone, Default)]
        pub struct TextInput {
            value: String,
        }

        impl TextInput {
            pub fn new() -> Self {
                Self::default()
            }

            pub fn set_placeholder(&mut self, _placeholder: &str) {}

            pub fn focus(&mut self) {}

            pub fn init(&self) -> Option<Cmd> {
                None
            }

            pub fn update(&mut self, _msg: Message) -> Option<Cmd> {
                None
            }

            pub fn view(&self) -> String {
                self.value.clone()
            }

            pub fn value(&self) -> String {
                self.value.clone()
            }
        }
    }
}

use bubbles::textinput::TextInput;
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Program, quit};
use lipgloss::Style;

/// Application state tracking input and submission.
///
/// Uses `#[derive(bubbletea::Model)]` to auto-implement the Model trait,
/// delegating to the inherent `init`, `update`, and `view` methods.
#[derive(bubbletea::Model)]
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

    /// Initialize the model. Called once when the program starts.
    fn init(&self) -> Option<Cmd> {
        // Initialize cursor blinking for the text input
        self.input.init()
    }

    /// Handle messages and update the model state.
    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle keyboard input
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::Enter => {
                    if self.submitted {
                        return Some(quit());
                    }
                    self.name = self.input.value();
                    self.submitted = true;
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

    /// Render the model as a string for display.
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
