# Proposed Rust Charmed Architecture

> **THE DESIGN:** Synthesized from rich_rust, beads_rust, and Rust best practices.
> This document defines HOW to implement—consult EXISTING_CHARM_STRUCTURE_AND_ARCHITECTURE.md for WHAT to implement.

---

## Table of Contents

1. [Workspace Structure](#1-workspace-structure)
2. [Shared Patterns](#2-shared-patterns)
3. [Crate: harmonica](#3-crate-harmonica)
4. [Crate: lipgloss](#4-crate-lipgloss)
5. [Crate: bubbletea](#5-crate-bubbletea)
6. [Crate: charmed_log](#6-crate-charmed_log)
7. [Crate: glamour](#7-crate-glamour)
8. [Crate: bubbles](#8-crate-bubbles)
9. [Crate: huh](#9-crate-huh)
10. [Crate: wish](#10-crate-wish)
11. [Crate: glow](#11-crate-glow)
12. [Crate: charmed-wasm](#12-crate-charmed-wasm)
13. [Crate: bubbletea-macros](#13-crate-bubbletea-macros)
14. [Error Handling Strategy](#14-error-handling-strategy)
15. [Testing Strategy](#15-testing-strategy)
16. [Performance Guidelines](#16-performance-guidelines)

---

## 1. Workspace Structure

### Cargo Workspace Layout

```
charmed_rust/
├── Cargo.toml                    # Workspace root
├── rust-toolchain.toml           # Nightly toolchain
├── .cargo/
│   └── config.toml               # Cargo configuration
├── crates/
│   ├── harmonica/                # Phase 1: Spring physics
│   ├── lipgloss/                 # Phase 1: Terminal styling + theming
│   ├── bubbletea-macros/         # Proc-macros for bubbletea
│   ├── bubbletea/                # Phase 2: TUI framework
│   ├── charmed_log/              # Phase 2: Logging
│   ├── glamour/                  # Phase 3: Markdown rendering
│   ├── bubbles/                  # Phase 4: Components (16 total)
│   ├── huh/                      # Phase 5: Forms
│   ├── wish/                     # Phase 5: SSH apps
│   ├── glow/                     # Phase 5: CLI binary
│   └── charmed-wasm/             # WASM bindings for lipgloss
├── examples/                     # Cross-crate examples (basic, intermediate, advanced, themes)
├── tests/
│   └── conformance/              # Conformance testing harness (workspace member)
├── docs/                         # Documentation (19 MD files)
├── demo-website/                 # Web demo assets
├── reference/                    # Reference materials
└── scripts/                      # Build/utility scripts
```

> **Note**: Benchmarks are defined per-crate in their respective `benches/` directories
> rather than a top-level `benches/` folder.

### Root Cargo.toml

```toml
[workspace]
resolver = "2"
members = [
    "crates/harmonica",        # Smooth animations (standalone)
    "crates/lipgloss",         # Style definitions (standalone)
    "crates/charmed_log",      # Logging (uses lipgloss)
    "crates/bubbletea-macros", # Proc-macros for bubbletea
    "crates/bubbletea",        # TUI framework (uses lipgloss, harmonica)
    "crates/glamour",          # Markdown rendering (uses lipgloss)
    "crates/bubbles",          # TUI components (uses bubbletea, lipgloss)
    "crates/huh",              # Forms/prompts (uses bubbletea, lipgloss, bubbles)
    "crates/wish",             # SSH apps (uses bubbletea)
    "crates/glow",             # Markdown reader CLI (uses all)
    "crates/charmed-wasm",     # WASM bindings for web (uses lipgloss)
    "tests/conformance",       # Conformance testing harness
]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"
repository = "https://github.com/user/charmed_rust"
authors = ["Jeffrey Emanuel"]

[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
cargo = "warn"

[workspace.dependencies]
# Shared dependencies - use `dep.workspace = true` in crates
thiserror = "2"
crossterm = "0.29"
unicode-width = "0.2"
unicode-segmentation = "1.10"
bitflags = "2"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

### rust-toolchain.toml

```toml
[toolchain]
channel = "nightly"
components = ["rustfmt", "clippy"]
```

---

## 2. Shared Patterns

### 2.1 Module Organization

Each crate follows this structure:

```
crates/lipgloss/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Public API, re-exports, prelude
│   ├── color.rs         # Color types (if applicable)
│   ├── style.rs         # Main functionality
│   ├── error.rs         # Crate-specific errors
│   └── tests.rs         # Unit tests (or inline #[cfg(test)])
```

### 2.2 lib.rs Template

```rust
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

//! Brief module documentation here.

mod color;
mod style;
mod error;

// Re-exports (flat public API)
pub use color::{Color, ColorProfile, RgbColor, AnsiColor};
pub use style::{Style, StyleBuilder};
pub use error::{Error, Result};

/// Prelude for convenient imports.
pub mod prelude {
    pub use crate::color::{Color, ColorProfile};
    pub use crate::style::Style;
    pub use crate::error::Result;
}
```

### 2.3 Builder Pattern

Use consuming builders that return `Self` for fluent APIs:

```rust
#[derive(Debug, Clone, Default)]
pub struct Style {
    foreground: Option<Color>,
    background: Option<Color>,
    attributes: Attributes,
}

impl Style {
    /// Create a new empty style.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set foreground color.
    pub fn foreground(mut self, color: impl Into<Color>) -> Self {
        self.foreground = Some(color.into());
        self
    }

    /// Set background color.
    pub fn background(mut self, color: impl Into<Color>) -> Self {
        self.background = Some(color.into());
        self
    }

    /// Make text bold.
    pub fn bold(mut self) -> Self {
        self.attributes.insert(Attributes::BOLD);
        self
    }

    /// Render text with this style.
    pub fn render(&self, text: &str) -> String {
        // ...
    }
}
```

### 2.4 From/Into Conversions

Use generous `From` implementations for ergonomic APIs:

```rust
impl From<&str> for Color {
    fn from(s: &str) -> Self {
        Color::parse(s).unwrap_or_default()
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Color::Rgb(RgbColor { r, g, b })
    }
}

// Usage:
let style = Style::new()
    .foreground("#ff0000")      // From &str
    .background((0, 0, 255));   // From tuple
```

### 2.5 Bitflags for Options

Use `bitflags` for attribute sets:

```rust
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Attributes: u16 {
        const BOLD          = 1 << 0;
        const DIM           = 1 << 1;
        const ITALIC        = 1 << 2;
        const UNDERLINE     = 1 << 3;
        const BLINK         = 1 << 4;
        const REVERSE       = 1 << 5;
        const STRIKETHROUGH = 1 << 6;
    }
}
```

### 2.6 Newtype Wrappers for Clarity

Use newtypes for semantic clarity:

```rust
/// Width in terminal cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Width(pub usize);

/// Height in terminal rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Height(pub usize);

/// Padding/margin values.
#[derive(Debug, Clone, Copy, Default)]
pub struct Spacing {
    pub top: usize,
    pub right: usize,
    pub bottom: usize,
    pub left: usize,
}
```

---

## 3. Crate: harmonica

**Purpose:** Physics-based animation with damped springs and projectiles.

**Dependencies:** None (standalone, no_std compatible)

### Architecture

```rust
// src/lib.rs
#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]

mod spring;
mod projectile;

pub use spring::{Spring, fps};
pub use projectile::{Projectile, Point, Vector, GRAVITY, TERMINAL_GRAVITY};

pub mod prelude {
    pub use crate::{Spring, fps, Projectile, Point, Vector};
}
```

### Key Design Decisions

1. **No external dependencies** — Pure math, `no_std` compatible
2. **Precomputed coefficients** — Spring coefficients computed once in `new()`
3. **Immutable Spring** — `update()` returns new state, doesn't mutate
4. **Mutable Projectile** — `update()` mutates in place (physics simulation pattern)

### API Surface

```rust
// Spring usage
let spring = Spring::new(fps(60), 6.0, 0.2);
let (new_pos, new_vel) = spring.update(pos, vel, target);

// Projectile usage
let mut proj = Projectile::new(fps(60), pos, vel, TERMINAL_GRAVITY);
let new_pos = proj.update();
```

---

## 4. Crate: lipgloss

**Purpose:** Terminal styling with colors, borders, padding, and layout.

**Dependencies:** crossterm, unicode-width, bitflags

### Architecture

```
crates/lipgloss/
├── src/
│   ├── lib.rs           # Re-exports, prelude
│   ├── color.rs         # Color, AnsiColor, RgbColor, AdaptiveColor (23KB)
│   ├── style.rs         # Style struct with builder (58KB)
│   ├── border.rs        # Border definitions and rendering (14KB)
│   ├── position.rs      # Position enum, Sides struct (4KB)
│   ├── renderer.rs      # Renderer with color profile detection (5KB)
│   ├── backend.rs       # Terminal backend abstraction, crossterm integration (26KB)
│   ├── theme.rs         # Theming system - presets, slots, runtime switching (80KB)
│   └── wasm.rs          # WebAssembly bindings for browser contexts (18KB)
```

> **Note**: The theming system (`theme.rs`) is a major feature providing built-in presets
> (Dark, Light, Dracula, Nord, Catppuccin, Tokyo Night) with semantic color slots
> (Primary, Error, Success, Warning, etc.) and runtime theme switching.

### Color System (from rich_rust)

Three-layer color system for flexibility:

```rust
/// Raw RGB triplet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// ANSI color number (0-255).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnsiColor(pub u8);

/// Color profile capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorProfile {
    #[default]
    Ascii,      // No colors
    Ansi,       // 16 colors (0-15)
    Ansi256,    // 256 colors (0-255)
    TrueColor,  // 24-bit RGB
}

/// Unified color type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Color {
    None,
    Ansi(AnsiColor),
    Rgb(RgbColor),
}

/// Adaptive color that changes based on background.
#[derive(Debug, Clone)]
pub struct AdaptiveColor {
    pub light: Color,
    pub dark: Color,
}
```

### Style System

Property bitfield approach (from Go lipgloss):

```rust
use bitflags::bitflags;

bitflags! {
    /// Tracks which properties have been explicitly set.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct PropertySet: u64 {
        const FOREGROUND     = 1 << 0;
        const BACKGROUND     = 1 << 1;
        const BOLD           = 1 << 2;
        const ITALIC         = 1 << 3;
        const UNDERLINE      = 1 << 4;
        // ... 43 total properties
        const PADDING_TOP    = 1 << 20;
        const PADDING_RIGHT  = 1 << 21;
        const PADDING_BOTTOM = 1 << 22;
        const PADDING_LEFT   = 1 << 23;
        const MARGIN_TOP     = 1 << 24;
        // ... etc
    }
}

#[derive(Debug, Clone, Default)]
pub struct Style {
    props: PropertySet,           // Which properties are set
    foreground: Option<Color>,
    background: Option<Color>,
    attributes: Attributes,
    border: Option<Border>,
    padding: Spacing,
    margin: Spacing,
    width: Option<usize>,
    height: Option<usize>,
    align_horizontal: Position,
    align_vertical: Position,
    // ... other properties
}
```

### Renderer

```rust
pub struct Renderer {
    profile: ColorProfile,
    has_dark_background: bool,
}

impl Renderer {
    /// Detect color profile from environment/terminal.
    pub fn new() -> Self {
        Self {
            profile: detect_color_profile(),
            has_dark_background: detect_dark_background(),
        }
    }

    /// Render styled text.
    pub fn render(&self, style: &Style, text: &str) -> String {
        // Apply colors, attributes, padding, borders, etc.
    }
}

/// Global default renderer (thread-safe).
pub fn default_renderer() -> &'static Renderer {
    static RENDERER: OnceLock<Renderer> = OnceLock::new();
    RENDERER.get_or_init(Renderer::new)
}
```

---

## 5. Crate: bubbletea

**Purpose:** Elm-architecture TUI framework with async event loop.

**Dependencies:** lipgloss, harmonica, crossterm, tokio

### Architecture

```
crates/bubbletea/
├── src/
│   ├── lib.rs           # Re-exports, prelude, Model trait definition (6KB)
│   ├── program.rs       # Program runtime, event loop, terminal lifecycle (56KB)
│   ├── command.rs       # Cmd type, batch/sequence, built-in commands (32KB)
│   ├── message.rs       # Message trait, type-safe downcasting (4KB)
│   ├── key.rs           # KeyMsg, KeyType, rune handling (34KB)
│   ├── mouse.rs         # MouseMsg, MouseButton, MouseAction (15KB)
│   ├── screen.rs        # Terminal state, release/restore messages (5KB)
│   └── simulator.rs     # Headless testing infrastructure (12KB)
```

> **Note**: The Model trait is defined in `lib.rs` (not a separate `model.rs`).
> Window size messages are in `message.rs`. Options are part of `program.rs`.

### Core Traits and Types

```rust
/// The Model trait defines a Bubble Tea component.
/// Uses mutable reference pattern for ergonomic state updates.
pub trait Model: Send + 'static {
    /// Initialize the model, returning initial command.
    fn init(&self) -> Option<Cmd>;

    /// Update model based on message, return optional command.
    /// Uses &mut self for ergonomic state mutation.
    fn update(&mut self, msg: Message) -> Option<Cmd>;

    /// Render the model to a string.
    fn view(&self) -> String;
}

/// A type-erased message using dynamic dispatch.
pub type Message = Box<dyn std::any::Any + Send>;

/// A command represents a lazy side effect that produces a message.
pub struct Cmd { /* internal representation */ }

impl Cmd {
    /// No-op command.
    pub fn none() -> Option<Self> { None }

    /// Create a quit command.
    pub fn quit() -> Self { /* ... */ }

    /// Batch multiple commands (execute concurrently).
    pub fn batch(cmds: Vec<Self>) -> Self { /* ... */ }

    /// Sequence commands (execute in order).
    pub fn sequence(cmds: Vec<Self>) -> Self { /* ... */ }

    /// Execute a closure that returns a message.
    pub fn exec<F, M>(f: F) -> Self
    where
        F: FnOnce() -> M + Send + 'static,
        M: Send + 'static,
    { /* ... */ }
}
```

> **Design Note**: The actual implementation uses `&mut self` for update (not consuming self)
> and returns `Option<Cmd>` rather than `(Self, Cmd)`. This is more ergonomic for Rust
> while achieving the same functional goals as the Go implementation.
```

### Program Runtime

```rust
pub struct Program<M: Model> {
    model: M,
    options: ProgramOptions,
}

impl<M: Model> Program<M> {
    pub fn new(model: M) -> Self {
        Self {
            model,
            options: ProgramOptions::default(),
        }
    }

    /// Enable alternate screen buffer.
    pub fn with_alt_screen(mut self) -> Self {
        self.options.alt_screen = true;
        self
    }

    /// Enable mouse support.
    pub fn with_mouse(mut self) -> Self {
        self.options.mouse = true;
        self
    }

    /// Run the program (blocking).
    pub async fn run(self) -> Result<M, ProgramError> {
        // 1. Setup terminal (raw mode, alt screen, mouse)
        // 2. Run event loop
        // 3. Restore terminal on exit
        // 4. Return final model
    }
}

#[derive(Debug, Clone, Default)]
struct ProgramOptions {
    alt_screen: bool,
    mouse: bool,
    bracketed_paste: bool,
    ansi_compression: bool,
    report_focus: bool,
}
```

### Input Messages

```rust
/// Keyboard input message.
#[derive(Debug, Clone)]
pub struct KeyMsg {
    pub key_type: KeyType,
    pub runes: String,
    pub alt: bool,
    pub paste: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    Char(char),
    Enter,
    Tab,
    Backspace,
    Escape,
    Up, Down, Left, Right,
    Home, End,
    PageUp, PageDown,
    Insert, Delete,
    F(u8),
    // ... etc
}

/// Mouse input message.
#[derive(Debug, Clone)]
pub struct MouseMsg {
    pub x: u16,
    pub y: u16,
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
    pub action: MouseAction,
    pub button: MouseButton,
}

#[derive(Debug, Clone, Copy)]
pub enum MouseAction {
    Press,
    Release,
    Motion,
    Wheel,
}

#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    WheelUp,
    WheelDown,
    None,
}
```

---

## 6. Crate: charmed_log

**Purpose:** Structured logging with lipgloss styling.

**Dependencies:** lipgloss, tracing, tracing-subscriber

### Architecture

```rust
pub struct Logger {
    level: Level,
    formatter: Formatter,
    styles: Styles,
    prefix: Option<String>,
    time_format: Option<String>,
    report_caller: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

#[derive(Debug, Clone, Copy)]
pub enum Formatter {
    Text,
    Json,
    Logfmt,
}

#[derive(Debug, Clone)]
pub struct Styles {
    pub timestamp: Style,
    pub level: LevelStyles,
    pub caller: Style,
    pub prefix: Style,
    pub message: Style,
    pub key: Style,
    pub value: Style,
    pub separator: Style,
}

#[derive(Debug, Clone)]
pub struct LevelStyles {
    pub debug: Style,
    pub info: Style,
    pub warn: Style,
    pub error: Style,
    pub fatal: Style,
}
```

---

## 7. Crate: glamour

**Purpose:** Markdown to ANSI rendering.

**Dependencies:** lipgloss, pulldown-cmark, syntect (optional)

### Architecture

```rust
pub struct TermRenderer {
    styles: StyleConfig,
    word_wrap: usize,
    color_profile: ColorProfile,
}

impl TermRenderer {
    pub fn new(word_wrap: usize) -> Self { ... }

    pub fn with_style_config(mut self, config: StyleConfig) -> Self { ... }

    pub fn render(&self, markdown: &str) -> Result<String, RenderError> {
        // Parse markdown with pulldown-cmark
        // Apply styles per element type
        // Handle code highlighting with syntect
    }
}

/// Style configuration for each Markdown element.
pub struct StyleConfig {
    pub document: DocumentStyle,
    pub heading: HeadingStyles,
    pub code_block: CodeBlockStyle,
    pub list: ListStyle,
    pub table: TableStyle,
    pub paragraph: Style,
    pub link: LinkStyle,
    // ... etc
}

/// Built-in themes.
pub mod themes {
    pub fn dark() -> StyleConfig { ... }
    pub fn light() -> StyleConfig { ... }
    pub fn notty() -> StyleConfig { ... }  // No colors
    pub fn auto() -> StyleConfig { ... }   // Detect from terminal
}
```

---

## 8. Crate: bubbles

**Purpose:** Reusable TUI components.

**Dependencies:** bubbletea, lipgloss, harmonica

### Architecture

Each component is a separate flat module implementing the component pattern:

```
crates/bubbles/
├── src/
│   ├── lib.rs           # Re-exports all components (2.5KB)
│   ├── textinput.rs     # Single-line text input with suggestions (39KB)
│   ├── textarea.rs      # Multi-line text editor with word transforms (48KB)
│   ├── list.rs          # Filterable list with fuzzy search (34KB)
│   ├── table.rs         # Data table with headers and navigation (34KB)
│   ├── viewport.rs      # Scrollable content area (19KB)
│   ├── filepicker.rs    # File system browser (53KB)
│   ├── spinner.rs       # Loading indicator animations (11KB)
│   ├── progress.rs      # Progress bar with gradients (17KB)
│   ├── timer.rs         # Countdown timer (17KB)
│   ├── stopwatch.rs     # Elapsed time tracking (17KB)
│   ├── paginator.rs     # Pagination control (13KB)
│   ├── cursor.rs        # Text cursor with blinking (12KB)
│   ├── help.rs          # Key binding help display (23KB)
│   ├── key.rs           # Key binding definitions (7.6KB)
│   └── runeutil.rs      # Input sanitization utilities (8.4KB)
```

> **Note**: All components are flat modules (no subdirectories). The list component
> includes filtering logic inline. Total: 16 component modules.

### Component Pattern

```rust
pub struct TextInput {
    value: String,
    cursor: usize,
    placeholder: String,
    style: TextInputStyle,
    // ...
}

impl Model for TextInput {
    type Msg = TextInputMsg;

    fn init(&self) -> Cmd<Self::Msg> {
        Cmd::none()
    }

    fn update(mut self, msg: Self::Msg) -> (Self, Cmd<Self::Msg>) {
        match msg {
            TextInputMsg::Key(key) => self.handle_key(key),
            TextInputMsg::SetValue(v) => {
                self.value = v;
                (self, Cmd::none())
            }
            TextInputMsg::Focus => {
                self.focused = true;
                (self, Cmd::none())
            }
            TextInputMsg::Blur => {
                self.focused = false;
                (self, Cmd::none())
            }
        }
    }

    fn view(&self) -> String {
        // Render input with cursor, placeholder, styles
    }
}

// Builder pattern for configuration
impl TextInput {
    pub fn new() -> Self { Self::default() }
    pub fn placeholder(mut self, p: impl Into<String>) -> Self { ... }
    pub fn width(mut self, w: usize) -> Self { ... }
    pub fn style(mut self, s: TextInputStyle) -> Self { ... }
}
```

### List with Delegate Pattern

```rust
/// Trait for rendering list items.
pub trait ItemDelegate {
    type Item;

    fn height(&self) -> usize;
    fn spacing(&self) -> usize;
    fn update(&mut self, msg: ListMsg, model: &mut List<Self::Item>);
    fn render(&self, item: &Self::Item, index: usize, selected: bool) -> String;
}

pub struct List<T> {
    items: Vec<T>,
    selected: usize,
    delegate: Box<dyn ItemDelegate<Item = T>>,
    // ...
}
```

---

## 9. Crate: huh

**Purpose:** Interactive forms and prompts.

**Dependencies:** bubbletea, lipgloss, bubbles

### Architecture

```rust
/// A form containing groups of fields.
pub struct Form {
    groups: Vec<Group>,
    current_group: usize,
    theme: Theme,
    accessible: bool,
    // ...
}

/// A group of related fields (shown together).
pub struct Group {
    fields: Vec<Box<dyn Field>>,
    current_field: usize,
    title: Option<String>,
    description: Option<String>,
}

/// Trait for form fields.
pub trait Field: Model {
    fn value(&self) -> &dyn std::any::Any;
    fn key(&self) -> Option<&str>;
    fn set_focused(&mut self, focused: bool);
    fn is_valid(&self) -> bool;
    fn skip(&self) -> bool;
}

// Field types
pub struct Input { ... }       // Single-line text
pub struct Select<T> { ... }   // Single selection
pub struct MultiSelect<T> { ... }  // Multiple selection
pub struct Confirm { ... }     // Yes/No
pub struct Text { ... }        // Multi-line text
pub struct Note { ... }        // Display-only
pub struct FilePicker { ... }  // File selection
```

---

## 10. Crate: wish

**Purpose:** SSH application framework.

**Dependencies:** bubbletea, lipgloss, charmed_log, russh

### Architecture

```rust
pub struct Server {
    config: ServerConfig,
    middleware: Vec<Box<dyn Middleware>>,
}

pub struct ServerConfig {
    host_key_path: PathBuf,
    host_key_algorithms: Vec<HostKeyAlgorithm>,
    idle_timeout: Duration,
    max_auth_tries: u8,
    banner: Option<String>,
}

/// Middleware for processing SSH sessions.
pub trait Middleware: Send + Sync {
    fn handle(&self, session: &mut Session, next: &dyn Fn(&mut Session));
}

/// An SSH session with Bubble Tea integration.
pub struct Session {
    pub user: String,
    pub remote_addr: SocketAddr,
    pub pty: Option<Pty>,
    pub env: HashMap<String, String>,
    program: Option<Box<dyn Model>>,
}

// Built-in middleware
pub mod middleware {
    pub struct AccessControl { ... }
    pub struct Logging { ... }
    pub struct RateLimiter { ... }
    pub struct PanicRecovery { ... }
}
```

---

## 11. Crate: glow

**Purpose:** Markdown reader CLI application.

**Dependencies:** glamour, bubbletea, lipgloss, bubbles, clap

### Architecture

```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "glow", about = "Render markdown beautifully")]
pub struct Args {
    /// Source file, directory, or URL
    #[arg(default_value = ".")]
    source: String,

    /// Style to use (auto, dark, light, notty)
    #[arg(short, long, default_value = "auto")]
    style: String,

    /// Word wrap width (0 for terminal width)
    #[arg(short, long, default_value = "0")]
    width: usize,

    /// Use pager mode
    #[arg(short, long)]
    pager: bool,

    /// Enable local mode (no network)
    #[arg(short, long)]
    local: bool,

    /// Show all files (including hidden)
    #[arg(short, long)]
    all: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    match resolve_source(&args.source, args.local)? {
        Source::File(path) => render_file(&path, &args),
        Source::Directory(path) => run_browser(&path, &args),
        Source::Stdin => render_stdin(&args),
        Source::Url(url) => render_url(&url, &args),
    }
}
```

---

## 12. Crate: charmed-wasm

**Purpose:** WASM bindings for lipgloss styling in web contexts.

**Dependencies:** lipgloss, wasm-bindgen (plus optional wee_alloc)

### Architecture

- Expose a JS-friendly API that mirrors lipgloss style builders and layout
  helpers (join/placement utilities).
- Keep output semantics identical to native lipgloss for equivalent inputs,
  with capability detection disabled in WASM.
- Provide a minimal `version()` and `isReady()` API for diagnostics.

---

## 13. Crate: bubbletea-macros

**Purpose:** Proc-macro helpers for bubbletea to reduce Model boilerplate.

**Dependencies:** proc-macro crate only (compile-time)

### Architecture

- `#[derive(Model)]` generates a `Model` trait impl that delegates to inherent
  `init`, `update`, and `view` methods.
- Optional `#[state]` tracking emits snapshot and diff helpers for render
  decisions but must not mutate user state.

---

## 14. Error Handling Strategy

### Per-Crate Error Types

Each crate defines its own error type using `thiserror`:

```rust
// crates/lipgloss/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid color: {0}")]
    InvalidColor(String),

    #[error("invalid hex color: {0}")]
    InvalidHex(String),

    #[error("border render error: {0}")]
    BorderRender(String),
}

pub type Result<T> = std::result::Result<T, Error>;
```

### Cross-Crate Error Composition

```rust
// crates/glamour/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("style error: {0}")]
    Style(#[from] lipgloss::Error),

    #[error("parse error at line {line}: {message}")]
    Parse { line: usize, message: String },

    #[error("syntax highlighting error: {0}")]
    SyntaxHighlight(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
```

### Error Display Guidelines

1. **Lowercase first letter** — "invalid color" not "Invalid color"
2. **No trailing punctuation** — "invalid color: red" not "Invalid color: red."
3. **Include context** — "invalid color at line 5: red" not just "invalid color"
4. **Use `#[from]` for conversions** — Automatic error wrapping

---

## 15. Testing Strategy

### Unit Tests (Inline)

```rust
// In each module
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_parse_hex() {
        let color = Color::from("#ff0000");
        assert_eq!(color, Color::Rgb(RgbColor { r: 255, g: 0, b: 0 }));
    }

    #[test]
    fn test_color_parse_invalid() {
        let result = Color::try_from("invalid");
        assert!(result.is_err());
    }
}
```

### Integration Tests

```rust
// tests/lipgloss_integration.rs
use lipgloss::{Style, Color, Border};

#[test]
fn test_full_style_rendering() {
    let style = Style::new()
        .bold()
        .foreground("#ff0000")
        .border(Border::rounded())
        .padding(1);

    let output = style.render("Hello");
    // Verify ANSI codes, border characters, etc.
}
```

### Snapshot Tests (insta)

```rust
#[test]
fn test_table_render() {
    let table = Table::new(headers, rows);
    insta::assert_snapshot!(table.view());
}
```

### Property Tests (proptest)

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn spring_always_converges(
        target in -1000.0..1000.0f64,
        damping in 0.1..2.0f64,
    ) {
        let spring = Spring::new(fps(60), 6.0, damping);
        let mut pos = 0.0;
        let mut vel = 0.0;

        for _ in 0..1000 {
            (pos, vel) = spring.update(pos, vel, target);
        }

        // Should converge close to target
        prop_assert!((pos - target).abs() < 1.0);
    }
}
```

---

## 16. Performance Guidelines

### Memory Efficiency

1. **Avoid allocations in hot paths** — Reuse buffers
2. **Use `Cow<str>`** — For strings that may or may not be owned
3. **Small string optimization** — Consider `smol_str` for short strings
4. **Pool allocators** — For rendering buffers

### Rendering Optimization

1. **ANSI compression** — Minimize escape sequences
2. **Diff-based updates** — Only redraw changed lines
3. **Lazy evaluation** — Don't compute styles until render time
4. **Caching** — LRU cache for parsed colors (from rich_rust)

### Async Best Practices

1. **Spawn blocking for I/O** — `tokio::task::spawn_blocking`
2. **Bounded channels** — Prevent unbounded queue growth
3. **Graceful shutdown** — Handle SIGINT/SIGTERM properly

### Build Optimization

```toml
[profile.release]
opt-level = "z"      # Size optimization
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization
panic = "abort"      # Smaller binary
strip = true         # Remove symbols

[profile.dev]
opt-level = 1        # Slightly optimize dev builds
```

---

## Implementation Checklist

### Phase 1: Foundations ✅
- [x] harmonica — Spring physics (1.1K LOC), projectile motion, no_std compatible
- [x] lipgloss — Colors, styles, borders, theming system (5.9K LOC)

### Phase 2: Core Runtime ✅
- [x] bubbletea — Model trait, Program, async event loop (6.2K LOC)
- [x] bubbletea-macros — Proc-macro helpers for Model trait
- [x] charmed_log — Logger, formatters, styles (0.7K LOC)

### Phase 3: Rendering ✅
- [x] glamour — Markdown parser, themes, optional syntax highlighting (3.2K LOC)

### Phase 4: Components ✅
- [x] bubbles — All 16 components implemented (13.5K LOC)

### Phase 5: Applications ⚠️ (Maturing)
- [x] huh — Forms, fields, validation (5.5K LOC, ~85% parity)
- [x] wish — SSH server, middleware (3.8K LOC, ~80% parity, beta)
- [x] glow — CLI, browser, pager (2.4K LOC, ~90% parity)
- [x] charmed-wasm — WASM bindings for lipgloss (0.2K LOC)

### Additional Infrastructure ✅
- [x] tests/conformance — Conformance testing harness
- [x] examples/ — Basic, intermediate, advanced, and theme examples
- [x] docs/ — Comprehensive documentation (19 MD files)

> **Total Production Code**: ~49.5K lines across all crates.
> See `FEATURE_PARITY.md` for detailed parity status.

---

*Last updated: 2026-01-27*
