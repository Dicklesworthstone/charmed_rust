# Bubbletea

An Elm-architecture TUI framework for Rust: pure `update`/`view`, command-driven
effects, and a deterministic event loop.

## TL;DR

**The Problem:** Terminal apps often mix state mutation and I/O in ways that are
hard to test and reason about.

**The Solution:** Bubbletea separates pure state transitions from effects.
`Model` + `Message` + `Cmd` gives you a testable core and a predictable runtime.

**Why Bubbletea**

- **Predictable**: pure `update` and `view` functions.
- **Composable**: child models compose cleanly.
- **Testable**: drive updates with messages and assert on views.
- **Async-ready**: optional async runtime support.

## Role in the charmed_rust (FrankenTUI) stack

Bubbletea is the runtime core. `bubbles`, `huh`, `glow`, and `wish` all build on
bubbletea’s `Model` abstraction. `lipgloss` styles the output, and `harmonica`
provides time-based motion helpers used in animations.

## Crates.io package

Package name: `charmed-bubbletea`  
Library crate name: `bubbletea`

## Installation

```toml
[dependencies]
bubbletea = { package = "charmed-bubbletea", version = "0.1.2" }
```

Enable optional features:

```toml
bubbletea = { package = "charmed-bubbletea", version = "0.1.2", features = ["async", "macros"] }
```

## Quick Start

```rust
use bubbletea::{Cmd, Message, Model, Program};

struct Counter {
    count: i32,
}

impl Model for Counter {
    fn update(&mut self, msg: Message) -> Cmd {
        if let Some(delta) = msg.downcast_ref::<i32>() {
            self.count += delta;
        }
        Cmd::none()
    }

    fn view(&self) -> String {
        format!("Count: {}", self.count)
    }
}

fn main() {
    Program::new(Counter { count: 0 }).run().unwrap();
}
```

## Core Concepts

- **Model**: your application state and logic.
- **Message**: events that drive state transitions.
- **Cmd**: effect descriptions (I/O, timers, async work).
- **Program**: event loop runner.

## API Overview

Key types live in `crates/bubbletea/src/`:

- `model.rs`: `Model` trait and lifecycle hooks.
- `message.rs`: `Message` type and downcasting helpers.
- `cmd.rs`: command helpers (`Cmd::none`, `Cmd::batch`, `Cmd::quit`).
- `program.rs`: event loop and runner configuration.
- `key.rs`: key event types and helpers.

## Feature Flags

- `macros`: derive macros for `Model` via `bubbletea-macros`.
- `async`: async runtime integration (tokio-based).
- `thread-pool`: batch commands with rayon.

## Testing Guidance

Test `update` and `view` directly:

```rust
let mut app = Counter { count: 0 };
app.update(Message::new(1));
assert_eq!(app.view(), "Count: 1");
```

For headless runs:

```rust
let model = Program::new(app).without_renderer().run().unwrap();
```

## Troubleshooting

- **No input events**: ensure the program is running in a tty.
- **Output garbled**: disable alternate screen or set a compatible `TERM`.
- **Async commands not running**: enable the `async` feature and use `tokio`.

## Limitations

- Terminal rendering is bound by terminal capabilities.
- High-frequency animation depends on your event loop tick rate.

## FAQ

**Can I use bubbletea without lipgloss?**  
Yes. Bubbletea renders strings; you can render them any way you want.

**Do I need the macros crate?**  
No. It’s optional and only adds ergonomics.

**Is async required?**  
No. Synchronous apps work out of the box.

## About Contributions

Please don't take this the wrong way, but I do not accept outside contributions for any of my projects. I simply don't have the mental bandwidth to review anything, and it's my name on the thing, so I'm responsible for any problems it causes; thus, the risk-reward is highly asymmetric from my perspective. I'd also have to worry about other "stakeholders," which seems unwise for tools I mostly make for myself for free. Feel free to submit issues, and even PRs if you want to illustrate a proposed fix, but know I won't merge them directly. Instead, I'll have Claude or Codex review submissions via `gh` and independently decide whether and how to address them. Bug reports in particular are welcome. Sorry if this offends, but I want to avoid wasted time and hurt feelings. I understand this isn't in sync with the prevailing open-source ethos that seeks community contributions, but it's the only way I can move at this velocity and keep my sanity.

## License

MIT. See `LICENSE` at the repository root.
