//! Interaction counter widget demonstrating `bubbletea-macros` derive.
//!
//! This module provides a small, self-contained example of using `#[derive(Model)]`
//! to reduce boilerplate when implementing the `Model` trait.
//!
//! ## Why Use `#[derive(Model)]`?
//!
//! The derive macro is most useful when:
//! - You have a simple model with straightforward state
//! - You want automatic change tracking via `#[state]` attributes
//! - You want to reduce trait implementation boilerplate
//!
//! For complex models (like the main `App`), manual implementation provides
//! more clarity and control over the update/view logic.
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use bubbletea::Model;  // The derive macro and trait share the same name
//!
//! #[derive(Model)]
//! struct Counter {
//!     #[state]  // Changes to this field signal re-render
//!     count: u32,
//! }
//!
//! impl Counter {
//!     fn init(&self) -> Option<Cmd> { None }
//!     fn update(&mut self, msg: Message) -> Option<Cmd> { /* ... */ None }
//!     fn view(&self) -> String { format!("Count: {}", self.count) }
//! }
//! ```

use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model};

use crate::theme::Theme;

// ============================================================================
// Message Types
// ============================================================================

/// Messages for the interaction counter.
#[derive(Debug, Clone)]
pub enum CounterMsg {
    /// Increment the counter.
    Increment,
    /// Decrement the counter (won't go below 0).
    Decrement,
    /// Reset the counter to zero.
    Reset,
}

impl CounterMsg {
    /// Convert to a bubbletea Message.
    #[must_use]
    pub fn into_message(self) -> Message {
        Message::new(self)
    }
}

// ============================================================================
// Interaction Counter Model (using derive macro)
// ============================================================================

/// A simple counter widget demonstrating the `#[derive(Model)]` macro.
///
/// This widget tracks user interactions (key presses) and displays a count.
/// It's intentionally simple to serve as a reference for using the derive macro.
///
/// ## How the Derive Macro Helps
///
/// Without the derive macro, we'd write:
/// ```rust,ignore
/// impl Model for InteractionCounter {
///     fn init(&self) -> Option<Cmd> { self.init() }
///     fn update(&mut self, msg: Message) -> Option<Cmd> { self.update(msg) }
///     fn view(&self) -> String { self.view() }
/// }
/// ```
///
/// With `#[derive(Model)]`, this boilerplate is generated automatically.
/// The `#[state]` attribute enables optimized change detection.
#[derive(Model, Clone)]
pub struct InteractionCounter {
    /// Current count value. Marked with `#[state]` to enable change tracking.
    ///
    /// When this field changes, bubbletea knows a re-render is needed.
    #[state]
    count: u32,

    /// Maximum value the counter can reach.
    ///
    /// Not marked with `#[state]` since changes don't require a visual update
    /// (until count is affected).
    max_value: u32,

    /// Label displayed above the counter.
    #[state]
    label: String,
}

impl InteractionCounter {
    /// Create a new interaction counter.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            count: 0,
            max_value: 999,
            label: label.into(),
        }
    }

    /// Create a counter with a custom maximum value.
    #[must_use]
    pub const fn with_max(mut self, max: u32) -> Self {
        self.max_value = max;
        self
    }

    /// Get the current count.
    #[must_use]
    pub const fn count(&self) -> u32 {
        self.count
    }

    /// Check if counter is at maximum.
    #[must_use]
    pub const fn is_at_max(&self) -> bool {
        self.count >= self.max_value
    }

    /// Check if counter is at zero.
    #[must_use]
    pub const fn is_at_zero(&self) -> bool {
        self.count == 0
    }

    // ------------------------------------------------------------------------
    // Model trait methods (delegated by the derive macro)
    // ------------------------------------------------------------------------

    /// Initialize the counter (no-op for this simple widget).
    #[expect(clippy::unused_self)] // Required by Model trait
    const fn init(&self) -> Option<Cmd> {
        None
    }

    /// Handle messages to update the counter state.
    #[expect(clippy::needless_pass_by_value)] // Required by Model trait signature
    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle CounterMsg
        if let Some(counter_msg) = msg.downcast_ref::<CounterMsg>() {
            match counter_msg {
                CounterMsg::Increment => {
                    if self.count < self.max_value {
                        self.count += 1;
                    }
                }
                CounterMsg::Decrement => {
                    self.count = self.count.saturating_sub(1);
                }
                CounterMsg::Reset => {
                    self.count = 0;
                }
            }
            return None;
        }

        // Handle keyboard shortcuts
        if let Some(key) = msg.downcast_ref::<KeyMsg>()
            && key.key_type == KeyType::Runes
        {
            match key.runes.as_slice() {
                ['+' | '='] => {
                    if self.count < self.max_value {
                        self.count += 1;
                    }
                }
                ['-' | '_'] => {
                    self.count = self.count.saturating_sub(1);
                }
                ['0'] => {
                    self.count = 0;
                }
                _ => {}
            }
        }

        None
    }

    /// Render the counter as a string.
    ///
    /// This uses basic styling. For themed rendering, use [`view_themed`].
    fn view(&self) -> String {
        format!("{}: {}", self.label, self.count)
    }

    // ------------------------------------------------------------------------
    // Additional view methods
    // ------------------------------------------------------------------------

    /// Render the counter with theme styling.
    ///
    /// This is the preferred method when a theme is available.
    #[must_use]
    pub fn view_themed(&self, theme: &Theme) -> String {
        let label_styled = theme.muted_style().render(&self.label);
        let count_styled = if self.is_at_max() {
            theme.warning_style().bold().render(&self.count.to_string())
        } else if self.is_at_zero() {
            theme.muted_style().render(&self.count.to_string())
        } else {
            theme.info_style().bold().render(&self.count.to_string())
        };

        format!("{label_styled}: {count_styled}")
    }

    /// Render a compact version (just the number).
    #[must_use]
    pub fn view_compact(&self, theme: &Theme) -> String {
        let style = if self.is_at_max() {
            theme.warning_style()
        } else {
            theme.info_style()
        };
        style.render(&self.count.to_string())
    }

    /// Render with a progress-bar style indicator.
    #[must_use]
    pub fn view_with_bar(&self, theme: &Theme, width: usize) -> String {
        let label = self.view_themed(theme);
        let ratio = if self.max_value > 0 {
            f64::from(self.count) / f64::from(self.max_value)
        } else {
            0.0
        };

        let bar_width = width.saturating_sub(20);
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let filled = (ratio * bar_width as f64) as usize;
        let empty = bar_width.saturating_sub(filled);

        let fill_char = if self.is_at_max() { "█" } else { "▓" };
        let bar = format!("{}{}", fill_char.repeat(filled), "░".repeat(empty));

        let bar_styled = if self.is_at_max() {
            theme.warning_style().render(&bar)
        } else {
            theme.info_style().render(&bar)
        };

        format!("{label}  [{bar_styled}]")
    }
}

impl Default for InteractionCounter {
    fn default() -> Self {
        Self::new("Interactions")
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_starts_at_zero() {
        let counter = InteractionCounter::new("Test");
        assert_eq!(counter.count(), 0);
        assert!(counter.is_at_zero());
    }

    #[test]
    fn counter_increments() {
        let mut counter = InteractionCounter::new("Test");
        counter.update(CounterMsg::Increment.into_message());
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn counter_decrements() {
        let mut counter = InteractionCounter::new("Test");
        counter.update(CounterMsg::Increment.into_message());
        counter.update(CounterMsg::Increment.into_message());
        counter.update(CounterMsg::Decrement.into_message());
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn counter_doesnt_go_negative() {
        let mut counter = InteractionCounter::new("Test");
        counter.update(CounterMsg::Decrement.into_message());
        assert_eq!(counter.count(), 0);
    }

    #[test]
    fn counter_respects_max() {
        let mut counter = InteractionCounter::new("Test").with_max(3);
        counter.update(CounterMsg::Increment.into_message());
        counter.update(CounterMsg::Increment.into_message());
        counter.update(CounterMsg::Increment.into_message());
        counter.update(CounterMsg::Increment.into_message()); // Should be capped
        assert_eq!(counter.count(), 3);
        assert!(counter.is_at_max());
    }

    #[test]
    fn counter_resets() {
        let mut counter = InteractionCounter::new("Test");
        counter.update(CounterMsg::Increment.into_message());
        counter.update(CounterMsg::Increment.into_message());
        counter.update(CounterMsg::Reset.into_message());
        assert_eq!(counter.count(), 0);
    }

    #[test]
    fn counter_view_contains_label() {
        let counter = InteractionCounter::new("Clicks");
        let view = counter.view();
        assert!(view.contains("Clicks"));
        assert!(view.contains('0'));
    }

    #[test]
    fn counter_keyboard_increment() {
        let mut counter = InteractionCounter::new("Test");
        let key = KeyMsg {
            key_type: KeyType::Runes,
            runes: vec!['+'],
            alt: false,
            paste: false,
        };
        counter.update(Message::new(key));
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn counter_keyboard_decrement() {
        let mut counter = InteractionCounter::new("Test");
        counter.count = 5;
        let key = KeyMsg {
            key_type: KeyType::Runes,
            runes: vec!['-'],
            alt: false,
            paste: false,
        };
        counter.update(Message::new(key));
        assert_eq!(counter.count(), 4);
    }

    #[test]
    fn counter_keyboard_reset() {
        let mut counter = InteractionCounter::new("Test");
        counter.count = 5;
        let key = KeyMsg {
            key_type: KeyType::Runes,
            runes: vec!['0'],
            alt: false,
            paste: false,
        };
        counter.update(Message::new(key));
        assert_eq!(counter.count(), 0);
    }

    #[test]
    fn counter_themed_view() {
        let counter = InteractionCounter::new("Sessions");
        let theme = Theme::dark();
        let view = counter.view_themed(&theme);
        assert!(view.contains("Sessions"));
    }

    #[test]
    fn counter_with_bar_view() {
        let mut counter = InteractionCounter::new("Progress").with_max(10);
        counter.count = 5;
        let theme = Theme::dark();
        let view = counter.view_with_bar(&theme, 40);
        assert!(view.contains("Progress"));
        assert!(view.contains('['));
        assert!(view.contains(']'));
    }

    #[test]
    fn counter_clone_is_independent() {
        let mut original = InteractionCounter::new("Test");
        original.update(CounterMsg::Increment.into_message());

        let clone = original.clone();
        original.update(CounterMsg::Increment.into_message());

        assert_eq!(original.count(), 2);
        assert_eq!(clone.count(), 1);
    }
}
