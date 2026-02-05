//! Test that #[state(skip)] and #[state(eq = "fn")] work correctly.

extern crate self as bubbletea;

pub use bubbletea_macros::Model;

#[derive(Clone, Debug)]
pub struct Cmd;

#[derive(Clone, Debug)]
pub struct Message;

impl Message {
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        None
    }
}

pub trait Model {
    fn init(&self) -> Option<Cmd>;
    fn update(&mut self, msg: Message) -> Option<Cmd>;
    fn view(&self) -> String;
}

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
