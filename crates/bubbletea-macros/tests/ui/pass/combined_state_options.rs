//! Test that combined state options work correctly.
//!
//! This verifies using `#[state(eq = "...", debug)]` on a single field.
//!
//! Note: Multiple fields with combined eq+debug may have parsing issues
//! in some Rust versions due to block/|| interaction. See model.rs unit tests
//! for coverage of multi-field scenarios.

use bubbletea::{Cmd, Message, Model};

fn float_approx_eq(a: &f64, b: &f64) -> bool {
    (a - b).abs() < 0.001
}

#[derive(Model)]
struct CombinedOptions {
    // Single field with custom equality and debug logging combined
    #[state(eq = "float_approx_eq", debug)]
    progress: f64,
}

impl CombinedOptions {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, _msg: Message) -> Option<Cmd> {
        None
    }

    fn view(&self) -> String {
        format!("progress={:.2}", self.progress)
    }
}

fn main() {
    let mut app = CombinedOptions {
        progress: 0.5,
    };

    // Test custom float equality with debug
    let snapshot = app.__snapshot_state();
    app.progress = 0.5001; // Within tolerance
    assert!(!app.__state_changed(&snapshot), "Small float change should be ignored");

    let snapshot2 = app.__snapshot_state();
    app.progress = 0.6; // Beyond tolerance
    assert!(app.__state_changed(&snapshot2), "Large float change should be detected");
}
