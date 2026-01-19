//! Commands for side effects.
//!
//! Commands represent IO operations that produce messages. They are the only
//! way to perform side effects in the Elm Architecture.
//!
//! # Sync vs Async Commands
//!
//! The crate supports both synchronous and asynchronous commands:
//!
//! - `Cmd` - Synchronous commands that run on a blocking thread pool
//! - `AsyncCmd` - Asynchronous commands that run on the tokio runtime (requires `async` feature)
//!
//! Both types are automatically handled by the program's command executor.

use std::time::{Duration, Instant};

use crate::message::{
    BatchMsg, Message, QuitMsg, RequestWindowSizeMsg, SequenceMsg, SetWindowTitleMsg,
};

#[cfg(feature = "async")]
use std::future::Future;
#[cfg(feature = "async")]
use std::pin::Pin;

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

// =============================================================================
// Async Commands (requires "async" feature)
// =============================================================================

/// An asynchronous command that produces a message when executed.
///
/// Unlike `Cmd`, async commands can await I/O operations without blocking
/// a thread. They run on the tokio runtime's async task pool.
///
/// # Example
///
/// ```rust,ignore
/// use bubbletea::{AsyncCmd, Message};
///
/// fn fetch_data() -> AsyncCmd {
///     AsyncCmd::new(|| async {
///         let data = reqwest::get("https://api.example.com/data")
///             .await
///             .unwrap()
///             .text()
///             .await
///             .unwrap();
///         Message::new(data)
///     })
/// }
/// ```
#[cfg(feature = "async")]
#[allow(clippy::type_complexity)]
pub struct AsyncCmd(
    Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = Option<Message>> + Send>> + Send + 'static>,
);

#[cfg(feature = "async")]
impl AsyncCmd {
    /// Create a new async command from an async closure.
    pub fn new<F, Fut>(f: F) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Message> + Send + 'static,
    {
        Self(Box::new(move || Box::pin(async move { Some(f().await) })))
    }

    /// Create an async command that may not produce a message.
    pub fn new_optional<F, Fut>(f: F) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Option<Message>> + Send + 'static,
    {
        Self(Box::new(move || Box::pin(f())))
    }

    /// Create an empty async command that does nothing.
    pub fn none() -> Option<Self> {
        None
    }

    /// Execute the async command and return the resulting message.
    pub async fn execute(self) -> Option<Message> {
        (self.0)().await
    }
}

/// Internal enum for handling both sync and async commands.
#[cfg(feature = "async")]
pub(crate) enum CommandKind {
    /// Synchronous command (runs on blocking thread pool)
    Sync(Cmd),
    /// Asynchronous command (runs on async task pool)
    Async(AsyncCmd),
}

#[cfg(feature = "async")]
impl CommandKind {
    /// Execute the command, handling both sync and async variants.
    pub async fn execute(self) -> Option<Message> {
        match self {
            CommandKind::Sync(cmd) => {
                // Run blocking code on tokio's blocking thread pool
                tokio::task::spawn_blocking(move || cmd.execute())
                    .await
                    .ok()
                    .flatten()
            }
            CommandKind::Async(cmd) => cmd.execute().await,
        }
    }
}

#[cfg(feature = "async")]
impl From<Cmd> for CommandKind {
    fn from(cmd: Cmd) -> Self {
        CommandKind::Sync(cmd)
    }
}

#[cfg(feature = "async")]
impl From<AsyncCmd> for CommandKind {
    fn from(cmd: AsyncCmd) -> Self {
        CommandKind::Async(cmd)
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

// =============================================================================
// Async Tick Commands (requires "async" feature)
// =============================================================================

/// Async command that ticks after a duration using tokio::time.
///
/// Unlike the sync `tick`, this doesn't block a thread while waiting.
/// Use this when running on an async runtime.
///
/// # Example
///
/// ```rust,ignore
/// use bubbletea::{tick_async, AsyncCmd, Message};
/// use std::time::{Duration, Instant};
///
/// struct TickMsg(Instant);
///
/// fn do_tick() -> AsyncCmd {
///     tick_async(Duration::from_secs(1), |t| Message::new(TickMsg(t)))
/// }
/// ```
#[cfg(feature = "async")]
pub fn tick_async<F>(duration: Duration, f: F) -> AsyncCmd
where
    F: FnOnce(Instant) -> Message + Send + 'static,
{
    AsyncCmd::new(move || async move {
        tokio::time::sleep(duration).await;
        f(Instant::now())
    })
}

/// Async command that ticks in sync with the system clock using tokio::time.
///
/// Unlike the sync `every`, this doesn't block a thread while waiting.
/// Use this when running on an async runtime.
///
/// # Example
///
/// ```rust,ignore
/// use bubbletea::{every_async, AsyncCmd, Message};
/// use std::time::{Duration, Instant};
///
/// struct TickMsg(Instant);
///
/// fn tick_every_second() -> AsyncCmd {
///     every_async(Duration::from_secs(1), |t| Message::new(TickMsg(t)))
/// }
/// ```
#[cfg(feature = "async")]
pub fn every_async<F>(duration: Duration, f: F) -> AsyncCmd
where
    F: FnOnce(Instant) -> Message + Send + 'static,
{
    AsyncCmd::new(move || async move {
        let now = Instant::now();
        // Calculate time until next tick aligned with system clock
        let now_nanos = now.elapsed().as_nanos() as u64;
        let duration_nanos = duration.as_nanos() as u64;
        let next_tick_nanos = ((now_nanos / duration_nanos) + 1) * duration_nanos;
        let sleep_nanos = next_tick_nanos - now_nanos;
        tokio::time::sleep(Duration::from_nanos(sleep_nanos)).await;
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

    // =============================================================================
    // Async Command Tests (requires "async" feature)
    // =============================================================================

    #[cfg(feature = "async")]
    mod async_tests {
        use super::*;

        #[tokio::test]
        async fn test_async_cmd_new() {
            let cmd = AsyncCmd::new(|| async { Message::new(42i32) });
            let msg = cmd.execute().await.unwrap();
            assert_eq!(msg.downcast::<i32>().unwrap(), 42);
        }

        #[tokio::test]
        async fn test_async_cmd_new_optional_some() {
            let cmd = AsyncCmd::new_optional(|| async { Some(Message::new("hello")) });
            let msg = cmd.execute().await.unwrap();
            assert_eq!(msg.downcast::<&str>().unwrap(), "hello");
        }

        #[tokio::test]
        async fn test_async_cmd_new_optional_none() {
            let cmd = AsyncCmd::new_optional(|| async { None });
            assert!(cmd.execute().await.is_none());
        }

        #[tokio::test]
        async fn test_async_cmd_none() {
            assert!(AsyncCmd::none().is_none());
        }

        #[tokio::test]
        async fn test_command_kind_sync() {
            let cmd = Cmd::new(|| Message::new(100i32));
            let kind: CommandKind = cmd.into();
            let msg = kind.execute().await.unwrap();
            assert_eq!(msg.downcast::<i32>().unwrap(), 100);
        }

        #[tokio::test]
        async fn test_command_kind_async() {
            let cmd = AsyncCmd::new(|| async { Message::new(200i32) });
            let kind: CommandKind = cmd.into();
            let msg = kind.execute().await.unwrap();
            assert_eq!(msg.downcast::<i32>().unwrap(), 200);
        }

        #[tokio::test]
        async fn test_tick_async_produces_message() {
            struct TickMsg(#[allow(dead_code)] Instant);

            let cmd = tick_async(Duration::from_millis(1), |t| Message::new(TickMsg(t)));
            let msg = cmd.execute().await.unwrap();
            assert!(msg.is::<TickMsg>());
        }
    }
}
