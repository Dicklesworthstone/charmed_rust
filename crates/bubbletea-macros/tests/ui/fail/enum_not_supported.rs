//! Test that derive(Model) fails on enums.

extern crate self as bubbletea;

pub use bubbletea_macros::Model;

pub trait Model {
    fn init(&self) -> Option<Cmd>;
    fn update(&mut self, msg: Message) -> Option<Cmd>;
    fn view(&self) -> String;
}

pub struct Cmd;
pub struct Message;

#[derive(Model)]
enum NotAStruct {
    Variant1,
    Variant2,
}

fn main() {}
