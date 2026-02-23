#![forbid(unsafe_code)]
// Crate-wide allows for doc formatting and Debug impl flexibility.
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_fields_in_debug)]
#![allow(clippy::suspicious_operation_groupings)]

//! # Bubbles
//!
//! A collection of reusable TUI components for the Bubbletea framework.
//!
//! Bubbles provides ready-to-use components including:
//! - **cursor** - Text cursor with blinking support
//! - **spinner** - Animated loading indicators with multiple styles
//! - **timer** - Countdown timer with timeout notifications
//! - **stopwatch** - Elapsed time tracking
//! - **paginator** - Pagination for lists and tables
//! - **progress** - Progress bar with gradient and animation support
//! - **viewport** - Scrollable content viewport
//! - **help** - Help view for displaying key bindings
//! - **key** - Key binding definitions and matching
//! - **runeutil** - Input sanitization utilities
//! - **textinput** - Single-line text input with suggestions
//! - **textarea** - Multi-line text editor
//! - **table** - Data table with keyboard navigation
//! - **list** - Feature-rich filterable list
//! - **filepicker** - File system browser
//!
//! ## Role in `charmed_rust`
//!
//! Bubbles is the component layer that sits on top of bubbletea and lipgloss:
//! - **bubbletea** provides the runtime and message loop.
//! - **lipgloss** provides styling used by every component.
//! - **huh** and **glow** embed bubbles components directly.
//! - **demo_showcase** uses bubbles to demonstrate real-world UI composition.
//!
//! ## Example
//!
//! ```rust,ignore
//! use bubbles::spinner::{SpinnerModel, spinners};
//!
//! let spinner = SpinnerModel::with_spinner(spinners::dot());
//! let tick_msg = spinner.tick();
//! ```

#[allow(
    clippy::missing_const_for_fn,
    clippy::needless_pass_by_value,
    clippy::use_self
)]
pub mod cursor;
#[allow(
    clippy::missing_const_for_fn,
    clippy::redundant_closure_for_method_calls,
    clippy::single_char_pattern,
    clippy::uninlined_format_args
)]
pub mod help;
#[allow(clippy::missing_const_for_fn)]
pub mod key;
#[allow(
    clippy::missing_const_for_fn,
    clippy::needless_pass_by_value,
    clippy::use_self
)]
pub mod paginator;
#[allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::float_cmp,
    clippy::format_push_string,
    clippy::literal_string_with_formatting_args,
    clippy::many_single_char_names,
    clippy::missing_const_for_fn,
    clippy::needless_pass_by_value,
    clippy::option_if_let_else,
    clippy::suboptimal_flops,
    clippy::uninlined_format_args,
    clippy::unnecessary_wraps,
    clippy::use_self
)]
pub mod progress;
pub mod runeutil;
#[allow(
    clippy::missing_const_for_fn,
    clippy::needless_pass_by_value,
    clippy::use_self
)]
pub mod spinner;
#[allow(
    clippy::duration_suboptimal_units,
    clippy::missing_const_for_fn,
    clippy::must_use_candidate,
    clippy::needless_pass_by_value,
    clippy::redundant_else,
    clippy::uninlined_format_args,
    clippy::use_self
)]
pub mod stopwatch;
#[allow(
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::missing_const_for_fn,
    clippy::range_plus_one,
    clippy::redundant_closure_for_method_calls,
    clippy::single_char_pattern,
    clippy::too_many_lines,
    clippy::use_self
)]
pub mod textarea;
#[allow(
    clippy::if_not_else,
    clippy::missing_const_for_fn,
    clippy::range_plus_one,
    clippy::single_char_pattern,
    clippy::uninlined_format_args,
    clippy::use_self
)]
pub mod textinput;
#[allow(
    clippy::duration_suboptimal_units,
    clippy::missing_const_for_fn,
    clippy::needless_pass_by_value,
    clippy::redundant_else,
    clippy::uninlined_format_args
)]
pub mod timer;
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::missing_const_for_fn,
    clippy::use_self
)]
pub mod viewport;

// Complex components
#[allow(
    clippy::assigning_clones,
    clippy::cast_precision_loss,
    clippy::missing_const_for_fn,
    clippy::must_use_candidate,
    clippy::needless_pass_by_value,
    clippy::struct_excessive_bools,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    clippy::unreadable_literal,
    clippy::use_self
)]
pub mod filepicker;
#[allow(
    clippy::format_push_string,
    clippy::missing_const_for_fn,
    clippy::must_use_candidate,
    clippy::struct_excessive_bools,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    clippy::unnecessary_literal_bound,
    clippy::use_self
)]
pub mod list;
#[allow(
    clippy::map_unwrap_or,
    clippy::missing_const_for_fn,
    clippy::uninlined_format_args,
    clippy::use_self
)]
pub mod table;

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::cursor::{Cursor, Mode as CursorMode, blink_cmd};
    pub use crate::help::Help;
    pub use crate::key::{Binding, Help as KeyHelp, matches};
    pub use crate::paginator::{Paginator, Type as PaginatorType};
    pub use crate::progress::Progress;
    pub use crate::runeutil::Sanitizer;
    pub use crate::spinner::{Spinner, SpinnerModel, spinners};
    pub use crate::stopwatch::Stopwatch;
    pub use crate::textarea::TextArea;
    pub use crate::textinput::TextInput;
    pub use crate::timer::Timer;
    pub use crate::viewport::Viewport;

    // Complex components
    pub use crate::filepicker::{DirEntry, FilePicker, ReadDirErrMsg, ReadDirMsg};
    pub use crate::list::{DefaultDelegate, FilterState, Item, ItemDelegate, List};
    pub use crate::table::{Column, Row, Table};
}
