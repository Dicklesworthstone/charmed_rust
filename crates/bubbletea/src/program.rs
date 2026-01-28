//! Program lifecycle and event loop.
//!
//! The Program struct manages the entire TUI application lifecycle,
//! including terminal setup, event handling, and rendering.

use std::io::{self, Read, Write};
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

use crate::command::Cmd;
use crate::key::{from_crossterm_key, is_sequence_prefix};
use crate::message::{
    BatchMsg, BlurMsg, FocusMsg, InterruptMsg, Message, PrintLineMsg, QuitMsg,
    RequestWindowSizeMsg, SequenceMsg, SetWindowTitleMsg, WindowSizeMsg,
};
use crate::mouse::from_crossterm_mouse;
use crate::screen::{ReleaseTerminalMsg, RestoreTerminalMsg};
use crate::{KeyMsg, KeyType};

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
/// | [`RawModeFailure`](Error::RawModeFailure) | Check terminal compatibility |
/// | [`AltScreenFailure`](Error::AltScreenFailure) | Disable alt screen option |
/// | [`EventPoll`](Error::EventPoll) | Terminal may be disconnected |
/// | [`Render`](Error::Render) | Check output stream, retry |
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
///     Err(Error::RawModeFailure { .. }) => {
///         eprintln!("Terminal doesn't support raw mode. Try a different terminal.");
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

    /// Failed to enable or disable raw mode.
    ///
    /// Raw mode is required for TUI operation as it disables terminal
    /// line buffering and echo. This error typically indicates the
    /// terminal doesn't support raw mode or isn't a TTY.
    ///
    /// # Recovery
    ///
    /// Verify the program is running in an interactive terminal.
    /// Some terminals (especially on Windows) may have limited support.
    #[error("failed to {action} raw mode: {source}")]
    RawModeFailure {
        /// Whether we were trying to enable or disable raw mode.
        action: &'static str,
        /// The underlying I/O error.
        #[source]
        source: io::Error,
    },

    /// Failed to enter or exit alternate screen.
    ///
    /// Alternate screen provides a separate buffer that preserves
    /// the user's terminal content. This error may indicate the
    /// terminal doesn't support alternate screen mode.
    ///
    /// # Recovery
    ///
    /// Try running without `.with_alt_screen()`.
    #[error("failed to {action} alternate screen: {source}")]
    AltScreenFailure {
        /// Whether we were trying to enter or exit alt screen.
        action: &'static str,
        /// The underlying I/O error.
        #[source]
        source: io::Error,
    },

    /// Failed to poll for terminal events.
    ///
    /// This error occurs when the event polling system fails,
    /// typically because the terminal was disconnected or closed.
    ///
    /// # Recovery
    ///
    /// The terminal connection may be lost. Save state and exit.
    #[error("failed to poll terminal events: {0}")]
    EventPoll(io::Error),

    /// Failed to render the view to the terminal.
    ///
    /// This error occurs when writing the view output fails,
    /// typically due to a broken pipe or disconnected terminal.
    ///
    /// # Recovery
    ///
    /// The output stream may be closed. Save state and exit.
    #[error("failed to render view: {0}")]
    Render(io::Error),
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

/// Handle to a running program.
///
/// Returned by [`Program::start()`] to allow external interaction with the
/// running TUI program. This is particularly useful for SSH applications
/// where events need to be injected from outside the program.
///
/// # Example
///
/// ```rust,ignore
/// use bubbletea::{Program, Message};
///
/// let handle = Program::new(MyModel::default())
///     .with_custom_io()
///     .start();
///
/// // Send a message to the running program
/// handle.send(MyMessage::DoSomething);
///
/// // Wait for the program to finish
/// let final_model = handle.wait()?;
/// ```
pub struct ProgramHandle<M: Model> {
    tx: Sender<Message>,
    handle: Option<thread::JoinHandle<Result<M>>>,
}

impl<M: Model> ProgramHandle<M> {
    /// Send a message to the running program.
    ///
    /// This queues the message for processing in the program's event loop.
    /// Returns `true` if the message was sent successfully, `false` if the
    /// program has already exited.
    pub fn send<T: Into<Message>>(&self, msg: T) -> bool {
        self.tx.send(msg.into()).is_ok()
    }

    /// Request the program to quit.
    ///
    /// This sends a `QuitMsg` to the program's event loop.
    pub fn quit(&self) {
        let _ = self.tx.send(Message::new(QuitMsg));
    }

    /// Wait for the program to finish and return the final model state.
    ///
    /// This blocks until the program exits.
    pub fn wait(mut self) -> Result<M> {
        if let Some(handle) = self.handle.take() {
            handle
                .join()
                .map_err(|_| Error::Io(io::Error::other("program thread panicked")))?
        } else {
            Err(Error::Io(io::Error::other("program already joined")))
        }
    }

    /// Check if the program is still running.
    pub fn is_running(&self) -> bool {
        self.handle.as_ref().is_some_and(|h| !h.is_finished())
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
    input: Option<Box<dyn Read + Send>>,
    output: Option<Box<dyn Write + Send>>,
}

impl<M: Model> Program<M> {
    /// Create a new program with the given model.
    pub fn new(model: M) -> Self {
        Self {
            model,
            options: ProgramOptions::default(),
            external_rx: None,
            input: None,
            output: None,
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

    /// Provide a custom input reader.
    ///
    /// This enables custom I/O mode and reads raw bytes from the given reader,
    /// translating them into Bubbletea messages.
    pub fn with_input<R: Read + Send + 'static>(mut self, input: R) -> Self {
        self.input = Some(Box::new(input));
        self.options.custom_io = true;
        self
    }

    /// Provide a custom output writer.
    ///
    /// This enables custom I/O mode and writes render output to the given writer.
    pub fn with_output<W: Write + Send + 'static>(mut self, output: W) -> Self {
        self.output = Some(Box::new(output));
        self.options.custom_io = true;
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
    pub fn run(mut self) -> Result<M> {
        if let Some(output) = self.output.take() {
            return self.run_with_writer(output);
        }

        let stdout = io::stdout();
        self.run_with_writer(stdout)
    }

    /// Start the program in a background thread and return a handle for interaction.
    ///
    /// This is useful for SSH applications and other scenarios where you need to
    /// inject events from external sources after the program has started.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use bubbletea::{Program, Message};
    ///
    /// let handle = Program::new(MyModel::default())
    ///     .with_custom_io()
    ///     .start();
    ///
    /// // Inject a key event
    /// handle.send(KeyMsg::from_char('a'));
    ///
    /// // Later, quit the program
    /// handle.quit();
    ///
    /// // Wait for completion
    /// let final_model = handle.wait()?;
    /// ```
    pub fn start(mut self) -> ProgramHandle<M> {
        // Create channel for external message injection
        let (tx, rx) = mpsc::channel();

        // Set up external receiver (will be forwarded in event_loop)
        self.external_rx = Some(rx);

        // Take ownership of custom output if provided
        let output = self.output.take();

        // Spawn program in background thread
        let handle = thread::spawn(move || {
            if let Some(output) = output {
                self.run_with_writer(output)
            } else {
                let stdout = io::stdout();
                self.run_with_writer(stdout)
            }
        });

        ProgramHandle {
            tx,
            handle: Some(handle),
        }
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

        // Read custom input stream and inject messages.
        if let Some(mut input) = self.input.take() {
            let tx_clone = tx.clone();
            thread::spawn(move || {
                let mut parser = InputParser::new();
                let mut buf = [0u8; 256];
                loop {
                    match input.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            // We always assume there could be more data unless we hit EOF (Ok(0))
                            let can_have_more_data = true;
                            for msg in parser.push_bytes(&buf[..n], can_have_more_data) {
                                let _ = tx_clone.send(msg);
                            }
                        }
                        Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                            thread::yield_now();
                        }
                        Err(_) => break,
                    }
                }

                for msg in parser.flush() {
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
            // In custom IO mode, events are injected via `with_input_receiver()` or `with_input()`.
            // Crossterm polling is skipped since input comes from external sources.
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

                // Handle print line message (only when not in alt screen)
                if let Some(print_msg) = msg.downcast_ref::<PrintLineMsg>() {
                    if !self.options.alt_screen {
                        // Print each line above the TUI
                        for line in print_msg.0.lines() {
                            let _ = writeln!(writer, "{}", line);
                        }
                        let _ = writer.flush();
                        // Force a full re-render since we printed above
                        last_view.clear();
                        needs_render = true;
                    }
                    continue;
                }

                // Handle release terminal
                if msg.is::<ReleaseTerminalMsg>() {
                    if !self.options.custom_io {
                        // Disable features in reverse order
                        if self.options.bracketed_paste {
                            let _ = execute!(writer, event::DisableBracketedPaste);
                        }
                        if self.options.report_focus {
                            let _ = execute!(writer, event::DisableFocusChange);
                        }
                        if self.options.mouse_all_motion || self.options.mouse_cell_motion {
                            let _ = execute!(writer, DisableMouseCapture);
                        }
                        let _ = execute!(writer, Show);
                        if self.options.alt_screen {
                            let _ = execute!(writer, LeaveAlternateScreen);
                        }
                        let _ = disable_raw_mode();
                    }
                    continue;
                }

                // Handle restore terminal
                if msg.is::<RestoreTerminalMsg>() {
                    if !self.options.custom_io {
                        // Re-enable features in original order
                        let _ = enable_raw_mode();
                        if self.options.alt_screen {
                            let _ = execute!(writer, EnterAlternateScreen);
                        }
                        let _ = execute!(writer, Hide);
                        if self.options.mouse_all_motion {
                            let _ = execute!(writer, EnableMouseCapture);
                        } else if self.options.mouse_cell_motion {
                            let _ = execute!(writer, EnableMouseCapture);
                        }
                        if self.options.report_focus {
                            let _ = execute!(writer, event::EnableFocusChange);
                        }
                        if self.options.bracketed_paste {
                            let _ = execute!(writer, event::EnableBracketedPaste);
                        }
                        // Force a full re-render
                        last_view.clear();
                    }
                    needs_render = true;
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
    pub async fn run_async(mut self) -> Result<M> {
        if let Some(output) = self.output.take() {
            return self.run_async_with_writer(output).await;
        }

        let stdout = io::stdout();
        self.run_async_with_writer(stdout).await
    }

    /// Run the program using the tokio async runtime with a custom writer.
    pub async fn run_async_with_writer<W: Write + Send + 'static>(
        self,
        mut writer: W,
    ) -> Result<M> {
        // Save options for cleanup (since self will be moved)
        let options = self.options.clone();

        // Setup terminal (skip for custom I/O)
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

        // Run the async event loop
        let result = self.event_loop_async(&mut writer).await;

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

    async fn event_loop_async<W: Write>(mut self, stdout: &mut W) -> Result<M> {
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

        // Read custom input stream and inject messages.
        if let Some(mut input) = self.input.take() {
            let tx_clone = tx.clone();
            let cancel_clone = cancel_token.clone();
            task_tracker.spawn_blocking(move || {
                let mut parser = InputParser::new();
                let mut buf = [0u8; 256];
                loop {
                    if cancel_clone.is_cancelled() {
                        break;
                    }
                    match input.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            // We always assume there could be more data unless we hit EOF (Ok(0))
                            let can_have_more_data = true;
                            for msg in parser.push_bytes(&buf[..n], can_have_more_data) {
                                if tx_clone.blocking_send(msg).is_err() {
                                    return;
                                }
                            }
                        }
                        Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                            std::thread::yield_now();
                        }
                        Err(_) => break,
                    }
                }

                for msg in parser.flush() {
                    if tx_clone.blocking_send(msg).is_err() {
                        break;
                    }
                }
            });
        }

        // Spawn event listener thread
        let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<Event>(100);
        let event_cancel = cancel_token.clone();

        if !self.options.custom_io {
            std::thread::spawn(move || {
                loop {
                    if event_cancel.is_cancelled() {
                        break;
                    }
                    // Poll with timeout to check cancellation
                    match event::poll(Duration::from_millis(100)) {
                        Ok(true) => {
                            if let Ok(evt) = event::read()
                                && event_tx.blocking_send(evt).is_err()
                            {
                                break;
                            }
                        }
                        Ok(false) => {} // timeout
                        Err(_) => {
                            break;
                        } // error
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
                // Check for terminal events via channel
                Some(event) = event_rx.recv(), if !self.options.custom_io => {
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

                    // Handle print line message (only when not in alt screen)
                    if let Some(print_msg) = msg.downcast_ref::<PrintLineMsg>() {
                        if !self.options.alt_screen {
                            // Print each line above the TUI
                            for line in print_msg.0.lines() {
                                let _ = writeln!(stdout, "{}", line);
                            }
                            let _ = stdout.flush();
                            // Force a full re-render since we printed above
                            last_view.clear();
                        }
                        self.render(stdout, &mut last_view)?;
                        continue;
                    }

                    // Handle release terminal
                    if msg.is::<ReleaseTerminalMsg>() {
                        if !self.options.custom_io {
                            // Disable features in reverse order
                            if self.options.bracketed_paste {
                                let _ = execute!(stdout, event::DisableBracketedPaste);
                            }
                            if self.options.report_focus {
                                let _ = execute!(stdout, event::DisableFocusChange);
                            }
                            if self.options.mouse_all_motion || self.options.mouse_cell_motion {
                                let _ = execute!(stdout, DisableMouseCapture);
                            }
                            let _ = execute!(stdout, Show);
                            if self.options.alt_screen {
                                let _ = execute!(stdout, LeaveAlternateScreen);
                            }
                            let _ = disable_raw_mode();
                        }
                        continue;
                    }

                    // Handle restore terminal
                    if msg.is::<RestoreTerminalMsg>() {
                        if !self.options.custom_io {
                            // Re-enable features in original order
                            let _ = enable_raw_mode();
                            if self.options.alt_screen {
                                let _ = execute!(stdout, EnterAlternateScreen);
                            }
                            let _ = execute!(stdout, Hide);
                            if self.options.mouse_all_motion {
                                let _ = execute!(stdout, EnableMouseCapture);
                            } else if self.options.mouse_cell_motion {
                                let _ = execute!(stdout, EnableMouseCapture);
                            }
                            if self.options.report_focus {
                                let _ = execute!(stdout, event::EnableFocusChange);
                            }
                            if self.options.bracketed_paste {
                                let _ = execute!(stdout, event::EnableBracketedPaste);
                            }
                            // Force a full re-render
                            last_view.clear();
                        }
                        self.render(stdout, &mut last_view)?;
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

// =============================================================================
// Custom Input Parsing (for custom I/O mode)
// =============================================================================

struct InputParser {
    buffer: Vec<u8>,
}

impl InputParser {
    fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    fn push_bytes(&mut self, bytes: &[u8], can_have_more_data: bool) -> Vec<Message> {
        if !bytes.is_empty() {
            self.buffer.extend_from_slice(bytes);
        }

        let mut messages = Vec::new();
        loop {
            if self.buffer.is_empty() {
                break;
            }

            match parse_one_message(&self.buffer, can_have_more_data) {
                ParseOutcome::NeedMore => break,
                ParseOutcome::Parsed(consumed, msg) => {
                    self.buffer.drain(0..consumed);
                    if let Some(msg) = msg {
                        messages.push(msg);
                    }
                }
            }
        }

        messages
    }

    fn flush(&mut self) -> Vec<Message> {
        let mut messages = Vec::new();
        loop {
            if self.buffer.is_empty() {
                break;
            }

            match parse_one_message(&self.buffer, false) {
                ParseOutcome::NeedMore => break,
                ParseOutcome::Parsed(consumed, msg) => {
                    self.buffer.drain(0..consumed);
                    if let Some(msg) = msg {
                        messages.push(msg);
                    }
                }
            }
        }
        messages
    }
}

enum ParseOutcome {
    NeedMore,
    Parsed(usize, Option<Message>),
}

fn parse_one_message(buf: &[u8], can_have_more_data: bool) -> ParseOutcome {
    if buf.is_empty() {
        return ParseOutcome::NeedMore;
    }

    if let Some(outcome) = parse_mouse_event(buf, can_have_more_data) {
        return outcome;
    }

    if let Some(outcome) = parse_focus_event(buf, can_have_more_data) {
        return outcome;
    }

    if let Some(outcome) = parse_bracketed_paste(buf, can_have_more_data) {
        return outcome;
    }

    if let Some(outcome) = parse_key_sequence(buf, can_have_more_data) {
        return outcome;
    }

    parse_runes_or_control(buf, can_have_more_data)
}

fn parse_mouse_event(buf: &[u8], can_have_more_data: bool) -> Option<ParseOutcome> {
    if buf.starts_with(b"\x1b[M") {
        if buf.len() < 6 {
            return Some(if can_have_more_data {
                ParseOutcome::NeedMore
            } else {
                ParseOutcome::Parsed(1, Some(replacement_message()))
            });
        }
        let seq = &buf[..6];
        return Some(match crate::mouse::parse_mouse_event_sequence(seq) {
            Ok(msg) => ParseOutcome::Parsed(6, Some(Message::new(msg))),
            Err(_) => ParseOutcome::Parsed(1, Some(replacement_message())),
        });
    }

    if buf.starts_with(b"\x1b[<") {
        if let Some(end_idx) = buf.iter().position(|b| *b == b'M' || *b == b'm') {
            let seq = &buf[..=end_idx];
            return Some(match crate::mouse::parse_mouse_event_sequence(seq) {
                Ok(msg) => ParseOutcome::Parsed(seq.len(), Some(Message::new(msg))),
                Err(_) => ParseOutcome::Parsed(1, Some(replacement_message())),
            });
        }
        return Some(if can_have_more_data {
            ParseOutcome::NeedMore
        } else {
            ParseOutcome::Parsed(1, Some(replacement_message()))
        });
    }

    None
}

fn parse_focus_event(buf: &[u8], can_have_more_data: bool) -> Option<ParseOutcome> {
    if buf.len() < 3 && buf.starts_with(b"\x1b[") && can_have_more_data {
        return Some(ParseOutcome::NeedMore);
    }

    if buf.starts_with(b"\x1b[I") {
        return Some(ParseOutcome::Parsed(3, Some(Message::new(FocusMsg))));
    }

    if buf.starts_with(b"\x1b[O") {
        return Some(ParseOutcome::Parsed(3, Some(Message::new(BlurMsg))));
    }

    None
}

fn parse_bracketed_paste(buf: &[u8], can_have_more_data: bool) -> Option<ParseOutcome> {
    const BP_START: &[u8] = b"\x1b[200~";
    const BP_END: &[u8] = b"\x1b[201~";

    if !buf.starts_with(BP_START) {
        return None;
    }

    if let Some(idx) = buf.windows(BP_END.len()).position(|w| w == BP_END) {
        let content = &buf[BP_START.len()..idx];
        let text = String::from_utf8_lossy(content);
        let runes = text.chars().collect::<Vec<char>>();
        let key = KeyMsg::from_runes(runes).with_paste();
        let total_len = idx + BP_END.len();
        return Some(ParseOutcome::Parsed(total_len, Some(message_from_key(key))));
    }

    Some(if can_have_more_data {
        ParseOutcome::NeedMore
    } else {
        let content = &buf[BP_START.len()..];
        let text = String::from_utf8_lossy(content);
        let runes = text.chars().collect::<Vec<char>>();
        let key = KeyMsg::from_runes(runes).with_paste();
        ParseOutcome::Parsed(buf.len(), Some(message_from_key(key)))
    })
}

fn parse_key_sequence(buf: &[u8], can_have_more_data: bool) -> Option<ParseOutcome> {
    if let Some((key, len)) = crate::key::parse_sequence_prefix(buf) {
        return Some(ParseOutcome::Parsed(len, Some(message_from_key(key))));
    }

    // Check if it's a prefix of a known sequence
    if can_have_more_data && is_sequence_prefix(buf) {
        return Some(ParseOutcome::NeedMore);
    }

    if buf.starts_with(b"\x1b")
        && let Some((mut key, len)) = crate::key::parse_sequence_prefix(&buf[1..])
    {
        if !key.alt {
            key = key.with_alt();
        }
        return Some(ParseOutcome::Parsed(len + 1, Some(message_from_key(key))));
    }

    None
}

fn parse_runes_or_control(buf: &[u8], can_have_more_data: bool) -> ParseOutcome {
    let mut alt = false;
    let mut idx = 0;

    if buf[0] == 0x1b {
        if buf.len() == 1 {
            return if can_have_more_data {
                ParseOutcome::NeedMore
            } else {
                ParseOutcome::Parsed(1, Some(message_from_key(KeyMsg::from_type(KeyType::Esc))))
            };
        }
        alt = true;
        idx = 1;
    }

    if idx >= buf.len() {
        return ParseOutcome::NeedMore;
    }

    if let Some(key_type) = control_key_type(buf[idx]) {
        let mut key = KeyMsg::from_type(key_type);
        if alt {
            key = key.with_alt();
        }
        return ParseOutcome::Parsed(idx + 1, Some(message_from_key(key)));
    }

    let mut runes = Vec::new();
    let mut i = idx;
    while i < buf.len() {
        let b = buf[i];
        if is_control_or_space(b) {
            break;
        }

        let (ch, width, valid) = match decode_char(&buf[i..], can_have_more_data) {
            DecodeOutcome::NeedMore => return ParseOutcome::NeedMore,
            DecodeOutcome::Decoded(ch, width, valid) => (ch, width, valid),
        };

        if !valid {
            runes.push(std::char::REPLACEMENT_CHARACTER);
            i += 1;
        } else {
            runes.push(ch);
            i += width;
        }

        if alt {
            break;
        }
    }

    if !runes.is_empty() {
        let mut key = KeyMsg::from_runes(runes);
        if alt {
            key = key.with_alt();
        }
        return ParseOutcome::Parsed(i, Some(message_from_key(key)));
    }

    ParseOutcome::Parsed(1, Some(replacement_message()))
}

fn control_key_type(byte: u8) -> Option<KeyType> {
    match byte {
        0x00 => Some(KeyType::Null),
        0x01 => Some(KeyType::CtrlA),
        0x02 => Some(KeyType::CtrlB),
        0x03 => Some(KeyType::CtrlC),
        0x04 => Some(KeyType::CtrlD),
        0x05 => Some(KeyType::CtrlE),
        0x06 => Some(KeyType::CtrlF),
        0x07 => Some(KeyType::CtrlG),
        0x08 => Some(KeyType::CtrlH),
        0x09 => Some(KeyType::Tab),
        0x0A => Some(KeyType::CtrlJ),
        0x0B => Some(KeyType::CtrlK),
        0x0C => Some(KeyType::CtrlL),
        0x0D => Some(KeyType::Enter),
        0x0E => Some(KeyType::CtrlN),
        0x0F => Some(KeyType::CtrlO),
        0x10 => Some(KeyType::CtrlP),
        0x11 => Some(KeyType::CtrlQ),
        0x12 => Some(KeyType::CtrlR),
        0x13 => Some(KeyType::CtrlS),
        0x14 => Some(KeyType::CtrlT),
        0x15 => Some(KeyType::CtrlU),
        0x16 => Some(KeyType::CtrlV),
        0x17 => Some(KeyType::CtrlW),
        0x18 => Some(KeyType::CtrlX),
        0x19 => Some(KeyType::CtrlY),
        0x1A => Some(KeyType::CtrlZ),
        0x1B => Some(KeyType::Esc),
        0x1C => Some(KeyType::CtrlBackslash),
        0x1D => Some(KeyType::CtrlCloseBracket),
        0x1E => Some(KeyType::CtrlCaret),
        0x1F => Some(KeyType::CtrlUnderscore),
        0x20 => Some(KeyType::Space),
        0x7F => Some(KeyType::Backspace),
        _ => None,
    }
}

fn is_control_or_space(byte: u8) -> bool {
    byte <= 0x1F || byte == 0x7F || byte == b' '
}

enum DecodeOutcome {
    NeedMore,
    Decoded(char, usize, bool),
}

fn decode_char(input: &[u8], can_have_more_data: bool) -> DecodeOutcome {
    let first = input[0];
    let width = if first < 0x80 {
        1
    } else if (first & 0xE0) == 0xC0 {
        2
    } else if (first & 0xF0) == 0xE0 {
        3
    } else if (first & 0xF8) == 0xF0 {
        4
    } else {
        return DecodeOutcome::Decoded(std::char::REPLACEMENT_CHARACTER, 1, false);
    };

    if input.len() < width {
        return if can_have_more_data {
            DecodeOutcome::NeedMore
        } else {
            DecodeOutcome::Decoded(std::char::REPLACEMENT_CHARACTER, 1, false)
        };
    }

    match std::str::from_utf8(&input[..width]) {
        Ok(s) => {
            let ch = s.chars().next().unwrap_or(std::char::REPLACEMENT_CHARACTER);
            DecodeOutcome::Decoded(ch, width, true)
        }
        Err(_) => DecodeOutcome::Decoded(std::char::REPLACEMENT_CHARACTER, 1, false),
    }
}

fn message_from_key(key: KeyMsg) -> Message {
    if key.key_type == KeyType::CtrlC {
        Message::new(InterruptMsg)
    } else {
        Message::new(key)
    }
}

fn replacement_message() -> Message {
    Message::new(KeyMsg::from_char(std::char::REPLACEMENT_CHARACTER))
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

    // === Bracketed Paste Parsing Tests ===

    #[test]
    fn test_parse_bracketed_paste_basic() {
        // Bracketed paste sequence: ESC[200~ ... ESC[201~
        let input = b"\x1b[200~hello world\x1b[201~";
        let result = parse_bracketed_paste(input, false);

        assert!(result.is_some());
        if let Some(ParseOutcome::Parsed(len, Some(msg))) = result {
            assert_eq!(len, input.len());
            let key = msg.downcast_ref::<KeyMsg>().unwrap();
            assert!(key.paste, "Key should have paste flag set");
            assert_eq!(key.runes, vec!['h', 'e', 'l', 'l', 'o', ' ', 'w', 'o', 'r', 'l', 'd']);
        } else {
            panic!("Expected Parsed outcome");
        }
    }

    #[test]
    fn test_parse_bracketed_paste_empty() {
        let input = b"\x1b[200~\x1b[201~";
        let result = parse_bracketed_paste(input, false);

        assert!(result.is_some());
        if let Some(ParseOutcome::Parsed(len, Some(msg))) = result {
            assert_eq!(len, input.len());
            let key = msg.downcast_ref::<KeyMsg>().unwrap();
            assert!(key.paste);
            assert!(key.runes.is_empty());
        } else {
            panic!("Expected Parsed outcome");
        }
    }

    #[test]
    fn test_parse_bracketed_paste_multiline() {
        let input = b"\x1b[200~line1\nline2\nline3\x1b[201~";
        let result = parse_bracketed_paste(input, false);

        assert!(result.is_some());
        if let Some(ParseOutcome::Parsed(len, Some(msg))) = result {
            assert_eq!(len, input.len());
            let key = msg.downcast_ref::<KeyMsg>().unwrap();
            assert!(key.paste);
            let text: String = key.runes.iter().collect();
            assert_eq!(text, "line1\nline2\nline3");
        } else {
            panic!("Expected Parsed outcome");
        }
    }

    #[test]
    fn test_parse_bracketed_paste_unicode() {
        let input = "\x1b[200~hello 世界 🌍\x1b[201~".as_bytes();
        let result = parse_bracketed_paste(input, false);

        assert!(result.is_some());
        if let Some(ParseOutcome::Parsed(_, Some(msg))) = result {
            let key = msg.downcast_ref::<KeyMsg>().unwrap();
            assert!(key.paste);
            let text: String = key.runes.iter().collect();
            assert_eq!(text, "hello 世界 🌍");
        } else {
            panic!("Expected Parsed outcome");
        }
    }

    #[test]
    fn test_parse_bracketed_paste_incomplete() {
        // Missing end sequence, with more data expected
        let input = b"\x1b[200~hello";
        let result = parse_bracketed_paste(input, true);

        assert!(matches!(result, Some(ParseOutcome::NeedMore)));
    }

    #[test]
    fn test_parse_bracketed_paste_incomplete_no_more_data() {
        // Missing end sequence, no more data expected - should parse what we have
        let input = b"\x1b[200~hello";
        let result = parse_bracketed_paste(input, false);

        assert!(result.is_some());
        if let Some(ParseOutcome::Parsed(len, Some(msg))) = result {
            assert_eq!(len, input.len());
            let key = msg.downcast_ref::<KeyMsg>().unwrap();
            assert!(key.paste);
            let text: String = key.runes.iter().collect();
            assert_eq!(text, "hello");
        } else {
            panic!("Expected Parsed outcome");
        }
    }

    #[test]
    fn test_parse_bracketed_paste_not_bracketed() {
        // Regular input, not bracketed paste
        let input = b"hello";
        let result = parse_bracketed_paste(input, false);
        assert!(result.is_none(), "Non-paste input should return None");
    }

    #[test]
    fn test_parse_bracketed_paste_large() {
        // Large paste (simulating a big paste operation)
        let content = "a".repeat(10000);
        let input = format!("\x1b[200~{}\x1b[201~", content);
        let result = parse_bracketed_paste(input.as_bytes(), false);

        assert!(result.is_some());
        if let Some(ParseOutcome::Parsed(len, Some(msg))) = result {
            assert_eq!(len, input.len());
            let key = msg.downcast_ref::<KeyMsg>().unwrap();
            assert!(key.paste);
            assert_eq!(key.runes.len(), 10000);
        } else {
            panic!("Expected Parsed outcome");
        }
    }
}
