# Spinner Example

Demonstrates using the bubbles spinner component with tick-based animations and component composition.

## Running

```bash
cargo run -p example-spinner
```

## Controls

| Key | Action |
|-----|--------|
| `q` / `Q` / `Esc` / `Ctrl+C` | Quit |

## Key Concepts

### Component Composition

In the Elm Architecture, components are Models. Compose complex UIs by embedding components:

```rust
struct App {
    spinner: SpinnerModel,
    loading: bool,
}
```

### Delegating to Components

Forward relevant messages to child components in your update function:

```rust
fn update(&mut self, msg: Message) -> Option<Cmd> {
    // Handle your messages first
    if let Some(key) = msg.downcast_ref::<KeyMsg>() {
        // ...
    }

    // Then delegate to child components
    self.spinner.update(msg)
}
```

### Tick-Based Animation

Spinners animate via tick messages. Initialize the spinner to start its animation loop:

```rust
fn init(&self) -> Option<Cmd> {
    self.spinner.init()  // Starts the tick loop
}
```

### Styling with lipgloss

Apply styles to components using lipgloss:

```rust
let style = Style::new().foreground("212");  // Pink color
let spinner = SpinnerModel::with_spinner(spinners::dot()).style(style);
```

## Available Spinner Types

The `spinners` module provides many spinner animations:

- `spinners::dot()` - Braille dot pattern
- `spinners::line()` - Simple line rotation
- `spinners::mini_dot()` - Small dots
- `spinners::jump()` - Bouncing dots
- `spinners::pulse()` - Pulsing effect
- `spinners::points()` - Point animation
- `spinners::globe()` - Rotating globe
- `spinners::moon()` - Moon phases
- `spinners::monkey()` - See-no-evil monkeys

## Code Walkthrough

1. **Create styled spinner**: `SpinnerModel::with_spinner(spinners::dot()).style(style)`
2. **Initialize tick loop**: Return `self.spinner.init()` from `init()`
3. **Forward ticks**: Pass messages to spinner in `update()`
4. **Render frame**: Include `self.spinner.view()` in output

## Related Examples

- [Counter](../counter/) - Simpler example without components
- [Text Input](../textinput/) - Another bubbles component
