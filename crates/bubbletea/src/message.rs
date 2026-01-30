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

/// Message for batch command execution.
///
/// This is produced by [`batch`](crate::batch) and handled by the program runtime.
pub struct BatchMsg(pub Vec<super::Cmd>);

/// Message for sequential command execution.
///
/// This is produced by [`sequence`](crate::sequence) and handled by the program runtime.
pub struct SequenceMsg(pub Vec<super::Cmd>);

/// Internal message for printing lines outside the TUI renderer.
///
/// This is produced by [`println`](crate::println) and [`printf`](crate::printf)
/// and handled by the program runtime. Output is only written when not in
/// alternate screen mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PrintLineMsg(pub String);

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

    // =========================================================================
    // Comprehensive Message Tests (bd-1u1s)
    // =========================================================================

    #[test]
    fn test_message_downcast_ref_success() {
        struct TestMsg(i32);

        let msg = Message::new(TestMsg(42));
        // Use downcast_ref to borrow without consuming
        let inner_ref = msg.downcast_ref::<TestMsg>().unwrap();
        assert_eq!(inner_ref.0, 42);

        // Can call downcast_ref multiple times
        let inner_ref2 = msg.downcast_ref::<TestMsg>().unwrap();
        assert_eq!(inner_ref2.0, 42);
    }

    #[test]
    fn test_message_downcast_ref_wrong_type() {
        struct TestMsg1(#[expect(dead_code)] i32);
        struct TestMsg2;

        let msg = Message::new(TestMsg1(42));
        // downcast_ref to wrong type returns None
        assert!(msg.downcast_ref::<TestMsg2>().is_none());
    }

    #[test]
    fn test_message_is_without_consuming() {
        struct TestMsg(i32);

        let msg = Message::new(TestMsg(42));
        // is<T>() doesn't consume the message
        assert!(msg.is::<TestMsg>());
        // Can still use the message after is<T>()
        assert!(msg.is::<TestMsg>());
        // And downcast it
        assert_eq!(msg.downcast::<TestMsg>().unwrap().0, 42);
    }

    #[test]
    fn test_message_debug_format() {
        struct TestMsg;

        let msg = Message::new(TestMsg);
        let debug_str = format!("{:?}", msg);
        // Debug should output something reasonable
        assert!(debug_str.contains("Message"));
    }

    #[test]
    fn test_interrupt_msg() {
        let msg = Message::new(InterruptMsg);
        assert!(msg.is::<InterruptMsg>());
        // Verify it can be downcast
        assert!(msg.downcast::<InterruptMsg>().is_some());
    }

    #[test]
    fn test_suspend_msg() {
        let msg = Message::new(SuspendMsg);
        assert!(msg.is::<SuspendMsg>());
    }

    #[test]
    fn test_resume_msg() {
        let msg = Message::new(ResumeMsg);
        assert!(msg.is::<ResumeMsg>());
    }

    #[test]
    fn test_focus_msg() {
        let msg = Message::new(FocusMsg);
        assert!(msg.is::<FocusMsg>());
    }

    #[test]
    fn test_blur_msg() {
        let msg = Message::new(BlurMsg);
        assert!(msg.is::<BlurMsg>());
    }

    #[test]
    fn test_window_size_msg_in_message() {
        let size = WindowSizeMsg {
            width: 120,
            height: 40,
        };
        let msg = Message::new(size);

        assert!(msg.is::<WindowSizeMsg>());

        let size_ref = msg.downcast_ref::<WindowSizeMsg>().unwrap();
        assert_eq!(size_ref.width, 120);
        assert_eq!(size_ref.height, 40);
    }

    #[test]
    fn test_message_with_string() {
        let msg = Message::new(String::from("hello"));
        assert!(msg.is::<String>());
        assert_eq!(msg.downcast::<String>().unwrap(), "hello");
    }

    #[test]
    fn test_message_with_vec() {
        let msg = Message::new(vec![1, 2, 3]);
        assert!(msg.is::<Vec<i32>>());
        assert_eq!(msg.downcast::<Vec<i32>>().unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn test_message_with_tuple() {
        let msg = Message::new((1i32, "hello", 2.71f64));
        assert!(msg.is::<(i32, &str, f64)>());

        let (a, b, c) = msg.downcast::<(i32, &str, f64)>().unwrap();
        assert_eq!(a, 1);
        assert_eq!(b, "hello");
        assert!((c - 2.71).abs() < f64::EPSILON);
    }

    #[test]
    fn test_message_with_unit() {
        let msg = Message::new(());
        assert!(msg.is::<()>());
        assert!(msg.downcast::<()>().is_some());
    }

    #[test]
    fn test_builtin_msg_equality() {
        // Test PartialEq for built-in message types
        assert_eq!(QuitMsg, QuitMsg);
        assert_eq!(InterruptMsg, InterruptMsg);
        assert_eq!(SuspendMsg, SuspendMsg);
        assert_eq!(ResumeMsg, ResumeMsg);
        assert_eq!(FocusMsg, FocusMsg);
        assert_eq!(BlurMsg, BlurMsg);

        let size1 = WindowSizeMsg {
            width: 80,
            height: 24,
        };
        let size2 = WindowSizeMsg {
            width: 80,
            height: 24,
        };
        let size3 = WindowSizeMsg {
            width: 120,
            height: 40,
        };
        assert_eq!(size1, size2);
        assert_ne!(size1, size3);
    }

    #[test]
    fn test_builtin_msg_clone() {
        // Test Clone/Copy for built-in message types
        let quit = QuitMsg;
        let quit_copy = quit;
        assert_eq!(quit, quit_copy);

        let size = WindowSizeMsg {
            width: 80,
            height: 24,
        };
        let size_copy = size;
        assert_eq!(size, size_copy);
    }

    #[test]
    fn test_builtin_msg_copy() {
        // Test Copy for built-in message types
        let quit = QuitMsg;
        let quit_copy = quit; // Copy, not move
        assert_eq!(quit, quit_copy);

        let size = WindowSizeMsg {
            width: 80,
            height: 24,
        };
        let size_copy = size; // Copy, not move
        assert_eq!(size, size_copy);
    }
}
