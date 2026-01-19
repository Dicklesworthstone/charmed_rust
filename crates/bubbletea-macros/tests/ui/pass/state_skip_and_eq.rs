//! Test that #[state(skip)] and #[state(eq = "fn")] work correctly.

use bubbletea::{Cmd, Message, Model};

fn float_approx_eq(a: &f64, b: &f64) -> bool {
    (a - b).abs() < 0.001
}

#[derive(Model)]
struct AppWithAdvancedState {
    #[state]
    counter: i32,

    #[state(eq = "float_approx_eq")]
    progress: f64,

    #[state(skip)]
    last_tick: u64,
}

impl AppWithAdvancedState {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, _msg: Message) -> Option<Cmd> {
        self.counter += 1;
        None
    }

    fn view(&self) -> String {
        format!("Count: {}, Progress: {:.2}", self.counter, self.progress)
    }
}

fn main() {
    let app = AppWithAdvancedState {
        counter: 0,
        progress: 0.0,
        last_tick: 0,
    };

    // Test that state snapshot methods are generated
    let snapshot = app.__snapshot_state();

    // Note: last_tick is NOT in the snapshot because it has skip
    let _ = app.__state_changed(&snapshot);
}
