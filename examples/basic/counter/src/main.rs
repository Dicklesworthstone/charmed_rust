//! Simple Counter Example
//!
//! This is the most basic Bubble Tea application - a counter that you can
//! increment and decrement using keyboard keys.
//!
//! Run with: `cargo run -p example-counter`

#![forbid(unsafe_code)]

use bubbletea::{Cmd, KeyMsg, KeyType, Message, Program, quit};

/// The application model using the derive macro.
///
/// The `#[derive(Model)]` macro generates the `impl Model for Counter` that
/// delegates to the inherent methods `init`, `update`, and `view`.
#[derive(bubbletea::Model)]
struct Counter {
    count: i32,
}

impl Counter {
    /// Create a new counter starting at zero.
    fn new() -> Self {
        Self { count: 0 }
    }

    /// Initialize the model - no startup commands needed.
    fn init(&self) -> Option<Cmd> {
        None
    }

    /// Handle messages and update the model.
    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle keyboard input
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::Runes => {
                    if let Some(&ch) = key.runes.first() {
                        match ch {
                            '+' | '=' | 'k' => self.count += 1,
                            '-' | '_' | 'j' => self.count -= 1,
                            'q' | 'Q' => return Some(quit()),
                            _ => {}
                        }
                    }
                }
                KeyType::Up => self.count += 1,
                KeyType::Down => self.count -= 1,
                KeyType::CtrlC | KeyType::Esc => return Some(quit()),
                _ => {}
            }
        }
        None
    }

    /// Render the view as a string.
    fn view(&self) -> String {
        format!(
            "\n  Counter: {}\n\n  [+/-] or [k/j] to change\n  [q] or [Esc] to quit\n",
            self.count
        )
    }
}

fn main() -> anyhow::Result<()> {
    // Create and run the program
    let final_model = Program::new(Counter::new()).with_alt_screen().run()?;

    println!("Final count: {}", final_model.count);
    Ok(())
}
