//! Message types for the Elm Architecture.
//!
//! Messages are the only way to update the model in bubbletea. All user input,
//! timer events, and custom events are represented as messages.

use std::any::Any;
use std::fmt;

/// A type-erased message container.
///
/// Messages can be any type that is `Send + 'static`. Use [`Message::new`] to create
/// a message and [`Message::downcast`] to retrieve the original type.
///
/// # Example
///
/// ```rust
/// use bubbletea::Message;
///
/// struct MyMsg(i32);
///
/// let msg = Message::new(MyMsg(42));
/// if let Some(my_msg) = msg.downcast::<MyMsg>() {
///     assert_eq!(my_msg.0, 42);
/// }
/// ```
pub struct Message(Box<dyn Any + Send>);

impl Message {
    /// Create a new message from any sendable type.
    pub fn new<M: Any + Send + 'static>(msg: M) -> Self {
        Self(Box::new(msg))
    }

    /// Try to downcast to a specific message type.
    ///
    /// Returns `Some(T)` if the message is of type `T`, otherwise `None`.
    pub fn downcast<M: Any + Send + 'static>(self) -> Option<M> {
        self.0.downcast::<M>().ok().map(|b| *b)
    }

    /// Try to get a reference to the message as a specific type.
    pub fn downcast_ref<M: Any + Send + 'static>(&self) -> Option<&M> {
        self.0.downcast_ref::<M>()
    }

    /// Check if the message is of a specific type.
    pub fn is<M: Any + Send + 'static>(&self) -> bool {
        self.0.is::<M>()
    }
}

impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Message").finish_non_exhaustive()
    }
}

// Built-in message types

/// Message to quit the program gracefully.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuitMsg;

/// Message for Ctrl+C interrupt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InterruptMsg;

/// Message to suspend the program (Ctrl+Z).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SuspendMsg;

/// Message when program resumes from suspension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResumeMsg;

/// Message containing terminal window size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowSizeMsg {
    /// Terminal width in columns.
    pub width: u16,
    /// Terminal height in rows.
    pub height: u16,
}

/// Message when terminal gains focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusMsg;

/// Message when terminal loses focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlurMsg;

/// Internal message to set window title.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SetWindowTitleMsg(pub String);

/// Internal message to request window size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RequestWindowSizeMsg;

/// Internal message for batch command execution.
pub(crate) struct BatchMsg(pub Vec<super::Cmd>);

/// Internal message for sequential command execution.
pub(crate) struct SequenceMsg(pub Vec<super::Cmd>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_downcast() {
        struct TestMsg(i32);

        let msg = Message::new(TestMsg(42));
        assert!(msg.is::<TestMsg>());
        let inner = msg.downcast::<TestMsg>().unwrap();
        assert_eq!(inner.0, 42);
    }

    #[test]
    fn test_message_downcast_wrong_type() {
        struct TestMsg1;
        struct TestMsg2;

        let msg = Message::new(TestMsg1);
        assert!(!msg.is::<TestMsg2>());
        assert!(msg.downcast::<TestMsg2>().is_none());
    }

    #[test]
    fn test_quit_msg() {
        let msg = Message::new(QuitMsg);
        assert!(msg.is::<QuitMsg>());
    }

    #[test]
    fn test_window_size_msg() {
        let msg = WindowSizeMsg {
            width: 80,
            height: 24,
        };
        assert_eq!(msg.width, 80);
        assert_eq!(msg.height, 24);
    }
}
