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

use std::time::{Duration, Instant, SystemTime};

use crate::message::{
    BatchMsg, Message, PrintLineMsg, QuitMsg, RequestWindowSizeMsg, SequenceMsg, SetWindowTitleMsg,
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

    /// Create a command that performs blocking I/O.
    ///
    /// This is semantically equivalent to `Cmd::new()` but makes the blocking
    /// intent explicit. When the `async` feature is enabled, blocking commands
    /// are automatically run on tokio's blocking thread pool via `spawn_blocking`.
    ///
    /// Use this for operations like:
    /// - File I/O (`std::fs::read`, `std::fs::write`)
    /// - Network operations with blocking APIs
    /// - CPU-intensive computations
    /// - Thread sleep operations
    ///
    /// # Example
    ///
    /// ```rust
    /// use bubbletea::{Cmd, Message};
    ///
    /// fn read_config() -> Cmd {
    ///     Cmd::blocking(|| {
    ///         let content = std::fs::read_to_string("config.toml").unwrap();
    ///         Message::new(content)
    ///     })
    /// }
    /// ```
    pub fn blocking<F>(f: F) -> Self
    where
        F: FnOnce() -> Message + Send + 'static,
    {
        // Blocking commands are handled the same as regular commands.
        // When the async feature is enabled, CommandKind::execute() runs
        // sync commands via tokio::task::spawn_blocking automatically.
        Self::new(f)
    }

    /// Create a command that performs a blocking operation returning a Result.
    ///
    /// Converts `Result<T, E>` into a message, wrapping both success and error
    /// cases. This is convenient for I/O operations that can fail.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bubbletea::{Cmd, Message};
    /// use std::io;
    ///
    /// struct FileContent(String);
    /// struct FileError(io::Error);
    ///
    /// fn read_file(path: &'static str) -> Cmd {
    ///     Cmd::blocking_result(
    ///         move || std::fs::read_to_string(path),
    ///         |content| Message::new(FileContent(content)),
    ///         |err| Message::new(FileError(err)),
    ///     )
    /// }
    /// ```
    pub fn blocking_result<F, T, E, S, Err>(f: F, on_success: S, on_error: Err) -> Self
    where
        F: FnOnce() -> Result<T, E> + Send + 'static,
        S: FnOnce(T) -> Message + Send + 'static,
        Err: FnOnce(E) -> Message + Send + 'static,
    {
        Self::new(move || match f() {
            Ok(value) => on_success(value),
            Err(err) => on_error(err),
        })
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
        let duration_nanos = duration.as_nanos() as u64;
        if duration_nanos == 0 {
            // Zero duration means tick immediately
            return f(Instant::now());
        }

        // Get current wall clock time as nanos since Unix epoch
        let now_nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        // Calculate time until next tick aligned with system clock
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
        let duration_nanos = duration.as_nanos() as u64;
        if duration_nanos == 0 {
            // Zero duration means tick immediately
            return f(Instant::now());
        }

        // Get current wall clock time as nanos since Unix epoch
        let now_nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        // Calculate time until next tick aligned with system clock
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

/// Print a line above the program's TUI output.
///
/// This output is unmanaged by the program and will persist across renders.
/// Unlike `std::println!`, the message is printed on its own line (similar to `log::info!`).
///
/// **Note:** If the alternate screen is active, no output will be printed.
/// This is because alternate screen mode uses a separate buffer that doesn't
/// support this kind of unmanaged output.
///
/// # Example
///
/// ```rust,ignore
/// use bubbletea::{Model, Message, Cmd, println};
///
/// impl Model for MyModel {
///     fn update(&mut self, msg: Message) -> Option<Cmd> {
///         if msg.is::<DownloadComplete>() {
///             return Some(println("Download finished!"));
///         }
///         None
///     }
/// }
/// ```
pub fn println(msg: impl Into<String>) -> Cmd {
    let msg = msg.into();
    Cmd::new(move || Message::new(PrintLineMsg(msg)))
}

/// Print a formatted line above the program's TUI output.
///
/// This works like [`std::format!`] but prints above the TUI.
/// Output is unmanaged by the program and will persist across renders.
/// Unlike `std::print!`, the message is printed on its own line (similar to `log::info!`).
///
/// **Note:** If the alternate screen is active, no output will be printed.
/// This is because alternate screen mode uses a separate buffer that doesn't
/// support this kind of unmanaged output.
///
/// # Example
///
/// ```rust,ignore
/// use bubbletea::{Model, Message, Cmd, printf};
///
/// impl Model for MyModel {
///     fn update(&mut self, msg: Message) -> Option<Cmd> {
///         if let Some(size) = msg.downcast_ref::<WindowSizeMsg>() {
///             return Some(printf(format!("Window size: {}x{}", size.width, size.height)));
///         }
///         None
///     }
/// }
/// ```
///
/// For more complex formatting, use [`std::format!`] with [`println`]:
///
/// ```rust,ignore
/// println(format!("Processing {} of {} items", current, total))
/// ```
pub fn printf(msg: impl Into<String>) -> Cmd {
    println(msg)
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

    // =========================================================================
    // Batch and Sequence Comprehensive Tests (bd-1u1s)
    // =========================================================================

    #[test]
    fn test_batch_multiple_commands() {
        // Batch with multiple commands should return Some
        let cmd = batch(vec![
            Some(Cmd::new(|| Message::new(1i32))),
            Some(Cmd::new(|| Message::new(2i32))),
            Some(Cmd::new(|| Message::new(3i32))),
        ]);
        assert!(cmd.is_some());

        // Execute returns a BatchMsg containing all commands
        let msg = cmd.unwrap().execute().unwrap();
        assert!(msg.is::<BatchMsg>());
    }

    #[test]
    fn test_batch_filters_none_values() {
        // Batch should filter out None values and still work
        let cmd = batch(vec![
            Some(Cmd::new(|| Message::new(1i32))),
            None, // This should be filtered
            Some(Cmd::new(|| Message::new(2i32))),
            None, // This should be filtered
        ]);
        assert!(cmd.is_some());

        // Verify it produces a BatchMsg
        let msg = cmd.unwrap().execute().unwrap();
        let batch_msg = msg.downcast::<BatchMsg>().unwrap();
        // Should have 2 commands (the two Some values)
        assert_eq!(batch_msg.0.len(), 2);
    }

    #[test]
    fn test_batch_all_none_returns_none() {
        // Batch with all None should return None
        let cmd = batch(vec![None, None, None]);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_batch_mixed_with_single_some() {
        // Batch with only one Some among Nones
        let cmd = batch(vec![None, Some(Cmd::new(|| Message::new(42i32))), None]);
        assert!(cmd.is_some());
    }

    #[test]
    fn test_sequence_single() {
        // Sequence with a single command returns the command directly (optimization)
        let cmd = sequence(vec![Some(Cmd::new(|| Message::new(42i32)))]);
        assert!(cmd.is_some());

        // Single command executes directly, not wrapped in SequenceMsg
        let msg = cmd.unwrap().execute().unwrap();
        assert!(msg.is::<i32>());
        assert_eq!(msg.downcast::<i32>().unwrap(), 42);
    }

    #[test]
    fn test_sequence_multiple_commands() {
        // Sequence with multiple commands
        let cmd = sequence(vec![
            Some(Cmd::new(|| Message::new(1i32))),
            Some(Cmd::new(|| Message::new(2i32))),
            Some(Cmd::new(|| Message::new(3i32))),
        ]);
        assert!(cmd.is_some());

        // Verify it produces a SequenceMsg
        let msg = cmd.unwrap().execute().unwrap();
        assert!(msg.is::<SequenceMsg>());
    }

    #[test]
    fn test_sequence_filters_none_values() {
        // Sequence should filter out None values
        let cmd = sequence(vec![
            Some(Cmd::new(|| Message::new(1i32))),
            None,
            Some(Cmd::new(|| Message::new(2i32))),
        ]);
        assert!(cmd.is_some());

        // Verify the SequenceMsg has correct number of commands
        let msg = cmd.unwrap().execute().unwrap();
        let seq_msg = msg.downcast::<SequenceMsg>().unwrap();
        assert_eq!(seq_msg.0.len(), 2);
    }

    #[test]
    fn test_sequence_all_none_returns_none() {
        // Sequence with all None should return None
        let cmd = sequence(vec![None, None]);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_cmd_new_with_closure() {
        // Test Cmd::new with various closure types
        let cmd = Cmd::new(|| Message::new("hello"));
        let msg = cmd.execute().unwrap();
        assert!(msg.is::<&str>());
        assert_eq!(msg.downcast::<&str>().unwrap(), "hello");
    }

    #[test]
    fn test_cmd_new_with_captured_value() {
        // Test Cmd::new with closure capturing values
        let value = 42i32;
        let cmd = Cmd::new(move || Message::new(value));
        let msg = cmd.execute().unwrap();
        assert_eq!(msg.downcast::<i32>().unwrap(), 42);
    }

    #[test]
    fn test_cmd_new_optional_some() {
        // Test Cmd::new_optional returning Some
        let cmd = Cmd::new_optional(|| Some(Message::new(42i32)));
        let msg = cmd.execute();
        assert!(msg.is_some());
        assert_eq!(msg.unwrap().downcast::<i32>().unwrap(), 42);
    }

    #[test]
    fn test_cmd_new_optional_none() {
        // Test Cmd::new_optional returning None
        let cmd = Cmd::new_optional(|| None);
        let msg = cmd.execute();
        assert!(msg.is_none());
    }

    #[test]
    fn test_blocking_executes() {
        // Test Cmd::blocking actually executes
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        let executed = Arc::new(AtomicBool::new(false));
        let executed_clone = Arc::clone(&executed);

        let cmd = Cmd::blocking(move || {
            executed_clone.store(true, Ordering::SeqCst);
            Message::new(())
        });

        let msg = cmd.execute();
        assert!(msg.is_some());
        assert!(executed.load(Ordering::SeqCst));
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

    #[test]
    fn test_println() {
        let cmd = println("Hello, World!");
        let msg = cmd.execute().unwrap();
        assert!(msg.is::<PrintLineMsg>());
        let print_msg = msg.downcast::<PrintLineMsg>().unwrap();
        assert_eq!(print_msg.0, "Hello, World!");
    }

    #[test]
    fn test_println_from_string() {
        let cmd = println(String::from("From String"));
        let msg = cmd.execute().unwrap();
        let print_msg = msg.downcast::<PrintLineMsg>().unwrap();
        assert_eq!(print_msg.0, "From String");
    }

    #[test]
    fn test_printf() {
        let cmd = printf(format!("Count: {}", 42));
        let msg = cmd.execute().unwrap();
        assert!(msg.is::<PrintLineMsg>());
        let print_msg = msg.downcast::<PrintLineMsg>().unwrap();
        assert_eq!(print_msg.0, "Count: 42");
    }

    #[test]
    fn test_println_multiline() {
        let cmd = println("Line 1\nLine 2\nLine 3");
        let msg = cmd.execute().unwrap();
        let print_msg = msg.downcast::<PrintLineMsg>().unwrap();
        assert_eq!(print_msg.0, "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_blocking() {
        let cmd = Cmd::blocking(|| Message::new("blocked"));
        let msg = cmd.execute().unwrap();
        assert_eq!(msg.downcast::<&str>().unwrap(), "blocked");
    }

    #[test]
    fn test_blocking_result_success() {
        struct FileContent(String);

        let cmd = Cmd::blocking_result(
            || Ok::<_, std::io::Error>("file content".to_string()),
            |content| Message::new(FileContent(content)),
            |_err| Message::new("error"),
        );
        let msg = cmd.execute().unwrap();
        assert!(msg.is::<FileContent>());
        let content = msg.downcast::<FileContent>().unwrap();
        assert_eq!(content.0, "file content");
    }

    #[test]
    fn test_blocking_result_error() {
        #[allow(dead_code)] // Field unused; we only check is::<FileError>()
        struct FileError(std::io::Error);

        let cmd = Cmd::blocking_result(
            || {
                Err::<String, _>(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "not found",
                ))
            },
            |_content| Message::new("success"),
            |err| Message::new(FileError(err)),
        );
        let msg = cmd.execute().unwrap();
        assert!(msg.is::<FileError>());
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

        #[tokio::test]
        async fn test_blocking_via_spawn_blocking() {
            // Verify that Cmd::blocking runs via spawn_blocking in async context
            let cmd = Cmd::blocking(|| {
                // Simulate a blocking operation
                std::thread::sleep(Duration::from_millis(1));
                Message::new("blocked_async")
            });
            let kind: CommandKind = cmd.into();
            let msg = kind.execute().await.unwrap();
            assert_eq!(msg.downcast::<&str>().unwrap(), "blocked_async");
        }

        #[tokio::test]
        async fn test_blocking_result_via_spawn_blocking() {
            #[allow(dead_code)]
            struct FileContent(String);

            let cmd = Cmd::blocking_result(
                || {
                    // Simulate blocking I/O
                    std::thread::sleep(Duration::from_millis(1));
                    Ok::<_, std::io::Error>("async file content".to_string())
                },
                |content| Message::new(FileContent(content)),
                |_err| Message::new("error"),
            );
            let kind: CommandKind = cmd.into();
            let msg = kind.execute().await.unwrap();
            assert!(msg.is::<FileContent>());
        }

        // =========================================================================
        // Error Handling Tests
        // =========================================================================

        #[tokio::test]
        async fn test_blocking_result_error_in_async_context() {
            #[allow(dead_code)]
            struct ErrorResult(String);

            let cmd = Cmd::blocking_result(
                || {
                    Err::<String, _>(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "not found",
                    ))
                },
                |_content| Message::new("success"),
                |err| Message::new(ErrorResult(err.to_string())),
            );
            let kind: CommandKind = cmd.into();
            let msg = kind.execute().await.unwrap();
            assert!(msg.is::<ErrorResult>());
        }

        #[tokio::test]
        async fn test_async_cmd_with_io_error() {
            #[allow(dead_code)]
            struct IoError(String);

            let cmd = AsyncCmd::new(|| async {
                // Simulate an async operation that fails
                let result: Result<String, std::io::Error> = Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "file not found",
                ));
                match result {
                    Ok(data) => Message::new(data),
                    Err(e) => Message::new(IoError(e.to_string())),
                }
            });
            let msg = cmd.execute().await.unwrap();
            assert!(msg.is::<IoError>());
        }

        #[tokio::test]
        async fn test_async_cmd_optional_returns_none_on_error() {
            let cmd = AsyncCmd::new_optional(|| async {
                // Simulate operation that fails silently
                let result: Result<i32, &str> = Err("failed");
                result.ok().map(Message::new)
            });
            assert!(cmd.execute().await.is_none());
        }

        // =========================================================================
        // Timeout Tests
        // =========================================================================

        #[tokio::test]
        async fn test_tick_async_respects_duration() {
            struct TimerFired;

            let start = std::time::Instant::now();
            let cmd = tick_async(Duration::from_millis(50), |_| Message::new(TimerFired));
            let msg = cmd.execute().await.unwrap();
            let elapsed = start.elapsed();

            assert!(msg.is::<TimerFired>());
            assert!(elapsed >= Duration::from_millis(50));
            assert!(elapsed < Duration::from_millis(150)); // Allow some slack
        }

        #[tokio::test]
        async fn test_async_cmd_with_timeout() {
            use tokio::time::timeout;

            struct SlowResult;

            let cmd = AsyncCmd::new(|| async {
                tokio::time::sleep(Duration::from_millis(10)).await;
                Message::new(SlowResult)
            });

            // Should complete within timeout
            let result = timeout(Duration::from_millis(100), cmd.execute()).await;
            assert!(result.is_ok());
            assert!(result.unwrap().unwrap().is::<SlowResult>());
        }

        #[tokio::test]
        async fn test_async_cmd_timeout_expires() {
            use tokio::time::timeout;

            let cmd = AsyncCmd::new(|| async {
                tokio::time::sleep(Duration::from_secs(10)).await;
                Message::new("never")
            });

            // Should timeout
            let result = timeout(Duration::from_millis(10), cmd.execute()).await;
            assert!(result.is_err()); // Timeout elapsed
        }

        // =========================================================================
        // Concurrency Tests
        // =========================================================================

        #[tokio::test]
        async fn test_concurrent_async_commands() {
            use std::sync::Arc;
            use std::sync::atomic::{AtomicUsize, Ordering};

            #[allow(dead_code)]
            struct CounterResult(usize);

            let counter = Arc::new(AtomicUsize::new(0));
            let mut handles = vec![];

            // Spawn 10 concurrent async commands
            for i in 0..10 {
                let counter = Arc::clone(&counter);
                let cmd = AsyncCmd::new(move || async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    tokio::time::sleep(Duration::from_millis(1)).await;
                    Message::new(CounterResult(i))
                });
                handles.push(tokio::spawn(async move { cmd.execute().await }));
            }

            // Wait for all
            for handle in handles {
                let msg = handle.await.unwrap().unwrap();
                assert!(msg.is::<CounterResult>());
            }

            // All 10 should have run
            assert_eq!(counter.load(Ordering::SeqCst), 10);
        }

        #[tokio::test]
        async fn test_concurrent_command_kind_mixed() {
            use std::sync::Arc;
            use std::sync::atomic::{AtomicUsize, Ordering};

            let counter = Arc::new(AtomicUsize::new(0));
            let mut handles = vec![];

            // Mix of sync and async commands
            for i in 0..6 {
                let counter = Arc::clone(&counter);
                let kind: CommandKind = if i % 2 == 0 {
                    // Sync command (runs via spawn_blocking)
                    let counter = Arc::clone(&counter);
                    Cmd::new(move || {
                        counter.fetch_add(1, Ordering::SeqCst);
                        Message::new(i)
                    })
                    .into()
                } else {
                    // Async command
                    let counter = Arc::clone(&counter);
                    AsyncCmd::new(move || async move {
                        counter.fetch_add(1, Ordering::SeqCst);
                        Message::new(i)
                    })
                    .into()
                };
                handles.push(tokio::spawn(async move { kind.execute().await }));
            }

            // Wait for all
            for handle in handles {
                assert!(handle.await.unwrap().is_some());
            }

            // All 6 should have run
            assert_eq!(counter.load(Ordering::SeqCst), 6);
        }

        #[tokio::test]
        async fn test_command_kind_ordering_within_single_task() {
            use std::sync::Arc;
            use std::sync::atomic::{AtomicUsize, Ordering};

            #[derive(Debug, PartialEq)]
            struct OrderedResult {
                index: usize,
                order: usize,
            }

            let order = Arc::new(AtomicUsize::new(0));
            let mut results = vec![];

            // Execute commands sequentially within single task
            for i in 0..3usize {
                let order = Arc::clone(&order);
                let cmd = AsyncCmd::new(move || async move {
                    let n = order.fetch_add(1, Ordering::SeqCst);
                    Message::new(OrderedResult { index: i, order: n })
                });
                let msg = cmd.execute().await.unwrap();
                results.push(msg.downcast::<OrderedResult>().unwrap());
            }

            // Should execute in order
            assert_eq!(results[0], OrderedResult { index: 0, order: 0 });
            assert_eq!(results[1], OrderedResult { index: 1, order: 1 });
            assert_eq!(results[2], OrderedResult { index: 2, order: 2 });
        }

        // =========================================================================
        // Edge Cases
        // =========================================================================

        #[tokio::test]
        async fn test_async_cmd_with_large_message() {
            let large_data = vec![42u8; 1024 * 1024]; // 1MB
            let cmd = AsyncCmd::new(move || async move { Message::new(large_data) });
            let msg = cmd.execute().await.unwrap();
            let data = msg.downcast::<Vec<u8>>().unwrap();
            assert_eq!(data.len(), 1024 * 1024);
            assert!(data.iter().all(|&b| b == 42));
        }

        #[tokio::test]
        async fn test_every_async_produces_message() {
            struct EveryTick;

            let cmd = every_async(Duration::from_millis(1), |_| Message::new(EveryTick));
            let msg = cmd.execute().await.unwrap();
            assert!(msg.is::<EveryTick>());
        }

        #[tokio::test]
        async fn test_command_kind_from_conversions() {
            // Test From<Cmd> for CommandKind
            let sync_cmd = Cmd::new(|| Message::new(1i32));
            let kind: CommandKind = sync_cmd.into();
            assert!(matches!(kind, CommandKind::Sync(_)));

            // Test From<AsyncCmd> for CommandKind
            let async_cmd = AsyncCmd::new(|| async { Message::new(2i32) });
            let kind: CommandKind = async_cmd.into();
            assert!(matches!(kind, CommandKind::Async(_)));
        }

        #[tokio::test]
        async fn test_spawn_blocking_does_not_block_runtime() {
            use std::time::Instant;

            let start = Instant::now();

            // Start two blocking commands concurrently
            let cmd1: CommandKind = Cmd::blocking(|| {
                std::thread::sleep(Duration::from_millis(50));
                Message::new(1)
            })
            .into();

            let cmd2: CommandKind = Cmd::blocking(|| {
                std::thread::sleep(Duration::from_millis(50));
                Message::new(2)
            })
            .into();

            let (r1, r2) = tokio::join!(cmd1.execute(), cmd2.execute());

            let elapsed = start.elapsed();

            assert!(r1.is_some());
            assert!(r2.is_some());

            // Should run concurrently, so total time should be ~50ms, not ~100ms
            assert!(elapsed < Duration::from_millis(100));
        }
    }
}
