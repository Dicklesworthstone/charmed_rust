//! Test that structs with static lifetime references derive correctly.

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
struct StaticRef {
    text: &'static str,
    #[state]
    count: i32,
}

impl StaticRef {
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
        format!("{}: {}", self.text, self.count)
    }
}

fn main() {
    let model = StaticRef {
        text: "Counter",
        count: 0,
    };

    // Verify Model trait is implemented
    let _ = <StaticRef as Model>::view(&model);
    assert_eq!(model.view(), "Counter: 0");

    // Verify state tracking works
    let snapshot = model.__snapshot_state();
    assert!(!model.__state_changed(&snapshot));
}
