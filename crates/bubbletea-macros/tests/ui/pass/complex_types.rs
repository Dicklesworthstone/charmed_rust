//! Test that structs with complex field types derive correctly.
//!
//! This verifies the macro handles Vec, HashMap, Option, and other complex types.

use std::collections::HashMap;

use bubbletea::{Cmd, Message, Model};

#[derive(Model)]
struct ComplexApp {
    #[state]
    items: Vec<String>,

    #[state]
    lookup: HashMap<String, i32>,

    #[state]
    optional: Option<String>,

    #[state]
    nested: Vec<Vec<i32>>,

    #[state]
    boxed: Box<String>,
}

impl ComplexApp {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, _msg: Message) -> Option<Cmd> {
        self.items.push("new".to_string());
        None
    }

    fn view(&self) -> String {
        format!(
            "items: {:?}, lookup: {:?}, optional: {:?}",
            self.items, self.lookup, self.optional
        )
    }
}

fn main() {
    let mut app = ComplexApp {
        items: vec!["a".to_string(), "b".to_string()],
        lookup: HashMap::from([("key".to_string(), 42)]),
        optional: Some("value".to_string()),
        nested: vec![vec![1, 2], vec![3, 4]],
        boxed: Box::new("boxed".to_string()),
    };

    // Verify Model trait works
    let _ = <ComplexApp as Model>::view(&app);

    // Verify state change detection for Vec
    let snapshot = app.__snapshot_state();
    assert!(!app.__state_changed(&snapshot));
    app.items.push("c".to_string());
    assert!(app.__state_changed(&snapshot), "Vec change should be detected");

    // Verify state change detection for HashMap
    let snapshot2 = app.__snapshot_state();
    app.lookup.insert("new_key".to_string(), 100);
    assert!(app.__state_changed(&snapshot2), "HashMap change should be detected");

    // Verify state change detection for Option
    let snapshot3 = app.__snapshot_state();
    app.optional = None;
    assert!(app.__state_changed(&snapshot3), "Option change should be detected");

    // Verify state change detection for nested Vec
    let snapshot4 = app.__snapshot_state();
    app.nested[0].push(5);
    assert!(app.__state_changed(&snapshot4), "Nested Vec change should be detected");

    // Verify state change detection for Box
    let snapshot5 = app.__snapshot_state();
    *app.boxed = "modified".to_string();
    assert!(app.__state_changed(&snapshot5), "Box content change should be detected");
}
