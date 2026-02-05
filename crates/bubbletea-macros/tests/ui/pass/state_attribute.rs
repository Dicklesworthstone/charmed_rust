//! Test that #[state] attribute works correctly.

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

#[derive(Model)]
struct AppWithState {
    #[state]
    counter: i32,

    #[state(debug)]
    selected: usize,

    // Not tracked - no #[state]
    cache: String,
}

impl AppWithState {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, _msg: Message) -> Option<Cmd> {
        self.counter += 1;
        None
    }

    fn view(&self) -> String {
        format!("Count: {}", self.counter)
    }
}

fn main() {
    let app = AppWithState {
        counter: 0,
        selected: 0,
        cache: String::new(),
    };

    // Test that state snapshot methods are generated
    let snapshot = app.__snapshot_state();
    let _ = app.__state_changed(&snapshot);
}
