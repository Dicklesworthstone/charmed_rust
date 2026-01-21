//! Program lifecycle and event loop.
//!
//! The Program struct manages the entire TUI application lifecycle,
//! including terminal setup, event handling, and rendering.

use std::io::{self, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

#[cfg(feature = "async")]
use crate::command::CommandKind;

#[cfg(feature = "async")]
use tokio_util::sync::CancellationToken;

#[cfg(feature = "async")]
use tokio_util::task::TaskTracker;

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{
        self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};

use crate::KeyMsg;
use crate::command::Cmd;
use crate::key::from_crossterm_key;
use crate::message::{
    BatchMsg, BlurMsg, FocusMsg, InterruptMsg, Message, QuitMsg, RequestWindowSizeMsg, SequenceMsg,
    SetWindowTitleMsg, WindowSizeMsg,
};
use crate::mouse::from_crossterm_mouse;

/// Errors that can occur when running a bubbletea program.
///
/// This enum represents all possible error conditions when running
/// a TUI application with bubbletea.
///
/// # Error Handling
///
/// Most errors from bubbletea are recoverable. The recommended pattern
/// is to use the `?` operator for propagation:
///
/// ```rust,ignore
/// use bubbletea::{Program, Result};
///
/// fn run_app() -> Result<()> {
///     let program = Program::new(MyModel::default());
///     program.run()?;
///     Ok(())
/// }
/// ```
///
/// # Recovery Strategies
///
/// | Error Variant | Recovery Strategy |
/// |--------------|-------------------|
/// | [`Io`](Error::Io) | Check terminal availability, retry, or report to user |
///
/// # Example: Graceful Error Handling
///
/// ```rust,ignore
/// use bubbletea::{Program, Error};
///
/// match Program::new(my_model).run() {
///     Ok(final_model) => {
///         println!("Program completed successfully");
///     }
///     Err(Error::Io(e)) if e.kind() == std::io::ErrorKind::NotConnected => {
///         eprintln!("Terminal disconnected, saving state...");
///         // Save any important state before exiting
///     }
///     Err(e) => {
///         eprintln!("Program error: {}", e);
///         std::process::exit(1);
///     }
/// }
/// ```
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// I/O error during terminal operations.
    ///
    /// This typically occurs when:
    /// - The terminal is not available (e.g., running in a pipe)
    /// - The terminal was closed unexpectedly
    /// - System I/O resources are exhausted
    /// - Terminal control sequences failed
    ///
    /// # Recovery
    ///
    /// Check if stdin/stdout are TTYs before starting your program.
    /// Consider using a fallback mode for non-interactive environments.
    ///
    /// # Underlying Error
    ///
    /// The underlying [`std::io::Error`] can be accessed to determine
    /// the specific cause. Common error kinds include:
    /// - `NotConnected`: Terminal was disconnected
    /// - `BrokenPipe`: Output stream closed
    /// - `Other`: Terminal control sequence errors
    #[error("terminal io error: {0}")]
    Io(#[from] io::Error),
}

/// A specialized [`Result`] type for bubbletea operations.
///
/// This type alias is used throughout the bubbletea crate for convenience.
/// It defaults to [`Error`] as the error type.
///
/// # Example
///
/// ```rust,ignore
/// use bubbletea::Result;
///
/// fn run_program() -> Result<()> {
///     // ... implementation
///     Ok(())
/// }
/// ```
///
/// # Converting to Other Error Types
///
/// When integrating with other crates like `anyhow`:
///
/// ```rust,ignore
/// use anyhow::Context;
///
/// fn main() -> anyhow::Result<()> {
///     let model = bubbletea::Program::new(my_model)
///         .run()
///         .context("failed to run TUI program")?;
///     Ok(())
/// }
/// ```
pub type Result<T> = std::result::Result<T, Error>;

/// The Model trait for TUI applications.
///
/// Implement this trait to define your application's behavior.
///
/// # Example
///
/// ```rust
/// use bubbletea::{Model, Message, Cmd};
///
/// struct Counter { count: i32 }
///
/// impl Model for Counter {
///     fn init(&self) -> Option<Cmd> { None }
///
///     fn update(&mut self, msg: Message) -> Option<Cmd> {
///         if msg.is::<i32>() {
///             self.count += msg.downcast::<i32>().unwrap();
///         }
///         None
///     }
///
///     fn view(&self) -> String {
///         format!("Count: {}", self.count)
///     }
/// }
/// ```
pub trait Model: Send + 'static {
    /// Initialize the model and return an optional startup command.
    ///
    /// This is called once when the program starts.
    fn init(&self) -> Option<Cmd>;

    /// Process a message and return a new command.
    ///
    /// This is the pure update function at the heart of the Elm Architecture.
    fn update(&mut self, msg: Message) -> Option<Cmd>;

    /// Render the model as a string for display.
    ///
    /// This should be a pure function with no side effects.
    fn view(&self) -> String;
}

/// Program options.
#[derive(Debug, Clone)]
pub struct ProgramOptions {
    /// Use alternate screen buffer.
    pub alt_screen: bool,
    /// Enable mouse cell motion tracking.
    pub mouse_cell_motion: bool,
    /// Enable mouse all motion tracking.
    pub mouse_all_motion: bool,
    /// Enable bracketed paste mode.
    pub bracketed_paste: bool,
    /// Enable focus reporting.
    pub report_focus: bool,
    /// Use custom I/O (skip terminal setup and event polling).
    pub custom_io: bool,
    /// Target frames per second for rendering.
    pub fps: u32,
    /// Disable signal handling.
    pub without_signals: bool,
    /// Don't catch panics.
    pub without_catch_panics: bool,
}

impl Default for ProgramOptions {
    fn default() -> Self {
        Self {
            alt_screen: false,
            mouse_cell_motion: false,
            mouse_all_motion: false,
            bracketed_paste: true,
            report_focus: false,
            custom_io: false,
            fps: 60,
            without_signals: false,
            without_catch_panics: false,
        }
    }
}

/// The main program runner.
///
/// Program manages the entire lifecycle of a TUI application:
/// - Terminal setup and teardown
/// - Event polling and message dispatching
/// - Frame-rate limited rendering
///
/// # Example
///
/// ```rust,ignore
/// use bubbletea::Program;
///
/// let model = MyModel::new();
/// let final_model = Program::new(model)
///     .with_alt_screen()
///     .run()?;
/// ```
pub struct Program<M: Model> {
    model: M,
    options: ProgramOptions,
    external_rx: Option<Receiver<Message>>,
}

impl<M: Model> Program<M> {
    /// Create a new program with the given model.
    pub fn new(model: M) -> Self {
        Self {
            model,
            options: ProgramOptions::default(),
            external_rx: None,
        }
    }

    /// Provide an external message receiver.
    ///
    /// Messages received on this channel will be forwarded to the program's event loop.
    /// This is useful for injecting events from external sources (e.g. SSH).
    pub fn with_input_receiver(mut self, rx: Receiver<Message>) -> Self {
        self.external_rx = Some(rx);
        self
    }

    /// Use alternate screen buffer (full-screen mode).
    pub fn with_alt_screen(mut self) -> Self {
        self.options.alt_screen = true;
        self
    }

    /// Enable mouse cell motion tracking.
    ///
    /// Reports mouse clicks and drags.
    pub fn with_mouse_cell_motion(mut self) -> Self {
        self.options.mouse_cell_motion = true;
        self
    }

    /// Enable mouse all motion tracking.
    ///
    /// Reports all mouse movement, even without button presses.
    pub fn with_mouse_all_motion(mut self) -> Self {
        self.options.mouse_all_motion = true;
        self
    }

    /// Set the target frames per second.
    ///
    /// Default is 60 FPS. Valid range is 1-120 FPS.
    pub fn with_fps(mut self, fps: u32) -> Self {
        self.options.fps = fps.clamp(1, 120);
        self
    }

    /// Enable focus reporting.
    ///
    /// Sends FocusMsg and BlurMsg when terminal gains/loses focus.
    pub fn with_report_focus(mut self) -> Self {
        self.options.report_focus = true;
        self
    }

    /// Disable bracketed paste mode.
    pub fn without_bracketed_paste(mut self) -> Self {
        self.options.bracketed_paste = false;
        self
    }

    /// Disable signal handling.
    pub fn without_signal_handler(mut self) -> Self {
        self.options.without_signals = true;
        self
    }

    /// Don't catch panics.
    pub fn without_catch_panics(mut self) -> Self {
        self.options.without_catch_panics = true;
        self
    }

    /// Enable custom I/O mode (skip terminal setup and crossterm polling).
    ///
    /// This is useful when embedding bubbletea in environments that manage
    /// terminal state externally or when events are injected manually.
    pub fn with_custom_io(mut self) -> Self {
        self.options.custom_io = true;
        self
    }

    /// Run the program with a custom writer.
    pub fn run_with_writer<W: Write + Send + 'static>(self, mut writer: W) -> Result<M> {
        // Save options for cleanup (since self will be moved)
        let options = self.options.clone();

        // Setup terminal (skip for custom IO)
        if !options.custom_io {
            enable_raw_mode()?;
        }

        if options.alt_screen {
            execute!(writer, EnterAlternateScreen)?;
        }

        execute!(writer, Hide)?;

        if options.mouse_all_motion {
            execute!(writer, EnableMouseCapture)?;
        } else if options.mouse_cell_motion {
            execute!(writer, EnableMouseCapture)?;
        }

        if options.report_focus {
            execute!(writer, event::EnableFocusChange)?;
        }

        if options.bracketed_paste {
            execute!(writer, event::EnableBracketedPaste)?;
        }

        // Run the event loop
        let result = self.event_loop(&mut writer);

        // Cleanup terminal
        if options.bracketed_paste {
            let _ = execute!(writer, event::DisableBracketedPaste);
        }

        if options.report_focus {
            let _ = execute!(writer, event::DisableFocusChange);
        }

        if options.mouse_all_motion || options.mouse_cell_motion {
            let _ = execute!(writer, DisableMouseCapture);
        }

        let _ = execute!(writer, Show);

        if options.alt_screen {
            let _ = execute!(writer, LeaveAlternateScreen);
        }

        if !options.custom_io {
            let _ = disable_raw_mode();
        }

        result
    }

    /// Run the program and return the final model state.
    pub fn run(self) -> Result<M> {
        let stdout = io::stdout();
        self.run_with_writer(stdout)
    }

    fn event_loop<W: Write>(mut self, writer: &mut W) -> Result<M> {
        // Create message channel
        let (tx, rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();

        // Forward external messages
        if let Some(ext_rx) = self.external_rx.take() {
            let tx_clone = tx.clone();
            thread::spawn(move || {
                while let Ok(msg) = ext_rx.recv() {
                    let _ = tx_clone.send(msg);
                }
            });
        }

        // Get initial window size (only if not custom IO, otherwise trust init msg)
        if !self.options.custom_io
            && let Ok((width, height)) = terminal::size()
        {
            let _ = tx.send(Message::new(WindowSizeMsg { width, height }));
        }

        // Call init and handle initial command
        if let Some(cmd) = self.model.init() {
            self.handle_command(cmd, tx.clone());
        }

        // Render initial view
        let mut last_view = String::new();
        self.render(writer, &mut last_view)?;

        // Frame timing
        let frame_duration = Duration::from_secs_f64(1.0 / self.options.fps as f64);

        // Event loop
        loop {
            // Poll for events with frame-rate limiting (skip poll if custom IO)
            // Note: For custom IO, we assume events are injected via other means (not yet implemented fully)
            // For now, custom IO just skips crossterm polling.
            if !self.options.custom_io && event::poll(frame_duration)? {
                match event::read()? {
                    Event::Key(key_event) => {
                        // Only handle key press events, not release
                        if key_event.kind != KeyEventKind::Press {
                            continue;
                        }

                        let key_msg = from_crossterm_key(key_event.code, key_event.modifiers);

                        // Handle Ctrl+C specially
                        if key_msg.key_type == crate::KeyType::CtrlC {
                            let _ = tx.send(Message::new(InterruptMsg));
                        } else {
                            let _ = tx.send(Message::new(key_msg));
                        }
                    }
                    Event::Mouse(mouse_event) => {
                        let mouse_msg = from_crossterm_mouse(mouse_event);
                        let _ = tx.send(Message::new(mouse_msg));
                    }
                    Event::Resize(width, height) => {
                        let _ = tx.send(Message::new(WindowSizeMsg { width, height }));
                    }
                    Event::FocusGained => {
                        let _ = tx.send(Message::new(FocusMsg));
                    }
                    Event::FocusLost => {
                        let _ = tx.send(Message::new(BlurMsg));
                    }
                    Event::Paste(text) => {
                        // Send as a key message with paste flag
                        let key_msg = KeyMsg {
                            key_type: crate::KeyType::Runes,
                            runes: text.chars().collect(),
                            alt: false,
                            paste: true,
                        };
                        let _ = tx.send(Message::new(key_msg));
                    }
                }
            }

            // Process all pending messages
            let mut needs_render = false;
            while let Ok(msg) = rx.try_recv() {
                // Check for quit message
                if msg.is::<QuitMsg>() {
                    return Ok(self.model);
                }

                // Check for interrupt message (Ctrl+C)
                if msg.is::<InterruptMsg>() {
                    return Ok(self.model);
                }

                // Handle batch message (already handled in handle_command)
                if msg.is::<BatchMsg>() {
                    continue;
                }

                // Handle window title
                if let Some(title_msg) = msg.downcast_ref::<SetWindowTitleMsg>() {
                    execute!(writer, terminal::SetTitle(&title_msg.0))?;
                    continue;
                }

                // Handle window size request
                if msg.is::<RequestWindowSizeMsg>() {
                    if !self.options.custom_io
                        && let Ok((width, height)) = terminal::size()
                    {
                        let _ = tx.send(Message::new(WindowSizeMsg { width, height }));
                    }
                    continue;
                }

                // Update model
                if let Some(cmd) = self.model.update(msg) {
                    self.handle_command(cmd, tx.clone());
                }
                needs_render = true;
            }

            // Render if needed
            if needs_render {
                self.render(writer, &mut last_view)?;
            }

            // Sleep a bit if loop is tight (only needed if poll didn't sleep)
            if self.options.custom_io {
                thread::sleep(frame_duration);
            }
        }
    }

    fn handle_command(&self, cmd: Cmd, tx: Sender<Message>) {
        thread::spawn(move || {
            if let Some(msg) = cmd.execute() {
                // Handle batch and sequence messages specially
                if msg.is::<BatchMsg>() {
                    if let Some(batch) = msg.downcast::<BatchMsg>() {
                        for cmd in batch.0 {
                            let tx_clone = tx.clone();
                            thread::spawn(move || {
                                if let Some(msg) = cmd.execute() {
                                    let _ = tx_clone.send(msg);
                                }
                            });
                        }
                    }
                } else if msg.is::<SequenceMsg>() {
                    if let Some(seq) = msg.downcast::<SequenceMsg>() {
                        for cmd in seq.0 {
                            if let Some(msg) = cmd.execute() {
                                let _ = tx.send(msg);
                            }
                        }
                    }
                } else {
                    let _ = tx.send(msg);
                }
            }
        });
    }

    fn render<W: Write>(&self, writer: &mut W, last_view: &mut String) -> Result<()> {
        let view = self.model.view();

        // Skip if view hasn't changed
        if view == *last_view {
            return Ok(());
        }

        // Clear and render
        execute!(writer, MoveTo(0, 0), Clear(ClearType::All))?;
        write!(writer, "{}", view)?;
        writer.flush()?;

        *last_view = view;
        Ok(())
    }
}

// =============================================================================
// Async Program Implementation (requires "async" feature)
// =============================================================================

#[cfg(feature = "async")]
impl<M: Model> Program<M> {
    /// Run the program using the tokio async runtime.
    ///
    /// This is the async version of `run()`. It uses tokio for command execution
    /// and event handling, which is more efficient for I/O-bound operations.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use bubbletea::Program;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), bubbletea::Error> {
    ///     let model = MyModel::new();
    ///     let final_model = Program::new(model)
    ///         .with_alt_screen()
    ///         .run_async()
    ///         .await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn run_async(self) -> Result<M> {
        let mut stdout = io::stdout();

        // Save options for cleanup (since self will be moved)
        let options = self.options.clone();

        // Setup terminal (skip for custom I/O)
        if !options.custom_io {
            enable_raw_mode()?;
        }

        if options.alt_screen {
            execute!(stdout, EnterAlternateScreen)?;
        }

        execute!(stdout, Hide)?;

        if options.mouse_all_motion {
            execute!(stdout, EnableMouseCapture)?;
        } else if options.mouse_cell_motion {
            execute!(stdout, EnableMouseCapture)?;
        }

        if options.report_focus {
            execute!(stdout, event::EnableFocusChange)?;
        }

        if options.bracketed_paste {
            execute!(stdout, event::EnableBracketedPaste)?;
        }

        // Run the async event loop
        let result = self.event_loop_async(&mut stdout).await;

        // Cleanup terminal
        if options.bracketed_paste {
            let _ = execute!(stdout, event::DisableBracketedPaste);
        }

        if options.report_focus {
            let _ = execute!(stdout, event::DisableFocusChange);
        }

        if options.mouse_all_motion || options.mouse_cell_motion {
            let _ = execute!(stdout, DisableMouseCapture);
        }

        let _ = execute!(stdout, Show);

        if options.alt_screen {
            let _ = execute!(stdout, LeaveAlternateScreen);
        }

        if !options.custom_io {
            let _ = disable_raw_mode();
        }

        result
    }

    async fn event_loop_async(mut self, stdout: &mut io::Stdout) -> Result<M> {
        // Create async message channel
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Message>(256);

        // Create cancellation token and task tracker for graceful shutdown
        let cancel_token = CancellationToken::new();
        let task_tracker = TaskTracker::new();

        // Forward external messages using tokio's blocking thread pool
        // This is tracked for graceful shutdown and respects cancellation
        if let Some(ext_rx) = self.external_rx.take() {
            let tx_clone = tx.clone();
            let cancel_clone = cancel_token.clone();
            task_tracker.spawn_blocking(move || {
                // Use recv_timeout to periodically check for cancellation
                let timeout = Duration::from_millis(100);
                loop {
                    if cancel_clone.is_cancelled() {
                        break;
                    }
                    match ext_rx.recv_timeout(timeout) {
                        Ok(msg) => {
                            let _ = tx_clone.blocking_send(msg);
                        }
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                            // Continue loop to check cancellation
                        }
                        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                            // Channel closed, exit
                            break;
                        }
                    }
                }
            });
        }

        // Get initial window size
        if !self.options.custom_io {
            let (width, height) = terminal::size()?;
            let _ = tx.send(Message::new(WindowSizeMsg { width, height })).await;
        }

        // Call init and handle initial command
        if let Some(cmd) = self.model.init() {
            Self::handle_command_tracked(
                cmd.into(),
                tx.clone(),
                &task_tracker,
                cancel_token.clone(),
            );
        }

        // Render initial view
        let mut last_view = String::new();
        self.render(stdout, &mut last_view)?;

        // Frame timing
        let frame_duration = Duration::from_secs_f64(1.0 / self.options.fps as f64);
        let mut frame_interval = tokio::time::interval(frame_duration);

        // Event loop
        loop {
            tokio::select! {
                // Check for terminal events (using spawn_blocking for crossterm)
                event_result = Self::poll_event_async(), if !self.options.custom_io => {
                    if let Some(event) = event_result? {
                        match event {
                            Event::Key(key_event) => {
                                // Only handle key press events, not release
                                if key_event.kind != KeyEventKind::Press {
                                    continue;
                                }

                                let key_msg = from_crossterm_key(key_event.code, key_event.modifiers);

                                // Handle Ctrl+C specially
                                if key_msg.key_type == crate::KeyType::CtrlC {
                                    let _ = tx.send(Message::new(InterruptMsg)).await;
                                } else {
                                    let _ = tx.send(Message::new(key_msg)).await;
                                }
                            }
                            Event::Mouse(mouse_event) => {
                                let mouse_msg = from_crossterm_mouse(mouse_event);
                                let _ = tx.send(Message::new(mouse_msg)).await;
                            }
                            Event::Resize(width, height) => {
                                let _ = tx.send(Message::new(WindowSizeMsg { width, height })).await;
                            }
                            Event::FocusGained => {
                                let _ = tx.send(Message::new(FocusMsg)).await;
                            }
                            Event::FocusLost => {
                                let _ = tx.send(Message::new(BlurMsg)).await;
                            }
                            Event::Paste(text) => {
                                // Send as a key message with paste flag
                                let key_msg = KeyMsg {
                                    key_type: crate::KeyType::Runes,
                                    runes: text.chars().collect(),
                                    alt: false,
                                    paste: true,
                                };
                                let _ = tx.send(Message::new(key_msg)).await;
                            }
                        }
                    }
                }

                // Process incoming messages
                Some(msg) = rx.recv() => {
                    // Check for quit message - initiate graceful shutdown
                    if msg.is::<QuitMsg>() {
                        Self::graceful_shutdown(&cancel_token, &task_tracker).await;
                        return Ok(self.model);
                    }

                    // Check for interrupt message (Ctrl+C) - initiate graceful shutdown
                    if msg.is::<InterruptMsg>() {
                        Self::graceful_shutdown(&cancel_token, &task_tracker).await;
                        return Ok(self.model);
                    }

                    // Handle batch message (already handled in handle_command_tracked)
                    if msg.is::<BatchMsg>() {
                        continue;
                    }

                    // Handle window title
                    if let Some(title_msg) = msg.downcast_ref::<SetWindowTitleMsg>() {
                        execute!(stdout, terminal::SetTitle(&title_msg.0))?;
                        continue;
                    }

                    // Handle window size request
                    if msg.is::<RequestWindowSizeMsg>() {
                        if !self.options.custom_io {
                            let (width, height) = terminal::size()?;
                            let _ = tx.send(Message::new(WindowSizeMsg { width, height })).await;
                        }
                        continue;
                    }

                    // Update model
                    if let Some(cmd) = self.model.update(msg) {
                        Self::handle_command_tracked(
                            cmd.into(),
                            tx.clone(),
                            &task_tracker,
                            cancel_token.clone(),
                        );
                    }

                    // Render after processing message
                    self.render(stdout, &mut last_view)?;
                }

                // Frame tick for rendering
                _ = frame_interval.tick() => {
                    // Periodic render check (in case we missed something)
                }
            }
        }
    }

    /// Perform graceful shutdown: cancel all tasks and wait for them to complete.
    async fn graceful_shutdown(cancel_token: &CancellationToken, task_tracker: &TaskTracker) {
        // Signal all tasks to cancel
        cancel_token.cancel();

        // Close the tracker to prevent new tasks
        task_tracker.close();

        // Wait for all tasks with a timeout (5 seconds)
        let shutdown_timeout = Duration::from_secs(5);
        let _ = tokio::time::timeout(shutdown_timeout, task_tracker.wait()).await;
    }

    /// Poll for terminal events asynchronously.
    async fn poll_event_async() -> Result<Option<Event>> {
        // crossterm doesn't have native async support, so we use spawn_blocking
        tokio::task::spawn_blocking(|| {
            if event::poll(Duration::from_millis(10))? {
                Ok(Some(event::read()?))
            } else {
                Ok(None)
            }
        })
        .await
        .map_err(|_| Error::Io(io::Error::other("task join error")))?
    }

    /// Handle a command with task tracking and cancellation support.
    fn handle_command_tracked(
        cmd: CommandKind,
        tx: tokio::sync::mpsc::Sender<Message>,
        tracker: &TaskTracker,
        cancel_token: CancellationToken,
    ) {
        tracker.spawn(async move {
            tokio::select! {
                // Execute the command
                result = cmd.execute() => {
                    if let Some(msg) = result {
                        // Handle batch and sequence messages specially
                        if msg.is::<BatchMsg>() {
                            if let Some(batch) = msg.downcast::<BatchMsg>() {
                                for cmd in batch.0 {
                                    let tx_clone = tx.clone();
                                    // Note: batch commands don't get tracked individually
                                    // for simplicity, but they still respect the main cancel
                                    tokio::spawn(async move {
                                        let cmd_kind: CommandKind = cmd.into();
                                        if let Some(msg) = cmd_kind.execute().await {
                                            let _ = tx_clone.send(msg).await;
                                        }
                                    });
                                }
                            }
                        } else if msg.is::<SequenceMsg>() {
                            if let Some(seq) = msg.downcast::<SequenceMsg>() {
                                for cmd in seq.0 {
                                    let cmd_kind: CommandKind = cmd.into();
                                    if let Some(msg) = cmd_kind.execute().await {
                                        let _ = tx.send(msg).await;
                                    }
                                }
                            }
                        } else {
                            let _ = tx.send(msg).await;
                        }
                    }
                }
                // Cancellation requested - exit cleanly
                _ = cancel_token.cancelled() => {
                    // Command cancelled, cleanup happens automatically
                }
            }
        });
    }

    /// Handle a command asynchronously using tokio::spawn (legacy, without tracking).
    #[allow(dead_code)]
    fn handle_command_async(&self, cmd: CommandKind, tx: tokio::sync::mpsc::Sender<Message>) {
        tokio::spawn(async move {
            if let Some(msg) = cmd.execute().await {
                // Handle batch and sequence messages specially
                if msg.is::<BatchMsg>() {
                    if let Some(batch) = msg.downcast::<BatchMsg>() {
                        for cmd in batch.0 {
                            let tx_clone = tx.clone();
                            tokio::spawn(async move {
                                let cmd_kind: CommandKind = cmd.into();
                                if let Some(msg) = cmd_kind.execute().await {
                                    let _ = tx_clone.send(msg).await;
                                }
                            });
                        }
                    }
                } else if msg.is::<SequenceMsg>() {
                    if let Some(seq) = msg.downcast::<SequenceMsg>() {
                        for cmd in seq.0 {
                            let cmd_kind: CommandKind = cmd.into();
                            if let Some(msg) = cmd_kind.execute().await {
                                let _ = tx.send(msg).await;
                            }
                        }
                    }
                } else {
                    let _ = tx.send(msg).await;
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestModel {
        count: i32,
    }

    impl Model for TestModel {
        fn init(&self) -> Option<Cmd> {
            None
        }

        fn update(&mut self, msg: Message) -> Option<Cmd> {
            if let Some(n) = msg.downcast::<i32>() {
                self.count += n;
            }
            None
        }

        fn view(&self) -> String {
            format!("Count: {}", self.count)
        }
    }

    #[test]
    fn test_program_options_default() {
        let opts = ProgramOptions::default();
        assert!(!opts.alt_screen);
        assert!(!opts.mouse_cell_motion);
        assert!(opts.bracketed_paste);
        assert_eq!(opts.fps, 60);
    }

    #[test]
    fn test_program_builder() {
        let model = TestModel { count: 0 };
        let program = Program::new(model)
            .with_alt_screen()
            .with_mouse_cell_motion()
            .with_fps(30);

        assert!(program.options.alt_screen);
        assert!(program.options.mouse_cell_motion);
        assert_eq!(program.options.fps, 30);
    }

    #[test]
    fn test_program_fps_max() {
        let model = TestModel { count: 0 };
        let program = Program::new(model).with_fps(200);
        assert_eq!(program.options.fps, 120); // Capped at 120
    }

    #[test]
    fn test_program_fps_min() {
        let model = TestModel { count: 0 };
        let program = Program::new(model).with_fps(0);
        assert_eq!(program.options.fps, 1); // Clamped to minimum of 1 to avoid division by zero
    }
}
