//! Test that an empty named struct derives correctly.
//!
//! Empty structs are allowed - they just don't track any state.

use bubbletea::{Cmd, Message, Model};

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
