//! Spinner Example
//!
//! This example demonstrates:
//! - Using the bubbles spinner component
//! - Async tick-based animations
//! - Component composition in the Elm Architecture
//!
//! Run with: `cargo run -p example-spinner`

#![forbid(unsafe_code)]

use bubbles::spinner::{SpinnerModel, spinners};
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, Program, quit};
use lipgloss::Style;

/// Application model that wraps the spinner component.
///
/// In the Elm Architecture, your Model contains all application state.
/// Here, we compose a SpinnerModel from the bubbles crate.
#[derive(bubbletea::Model)]
struct App {
    spinner: SpinnerModel,
    loading: bool,
}

impl App {
    /// Create a new app with a styled spinner.
    fn new() -> Self {
        // Create a pink-colored dot spinner
        let style = Style::new().foreground("212");
        let spinner = SpinnerModel::with_spinner(spinners::dot()).style(style);

        Self {
            spinner,
            loading: true,
        }
    }

    /// Initialize - delegate to spinner's init for its tick command.
    fn init(&self) -> Option<Cmd> {
        // The spinner needs to start its tick loop
        self.spinner.init()
    }

    /// Handle messages - keyboard input and spinner ticks.
    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle keyboard input first
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::Runes => {
                    if let Some('q' | 'Q') = key.runes.first() {
                        return Some(quit());
                    }
                }
                KeyType::CtrlC | KeyType::Esc => return Some(quit()),
                _ => {}
            }
        }

        // Forward message to spinner for tick handling
        self.spinner.update(msg)
    }

    /// Render the view with spinner animation.
    fn view(&self) -> String {
        if self.loading {
            format!(
                "\n  {} Loading... please wait\n\n  Press [q] or [Esc] to quit\n",
                self.spinner.view()
            )
        } else {
            "  Done!\n".to_string()
        }
    }
}

fn main() -> anyhow::Result<()> {
    // Create and run the program with alternate screen
    Program::new(App::new()).with_alt_screen().run()?;

    println!("Goodbye!");
    Ok(())
}
