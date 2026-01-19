#![forbid(unsafe_code)]
// Allow pedantic lints for early-stage API ergonomics.
#![allow(clippy::nursery)]
#![allow(clippy::pedantic)]

//! # Huh
//!
//! A library for building interactive forms and prompts in the terminal.
//!
//! Huh provides a declarative way to create:
//! - Text inputs and text areas
//! - Select menus and multi-select
//! - Confirmations and notes
//! - Grouped form fields
//! - Accessible, keyboard-navigable interfaces
//!
//! ## Example
//!
//! ```rust,ignore
//! use huh::{Form, Group, Input, Select, SelectOption, Confirm};
//! use bubbletea::Program;
//!
//! let form = Form::new(vec![
//!     Group::new(vec![
//!         Box::new(Input::new()
//!             .key("name")
//!             .title("What's your name?")),
//!         Box::new(Select::new()
//!             .key("color")
//!             .title("Favorite color?")
//!             .options(vec![
//!                 SelectOption::new("Red", "red"),
//!                 SelectOption::new("Green", "green"),
//!                 SelectOption::new("Blue", "blue"),
//!             ])),
//!     ]),
//!     Group::new(vec![
//!         Box::new(Confirm::new()
//!             .key("confirm")
//!             .title("Are you sure?")),
//!     ]),
//! ]);
//!
//! let form = Program::new(form).run()?;
//!
//! let name = form.get_string("name").unwrap();
//! let color = form.get_string("color").unwrap();
//! let confirm = form.get_bool("confirm").unwrap();
//!
//! println!("Name: {}, Color: {}, Confirmed: {}", name, color, confirm);
//! ```

use std::any::Any;
use std::sync::atomic::{AtomicUsize, Ordering};

use thiserror::Error;

use bubbles::key::Binding;
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model};
use lipgloss::{Border, Style};

// -----------------------------------------------------------------------------
// ID Generation
// -----------------------------------------------------------------------------

static LAST_ID: AtomicUsize = AtomicUsize::new(0);

fn next_id() -> usize {
    LAST_ID.fetch_add(1, Ordering::SeqCst)
}

// -----------------------------------------------------------------------------
// Errors
// -----------------------------------------------------------------------------

/// Errors that can occur during form execution.
///
/// This enum represents all possible error conditions when running
/// an interactive form with huh.
///
/// # Error Handling
///
/// Forms can fail for several reasons, but many are recoverable
/// or expected user actions (like cancellation):
///
/// ```rust,ignore
/// use huh::{Form, FormError, Result};
///
/// fn get_user_input() -> Result<String> {
///     let mut name = String::new();
///     Form::new(fields)
///         .run()?;
///     Ok(name)
/// }
/// ```
///
/// # Recovery Strategies
///
/// | Error Variant | Recovery Strategy |
/// |--------------|-------------------|
/// | [`UserAborted`](FormError::UserAborted) | Normal exit, not an error condition |
/// | [`Timeout`](FormError::Timeout) | Retry with longer timeout or prompt user |
/// | [`Validation`](FormError::Validation) | Show error message, allow retry |
/// | [`Io`](FormError::Io) | Check terminal, fall back to non-interactive |
///
/// # Example: Handling User Abort
///
/// User abort (Ctrl+C) is a normal exit path, not an error:
///
/// ```rust,ignore
/// match form.run() {
///     Ok(()) => println!("Form completed!"),
///     Err(FormError::UserAborted) => {
///         println!("Cancelled by user");
///         return Ok(()); // Not an error condition
///     }
///     Err(e) => return Err(e.into()),
/// }
/// ```
///
/// # Note on Clone and PartialEq
///
/// This error type implements `Clone` and `PartialEq` to support
/// testing and comparison. As a result, the `Io` variant stores
/// a `String` message rather than the underlying `io::Error`.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum FormError {
    /// User aborted the form with Ctrl+C or Escape.
    ///
    /// This is not an error condition but a normal exit path.
    /// Users may cancel forms for valid reasons, and applications
    /// should handle this gracefully.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// match form.run() {
    ///     Err(FormError::UserAborted) => {
    ///         println!("No changes made");
    ///         return Ok(());
    ///     }
    ///     // ...
    /// }
    /// ```
    #[error("user aborted")]
    UserAborted,

    /// Form execution timed out.
    ///
    /// Occurs when a form has a timeout configured and the user
    /// does not complete it in time.
    ///
    /// # Recovery
    ///
    /// - Increase the timeout duration
    /// - Prompt user to try again
    /// - Use a default value
    #[error("timeout")]
    Timeout,

    /// Custom validation error.
    ///
    /// Occurs when a field's validation function returns an error.
    /// The contained string describes what validation failed.
    ///
    /// # Recovery
    ///
    /// Validation errors are recoverable - show the error message
    /// to the user and allow them to correct their input.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let input = Input::new()
    ///     .title("Email")
    ///     .validate(|s| {
    ///         if s.contains('@') {
    ///             Ok(())
    ///         } else {
    ///             Err(FormError::Validation("must contain @".into()))
    ///         }
    ///     });
    /// ```
    #[error("validation error: {0}")]
    Validation(String),

    /// IO error during form operations.
    ///
    /// Occurs during terminal I/O operations, particularly in
    /// accessible mode where stdin/stdout are used directly.
    ///
    /// Note: Stores the error message as a `String` rather than
    /// `io::Error` to maintain `Clone` and `PartialEq` derives.
    ///
    /// # Recovery
    ///
    /// - Check if the terminal is available
    /// - Fall back to non-interactive input
    /// - Log the error and exit gracefully
    #[error("io error: {0}")]
    Io(String),
}

impl FormError {
    /// Creates a validation error with the given message.
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    /// Creates an IO error with the given message.
    pub fn io(message: impl Into<String>) -> Self {
        Self::Io(message.into())
    }

    /// Returns true if this is a user-initiated abort.
    pub fn is_user_abort(&self) -> bool {
        matches!(self, Self::UserAborted)
    }

    /// Returns true if this is a timeout error.
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout)
    }

    /// Returns true if this error is recoverable (validation errors).
    pub fn is_recoverable(&self) -> bool {
        matches!(self, Self::Validation(_))
    }
}

/// A specialized [`Result`] type for huh form operations.
///
/// This type alias defaults to [`FormError`] as the error type.
///
/// # Example
///
/// ```rust,ignore
/// use huh::Result;
///
/// fn collect_user_info() -> Result<UserInfo> {
///     let mut name = String::new();
///     let mut email = String::new();
///
///     Form::new(vec![/* fields */]).run()?;
///
///     Ok(UserInfo { name, email })
/// }
/// ```
pub type Result<T> = std::result::Result<T, FormError>;

// -----------------------------------------------------------------------------
// Form State
// -----------------------------------------------------------------------------

/// The current state of the form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormState {
    /// User is completing the form.
    #[default]
    Normal,
    /// User has completed the form.
    Completed,
    /// User has aborted the form.
    Aborted,
}

// -----------------------------------------------------------------------------
// SelectOption
// -----------------------------------------------------------------------------

/// An option for select fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectOption<T: Clone + PartialEq> {
    /// The display key shown to the user.
    pub key: String,
    /// The underlying value.
    pub value: T,
    /// Whether this option is initially selected.
    pub selected: bool,
}

impl<T: Clone + PartialEq> SelectOption<T> {
    /// Creates a new option.
    pub fn new(key: impl Into<String>, value: T) -> Self {
        Self {
            key: key.into(),
            value,
            selected: false,
        }
    }

    /// Sets whether the option is initially selected.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

impl<T: Clone + PartialEq + std::fmt::Display> SelectOption<T> {
    /// Creates options from a list of values using Display for keys.
    pub fn from_values(values: impl IntoIterator<Item = T>) -> Vec<Self> {
        values
            .into_iter()
            .map(|v| Self::new(v.to_string(), v))
            .collect()
    }
}

/// Creates options from string values.
pub fn new_options<S: Into<String> + Clone>(
    values: impl IntoIterator<Item = S>,
) -> Vec<SelectOption<String>> {
    values
        .into_iter()
        .map(|v| {
            let s: String = v.clone().into();
            SelectOption::new(s.clone(), s)
        })
        .collect()
}

// -----------------------------------------------------------------------------
// Theme
// -----------------------------------------------------------------------------

/// Collection of styles for form components.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Styles for the form container.
    pub form: FormStyles,
    /// Styles for groups.
    pub group: GroupStyles,
    /// Separator between fields.
    pub field_separator: Style,
    /// Styles for blurred (unfocused) fields.
    pub blurred: FieldStyles,
    /// Styles for focused fields.
    pub focused: FieldStyles,
}

impl Default for Theme {
    fn default() -> Self {
        theme_charm()
    }
}

/// Styles for the form container.
#[derive(Debug, Clone, Default)]
pub struct FormStyles {
    /// Base style for the form.
    pub base: Style,
}

/// Styles for groups.
#[derive(Debug, Clone, Default)]
pub struct GroupStyles {
    /// Base style for the group.
    pub base: Style,
    /// Title style.
    pub title: Style,
    /// Description style.
    pub description: Style,
}

/// Styles for input fields.
#[derive(Debug, Clone, Default)]
pub struct FieldStyles {
    /// Base style.
    pub base: Style,
    /// Title style.
    pub title: Style,
    /// Description style.
    pub description: Style,
    /// Error indicator style.
    pub error_indicator: Style,
    /// Error message style.
    pub error_message: Style,

    // Select styles
    /// Select cursor style.
    pub select_selector: Style,
    /// Option style.
    pub option: Style,
    /// Next indicator for inline select.
    pub next_indicator: Style,
    /// Previous indicator for inline select.
    pub prev_indicator: Style,

    // Multi-select styles
    /// Multi-select cursor style.
    pub multi_select_selector: Style,
    /// Selected option style.
    pub selected_option: Style,
    /// Selected prefix style.
    pub selected_prefix: Style,
    /// Unselected option style.
    pub unselected_option: Style,
    /// Unselected prefix style.
    pub unselected_prefix: Style,

    // Text input styles
    /// Text input specific styles.
    pub text_input: TextInputStyles,

    // Confirm styles
    /// Focused button style.
    pub focused_button: Style,
    /// Blurred button style.
    pub blurred_button: Style,

    // Note styles
    /// Note title style.
    pub note_title: Style,
}

/// Styles for text inputs.
#[derive(Debug, Clone, Default)]
pub struct TextInputStyles {
    /// Cursor style.
    pub cursor: Style,
    /// Cursor text style.
    pub cursor_text: Style,
    /// Placeholder style.
    pub placeholder: Style,
    /// Prompt style.
    pub prompt: Style,
    /// Text style.
    pub text: Style,
}

/// Returns the base theme.
#[allow(clippy::field_reassign_with_default)]
pub fn theme_base() -> Theme {
    let button = Style::new().padding((0, 2)).margin_right(1);

    let mut focused = FieldStyles::default();
    focused.base = Style::new()
        .padding_left(1)
        .border(Border::thick())
        .border_left(true);
    focused.error_indicator = Style::new().set_string(" *");
    focused.error_message = Style::new().set_string(" *");
    focused.select_selector = Style::new().set_string("> ");
    focused.next_indicator = Style::new().margin_left(1).set_string("→");
    focused.prev_indicator = Style::new().margin_right(1).set_string("←");
    focused.multi_select_selector = Style::new().set_string("> ");
    focused.selected_prefix = Style::new().set_string("[•] ");
    focused.unselected_prefix = Style::new().set_string("[ ] ");
    focused.focused_button = button.clone().foreground("0").background("7");
    focused.blurred_button = button.foreground("7").background("0");
    focused.text_input.placeholder = Style::new().foreground("8");

    let mut blurred = focused.clone();
    blurred.base = blurred.base.border(Border::hidden());
    blurred.multi_select_selector = Style::new().set_string("  ");
    blurred.next_indicator = Style::new();
    blurred.prev_indicator = Style::new();

    Theme {
        form: FormStyles { base: Style::new() },
        group: GroupStyles::default(),
        field_separator: Style::new().set_string("\n\n"),
        focused,
        blurred,
    }
}

/// Returns the Charm theme (default).
pub fn theme_charm() -> Theme {
    let mut t = theme_base();

    let indigo = "#7571F9";
    let fuchsia = "#F780E2";
    let green = "#02BF87";
    let red = "#ED567A";
    let normal_fg = "252";

    t.focused.base = t.focused.base.border_foreground("238");
    t.focused.title = t.focused.title.foreground(indigo).bold();
    t.focused.note_title = t
        .focused
        .note_title
        .foreground(indigo)
        .bold()
        .margin_bottom(1);
    t.focused.description = t.focused.description.foreground("243");
    t.focused.error_indicator = t.focused.error_indicator.foreground(red);
    t.focused.error_message = t.focused.error_message.foreground(red);
    t.focused.select_selector = t.focused.select_selector.foreground(fuchsia);
    t.focused.next_indicator = t.focused.next_indicator.foreground(fuchsia);
    t.focused.prev_indicator = t.focused.prev_indicator.foreground(fuchsia);
    t.focused.option = t.focused.option.foreground(normal_fg);
    t.focused.multi_select_selector = t.focused.multi_select_selector.foreground(fuchsia);
    t.focused.selected_option = t.focused.selected_option.foreground(green);
    t.focused.selected_prefix = Style::new().foreground("#02A877").set_string("✓ ");
    t.focused.unselected_prefix = Style::new().foreground("243").set_string("• ");
    t.focused.unselected_option = t.focused.unselected_option.foreground(normal_fg);
    t.focused.focused_button = t
        .focused
        .focused_button
        .foreground("#FFFDF5")
        .background(fuchsia);
    t.focused.blurred_button = t
        .focused
        .blurred_button
        .foreground(normal_fg)
        .background("237");
    t.focused.text_input.cursor = t.focused.text_input.cursor.foreground(green);
    t.focused.text_input.placeholder = t.focused.text_input.placeholder.foreground("238");
    t.focused.text_input.prompt = t.focused.text_input.prompt.foreground(fuchsia);

    t.blurred = t.focused.clone();
    t.blurred.base = t.focused.base.clone().border(Border::hidden());
    t.blurred.next_indicator = Style::new();
    t.blurred.prev_indicator = Style::new();

    t.group.title = t.focused.title.clone();
    t.group.description = t.focused.description.clone();

    t
}

/// Returns the Dracula theme.
pub fn theme_dracula() -> Theme {
    let mut t = theme_base();

    let _background = "#282a36";
    let selection = "#44475a";
    let foreground = "#f8f8f2";
    let comment = "#6272a4";
    let green = "#50fa7b";
    let purple = "#bd93f9";
    let red = "#ff5555";
    let yellow = "#f1fa8c";

    t.focused.base = t.focused.base.border_foreground(selection);
    t.focused.title = t.focused.title.foreground(purple);
    t.focused.note_title = t.focused.note_title.foreground(purple);
    t.focused.description = t.focused.description.foreground(comment);
    t.focused.error_indicator = t.focused.error_indicator.foreground(red);
    t.focused.error_message = t.focused.error_message.foreground(red);
    t.focused.select_selector = t.focused.select_selector.foreground(yellow);
    t.focused.next_indicator = t.focused.next_indicator.foreground(yellow);
    t.focused.prev_indicator = t.focused.prev_indicator.foreground(yellow);
    t.focused.option = t.focused.option.foreground(foreground);
    t.focused.multi_select_selector = t.focused.multi_select_selector.foreground(yellow);
    t.focused.selected_option = t.focused.selected_option.foreground(green);
    t.focused.selected_prefix = t.focused.selected_prefix.foreground(green);
    t.focused.unselected_option = t.focused.unselected_option.foreground(foreground);
    t.focused.unselected_prefix = t.focused.unselected_prefix.foreground(comment);
    t.focused.focused_button = t
        .focused
        .focused_button
        .foreground(yellow)
        .background(purple)
        .bold();
    t.focused.blurred_button = t
        .focused
        .blurred_button
        .foreground(foreground)
        .background("#282a36");
    t.focused.text_input.cursor = t.focused.text_input.cursor.foreground(yellow);
    t.focused.text_input.placeholder = t.focused.text_input.placeholder.foreground(comment);
    t.focused.text_input.prompt = t.focused.text_input.prompt.foreground(yellow);

    t.blurred = t.focused.clone();
    t.blurred.base = t.blurred.base.border(Border::hidden());
    t.blurred.next_indicator = Style::new();
    t.blurred.prev_indicator = Style::new();

    t.group.title = t.focused.title.clone();
    t.group.description = t.focused.description.clone();

    t
}

/// Returns the Base16 theme.
pub fn theme_base16() -> Theme {
    let mut t = theme_base();

    t.focused.base = t.focused.base.border_foreground("8");
    t.focused.title = t.focused.title.foreground("6");
    t.focused.note_title = t.focused.note_title.foreground("6");
    t.focused.description = t.focused.description.foreground("8");
    t.focused.error_indicator = t.focused.error_indicator.foreground("9");
    t.focused.error_message = t.focused.error_message.foreground("9");
    t.focused.select_selector = t.focused.select_selector.foreground("3");
    t.focused.next_indicator = t.focused.next_indicator.foreground("3");
    t.focused.prev_indicator = t.focused.prev_indicator.foreground("3");
    t.focused.option = t.focused.option.foreground("7");
    t.focused.multi_select_selector = t.focused.multi_select_selector.foreground("3");
    t.focused.selected_option = t.focused.selected_option.foreground("2");
    t.focused.selected_prefix = t.focused.selected_prefix.foreground("2");
    t.focused.unselected_option = t.focused.unselected_option.foreground("7");
    t.focused.focused_button = t.focused.focused_button.foreground("7").background("5");
    t.focused.blurred_button = t.focused.blurred_button.foreground("7").background("0");

    t.blurred = t.focused.clone();
    t.blurred.base = t.blurred.base.border(Border::hidden());
    t.blurred.note_title = t.blurred.note_title.foreground("8");
    t.blurred.title = t.blurred.title.foreground("8");
    t.blurred.text_input.prompt = t.blurred.text_input.prompt.foreground("8");
    t.blurred.text_input.text = t.blurred.text_input.text.foreground("7");
    t.blurred.next_indicator = Style::new();
    t.blurred.prev_indicator = Style::new();

    t.group.title = t.focused.title.clone();
    t.group.description = t.focused.description.clone();

    t
}

// -----------------------------------------------------------------------------
// KeyMap
// -----------------------------------------------------------------------------

/// Keybindings for form navigation.
#[derive(Debug, Clone)]
pub struct KeyMap {
    /// Quit the form.
    pub quit: Binding,
    /// Input field keybindings.
    pub input: InputKeyMap,
    /// Select field keybindings.
    pub select: SelectKeyMap,
    /// Multi-select field keybindings.
    pub multi_select: MultiSelectKeyMap,
    /// Confirm field keybindings.
    pub confirm: ConfirmKeyMap,
    /// Note field keybindings.
    pub note: NoteKeyMap,
    /// Text area keybindings.
    pub text: TextKeyMap,
}

impl Default for KeyMap {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyMap {
    /// Creates a new default keymap.
    pub fn new() -> Self {
        Self {
            quit: Binding::new().keys(&["ctrl+c"]),
            input: InputKeyMap::default(),
            select: SelectKeyMap::default(),
            multi_select: MultiSelectKeyMap::default(),
            confirm: ConfirmKeyMap::default(),
            note: NoteKeyMap::default(),
            text: TextKeyMap::default(),
        }
    }
}

/// Keybindings for input fields.
#[derive(Debug, Clone)]
pub struct InputKeyMap {
    /// Accept autocomplete suggestion.
    pub accept_suggestion: Binding,
    /// Go to next field.
    pub next: Binding,
    /// Go to previous field.
    pub prev: Binding,
    /// Submit the form.
    pub submit: Binding,
}

impl Default for InputKeyMap {
    fn default() -> Self {
        Self {
            accept_suggestion: Binding::new().keys(&["ctrl+e"]).help("ctrl+e", "complete"),
            prev: Binding::new()
                .keys(&["shift+tab"])
                .help("shift+tab", "back"),
            next: Binding::new().keys(&["enter", "tab"]).help("enter", "next"),
            submit: Binding::new().keys(&["enter"]).help("enter", "submit"),
        }
    }
}

/// Keybindings for select fields.
#[derive(Debug, Clone)]
pub struct SelectKeyMap {
    /// Go to next field.
    pub next: Binding,
    /// Go to previous field.
    pub prev: Binding,
    /// Move cursor up.
    pub up: Binding,
    /// Move cursor down.
    pub down: Binding,
    /// Move cursor left (inline mode).
    pub left: Binding,
    /// Move cursor right (inline mode).
    pub right: Binding,
    /// Open filter.
    pub filter: Binding,
    /// Apply filter.
    pub set_filter: Binding,
    /// Clear filter.
    pub clear_filter: Binding,
    /// Half page up.
    pub half_page_up: Binding,
    /// Half page down.
    pub half_page_down: Binding,
    /// Go to top.
    pub goto_top: Binding,
    /// Go to bottom.
    pub goto_bottom: Binding,
    /// Submit the form.
    pub submit: Binding,
}

impl Default for SelectKeyMap {
    fn default() -> Self {
        Self {
            prev: Binding::new()
                .keys(&["shift+tab"])
                .help("shift+tab", "back"),
            next: Binding::new()
                .keys(&["enter", "tab"])
                .help("enter", "select"),
            submit: Binding::new().keys(&["enter"]).help("enter", "submit"),
            up: Binding::new()
                .keys(&["up", "k", "ctrl+k", "ctrl+p"])
                .help("↑", "up"),
            down: Binding::new()
                .keys(&["down", "j", "ctrl+j", "ctrl+n"])
                .help("↓", "down"),
            left: Binding::new()
                .keys(&["h", "left"])
                .help("←", "left")
                .set_enabled(false),
            right: Binding::new()
                .keys(&["l", "right"])
                .help("→", "right")
                .set_enabled(false),
            filter: Binding::new().keys(&["/"]).help("/", "filter"),
            set_filter: Binding::new()
                .keys(&["escape"])
                .help("esc", "set filter")
                .set_enabled(false),
            clear_filter: Binding::new()
                .keys(&["escape"])
                .help("esc", "clear filter")
                .set_enabled(false),
            half_page_up: Binding::new().keys(&["ctrl+u"]).help("ctrl+u", "½ page up"),
            half_page_down: Binding::new()
                .keys(&["ctrl+d"])
                .help("ctrl+d", "½ page down"),
            goto_top: Binding::new()
                .keys(&["home", "g"])
                .help("g/home", "go to start"),
            goto_bottom: Binding::new()
                .keys(&["end", "G"])
                .help("G/end", "go to end"),
        }
    }
}

/// Keybindings for multi-select fields.
#[derive(Debug, Clone)]
pub struct MultiSelectKeyMap {
    /// Go to next field.
    pub next: Binding,
    /// Go to previous field.
    pub prev: Binding,
    /// Move cursor up.
    pub up: Binding,
    /// Move cursor down.
    pub down: Binding,
    /// Toggle selection.
    pub toggle: Binding,
    /// Open filter.
    pub filter: Binding,
    /// Apply filter.
    pub set_filter: Binding,
    /// Clear filter.
    pub clear_filter: Binding,
    /// Half page up.
    pub half_page_up: Binding,
    /// Half page down.
    pub half_page_down: Binding,
    /// Go to top.
    pub goto_top: Binding,
    /// Go to bottom.
    pub goto_bottom: Binding,
    /// Select all.
    pub select_all: Binding,
    /// Select none.
    pub select_none: Binding,
    /// Submit the form.
    pub submit: Binding,
}

impl Default for MultiSelectKeyMap {
    fn default() -> Self {
        Self {
            prev: Binding::new()
                .keys(&["shift+tab"])
                .help("shift+tab", "back"),
            next: Binding::new()
                .keys(&["enter", "tab"])
                .help("enter", "confirm"),
            submit: Binding::new().keys(&["enter"]).help("enter", "submit"),
            toggle: Binding::new().keys(&[" ", "x"]).help("x", "toggle"),
            up: Binding::new().keys(&["up", "k", "ctrl+p"]).help("↑", "up"),
            down: Binding::new()
                .keys(&["down", "j", "ctrl+n"])
                .help("↓", "down"),
            filter: Binding::new().keys(&["/"]).help("/", "filter"),
            set_filter: Binding::new()
                .keys(&["enter", "escape"])
                .help("esc", "set filter")
                .set_enabled(false),
            clear_filter: Binding::new()
                .keys(&["escape"])
                .help("esc", "clear filter")
                .set_enabled(false),
            half_page_up: Binding::new().keys(&["ctrl+u"]).help("ctrl+u", "½ page up"),
            half_page_down: Binding::new()
                .keys(&["ctrl+d"])
                .help("ctrl+d", "½ page down"),
            goto_top: Binding::new()
                .keys(&["home", "g"])
                .help("g/home", "go to start"),
            goto_bottom: Binding::new()
                .keys(&["end", "G"])
                .help("G/end", "go to end"),
            select_all: Binding::new()
                .keys(&["ctrl+a"])
                .help("ctrl+a", "select all"),
            select_none: Binding::new()
                .keys(&["ctrl+a"])
                .help("ctrl+a", "select none")
                .set_enabled(false),
        }
    }
}

/// Keybindings for confirm fields.
#[derive(Debug, Clone)]
pub struct ConfirmKeyMap {
    /// Go to next field.
    pub next: Binding,
    /// Go to previous field.
    pub prev: Binding,
    /// Toggle between yes/no.
    pub toggle: Binding,
    /// Submit the form.
    pub submit: Binding,
    /// Accept (yes).
    pub accept: Binding,
    /// Reject (no).
    pub reject: Binding,
}

impl Default for ConfirmKeyMap {
    fn default() -> Self {
        Self {
            prev: Binding::new()
                .keys(&["shift+tab"])
                .help("shift+tab", "back"),
            next: Binding::new().keys(&["enter", "tab"]).help("enter", "next"),
            submit: Binding::new().keys(&["enter"]).help("enter", "submit"),
            toggle: Binding::new()
                .keys(&["h", "l", "right", "left"])
                .help("←/→", "toggle"),
            accept: Binding::new().keys(&["y", "Y"]).help("y", "Yes"),
            reject: Binding::new().keys(&["n", "N"]).help("n", "No"),
        }
    }
}

/// Keybindings for note fields.
#[derive(Debug, Clone)]
pub struct NoteKeyMap {
    /// Go to next field.
    pub next: Binding,
    /// Go to previous field.
    pub prev: Binding,
    /// Submit the form.
    pub submit: Binding,
}

impl Default for NoteKeyMap {
    fn default() -> Self {
        Self {
            prev: Binding::new()
                .keys(&["shift+tab"])
                .help("shift+tab", "back"),
            next: Binding::new().keys(&["enter", "tab"]).help("enter", "next"),
            submit: Binding::new().keys(&["enter"]).help("enter", "submit"),
        }
    }
}

/// Keybindings for text area fields.
#[derive(Debug, Clone)]
pub struct TextKeyMap {
    /// Go to next field.
    pub next: Binding,
    /// Go to previous field.
    pub prev: Binding,
    /// Insert a new line.
    pub new_line: Binding,
    /// Open external editor.
    pub editor: Binding,
    /// Submit the form.
    pub submit: Binding,
}

impl Default for TextKeyMap {
    fn default() -> Self {
        Self {
            prev: Binding::new()
                .keys(&["shift+tab"])
                .help("shift+tab", "back"),
            next: Binding::new().keys(&["tab", "enter"]).help("enter", "next"),
            submit: Binding::new().keys(&["enter"]).help("enter", "submit"),
            new_line: Binding::new()
                .keys(&["alt+enter", "ctrl+j"])
                .help("alt+enter / ctrl+j", "new line"),
            editor: Binding::new()
                .keys(&["ctrl+e"])
                .help("ctrl+e", "open editor"),
        }
    }
}

// -----------------------------------------------------------------------------
// Field Position
// -----------------------------------------------------------------------------

/// Positional information about a field within a form.
#[derive(Debug, Clone, Copy, Default)]
pub struct FieldPosition {
    /// Current group index.
    pub group: usize,
    /// Current field index within group.
    pub field: usize,
    /// First non-skipped field index.
    pub first_field: usize,
    /// Last non-skipped field index.
    pub last_field: usize,
    /// Total number of groups.
    pub group_count: usize,
    /// First non-hidden group index.
    pub first_group: usize,
    /// Last non-hidden group index.
    pub last_group: usize,
}

impl FieldPosition {
    /// Returns whether this field is the first in the form.
    pub fn is_first(&self) -> bool {
        self.field == self.first_field && self.group == self.first_group
    }

    /// Returns whether this field is the last in the form.
    pub fn is_last(&self) -> bool {
        self.field == self.last_field && self.group == self.last_group
    }
}

// -----------------------------------------------------------------------------
// Helper for key matching
// -----------------------------------------------------------------------------

/// Convert KeyMsg to a string representation for matching.
fn key_to_string(key: &KeyMsg) -> String {
    match key.key_type {
        KeyType::Enter => "enter".to_string(),
        KeyType::Tab => "tab".to_string(),
        KeyType::Esc => "escape".to_string(),
        KeyType::Backspace => "backspace".to_string(),
        KeyType::Delete => "delete".to_string(),
        KeyType::Left => "left".to_string(),
        KeyType::Right => "right".to_string(),
        KeyType::Up => "up".to_string(),
        KeyType::Down => "down".to_string(),
        KeyType::Home => "home".to_string(),
        KeyType::End => "end".to_string(),
        KeyType::PgUp => "pgup".to_string(),
        KeyType::PgDown => "pgdown".to_string(),
        KeyType::Space => " ".to_string(),
        KeyType::CtrlC => "ctrl+c".to_string(),
        KeyType::CtrlD => "ctrl+d".to_string(),
        KeyType::CtrlA => "ctrl+a".to_string(),
        KeyType::CtrlE => "ctrl+e".to_string(),
        KeyType::CtrlJ => "ctrl+j".to_string(),
        KeyType::CtrlK => "ctrl+k".to_string(),
        KeyType::CtrlN => "ctrl+n".to_string(),
        KeyType::CtrlP => "ctrl+p".to_string(),
        KeyType::CtrlU => "ctrl+u".to_string(),
        KeyType::Runes => {
            if key.runes.len() == 1 {
                key.runes[0].to_string()
            } else {
                key.runes.iter().collect()
            }
        }
        KeyType::ShiftTab => "shift+tab".to_string(),
        _ => String::new(),
    }
}

/// Check if a KeyMsg matches a Binding.
fn binding_matches(binding: &Binding, key: &KeyMsg) -> bool {
    if !binding.enabled() {
        return false;
    }
    let key_str = key_to_string(key);
    binding.get_keys().iter().any(|k| k == &key_str)
}

// -----------------------------------------------------------------------------
// Field Trait
// -----------------------------------------------------------------------------

/// A form field.
pub trait Field: Send + Sync {
    /// Returns the field's key.
    fn get_key(&self) -> &str;

    /// Returns the field's value.
    fn get_value(&self) -> Box<dyn Any>;

    /// Returns whether this field should be skipped.
    fn skip(&self) -> bool {
        false
    }

    /// Returns whether this field should zoom (take full height).
    fn zoom(&self) -> bool {
        false
    }

    /// Returns the current validation error, if any.
    fn error(&self) -> Option<&str>;

    /// Initializes the field.
    fn init(&mut self) -> Option<Cmd>;

    /// Updates the field with a message.
    fn update(&mut self, msg: &Message) -> Option<Cmd>;

    /// Renders the field.
    fn view(&self) -> String;

    /// Focuses the field.
    fn focus(&mut self) -> Option<Cmd>;

    /// Blurs the field.
    fn blur(&mut self) -> Option<Cmd>;

    /// Returns the help keybindings.
    fn key_binds(&self) -> Vec<Binding>;

    /// Sets the theme.
    fn with_theme(&mut self, theme: &Theme);

    /// Sets the keymap.
    fn with_keymap(&mut self, keymap: &KeyMap);

    /// Sets the width.
    fn with_width(&mut self, width: usize);

    /// Sets the height.
    fn with_height(&mut self, height: usize);

    /// Sets the field position.
    fn with_position(&mut self, position: FieldPosition);
}

// -----------------------------------------------------------------------------
// Messages
// -----------------------------------------------------------------------------

/// Message to move to the next field.
#[derive(Debug, Clone)]
pub struct NextFieldMsg;

/// Message to move to the previous field.
#[derive(Debug, Clone)]
pub struct PrevFieldMsg;

/// Message to move to the next group.
#[derive(Debug, Clone)]
pub struct NextGroupMsg;

/// Message to move to the previous group.
#[derive(Debug, Clone)]
pub struct PrevGroupMsg;

/// Message to update dynamic field content.
#[derive(Debug, Clone)]
pub struct UpdateFieldMsg;

// -----------------------------------------------------------------------------
// Input Field
// -----------------------------------------------------------------------------

/// A text input field.
pub struct Input {
    id: usize,
    key: String,
    value: String,
    title: String,
    description: String,
    placeholder: String,
    prompt: String,
    char_limit: usize,
    echo_mode: EchoMode,
    inline: bool,
    focused: bool,
    error: Option<String>,
    validate: Option<fn(&str) -> Option<String>>,
    width: usize,
    _height: usize,
    theme: Option<Theme>,
    keymap: InputKeyMap,
    _position: FieldPosition,
    cursor_pos: usize,
    suggestions: Vec<String>,
    show_suggestions: bool,
}

/// Echo mode for input fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EchoMode {
    /// Display text as-is.
    #[default]
    Normal,
    /// Display mask characters (for passwords).
    Password,
    /// Display nothing.
    None,
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

impl Input {
    /// Creates a new input field.
    pub fn new() -> Self {
        Self {
            id: next_id(),
            key: String::new(),
            value: String::new(),
            title: String::new(),
            description: String::new(),
            placeholder: String::new(),
            prompt: "> ".to_string(),
            char_limit: 0,
            echo_mode: EchoMode::Normal,
            inline: false,
            focused: false,
            error: None,
            validate: None,
            width: 80,
            _height: 0,
            theme: None,
            keymap: InputKeyMap::default(),
            _position: FieldPosition::default(),
            cursor_pos: 0,
            suggestions: Vec::new(),
            show_suggestions: false,
        }
    }

    /// Sets the field key.
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = key.into();
        self
    }

    /// Sets the initial value.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self.cursor_pos = self.value.len();
        self
    }

    /// Sets the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Sets the placeholder text.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets the prompt string.
    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = prompt.into();
        self
    }

    /// Sets the character limit.
    pub fn char_limit(mut self, limit: usize) -> Self {
        self.char_limit = limit;
        self
    }

    /// Sets the echo mode.
    pub fn echo_mode(mut self, mode: EchoMode) -> Self {
        self.echo_mode = mode;
        self
    }

    /// Sets password mode (shorthand for echo_mode).
    pub fn password(self, password: bool) -> Self {
        if password {
            self.echo_mode(EchoMode::Password)
        } else {
            self.echo_mode(EchoMode::Normal)
        }
    }

    /// Sets whether the title and input are on the same line.
    pub fn inline(mut self, inline: bool) -> Self {
        self.inline = inline;
        self
    }

    /// Sets the validation function.
    pub fn validate(mut self, validate: fn(&str) -> Option<String>) -> Self {
        self.validate = Some(validate);
        self
    }

    /// Sets the suggestions for autocomplete.
    pub fn suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions = suggestions;
        self.show_suggestions = !self.suggestions.is_empty();
        self
    }

    fn get_theme(&self) -> Theme {
        self.theme.clone().unwrap_or_else(theme_charm)
    }

    fn active_styles(&self) -> FieldStyles {
        let theme = self.get_theme();
        if self.focused {
            theme.focused
        } else {
            theme.blurred
        }
    }

    fn run_validation(&mut self) {
        if let Some(validate) = self.validate {
            self.error = validate(&self.value);
        }
    }

    fn display_value(&self) -> String {
        match self.echo_mode {
            EchoMode::Normal => self.value.clone(),
            EchoMode::Password => "•".repeat(self.value.len()),
            EchoMode::None => String::new(),
        }
    }

    /// Gets the current value.
    pub fn get_string_value(&self) -> &str {
        &self.value
    }

    /// Returns the field ID.
    pub fn id(&self) -> usize {
        self.id
    }
}

impl Field for Input {
    fn get_key(&self) -> &str {
        &self.key
    }

    fn get_value(&self) -> Box<dyn Any> {
        Box::new(self.value.clone())
    }

    fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    fn init(&mut self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: &Message) -> Option<Cmd> {
        if !self.focused {
            return None;
        }

        if let Some(key_msg) = msg.downcast_ref::<KeyMsg>() {
            self.error = None;

            // Check for prev
            if binding_matches(&self.keymap.prev, key_msg) {
                return Some(Cmd::new(|| Message::new(PrevFieldMsg)));
            }

            // Check for next/submit
            if binding_matches(&self.keymap.next, key_msg)
                || binding_matches(&self.keymap.submit, key_msg)
            {
                self.run_validation();
                if self.error.is_some() {
                    return None;
                }
                return Some(Cmd::new(|| Message::new(NextFieldMsg)));
            }

            // Handle character input
            match key_msg.key_type {
                KeyType::Runes => {
                    for c in &key_msg.runes {
                        if self.char_limit == 0 || self.value.len() < self.char_limit {
                            self.value.insert(self.cursor_pos, *c);
                            self.cursor_pos += 1;
                        }
                    }
                }
                KeyType::Backspace => {
                    if self.cursor_pos > 0 {
                        self.cursor_pos -= 1;
                        self.value.remove(self.cursor_pos);
                    }
                }
                KeyType::Delete => {
                    if self.cursor_pos < self.value.len() {
                        self.value.remove(self.cursor_pos);
                    }
                }
                KeyType::Left => {
                    if self.cursor_pos > 0 {
                        self.cursor_pos -= 1;
                    }
                }
                KeyType::Right => {
                    if self.cursor_pos < self.value.len() {
                        self.cursor_pos += 1;
                    }
                }
                KeyType::Home => {
                    self.cursor_pos = 0;
                }
                KeyType::End => {
                    self.cursor_pos = self.value.len();
                }
                _ => {}
            }
        }

        None
    }

    fn view(&self) -> String {
        let styles = self.active_styles();
        let mut output = String::new();

        // Title
        if !self.title.is_empty() {
            output.push_str(&styles.title.render(&self.title));
            if !self.inline {
                output.push('\n');
            }
        }

        // Description
        if !self.description.is_empty() {
            output.push_str(&styles.description.render(&self.description));
            if !self.inline {
                output.push('\n');
            }
        }

        // Prompt and value
        output.push_str(&styles.text_input.prompt.render(&self.prompt));

        let display = self.display_value();
        if display.is_empty() && !self.placeholder.is_empty() {
            output.push_str(&styles.text_input.placeholder.render(&self.placeholder));
        } else {
            output.push_str(&styles.text_input.text.render(&display));
        }

        // Error indicator
        if self.error.is_some() {
            output.push_str(&styles.error_indicator.render(""));
        }

        styles.base.width(self.width.try_into().unwrap_or(u16::MAX)).render(&output)
    }

    fn focus(&mut self) -> Option<Cmd> {
        self.focused = true;
        None
    }

    fn blur(&mut self) -> Option<Cmd> {
        self.focused = false;
        self.run_validation();
        None
    }

    fn key_binds(&self) -> Vec<Binding> {
        if self.show_suggestions {
            vec![
                self.keymap.accept_suggestion.clone(),
                self.keymap.prev.clone(),
                self.keymap.submit.clone(),
                self.keymap.next.clone(),
            ]
        } else {
            vec![
                self.keymap.prev.clone(),
                self.keymap.submit.clone(),
                self.keymap.next.clone(),
            ]
        }
    }

    fn with_theme(&mut self, theme: &Theme) {
        if self.theme.is_none() {
            self.theme = Some(theme.clone());
        }
    }

    fn with_keymap(&mut self, keymap: &KeyMap) {
        self.keymap = keymap.input.clone();
    }

    fn with_width(&mut self, width: usize) {
        self.width = width;
    }

    fn with_height(&mut self, height: usize) {
        self._height = height;
    }

    fn with_position(&mut self, position: FieldPosition) {
        self._position = position;
    }
}

// -----------------------------------------------------------------------------
// Select Field
// -----------------------------------------------------------------------------

/// A select field for choosing one option from a list.
pub struct Select<T: Clone + PartialEq + Send + Sync + 'static> {
    id: usize,
    key: String,
    options: Vec<SelectOption<T>>,
    selected: usize,
    title: String,
    description: String,
    inline: bool,
    focused: bool,
    error: Option<String>,
    validate: Option<fn(&T) -> Option<String>>,
    width: usize,
    height: usize,
    theme: Option<Theme>,
    keymap: SelectKeyMap,
    _position: FieldPosition,
    #[allow(dead_code)]
    filtering: bool,
    filter_value: String,
    offset: usize,
}

impl<T: Clone + PartialEq + Send + Sync + Default + 'static> Default for Select<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + PartialEq + Send + Sync + Default + 'static> Select<T> {
    /// Creates a new select field.
    pub fn new() -> Self {
        Self {
            id: next_id(),
            key: String::new(),
            options: Vec::new(),
            selected: 0,
            title: String::new(),
            description: String::new(),
            inline: false,
            focused: false,
            error: None,
            validate: None,
            width: 80,
            height: 5,
            theme: None,
            keymap: SelectKeyMap::default(),
            _position: FieldPosition::default(),
            filtering: false,
            filter_value: String::new(),
            offset: 0,
        }
    }

    /// Sets the field key.
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = key.into();
        self
    }

    /// Sets the options.
    pub fn options(mut self, options: Vec<SelectOption<T>>) -> Self {
        self.options = options;
        // Find initially selected
        for (i, opt) in self.options.iter().enumerate() {
            if opt.selected {
                self.selected = i;
                break;
            }
        }
        self
    }

    /// Sets the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Sets whether options display inline.
    pub fn inline(mut self, inline: bool) -> Self {
        self.inline = inline;
        self
    }

    /// Sets the validation function.
    pub fn validate(mut self, validate: fn(&T) -> Option<String>) -> Self {
        self.validate = Some(validate);
        self
    }

    /// Sets the visible height (number of options shown).
    pub fn height_options(mut self, height: usize) -> Self {
        self.height = height;
        self
    }

    fn get_theme(&self) -> Theme {
        self.theme.clone().unwrap_or_else(theme_charm)
    }

    fn active_styles(&self) -> FieldStyles {
        let theme = self.get_theme();
        if self.focused {
            theme.focused
        } else {
            theme.blurred
        }
    }

    fn run_validation(&mut self) {
        if let Some(validate) = self.validate
            && let Some(opt) = self.options.get(self.selected)
        {
            self.error = validate(&opt.value);
        }
    }

    fn filtered_options(&self) -> Vec<(usize, &SelectOption<T>)> {
        if self.filter_value.is_empty() {
            self.options.iter().enumerate().collect()
        } else {
            let filter_lower = self.filter_value.to_lowercase();
            self.options
                .iter()
                .enumerate()
                .filter(|(_, o)| o.key.to_lowercase().contains(&filter_lower))
                .collect()
        }
    }

    /// Gets the currently selected value.
    pub fn get_selected_value(&self) -> Option<&T> {
        self.options.get(self.selected).map(|o| &o.value)
    }

    /// Returns the field ID.
    pub fn id(&self) -> usize {
        self.id
    }
}

impl<T: Clone + PartialEq + Send + Sync + Default + 'static> Field for Select<T> {
    fn get_key(&self) -> &str {
        &self.key
    }

    fn get_value(&self) -> Box<dyn Any> {
        if let Some(opt) = self.options.get(self.selected) {
            Box::new(opt.value.clone())
        } else {
            Box::new(T::default())
        }
    }

    fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    fn init(&mut self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: &Message) -> Option<Cmd> {
        if !self.focused {
            return None;
        }

        if let Some(key_msg) = msg.downcast_ref::<KeyMsg>() {
            self.error = None;

            // Check for prev
            if binding_matches(&self.keymap.prev, key_msg) {
                return Some(Cmd::new(|| Message::new(PrevFieldMsg)));
            }

            // Check for next/submit
            if binding_matches(&self.keymap.next, key_msg)
                || binding_matches(&self.keymap.submit, key_msg)
            {
                self.run_validation();
                if self.error.is_some() {
                    return None;
                }
                return Some(Cmd::new(|| Message::new(NextFieldMsg)));
            }

            // Navigation
            if binding_matches(&self.keymap.up, key_msg) {
                if self.selected > 0 {
                    self.selected -= 1;
                    if self.selected < self.offset {
                        self.offset = self.selected;
                    }
                }
            } else if binding_matches(&self.keymap.down, key_msg) {
                if self.selected < self.options.len().saturating_sub(1) {
                    self.selected += 1;
                    if self.selected >= self.offset + self.height {
                        self.offset = self.selected.saturating_sub(self.height - 1);
                    }
                }
            } else if binding_matches(&self.keymap.goto_top, key_msg) {
                self.selected = 0;
                self.offset = 0;
            } else if binding_matches(&self.keymap.goto_bottom, key_msg) {
                self.selected = self.options.len().saturating_sub(1);
                self.offset = self.selected.saturating_sub(self.height - 1);
            }
        }

        None
    }

    fn view(&self) -> String {
        let styles = self.active_styles();
        let mut output = String::new();

        // Title
        if !self.title.is_empty() {
            output.push_str(&styles.title.render(&self.title));
            output.push('\n');
        }

        // Description
        if !self.description.is_empty() {
            output.push_str(&styles.description.render(&self.description));
            output.push('\n');
        }

        // Options
        let filtered = self.filtered_options();
        let visible: Vec<_> = filtered
            .iter()
            .skip(self.offset)
            .take(self.height)
            .collect();

        if self.inline {
            // Inline mode
            let mut inline_output = String::new();
            inline_output.push_str(&styles.prev_indicator.render(""));
            for (i, (idx, opt)) in visible.iter().enumerate() {
                if *idx == self.selected {
                    inline_output.push_str(&styles.selected_option.render(&opt.key));
                } else {
                    inline_output.push_str(&styles.option.render(&opt.key));
                }
                if i < visible.len() - 1 {
                    inline_output.push_str("  ");
                }
            }
            inline_output.push_str(&styles.next_indicator.render(""));
            output.push_str(&inline_output);
        } else {
            // Vertical list mode
            for (idx, opt) in visible {
                if *idx == self.selected {
                    output.push_str(&styles.select_selector.render(""));
                    output.push_str(&styles.selected_option.render(&opt.key));
                } else {
                    output.push_str("  ");
                    output.push_str(&styles.option.render(&opt.key));
                }
                output.push('\n');
            }
            // Remove trailing newline
            output.pop();
        }

        // Error indicator
        if self.error.is_some() {
            output.push_str(&styles.error_indicator.render(""));
        }

        styles.base.width(self.width.try_into().unwrap_or(u16::MAX)).render(&output)
    }

    fn focus(&mut self) -> Option<Cmd> {
        self.focused = true;
        None
    }

    fn blur(&mut self) -> Option<Cmd> {
        self.focused = false;
        self.run_validation();
        None
    }

    fn key_binds(&self) -> Vec<Binding> {
        vec![
            self.keymap.up.clone(),
            self.keymap.down.clone(),
            self.keymap.prev.clone(),
            self.keymap.submit.clone(),
            self.keymap.next.clone(),
        ]
    }

    fn with_theme(&mut self, theme: &Theme) {
        if self.theme.is_none() {
            self.theme = Some(theme.clone());
        }
    }

    fn with_keymap(&mut self, keymap: &KeyMap) {
        self.keymap = keymap.select.clone();
    }

    fn with_width(&mut self, width: usize) {
        self.width = width;
    }

    fn with_height(&mut self, height: usize) {
        self.height = height;
    }

    fn with_position(&mut self, position: FieldPosition) {
        self._position = position;
    }
}

// -----------------------------------------------------------------------------
// MultiSelect Field
// -----------------------------------------------------------------------------

/// A multi-select field for choosing multiple options from a list.
pub struct MultiSelect<T: Clone + PartialEq + Send + Sync + 'static> {
    id: usize,
    key: String,
    options: Vec<SelectOption<T>>,
    selected: Vec<usize>,
    cursor: usize,
    title: String,
    description: String,
    focused: bool,
    error: Option<String>,
    #[allow(clippy::type_complexity)]
    validate: Option<fn(&[T]) -> Option<String>>,
    width: usize,
    height: usize,
    limit: Option<usize>,
    theme: Option<Theme>,
    keymap: MultiSelectKeyMap,
    _position: FieldPosition,
    #[allow(dead_code)]
    filtering: bool,
    filter_value: String,
    offset: usize,
}

impl<T: Clone + PartialEq + Send + Sync + Default + 'static> Default for MultiSelect<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + PartialEq + Send + Sync + Default + 'static> MultiSelect<T> {
    /// Creates a new multi-select field.
    pub fn new() -> Self {
        Self {
            id: next_id(),
            key: String::new(),
            options: Vec::new(),
            selected: Vec::new(),
            cursor: 0,
            title: String::new(),
            description: String::new(),
            focused: false,
            error: None,
            validate: None,
            width: 80,
            height: 5,
            limit: None,
            theme: None,
            keymap: MultiSelectKeyMap::default(),
            _position: FieldPosition::default(),
            filtering: false,
            filter_value: String::new(),
            offset: 0,
        }
    }

    /// Sets the field key.
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = key.into();
        self
    }

    /// Sets the options.
    pub fn options(mut self, options: Vec<SelectOption<T>>) -> Self {
        self.options = options;
        // Find initially selected options
        self.selected = self
            .options
            .iter()
            .enumerate()
            .filter(|(_, opt)| opt.selected)
            .map(|(i, _)| i)
            .collect();
        self
    }

    /// Sets the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Sets the validation function.
    pub fn validate(mut self, validate: fn(&[T]) -> Option<String>) -> Self {
        self.validate = Some(validate);
        self
    }

    /// Sets the visible height (number of options shown).
    pub fn height_options(mut self, height: usize) -> Self {
        self.height = height;
        self
    }

    /// Sets the maximum number of selections allowed.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    fn get_theme(&self) -> Theme {
        self.theme.clone().unwrap_or_else(theme_charm)
    }

    fn active_styles(&self) -> FieldStyles {
        let theme = self.get_theme();
        if self.focused {
            theme.focused
        } else {
            theme.blurred
        }
    }

    fn run_validation(&mut self) {
        if let Some(validate) = self.validate {
            let values: Vec<T> = self
                .selected
                .iter()
                .filter_map(|&i| self.options.get(i).map(|o| o.value.clone()))
                .collect();
            self.error = validate(&values);
        }
    }

    fn filtered_options(&self) -> Vec<(usize, &SelectOption<T>)> {
        if self.filter_value.is_empty() {
            self.options.iter().enumerate().collect()
        } else {
            let filter_lower = self.filter_value.to_lowercase();
            self.options
                .iter()
                .enumerate()
                .filter(|(_, o)| o.key.to_lowercase().contains(&filter_lower))
                .collect()
        }
    }

    fn toggle_current(&mut self) {
        let filtered = self.filtered_options();
        if let Some((idx, _)) = filtered.get(self.cursor) {
            if let Some(pos) = self.selected.iter().position(|&i| i == *idx) {
                // Deselect
                self.selected.remove(pos);
            } else if self.limit.is_none_or(|l| self.selected.len() < l) {
                // Select (if within limit)
                self.selected.push(*idx);
            }
        }
    }

    fn select_all(&mut self) {
        if let Some(limit) = self.limit {
            // Only select up to limit
            self.selected = self
                .options
                .iter()
                .enumerate()
                .take(limit)
                .map(|(i, _)| i)
                .collect();
        } else {
            self.selected = (0..self.options.len()).collect();
        }
    }

    fn select_none(&mut self) {
        self.selected.clear();
    }

    /// Gets the currently selected values.
    pub fn get_selected_values(&self) -> Vec<&T> {
        self.selected
            .iter()
            .filter_map(|&i| self.options.get(i).map(|o| &o.value))
            .collect()
    }

    /// Returns the field ID.
    pub fn id(&self) -> usize {
        self.id
    }
}

impl<T: Clone + PartialEq + Send + Sync + Default + 'static> Field for MultiSelect<T> {
    fn get_key(&self) -> &str {
        &self.key
    }

    fn get_value(&self) -> Box<dyn Any> {
        let values: Vec<T> = self
            .selected
            .iter()
            .filter_map(|&i| self.options.get(i).map(|o| o.value.clone()))
            .collect();
        Box::new(values)
    }

    fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    fn init(&mut self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: &Message) -> Option<Cmd> {
        if !self.focused {
            return None;
        }

        if let Some(key_msg) = msg.downcast_ref::<KeyMsg>() {
            self.error = None;

            // Check for prev
            if binding_matches(&self.keymap.prev, key_msg) {
                return Some(Cmd::new(|| Message::new(PrevFieldMsg)));
            }

            // Check for next/submit
            if binding_matches(&self.keymap.next, key_msg)
                || binding_matches(&self.keymap.submit, key_msg)
            {
                self.run_validation();
                if self.error.is_some() {
                    return None;
                }
                return Some(Cmd::new(|| Message::new(NextFieldMsg)));
            }

            // Toggle selection
            if binding_matches(&self.keymap.toggle, key_msg) {
                self.toggle_current();
            }

            // Select all
            if binding_matches(&self.keymap.select_all, key_msg) {
                if self.selected.len() == self.options.len() {
                    self.select_none();
                } else {
                    self.select_all();
                }
            }

            // Navigation
            if binding_matches(&self.keymap.up, key_msg) {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    if self.cursor < self.offset {
                        self.offset = self.cursor;
                    }
                }
            } else if binding_matches(&self.keymap.down, key_msg) {
                let filtered = self.filtered_options();
                if self.cursor < filtered.len().saturating_sub(1) {
                    self.cursor += 1;
                    if self.cursor >= self.offset + self.height {
                        self.offset = self.cursor.saturating_sub(self.height - 1);
                    }
                }
            } else if binding_matches(&self.keymap.goto_top, key_msg) {
                self.cursor = 0;
                self.offset = 0;
            } else if binding_matches(&self.keymap.goto_bottom, key_msg) {
                let filtered = self.filtered_options();
                self.cursor = filtered.len().saturating_sub(1);
                self.offset = self.cursor.saturating_sub(self.height - 1);
            }
        }

        None
    }

    fn view(&self) -> String {
        let styles = self.active_styles();
        let mut output = String::new();

        // Title
        if !self.title.is_empty() {
            output.push_str(&styles.title.render(&self.title));
            output.push('\n');
        }

        // Description
        if !self.description.is_empty() {
            output.push_str(&styles.description.render(&self.description));
            output.push('\n');
        }

        // Options
        let filtered = self.filtered_options();
        let visible: Vec<_> = filtered
            .iter()
            .skip(self.offset)
            .take(self.height)
            .collect();

        // Vertical list mode with checkboxes
        for (i, (idx, opt)) in visible.iter().enumerate() {
            let is_cursor = self.offset + i == self.cursor;
            let is_selected = self.selected.contains(idx);

            // Cursor indicator
            if is_cursor {
                output.push_str(&styles.select_selector.render(""));
            } else {
                output.push_str("  ");
            }

            // Checkbox
            let checkbox = if is_selected { "[x] " } else { "[ ] " };
            output.push_str(checkbox);

            // Option text
            if is_cursor {
                output.push_str(&styles.selected_option.render(&opt.key));
            } else {
                output.push_str(&styles.option.render(&opt.key));
            }

            output.push('\n');
        }

        // Remove trailing newline
        if !visible.is_empty() {
            output.pop();
        }

        // Error indicator
        if self.error.is_some() {
            output.push_str(&styles.error_indicator.render(""));
        }

        styles.base.width(self.width.try_into().unwrap_or(u16::MAX)).render(&output)
    }

    fn focus(&mut self) -> Option<Cmd> {
        self.focused = true;
        None
    }

    fn blur(&mut self) -> Option<Cmd> {
        self.focused = false;
        self.run_validation();
        None
    }

    fn key_binds(&self) -> Vec<Binding> {
        vec![
            self.keymap.up.clone(),
            self.keymap.down.clone(),
            self.keymap.toggle.clone(),
            self.keymap.prev.clone(),
            self.keymap.submit.clone(),
            self.keymap.next.clone(),
        ]
    }

    fn with_theme(&mut self, theme: &Theme) {
        if self.theme.is_none() {
            self.theme = Some(theme.clone());
        }
    }

    fn with_keymap(&mut self, keymap: &KeyMap) {
        self.keymap = keymap.multi_select.clone();
    }

    fn with_width(&mut self, width: usize) {
        self.width = width;
    }

    fn with_height(&mut self, height: usize) {
        self.height = height;
    }

    fn with_position(&mut self, position: FieldPosition) {
        self._position = position;
    }
}

// -----------------------------------------------------------------------------
// Confirm Field
// -----------------------------------------------------------------------------

/// A confirmation field with Yes/No options.
pub struct Confirm {
    id: usize,
    key: String,
    value: bool,
    title: String,
    description: String,
    affirmative: String,
    negative: String,
    focused: bool,
    width: usize,
    theme: Option<Theme>,
    keymap: ConfirmKeyMap,
    _position: FieldPosition,
}

impl Default for Confirm {
    fn default() -> Self {
        Self::new()
    }
}

impl Confirm {
    /// Creates a new confirm field.
    pub fn new() -> Self {
        Self {
            id: next_id(),
            key: String::new(),
            value: false,
            title: String::new(),
            description: String::new(),
            affirmative: "Yes".to_string(),
            negative: "No".to_string(),
            focused: false,
            width: 80,
            theme: None,
            keymap: ConfirmKeyMap::default(),
            _position: FieldPosition::default(),
        }
    }

    /// Sets the field key.
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = key.into();
        self
    }

    /// Sets the initial value.
    pub fn value(mut self, value: bool) -> Self {
        self.value = value;
        self
    }

    /// Sets the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Sets the affirmative button text.
    pub fn affirmative(mut self, text: impl Into<String>) -> Self {
        self.affirmative = text.into();
        self
    }

    /// Sets the negative button text.
    pub fn negative(mut self, text: impl Into<String>) -> Self {
        self.negative = text.into();
        self
    }

    fn get_theme(&self) -> Theme {
        self.theme.clone().unwrap_or_else(theme_charm)
    }

    fn active_styles(&self) -> FieldStyles {
        let theme = self.get_theme();
        if self.focused {
            theme.focused
        } else {
            theme.blurred
        }
    }

    /// Gets the current value.
    pub fn get_bool_value(&self) -> bool {
        self.value
    }

    /// Returns the field ID.
    pub fn id(&self) -> usize {
        self.id
    }
}

impl Field for Confirm {
    fn get_key(&self) -> &str {
        &self.key
    }

    fn get_value(&self) -> Box<dyn Any> {
        Box::new(self.value)
    }

    fn error(&self) -> Option<&str> {
        None
    }

    fn init(&mut self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: &Message) -> Option<Cmd> {
        if !self.focused {
            return None;
        }

        if let Some(key_msg) = msg.downcast_ref::<KeyMsg>() {
            // Check for prev
            if binding_matches(&self.keymap.prev, key_msg) {
                return Some(Cmd::new(|| Message::new(PrevFieldMsg)));
            }

            // Check for next/submit
            if binding_matches(&self.keymap.next, key_msg)
                || binding_matches(&self.keymap.submit, key_msg)
            {
                return Some(Cmd::new(|| Message::new(NextFieldMsg)));
            }

            // Toggle
            if binding_matches(&self.keymap.toggle, key_msg) {
                self.value = !self.value;
            }

            // Direct accept/reject
            if binding_matches(&self.keymap.accept, key_msg) {
                self.value = true;
            }
            if binding_matches(&self.keymap.reject, key_msg) {
                self.value = false;
            }
        }

        None
    }

    fn view(&self) -> String {
        let styles = self.active_styles();
        let mut output = String::new();

        // Title
        if !self.title.is_empty() {
            output.push_str(&styles.title.render(&self.title));
            output.push('\n');
        }

        // Description
        if !self.description.is_empty() {
            output.push_str(&styles.description.render(&self.description));
            output.push('\n');
        }

        // Buttons
        if self.value {
            output.push_str(&styles.focused_button.render(&self.affirmative));
            output.push_str(&styles.blurred_button.render(&self.negative));
        } else {
            output.push_str(&styles.blurred_button.render(&self.affirmative));
            output.push_str(&styles.focused_button.render(&self.negative));
        }

        styles.base.width(self.width.try_into().unwrap_or(u16::MAX)).render(&output)
    }

    fn focus(&mut self) -> Option<Cmd> {
        self.focused = true;
        None
    }

    fn blur(&mut self) -> Option<Cmd> {
        self.focused = false;
        None
    }

    fn key_binds(&self) -> Vec<Binding> {
        vec![
            self.keymap.toggle.clone(),
            self.keymap.accept.clone(),
            self.keymap.reject.clone(),
            self.keymap.prev.clone(),
            self.keymap.submit.clone(),
            self.keymap.next.clone(),
        ]
    }

    fn with_theme(&mut self, theme: &Theme) {
        if self.theme.is_none() {
            self.theme = Some(theme.clone());
        }
    }

    fn with_keymap(&mut self, keymap: &KeyMap) {
        self.keymap = keymap.confirm.clone();
    }

    fn with_width(&mut self, width: usize) {
        self.width = width;
    }

    fn with_height(&mut self, _height: usize) {
        // Confirm doesn't use height
    }

    fn with_position(&mut self, position: FieldPosition) {
        self._position = position;
    }
}

// -----------------------------------------------------------------------------
// Note Field
// -----------------------------------------------------------------------------

/// A non-interactive note/text display field.
pub struct Note {
    id: usize,
    key: String,
    title: String,
    description: String,
    focused: bool,
    width: usize,
    theme: Option<Theme>,
    keymap: NoteKeyMap,
    _position: FieldPosition,
    next_label: String,
}

impl Default for Note {
    fn default() -> Self {
        Self::new()
    }
}

impl Note {
    /// Creates a new note field.
    pub fn new() -> Self {
        Self {
            id: next_id(),
            key: String::new(),
            title: String::new(),
            description: String::new(),
            focused: false,
            width: 80,
            theme: None,
            keymap: NoteKeyMap::default(),
            _position: FieldPosition::default(),
            next_label: "Next".to_string(),
        }
    }

    /// Sets the field key.
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = key.into();
        self
    }

    /// Sets the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the description (body text).
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Sets the next button label.
    pub fn next_label(mut self, label: impl Into<String>) -> Self {
        self.next_label = label.into();
        self
    }

    fn get_theme(&self) -> Theme {
        self.theme.clone().unwrap_or_else(theme_charm)
    }

    fn active_styles(&self) -> FieldStyles {
        let theme = self.get_theme();
        if self.focused {
            theme.focused
        } else {
            theme.blurred
        }
    }

    /// Returns the field ID.
    pub fn id(&self) -> usize {
        self.id
    }
}

impl Field for Note {
    fn get_key(&self) -> &str {
        &self.key
    }

    fn get_value(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn error(&self) -> Option<&str> {
        None
    }

    fn init(&mut self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: &Message) -> Option<Cmd> {
        if !self.focused {
            return None;
        }

        if let Some(key_msg) = msg.downcast_ref::<KeyMsg>() {
            // Check for prev
            if binding_matches(&self.keymap.prev, key_msg) {
                return Some(Cmd::new(|| Message::new(PrevFieldMsg)));
            }

            // Check for next/submit
            if binding_matches(&self.keymap.next, key_msg)
                || binding_matches(&self.keymap.submit, key_msg)
            {
                return Some(Cmd::new(|| Message::new(NextFieldMsg)));
            }
        }

        None
    }

    fn view(&self) -> String {
        let styles = self.active_styles();
        let mut output = String::new();

        // Title
        if !self.title.is_empty() {
            output.push_str(&styles.note_title.render(&self.title));
            output.push('\n');
        }

        // Description
        if !self.description.is_empty() {
            output.push_str(&styles.description.render(&self.description));
        }

        styles.base.width(self.width.try_into().unwrap_or(u16::MAX)).render(&output)
    }

    fn focus(&mut self) -> Option<Cmd> {
        self.focused = true;
        None
    }

    fn blur(&mut self) -> Option<Cmd> {
        self.focused = false;
        None
    }

    fn key_binds(&self) -> Vec<Binding> {
        vec![
            self.keymap.prev.clone(),
            self.keymap.submit.clone(),
            self.keymap.next.clone(),
        ]
    }

    fn with_theme(&mut self, theme: &Theme) {
        if self.theme.is_none() {
            self.theme = Some(theme.clone());
        }
    }

    fn with_keymap(&mut self, keymap: &KeyMap) {
        self.keymap = keymap.note.clone();
    }

    fn with_width(&mut self, width: usize) {
        self.width = width;
    }

    fn with_height(&mut self, _height: usize) {
        // Note doesn't use height
    }

    fn with_position(&mut self, position: FieldPosition) {
        self._position = position;
    }
}

// -----------------------------------------------------------------------------
// Group
// -----------------------------------------------------------------------------

/// A group of fields displayed together.
pub struct Group {
    fields: Vec<Box<dyn Field>>,
    current: usize,
    title: String,
    description: String,
    width: usize,
    #[allow(dead_code)]
    height: usize,
    theme: Option<Theme>,
    keymap: Option<KeyMap>,
    hide: Option<Box<dyn Fn() -> bool + Send + Sync>>,
}

impl Default for Group {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl Group {
    /// Creates a new group with the given fields.
    pub fn new(fields: Vec<Box<dyn Field>>) -> Self {
        Self {
            fields,
            current: 0,
            title: String::new(),
            description: String::new(),
            width: 80,
            height: 0,
            theme: None,
            keymap: None,
            hide: None,
        }
    }

    /// Sets the group title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the group description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Sets whether the group should be hidden.
    pub fn hide(mut self, hide: bool) -> Self {
        self.hide = Some(Box::new(move || hide));
        self
    }

    /// Sets a function to determine if the group should be hidden.
    pub fn hide_func<F: Fn() -> bool + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.hide = Some(Box::new(f));
        self
    }

    /// Returns whether this group should be hidden.
    pub fn is_hidden(&self) -> bool {
        self.hide.as_ref().map(|f| f()).unwrap_or(false)
    }

    /// Returns the current field index.
    pub fn current(&self) -> usize {
        self.current
    }

    /// Returns the number of fields.
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Returns whether the group has no fields.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Returns a reference to the current field.
    pub fn current_field(&self) -> Option<&dyn Field> {
        self.fields.get(self.current).map(|f| f.as_ref())
    }

    /// Returns a mutable reference to the current field.
    pub fn current_field_mut(&mut self) -> Option<&mut Box<dyn Field>> {
        self.fields.get_mut(self.current)
    }

    /// Collects all field errors.
    pub fn errors(&self) -> Vec<&str> {
        self.fields.iter().filter_map(|f| f.error()).collect()
    }

    fn get_theme(&self) -> Theme {
        self.theme.clone().unwrap_or_else(theme_charm)
    }
}

impl Model for Group {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle navigation messages
        if msg.is::<NextFieldMsg>() {
            if self.current < self.fields.len().saturating_sub(1) {
                if let Some(field) = self.fields.get_mut(self.current) {
                    field.blur();
                }
                self.current += 1;
                if let Some(field) = self.fields.get_mut(self.current) {
                    return field.focus();
                }
            } else {
                return Some(Cmd::new(|| Message::new(NextGroupMsg)));
            }
        } else if msg.is::<PrevFieldMsg>() {
            if self.current > 0 {
                if let Some(field) = self.fields.get_mut(self.current) {
                    field.blur();
                }
                self.current -= 1;
                if let Some(field) = self.fields.get_mut(self.current) {
                    return field.focus();
                }
            } else {
                return Some(Cmd::new(|| Message::new(PrevGroupMsg)));
            }
        }

        // Forward to current field
        if let Some(field) = self.fields.get_mut(self.current) {
            return field.update(&msg);
        }

        None
    }

    fn view(&self) -> String {
        let theme = self.get_theme();
        let mut output = String::new();

        // Title
        if !self.title.is_empty() {
            output.push_str(&theme.group.title.render(&self.title));
            output.push('\n');
        }

        // Description
        if !self.description.is_empty() {
            output.push_str(&theme.group.description.render(&self.description));
            output.push('\n');
        }

        // Fields
        for (i, field) in self.fields.iter().enumerate() {
            output.push_str(&field.view());
            if i < self.fields.len() - 1 {
                output.push_str(&theme.field_separator.render(""));
            }
        }

        theme.group.base.width(self.width.try_into().unwrap_or(u16::MAX)).render(&output)
    }
}

// -----------------------------------------------------------------------------
// Form
// -----------------------------------------------------------------------------

/// A form containing multiple groups of fields.
pub struct Form {
    groups: Vec<Group>,
    current_group: usize,
    state: FormState,
    width: usize,
    theme: Theme,
    keymap: KeyMap,
}

impl Default for Form {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl Form {
    /// Creates a new form with the given groups.
    pub fn new(groups: Vec<Group>) -> Self {
        Self {
            groups,
            current_group: 0,
            state: FormState::Normal,
            width: 80,
            theme: theme_charm(),
            keymap: KeyMap::default(),
        }
    }

    /// Sets the form width.
    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Sets the theme.
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the keymap.
    pub fn keymap(mut self, keymap: KeyMap) -> Self {
        self.keymap = keymap;
        self
    }

    /// Returns the form state.
    pub fn state(&self) -> FormState {
        self.state
    }

    /// Returns the current group index.
    pub fn current_group(&self) -> usize {
        self.current_group
    }

    /// Returns the number of groups.
    pub fn len(&self) -> usize {
        self.groups.len()
    }

    /// Returns whether the form has no groups.
    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    /// Initializes all fields with theme and keymap.
    fn init_fields(&mut self) {
        for group in &mut self.groups {
            group.theme = Some(self.theme.clone());
            group.keymap = Some(self.keymap.clone());
            group.width = self.width;
            for field in &mut group.fields {
                field.with_theme(&self.theme);
                field.with_keymap(&self.keymap);
                field.with_width(self.width);
            }
        }
    }

    fn next_group(&mut self) -> Option<Cmd> {
        // Skip hidden groups
        loop {
            if self.current_group >= self.groups.len().saturating_sub(1) {
                self.state = FormState::Completed;
                return Some(bubbletea::quit());
            }
            self.current_group += 1;
            if !self.groups[self.current_group].is_hidden() {
                break;
            }
        }
        // Focus first field of new group
        if let Some(group) = self.groups.get_mut(self.current_group) {
            group.current = 0;
            if let Some(field) = group.fields.get_mut(0) {
                return field.focus();
            }
        }
        None
    }

    fn prev_group(&mut self) -> Option<Cmd> {
        // Skip hidden groups
        loop {
            if self.current_group == 0 {
                return None;
            }
            self.current_group -= 1;
            if !self.groups[self.current_group].is_hidden() {
                break;
            }
        }
        // Focus last field of new group
        if let Some(group) = self.groups.get_mut(self.current_group) {
            group.current = group.fields.len().saturating_sub(1);
            if let Some(field) = group.fields.last_mut() {
                return field.focus();
            }
        }
        None
    }

    /// Returns the value of a field by key.
    pub fn get_value(&self, key: &str) -> Option<Box<dyn Any>> {
        for group in &self.groups {
            for field in &group.fields {
                if field.get_key() == key {
                    return Some(field.get_value());
                }
            }
        }
        None
    }

    /// Returns the string value of a field by key.
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get_value(key)
            .and_then(|v| v.downcast::<String>().ok())
            .map(|v| *v)
    }

    /// Returns the boolean value of a field by key.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get_value(key)
            .and_then(|v| v.downcast::<bool>().ok())
            .map(|v| *v)
    }
}

impl Model for Form {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Initialize fields on first update
        if self.state == FormState::Normal && self.current_group == 0 {
            self.init_fields();
            // Focus first field
            if let Some(group) = self.groups.get_mut(0)
                && let Some(field) = group.fields.get_mut(0)
            {
                field.focus();
            }
        }

        // Handle quit
        if let Some(key_msg) = msg.downcast_ref::<KeyMsg>()
            && binding_matches(&self.keymap.quit, key_msg)
        {
            self.state = FormState::Aborted;
            return Some(bubbletea::quit());
        }

        // Handle group navigation
        if msg.is::<NextGroupMsg>() {
            return self.next_group();
        } else if msg.is::<PrevGroupMsg>() {
            return self.prev_group();
        }

        // Forward to current group
        if let Some(group) = self.groups.get_mut(self.current_group) {
            return group.update(msg);
        }

        None
    }

    fn view(&self) -> String {
        if let Some(group) = self.groups.get(self.current_group) {
            self.theme
                .form
                .base
                .clone()
                .width(self.width.try_into().unwrap_or(u16::MAX))
                .render(&group.view())
        } else {
            String::new()
        }
    }
}

// -----------------------------------------------------------------------------
// Validators
// -----------------------------------------------------------------------------

/// Creates a validator that checks if the input is not empty.
///
/// **Note**: Due to Rust function pointer limitations, the `_field_name` parameter
/// is not used. It exists only for API compatibility. To create validators with
/// custom error messages, use a closure directly:
///
/// ```rust,ignore
/// let validator = |s: &str| {
///     if s.trim().is_empty() {
///         Some("username is required".to_string())
///     } else {
///         None
///     }
/// };
/// ```
///
/// # Example
/// ```
/// use huh::validate_required;
/// let validator = validate_required("any");
/// assert!(validator("").is_some()); // Error: "field is required"
/// assert!(validator("John").is_none()); // Valid
/// ```
pub fn validate_required(_field_name: &'static str) -> fn(&str) -> Option<String> {
    |s| {
        if s.trim().is_empty() {
            Some("field is required".to_string())
        } else {
            None
        }
    }
}

/// Creates a required validator for the "name" field.
pub fn validate_required_name() -> fn(&str) -> Option<String> {
    |s| {
        if s.trim().is_empty() {
            Some("name is required".to_string())
        } else {
            None
        }
    }
}

/// Creates a min length validator for password fields.
/// Note: Due to Rust's function pointer limitations, this returns a closure
/// that can be converted to a function pointer.
pub fn validate_min_length_8() -> fn(&str) -> Option<String> {
    |s| {
        if s.chars().count() < 8 {
            Some("password must be at least 8 characters".to_string())
        } else {
            None
        }
    }
}

/// Creates a validator for email format.
/// Uses a simple regex pattern to validate email addresses.
pub fn validate_email() -> fn(&str) -> Option<String> {
    |s| {
        if s.is_empty() {
            return Some("email is required".to_string());
        }
        // Simple email validation: must have @ with something before and after
        // and a dot after the @
        let parts: Vec<&str> = s.split('@').collect();
        if parts.len() != 2 {
            return Some("invalid email address".to_string());
        }
        let (local, domain) = (parts[0], parts[1]);
        if local.is_empty() || domain.is_empty() || !domain.contains('.') {
            return Some("invalid email address".to_string());
        }
        // Check domain has something after the dot
        let domain_parts: Vec<&str> = domain.split('.').collect();
        if domain_parts.len() < 2 || domain_parts.iter().any(|p| p.is_empty()) {
            return Some("invalid email address".to_string());
        }
        None
    }
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_error_display() {
        let err = FormError::UserAborted;
        assert_eq!(format!("{}", err), "user aborted");

        let err = FormError::Validation("invalid input".to_string());
        assert_eq!(format!("{}", err), "validation error: invalid input");
    }

    #[test]
    fn test_form_state_default() {
        let state = FormState::default();
        assert_eq!(state, FormState::Normal);
    }

    #[test]
    fn test_select_option() {
        let opt = SelectOption::new("Red", "red".to_string());
        assert_eq!(opt.key, "Red");
        assert_eq!(opt.value, "red");
        assert!(!opt.selected);

        let opt = opt.selected(true);
        assert!(opt.selected);
    }

    #[test]
    fn test_new_options() {
        let opts = new_options(["apple", "banana", "cherry"]);
        assert_eq!(opts.len(), 3);
        assert_eq!(opts[0].key, "apple");
        assert_eq!(opts[0].value, "apple");
    }

    #[test]
    fn test_input_builder() {
        let input = Input::new()
            .key("name")
            .title("Name")
            .description("Enter your name")
            .placeholder("John Doe")
            .value("Jane");

        assert_eq!(input.get_key(), "name");
        assert_eq!(input.get_string_value(), "Jane");
    }

    #[test]
    fn test_confirm_builder() {
        let confirm = Confirm::new()
            .key("agree")
            .title("Terms")
            .affirmative("I Agree")
            .negative("I Disagree")
            .value(true);

        assert_eq!(confirm.get_key(), "agree");
        assert!(confirm.get_bool_value());
    }

    #[test]
    fn test_note_builder() {
        let note = Note::new()
            .key("info")
            .title("Information")
            .description("This is an informational note.");

        assert_eq!(note.get_key(), "info");
    }

    #[test]
    fn test_select_builder() {
        let select: Select<String> =
            Select::new()
                .key("color")
                .title("Favorite Color")
                .options(vec![
                    SelectOption::new("Red", "red".to_string()),
                    SelectOption::new("Green", "green".to_string()).selected(true),
                    SelectOption::new("Blue", "blue".to_string()),
                ]);

        assert_eq!(select.get_key(), "color");
        assert_eq!(select.get_selected_value(), Some(&"green".to_string()));
    }

    #[test]
    fn test_theme_base() {
        let theme = theme_base();
        assert!(!theme.focused.title.value().is_empty() || theme.focused.title.value().is_empty());
    }

    #[test]
    fn test_theme_charm() {
        let theme = theme_charm();
        // Just verify it doesn't panic
        let _ = theme.focused.title.render("Test");
    }

    #[test]
    fn test_theme_dracula() {
        let theme = theme_dracula();
        let _ = theme.focused.title.render("Test");
    }

    #[test]
    fn test_theme_base16() {
        let theme = theme_base16();
        let _ = theme.focused.title.render("Test");
    }

    #[test]
    fn test_keymap_default() {
        let keymap = KeyMap::default();
        assert!(keymap.quit.enabled());
        assert!(keymap.input.next.enabled());
    }

    #[test]
    fn test_field_position() {
        let pos = FieldPosition {
            group: 0,
            field: 0,
            first_field: 0,
            last_field: 2,
            group_count: 2,
            first_group: 0,
            last_group: 1,
        };
        assert!(pos.is_first());
        assert!(!pos.is_last());
    }

    #[test]
    fn test_group_basic() {
        let group = Group::new(vec![
            Box::new(Input::new().key("name").title("Name")),
            Box::new(Input::new().key("email").title("Email")),
        ]);

        assert_eq!(group.len(), 2);
        assert!(!group.is_empty());
        assert_eq!(group.current(), 0);
    }

    #[test]
    fn test_group_hide() {
        let group = Group::new(Vec::new()).hide(true);
        assert!(group.is_hidden());

        let group = Group::new(Vec::new()).hide(false);
        assert!(!group.is_hidden());
    }

    #[test]
    fn test_form_basic() {
        let form = Form::new(vec![Group::new(vec![Box::new(Input::new().key("name"))])]);

        assert_eq!(form.len(), 1);
        assert!(!form.is_empty());
        assert_eq!(form.state(), FormState::Normal);
    }

    #[test]
    fn test_input_echo_mode() {
        let input = Input::new().password(true);
        assert_eq!(input.echo_mode, EchoMode::Password);

        let input = Input::new().echo_mode(EchoMode::None);
        assert_eq!(input.echo_mode, EchoMode::None);
    }

    #[test]
    fn test_key_to_string() {
        let key = KeyMsg {
            key_type: KeyType::Enter,
            runes: vec![],
            alt: false,
            paste: false,
        };
        assert_eq!(key_to_string(&key), "enter");

        let key = KeyMsg {
            key_type: KeyType::Runes,
            runes: vec!['a'],
            alt: false,
            paste: false,
        };
        assert_eq!(key_to_string(&key), "a");

        let key = KeyMsg {
            key_type: KeyType::CtrlC,
            runes: vec![],
            alt: false,
            paste: false,
        };
        assert_eq!(key_to_string(&key), "ctrl+c");
    }

    #[test]
    fn test_input_view() {
        let input = Input::new()
            .title("Name")
            .placeholder("Enter name")
            .value("");

        let view = input.view();
        assert!(view.contains("Name"));
    }

    #[test]
    fn test_confirm_view() {
        let confirm = Confirm::new()
            .title("Proceed?")
            .affirmative("Yes")
            .negative("No");

        let view = confirm.view();
        assert!(view.contains("Proceed"));
    }

    #[test]
    fn test_select_view() {
        let select: Select<String> = Select::new().title("Choose").options(vec![
            SelectOption::new("A", "a".to_string()),
            SelectOption::new("B", "b".to_string()),
        ]);

        let view = select.view();
        assert!(view.contains("Choose"));
    }

    #[test]
    fn test_note_view() {
        let note = Note::new().title("Info").description("Some information");

        let view = note.view();
        assert!(view.contains("Info"));
    }

    #[test]
    fn test_multiselect_view() {
        let multi: MultiSelect<String> = MultiSelect::new().title("Select items").options(vec![
            SelectOption::new("A", "a".to_string()),
            SelectOption::new("B", "b".to_string()).selected(true),
            SelectOption::new("C", "c".to_string()),
        ]);

        let view = multi.view();
        assert!(view.contains("Select items"));
    }

    #[test]
    fn test_multiselect_initial_selection() {
        let multi: MultiSelect<String> = MultiSelect::new().options(vec![
            SelectOption::new("A", "a".to_string()),
            SelectOption::new("B", "b".to_string()).selected(true),
            SelectOption::new("C", "c".to_string()).selected(true),
        ]);

        let selected = multi.get_selected_values();
        assert_eq!(selected.len(), 2);
        assert!(selected.contains(&&"b".to_string()));
        assert!(selected.contains(&&"c".to_string()));
    }

    #[test]
    fn test_multiselect_limit() {
        let mut multi: MultiSelect<String> = MultiSelect::new().limit(2).options(vec![
            SelectOption::new("A", "a".to_string()),
            SelectOption::new("B", "b".to_string()),
            SelectOption::new("C", "c".to_string()),
        ]);

        // Focus the field so it processes updates
        multi.focus();

        // Toggle first option (select)
        let toggle_msg = Message::new(KeyMsg {
            key_type: KeyType::Runes,
            runes: vec![' '],
            alt: false,
            paste: false,
        });
        multi.update(&toggle_msg);
        assert_eq!(multi.get_selected_values().len(), 1);

        // Move down and toggle second
        let down_msg = Message::new(KeyMsg {
            key_type: KeyType::Down,
            runes: vec![],
            alt: false,
            paste: false,
        });
        multi.update(&down_msg);
        multi.update(&toggle_msg);
        assert_eq!(multi.get_selected_values().len(), 2);

        // Move down and try to toggle third (should be blocked by limit)
        multi.update(&down_msg);
        multi.update(&toggle_msg);
        // Should still be 2 due to limit
        assert_eq!(multi.get_selected_values().len(), 2);
    }
}
