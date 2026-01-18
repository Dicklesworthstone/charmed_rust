//! Program lifecycle and event loop.
//!
//! The Program struct manages the entire TUI application lifecycle,
//! including terminal setup, event handling, and rendering.

use std::io::{self, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

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

/// Error type for program execution.
#[derive(Debug)]
pub enum Error {
    /// IO error.
    Io(io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

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
}

impl<M: Model> Program<M> {
    /// Create a new program with the given model.
    pub fn new(model: M) -> Self {
        Self {
            model,
            options: ProgramOptions::default(),
        }
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
    /// Default is 60 FPS. Maximum is 120 FPS.
    pub fn with_fps(mut self, fps: u32) -> Self {
        self.options.fps = fps.min(120);
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

    /// Run the program and return the final model state.
    pub fn run(self) -> Result<M, Error> {
        let mut stdout = io::stdout();

        // Save options for cleanup (since self will be moved)
        let options = self.options.clone();

        // Setup terminal
        enable_raw_mode()?;

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

        // Run the event loop
        let result = self.event_loop(&mut stdout);

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

        let _ = disable_raw_mode();

        result
    }

    fn event_loop(mut self, stdout: &mut io::Stdout) -> Result<M, Error> {
        // Create message channel
        let (tx, rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();

        // Get initial window size
        let (width, height) = terminal::size()?;
        let _ = tx.send(Message::new(WindowSizeMsg { width, height }));

        // Call init and handle initial command
        if let Some(cmd) = self.model.init() {
            self.handle_command(cmd, tx.clone());
        }

        // Render initial view
        let mut last_view = String::new();
        self.render(stdout, &mut last_view)?;

        // Frame timing
        let frame_duration = Duration::from_secs_f64(1.0 / self.options.fps as f64);

        // Event loop
        loop {
            // Poll for events with frame-rate limiting
            if event::poll(frame_duration)? {
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
                    execute!(stdout, terminal::SetTitle(&title_msg.0))?;
                    continue;
                }

                // Handle window size request
                if msg.is::<RequestWindowSizeMsg>() {
                    let (width, height) = terminal::size()?;
                    let _ = tx.send(Message::new(WindowSizeMsg { width, height }));
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
                self.render(stdout, &mut last_view)?;
            }
        }
    }

    fn handle_command(&self, cmd: Cmd, tx: Sender<Message>) {
        // Execute command in a separate thread
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

    fn render(&self, stdout: &mut io::Stdout, last_view: &mut String) -> Result<(), Error> {
        let view = self.model.view();

        // Skip if view hasn't changed
        if view == *last_view {
            return Ok(());
        }

        // Clear and render
        execute!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;
        write!(stdout, "{}", view)?;
        stdout.flush()?;

        *last_view = view;
        Ok(())
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
}
