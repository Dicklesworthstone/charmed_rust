//! Test that a generic model derives correctly.

use bubbletea::{Cmd, Message, Model};
use std::fmt::Display;

#[derive(Model)]
struct Container<T: Send + 'static> {
    value: T,
}

impl<T: Send + 'static> Container<T> {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, _msg: Message) -> Option<Cmd> {
        None
    }

    fn view(&self) -> String {
        "Container".to_string()
    }
}

#[derive(Model)]
struct BoundedContainer<T>
where
    T: Clone + Display + Send + 'static,
{
    value: T,
}

impl<T> BoundedContainer<T>
where
    T: Clone + Display + Send + 'static,
{
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, _msg: Message) -> Option<Cmd> {
        None
    }

    fn view(&self) -> String {
        format!("Value: {}", self.value)
    }
}

fn main() {
    // Generic type parameter
    let container = Container { value: 42i32 };
    let _ = <Container<i32> as Model>::view(&container);
    assert_eq!(container.view(), "Container");

    // Generic with where clause
    let bounded = BoundedContainer { value: "test".to_string() };
    let _ = <BoundedContainer<String> as Model>::view(&bounded);
    assert_eq!(bounded.view(), "Value: test");
}
