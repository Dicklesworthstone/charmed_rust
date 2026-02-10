//! Counter example demonstrating `#[derive(Model)]` usage.
//!
//! This example shows the basic usage of the derive macro to implement
//! the Model trait with automatic state tracking.
//!
//! Run with: `cargo run -p charmed-bubbletea-macros --example counter_derive`

#![forbid(unsafe_code)]
#![allow(dead_code)]
#![allow(clippy::unused_self)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::needless_pass_by_value)]

extern crate self as bubbletea;

pub use bubbletea_macros::Model;

#[derive(Clone, Debug)]
pub struct Cmd;

#[derive(Clone, Debug)]
pub struct Message;

impl Message {
    #[must_use]
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        None
    }
}

#[derive(Clone, Debug)]
pub enum KeyType {
    Runes,
    CtrlC,
    Esc,
    Other,
}

#[derive(Clone, Debug)]
pub struct KeyMsg {
    pub key_type: KeyType,
    pub runes: Vec<char>,
}

pub struct Program<M>(M);

impl<M> Program<M> {
    pub fn new(model: M) -> Self {
        Self(model)
    }

    /// # Errors
    ///
    /// This stub never returns an error.
    pub fn run(self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct Error;

#[must_use]
pub fn quit() -> Cmd {
    Cmd
}

pub trait Model {
    fn init(&self) -> Option<Cmd>;
    fn update(&mut self, msg: Message) -> Option<Cmd>;
    fn view(&self) -> String;
}

/// Counter model using derive macro.
///
/// The `#[state]` attribute marks the `count` field for change tracking,
/// enabling optimized re-rendering when only this field changes.
#[derive(bubbletea::Model)]
struct Counter {
    #[state]
    count: i32,
}

impl Counter {
    /// Initialize the counter at zero.
    fn init(&self) -> Option<Cmd> {
        None
    }

    /// Handle keyboard input.
    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::Runes => {
                    if let Some(&ch) = key.runes.first() {
                        match ch {
                            '+' | '=' => self.count = self.count.saturating_add(1),
                            '-' | '_' => self.count = self.count.saturating_sub(1),
                            'q' | 'Q' => return Some(quit()),
                            _ => {}
                        }
                    }
                }
                KeyType::CtrlC | KeyType::Esc => return Some(quit()),
                KeyType::Other => {}
            }
        }
        None
    }

    /// Render the counter view.
    fn view(&self) -> String {
        format!(
            "Count: {}\n\n\
             Press + / - to change, q to quit.",
            self.count
        )
    }
}

fn main() -> Result<(), bubbletea::Error> {
    let model = Counter { count: 0 };
    Program::new(model).run()?;
    Ok(())
}
