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
    /// Style for help text at the bottom of the form.
    pub help: Style,
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
        help: Style::new().foreground("241").margin_top(1),
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
    t.help = Style::new().foreground("241").margin_top(1);

    t
}

/// Returns the Dracula theme.
pub fn theme_dracula() -> Theme {
    let mut t = theme_base();

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
    t.help = Style::new().foreground(comment).margin_top(1);

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
    t.help = Style::new().foreground("8").margin_top(1);

    t
}

/// Returns the Catppuccin theme.
///
/// This theme is based on the Catppuccin color scheme (Mocha variant).
/// See <https://github.com/catppuccin/catppuccin> for more details.
pub fn theme_catppuccin() -> Theme {
    let mut t = theme_base();

    // Catppuccin Mocha palette
    let base = "#1e1e2e";
    let text = "#cdd6f4";
    let subtext1 = "#bac2de";
    let subtext0 = "#a6adc8";
    let _overlay1 = "#7f849c";
    let overlay0 = "#6c7086";
    let green = "#a6e3a1";
    let red = "#f38ba8";
    let pink = "#f5c2e7";
    let mauve = "#cba6f7";
    let rosewater = "#f5e0dc";

    t.focused.base = t.focused.base.border_foreground(subtext1);
    t.focused.title = t.focused.title.foreground(mauve);
    t.focused.note_title = t.focused.note_title.foreground(mauve);
    t.focused.description = t.focused.description.foreground(subtext0);
    t.focused.error_indicator = t.focused.error_indicator.foreground(red);
    t.focused.error_message = t.focused.error_message.foreground(red);
    t.focused.select_selector = t.focused.select_selector.foreground(pink);
    t.focused.next_indicator = t.focused.next_indicator.foreground(pink);
    t.focused.prev_indicator = t.focused.prev_indicator.foreground(pink);
    t.focused.option = t.focused.option.foreground(text);
    t.focused.multi_select_selector = t.focused.multi_select_selector.foreground(pink);
    t.focused.selected_option = t.focused.selected_option.foreground(green);
    t.focused.selected_prefix = t.focused.selected_prefix.foreground(green);
    t.focused.unselected_prefix = t.focused.unselected_prefix.foreground(text);
    t.focused.unselected_option = t.focused.unselected_option.foreground(text);
    t.focused.focused_button = t.focused.focused_button.foreground(base).background(pink);
    t.focused.blurred_button = t.focused.blurred_button.foreground(text).background(base);

    t.focused.text_input.cursor = t.focused.text_input.cursor.foreground(rosewater);
    t.focused.text_input.placeholder = t.focused.text_input.placeholder.foreground(overlay0);
    t.focused.text_input.prompt = t.focused.text_input.prompt.foreground(pink);

    t.blurred = t.focused.clone();
    t.blurred.base = t.blurred.base.border(Border::hidden());
    t.blurred.next_indicator = Style::new();
    t.blurred.prev_indicator = Style::new();

    t.group.title = t.focused.title.clone();
    t.group.description = t.focused.description.clone();
    t.help = Style::new().foreground(subtext0).margin_top(1);

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
    /// File picker keybindings.
    pub file_picker: FilePickerKeyMap,
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
            file_picker: FilePickerKeyMap::default(),
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
    /// Uppercase word forward.
    pub uppercase_word_forward: Binding,
    /// Lowercase word forward.
    pub lowercase_word_forward: Binding,
    /// Capitalize word forward.
    pub capitalize_word_forward: Binding,
    /// Transpose character backward.
    pub transpose_character_backward: Binding,
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
            uppercase_word_forward: Binding::new()
                .keys(&["alt+u"])
                .help("alt+u", "uppercase word"),
            lowercase_word_forward: Binding::new()
                .keys(&["alt+l"])
                .help("alt+l", "lowercase word"),
            capitalize_word_forward: Binding::new()
                .keys(&["alt+c"])
                .help("alt+c", "capitalize word"),
            transpose_character_backward: Binding::new()
                .keys(&["ctrl+t"])
                .help("ctrl+t", "transpose"),
        }
    }
}

/// Keybindings for file picker fields.
#[derive(Debug, Clone)]
pub struct FilePickerKeyMap {
    /// Go to next field.
    pub next: Binding,
    /// Go to previous field.
    pub prev: Binding,
    /// Submit the form.
    pub submit: Binding,
    /// Move up in file list.
    pub up: Binding,
    /// Move down in file list.
    pub down: Binding,
    /// Open directory or select file.
    pub open: Binding,
    /// Close picker / go back.
    pub close: Binding,
    /// Go back to parent directory.
    pub back: Binding,
    /// Select current item.
    pub select: Binding,
    /// Go to top of list.
    pub goto_top: Binding,
    /// Go to bottom of list.
    pub goto_bottom: Binding,
    /// Page up.
    pub page_up: Binding,
    /// Page down.
    pub page_down: Binding,
}

impl Default for FilePickerKeyMap {
    fn default() -> Self {
        Self {
            prev: Binding::new()
                .keys(&["shift+tab"])
                .help("shift+tab", "back"),
            next: Binding::new().keys(&["tab"]).help("tab", "next"),
            submit: Binding::new().keys(&["enter"]).help("enter", "submit"),
            up: Binding::new().keys(&["up", "k"]).help("↑/k", "up"),
            down: Binding::new().keys(&["down", "j"]).help("↓/j", "down"),
            open: Binding::new().keys(&["enter", "l"]).help("enter", "open"),
            close: Binding::new().keys(&["esc", "q"]).help("esc", "close"),
            back: Binding::new().keys(&["backspace", "h"]).help("h", "back"),
            select: Binding::new().keys(&["enter"]).help("enter", "select"),
            goto_top: Binding::new().keys(&["g"]).help("g", "first"),
            goto_bottom: Binding::new().keys(&["G"]).help("G", "last"),
            page_up: Binding::new().keys(&["pgup", "K"]).help("pgup", "page up"),
            page_down: Binding::new()
                .keys(&["pgdown", "J"])
                .help("pgdown", "page down"),
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

/// Check if a KeyMsg matches a Binding.
fn binding_matches(binding: &Binding, key: &KeyMsg) -> bool {
    if !binding.enabled() {
        return false;
    }
    let key_str = key.to_string();
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
        self.cursor_pos = self.value.chars().count();
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
            EchoMode::Password => "•".repeat(self.value.chars().count()),
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
            // Note: cursor_pos is a character index (not byte index) for proper Unicode support
            match key_msg.key_type {
                KeyType::Runes => {
                    // Preprocess paste content: for single-line inputs, collapse newlines/tabs to spaces
                    let chars_to_insert: Vec<char> = if key_msg.paste {
                        key_msg
                            .runes
                            .iter()
                            .map(|&c| {
                                if c == '\n' || c == '\r' || c == '\t' {
                                    ' '
                                } else {
                                    c
                                }
                            })
                            // Collapse multiple consecutive spaces into one
                            .fold(Vec::new(), |mut acc, c| {
                                if c == ' ' && acc.last() == Some(&' ') {
                                    // Skip duplicate space
                                } else {
                                    acc.push(c);
                                }
                                acc
                            })
                    } else {
                        key_msg.runes.clone()
                    };

                    // Calculate how many chars we can insert respecting char_limit
                    let current_count = self.value.chars().count();
                    let available = if self.char_limit == 0 {
                        usize::MAX
                    } else {
                        self.char_limit.saturating_sub(current_count)
                    };
                    let chars_to_add: Vec<char> =
                        chars_to_insert.into_iter().take(available).collect();

                    if !chars_to_add.is_empty() {
                        // Convert character position to byte position for insertion
                        let byte_pos = self
                            .value
                            .char_indices()
                            .nth(self.cursor_pos)
                            .map(|(i, _)| i)
                            .unwrap_or(self.value.len());

                        // Build the new string efficiently for bulk insert
                        let insert_str: String = chars_to_add.iter().collect();
                        self.value.insert_str(byte_pos, &insert_str);
                        self.cursor_pos += chars_to_add.len();
                    }
                }
                KeyType::Backspace => {
                    if self.cursor_pos > 0 {
                        self.cursor_pos -= 1;
                        // Convert character position to byte position for removal
                        if let Some((byte_pos, _)) = self.value.char_indices().nth(self.cursor_pos)
                        {
                            self.value.remove(byte_pos);
                        }
                    }
                }
                KeyType::Delete => {
                    let char_count = self.value.chars().count();
                    if self.cursor_pos < char_count {
                        // Convert character position to byte position for removal
                        if let Some((byte_pos, _)) = self.value.char_indices().nth(self.cursor_pos)
                        {
                            self.value.remove(byte_pos);
                        }
                    }
                }
                KeyType::Left => {
                    if self.cursor_pos > 0 {
                        self.cursor_pos -= 1;
                    }
                }
                KeyType::Right => {
                    let char_count = self.value.chars().count();
                    if self.cursor_pos < char_count {
                        self.cursor_pos += 1;
                    }
                }
                KeyType::Home => {
                    self.cursor_pos = 0;
                }
                KeyType::End => {
                    self.cursor_pos = self.value.chars().count();
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

        styles
            .base
            .width(self.width.try_into().unwrap_or(u16::MAX))
            .render(&output)
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
            let has_visible = !visible.is_empty();
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
            if has_visible {
                output.pop();
            }
        }

        // Error indicator
        if self.error.is_some() {
            output.push_str(&styles.error_indicator.render(""));
        }

        styles
            .base
            .width(self.width.try_into().unwrap_or(u16::MAX))
            .render(&output)
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

    /// Enables or disables filtering mode.
    ///
    /// When enabled, pressing '/' enters filter mode where typing filters options.
    pub fn filterable(mut self, enabled: bool) -> Self {
        self.filtering = enabled;
        self
    }

    /// Updates the filter value with proper cursor adjustment.
    ///
    /// This method ensures the cursor stays on the same item when possible,
    /// or clamps to valid bounds if the current item is filtered out.
    fn update_filter(&mut self, new_value: String) {
        // Remember what item cursor is currently pointing to (original index)
        let old_filtered = self.filtered_options();
        let current_item_idx = old_filtered.get(self.cursor).map(|(idx, _)| *idx);

        // Update the filter
        self.filter_value = new_value;

        // Recalculate filtered options
        let new_filtered = self.filtered_options();

        // Try to keep cursor on the same item
        if let Some(item_idx) = current_item_idx
            && let Some(new_pos) = new_filtered.iter().position(|(idx, _)| *idx == item_idx)
        {
            self.cursor = new_pos;
            self.adjust_offset();
            return;
        }

        // Item no longer visible, clamp cursor to valid range
        self.cursor = self.cursor.min(new_filtered.len().saturating_sub(1));
        self.adjust_offset();
    }

    /// Adjusts the offset to keep the cursor visible within the view.
    fn adjust_offset(&mut self) {
        // Ensure cursor is within visible window
        if self.cursor < self.offset {
            self.offset = self.cursor;
        } else if self.cursor >= self.offset + self.height {
            self.offset = self.cursor.saturating_sub(self.height - 1);
        }
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

            // Handle filter input when filtering is enabled
            if self.filtering {
                // Clear filter on Escape
                if key_msg.key_type == KeyType::Esc {
                    self.update_filter(String::new());
                    return None;
                }

                // Remove character on Backspace
                if key_msg.key_type == KeyType::Backspace {
                    if !self.filter_value.is_empty() {
                        let mut new_filter = self.filter_value.clone();
                        new_filter.pop();
                        self.update_filter(new_filter);
                    }
                    return None;
                }

                // Add characters to filter
                if key_msg.key_type == KeyType::Runes {
                    let mut new_filter = self.filter_value.clone();
                    for c in &key_msg.runes {
                        // Only add printable characters that aren't navigation/toggle keys
                        // Always skip these keys so they work for navigation/toggle
                        match c {
                            'j' | 'k' | 'g' | 'G' | ' ' | 'x' | '/' => continue,
                            _ => {}
                        }
                        if c.is_alphanumeric() || c.is_whitespace() || c.is_ascii_punctuation() {
                            new_filter.push(*c);
                        }
                    }
                    if new_filter != self.filter_value {
                        self.update_filter(new_filter);
                        return None;
                    }
                }
            }

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

        // Filter input (if filtering is enabled and filter is active)
        if self.filtering && !self.filter_value.is_empty() {
            let filter_display = format!("Filter: {}_", self.filter_value);
            output.push_str(&styles.description.render(&filter_display));
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

        styles
            .base
            .width(self.width.try_into().unwrap_or(u16::MAX))
            .render(&output)
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

        styles
            .base
            .width(self.width.try_into().unwrap_or(u16::MAX))
            .render(&output)
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

    /// Sets the next button label (alias for `next_label`).
    ///
    /// This method is provided for compatibility with Go's huh API.
    pub fn next(self, label: impl Into<String>) -> Self {
        self.next_label(label)
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

        styles
            .base
            .width(self.width.try_into().unwrap_or(u16::MAX))
            .render(&output)
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
// Text Field (Textarea)
// -----------------------------------------------------------------------------

/// A multi-line text area field.
///
/// The Text field is used for gathering longer-form user input.
/// It wraps the bubbles textarea component and integrates it with the huh form system.
///
/// # Example
///
/// ```rust,ignore
/// use huh::Text;
///
/// let text = Text::new()
///     .key("bio")
///     .title("Biography")
///     .description("Tell us about yourself")
///     .placeholder("Enter your bio...")
///     .lines(5);
/// ```
pub struct Text {
    id: usize,
    key: String,
    value: String,
    title: String,
    description: String,
    placeholder: String,
    lines: usize,
    char_limit: usize,
    show_line_numbers: bool,
    focused: bool,
    error: Option<String>,
    validate: Option<fn(&str) -> Option<String>>,
    width: usize,
    height: usize,
    theme: Option<Theme>,
    keymap: TextKeyMap,
    _position: FieldPosition,
    cursor_row: usize,
    cursor_col: usize,
}

impl Default for Text {
    fn default() -> Self {
        Self::new()
    }
}

impl Text {
    /// Creates a new text area field.
    pub fn new() -> Self {
        Self {
            id: next_id(),
            key: String::new(),
            value: String::new(),
            title: String::new(),
            description: String::new(),
            placeholder: String::new(),
            lines: 5,
            char_limit: 0,
            show_line_numbers: false,
            focused: false,
            error: None,
            validate: None,
            width: 80,
            height: 0,
            theme: None,
            keymap: TextKeyMap::default(),
            _position: FieldPosition::default(),
            cursor_row: 0,
            cursor_col: 0,
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

    /// Sets the number of visible lines.
    pub fn lines(mut self, lines: usize) -> Self {
        self.lines = lines;
        self
    }

    /// Sets the character limit (0 = no limit).
    pub fn char_limit(mut self, limit: usize) -> Self {
        self.char_limit = limit;
        self
    }

    /// Sets whether to show line numbers.
    pub fn show_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    /// Sets the validation function.
    pub fn validate(mut self, validate: fn(&str) -> Option<String>) -> Self {
        self.validate = Some(validate);
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

    /// Gets the current value.
    pub fn get_string_value(&self) -> &str {
        &self.value
    }

    /// Returns the field ID.
    pub fn id(&self) -> usize {
        self.id
    }

    fn visible_lines(&self) -> Vec<&str> {
        let lines: Vec<&str> = self.value.lines().collect();
        if lines.is_empty() { vec![""] } else { lines }
    }

    /// Transpose the character at cursor with the one before it.
    ///
    /// If at the end of the line, moves cursor back first. After swapping,
    /// moves cursor right if not at end of line. No-op if cursor is at
    /// beginning of line or line has fewer than 2 characters.
    fn transpose_left(&mut self) {
        let lines: Vec<String> = self.value.lines().map(String::from).collect();
        if self.cursor_row >= lines.len() {
            return;
        }

        let line_chars: Vec<char> = lines[self.cursor_row].chars().collect();

        // No-op if at beginning or line too short
        if self.cursor_col == 0 || line_chars.len() < 2 {
            return;
        }

        let mut col = self.cursor_col;

        // If at end, move back first
        if col >= line_chars.len() {
            col = line_chars.len() - 1;
            self.cursor_col = col;
        }

        // Swap chars at col-1 and col
        let mut new_chars = line_chars;
        new_chars.swap(col - 1, col);

        // Rebuild value
        let mut new_lines = lines;
        new_lines[self.cursor_row] = new_chars.into_iter().collect();
        self.value = new_lines.join("\n");

        // Move right if not at end of line
        let new_line_len = self
            .value
            .lines()
            .nth(self.cursor_row)
            .map(|l| l.chars().count())
            .unwrap_or(0);
        if self.cursor_col < new_line_len {
            self.cursor_col += 1;
        }
    }

    /// Helper for word operations - operates on current line.
    ///
    /// Skips whitespace forward, then processes each character in the word
    /// using the provided function. Moves cursor to the end of the word.
    fn do_word_right<F>(&mut self, mut f: F)
    where
        F: FnMut(usize, char) -> char,
    {
        let lines: Vec<String> = self.value.lines().map(String::from).collect();
        if self.cursor_row >= lines.len() {
            return;
        }

        let mut chars: Vec<char> = lines[self.cursor_row].chars().collect();
        let len = chars.len();

        // Skip spaces forward
        while self.cursor_col < len && chars[self.cursor_col].is_whitespace() {
            self.cursor_col += 1;
        }

        // Process word chars
        let mut char_idx = 0;
        while self.cursor_col < len && !chars[self.cursor_col].is_whitespace() {
            chars[self.cursor_col] = f(char_idx, chars[self.cursor_col]);
            self.cursor_col += 1;
            char_idx += 1;
        }

        // Rebuild value
        let mut new_lines = lines;
        new_lines[self.cursor_row] = chars.into_iter().collect();
        self.value = new_lines.join("\n");
    }

    /// Uppercase the word to the right of the cursor.
    fn uppercase_right(&mut self) {
        self.do_word_right(|_, c| c.to_uppercase().next().unwrap_or(c));
    }

    /// Lowercase the word to the right of the cursor.
    fn lowercase_right(&mut self) {
        self.do_word_right(|_, c| c.to_lowercase().next().unwrap_or(c));
    }

    /// Capitalize the word to the right (first char uppercase, rest unchanged).
    fn capitalize_right(&mut self) {
        self.do_word_right(|idx, c| {
            if idx == 0 {
                c.to_uppercase().next().unwrap_or(c)
            } else {
                c
            }
        });
    }
}

impl Field for Text {
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

            // Check for next/submit (tab submits in text area)
            if binding_matches(&self.keymap.next, key_msg)
                || binding_matches(&self.keymap.submit, key_msg)
            {
                self.run_validation();
                if self.error.is_some() {
                    return None;
                }
                return Some(Cmd::new(|| Message::new(NextFieldMsg)));
            }

            // Check for new line
            if binding_matches(&self.keymap.new_line, key_msg) {
                if self.char_limit == 0 || self.value.len() < self.char_limit {
                    self.value.push('\n');
                    self.cursor_row += 1;
                    self.cursor_col = 0;
                }
                return None;
            }

            // Check for word transformation operations
            if binding_matches(&self.keymap.uppercase_word_forward, key_msg) {
                self.uppercase_right();
                return None;
            }
            if binding_matches(&self.keymap.lowercase_word_forward, key_msg) {
                self.lowercase_right();
                return None;
            }
            if binding_matches(&self.keymap.capitalize_word_forward, key_msg) {
                self.capitalize_right();
                return None;
            }
            if binding_matches(&self.keymap.transpose_character_backward, key_msg) {
                self.transpose_left();
                return None;
            }

            // Handle text input
            match key_msg.key_type {
                KeyType::Runes => {
                    // Calculate how many chars we can insert respecting char_limit
                    let current_count = self.value.chars().count();
                    let available = if self.char_limit == 0 {
                        usize::MAX
                    } else {
                        self.char_limit.saturating_sub(current_count)
                    };

                    // For paste operations, handle bulk insert with proper cursor tracking
                    // Multi-line textareas preserve newlines
                    let chars_to_add: Vec<char> =
                        key_msg.runes.iter().copied().take(available).collect();

                    for c in chars_to_add {
                        self.value.push(c);
                        if c == '\n' {
                            self.cursor_row += 1;
                            self.cursor_col = 0;
                        } else {
                            self.cursor_col += 1;
                        }
                    }
                }
                KeyType::Backspace => {
                    if !self.value.is_empty() {
                        let removed = self.value.pop();
                        if removed == Some('\n') {
                            self.cursor_row = self.cursor_row.saturating_sub(1);
                            let lines = self.visible_lines();
                            self.cursor_col =
                                lines.get(self.cursor_row).map(|l| l.len()).unwrap_or(0);
                        } else {
                            self.cursor_col = self.cursor_col.saturating_sub(1);
                        }
                    }
                }
                KeyType::Enter => {
                    // Enter inserts newline in text areas
                    if self.char_limit == 0 || self.value.len() < self.char_limit {
                        self.value.push('\n');
                        self.cursor_row += 1;
                        self.cursor_col = 0;
                    }
                }
                KeyType::Up => {
                    self.cursor_row = self.cursor_row.saturating_sub(1);
                }
                KeyType::Down => {
                    let line_count = self.visible_lines().len();
                    if self.cursor_row < line_count.saturating_sub(1) {
                        self.cursor_row += 1;
                    }
                }
                KeyType::Left => {
                    if self.cursor_col > 0 {
                        self.cursor_col -= 1;
                    }
                }
                KeyType::Right => {
                    let lines = self.visible_lines();
                    let current_line_len = lines.get(self.cursor_row).map(|l| l.len()).unwrap_or(0);
                    if self.cursor_col < current_line_len {
                        self.cursor_col += 1;
                    }
                }
                KeyType::Home => {
                    self.cursor_col = 0;
                }
                KeyType::End => {
                    let lines = self.visible_lines();
                    self.cursor_col = lines.get(self.cursor_row).map(|l| l.len()).unwrap_or(0);
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
            if self.error.is_some() {
                output.push_str(&styles.error_indicator.render(""));
            }
            output.push('\n');
        }

        // Description
        if !self.description.is_empty() {
            output.push_str(&styles.description.render(&self.description));
            output.push('\n');
        }

        // Text area content
        let lines = self.visible_lines();
        let visible_lines = self.lines.min(lines.len().max(1));

        for (i, line) in lines.iter().take(visible_lines).enumerate() {
            if self.show_line_numbers {
                let line_num = format!("{:3} ", i + 1);
                output.push_str(&styles.description.render(&line_num));
            }

            if line.is_empty() && i == 0 && self.value.is_empty() && !self.placeholder.is_empty() {
                output.push_str(&styles.text_input.placeholder.render(&self.placeholder));
            } else {
                output.push_str(&styles.text_input.text.render(line));
            }

            if i < visible_lines - 1 {
                output.push('\n');
            }
        }

        // Pad with empty lines if needed
        for i in lines.len()..visible_lines {
            output.push('\n');
            if self.show_line_numbers {
                let line_num = format!("{:3} ", i + 1);
                output.push_str(&styles.description.render(&line_num));
            }
        }

        // Error message
        if let Some(ref err) = self.error {
            output.push('\n');
            output.push_str(&styles.error_message.render(err));
        }

        styles
            .base
            .width(self.width.try_into().unwrap_or(u16::MAX))
            .render(&output)
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
            self.keymap.new_line.clone(),
            self.keymap.prev.clone(),
            self.keymap.submit.clone(),
            self.keymap.next.clone(),
            self.keymap.uppercase_word_forward.clone(),
            self.keymap.lowercase_word_forward.clone(),
            self.keymap.capitalize_word_forward.clone(),
            self.keymap.transpose_character_backward.clone(),
        ]
    }

    fn with_theme(&mut self, theme: &Theme) {
        if self.theme.is_none() {
            self.theme = Some(theme.clone());
        }
    }

    fn with_keymap(&mut self, keymap: &KeyMap) {
        self.keymap = keymap.text.clone();
    }

    fn with_width(&mut self, width: usize) {
        self.width = width;
    }

    fn with_height(&mut self, height: usize) {
        self.height = height;
        // Adjust lines based on height minus title/description
        let adjust = if self.title.is_empty() { 0 } else { 1 }
            + if self.description.is_empty() { 0 } else { 1 };
        if height > adjust {
            self.lines = height - adjust;
        }
    }

    fn with_position(&mut self, position: FieldPosition) {
        self._position = position;
    }
}

// -----------------------------------------------------------------------------
// FilePicker Field
// -----------------------------------------------------------------------------

/// A file picker field for selecting files and directories.
///
/// The FilePicker field allows users to browse the filesystem and select files
/// or directories. It can be configured to filter by file type, show/hide hidden
/// files, and control whether files and/or directories can be selected.
///
/// # Example
///
/// ```rust,ignore
/// use huh::FilePicker;
///
/// let picker = FilePicker::new()
///     .key("config_file")
///     .title("Select Configuration File")
///     .description("Choose a .toml or .json file")
///     .allowed_types(vec![".toml".to_string(), ".json".to_string()])
///     .current_directory(".");
/// ```
pub struct FilePicker {
    id: usize,
    key: String,
    selected_path: Option<String>,
    title: String,
    description: String,
    current_directory: String,
    allowed_types: Vec<String>,
    show_hidden: bool,
    show_size: bool,
    show_permissions: bool,
    file_allowed: bool,
    dir_allowed: bool,
    picking: bool,
    focused: bool,
    error: Option<String>,
    validate: Option<fn(&str) -> Option<String>>,
    width: usize,
    height: usize,
    theme: Option<Theme>,
    keymap: FilePickerKeyMap,
    _position: FieldPosition,
    // Simple file list for display
    files: Vec<FileEntry>,
    selected_index: usize,
    offset: usize,
}

/// A file entry in the picker.
#[derive(Debug, Clone)]
struct FileEntry {
    name: String,
    path: String,
    is_dir: bool,
    size: u64,
    #[allow(dead_code)]
    mode: String,
}

impl Default for FilePicker {
    fn default() -> Self {
        Self::new()
    }
}

impl FilePicker {
    /// Creates a new file picker field.
    pub fn new() -> Self {
        Self {
            id: next_id(),
            key: String::new(),
            selected_path: None,
            title: String::new(),
            description: String::new(),
            current_directory: ".".to_string(),
            allowed_types: Vec::new(),
            show_hidden: false,
            show_size: false,
            show_permissions: false,
            file_allowed: true,
            dir_allowed: false,
            picking: false,
            focused: false,
            error: None,
            validate: None,
            width: 80,
            height: 10,
            theme: None,
            keymap: FilePickerKeyMap::default(),
            _position: FieldPosition::default(),
            files: Vec::new(),
            selected_index: 0,
            offset: 0,
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

    /// Sets the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Sets the starting directory.
    pub fn current_directory(mut self, dir: impl Into<String>) -> Self {
        self.current_directory = dir.into();
        self
    }

    /// Sets the allowed file types (extensions).
    pub fn allowed_types(mut self, types: Vec<String>) -> Self {
        self.allowed_types = types;
        self
    }

    /// Sets whether to show hidden files.
    pub fn show_hidden(mut self, show: bool) -> Self {
        self.show_hidden = show;
        self
    }

    /// Sets whether to show file sizes.
    pub fn show_size(mut self, show: bool) -> Self {
        self.show_size = show;
        self
    }

    /// Sets whether to show file permissions.
    pub fn show_permissions(mut self, show: bool) -> Self {
        self.show_permissions = show;
        self
    }

    /// Sets whether files can be selected.
    pub fn file_allowed(mut self, allowed: bool) -> Self {
        self.file_allowed = allowed;
        self
    }

    /// Sets whether directories can be selected.
    pub fn dir_allowed(mut self, allowed: bool) -> Self {
        self.dir_allowed = allowed;
        self
    }

    /// Sets the validation function.
    pub fn validate(mut self, validate: fn(&str) -> Option<String>) -> Self {
        self.validate = Some(validate);
        self
    }

    /// Sets the visible height (number of entries shown).
    pub fn height_entries(mut self, height: usize) -> Self {
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
            && let Some(ref path) = self.selected_path
        {
            self.error = validate(path);
        }
    }

    fn read_directory(&mut self) {
        self.files.clear();
        self.selected_index = 0;
        self.offset = 0;

        // Add parent directory entry if not at root
        if self.current_directory != "/" {
            self.files.push(FileEntry {
                name: "..".to_string(),
                path: "..".to_string(),
                is_dir: true,
                size: 0,
                mode: String::new(),
            });
        }

        // Read directory contents
        if let Ok(entries) = std::fs::read_dir(&self.current_directory) {
            let mut entries: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter_map(|entry| {
                    let name = entry.file_name().to_string_lossy().to_string();

                    // Skip hidden files unless show_hidden is true
                    if !self.show_hidden && name.starts_with('.') {
                        return None;
                    }

                    let metadata = entry.metadata().ok()?;
                    let is_dir = metadata.is_dir();
                    let size = metadata.len();

                    // Filter by allowed types (only for files)
                    if !is_dir && !self.allowed_types.is_empty() {
                        let matches = self.allowed_types.iter().any(|ext| {
                            name.ends_with(ext)
                                || name.ends_with(&ext.trim_start_matches('.').to_string())
                        });
                        if !matches {
                            return None;
                        }
                    }

                    let path = entry.path().to_string_lossy().to_string();

                    Some(FileEntry {
                        name,
                        path,
                        is_dir,
                        size,
                        mode: String::new(),
                    })
                })
                .collect();

            // Sort: directories first, then alphabetically
            entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            });

            self.files.extend(entries);
        }
    }

    fn is_selectable(&self, entry: &FileEntry) -> bool {
        if entry.is_dir {
            self.dir_allowed
        } else {
            self.file_allowed
        }
    }

    fn format_size(size: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if size >= GB {
            format!("{:.1}G", size as f64 / GB as f64)
        } else if size >= MB {
            format!("{:.1}M", size as f64 / MB as f64)
        } else if size >= KB {
            format!("{:.1}K", size as f64 / KB as f64)
        } else {
            format!("{}B", size)
        }
    }

    /// Gets the currently selected path.
    pub fn get_selected_path(&self) -> Option<&str> {
        self.selected_path.as_deref()
    }

    /// Returns the field ID.
    pub fn id(&self) -> usize {
        self.id
    }
}

impl Field for FilePicker {
    fn get_key(&self) -> &str {
        &self.key
    }

    fn get_value(&self) -> Box<dyn Any> {
        Box::new(self.selected_path.clone().unwrap_or_default())
    }

    fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    fn init(&mut self) -> Option<Cmd> {
        self.read_directory();
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
                self.picking = false;
                return Some(Cmd::new(|| Message::new(PrevFieldMsg)));
            }

            // Check for next (tab)
            if binding_matches(&self.keymap.next, key_msg) {
                self.picking = false;
                self.run_validation();
                if self.error.is_some() {
                    return None;
                }
                return Some(Cmd::new(|| Message::new(NextFieldMsg)));
            }

            // Handle close/escape
            if binding_matches(&self.keymap.close, key_msg) {
                if self.picking {
                    self.picking = false;
                } else {
                    return Some(Cmd::new(|| Message::new(NextFieldMsg)));
                }
                return None;
            }

            // Handle open (enter picker mode or select)
            if binding_matches(&self.keymap.open, key_msg) {
                if !self.picking {
                    self.picking = true;
                    self.read_directory();
                    return None;
                }

                // In picking mode, open directory or select file
                if let Some(entry) = self.files.get(self.selected_index) {
                    if entry.name == ".." {
                        // Go to parent directory
                        if let Some(parent) = std::path::Path::new(&self.current_directory).parent()
                        {
                            self.current_directory = parent.to_string_lossy().to_string();
                            if self.current_directory.is_empty() {
                                self.current_directory = "/".to_string();
                            }
                            self.read_directory();
                        }
                    } else if entry.is_dir {
                        // Enter directory
                        self.current_directory = entry.path.clone();
                        self.read_directory();
                    } else if self.is_selectable(entry) {
                        // Select file
                        self.selected_path = Some(entry.path.clone());
                        self.picking = false;
                        self.run_validation();
                        if self.error.is_some() {
                            return None;
                        }
                        return Some(Cmd::new(|| Message::new(NextFieldMsg)));
                    }
                }
                return None;
            }

            // Handle back (go to parent directory)
            if self.picking && binding_matches(&self.keymap.back, key_msg) {
                if let Some(parent) = std::path::Path::new(&self.current_directory).parent() {
                    self.current_directory = parent.to_string_lossy().to_string();
                    if self.current_directory.is_empty() {
                        self.current_directory = "/".to_string();
                    }
                    self.read_directory();
                }
                return None;
            }

            // Navigation in picker mode
            if self.picking {
                if binding_matches(&self.keymap.up, key_msg) {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                        if self.selected_index < self.offset {
                            self.offset = self.selected_index;
                        }
                    }
                } else if binding_matches(&self.keymap.down, key_msg) {
                    if self.selected_index < self.files.len().saturating_sub(1) {
                        self.selected_index += 1;
                        if self.selected_index >= self.offset + self.height {
                            self.offset = self.selected_index.saturating_sub(self.height - 1);
                        }
                    }
                } else if binding_matches(&self.keymap.goto_top, key_msg) {
                    self.selected_index = 0;
                    self.offset = 0;
                } else if binding_matches(&self.keymap.goto_bottom, key_msg) {
                    self.selected_index = self.files.len().saturating_sub(1);
                    self.offset = self.selected_index.saturating_sub(self.height - 1);
                }
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
            if self.error.is_some() {
                output.push_str(&styles.error_indicator.render(""));
            }
            output.push('\n');
        }

        // Description
        if !self.description.is_empty() {
            output.push_str(&styles.description.render(&self.description));
            output.push('\n');
        }

        if self.picking {
            // Show file list
            let visible: Vec<_> = self
                .files
                .iter()
                .skip(self.offset)
                .take(self.height)
                .collect();

            for (i, entry) in visible.iter().enumerate() {
                let idx = self.offset + i;
                let is_selected = idx == self.selected_index;
                let is_selectable = self.is_selectable(entry);

                // Cursor
                if is_selected {
                    output.push_str(&styles.select_selector.render(""));
                } else {
                    output.push_str("  ");
                }

                // Entry display
                let mut entry_str = String::new();

                // Directory/file indicator
                if entry.is_dir {
                    entry_str.push_str("📁 ");
                } else {
                    entry_str.push_str("   ");
                }

                entry_str.push_str(&entry.name);

                // Size
                if self.show_size && !entry.is_dir {
                    entry_str.push_str(&format!(" ({})", Self::format_size(entry.size)));
                }

                if is_selected && is_selectable {
                    output.push_str(&styles.selected_option.render(&entry_str));
                } else if !is_selectable && !entry.is_dir && entry.name != ".." {
                    output.push_str(&styles.text_input.placeholder.render(&entry_str));
                } else {
                    output.push_str(&styles.option.render(&entry_str));
                }

                output.push('\n');
            }

            // Remove trailing newline
            if !visible.is_empty() {
                output.pop();
            }

            // Show current directory
            output.push('\n');
            output.push_str(
                &styles
                    .description
                    .render(&format!("📂 {}", self.current_directory)),
            );
        } else {
            // Show selected file or placeholder
            if let Some(ref path) = self.selected_path {
                output.push_str(&styles.selected_option.render(path));
            } else {
                output.push_str(
                    &styles
                        .text_input
                        .placeholder
                        .render("No file selected. Press Enter to browse."),
                );
            }
        }

        // Error message
        if let Some(ref err) = self.error {
            output.push('\n');
            output.push_str(&styles.error_message.render(err));
        }

        styles
            .base
            .width(self.width.try_into().unwrap_or(u16::MAX))
            .render(&output)
    }

    fn focus(&mut self) -> Option<Cmd> {
        self.focused = true;
        None
    }

    fn blur(&mut self) -> Option<Cmd> {
        self.focused = false;
        self.picking = false;
        self.run_validation();
        None
    }

    fn key_binds(&self) -> Vec<Binding> {
        if self.picking {
            vec![
                self.keymap.up.clone(),
                self.keymap.down.clone(),
                self.keymap.open.clone(),
                self.keymap.back.clone(),
                self.keymap.close.clone(),
            ]
        } else {
            vec![
                self.keymap.open.clone(),
                self.keymap.prev.clone(),
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
        self.keymap = keymap.file_picker.clone();
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

    /// Returns the header portion of the group (title and description).
    ///
    /// This is useful for custom layouts that want to render the header
    /// separately from the content.
    pub fn header(&self) -> String {
        let theme = self.get_theme();
        let mut output = String::new();

        if !self.title.is_empty() {
            output.push_str(&theme.group.title.render(&self.title));
            output.push('\n');
        }

        if !self.description.is_empty() {
            output.push_str(&theme.group.description.render(&self.description));
            output.push('\n');
        }

        output
    }

    /// Returns the content portion of the group (just the fields).
    ///
    /// This is useful for custom layouts that want to render the content
    /// separately from the header and footer.
    pub fn content(&self) -> String {
        let theme = self.get_theme();
        let mut output = String::new();

        for (i, field) in self.fields.iter().enumerate() {
            output.push_str(&field.view());
            if i < self.fields.len() - 1 {
                output.push_str(&theme.field_separator.render(""));
            }
        }

        output
    }

    /// Returns the footer portion of the group (currently errors).
    ///
    /// This is useful for custom layouts that want to render the footer
    /// separately from the content.
    pub fn footer(&self) -> String {
        let theme = self.get_theme();
        let errors = self.errors();

        if errors.is_empty() {
            return String::new();
        }

        let error_text = errors.join(", ");
        theme.focused.error_message.render(&error_text)
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

        theme
            .group
            .base
            .width(self.width.try_into().unwrap_or(u16::MAX))
            .render(&output)
    }
}

// -----------------------------------------------------------------------------
// Layout
// -----------------------------------------------------------------------------

/// Layout determines how groups are arranged within a form.
///
/// The layout system controls how multiple groups are displayed:
/// - `Default`: Shows one group at a time (traditional wizard-style)
/// - `Stack`: Shows all groups stacked vertically
/// - `Columns`: Distributes groups across columns
/// - `Grid`: Arranges groups in a grid pattern
pub trait Layout: Send + Sync {
    /// Renders the form using this layout.
    fn view(&self, form: &Form) -> String;

    /// Returns the width allocated to a specific group.
    fn group_width(&self, form: &Form, group_index: usize, total_width: usize) -> usize;
}

/// Default layout - shows one group at a time.
///
/// This is the traditional wizard-style form layout where only the
/// current group is visible and users navigate between groups.
#[derive(Debug, Clone, Default)]
pub struct LayoutDefault;

impl Layout for LayoutDefault {
    fn view(&self, form: &Form) -> String {
        if let Some(group) = form.groups.get(form.current_group) {
            if group.is_hidden() {
                return String::new();
            }
            form.theme
                .form
                .base
                .clone()
                .width(form.width.try_into().unwrap_or(u16::MAX))
                .render(&group.view())
        } else {
            String::new()
        }
    }

    fn group_width(&self, form: &Form, _group_index: usize, _total_width: usize) -> usize {
        form.width
    }
}

/// Stack layout - shows all groups stacked vertically.
///
/// All groups are rendered one after another, with the form's
/// field separator between them.
#[derive(Debug, Clone, Default)]
pub struct LayoutStack;

impl Layout for LayoutStack {
    fn view(&self, form: &Form) -> String {
        let mut output = String::new();
        let visible_groups: Vec<_> = form
            .groups
            .iter()
            .enumerate()
            .filter(|(_, g)| !g.is_hidden())
            .collect();

        for (i, (_, group)) in visible_groups.iter().enumerate() {
            output.push_str(&group.view());
            if i < visible_groups.len() - 1 {
                output.push('\n');
            }
        }

        form.theme
            .form
            .base
            .clone()
            .width(form.width.try_into().unwrap_or(u16::MAX))
            .render(&output)
    }

    fn group_width(&self, form: &Form, _group_index: usize, _total_width: usize) -> usize {
        form.width
    }
}

/// Columns layout - distributes groups across columns.
///
/// Groups are arranged in columns, wrapping to the next row when needed.
#[derive(Debug, Clone)]
pub struct LayoutColumns {
    columns: usize,
}

impl LayoutColumns {
    /// Creates a new columns layout with the specified number of columns.
    pub fn new(columns: usize) -> Self {
        Self {
            columns: columns.max(1),
        }
    }
}

impl Default for LayoutColumns {
    fn default() -> Self {
        Self::new(2)
    }
}

impl Layout for LayoutColumns {
    fn view(&self, form: &Form) -> String {
        let visible_groups: Vec<_> = form
            .groups
            .iter()
            .enumerate()
            .filter(|(_, g)| !g.is_hidden())
            .collect();

        if visible_groups.is_empty() {
            return String::new();
        }

        let column_width = form.width / self.columns;
        let mut rows: Vec<String> = Vec::new();

        for chunk in visible_groups.chunks(self.columns) {
            let mut row_parts: Vec<String> = Vec::new();
            for (_, group) in chunk {
                // Render each group with column width
                let group_view = group.view();
                // Pad to column width
                let lines: Vec<&str> = group_view.lines().collect();
                let padded: Vec<String> = lines
                    .iter()
                    .map(|line| {
                        let visual_width = lipgloss::width(line);
                        if visual_width < column_width {
                            format!("{}{}", line, " ".repeat(column_width - visual_width))
                        } else {
                            line.to_string()
                        }
                    })
                    .collect();
                row_parts.push(padded.join("\n"));
            }

            // Join columns horizontally using lipgloss
            if row_parts.len() == 1 {
                rows.push(row_parts.into_iter().next().unwrap());
            } else {
                let row_refs: Vec<&str> = row_parts.iter().map(|s| s.as_str()).collect();
                rows.push(lipgloss::join_horizontal(
                    lipgloss::Position::Top,
                    &row_refs,
                ));
            }
        }

        let output = rows.join("\n");
        form.theme
            .form
            .base
            .clone()
            .width(form.width.try_into().unwrap_or(u16::MAX))
            .render(&output)
    }

    fn group_width(&self, form: &Form, _group_index: usize, _total_width: usize) -> usize {
        form.width / self.columns
    }
}

/// Grid layout - arranges groups in a fixed grid pattern.
///
/// Groups are arranged in a grid with the specified number of rows and columns.
/// If there are more groups than cells, extra groups are not displayed.
#[derive(Debug, Clone)]
pub struct LayoutGrid {
    rows: usize,
    columns: usize,
}

impl LayoutGrid {
    /// Creates a new grid layout with the specified dimensions.
    pub fn new(rows: usize, columns: usize) -> Self {
        Self {
            rows: rows.max(1),
            columns: columns.max(1),
        }
    }
}

impl Default for LayoutGrid {
    fn default() -> Self {
        Self::new(2, 2)
    }
}

impl Layout for LayoutGrid {
    fn view(&self, form: &Form) -> String {
        let visible_groups: Vec<_> = form
            .groups
            .iter()
            .enumerate()
            .filter(|(_, g)| !g.is_hidden())
            .collect();

        if visible_groups.is_empty() {
            return String::new();
        }

        let column_width = form.width / self.columns;
        let max_cells = self.rows * self.columns;
        let mut rows: Vec<String> = Vec::new();

        for row_idx in 0..self.rows {
            let start = row_idx * self.columns;
            if start >= visible_groups.len() || start >= max_cells {
                break;
            }
            let end = (start + self.columns)
                .min(visible_groups.len())
                .min(max_cells);

            let mut row_parts: Vec<String> = Vec::new();
            for (_, group) in &visible_groups[start..end] {
                let group_view = group.view();
                let lines: Vec<&str> = group_view.lines().collect();
                let padded: Vec<String> = lines
                    .iter()
                    .map(|line| {
                        let visual_width = lipgloss::width(line);
                        if visual_width < column_width {
                            format!("{}{}", line, " ".repeat(column_width - visual_width))
                        } else {
                            line.to_string()
                        }
                    })
                    .collect();
                row_parts.push(padded.join("\n"));
            }

            if row_parts.len() == 1 {
                rows.push(row_parts.into_iter().next().unwrap());
            } else {
                let row_refs: Vec<&str> = row_parts.iter().map(|s| s.as_str()).collect();
                rows.push(lipgloss::join_horizontal(
                    lipgloss::Position::Top,
                    &row_refs,
                ));
            }
        }

        let output = rows.join("\n");
        form.theme
            .form
            .base
            .clone()
            .width(form.width.try_into().unwrap_or(u16::MAX))
            .render(&output)
    }

    fn group_width(&self, form: &Form, _group_index: usize, _total_width: usize) -> usize {
        form.width / self.columns
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
    layout: Box<dyn Layout>,
    show_help: bool,
    show_errors: bool,
    accessible: bool,
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
            layout: Box::new(LayoutDefault),
            show_help: true,
            show_errors: true,
            accessible: false,
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

    /// Sets the layout for the form.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use huh::{Form, Group, LayoutColumns};
    ///
    /// let form = Form::new(vec![group1, group2, group3])
    ///     .layout(LayoutColumns::new(2));
    /// ```
    pub fn layout<L: Layout + 'static>(mut self, layout: L) -> Self {
        self.layout = Box::new(layout);
        self
    }

    /// Sets whether to show help at the bottom of the form.
    pub fn show_help(mut self, show: bool) -> Self {
        self.show_help = show;
        self
    }

    /// Sets whether to show validation errors.
    pub fn show_errors(mut self, show: bool) -> Self {
        self.show_errors = show;
        self
    }

    /// Enables or disables accessible mode.
    ///
    /// When accessible mode is enabled, the form renders in a more
    /// screen-reader-friendly format with simpler styling and clearer
    /// field labels. This mode prioritizes accessibility over visual
    /// aesthetics.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use huh::Form;
    ///
    /// let form = Form::new(groups)
    ///     .with_accessible(true);
    /// ```
    pub fn with_accessible(mut self, accessible: bool) -> Self {
        self.accessible = accessible;
        self
    }

    /// Returns whether accessible mode is enabled.
    pub fn is_accessible(&self) -> bool {
        self.accessible
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

    /// Collects all validation errors from all groups.
    pub fn all_errors(&self) -> Vec<String> {
        self.groups
            .iter()
            .flat_map(|g| g.errors())
            .map(|s| s.to_string())
            .collect()
    }

    /// Returns a view of all validation errors.
    fn errors_view(&self) -> String {
        let errors = self.all_errors();
        if errors.is_empty() {
            return String::new();
        }

        let error_text = errors.join(", ");
        self.theme.focused.error_message.render(&error_text)
    }

    /// Returns a help view with available keybindings.
    fn help_view(&self) -> String {
        // Build help text from keybindings
        let mut help_parts = Vec::new();

        // Get current field's keybindings if available
        if let Some(group) = self.groups.get(self.current_group)
            && let Some(field) = group.fields.get(group.current)
        {
            for binding in field.key_binds() {
                let help = binding.get_help();
                if binding.enabled() && !help.desc.is_empty() {
                    let keys = binding.get_keys();
                    if !keys.is_empty() {
                        help_parts.push(format!("{}: {}", keys.join("/"), help.desc));
                    }
                }
            }
        }

        // Add form-level keybindings
        let quit_help = self.keymap.quit.get_help();
        if self.keymap.quit.enabled() && !quit_help.desc.is_empty() {
            let keys = self.keymap.quit.get_keys();
            if !keys.is_empty() {
                help_parts.push(format!("{}: {}", keys.join("/"), quit_help.desc));
            }
        }

        if help_parts.is_empty() {
            return String::new();
        }

        // Style the help text
        let help_text = help_parts.join(" • ");
        self.theme.help.render(&help_text)
    }

    /// Returns the width allocated to a specific group based on the current layout.
    pub fn group_width(&self, group_index: usize) -> usize {
        self.layout.group_width(self, group_index, self.width)
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
        let mut output = self.layout.view(self);

        // Add help footer if enabled
        if self.show_help {
            let help_text = self.help_view();
            if !help_text.is_empty() {
                output.push('\n');
                output.push_str(&help_text);
            }
        }

        // Add errors if enabled
        if self.show_errors {
            let errors = self.errors_view();
            if !errors.is_empty() {
                output.push('\n');
                output.push_str(&errors);
            }
        }

        output
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
    fn test_text_builder() {
        let text = Text::new()
            .key("bio")
            .title("Biography")
            .description("Tell us about yourself")
            .placeholder("Enter your bio...")
            .lines(10)
            .value("Hello world");

        assert_eq!(text.get_key(), "bio");
        assert_eq!(text.get_string_value(), "Hello world");
    }

    #[test]
    fn test_text_char_limit() {
        let text = Text::new().char_limit(50).show_line_numbers(true);

        assert_eq!(text.char_limit, 50);
        assert!(text.show_line_numbers);
    }

    #[test]
    fn test_filepicker_builder() {
        let picker = FilePicker::new()
            .key("config_file")
            .title("Select Configuration")
            .description("Choose a file")
            .current_directory("/tmp")
            .show_hidden(true)
            .file_allowed(true)
            .dir_allowed(false);

        assert_eq!(picker.get_key(), "config_file");
        assert!(picker.file_allowed);
        assert!(!picker.dir_allowed);
        assert!(picker.show_hidden);
    }

    #[test]
    fn test_filepicker_allowed_types() {
        let picker = FilePicker::new()
            .allowed_types(vec![".toml".to_string(), ".json".to_string()])
            .show_size(true);

        assert_eq!(picker.allowed_types.len(), 2);
        assert!(picker.show_size);
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
    fn test_theme_catppuccin() {
        let theme = theme_catppuccin();
        // Verify it doesn't panic and has expected Catppuccin colors
        let _ = theme.focused.title.render("Test");
        let _ = theme.focused.selected_option.render("Selected");
        let _ = theme.focused.focused_button.render("OK");
        let _ = theme.blurred.title.render("Blurred");
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
        assert_eq!(key.to_string(), "enter");

        let key = KeyMsg {
            key_type: KeyType::Runes,
            runes: vec!['a'],
            alt: false,
            paste: false,
        };
        assert_eq!(key.to_string(), "a");

        let key = KeyMsg {
            key_type: KeyType::CtrlC,
            runes: vec![],
            alt: false,
            paste: false,
        };
        assert_eq!(key.to_string(), "ctrl+c");
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

    #[test]
    fn test_input_unicode_cursor_handling() {
        // Test that cursor position works correctly with multi-byte UTF-8 characters
        let mut input = Input::new().value("café"); // 'é' is 2 bytes in UTF-8

        // Focus to enable updates
        input.focus();

        // cursor_pos should be at end (4 characters, not 5 bytes)
        assert_eq!(input.cursor_pos, 4);
        assert_eq!(input.value.chars().count(), 4);

        // Press End to ensure cursor is at end
        let end_msg = Message::new(KeyMsg {
            key_type: KeyType::End,
            runes: vec![],
            alt: false,
            paste: false,
        });
        input.update(&end_msg);
        assert_eq!(input.cursor_pos, 4);

        // Press Left to move before 'é'
        let left_msg = Message::new(KeyMsg {
            key_type: KeyType::Left,
            runes: vec![],
            alt: false,
            paste: false,
        });
        input.update(&left_msg);
        assert_eq!(input.cursor_pos, 3);

        // Press Backspace to delete 'f'
        let backspace_msg = Message::new(KeyMsg {
            key_type: KeyType::Backspace,
            runes: vec![],
            alt: false,
            paste: false,
        });
        input.update(&backspace_msg);
        assert_eq!(input.get_string_value(), "caé");
        assert_eq!(input.cursor_pos, 2);

        // Insert a character at current position
        let insert_msg = Message::new(KeyMsg {
            key_type: KeyType::Runes,
            runes: vec!['ñ'], // Another multi-byte char
            alt: false,
            paste: false,
        });
        input.update(&insert_msg);
        assert_eq!(input.get_string_value(), "cañé");
        assert_eq!(input.cursor_pos, 3);

        // Delete character at cursor (should delete 'é')
        let delete_msg = Message::new(KeyMsg {
            key_type: KeyType::Delete,
            runes: vec![],
            alt: false,
            paste: false,
        });
        input.update(&delete_msg);
        assert_eq!(input.get_string_value(), "cañ");

        // Home should move to position 0
        let home_msg = Message::new(KeyMsg {
            key_type: KeyType::Home,
            runes: vec![],
            alt: false,
            paste: false,
        });
        input.update(&home_msg);
        assert_eq!(input.cursor_pos, 0);
    }

    #[test]
    fn test_input_char_limit_with_unicode() {
        // Test that char_limit counts characters, not bytes
        let mut input = Input::new().char_limit(5);
        input.focus();

        // Insert 5 multi-byte characters (each would be 2+ bytes in UTF-8)
        let chars = ['日', '本', '語', '文', '字']; // 5 Japanese characters
        for c in chars {
            let msg = Message::new(KeyMsg {
                key_type: KeyType::Runes,
                runes: vec![c],
                alt: false,
                paste: false,
            });
            input.update(&msg);
        }

        // Should have exactly 5 characters (not blocked due to byte count)
        assert_eq!(input.value.chars().count(), 5);
        assert_eq!(input.get_string_value(), "日本語文字");

        // Try to add one more - should be blocked by char limit
        let msg = Message::new(KeyMsg {
            key_type: KeyType::Runes,
            runes: vec!['!'],
            alt: false,
            paste: false,
        });
        input.update(&msg);

        // Should still be 5 characters
        assert_eq!(input.value.chars().count(), 5);
    }

    #[test]
    fn test_layout_default() {
        let _layout = LayoutDefault;
        // Just ensure it compiles and can be created
    }

    #[test]
    fn test_layout_stack() {
        let _layout = LayoutStack;
        // Just ensure it compiles and can be created
    }

    #[test]
    fn test_layout_columns() {
        let layout = LayoutColumns::new(3);
        assert_eq!(layout.columns, 3);

        // Minimum of 1 column
        let layout = LayoutColumns::new(0);
        assert_eq!(layout.columns, 1);
    }

    #[test]
    fn test_layout_grid() {
        let layout = LayoutGrid::new(2, 3);
        assert_eq!(layout.rows, 2);
        assert_eq!(layout.columns, 3);

        // Minimum of 1x1
        let layout = LayoutGrid::new(0, 0);
        assert_eq!(layout.rows, 1);
        assert_eq!(layout.columns, 1);
    }

    #[test]
    fn test_form_with_layout() {
        let form = Form::new(vec![
            Group::new(vec![Box::new(Input::new().key("a"))]),
            Group::new(vec![Box::new(Input::new().key("b"))]),
        ])
        .layout(LayoutColumns::new(2));

        // Form should have the layout set
        assert_eq!(form.len(), 2);
    }

    #[test]
    fn test_form_show_help() {
        let form = Form::new(Vec::new()).show_help(false).show_errors(false);

        // Just verify the builder works
        assert!(!form.show_help);
        assert!(!form.show_errors);
    }

    #[test]
    fn test_group_header_footer_content() {
        let group = Group::new(vec![Box::new(Input::new().key("test").title("Test Input"))])
            .title("Group Title")
            .description("Group Description");

        let header = group.header();
        assert!(header.contains("Group Title"));
        assert!(header.contains("Group Description"));

        let content = group.content();
        assert!(content.contains("Test Input"));

        let footer = group.footer();
        // No errors, so footer should be empty
        assert!(footer.is_empty());
    }

    #[test]
    fn test_form_all_errors() {
        let form = Form::new(vec![Group::new(Vec::new())]);

        // No errors initially
        let errors = form.all_errors();
        assert!(errors.is_empty());
    }

    // Word transformation tests matching Go bubbles/textarea behavior

    #[test]
    fn test_text_transpose_left() {
        let mut text = Text::new().value("hello");
        text.cursor_row = 0;
        text.cursor_col = 5; // At end of "hello"

        text.transpose_left();

        // At end, moves cursor back first, then swaps 'l' and 'o'
        assert_eq!(text.get_string_value(), "helol");
        assert_eq!(text.cursor_col, 5); // Cursor stays at end
    }

    #[test]
    fn test_text_transpose_left_middle() {
        let mut text = Text::new().value("hello");
        text.cursor_row = 0;
        text.cursor_col = 2; // After 'e', before 'l'

        text.transpose_left();

        // Swaps 'e' (pos 1) and 'l' (pos 2)
        assert_eq!(text.get_string_value(), "hlelo");
        assert_eq!(text.cursor_col, 3); // Cursor moves right
    }

    #[test]
    fn test_text_transpose_left_at_beginning() {
        let mut text = Text::new().value("hello");
        text.cursor_row = 0;
        text.cursor_col = 0; // At beginning

        text.transpose_left();

        // No-op when at beginning
        assert_eq!(text.get_string_value(), "hello");
        assert_eq!(text.cursor_col, 0);
    }

    #[test]
    fn test_text_uppercase_right() {
        let mut text = Text::new().value("hello world");
        text.cursor_row = 0;
        text.cursor_col = 0; // At beginning

        text.uppercase_right();

        assert_eq!(text.get_string_value(), "HELLO world");
        assert_eq!(text.cursor_col, 5); // Cursor moves past the word
    }

    #[test]
    fn test_text_uppercase_right_with_spaces() {
        let mut text = Text::new().value("  hello world");
        text.cursor_row = 0;
        text.cursor_col = 0; // Before spaces

        text.uppercase_right();

        // Skips spaces, then uppercases "hello"
        assert_eq!(text.get_string_value(), "  HELLO world");
        assert_eq!(text.cursor_col, 7); // Cursor after "HELLO"
    }

    #[test]
    fn test_text_lowercase_right() {
        let mut text = Text::new().value("HELLO WORLD");
        text.cursor_row = 0;
        text.cursor_col = 0;

        text.lowercase_right();

        assert_eq!(text.get_string_value(), "hello WORLD");
        assert_eq!(text.cursor_col, 5);
    }

    #[test]
    fn test_text_capitalize_right() {
        let mut text = Text::new().value("hello world");
        text.cursor_row = 0;
        text.cursor_col = 0;

        text.capitalize_right();

        // Only first char is uppercased
        assert_eq!(text.get_string_value(), "Hello world");
        assert_eq!(text.cursor_col, 5);
    }

    #[test]
    fn test_text_capitalize_right_already_upper() {
        let mut text = Text::new().value("HELLO WORLD");
        text.cursor_row = 0;
        text.cursor_col = 0;

        text.capitalize_right();

        // First char stays upper, rest unchanged (capitalize doesn't lowercase)
        assert_eq!(text.get_string_value(), "HELLO WORLD");
        assert_eq!(text.cursor_col, 5);
    }

    #[test]
    fn test_text_word_ops_multiline() {
        let mut text = Text::new().value("hello\nworld");
        text.cursor_row = 1;
        text.cursor_col = 0;

        text.uppercase_right();

        // Only operates on current line
        assert_eq!(text.get_string_value(), "hello\nWORLD");
        assert_eq!(text.cursor_row, 1);
        assert_eq!(text.cursor_col, 5);
    }

    #[test]
    fn test_text_transpose_multiline() {
        let mut text = Text::new().value("ab\ncd");
        text.cursor_row = 1;
        text.cursor_col = 2; // At end of "cd"

        text.transpose_left();

        // Swaps 'c' and 'd' on second line
        assert_eq!(text.get_string_value(), "ab\ndc");
    }

    #[test]
    fn test_text_word_ops_unicode() {
        let mut text = Text::new().value("café résumé");
        text.cursor_row = 0;
        text.cursor_col = 0;

        text.uppercase_right();

        assert_eq!(text.get_string_value(), "CAFÉ résumé");
        assert_eq!(text.cursor_col, 4);
    }

    #[test]
    fn test_text_keymap_has_word_ops() {
        let keymap = TextKeyMap::default();

        // Verify the new bindings exist and are enabled
        assert!(keymap.uppercase_word_forward.enabled());
        assert!(keymap.lowercase_word_forward.enabled());
        assert!(keymap.capitalize_word_forward.enabled());
        assert!(keymap.transpose_character_backward.enabled());

        // Verify expected key bindings
        assert!(
            keymap
                .uppercase_word_forward
                .get_keys()
                .contains(&"alt+u".to_string())
        );
        assert!(
            keymap
                .lowercase_word_forward
                .get_keys()
                .contains(&"alt+l".to_string())
        );
        assert!(
            keymap
                .capitalize_word_forward
                .get_keys()
                .contains(&"alt+c".to_string())
        );
        assert!(
            keymap
                .transpose_character_backward
                .get_keys()
                .contains(&"ctrl+t".to_string())
        );
    }

    // -------------------------------------------------------------------------
    // Paste handling tests (bd-3jg2)
    // -------------------------------------------------------------------------

    mod paste_tests {
        use super::*;
        use bubbletea::{KeyMsg, Message};

        /// Helper to create a paste KeyMsg from a string
        fn paste_msg(s: &str) -> Message {
            let key = KeyMsg::from_runes(s.chars().collect()).with_paste();
            Message::new(key)
        }

        /// Helper to create a regular typing KeyMsg from a string
        fn type_msg(s: &str) -> Message {
            let key = KeyMsg::from_runes(s.chars().collect());
            Message::new(key)
        }

        #[test]
        fn test_input_paste_collapses_newlines() {
            let mut input = Input::new().key("query");
            input.focused = true;

            // Paste multi-line content
            let msg = paste_msg("hello\nworld\nfoo");
            input.update(&msg);

            // Newlines should be collapsed to spaces
            assert_eq!(input.get_string_value(), "hello world foo");
        }

        #[test]
        fn test_input_paste_collapses_tabs() {
            let mut input = Input::new().key("query");
            input.focused = true;

            // Paste content with tabs
            let msg = paste_msg("col1\tcol2\tcol3");
            input.update(&msg);

            // Tabs should be collapsed to spaces
            assert_eq!(input.get_string_value(), "col1 col2 col3");
        }

        #[test]
        fn test_input_paste_collapses_multiple_spaces() {
            let mut input = Input::new().key("query");
            input.focused = true;

            // Paste content with multiple consecutive newlines/spaces
            let msg = paste_msg("hello\n\n\nworld");
            input.update(&msg);

            // Multiple consecutive whitespace should collapse to single space
            assert_eq!(input.get_string_value(), "hello world");
        }

        #[test]
        fn test_input_paste_respects_char_limit() {
            let mut input = Input::new().key("query").char_limit(10);
            input.focused = true;

            // Paste more than char_limit
            let msg = paste_msg("hello world this is too long");
            input.update(&msg);

            // Should be truncated at limit
            assert_eq!(input.get_string_value().chars().count(), 10);
            assert_eq!(input.get_string_value(), "hello worl");
        }

        #[test]
        fn test_input_paste_partial_fill() {
            let mut input = Input::new().key("query").char_limit(15);
            input.focused = true;

            // Type some chars first
            let msg = type_msg("hi ");
            input.update(&msg);

            // Paste more - should fill up to limit
            let msg = paste_msg("hello world this is long");
            input.update(&msg);

            assert_eq!(input.get_string_value().chars().count(), 15);
            assert_eq!(input.get_string_value(), "hi hello world ");
        }

        #[test]
        fn test_input_paste_cursor_position() {
            let mut input = Input::new().key("query");
            input.focused = true;

            // Paste some content
            let msg = paste_msg("hello world");
            input.update(&msg);

            // Cursor should be at end
            assert_eq!(input.cursor_pos, 11);
        }

        #[test]
        fn test_input_regular_typing_not_affected() {
            let mut input = Input::new().key("query");
            input.focused = true;

            // Regular typing of newline (shouldn't happen but test defensive behavior)
            let msg = type_msg("hello\nworld");
            input.update(&msg);

            // Regular typing should preserve newlines (they're just chars)
            assert_eq!(input.get_string_value(), "hello\nworld");
        }

        #[test]
        fn test_text_paste_preserves_newlines() {
            let mut text = Text::new().key("bio");
            text.focused = true;

            // Paste multi-line content
            let msg = paste_msg("line 1\nline 2\nline 3");
            text.update(&msg);

            // Newlines should be preserved in Text field
            assert_eq!(text.get_string_value(), "line 1\nline 2\nline 3");
        }

        #[test]
        fn test_text_paste_updates_cursor_row() {
            let mut text = Text::new().key("bio");
            text.focused = true;

            // Paste multi-line content
            let msg = paste_msg("line 1\nline 2\nline 3");
            text.update(&msg);

            // Cursor should be on line 3 (0-indexed = 2)
            assert_eq!(text.cursor_row, 2);
            // Cursor col should be at end of "line 3"
            assert_eq!(text.cursor_col, 6);
        }

        #[test]
        fn test_text_paste_respects_char_limit() {
            let mut text = Text::new().key("bio").char_limit(20);
            text.focused = true;

            // Paste content exceeding limit
            let msg = paste_msg("line 1\nline 2\nline 3 is very long");
            text.update(&msg);

            // Should truncate at 20 chars
            assert_eq!(text.get_string_value().chars().count(), 20);
        }

        #[test]
        fn test_input_paste_unicode() {
            let mut input = Input::new().key("query");
            input.focused = true;

            // Paste unicode content with newlines
            let msg = paste_msg("héllo\nwörld\n日本語");
            input.update(&msg);

            // Should collapse newlines, preserve unicode
            assert_eq!(input.get_string_value(), "héllo wörld 日本語");
        }

        #[test]
        fn test_text_paste_unicode_cursor() {
            let mut text = Text::new().key("bio");
            text.focused = true;

            // Paste unicode content
            let msg = paste_msg("日本語\n한국어");
            text.update(&msg);

            assert_eq!(text.get_string_value(), "日本語\n한국어");
            assert_eq!(text.cursor_row, 1);
            assert_eq!(text.cursor_col, 3); // 3 Korean chars
        }

        #[test]
        fn test_input_paste_empty() {
            let mut input = Input::new().key("query");
            input.focused = true;

            // Paste empty content
            let msg = paste_msg("");
            input.update(&msg);

            assert_eq!(input.get_string_value(), "");
            assert_eq!(input.cursor_pos, 0);
        }

        #[test]
        fn test_input_paste_crlf_handling() {
            let mut input = Input::new().key("query");
            input.focused = true;

            // Paste Windows-style line endings
            let msg = paste_msg("hello\r\nworld");
            input.update(&msg);

            // Both \r and \n should become spaces, then collapse
            assert_eq!(input.get_string_value(), "hello world");
        }

        #[test]
        fn test_input_not_focused_ignores_paste() {
            let mut input = Input::new().key("query");
            input.focused = false;

            let msg = paste_msg("hello world");
            input.update(&msg);

            // Should ignore paste when not focused
            assert_eq!(input.get_string_value(), "");
        }

        #[test]
        fn test_text_not_focused_ignores_paste() {
            let mut text = Text::new().key("bio");
            text.focused = false;

            let msg = paste_msg("hello\nworld");
            text.update(&msg);

            // Should ignore paste when not focused
            assert_eq!(text.get_string_value(), "");
        }

        #[test]
        fn test_input_large_paste() {
            let mut input = Input::new().key("query");
            input.focused = true;

            // Paste a large amount of text (simulating a real paste operation)
            let large_text: String = (0..1000).map(|i| format!("word{} ", i)).collect();
            let msg = paste_msg(&large_text);
            input.update(&msg);

            // Should handle large paste without panic
            assert!(input.get_string_value().chars().count() > 100);
        }

        #[test]
        fn test_text_large_paste() {
            let mut text = Text::new().key("bio");
            text.focused = true;

            // Paste large multi-line text
            let large_text: String = (0..100).map(|i| format!("line {}\n", i)).collect();
            let msg = paste_msg(&large_text);
            text.update(&msg);

            // Should handle large paste without panic
            assert!(text.get_string_value().contains('\n'));
            assert_eq!(text.cursor_row, 100); // 100 newlines = row 100
        }
    }

    #[test]
    fn test_multiselect_filter_cursor_stays_on_item() {
        // Test that cursor stays on the same item when filter narrows results
        let mut multi: MultiSelect<String> = MultiSelect::new().filterable(true).options(vec![
            SelectOption::new("Apple", "apple".to_string()),
            SelectOption::new("Banana", "banana".to_string()),
            SelectOption::new("Cherry", "cherry".to_string()),
            SelectOption::new("Blueberry", "blueberry".to_string()),
        ]);

        multi.focus();

        // Move cursor to "Banana" (index 1)
        let down_msg = Message::new(KeyMsg {
            key_type: KeyType::Down,
            runes: vec![],
            alt: false,
            paste: false,
        });
        multi.update(&down_msg);
        assert_eq!(multi.cursor, 1);

        // Apply filter "b" - should match Banana, Blueberry
        multi.update_filter("b".to_string());

        // Cursor should still be on "Banana" which is now at filtered index 0
        let filtered = multi.filtered_options();
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[multi.cursor].1.key, "Banana");
    }

    #[test]
    fn test_multiselect_filter_cursor_clamps() {
        // Test that cursor clamps when the current item is filtered out
        let mut multi: MultiSelect<String> = MultiSelect::new().filterable(true).options(vec![
            SelectOption::new("Apple", "apple".to_string()),
            SelectOption::new("Banana", "banana".to_string()),
            SelectOption::new("Cherry", "cherry".to_string()),
        ]);

        multi.focus();

        // Move cursor to "Cherry" (index 2)
        let down_msg = Message::new(KeyMsg {
            key_type: KeyType::Down,
            runes: vec![],
            alt: false,
            paste: false,
        });
        multi.update(&down_msg);
        multi.update(&down_msg);
        assert_eq!(multi.cursor, 2);

        // Apply filter "a" - should match Apple, Banana (not Cherry)
        multi.update_filter("a".to_string());

        // Cursor should be clamped to valid range (max index 1)
        let filtered = multi.filtered_options();
        assert_eq!(filtered.len(), 2);
        assert!(multi.cursor < filtered.len());
    }

    #[test]
    fn test_multiselect_filter_then_toggle() {
        // Test that toggling selection works correctly with filtered results
        let mut multi: MultiSelect<String> = MultiSelect::new().filterable(true).options(vec![
            SelectOption::new("Apple", "apple".to_string()),
            SelectOption::new("Banana", "banana".to_string()),
            SelectOption::new("Cherry", "cherry".to_string()),
            SelectOption::new("Blueberry", "blueberry".to_string()),
        ]);

        multi.focus();

        // Apply filter "b" - should match Banana, Blueberry
        multi.update_filter("b".to_string());

        // Move to second item (Blueberry)
        let down_msg = Message::new(KeyMsg {
            key_type: KeyType::Down,
            runes: vec![],
            alt: false,
            paste: false,
        });
        multi.update(&down_msg);

        // Toggle selection
        let toggle_msg = Message::new(KeyMsg {
            key_type: KeyType::Runes,
            runes: vec![' '],
            alt: false,
            paste: false,
        });
        multi.update(&toggle_msg);

        // Verify Blueberry (original index 3) is selected
        let selected = multi.get_selected_values();
        assert_eq!(selected.len(), 1);
        assert!(selected.contains(&&"blueberry".to_string()));

        // Clear filter and verify selection persists
        multi.update_filter(String::new());
        let selected = multi.get_selected_values();
        assert_eq!(selected.len(), 1);
        assert!(selected.contains(&&"blueberry".to_string()));
    }

    #[test]
    fn test_multiselect_filter_navigation_bounds() {
        // Test that navigation respects filtered list bounds
        let mut multi: MultiSelect<String> = MultiSelect::new().filterable(true).options(vec![
            SelectOption::new("Apple", "apple".to_string()),
            SelectOption::new("Banana", "banana".to_string()),
            SelectOption::new("Cherry", "cherry".to_string()),
            SelectOption::new("Date", "date".to_string()),
        ]);

        multi.focus();

        // Apply filter "a" - should match Apple, Banana, Date (3 items)
        multi.update_filter("a".to_string());
        let filtered = multi.filtered_options();
        assert_eq!(filtered.len(), 3);

        // Navigate down past the filtered list size
        let down_msg = Message::new(KeyMsg {
            key_type: KeyType::Down,
            runes: vec![],
            alt: false,
            paste: false,
        });
        multi.update(&down_msg);
        multi.update(&down_msg);
        multi.update(&down_msg); // Try to go past the end
        multi.update(&down_msg);

        // Cursor should be capped at last filtered index
        assert_eq!(multi.cursor, 2); // Max index is 2 (3 items: 0, 1, 2)
    }
}
