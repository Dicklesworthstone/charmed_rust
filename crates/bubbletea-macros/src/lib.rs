#![forbid(unsafe_code)]
#![allow(clippy::nursery)]
#![allow(clippy::pedantic)]

//! # bubbletea-macros
//!
//! Procedural macros for the bubbletea TUI framework.
//!
//! This crate provides derive macros to reduce boilerplate when implementing
//! the Model trait for bubbletea applications.
//!
//! ## Example
//!
//! ```rust,ignore
//! use bubbletea_macros::Model;
//!
//! #[derive(Model)]
//! struct Counter {
//!     #[state]
//!     count: i32,
//! }
//!
//! impl Counter {
//!     #[init]
//!     fn init() -> (Self, Command<Msg>) {
//!         (Counter { count: 0 }, Command::none())
//!     }
//!
//!     #[update]
//!     fn update(&mut self, msg: Msg) -> Command<Msg> {
//!         match msg {
//!             Msg::Increment => self.count += 1,
//!             Msg::Decrement => self.count -= 1,
//!         }
//!         Command::none()
//!     }
//!
//!     #[view]
//!     fn view(&self) -> String {
//!         format!("Count: {}", self.count)
//!     }
//! }
//! ```

use proc_macro::TokenStream;
use proc_macro_error2::proc_macro_error;

mod attributes;
mod error;
mod model;

/// Derive macro for implementing the Model trait.
///
/// This macro generates the boilerplate code needed to implement bubbletea's
/// Model trait for a struct, based on methods annotated with `#[init]`,
/// `#[update]`, and `#[view]` attributes.
///
/// # Attributes
///
/// - `#[state]` - Mark fields that are part of the component's state
/// - `#[init]` - Mark the initialization method
/// - `#[update]` - Mark the update method for handling messages
/// - `#[view]` - Mark the view method for rendering
///
/// # Example
///
/// ```rust,ignore
/// use bubbletea_macros::Model;
///
/// #[derive(Model)]
/// struct MyApp {
///     #[state]
///     text: String,
/// }
/// ```
#[proc_macro_derive(Model, attributes(state, init, update, view))]
#[proc_macro_error]
pub fn derive_model(input: TokenStream) -> TokenStream {
    model::derive_model_impl(input.into()).into()
}
