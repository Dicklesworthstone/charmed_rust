#![forbid(unsafe_code)]
#![allow(clippy::nursery)]
#![allow(clippy::pedantic)]

//! # bubbletea-macros
//!
//! Procedural macros for the bubbletea TUI framework.
//!
//! This crate provides the `#[derive(Model)]` macro which reduces boilerplate
//! when implementing the `Model` trait for your TUI applications.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use bubbletea::{Cmd, Message, Model};
//!
//! #[derive(Model)]
//! struct Counter {
//!     #[state]  // Marks fields for optimized change detection
//!     count: i32,
//! }
//!
//! impl Counter {
//!     fn init(&self) -> Option<Cmd> {
//!         None
//!     }
//!
//!     fn update(&mut self, msg: Message) -> Option<Cmd> {
//!         if let Some(&delta) = msg.downcast_ref::<i32>() {
//!             self.count += delta;
//!         }
//!         None
//!     }
//!
//!     fn view(&self) -> String {
//!         format!("Count: {}", self.count)
//!     }
//! }
//! ```
//!
//! ## How It Works
//!
//! The `#[derive(Model)]` macro generates a `Model` trait implementation
//! that delegates to inherent methods on your struct. You implement the
//! `init`, `update`, and `view` methods directly on your struct, and the
//! macro bridges them to the trait.
//!
//! ### Required Methods
//!
//! Your struct must implement these inherent methods:
//!
//! | Method | Signature | Purpose |
//! |--------|-----------|---------|
//! | `init` | `fn init(&self) -> Option<Cmd>` | Initial command on startup |
//! | `update` | `fn update(&mut self, msg: Message) -> Option<Cmd>` | Handle messages |
//! | `view` | `fn view(&self) -> String` | Render the UI |
//!
//! ## State Tracking with `#[state]`
//!
//! The `#[state]` attribute enables optimized re-rendering by tracking which
//! fields trigger view updates. Fields marked with `#[state]` are monitored
//! for changes, and only changes to these fields signal that a re-render
//! is needed.
//!
//! ### Basic Usage
//!
//! ```rust,ignore
//! #[derive(Model)]
//! struct App {
//!     #[state]
//!     counter: i32,      // Changes trigger re-render
//!
//!     cache: String,     // Not tracked (no re-render on change)
//! }
//! ```
//!
//! ### Advanced State Options
//!
//! ```rust,ignore
//! #[derive(Model)]
//! struct App {
//!     // Custom equality function for floating-point comparison
//!     #[state(eq = "float_approx_eq")]
//!     progress: f64,
//!
//!     // Excluded from change detection (internal bookkeeping)
//!     #[state(skip)]
//!     last_tick: std::time::Instant,
//!
//!     // Debug logging when this field changes
//!     #[state(debug)]
//!     selected_index: usize,
//! }
//!
//! fn float_approx_eq(a: &f64, b: &f64) -> bool {
//!     (a - b).abs() < 0.001
//! }
//! ```
//!
//! ### State Options
//!
//! | Option | Description |
//! |--------|-------------|
//! | `eq = "fn_name"` | Custom equality function `fn(&T, &T) -> bool` |
//! | `skip` | Exclude field from change detection |
//! | `debug` | Log changes to this field (debug builds only) |
//!
//! ## Generated Code
//!
//! The macro generates:
//!
//! 1. **Model trait implementation** - Delegates to your inherent methods
//! 2. **State snapshot struct** - For change detection (only if `#[state]` fields exist)
//! 3. **Helper methods** - `__snapshot_state()` and `__state_changed()`
//!
//! ## Generic Structs
//!
//! The derive macro supports generic structs with type parameters:
//!
//! ```rust,ignore
//! #[derive(Model)]
//! struct Container<T: Clone + PartialEq + Send + 'static> {
//!     #[state]
//!     value: T,
//! }
//! ```
//!
//! ## Error Messages
//!
//! The macro provides helpful error messages for common mistakes:
//!
//! - Using `#[derive(Model)]` on enums, unions, or tuple structs
//! - Fields marked `#[state]` that don't implement `Clone` or `PartialEq`
//!
//! ## Migration from Manual Implementation
//!
//! **Before (manual):**
//! ```rust,ignore
//! impl Model for Counter {
//!     fn init(&self) -> Option<Cmd> { None }
//!     fn update(&mut self, msg: Message) -> Option<Cmd> {
//!         // handle message
//!         None
//!     }
//!     fn view(&self) -> String {
//!         format!("{}", self.count)
//!     }
//! }
//! ```
//!
//! **After (derive):**
//! ```rust,ignore
//! #[derive(Model)]
//! struct Counter {
//!     #[state]
//!     count: i32,
//! }
//!
//! impl Counter {
//!     fn init(&self) -> Option<Cmd> { None }
//!     fn update(&mut self, msg: Message) -> Option<Cmd> {
//!         // handle message
//!         None
//!     }
//!     fn view(&self) -> String {
//!         format!("{}", self.count)
//!     }
//! }
//! ```
//!
//! The benefit is automatic state change tracking and cleaner separation
//! of your model logic from the trait boilerplate.

use proc_macro::TokenStream;
use proc_macro_error2::proc_macro_error;

mod attributes;
mod error;
mod model;
mod state;

/// Derive macro for implementing the `Model` trait.
///
/// This macro generates a `Model` trait implementation that delegates to
/// inherent methods named `init`, `update`, and `view` on your struct.
///
/// # Requirements
///
/// - Applied to a named struct (not enum, union, tuple struct, or unit struct)
/// - Struct must implement `init(&self) -> Option<Cmd>` as an inherent method
/// - Struct must implement `update(&mut self, msg: Message) -> Option<Cmd>`
/// - Struct must implement `view(&self) -> String`
///
/// # Field Attributes
///
/// - `#[state]` - Track field for change detection
/// - `#[state(eq = "fn_name")]` - Use custom equality function
/// - `#[state(skip)]` - Exclude from change detection
/// - `#[state(debug)]` - Log changes in debug builds
///
/// # Example
///
/// ```rust,ignore
/// use bubbletea::{Cmd, Message, Model};
///
/// #[derive(Model)]
/// struct Counter {
///     #[state]
///     count: i32,
/// }
///
/// impl Counter {
///     fn init(&self) -> Option<Cmd> {
///         None
///     }
///
///     fn update(&mut self, msg: Message) -> Option<Cmd> {
///         if let Some(&delta) = msg.downcast_ref::<i32>() {
///             self.count += delta;
///         }
///         None
///     }
///
///     fn view(&self) -> String {
///         format!("Count: {}", self.count)
///     }
/// }
/// ```
#[proc_macro_derive(Model, attributes(state, init, update, view, model))]
#[proc_macro_error]
pub fn derive_model(input: TokenStream) -> TokenStream {
    model::derive_model_impl(input.into()).into()
}
