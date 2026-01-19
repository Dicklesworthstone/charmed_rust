# Viewport Example

Demonstrates the bubbles Viewport component for displaying scrollable content with keyboard navigation.

## Running

```bash
cargo run -p example-viewport
```

## Key Concepts

### Viewport Component

The Viewport handles scrollable content with built-in keyboard support:

```rust
let mut viewport = Viewport::new(80, 20);
viewport.set_content(SAMPLE_CONTENT);
```

### Message Forwarding

The Viewport handles its own scroll messages when you forward updates:

```rust
fn update(&mut self, msg: Message) -> Option<Cmd> {
    // Handle app-specific keys first
    if let Some(key) = msg.downcast_ref::<KeyMsg>() {
        if matches!(key.key_type, KeyType::Runes) {
            if let Some('q' | 'Q') = key.runes.first() {
                return Some(quit());
            }
        }
    }

    // Forward to viewport for scroll handling
    self.viewport.update(&msg);
    None
}
```

### Scroll Position

Track and display scroll position using viewport methods:

```rust
let y_offset = self.viewport.y_offset();
let at_bottom = self.viewport.at_bottom();
let total_lines = self.viewport.total_line_count();
```

### Styled Header

Use lipgloss for consistent UI styling:

```rust
let header_style = Style::new().bold().foreground("212");
let indicator_style = Style::new().foreground("241");
```

## Controls

| Key | Action |
|-----|--------|
| `j` / `↓` | Scroll down one line |
| `k` / `↑` | Scroll up one line |
| `f` / `PgDn` | Page down |
| `b` / `PgUp` | Page up |
| `g` | Go to top |
| `G` | Go to bottom |
| `q` / `Esc` | Quit |

## Viewport Methods

| Method | Description |
|--------|-------------|
| `set_content(s)` | Set the scrollable content |
| `y_offset()` | Current scroll position |
| `at_top()` / `at_bottom()` | Check scroll boundaries |
| `total_line_count()` | Total lines in content |
| `view()` | Render visible portion |

## Related Examples

- [todo-list](../todo-list) - List navigation
- [progress](../progress) - Component composition
