#![forbid(unsafe_code)]

//! Spinner example demonstrating `#[derive(Model)]` with animated loading indicators.
//!
//! This example shows how to:
//! - Compose bubbles components into a custom model
//! - Handle async tick messages for animation
//! - Apply lipgloss styles to components
//!
//! Run with: cargo run -p charmed-bubbletea --example spinner

use bubbles::spinner::{SpinnerModel, spinners};
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, Program, quit};
use lipgloss::Style;

/// Main application model composing a spinner with app state.
///
/// Uses `#[derive(bubbletea::Model)]` to auto-implement the Model trait,
/// delegating to the inherent `init`, `update`, and `view` methods.
#[derive(bubbletea::Model)]
struct App {
    spinner: SpinnerModel,
    loading: bool,
}

impl App {
    fn new() -> Self {
        // Create a dot-style spinner with magenta color
        let style = Style::new().foreground("212");
        let spinner = SpinnerModel::with_spinner(spinners::dot()).style(style);

        Self {
            spinner,
            loading: true,
        }
    }

    /// Initialize the model. Called once when the program starts.
    fn init(&self) -> Option<Cmd> {
        // Start the spinner animation
        self.spinner.init()
    }

    /// Handle messages and update the model state.
    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle keyboard input
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::Runes => {
                    if let Some(&('q' | 'Q')) = key.runes.first() {
                        return Some(quit());
                    }
                }
                KeyType::CtrlC | KeyType::Esc => return Some(quit()),
                _ => {}
            }
        }

        // Update the spinner (handles tick messages)
        self.spinner.update(msg)
    }

    /// Render the model as a string for display.
    fn view(&self) -> String {
        if self.loading {
            format!(
                "{} Loading... please wait\n\nPress q to quit.",
                self.spinner.view()
            )
        } else {
            "Done!".to_string()
        }
    }
}

fn main() -> Result<(), bubbletea::Error> {
    let model = App::new();
    Program::new(model).run()?;
    Ok(())
}
