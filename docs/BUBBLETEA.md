# Bubbletea Crate Guide

## Overview

Bubbletea is a TUI (Terminal User Interface) framework implementing the Elm Architecture in Rust. It provides a purely functional approach to building interactive terminal applications with:

- **Model** - Application state
- **Update** - Pure function processing messages
- **View** - Renders state to string
- **Cmd** - Lazy IO operations returning messages

## Go Source Reference

Primary files from `legacy_bubbletea/`:
- `tea.go` - Core Program struct, lifecycle, event loop
- `commands.go` - Batch, Sequence, Tick, Every
- `key.go` - Keyboard input handling
- `key_sequences.go` - ANSI escape sequence mapping
- `mouse.go` - Mouse input handling
- `renderer.go` - Renderer trait definition
- `standard_renderer.go` - Frame-rate based renderer (60 FPS)
- `screen.go` - Screen control commands
- `tty.go` - Terminal state management
- `options.go` - Program configuration

## Architecture

### Core Traits

```rust
/// Application model trait - the heart of the Elm Architecture.
pub trait Model: Send + 'static {
    /// Initialize the model and return an optional startup command.
    fn init(&self) -> Option<Cmd>;

    /// Process a message and return an optional follow-up command.
    fn update(&mut self, msg: Message) -> Option<Cmd>;

    /// Render the model as a string for display.
    fn view(&self) -> String;
}
```

### Message System

```rust
/// Type-erased message container.
pub struct Message(Box<dyn Any + Send>);

impl Message {
    /// Create a new message from any sendable type.
    pub fn new<M: Any + Send + 'static>(msg: M) -> Self;

    /// Try to downcast to a specific message type.
    pub fn downcast<M: Any + Send + 'static>(self) -> Option<M>;

    /// Check if message is a specific type.
    pub fn is<M: Any + Send + 'static>(&self) -> bool;
}

// Built-in message types
pub struct QuitMsg;
pub struct InterruptMsg;
pub struct SuspendMsg;
pub struct ResumeMsg;
pub struct WindowSizeMsg { pub width: u16, pub height: u16 }
pub struct FocusMsg;
pub struct BlurMsg;
```

### Commands

```rust
/// A lazy IO operation that may produce a message.
pub struct Cmd(/* closure-backed command */);

impl Cmd {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> Message + Send + 'static;

    pub fn new_optional<F>(f: F) -> Self
    where
        F: FnOnce() -> Option<Message> + Send + 'static;

    pub fn execute(self) -> Option<Message>;
}

/// Batch multiple commands to run concurrently (unordered).
pub fn batch(cmds: Vec<Option<Cmd>>) -> Option<Cmd>;

/// Sequence commands to run in order.
pub fn sequence(cmds: Vec<Option<Cmd>>) -> Option<Cmd>;

/// Command that signals program to quit.
pub fn quit() -> Cmd;

/// Tick command for periodic updates.
pub fn tick<F>(duration: Duration, f: F) -> Cmd
where
    F: FnOnce(Instant) -> Message + Send + 'static;

/// Sync with system clock for precise timing.
pub fn every<F>(duration: Duration, f: F) -> Cmd
where
    F: FnOnce(Instant) -> Message + Send + 'static;
```

### Keyboard Input

```rust
/// Keyboard key event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyMsg {
    pub key_type: KeyType,
    pub runes: Vec<char>,
    pub alt: bool,
    pub paste: bool,
}

/// Key type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    // Control keys
    Null,
    Break,
    Enter,
    Backspace,
    Tab,
    Esc,
    Space,
    Delete,

    // Cursor movement
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,

    // Cursor with modifiers
    ShiftUp,
    ShiftDown,
    ShiftLeft,
    ShiftRight,
    CtrlUp,
    CtrlDown,
    CtrlLeft,
    CtrlRight,
    CtrlShiftUp,
    CtrlShiftDown,
    CtrlShiftLeft,
    CtrlShiftRight,
    AltUp,
    AltDown,
    AltLeft,
    AltRight,
    AltShiftUp,
    AltShiftDown,
    AltShiftLeft,
    AltShiftRight,
    CtrlAltUp,
    CtrlAltDown,
    CtrlAltLeft,
    CtrlAltRight,
    ShiftHome,
    ShiftEnd,
    CtrlHome,
    CtrlEnd,

    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    F13, F14, F15, F16, F17, F18, F19, F20,

    // Ctrl combinations
    CtrlAt,        // Ctrl+@
    CtrlA, CtrlB, CtrlC, CtrlD, CtrlE, CtrlF, CtrlG,
    CtrlH, CtrlI, CtrlJ, CtrlK, CtrlL, CtrlM, CtrlN,
    CtrlO, CtrlP, CtrlQ, CtrlR, CtrlS, CtrlT, CtrlU,
    CtrlV, CtrlW, CtrlX, CtrlY, CtrlZ,
    CtrlOpenBracket,
    CtrlBackslash,
    CtrlCloseBracket,
    CtrlCaret,
    CtrlUnderscore,

    // Regular character input
    Runes,
}

impl KeyMsg {
    /// Helper constructors for synthetic input in tests and simulations.
    pub fn from_type(key_type: KeyType) -> Self;
    pub fn from_char(c: char) -> Self;
    pub fn from_runes(runes: Vec<char>) -> Self;
}
```

### Mouse Input

```rust
/// Mouse event message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseMsg {
    pub x: u16,
    pub y: u16,
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
    pub action: MouseAction,
    pub button: MouseButton,
}

/// Mouse action type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseAction {
    Press,
    Release,
    Motion,
}

/// Mouse button identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    None,
    Left,
    Middle,
    Right,
    WheelUp,
    WheelDown,
    WheelLeft,
    WheelRight,
    Backward,
    Forward,
}

impl MouseMsg {
    /// Check if this is a wheel event.
    pub fn is_wheel(&self) -> bool;
}
```

### Program

```rust
/// The main program runner.
pub struct Program<M: Model> {
    model: M,
    options: ProgramOptions,
}

/// Program configuration options.
pub struct ProgramOptions {
    pub alt_screen: bool,
    pub mouse_cell_motion: bool,
    pub mouse_all_motion: bool,
    pub bracketed_paste: bool,
    pub report_focus: bool,
    pub fps: u32,
    pub without_signals: bool,
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

impl<M: Model> Program<M> {
    /// Create a new program with the given model.
    pub fn new(model: M) -> Self;

    /// Run the program and return the final model state.
    pub fn run(self) -> Result<M, Error>;

    // Builder methods
    pub fn with_alt_screen(mut self) -> Self;
    pub fn with_mouse_cell_motion(mut self) -> Self;
    pub fn with_mouse_all_motion(mut self) -> Self;
    pub fn with_fps(mut self, fps: u32) -> Self;
    pub fn with_report_focus(mut self) -> Self;
    pub fn without_bracketed_paste(mut self) -> Self;
    pub fn without_signal_handler(mut self) -> Self;
    pub fn without_catch_panics(mut self) -> Self;
    pub fn with_input<R: Read + Send + 'static>(mut self, input: R) -> Self;
    pub fn with_output<W: Write + Send + 'static>(mut self, output: W) -> Self;
    pub fn with_filter<F>(mut self, filter: F) -> Self
    where
        F: Fn(&M, &Message) -> Option<Message> + Send + 'static;
}
```

### Renderer

```rust
/// Renderer trait for display output.
pub trait Renderer: Send {
    /// Start the renderer.
    fn start(&mut self);

    /// Stop the renderer gracefully.
    fn stop(&mut self);

    /// Force stop the renderer.
    fn kill(&mut self);

    /// Write a frame to display.
    fn write(&mut self, view: &str);

    /// Request a repaint.
    fn repaint(&mut self);

    /// Clear the entire screen.
    fn clear_screen(&mut self);

    /// Enter alternate screen buffer.
    fn enter_alt_screen(&mut self);

    /// Exit alternate screen buffer.
    fn exit_alt_screen(&mut self);

    /// Show the cursor.
    fn show_cursor(&mut self);

    /// Hide the cursor.
    fn hide_cursor(&mut self);

    /// Enable mouse cell motion tracking.
    fn enable_mouse_cell_motion(&mut self);

    /// Enable mouse all motion tracking.
    fn enable_mouse_all_motion(&mut self);

    /// Disable mouse tracking.
    fn disable_mouse(&mut self);

    /// Enable bracketed paste mode.
    fn enable_bracketed_paste(&mut self);

    /// Disable bracketed paste mode.
    fn disable_bracketed_paste(&mut self);

    /// Enable focus reporting.
    fn enable_report_focus(&mut self);

    /// Disable focus reporting.
    fn disable_report_focus(&mut self);

    /// Set the window title.
    fn set_window_title(&mut self, title: &str);
}

/// Standard frame-rate based renderer.
pub struct StandardRenderer {
    output: Box<dyn Write + Send>,
    framerate: Duration,
    last_render: String,
    lines_rendered: usize,
    alt_screen_active: bool,
    cursor_hidden: bool,
    width: u16,
    height: u16,
}

impl StandardRenderer {
    /// Create a new renderer with the given output and FPS.
    pub fn new<W: Write + Send + 'static>(output: W, fps: u32) -> Self;
}
```

### Screen Commands

```rust
/// Command to clear the screen.
pub fn clear_screen() -> Cmd;

/// Command to enter alternate screen buffer.
pub fn enter_alt_screen() -> Cmd;

/// Command to exit alternate screen buffer.
pub fn exit_alt_screen() -> Cmd;

/// Command to show the cursor.
pub fn show_cursor() -> Cmd;

/// Command to hide the cursor.
pub fn hide_cursor() -> Cmd;

/// Command to enable mouse cell motion tracking.
pub fn enable_mouse_cell_motion() -> Cmd;

/// Command to enable mouse all motion tracking.
pub fn enable_mouse_all_motion() -> Cmd;

/// Command to disable mouse tracking.
pub fn disable_mouse() -> Cmd;

/// Command to enable bracketed paste mode.
pub fn enable_bracketed_paste() -> Cmd;

/// Command to disable bracketed paste mode.
pub fn disable_bracketed_paste() -> Cmd;

/// Command to set window title.
pub fn set_window_title(title: impl Into<String>) -> Cmd;

/// Command to query window size.
pub fn window_size() -> Cmd;

/// Command to enable focus reporting.
pub fn enable_report_focus() -> Cmd;

/// Command to disable focus reporting.
pub fn disable_report_focus() -> Cmd;
```

## Module Structure

```
crates/bubbletea/
├── Cargo.toml
└── src/
    ├── lib.rs         # Module exports and top-level rustdoc
    ├── command.rs     # Cmd type and combinators
    ├── key.rs         # Keyboard input types and sequence parsing
    ├── message.rs     # Message type and built-in messages
    ├── mouse.rs       # Mouse input types and parsing
    ├── program.rs     # Program runtime, event loop, options
    ├── screen.rs      # Screen control commands
    └── simulator.rs   # Deterministic model simulation utilities
```

## Dependencies

```toml
[dependencies]
crossterm = "0.29"         # Terminal control (raw mode, events)
tokio = { version = "1", features = ["rt-multi-thread", "sync", "time", "macros"], optional = true }
tokio-util = { version = "0.7", features = ["rt"], optional = true }
futures = { version = "0.3", optional = true }
parking_lot = "0.12"       # Fast mutexes

[features]
default = ["macros"]
macros = ["dep:bubbletea-macros"]
async = ["dep:tokio", "dep:tokio-util", "dep:futures"]
```

## Key Differences from Go

### Ownership and Lifetimes

Go uses interfaces with implicit ownership. Rust requires explicit ownership:

```rust
// Go: implicit reference passing
func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd)

// Rust: explicit mutable reference
fn update(&mut self, msg: Message) -> Option<Cmd>;
```

### Type-Safe Messages

Go uses `interface{}` for messages. Rust uses type erasure with downcasting:

```rust
// Go: type switch
switch msg := msg.(type) {
case KeyMsg:
    // ...
}

// Rust: downcast
if let Some(key_msg) = msg.downcast::<KeyMsg>() {
    // ...
}
```

### Command Execution

Go uses goroutines. Rust uses either:
- Blocking with thread pool (sync)
- Tokio tasks (async feature)

```rust
// Sync version
fn execute_command(cmd: Cmd, sender: Sender<Message>) {
    thread::spawn(move || {
        if let Some(msg) = cmd.execute() {
            let _ = sender.send(msg);
        }
    });
}

// Async version (with tokio feature)
async fn execute_command(cmd: Cmd, sender: Sender<Message>) {
    tokio::spawn(async move {
        if let Some(msg) = cmd.execute() {
            let _ = sender.send(msg).await;
        }
    });
}
```

### Rendering

Use crossterm for terminal manipulation instead of raw ANSI sequences:

```rust
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute,
    style::Print,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
```

## Usage Example

```rust
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, Program, quit};

struct Counter {
    count: i32,
}

struct IncrementMsg;
struct DecrementMsg;

impl Model for Counter {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if msg.is::<IncrementMsg>() {
            self.count += 1;
        } else if msg.is::<DecrementMsg>() {
            self.count -= 1;
        } else if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::CtrlC | KeyType::Esc => return Some(quit()),
                KeyType::Runes if key.runes == vec!['q'] => return Some(quit()),
                _ => {}
            }
        }
        None
    }

    fn view(&self) -> String {
        format!("Count: {}\n\nPress +/- to change, q to quit", self.count)
    }
}

fn main() -> Result<(), bubbletea::Error> {
    let model = Counter { count: 0 };
    let final_model = Program::new(model)
        .with_alt_screen()
        .run()?;
    println!("Final count: {}", final_model.count);
    Ok(())
}
```

## Getting Started Workflow

Follow this order when building a production-quality Bubble Tea app:

1. Define your domain state in a `Model` struct with no rendering concerns.
2. Define explicit message types for user intent, timer ticks, and external events.
3. Keep `update` deterministic and return `Cmd` values for all side effects.
4. Start with plain string `view` output, then layer in `lipgloss` styles.
5. Introduce `bubbles` components only where they replace common behavior (list, textinput, viewport, spinner, progress).
6. Add terminal behavior options intentionally (`with_alt_screen`, mouse tracking, bracketed paste).
7. Add tests for message transitions and quit/error paths before adding advanced features.

### Quality Gates for Bubble Tea Apps

- Verify all key paths can be driven by `Message` inputs alone (no hidden side effects).
- Verify intentional panic paths and quit behavior in tests.
- Verify resize, focus/blur, and interrupt messages do not leave stale UI state.
- Verify command-producing transitions always return quickly and do not block the update loop.
- Verify keyboard escape hatches (`q`, `esc`, `ctrl+c`) work from all major screens.

## Implementation Priority

1. **Phase 1**: Core types (Message, Cmd, Model trait)
2. **Phase 2**: Key and Mouse input handling
3. **Phase 3**: StandardRenderer with crossterm
4. **Phase 4**: Program lifecycle and event loop
5. **Phase 5**: Screen commands and helpers
6. **Phase 6**: Async support (optional tokio feature)

## Testing Strategy

1. **Unit tests**: Message handling, key parsing
2. **Integration tests**: Full program lifecycle with mock input
3. **Example programs**: Counter, todo list, text input

## ANSI Escape Sequence Reference

Key sequences to support (from `key_sequences.go`):

| Sequence | Key |
|----------|-----|
| `\x1b[A` | Up |
| `\x1b[B` | Down |
| `\x1b[C` | Right |
| `\x1b[D` | Left |
| `\x1b[1;2A` | Shift+Up |
| `\x1b[1;5A` | Ctrl+Up |
| `\x1b[H` | Home |
| `\x1b[F` | End |
| `\x1bOP` | F1 |
| ... | ... |

The full sequence map contains 500+ entries for cross-terminal compatibility.
