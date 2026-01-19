//! Test that structs with many fields derive correctly.
//!
//! This verifies the macro handles larger structs without issues.

use bubbletea::{Cmd, Message, Model};

#[derive(Model)]
struct LargeApp {
    #[state]
    a: i32,
    #[state]
    b: i32,
    #[state]
    c: i32,
    #[state]
    d: i32,
    #[state]
    e: i32,
    #[state]
    f: i32,
    #[state]
    g: i32,
    #[state]
    h: i32,

    // Non-tracked fields
    cache1: String,
    cache2: String,
    cache3: String,
}

impl LargeApp {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, _msg: Message) -> Option<Cmd> {
        self.a += 1;
        None
    }

    fn view(&self) -> String {
        format!(
            "a={} b={} c={} d={} e={} f={} g={} h={}",
            self.a, self.b, self.c, self.d, self.e, self.f, self.g, self.h
        )
    }
}

fn main() {
    let mut app = LargeApp {
        a: 1, b: 2, c: 3, d: 4,
        e: 5, f: 6, g: 7, h: 8,
        cache1: String::new(),
        cache2: String::new(),
        cache3: String::new(),
    };

    // Verify Model trait works
    let _ = <LargeApp as Model>::view(&app);

    // Verify state change detection
    let snapshot = app.__snapshot_state();
    assert!(!app.__state_changed(&snapshot), "Should not change before update");

    app.a = 100;
    assert!(app.__state_changed(&snapshot), "Should detect change to a");

    // Verify non-tracked field changes don't trigger state change
    let snapshot2 = app.__snapshot_state();
    app.cache1 = "modified".to_string();
    assert!(!app.__state_changed(&snapshot2), "Non-tracked field should not trigger change");
}
