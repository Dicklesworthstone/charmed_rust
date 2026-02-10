//! Generic model example demonstrating `#[derive(Model)]` with type parameters.
//!
//! This example shows how to use the derive macro with generic structs,
//! including proper trait bounds.
//!
//! Run with: `cargo run -p charmed-bubbletea-macros --example generic_model`

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

use std::fmt::Display;

/// Generic container model that can hold any displayable value.
///
/// The type parameter `T` must implement:
/// - `Clone` - for state snapshot creation
/// - `PartialEq` - for change detection
/// - `Display` - for rendering in the view
/// - `Default` - for resetting the value
/// - `Send` + `'static` - required by the Model trait
#[derive(bubbletea::Model)]
struct Container<T>
where
    T: Clone + PartialEq + Display + Default + Send + 'static,
{
    #[state]
    value: T,

    #[state]
    label: String,
}

impl<T> Container<T>
where
    T: Clone + PartialEq + Display + Default + Send + 'static,
{
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::Runes => {
                    if let Some(&ch) = key.runes.first() {
                        match ch {
                            'r' | 'R' => self.value = T::default(),
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

    fn view(&self) -> String {
        format!(
            "{}: {}\n\n\
             Press r to reset, q to quit",
            self.label, self.value
        )
    }
}

fn main() -> Result<(), bubbletea::Error> {
    // Use the generic container with a String value
    let model = Container {
        value: String::from("Hello, Generic Model!"),
        label: String::from("Value"),
    };
    Program::new(model).run()?;
    Ok(())
}
