//! Cursor component with blinking support.
//!
//! This module provides a cursor component that can be used in text input
//! components. It supports multiple modes including blinking, static, and hidden.
//!
//! # Example
//!
//! ```rust
//! use bubbles::cursor::{Cursor, Mode};
//!
//! let mut cursor = Cursor::new();
//! cursor.set_char("_");
//! cursor.set_mode(Mode::Static);
//!
//! // In your view function:
//! let rendered = cursor.view();
//! ```

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use bubbletea::{Cmd, Message, Model};
use lipgloss::Style;

/// Default blink speed (530ms).
const DEFAULT_BLINK_SPEED: Duration = Duration::from_millis(530);

/// Global ID counter for cursor instances.
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

/// Cursor display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Cursor blinks on and off.
    #[default]
    Blink,
    /// Cursor is always visible.
    Static,
    /// Cursor is hidden.
    Hide,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Blink => write!(f, "blink"),
            Self::Static => write!(f, "static"),
            Self::Hide => write!(f, "hidden"),
        }
    }
}

/// Message to initialize cursor blinking.
#[derive(Debug, Clone, Copy)]
pub struct InitialBlinkMsg;

/// Message signaling that the cursor should toggle its blink state.
#[derive(Debug, Clone, Copy)]
pub struct BlinkMsg {
    /// The cursor ID this message is for.
    pub id: u64,
    /// The blink tag to ensure message ordering.
    pub tag: u64,
}

/// Message sent when a blink operation is canceled.
#[derive(Debug, Clone, Copy)]
pub struct BlinkCanceledMsg;

/// A cursor component for text input.
#[derive(Debug, Clone)]
pub struct Cursor {
    /// The blink speed.
    pub blink_speed: Duration,
    /// Style for the cursor block.
    pub style: Style,
    /// Style for text when cursor is hidden (blinking off).
    pub text_style: Style,

    // Internal state
    char: String,
    id: u64,
    focus: bool,
    blink: bool,
    blink_tag: u64,
    mode: Mode,
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}

impl Cursor {
    /// Creates a new cursor with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            blink_speed: DEFAULT_BLINK_SPEED,
            style: Style::new(),
            text_style: Style::new(),
            char: String::new(),
            id: next_id(),
            focus: false,
            blink: true,
            blink_tag: 0,
            mode: Mode::Blink,
        }
    }

    /// Returns the cursor's unique ID.
    #[must_use]
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns the current cursor mode.
    #[must_use]
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Sets the cursor mode.
    ///
    /// Returns a command to start blinking if the mode is `Blink`.
    pub fn set_mode(&mut self, mode: Mode) -> Option<Cmd> {
        self.mode = mode;
        self.blink = mode == Mode::Hide || !self.focus;

        if mode == Mode::Blink {
            Some(blink_cmd())
        } else {
            None
        }
    }

    /// Sets the character displayed under the cursor.
    pub fn set_char(&mut self, c: &str) {
        self.char = c.to_string();
    }

    /// Returns the character under the cursor.
    #[must_use]
    pub fn char(&self) -> &str {
        &self.char
    }

    /// Returns whether the cursor is currently focused.
    #[must_use]
    pub fn focused(&self) -> bool {
        self.focus
    }

    /// Returns whether the cursor is currently in its "off" blink state.
    #[must_use]
    pub fn is_blinking_off(&self) -> bool {
        self.blink
    }

    /// Focuses the cursor, allowing it to blink if in blink mode.
    ///
    /// Returns a command to start blinking.
    pub fn focus(&mut self) -> Option<Cmd> {
        self.focus = true;
        self.blink = self.mode == Mode::Hide;

        if self.mode == Mode::Blink && self.focus {
            Some(self.blink_tick_cmd())
        } else {
            None
        }
    }

    /// Blurs (unfocuses) the cursor.
    pub fn blur(&mut self) {
        self.focus = false;
        self.blink = true;
    }

    /// Creates a command to trigger the next blink.
    fn blink_tick_cmd(&mut self) -> Cmd {
        if self.mode != Mode::Blink {
            return Cmd::new(|| Message::new(BlinkCanceledMsg));
        }

        self.blink_tag = self.blink_tag.wrapping_add(1);
        let id = self.id;
        let tag = self.blink_tag;
        let speed = self.blink_speed;

        Cmd::new(move || {
            std::thread::sleep(speed);
            Message::new(BlinkMsg { id, tag })
        })
    }

    /// Updates the cursor state based on incoming messages.
    pub fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle initial blink message
        if msg.is::<InitialBlinkMsg>() {
            if self.mode != Mode::Blink || !self.focus {
                return None;
            }
            return Some(self.blink_tick_cmd());
        }

        // Handle focus message
        if msg.is::<bubbletea::FocusMsg>() {
            return self.focus();
        }

        // Handle blur message
        if msg.is::<bubbletea::BlurMsg>() {
            self.blur();
            return None;
        }

        // Handle blink message
        if let Some(blink_msg) = msg.downcast_ref::<BlinkMsg>() {
            // Is this model blink-able?
            if self.mode != Mode::Blink || !self.focus {
                return None;
            }

            // Were we expecting this blink message?
            if blink_msg.id != self.id || blink_msg.tag != self.blink_tag {
                return None;
            }

            // Toggle blink state
            self.blink = !self.blink;
            return Some(self.blink_tick_cmd());
        }

        // Handle blink canceled (no-op)
        if msg.is::<BlinkCanceledMsg>() {
            return None;
        }

        None
    }

    /// Renders the cursor.
    #[must_use]
    pub fn view(&self) -> String {
        if self.blink {
            // Cursor is in "off" state, show normal text
            self.text_style.clone().inline().render(&self.char)
        } else {
            // Cursor is in "on" state, show reversed
            self.style.clone().inline().reverse().render(&self.char)
        }
    }
}

/// Creates a command to initialize cursor blinking.
#[must_use]
pub fn blink_cmd() -> Cmd {
    Cmd::new(|| Message::new(InitialBlinkMsg))
}

impl Model for Cursor {
    /// Initialize the cursor and return a blink command if in blink mode and focused.
    fn init(&self) -> Option<Cmd> {
        if self.mode == Mode::Blink && self.focus {
            Some(blink_cmd())
        } else {
            None
        }
    }

    /// Update the cursor state based on incoming messages.
    ///
    /// Handles:
    /// - `InitialBlinkMsg` - Start blinking if focused and in blink mode
    /// - `FocusMsg` - Focus the cursor and start blinking
    /// - `BlurMsg` - Blur the cursor and stop blinking
    /// - `BlinkMsg` - Toggle blink state and schedule next blink
    fn update(&mut self, msg: Message) -> Option<Cmd> {
        Cursor::update(self, msg)
    }

    /// Render the cursor.
    ///
    /// Returns the cursor character styled appropriately based on blink state.
    fn view(&self) -> String {
        Cursor::view(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_new() {
        let cursor = Cursor::new();
        assert_eq!(cursor.mode(), Mode::Blink);
        assert!(!cursor.focused());
        assert!(cursor.is_blinking_off());
    }

    #[test]
    fn test_cursor_unique_ids() {
        let cursor1 = Cursor::new();
        let cursor2 = Cursor::new();
        assert_ne!(cursor1.id(), cursor2.id());
    }

    #[test]
    fn test_cursor_focus_blur() {
        let mut cursor = Cursor::new();
        assert!(!cursor.focused());

        cursor.focus();
        assert!(cursor.focused());
        assert!(!cursor.is_blinking_off()); // Cursor should be visible when focused

        cursor.blur();
        assert!(!cursor.focused());
        assert!(cursor.is_blinking_off()); // Cursor hidden when blurred
    }

    #[test]
    fn test_cursor_mode_static() {
        let mut cursor = Cursor::new();
        cursor.set_mode(Mode::Static);
        assert_eq!(cursor.mode(), Mode::Static);
    }

    #[test]
    fn test_cursor_mode_hide() {
        let mut cursor = Cursor::new();
        cursor.set_mode(Mode::Hide);
        assert_eq!(cursor.mode(), Mode::Hide);
        assert!(cursor.is_blinking_off());
    }

    #[test]
    fn test_cursor_set_char() {
        let mut cursor = Cursor::new();
        cursor.set_char("_");
        assert_eq!(cursor.char(), "_");
    }

    #[test]
    fn test_cursor_view() {
        let mut cursor = Cursor::new();
        cursor.set_char("a");

        // When blinking off (default), should render with text style
        let view = cursor.view();
        assert!(!view.is_empty());
    }

    fn strip_ansi(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let mut in_escape = false;

        for c in s.chars() {
            if c == '\x1b' {
                in_escape = true;
                continue;
            }
            if in_escape {
                if c == 'm' {
                    in_escape = false;
                }
                continue;
            }
            result.push(c);
        }

        result
    }

    #[test]
    fn test_cursor_view_inline_removes_padding() {
        let mut cursor = Cursor::new();
        cursor.set_char("x");

        cursor.text_style = Style::new().padding(1);
        cursor.blink = true;
        assert_eq!(cursor.view(), "x");

        cursor.style = Style::new().padding(1);
        cursor.blink = false;
        assert_eq!(strip_ansi(&cursor.view()), "x");
    }

    #[test]
    fn test_mode_display() {
        assert_eq!(Mode::Blink.to_string(), "blink");
        assert_eq!(Mode::Static.to_string(), "static");
        assert_eq!(Mode::Hide.to_string(), "hidden");
    }

    #[test]
    fn test_blink_msg_routing() {
        let mut cursor1 = Cursor::new();
        let mut cursor2 = Cursor::new();

        cursor1.focus();
        cursor2.focus();

        // Message for cursor1 shouldn't affect cursor2
        let msg = Message::new(BlinkMsg {
            id: cursor1.id(),
            tag: cursor1.blink_tag,
        });

        let cmd2 = cursor2.update(msg);
        assert!(cmd2.is_none()); // cursor2 should ignore cursor1's message
    }

    // Model trait implementation tests
    #[test]
    fn test_model_init_unfocused() {
        let cursor = Cursor::new();
        // Unfocused cursor should not return init command
        let cmd = Model::init(&cursor);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_model_init_focused_blink() {
        let mut cursor = Cursor::new();
        cursor.focus();
        // Focused cursor in blink mode should return init command
        let cmd = Model::init(&cursor);
        assert!(cmd.is_some());
    }

    #[test]
    fn test_model_init_focused_static() {
        let mut cursor = Cursor::new();
        cursor.set_mode(Mode::Static);
        cursor.focus();
        // Focused cursor in static mode should not return init command
        let cmd = Model::init(&cursor);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_model_view() {
        let mut cursor = Cursor::new();
        cursor.set_char("x");
        // Model::view should return same result as Cursor::view
        let model_view = Model::view(&cursor);
        let cursor_view = Cursor::view(&cursor);
        assert_eq!(model_view, cursor_view);
    }
}
