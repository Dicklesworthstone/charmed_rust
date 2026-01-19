//! Test that a basic counter model derives correctly.

use bubbletea::{Cmd, Message, Model};

#[derive(Model)]
struct Counter {
    count: i32,
}

impl Counter {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(&delta) = msg.downcast_ref::<i32>() {
            self.count += delta;
        }
        None
    }

    fn view(&self) -> String {
        format!("Count: {}", self.count)
    }
}

fn main() {
    let counter = Counter { count: 0 };

    // Verify Model trait is implemented by calling trait methods
    let _ = <Counter as Model>::init(&counter);
    let _ = <Counter as Model>::view(&counter);

    // Also verify inherent methods work
    assert_eq!(counter.view(), "Count: 0");
}
