//! State tracking example demonstrating advanced `#[state]` attribute options.
//!
//! This example shows how to use:
//! - Basic `#[state]` for change detection
//! - `#[state(eq = "fn")]` for custom equality
//! - `#[state(skip)]` to exclude fields
//! - `#[state(debug)]` to log changes
//!
//! Run with: `cargo run -p bubbletea-macros --example state_tracking`

#![forbid(unsafe_code)]

use std::time::Instant;

use bubbletea::{Cmd, KeyMsg, KeyType, Message, Program, quit};

/// Custom equality function for floating-point comparison.
/// Uses approximate equality to avoid re-renders for tiny changes.
fn float_approx_eq(a: &f64, b: &f64) -> bool {
    (a - b).abs() < 0.001
}

/// App demonstrating various state tracking options.
#[derive(bubbletea::Model)]
struct StateDemo {
    /// Basic state tracking using PartialEq.
    #[state]
    counter: i32,

    /// Custom equality for floating-point comparison.
    /// Small changes (< 0.001) won't trigger re-renders.
    #[state(eq = "float_approx_eq")]
    progress: f64,

    /// Excluded from change detection.
    /// Useful for timestamps or internal bookkeeping.
    #[state(skip)]
    last_update: Option<Instant>,

    /// Debug logging - changes to this field are logged to stderr.
    /// Enable with: `RUST_LOG=debug cargo run --example state_tracking`
    #[state(debug)]
    selected_index: usize,

    /// Not tracked - no #[state] attribute.
    /// Changes to this field won't affect change detection.
    message_count: usize,
}

impl StateDemo {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::Runes => {
                    if let Some(&ch) = key.runes.first() {
                        match ch {
                            // Counter controls
                            '+' | '=' => self.counter = self.counter.saturating_add(1),
                            '-' | '_' => self.counter = self.counter.saturating_sub(1),
                            
                            // Progress controls (increments by 0.05)
                            '.' => self.progress = (self.progress + 0.05).min(1.0),
                            ',' => self.progress = (self.progress - 0.05).max(0.0),
                            
                            // Index controls
                            'j' | 'J' => self.selected_index = (self.selected_index + 1).min(9),
                            'k' | 'K' => self.selected_index = self.selected_index.saturating_sub(1),
                            
                            'q' | 'Q' => return Some(quit()),
                            _ => {}
                        }
                    }
                }
                KeyType::CtrlC | KeyType::Esc => return Some(quit()),
                _ => {}
            }
            
            // Update non-tracked fields
            self.message_count += 1;
            self.last_update = Some(Instant::now());
        }
        None
    }

    fn view(&self) -> String {
        format!(
            "State Tracking Demo\n\n\
             Counter: {} (basic #[state])\n\
             Progress: {:.2} (custom equality, < 0.001 ignored)\n\
             Index: {} (#[state(debug)] - watch stderr)\n\n\
             Messages received: {} (not tracked)\n\n\
             Controls:\n\
             +/- : Change counter\n\
             ,/. : Change progress\n\
             j/k : Change index\n\
             q   : Quit",
            self.counter,
            self.progress,
            self.selected_index,
            self.message_count
        )
    }
}

fn main() -> Result<(), bubbletea::Error> {
    let model = StateDemo {
        counter: 0,
        progress: 0.5,
        last_update: None,
        selected_index: 0,
        message_count: 0,
    };
    Program::new(model).run()?;
    Ok(())
}
