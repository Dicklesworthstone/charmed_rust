# Bubbletea Crate Specification

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
    fn init(&self) -> Option<Cmd<Self>>;

    /// Process a message and return the new model state with optional command.
    fn update(&mut self, msg: Message) -> Option<Cmd<Self>>;

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
/// A lazy IO operation that produces a message.
pub type Cmd<M> = Box<dyn FnOnce() -> Option<Message> + Send>;

/// Batch multiple commands to run concurrently (unordered).
pub fn batch<M: Model>(cmds: Vec<Option<Cmd<M>>>) -> Option<Cmd<M>>;

/// Sequence commands to run in order.
pub fn sequence<M: Model>(cmds: Vec<Option<Cmd<M>>>) -> Option<Cmd<M>>;

/// Command that signals program to quit.
pub fn quit<M: Model>() -> Cmd<M>;

/// Tick command for periodic updates.
pub fn tick<M, F>(duration: Duration, f: F) -> Cmd<M>
where
    M: Model,
    F: FnOnce(Instant) -> Message + Send + 'static;

/// Sync with system clock for precise timing.
pub fn every<M, F>(duration: Duration, f: F) -> Cmd<M>
where
    M: Model,
    F: FnOnce(Instant) -> Message + Send + 'static;
```

### Keyboard Input

```rust
/// Keyboard key event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyMsg {
    pub key: Key,
}

/// Key representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Key {
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

impl Key {
    /// Create a key from a character.
    pub fn from_char(c: char) -> Self;

    /// Create a key from a key type.
    pub fn from_type(key_type: KeyType) -> Self;

    /// Get the string representation of the key.
    pub fn string(&self) -> String;
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}
```

### Mouse Input

```rust
/// Mouse event message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MouseMsg {
    pub event: MouseEvent,
}

/// Mouse event details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MouseEvent {
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

impl MouseEvent {
    /// Check if this is a wheel event.
    pub fn is_wheel(&self) -> bool;

    /// Get string representation.
    pub fn string(&self) -> String;
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
pub fn clear_screen<M: Model>() -> Cmd<M>;

/// Command to enter alternate screen buffer.
pub fn enter_alt_screen<M: Model>() -> Cmd<M>;

/// Command to exit alternate screen buffer.
pub fn exit_alt_screen<M: Model>() -> Cmd<M>;

/// Command to show the cursor.
pub fn show_cursor<M: Model>() -> Cmd<M>;

/// Command to hide the cursor.
pub fn hide_cursor<M: Model>() -> Cmd<M>;

/// Command to enable mouse cell motion tracking.
pub fn enable_mouse_cell_motion<M: Model>() -> Cmd<M>;

/// Command to enable mouse all motion tracking.
pub fn enable_mouse_all_motion<M: Model>() -> Cmd<M>;

/// Command to disable mouse tracking.
pub fn disable_mouse<M: Model>() -> Cmd<M>;

/// Command to enable bracketed paste mode.
pub fn enable_bracketed_paste<M: Model>() -> Cmd<M>;

/// Command to disable bracketed paste mode.
pub fn disable_bracketed_paste<M: Model>() -> Cmd<M>;

/// Command to set window title.
pub fn set_window_title<M: Model>(title: impl Into<String>) -> Cmd<M>;

/// Command to query window size.
pub fn window_size<M: Model>() -> Cmd<M>;

/// Command to enable focus reporting.
pub fn enable_report_focus<M: Model>() -> Cmd<M>;

/// Command to disable focus reporting.
pub fn disable_report_focus<M: Model>() -> Cmd<M>;
```

## Module Structure

```
crates/bubbletea/
├── Cargo.toml
└── src/
    ├── lib.rs           # Module exports and documentation
    ├── model.rs         # Model trait definition
    ├── message.rs       # Message type and built-in messages
    ├── command.rs       # Cmd type and combinators
    ├── key.rs           # Keyboard input types
    ├── key_sequences.rs # ANSI escape sequence mapping
    ├── mouse.rs         # Mouse input types
    ├── program.rs       # Program struct and lifecycle
    ├── renderer.rs      # Renderer trait and StandardRenderer
    ├── screen.rs        # Screen control commands
    └── input.rs         # Terminal input reading
```

## Dependencies

```toml
[dependencies]
crossterm = "0.27"         # Terminal control (raw mode, events)
tokio = { version = "1", features = ["sync", "time", "rt-multi-thread"], optional = true }
parking_lot = "0.12"       # Fast mutexes

[features]
default = ["async"]
async = ["tokio"]
```

## Key Differences from Go

### Ownership and Lifetimes

Go uses interfaces with implicit ownership. Rust requires explicit ownership:

```rust
// Go: implicit reference passing
func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd)

// Rust: explicit mutable reference
fn update(&mut self, msg: Message) -> Option<Cmd<Self>>;
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
fn execute_command<M: Model>(cmd: Cmd<M>, sender: Sender<Message>) {
    thread::spawn(move || {
        if let Some(msg) = cmd() {
            let _ = sender.send(msg);
        }
    });
}

// Async version (with tokio feature)
async fn execute_command<M: Model>(cmd: Cmd<M>, sender: Sender<Message>) {
    tokio::spawn(async move {
        if let Some(msg) = cmd() {
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
use bubbletea::{Model, Message, Cmd, Program, KeyMsg, KeyType, quit};

struct Counter {
    count: i32,
}

struct IncrementMsg;
struct DecrementMsg;

impl Model for Counter {
    fn init(&self) -> Option<Cmd<Self>> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd<Self>> {
        if let Some(_) = msg.downcast::<IncrementMsg>() {
            self.count += 1;
        } else if let Some(_) = msg.downcast::<DecrementMsg>() {
            self.count -= 1;
        } else if let Some(key) = msg.downcast::<KeyMsg>() {
            match key.key.key_type {
                KeyType::CtrlC | KeyType::Esc => return Some(quit()),
                KeyType::Runes if key.key.runes == vec!['q'] => return Some(quit()),
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
