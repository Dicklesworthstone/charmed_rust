//! Test that an empty named struct derives correctly.
//!
//! Empty structs are allowed - they just don't track any state.

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
struct EmptyApp {}

impl EmptyApp {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, _msg: Message) -> Option<Cmd> {
        None
    }

    fn view(&self) -> String {
        "Empty App".to_string()
    }
}

fn main() {
    let app = EmptyApp {};

    // Verify Model trait is implemented
    let _ = <EmptyApp as Model>::init(&app);
    let _ = <EmptyApp as Model>::view(&app);

    // Verify state methods exist (no-op for empty structs)
    let snapshot = app.__snapshot_state();
    let changed = app.__state_changed(&snapshot);
    assert!(!changed, "Empty struct should never report changes");
}
