#![forbid(unsafe_code)]
#![allow(clippy::doc_markdown)] // Module-level docs reference crate names like bubbletea/lipgloss.

//! # Bubbletea
//!
//! A powerful TUI (Terminal User Interface) framework based on The Elm Architecture.
//!
//! Bubbletea provides a functional approach to building terminal applications with:
//! - A simple **Model-Update-View** architecture
//! - **Command-based** side effects
//! - **Type-safe messages** with downcasting
//! - Full **keyboard and mouse** support
//! - **Frame-rate limited** rendering (60 FPS default)
//!
//! ## Role in `charmed_rust`
//!
//! Bubbletea is the core runtime and event loop for the entire ecosystem:
//! - **bubbles** builds reusable widgets on top of the Model/Msg/Cmd pattern.
//! - **huh** composes form flows using bubbletea models.
//! - **wish** serves bubbletea programs over SSH.
//! - **glow** uses bubbletea for pager-style Markdown viewing.
//! - **demo_showcase** is the flagship multi-page bubbletea app.
//!
//! ## The Elm Architecture
//!
//! Bubbletea follows the Elm Architecture pattern:
//!
//! - **Model**: Your application state
//! - **Update**: A pure function that processes messages and returns commands
//! - **View**: A pure function that renders state to a string
//! - **Cmd**: Lazy IO operations that produce messages
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use bubbletea::{Program, Model, Message, Cmd, KeyMsg, KeyType};
//!
//! struct Counter {
//!     count: i32,
//! }
//!
//! struct IncrementMsg;
//! struct DecrementMsg;
//!
//! impl Model for Counter {
//!     fn init(&self) -> Option<Cmd> {
//!         None
//!     }
//!
//!     fn update(&mut self, msg: Message) -> Option<Cmd> {
//!         if msg.is::<IncrementMsg>() {
//!             self.count += 1;
//!         } else if msg.is::<DecrementMsg>() {
//!             self.count -= 1;
//!         } else if let Some(key) = msg.downcast_ref::<KeyMsg>() {
//!             match key.key_type {
//!                 KeyType::CtrlC | KeyType::Esc => return Some(bubbletea::quit()),
//!                 KeyType::Runes if key.runes == vec!['q'] => return Some(bubbletea::quit()),
//!                 _ => {}
//!             }
//!         }
//!         None
//!     }
//!
//!     fn view(&self) -> String {
//!         format!(
//!             "Count: {}\n\nPress +/- to change, q to quit",
//!             self.count
//!         )
//!     }
//! }
//!
//! fn main() -> Result<(), bubbletea::Error> {
//!     let model = Counter { count: 0 };
//!     let final_model = Program::new(model)
//!         .with_alt_screen()
//!         .run()?;
//!     println!("Final count: {}", final_model.count);
//!     Ok(())
//! }
//! ```
//!
//! ## Messages
//!
//! Messages are type-erased using [`Message`]. You can create custom message types
//! and downcast them in your update function:
//!
//! ```rust
//! use bubbletea::Message;
//!
//! struct MyCustomMsg { value: i32 }
//!
//! let msg = Message::new(MyCustomMsg { value: 42 });
//!
//! // Check type
//! if msg.is::<MyCustomMsg>() {
//!     // Downcast to access
//!     if let Some(custom) = msg.downcast::<MyCustomMsg>() {
//!         assert_eq!(custom.value, 42);
//!     }
//! }
//! ```
//!
//! ## Commands
//!
//! Commands are lazy IO operations that produce messages:
//!
//! ```rust
//! use bubbletea::{Cmd, Message, batch, sequence};
//! use std::time::Duration;
//!
//! // Simple command
//! let cmd = Cmd::new(|| Message::new("done"));
//!
//! // Batch commands (run concurrently)
//! let cmds = batch(vec![
//!     Some(Cmd::new(|| Message::new(1))),
//!     Some(Cmd::new(|| Message::new(2))),
//! ]);
//!
//! // Sequence commands (run in order)
//! let cmds = sequence(vec![
//!     Some(Cmd::new(|| Message::new(1))),
//!     Some(Cmd::new(|| Message::new(2))),
//! ]);
//! ```
//!
//! ## Keyboard Input
//!
//! Keyboard events are delivered as [`KeyMsg`]:
//!
//! ```rust
//! use bubbletea::{KeyMsg, KeyType, Message};
//!
//! fn handle_key(msg: Message) {
//!     if let Some(key) = msg.downcast_ref::<KeyMsg>() {
//!         match key.key_type {
//!             KeyType::Enter => println!("Enter pressed"),
//!             KeyType::CtrlC => println!("Ctrl+C pressed"),
//!             KeyType::Runes => println!("Typed: {:?}", key.runes),
//!             _ => {}
//!         }
//!     }
//! }
//! ```
//!
//! ## Mouse Input
//!
//! Enable mouse tracking with `with_mouse_cell_motion()` or `with_mouse_all_motion()`:
//!
//! ```rust,ignore
//! use bubbletea::{Program, MouseMsg, MouseButton, MouseAction};
//!
//! let program = Program::new(model)
//!     .with_mouse_cell_motion()  // Track clicks and drags
//!     .run()?;
//!
//! // In update:
//! if let Some(mouse) = msg.downcast_ref::<MouseMsg>() {
//!     if mouse.button == MouseButton::Left && mouse.action == MouseAction::Press {
//!         println!("Click at ({}, {})", mouse.x, mouse.y);
//!     }
//! }
//! ```
//!
//! ## Screen Control
//!
//! Control terminal features with screen commands:
//!
//! ```rust
//! use bubbletea::screen;
//!
//! // In update, return a command:
//! let cmd = screen::enter_alt_screen();
//! let cmd = screen::hide_cursor();
//! let cmd = screen::enable_mouse_cell_motion();
//! ```

#[allow(
    clippy::branches_sharing_code,
    clippy::cast_possible_truncation,
    clippy::map_unwrap_or,
    clippy::missing_const_for_fn,
    clippy::must_use_candidate,
    clippy::use_self
)]
pub mod command;
#[allow(
    clippy::doc_markdown,
    clippy::manual_let_else,
    clippy::missing_const_for_fn,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    clippy::use_self
)]
pub mod key;
#[allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::uninlined_format_args
)]
pub mod message;
#[allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::doc_markdown,
    clippy::items_after_statements,
    clippy::match_same_arms,
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::uninlined_format_args,
    clippy::use_self
)]
pub mod mouse;
#[allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::collapsible_if,
    clippy::doc_markdown,
    clippy::if_not_else,
    clippy::ignored_unit_patterns,
    clippy::items_after_statements,
    clippy::map_unwrap_or,
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::needless_pass_by_value,
    clippy::option_if_let_else,
    clippy::redundant_clone,
    clippy::redundant_closure_for_method_calls,
    clippy::return_self_not_must_use,
    clippy::semicolon_if_nothing_returned,
    clippy::stable_sort_primitive,
    clippy::struct_excessive_bools,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    clippy::unused_self,
    clippy::useless_let_if_seq
)]
pub mod program;
#[allow(clippy::doc_markdown, clippy::must_use_candidate)]
pub mod screen;
#[allow(
    clippy::doc_markdown,
    clippy::missing_const_for_fn,
    clippy::must_use_candidate,
    clippy::unreadable_literal
)]
pub mod simulator;

// Re-exports
pub use command::{
    Cmd, batch, every, printf, println, quit, sequence, set_window_title, tick, window_size,
};

#[cfg(feature = "async")]
pub use command::{AsyncCmd, every_async, tick_async};
pub use key::{KeyMsg, KeyType, parse_sequence, parse_sequence_prefix};
pub use message::{
    BlurMsg, FocusMsg, InterruptMsg, Message, QuitMsg, ResumeMsg, SuspendMsg, WindowSizeMsg,
};
pub use mouse::{MouseAction, MouseButton, MouseMsg, parse_mouse_event_sequence};
pub use program::{Error, Model, Program, ProgramHandle, ProgramOptions, Result};

// Re-export derive macro when macros feature is enabled.
// Derive macros and traits live in different namespaces, so both can be named `Model`.
// Users can write `#[derive(bubbletea::Model)]` for the macro and `impl bubbletea::Model` for the trait.
#[cfg(feature = "macros")]
#[doc(hidden)]
pub use bubbletea_macros::*;

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::command::{Cmd, batch, every, printf, println, quit, sequence, tick};
    pub use crate::key::{KeyMsg, KeyType};
    pub use crate::message::{Message, QuitMsg, WindowSizeMsg};
    pub use crate::mouse::{MouseAction, MouseButton, MouseMsg};
    pub use crate::program::{Model, Program};

    #[cfg(feature = "async")]
    pub use crate::command::{AsyncCmd, every_async, tick_async};
}
