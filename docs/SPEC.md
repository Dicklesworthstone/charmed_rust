# Charmed Rust - Comprehensive Porting Specification

## Executive Summary

This project ports the **Charm** suite of Go terminal UI libraries to idiomatic Rust. The Charm ecosystem provides beautiful terminal applications through a cohesive set of libraries built around the Elm architecture. Our Rust port will leverage Rust's ownership model, type system, and ecosystem to create an even more robust and performant implementation.

## Source Libraries

| Library | Purpose | Go LoC (est.) | Complexity | Priority |
|---------|---------|---------------|------------|----------|
| harmonica | Spring animations | ~300 | Low | P1 |
| lipgloss | Terminal styling | ~2,500 | Medium | P1 |
| bubbletea | TUI framework (Elm arch) | ~3,000 | High | P1 |
| bubbles | Pre-built components | ~5,000 | Medium | P2 |
| log | Styled logging | ~1,200 | Low | P2 |
| glamour | Markdown rendering | ~1,500 | Medium | P3 |
| huh | Form library | ~4,000 | High | P3 |
| wish | SSH app server | ~2,000 | High | P4 |
| glow | Markdown CLI app | ~3,000 | Medium | P5 (optional) |

## Dependency Graph

```
                    ┌─────────────┐
                    │   glow      │ (CLI app)
                    └──────┬──────┘
                           │
          ┌────────────────┼────────────────┐
          │                │                │
          ▼                ▼                ▼
    ┌──────────┐    ┌──────────┐    ┌──────────┐
    │  glamour │    │   huh    │    │   wish   │
    └────┬─────┘    └────┬─────┘    └────┬─────┘
         │               │               │
         │          ┌────┴────┐          │
         │          ▼         ▼          │
         │    ┌──────────┐    │          │
         │    │ bubbles  │    │          │
         │    └────┬─────┘    │          │
         │         │          │          │
         └────┬────┴──────────┴──────────┘
              │
              ▼
        ┌──────────┐         ┌──────────┐
        │bubbletea │         │   log    │
        └────┬─────┘         └────┬─────┘
             │                    │
             └────────┬───────────┘
                      ▼
               ┌──────────┐
               │ lipgloss │
               └────┬─────┘
                    │
                    ▼
               ┌──────────┐
               │ harmonica│ (standalone)
               └──────────┘
```

## Port Order (Respecting Dependencies)

### Phase 1: Foundation
1. **harmonica** - Zero dependencies, pure algorithms
2. **lipgloss** - Core styling, depends only on terminal crates

### Phase 2: Framework Core
3. **bubbletea** - TUI framework, uses lipgloss

### Phase 3: Components & Utilities
4. **bubbles** - Components using bubbletea + lipgloss
5. **log** - Logging using lipgloss

### Phase 4: Advanced
6. **glamour** - Markdown rendering with lipgloss styling
7. **huh** - Forms using bubbletea + bubbles

### Phase 5: Infrastructure
8. **wish** - SSH server for bubbletea apps

### Phase 6: Applications (Optional)
9. **glow** - Full CLI application

---

## Library-Specific Specifications

### 1. Harmonica (Spring Animations)

**Essence:** Attempt-driven spring physics for smooth UI animations.

**Go Core Pattern:**
```go
type Spring struct {
    Damping   float64  // ζ (zeta) - damping ratio
    Mass      float64  // m - mass of the object
    Stiffness float64  // k - spring stiffness
    // ... internal state
}

func (s *Spring) Update(targetPosition float64) float64
```

**Rust Target:**
```rust
pub struct Spring {
    damping: f64,
    mass: f64,
    stiffness: f64,
    // internal state
}

impl Spring {
    pub fn update(&mut self, target: f64) -> f64;
}
```

**Key Insights:**
- Pure mathematical computation, no I/O
- Critical for smooth animations in bubbletea
- Should support `no_std` for embedded use

**Rust Advantages:**
- `Copy` semantics for efficient passing
- Const generics for compile-time spring presets
- SIMD optimization potential

---

### 2. Lipgloss (Terminal Styling)

**Essence:** Composable, declarative terminal styling with CSS-like API.

**Go Core Pattern:**
```go
type Style struct {
    // internal style rules map
}

func (s Style) Bold(v ...bool) Style
func (s Style) Foreground(c TerminalColor) Style
func (s Style) Render(strs ...string) string
```

**Rust Target:**
```rust
#[derive(Clone, Default)]
pub struct Style {
    rules: StyleRules,
}

impl Style {
    pub fn bold(self) -> Self;
    pub fn foreground(self, color: impl Into<Color>) -> Self;
    pub fn render(&self, text: &str) -> String;
}
```

**Key Concepts:**
- **Adaptive colors**: Colors that adapt to terminal background
- **Complete colors**: Full RGB vs ANSI-256 vs ANSI-16 degradation
- **Whitespace handling**: Sophisticated margin/padding model
- **Border rendering**: Box drawing with 8 border positions

**Rust Advantages:**
- Builder pattern with ownership transfer (zero-cost)
- `Into<Color>` for flexible color input
- Derive macros for style composition

**External Dependencies:**
- `crossterm` or `termion` for terminal detection
- `unicode-width` for proper character width

---

### 3. Bubbletea (TUI Framework)

**Essence:** The Elm Architecture for terminal UIs.

**Go Core Pattern:**
```go
type Model interface {
    Init() Cmd
    Update(Msg) (Model, Cmd)
    View() string
}

type Cmd func() Msg
type Msg interface{}

func NewProgram(model Model) *Program
```

**Rust Target:**
```rust
pub trait Model: Sized {
    type Message;

    fn init(&self) -> Command<Self::Message>;
    fn update(&mut self, msg: Self::Message) -> Command<Self::Message>;
    fn view(&self) -> String;
}

pub struct Program<M: Model> { /* ... */ }
```

**Key Concepts:**
- **Message passing**: Type-safe, no interface{} reflection
- **Commands**: Async operations that produce messages
- **Subscriptions**: Event streams (keyboard, mouse, resize, timers)
- **Batching**: Combine multiple commands
- **Sequencing**: Run commands in order

**Input Handling:**
- Raw terminal mode
- Mouse support (click, motion, wheel)
- Bracketed paste
- Focus tracking
- Window resize

**Rust Advantages:**
- Associated types for Message (no reflection needed)
- `async`/`await` for commands instead of goroutines
- Stronger type safety prevents message misrouting
- `crossterm` integration for cross-platform input

**Architecture Decision:**
Use `tokio` for async runtime, `crossterm` for terminal I/O.

---

### 4. Bubbles (TUI Components)

**Essence:** Pre-built, composable TUI components following the Elm architecture.

**Components to Port:**

| Component | Purpose | Complexity |
|-----------|---------|------------|
| cursor | Blinking cursor | Low |
| help | Keybind help display | Low |
| key | Keyboard input matching | Medium |
| list | Scrollable list with filtering | High |
| paginator | Page navigation | Low |
| progress | Progress bars | Low |
| spinner | Loading spinners | Low |
| stopwatch | Time tracking | Low |
| table | Data tables | High |
| textarea | Multi-line text input | High |
| textinput | Single-line text input | Medium |
| timer | Countdown timer | Low |
| viewport | Scrollable content | Medium |
| filepicker | File browser | High |
| runeutil | Unicode utilities | Low |

**Rust Pattern:**
Each component is a separate module implementing `Model`:
```rust
pub mod textinput {
    pub struct TextInput { /* ... */ }

    pub enum Message {
        KeyPress(KeyEvent),
        Paste(String),
        // ...
    }

    impl Model for TextInput {
        type Message = Message;
        // ...
    }
}
```

---

### 5. Log (Styled Logging)

**Essence:** Beautiful, structured logging with lipgloss styling.

**Go Core Pattern:**
```go
type Logger struct {
    level Level
    styles Styles
}

func (l *Logger) Info(msg interface{}, keyvals ...interface{})
func (l *Logger) With(keyvals ...interface{}) *Logger
```

**Rust Target:**
```rust
pub struct Logger {
    level: Level,
    styles: Styles,
}

impl Logger {
    pub fn info(&self, msg: impl Display);
    pub fn with<K, V>(&self, key: K, value: V) -> Logger;
}

// Also provide macros
info!(logger, "message", key = value);
```

**Rust Advantages:**
- `tracing` ecosystem integration
- Compile-time log level filtering
- Structured logging with type safety

---

### 6. Glamour (Markdown Rendering)

**Essence:** Terminal markdown rendering with lipgloss styling.

**Go Approach:** Uses `goldmark` parser with custom renderers.

**Rust Target:**
```rust
pub fn render(markdown: &str) -> Result<String, Error>;
pub fn render_with_style(markdown: &str, style: &StyleConfig) -> Result<String, Error>;
```

**Rust Implementation:**
- Use `pulldown-cmark` for parsing
- Custom renderer using lipgloss for styling
- Support for embedded code highlighting via `syntect`

**Style Themes:**
- Dark (default)
- Light
- ASCII (no unicode)
- Custom JSON/TOML themes

---

### 7. Huh (Forms)

**Essence:** Interactive form library built on bubbletea.

**Go Core Pattern:**
```go
form := huh.NewForm(
    huh.NewGroup(
        huh.NewInput().Title("Name").Value(&name),
        huh.NewSelect[string]().Title("Color").Options(/*...*/),
    ),
)
```

**Rust Target:**
```rust
let form = Form::new()
    .group(Group::new()
        .field(Input::new("Name").bind(&mut name))
        .field(Select::new("Color").options(&colors))
    );
```

**Field Types:**
- Input (text)
- Text (multi-line)
- Select (single choice)
- MultiSelect
- Confirm (yes/no)
- FilePicker

---

### 8. Wish (SSH Apps)

**Essence:** SSH server that serves bubbletea applications.

**Go Approach:** Wraps `gliderlabs/ssh` and `charmbracelet/ssh`.

**Rust Target:**
Use `russh` (pure Rust SSH implementation):
```rust
pub struct Server<M: Model> {
    model_factory: fn() -> M,
}

impl<M: Model> Server<M> {
    pub async fn listen(&self, addr: SocketAddr) -> Result<()>;
}
```

**Security Considerations:**
- Middleware for authentication
- Rate limiting
- Access control lists

---

### 9. Glow (CLI App) - Optional

**Note:** This is an application, not a library. Consider whether to include.

If included, it demonstrates:
- Full bubbletea application
- Glamour integration
- File system browsing
- Stash (cloud storage) integration

---

## Rust-Specific Design Decisions

### Error Handling
```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("terminal error: {0}")]
    Terminal(#[from] crossterm::ErrorKind),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    // ...
}

pub type Result<T> = std::result::Result<T, Error>;
```

### Async Runtime
- Use `tokio` as the async runtime
- Commands return `impl Future<Output = Message>`
- Subscriptions use `tokio::sync::mpsc` channels

### Terminal Backend
- Primary: `crossterm` (cross-platform)
- Optional: `termion` feature for Unix-only optimization

### Feature Flags
```toml
[features]
default = ["crossterm"]
termion = ["dep:termion"]
syntect = ["dep:syntect"]  # Code highlighting in glamour
ssh = ["dep:russh"]        # Wish SSH support
```

### Testing Strategy
- Unit tests for pure functions (harmonica, style computation)
- Integration tests with virtual terminal
- Property-based tests for text layout (proptest)
- Visual regression tests for style rendering

---

## Migration Path for Go Users

### Naming Conventions
| Go | Rust |
|----|------|
| `NewStyle()` | `Style::new()` / `Style::default()` |
| `style.Bold(true)` | `style.bold()` |
| `tea.NewProgram(model)` | `Program::new(model)` |
| `tea.Batch(cmds...)` | `Command::batch(cmds)` |
| `tea.Quit` | `Command::quit()` |

### Import Structure
```rust
use charmed::prelude::*;           // Common imports
use charmed::lipgloss::Style;      // Specific imports
use charmed::bubbletea::{Program, Model, Command};
use charmed::bubbles::textinput::TextInput;
```

---

## Milestones

### M1: Foundation (Target: Week 1-2)
- [ ] harmonica complete with tests
- [ ] lipgloss core styling
- [ ] Basic color support

### M2: Framework (Target: Week 3-4)
- [ ] bubbletea core loop
- [ ] Keyboard/mouse input
- [ ] Basic commands and batching

### M3: Components (Target: Week 5-6)
- [ ] textinput, textarea
- [ ] spinner, progress
- [ ] viewport, list

### M4: Ecosystem (Target: Week 7-8)
- [ ] log crate
- [ ] glamour markdown
- [ ] huh forms

### M5: Advanced (Target: Week 9-10)
- [ ] wish SSH
- [ ] Full test coverage
- [ ] Documentation
- [ ] Examples

---

## References

- [Charm Go Libraries](https://github.com/charmbracelet)
- [The Elm Architecture](https://guide.elm-lang.org/architecture/)
- [crossterm](https://github.com/crossterm-rs/crossterm)
- [rich_rust exemplar](../rich_rust/) - Python Rich port
- [beads_rust exemplar](../beads_rust/) - Go beads port
