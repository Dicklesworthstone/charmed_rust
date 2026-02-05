# bubbletea-macros

Procedural macros that make `bubbletea` models ergonomic and efficient.

## TL;DR

**The Problem:** Writing boilerplate `Model` implementations and state tracking
logic by hand is noisy and error-prone.

**The Solution:** `bubbletea-macros` derives `Model` and generates efficient
state snapshotting for change detection.

**Why bubbletea-macros**

- **Less boilerplate**: derive `Model` directly on your struct.
- **Efficient rendering**: `#[state]` tracking enables change detection.
- **Customizable**: control equality, logging, and tracking behavior.

## Role in the charmed_rust (FrankenTUI) stack

This crate is an optional ergonomic layer for `bubbletea`. Most users should
enable the `macros` feature on `bubbletea` instead of depending on this crate
directly.

## Crates.io package

Package name: `charmed-bubbletea-macros`  
Library crate name: `bubbletea_macros`

## Installation

```toml
[dependencies]
bubbletea = { package = "charmed-bubbletea", version = "0.1.2", features = ["macros"] }
# Optional direct dependency:
bubbletea-macros = { package = "charmed-bubbletea-macros", version = "0.1.2" }
```

## Quick Start

```rust
use bubbletea::{Cmd, Message, Model};

#[derive(Model)]
struct Counter {
    #[state]
    count: i32,
}

impl Counter {
    fn init(&self) -> Option<Cmd> { None }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(&delta) = msg.downcast_ref::<i32>() {
            self.count += delta;
        }
        None
    }

    fn view(&self) -> String {
        format!("Count: {}", self.count)
    }
}
```

## State Tracking Attributes

- `#[state]`: include the field in change detection.
- `#[state(eq = "fn")]`: use a custom equality function.
- `#[state(skip)]`: ignore the field for change detection.
- `#[state(debug)]`: log changes in debug builds.

Example:

```rust
#[derive(Model)]
struct App {
    #[state]
    counter: i32,

    #[state(eq = "float_eq")]
    progress: f64,

    #[state(skip)]
    last_tick: std::time::Instant,
}
```

## Generated Methods

The macro generates a `Model` impl that delegates to your inherent methods and
adds internal helpers:

- `__snapshot_state()`
- `__state_changed()`

These support efficient re-rendering by comparing snapshots of `#[state]` fields.

## Requirements

Your struct must define these inherent methods:

- `fn init(&self) -> Option<Cmd>`
- `fn update(&mut self, msg: Message) -> Option<Cmd>`
- `fn view(&self) -> String`

## Limitations

- The macro inspects your struct at compile time and will error on missing
  methods or incompatible types.
- State comparison requires `Clone` and `PartialEq` unless you provide a custom
  equality function.

## Troubleshooting

- **Derive error about missing methods**: implement `init`, `update`, and `view`.
- **State field not tracked**: add `#[state]` to the field.
- **Unexpected re-renders**: supply a custom equality function via `eq = "fn"`.

## FAQ

**Do I need this crate to use bubbletea?**  
No. It’s optional and only provides ergonomics.

**Can I use it with generic structs?**  
Yes, generics and where-clauses are supported.

## About Contributions

Please don't take this the wrong way, but I do not accept outside contributions for any of my projects. I simply don't have the mental bandwidth to review anything, and it's my name on the thing, so I'm responsible for any problems it causes; thus, the risk-reward is highly asymmetric from my perspective. I'd also have to worry about other "stakeholders," which seems unwise for tools I mostly make for myself for free. Feel free to submit issues, and even PRs if you want to illustrate a proposed fix, but know I won't merge them directly. Instead, I'll have Claude or Codex review submissions via `gh` and independently decide whether and how to address them. Bug reports in particular are welcome. Sorry if this offends, but I want to avoid wasted time and hurt feelings. I understand this isn't in sync with the prevailing open-source ethos that seeks community contributions, but it's the only way I can move at this velocity and keep my sanity.

## License

MIT. See `LICENSE` at the repository root.
