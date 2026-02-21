#![forbid(unsafe_code)]
// Allow pedantic lints for early-stage API ergonomics.
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

#[allow(clippy::pedantic, clippy::nursery)]
pub mod cursor;
#[allow(clippy::pedantic, clippy::nursery)]
pub mod help;
#[allow(clippy::pedantic, clippy::nursery)]
pub mod key;
#[allow(clippy::pedantic, clippy::nursery)]
pub mod paginator;
#[allow(clippy::pedantic, clippy::nursery)]
pub mod progress;
pub mod runeutil;
#[allow(clippy::pedantic, clippy::nursery)]
pub mod spinner;
#[allow(clippy::pedantic, clippy::nursery)]
pub mod stopwatch;
#[allow(clippy::pedantic, clippy::nursery)]
pub mod textarea;
#[allow(clippy::pedantic, clippy::nursery)]
pub mod textinput;
#[allow(clippy::pedantic, clippy::nursery)]
pub mod timer;
#[allow(clippy::pedantic, clippy::nursery)]
pub mod viewport;

// Complex components
#[allow(clippy::pedantic, clippy::nursery)]
pub mod filepicker;
#[allow(clippy::pedantic, clippy::nursery)]
pub mod list;
#[allow(clippy::pedantic, clippy::nursery)]
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
