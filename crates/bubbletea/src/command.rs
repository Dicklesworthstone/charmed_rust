//! Commands for side effects.
//!
//! Commands represent IO operations that produce messages. They are the only
//! way to perform side effects in the Elm Architecture.

use std::time::{Duration, Instant};

use crate::message::{
    BatchMsg, Message, QuitMsg, RequestWindowSizeMsg, SequenceMsg, SetWindowTitleMsg,
};

/// A command that produces a message when executed.
///
/// Commands are lazy - they don't execute until the program runs them.
/// This allows for pure update functions that return commands without
/// side effects.
///
/// # Example
///
/// ```rust
/// use bubbletea::{Cmd, Message};
/// use std::time::Duration;
///
/// // A command that produces a message after a delay
/// fn delayed_message() -> Cmd {
///     Cmd::new(|| {
///         std::thread::sleep(Duration::from_secs(1));
///         Message::new("done")
///     })
/// }
/// ```
pub struct Cmd(Box<dyn FnOnce() -> Option<Message> + Send + 'static>);

impl Cmd {
    /// Create a new command from a closure.
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> Message + Send + 'static,
    {
        Self(Box::new(move || Some(f())))
    }

    /// Create a command that may not produce a message.
    pub fn new_optional<F>(f: F) -> Self
    where
        F: FnOnce() -> Option<Message> + Send + 'static,
    {
        Self(Box::new(f))
    }

    /// Create an empty command that does nothing.
    pub fn none() -> Option<Self> {
        None
    }

    /// Execute the command and return the resulting message.
    pub fn execute(self) -> Option<Message> {
        (self.0)()
    }
}

/// Batch multiple commands to run concurrently.
///
/// Commands in a batch run in parallel with no ordering guarantees.
/// Use this to return multiple commands from an update function.
///
/// # Example
///
/// ```rust
/// use bubbletea::{Cmd, Message, batch};
///
/// let cmd = batch(vec![
///     Some(Cmd::new(|| Message::new("first"))),
///     Some(Cmd::new(|| Message::new("second"))),
/// ]);
/// ```
pub fn batch(cmds: Vec<Option<Cmd>>) -> Option<Cmd> {
    let valid_cmds: Vec<Cmd> = cmds.into_iter().flatten().collect();

    match valid_cmds.len() {
        0 => None,
        1 => valid_cmds.into_iter().next(),
        _ => Some(Cmd::new_optional(move || {
            Some(Message::new(BatchMsg(valid_cmds)))
        })),
    }
}

/// Sequence commands to run one at a time, in order.
///
/// Unlike batch, sequenced commands run one after another.
/// Use this when the order of execution matters.
///
/// # Example
///
/// ```rust
/// use bubbletea::{Cmd, Message, sequence};
///
/// let cmd = sequence(vec![
///     Some(Cmd::new(|| Message::new("first"))),
///     Some(Cmd::new(|| Message::new("second"))),
/// ]);
/// ```
pub fn sequence(cmds: Vec<Option<Cmd>>) -> Option<Cmd> {
    let valid_cmds: Vec<Cmd> = cmds.into_iter().flatten().collect();

    match valid_cmds.len() {
        0 => None,
        1 => valid_cmds.into_iter().next(),
        _ => Some(Cmd::new_optional(move || {
            Some(Message::new(SequenceMsg(valid_cmds)))
        })),
    }
}

/// Command that signals the program to quit.
pub fn quit() -> Cmd {
    Cmd::new(|| Message::new(QuitMsg))
}

/// Command that ticks after a duration.
///
/// The tick runs for the full duration from when it's invoked.
/// To create periodic ticks, return another tick command from
/// your update function when handling the tick message.
///
/// # Example
///
/// ```rust,ignore
/// use bubbletea::{Cmd, tick, Message};
/// use std::time::{Duration, Instant};
///
/// struct TickMsg(Instant);
///
/// fn do_tick() -> Cmd {
///     tick(Duration::from_secs(1), |t| Message::new(TickMsg(t)))
/// }
/// ```
pub fn tick<F>(duration: Duration, f: F) -> Cmd
where
    F: FnOnce(Instant) -> Message + Send + 'static,
{
    Cmd::new(move || {
        std::thread::sleep(duration);
        f(Instant::now())
    })
}

/// Command that ticks in sync with the system clock.
///
/// Unlike `tick`, this aligns with the system clock. For example,
/// if you tick every second and the clock is at 12:34:20.5, the
/// next tick will happen at 12:34:21.0 (in 0.5 seconds).
///
/// # Example
///
/// ```rust,ignore
/// use bubbletea::{Cmd, every, Message};
/// use std::time::{Duration, Instant};
///
/// struct TickMsg(Instant);
///
/// fn tick_every_second() -> Cmd {
///     every(Duration::from_secs(1), |t| Message::new(TickMsg(t)))
/// }
/// ```
pub fn every<F>(duration: Duration, f: F) -> Cmd
where
    F: FnOnce(Instant) -> Message + Send + 'static,
{
    Cmd::new(move || {
        let now = Instant::now();
        // Calculate time until next tick aligned with system clock
        let now_nanos = now.elapsed().as_nanos() as u64;
        let duration_nanos = duration.as_nanos() as u64;
        let next_tick_nanos = ((now_nanos / duration_nanos) + 1) * duration_nanos;
        let sleep_nanos = next_tick_nanos - now_nanos;
        std::thread::sleep(Duration::from_nanos(sleep_nanos));
        f(Instant::now())
    })
}

/// Command to set the terminal window title.
pub fn set_window_title(title: impl Into<String>) -> Cmd {
    let title = title.into();
    Cmd::new(move || Message::new(SetWindowTitleMsg(title)))
}

/// Command to query the current window size.
///
/// The result is delivered as a `WindowSizeMsg`.
pub fn window_size() -> Cmd {
    Cmd::new(|| Message::new(RequestWindowSizeMsg))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd_new() {
        let cmd = Cmd::new(|| Message::new(42i32));
        let msg = cmd.execute().unwrap();
        assert_eq!(msg.downcast::<i32>().unwrap(), 42);
    }

    #[test]
    fn test_cmd_none() {
        assert!(Cmd::none().is_none());
    }

    #[test]
    fn test_batch_empty() {
        let cmd = batch(vec![]);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_batch_single() {
        let cmd = batch(vec![Some(Cmd::new(|| Message::new(42i32)))]);
        assert!(cmd.is_some());
    }

    #[test]
    fn test_sequence_empty() {
        let cmd = sequence(vec![]);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_quit() {
        let cmd = quit();
        let msg = cmd.execute().unwrap();
        assert!(msg.is::<QuitMsg>());
    }

    #[test]
    fn test_set_window_title() {
        let cmd = set_window_title("My App");
        let msg = cmd.execute().unwrap();
        assert!(msg.is::<SetWindowTitleMsg>());
    }
}
