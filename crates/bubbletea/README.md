# Bubbletea

An Elm Architecture TUI framework for Rust: pure `update`/`view`, command-driven
effects, and a deterministic event loop.

## Role in the charmed_rust (FrankenTUI) stack

Bubbletea is the central runtime that everything else builds on. It defines the
`Model`, `Message`, and `Cmd` abstractions and drives the event loop. Higher-level
crates such as `bubbles` (components), `huh` (forms), `wish` (SSH apps), and `glow`
(Markdown reader) are all built on top of bubbletea. `lipgloss` provides the
styling used by bubbletea-based views, while `harmonica` supplies animation
timing helpers.

## Crates.io package

Package name: `charmed-bubbletea`  
Library crate name: `bubbletea`

## What it provides

- `Model` trait for pure application state.
- `Message` and `Cmd` for event handling and effects.
- `Program` runner and event loop.
- Optional async runtime and thread-pool features.
- Derive macros via the optional `macros` feature.

## Typical usage

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

## Where to look next

- `crates/bubbletea/src/program.rs`
- `crates/bubbletea/src/model.rs`
