# bubbletea-macros

Procedural macros for the [bubbletea](../bubbletea) TUI framework.

## Features

- **`#[derive(Model)]`** - Automatically implement the `Model` trait
- **`#[state]`** - Track fields for optimized change detection
- **Custom equality** - Use custom comparison functions with `#[state(eq = "fn")]`
- **Debug logging** - Log state changes with `#[state(debug)]`
- **Generic support** - Works with generic structs

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
bubbletea = "0.1"
bubbletea-macros = "0.1"
```

Or use the re-export from bubbletea:

```rust
use bubbletea::Model;  // Re-exported from bubbletea-macros
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
    fn init(&self) -> Option<Cmd> {
        None
    }

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

## State Tracking

The `#[state]` attribute enables optimized re-rendering:

```rust
#[derive(Model)]
struct App {
    #[state]                           // Basic tracking (uses PartialEq)
    counter: i32,

    #[state(eq = "float_approx_eq")]   // Custom equality function
    progress: f64,

    #[state(skip)]                     // Excluded from tracking
    last_tick: std::time::Instant,

    #[state(debug)]                    // Log changes (debug builds)
    selected: usize,

    cache: String,                     // Not tracked (no #[state])
}
```

## How It Works

The derive macro generates:

1. **Model trait implementation** - Delegates to your inherent `init`, `update`, `view` methods
2. **State snapshot struct** - Stores clones of `#[state]` fields for comparison
3. **Change detection methods** - `__snapshot_state()` and `__state_changed()`

## Requirements

Your struct must implement these inherent methods:

| Method | Signature |
|--------|-----------|
| `init` | `fn init(&self) -> Option<Cmd>` |
| `update` | `fn update(&mut self, msg: Message) -> Option<Cmd>` |
| `view` | `fn view(&self) -> String` |

## Generic Structs

The macro fully supports generic type parameters and where clauses:

```rust
#[derive(bubbletea::Model)]
struct DataView<T>
where
    T: std::fmt::Display + Clone + Send + 'static,
{
    #[state]
    data: T,
}

impl<T> DataView<T>
where
    T: std::fmt::Display + Clone + PartialEq + Send + 'static,
{
    fn init(&self) -> Option<Cmd> { None }
    fn update(&mut self, _msg: Message) -> Option<Cmd> { None }
    fn view(&self) -> String { format!("{}", self.data) }
}
```

## Migration from Manual Implementation

**Before (manual trait implementation):**

```rust
struct Counter { count: i32 }

impl bubbletea::Model for Counter {
    fn init(&self) -> Option<Cmd> { None }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(&n) = msg.downcast_ref::<i32>() {
            self.count += n;
        }
        None
    }

    fn view(&self) -> String {
        format!("Count: {}", self.count)
    }
}
```

**After (using derive macro):**

```rust
#[derive(bubbletea::Model)]
struct Counter { count: i32 }

impl Counter {
    fn init(&self) -> Option<Cmd> { None }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(&n) = msg.downcast_ref::<i32>() {
            self.count += n;
        }
        None
    }

    fn view(&self) -> String {
        format!("Count: {}", self.count)
    }
}
```

The key benefits are:
- Use `#[state]` for optimized rendering
- Cleaner separation between struct definition and behavior
- Type-safe state change detection

## Limitations

- Only works with named structs (not enums, unions, or tuple structs)
- `#[state]` fields must implement `Clone` and `PartialEq` (unless using custom eq)

## Troubleshooting

### Error: "expected method `init`, found none"

Ensure your struct has all three required inherent methods with the exact signatures:
- `fn init(&self) -> Option<Cmd>`
- `fn update(&mut self, msg: Message) -> Option<Cmd>`
- `fn view(&self) -> String`

### Error: "conflicting implementations of trait `Model`"

You can't both derive `Model` and implement it manually. Remove one or the other.

### Error: "the trait bound `T: Clone` is not satisfied"

Fields marked with `#[state]` must implement `Clone` for snapshot generation.
Add the bound to your generic, or remove `#[state]` from the field.

### Error: "the trait bound `T: PartialEq` is not satisfied"

Fields marked with `#[state]` must implement `PartialEq` for change detection.
Either add the bound, use `#[state(eq = "custom_fn")]`, or remove `#[state]`.

## Examples

See the [examples directory](../../examples/) for complete runnable examples:
- `example-counter` - Basic counter with increment/decrement
- `example-todo-list` - Complex state with input modes
- `example-progress` - Timer-based progress with tick commands

## Documentation

See the [API documentation](https://docs.rs/bubbletea-macros) for complete details.

## License

MIT License - see [LICENSE](../../LICENSE)
