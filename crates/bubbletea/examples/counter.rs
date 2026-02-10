//! Simple counter example demonstrating `#[derive(Model)]` usage.
//!
//! This example shows the basic pattern for bubbletea applications using the
//! derive macro instead of manual trait implementation.
//!
//! Run with: `cargo run -p charmed-bubbletea --example counter`

#![forbid(unsafe_code)]
#![allow(clippy::unused_self)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::needless_pass_by_value)]

use bubbletea::{Cmd, KeyMsg, KeyType, Message, Program, quit};

/// Counter model using the derive macro.
///
/// The `#[derive(bubbletea::Model)]` macro generates the `Model` trait
/// implementation that delegates to the inherent `init`, `update`, and `view` methods.
#[derive(bubbletea::Model)]
struct Counter {
    count: i32,
}

impl Counter {
    const fn new() -> Self {
        Self { count: 0 }
    }

    /// Initialize the model. Called once when the program starts.
    fn init(&self) -> Option<Cmd> {
        None
    }

    /// Handle messages and update the model state.
    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::Runes => {
                    if let Some(&ch) = key.runes.first() {
                        match ch {
                            '+' => self.count += 1,
                            '-' => self.count -= 1,
                            'q' | 'Q' => return Some(quit()),
                            _ => {}
                        }
                    }
                }
                KeyType::CtrlC | KeyType::Esc => return Some(quit()),
                _ => {}
            }
        }
        None
    }

    /// Render the model as a string for display.
    fn view(&self) -> String {
        format!("Count: {}\n\nPress + / - to change, q to quit.", self.count)
    }
}

fn main() -> Result<(), bubbletea::Error> {
    let model = Counter::new();
    Program::new(model).run()?;
    Ok(())
}
