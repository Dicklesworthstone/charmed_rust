# charmed_rust - Charm's TUI Libraries for Rust

<div align="center">
  <img src="charmed_rust_illustration.webp" alt="charmed_rust - Charm's TUI libraries ported to idiomatic Rust" width="600">
</div>

<div align="center">

[![CI](https://github.com/Dicklesworthstone/charmed_rust/actions/workflows/ci.yml/badge.svg)](https://github.com/Dicklesworthstone/charmed_rust/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![Edition](https://img.shields.io/badge/edition-2024-blue.svg)](https://doc.rust-lang.org/edition-guide/)

</div>

A complete Rust port of [Charm's](https://charm.sh) TUI ecosystem: `bubbletea` (Elm Architecture), `lipgloss` (CSS-like styling), `bubbles` (16 components), `glamour` (Markdown), `wish` (SSH apps), and more.

[Quick Start](#quick-start) | [Components](#bubbles-components) | [Styling](#lipgloss-styling-examples) | [FAQ](#faq)

<div align="center">
<h3>Quick Install</h3>

```bash
cargo add bubbletea lipgloss bubbles
```

**Or add to your `Cargo.toml`:**

```toml
[dependencies]
bubbletea = { git = "https://github.com/Dicklesworthstone/charmed_rust" }
lipgloss = { git = "https://github.com/Dicklesworthstone/charmed_rust" }
bubbles = { git = "https://github.com/Dicklesworthstone/charmed_rust" }
```

</div>

---

## TL;DR

**The Problem**: Building terminal UIs in Rust is painful. You either wrangle raw ANSI codes, fight with complex ncurses bindings, or piece together half-baked abstractions. Meanwhile, Go developers enjoy Charm's elegant ecosystem—beautiful styles, functional architecture, and polished components—that makes TUI development actually fun.

**The Solution**: `charmed_rust` brings the entire Charm ecosystem to Rust. Same elegant APIs, same beautiful output, but with Rust's type safety, zero-cost abstractions, and fearless concurrency. Port your Go TUIs to Rust or build new ones with battle-tested patterns.

### Why Use charmed_rust?

| Feature | What It Does |
|---------|--------------|
| **Elm Architecture** | `bubbletea` provides a functional, testable TUI framework—pure `update` and `view` functions |
| **CSS-like Styling** | `lipgloss` gives you declarative styles: borders, colors, padding, margins, alignment |
| **16 Pre-built Components** | `bubbles` includes text inputs, lists, tables, spinners, viewports, file pickers |
| **Smooth Animations** | `harmonica` provides spring physics and projectile motion for fluid UIs |
| **Markdown Rendering** | `glamour` renders beautiful Markdown directly in the terminal |
| **SSH App Framework** | `wish` lets you serve TUI apps over SSH with middleware patterns |
| **100% Safe Rust** | `#![forbid(unsafe_code)]` across the entire workspace—no segfaults, ever |

---

## Quick Example

```rust
use bubbletea::{Program, Model, Message, Cmd};
use lipgloss::Style;

struct Counter { count: i32 }

impl Model for Counter {
    fn update(&mut self, msg: Message) -> Cmd {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.code {
                KeyCode::Char('+') => self.count += 1,
                KeyCode::Char('-') => self.count -= 1,
                KeyCode::Char('q') => return Cmd::quit(),
                _ => {}
            }
        }
        Cmd::none()
    }

    fn view(&self) -> String {
        let style = Style::new()
            .bold(true)
            .foreground("#FF69B4")
            .padding(1, 4);

        style.render(&format!("Count: {}", self.count))
    }
}

fn main() {
    Program::new(Counter { count: 0 }).run().unwrap();
}
```

**Terminal Output:**

```
╭────────────────╮
│  Count: 42     │
╰────────────────╯
```

---

## The Crate Ecosystem

```
┌─────────────────────────────────────────────────────────────────┐
│                        Applications                              │
│   glow (Markdown Reader)    huh (Interactive Forms)             │
└─────────────────────────────────────────────────────────────────┘
                              │
          ┌───────────────────┼───────────────────┐
          ▼                   ▼                   ▼
┌──────────────────┐ ┌──────────────────┐ ┌──────────────────┐
│    bubbles       │ │    glamour       │ │     wish         │
│ (TUI Components) │ │ (Markdown)       │ │ (SSH Framework)  │
│ - textinput      │ │ - themes         │ │ - middleware     │
│ - list, table    │ │ - word wrap      │ │ - sessions       │
│ - viewport       │ │ - syntax colors  │ │ - PTY support    │
│ - spinner        │ │                  │ │                  │
│ - filepicker     │ │                  │ │                  │
└──────────────────┘ └──────────────────┘ └──────────────────┘
          │                   │                   │
          └───────────────────┼───────────────────┘
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        bubbletea                                 │
│              (Elm Architecture TUI Framework)                    │
│   Model trait • Message passing • Commands • Event loop          │
└─────────────────────────────────────────────────────────────────┘
                              │
          ┌───────────────────┼───────────────────┐
          ▼                   ▼                   ▼
┌──────────────────┐ ┌──────────────────┐ ┌──────────────────┐
│    lipgloss      │ │    harmonica     │ │   charmed_log    │
│ (Terminal CSS)   │ │ (Animations)     │ │ (Logging)        │
│ - colors         │ │ - spring physics │ │ - text/json      │
│ - borders        │ │ - projectile     │ │ - styled output  │
│ - layout         │ │ - frame timing   │ │ - levels         │
└──────────────────┘ └──────────────────┘ └──────────────────┘
                              │
                              ▼
                     ┌──────────────────┐
                     │    crossterm     │
                     │ (Terminal I/O)   │
                     └──────────────────┘
```

### Crate Reference

| Crate | Purpose | Lines of Code |
|-------|---------|---------------|
| **harmonica** | Spring physics animations, projectile motion | ~1,100 |
| **lipgloss** | Terminal styling (colors, borders, padding, alignment) | ~3,000 |
| **charmed_log** | Structured logging with styled output | ~1,000 |
| **bubbletea** | Elm Architecture TUI framework | ~2,300 |
| **glamour** | Markdown rendering with themes | ~1,600 |
| **bubbles** | 16 pre-built TUI components | ~8,200 |
| **huh** | Interactive forms and prompts | ~2,700 |
| **wish** | SSH application framework | ~1,700 |
| **glow** | Markdown reader CLI | ~160 |

---

## Design Philosophy

### 1. Functional Core, Imperative Shell

`bubbletea` implements The Elm Architecture: your `Model` is pure data, `update` is a pure function, and `view` renders state to strings. Side effects happen through `Cmd` values—never in your business logic.

```rust
// Pure function: old state + message → new state + effects
fn update(&mut self, msg: Message) -> Cmd {
    // No I/O here, just state transitions
}
```

### 2. Composition Over Inheritance

Every component is a `Model`. Compose complex UIs by embedding models:

```rust
struct App {
    input: TextInput,    // From bubbles
    list: List<String>,  // From bubbles
    spinner: Spinner,    // From bubbles
}
```

### 3. CSS-like Styling

`lipgloss` brings web-style layout thinking to the terminal:

```rust
let style = Style::new()
    .border(Border::rounded())
    .padding(1, 2)
    .margin(1)
    .foreground("#7D56F4")
    .bold(true);
```

### 4. Zero Unsafe Code

The entire workspace uses `#![forbid(unsafe_code)]`. Memory safety isn't optional—it's guaranteed.

### 5. Go Conformance

Every crate is tested against the original Go implementation. Same inputs, same outputs. Migration from Go is seamless.

---

## How charmed_rust Compares

| Feature | charmed_rust | Go Charm | tui-rs/ratatui | ncurses-rs |
|---------|--------------|----------|----------------|------------|
| **Architecture** | Elm (functional) | Elm (functional) | Immediate mode | Imperative |
| **Styling** | CSS-like | CSS-like | Widget props | Raw attrs |
| **Type Safety** | Compile-time | Runtime | Compile-time | Minimal |
| **Async** | Native tokio | Goroutines | Manual | None |
| **Memory Safety** | Guaranteed | GC | Depends | Unsafe |
| **Components** | 16 included | 16 included | 20+ | Manual |
| **SSH Framework** | ✅ wish | ✅ wish | ❌ | ❌ |
| **Markdown** | ✅ glamour | ✅ glamour | ❌ | ❌ |

**When to use charmed_rust:**
- You want Go Charm's elegance with Rust's performance and safety
- You're porting a Go Charm app to Rust
- You prefer functional/Elm-style architecture over immediate mode
- You need SSH-served TUIs (`wish`)

**When charmed_rust might not be ideal:**
- You need maximum widget variety (ratatui has more widgets)
- You prefer immediate-mode rendering patterns
- You need ncurses compatibility for legacy systems

---

## Installation

### Add to Your Project

```toml
# Cargo.toml

[dependencies]
# Core framework
bubbletea = { git = "https://github.com/Dicklesworthstone/charmed_rust" }

# Styling (standalone, no TUI required)
lipgloss = { git = "https://github.com/Dicklesworthstone/charmed_rust" }

# Pre-built components
bubbles = { git = "https://github.com/Dicklesworthstone/charmed_rust" }

# Markdown rendering
glamour = { git = "https://github.com/Dicklesworthstone/charmed_rust" }

# Animations
harmonica = { git = "https://github.com/Dicklesworthstone/charmed_rust" }

# SSH apps
wish = { git = "https://github.com/Dicklesworthstone/charmed_rust" }

# Logging
charmed_log = { git = "https://github.com/Dicklesworthstone/charmed_rust" }

# Forms
huh = { git = "https://github.com/Dicklesworthstone/charmed_rust" }
```

### Build from Source

```bash
git clone https://github.com/Dicklesworthstone/charmed_rust.git
cd charmed_rust
cargo build --release
```

### Run the Glow Markdown Reader

```bash
cargo run -p glow -- README.md
```

### Requirements

- **Rust 1.85+** (nightly required for Rust Edition 2024)
- **Supported platforms:** Linux, macOS, Windows

---

## Quick Start

### Step 1: Create a New Project

```bash
cargo new my-tui-app
cd my-tui-app
```

### Step 2: Add Dependencies

```toml
# Cargo.toml
[dependencies]
bubbletea = { git = "https://github.com/Dicklesworthstone/charmed_rust" }
lipgloss = { git = "https://github.com/Dicklesworthstone/charmed_rust" }
```

### Step 3: Implement the Model Trait

```rust
// src/main.rs
use bubbletea::{Program, Model, Message, Cmd, KeyMsg, KeyCode};
use lipgloss::Style;

#[derive(Default)]
struct App {
    choice: usize,
    items: Vec<&'static str>,
}

impl Model for App {
    fn init(&mut self) -> Cmd {
        self.items = vec!["Option A", "Option B", "Option C"];
        Cmd::none()
    }

    fn update(&mut self, msg: Message) -> Cmd {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.choice > 0 { self.choice -= 1; }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.choice < self.items.len() - 1 { self.choice += 1; }
                }
                KeyCode::Enter => return Cmd::quit(),
                KeyCode::Char('q') => return Cmd::quit(),
                _ => {}
            }
        }
        Cmd::none()
    }

    fn view(&self) -> String {
        let title = Style::new().bold(true).render("Choose an option:\n\n");

        let items: String = self.items.iter().enumerate()
            .map(|(i, item)| {
                if i == self.choice {
                    Style::new().foreground("#FF69B4").render(&format!("> {item}\n"))
                } else {
                    format!("  {item}\n")
                }
            })
            .collect();

        format!("{title}{items}\nPress q to quit")
    }
}

fn main() {
    Program::new(App::default()).run().unwrap();
}
```

### Step 4: Run It

```bash
cargo run
```

---

## lipgloss Styling Examples

```rust
use lipgloss::{Style, Border, Color, Position};

// Basic text styling
let bold_pink = Style::new()
    .bold(true)
    .foreground("#FF69B4");
println!("{}", bold_pink.render("Hello!"));

// Box with border
let box_style = Style::new()
    .border(Border::rounded())
    .border_foreground("#7D56F4")
    .padding(1, 4)
    .margin(1);
println!("{}", box_style.render("Content in a box"));

// Adaptive colors (light/dark terminal)
let adaptive = Style::new()
    .foreground(Color::adaptive("#000000", "#FFFFFF"));

// Horizontal layout
let left = Style::new().width(20).render("Left");
let right = Style::new().width(20).render("Right");
println!("{}", lipgloss::join_horizontal(Position::Top, &[&left, &right]));

// Vertical centering
let centered = lipgloss::place(80, 24, Position::Center, Position::Center, "Centered!");
```

---

## bubbles Components

### TextInput

```rust
use bubbles::textinput::TextInput;

let mut input = TextInput::new();
input.set_placeholder("Enter your name...");
input.set_char_limit(50);
input.focus();
```

### List

```rust
use bubbles::list::{List, Item};

let items = vec![
    Item::new("Item 1", "Description 1"),
    Item::new("Item 2", "Description 2"),
];
let list = List::new(items, 10, 40);
```

### Table

```rust
use bubbles::table::{Table, Column};

let table = Table::new()
    .columns(vec![
        Column::new("Name", 20),
        Column::new("Status", 10),
    ])
    .rows(vec![
        vec!["Server 1", "Online"],
        vec!["Server 2", "Offline"],
    ]);
```

### Spinner

```rust
use bubbles::spinner::{Spinner, SpinnerType};

let spinner = Spinner::new()
    .spinner_type(SpinnerType::Dots)
    .style(Style::new().foreground("#FF69B4"));
```

### Progress

```rust
use bubbles::progress::Progress;

let progress = Progress::new()
    .width(40)
    .set_percent(0.75);
```

### Viewport (Scrollable Content)

```rust
use bubbles::viewport::Viewport;

let mut viewport = Viewport::new(80, 24);
viewport.set_content(long_text);
// viewport.line_up(1), viewport.line_down(1), etc.
```

---

## Troubleshooting

### "error: failed to select a version for `bubbletea`"

Ensure you're using the git dependency, not crates.io:

```toml
# Wrong
bubbletea = "0.1"

# Correct
bubbletea = { git = "https://github.com/Dicklesworthstone/charmed_rust" }
```

### Terminal not restoring after crash

`bubbletea` uses alternate screen mode. If your app crashes, run:

```bash
reset
# or
stty sane
```

### Colors not showing

Check your terminal supports true color:

```bash
echo $COLORTERM  # Should be "truecolor" or "24bit"
```

For 256-color fallback:

```rust
let color = Color::ansi256(205);  // Pink in 256-color
```

### "cannot find trait `Model`"

```rust
// Add the import
use bubbletea::Model;
```

### Windows terminal issues

Use Windows Terminal or enable virtual terminal processing:

```rust
// crossterm handles this automatically, but ensure you're
// running in a modern terminal, not cmd.exe
```

---

## Limitations

### What charmed_rust Doesn't Do (Yet)

- **No crates.io release**: Install from git for now (publishing planned)
- **No `wish` SSH in production**: SSH crate dependencies are beta; framework is ready
- **No mouse drag selection**: Click and scroll work; text selection requires terminal support
- **No built-in syntax highlighting**: `glamour` detects code blocks but doesn't colorize (use `syntect`)

### Known Limitations

| Capability | Current State | Notes |
|------------|---------------|-------|
| crates.io | ❌ Not yet | Install from git |
| Nightly Rust | Required | Rust 2024 edition |
| Windows SSH | ⚠️ Untested | Linux/macOS verified |
| Complex Unicode | ⚠️ Basic support | `unicode-width` handles most cases |

---

## FAQ

### Why "charmed_rust"?

It's a Rust port of Charm's libraries. Charmed = Charm + Rust = charmed_rust. Also, the results are delightfully charming.

### Can I use just lipgloss without bubbletea?

Yes! `lipgloss` is standalone:

```toml
[dependencies]
lipgloss = { git = "https://github.com/Dicklesworthstone/charmed_rust" }
```

```rust
use lipgloss::Style;
println!("{}", Style::new().bold(true).render("Just styling, no TUI"));
```

### Is this API-compatible with Go Charm?

Semantically yes, syntactically adapted for Rust. The Elm Architecture pattern is identical. Method names follow Rust conventions (`set_width` vs `Width`).

### How do I handle async operations?

Return a `Cmd` from `update`:

```rust
fn update(&mut self, msg: Message) -> Cmd {
    Cmd::perform(async {
        // Your async work
        fetch_data().await
    }, |result| Message::new(DataLoaded(result)))
}
```

### Does it work in Docker/CI?

Yes, but use `Program::new(...).without_renderer()` for headless testing.

### How do I contribute fixes?

See the "About Contributions" section below.

---

## Conformance Testing

charmed_rust includes a comprehensive conformance test suite that verifies behavior matches the original Go implementations:

```bash
# Run all conformance tests
cargo test -p charmed_conformance

# Run specific crate conformance
cargo test -p charmed_conformance test_harmonica
cargo test -p charmed_conformance test_lipgloss
cargo test -p charmed_conformance test_bubbletea
```

Test fixtures are captured from Go reference implementations in `tests/conformance/go_reference/`.

---

## About Contributions

Please don't take this the wrong way, but I do not accept outside contributions for any of my projects. I simply don't have the mental bandwidth to review anything, and it's my name on the thing, so I'm responsible for any problems it causes; thus, the risk-reward is highly asymmetric from my perspective. I'd also have to worry about other "stakeholders," which seems unwise for tools I mostly make for myself for free. Feel free to submit issues, and even PRs if you want to illustrate a proposed fix, but know I won't merge them directly. Instead, I'll have Claude or Codex review submissions via `gh` and independently decide whether and how to address them. Bug reports in particular are welcome. Sorry if this offends, but I want to avoid wasted time and hurt feelings. I understand this isn't in sync with the prevailing open-source ethos that seeks community contributions, but it's the only way I can move at this velocity and keep my sanity.

---

## License

MIT License - see [LICENSE](LICENSE) for details.

---

<div align="center">

**Built with Rust. Inspired by [Charm](https://charm.sh).**

</div>
