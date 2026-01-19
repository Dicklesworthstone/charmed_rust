# Counter Example

A minimal Bubble Tea application demonstrating the core Elm Architecture pattern: Model, Update, View.

## Running

```bash
cargo run -p example-counter
```

## Controls

| Key | Action |
|-----|--------|
| `+` / `=` / `k` / `Up` | Increment counter |
| `-` / `_` / `j` / `Down` | Decrement counter |
| `q` / `Q` / `Esc` / `Ctrl+C` | Quit |

## Key Concepts

### The Model Trait

The `Model` trait is the heart of Bubble Tea. It defines three methods:

1. **`init(&self) -> Option<Cmd>`** - Called once at startup. Return a command to execute, or `None`.
2. **`update(&mut self, msg: Message) -> Option<Cmd>`** - Handle messages and update state. Return a command or `None`.
3. **`view(&self) -> String`** - Render the current state as a string for display.

### Type-Erased Messages

Bubble Tea uses `Message` as a type-erased container. Check message types with `downcast_ref`:

```rust
if let Some(key) = msg.downcast_ref::<KeyMsg>() {
    // Handle keyboard input
}
```

### The Derive Macro

The `#[derive(bubbletea::Model)]` macro generates the `Model` trait implementation, delegating to your inherent methods:

```rust
#[derive(bubbletea::Model)]
struct Counter { count: i32 }

impl Counter {
    fn init(&self) -> Option<Cmd> { None }
    fn update(&mut self, msg: Message) -> Option<Cmd> { /* ... */ }
    fn view(&self) -> String { /* ... */ }
}
```

## Code Walkthrough

1. **Define state**: `Counter { count: i32 }`
2. **Handle input**: Match on `KeyMsg` to increment/decrement
3. **Render**: Format the count as a string
4. **Run**: `Program::new(model).with_alt_screen().run()`

## Related Examples

- [Spinner](../spinner/) - Adds animation with the bubbles spinner component
- [Text Input](../textinput/) - User text input handling
