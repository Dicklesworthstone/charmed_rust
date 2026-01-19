# Multi-Component Dashboard Example

Demonstrates combining multiple bubbles components into a cohesive dashboard application with focus management and coordinated state.

## Running

```bash
cargo run -p example-multi-component
```

## Key Concepts

### Focus Management

Track which component has focus:

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Sidebar,
    Content,
}

struct App {
    focus: Focus,
    // ...
}
```

### Focus Switching

Use Tab to switch between components:

```rust
KeyType::Tab => {
    self.focus = self.focus.toggle();
}
```

### Conditional Input Handling

Route input to the focused component:

```rust
KeyType::Up if self.focus == Focus::Sidebar => self.move_up(),
KeyType::Down if self.focus == Focus::Sidebar => self.move_down(),
_ => {
    if self.focus == Focus::Content {
        self.viewport.update(&msg);
    }
}
```

### Coordinated Layout

Combine component views side-by-side:

```rust
fn view(&self) -> String {
    let sidebar = self.render_sidebar();
    let content = self.render_content();

    // Combine line by line
    for i in 0..max_lines {
        output.push_str(&format!(
            "{} | {}\n",
            sidebar_lines[i],
            content_lines[i]
        ));
    }
}
```

### Visual Focus Indicators

Show which component is active:

```rust
let border_style = if focused {
    Style::new().foreground("212")  // Bright when focused
} else {
    Style::new().foreground("240")  // Dim when unfocused
};
```

## Controls

| Key | Action |
|-----|--------|
| `Tab` | Switch focus between sidebar and content |
| `j` / `↓` | Navigate down in focused pane |
| `k` / `↑` | Navigate up in focused pane |
| `Enter` | Select item (in sidebar) |
| `q` / `Esc` | Quit |

## Architecture

```
App
├── Focus (enum: Sidebar | Content)
├── selected (usize - current menu item)
├── viewport (Viewport - scrollable content)
└── status (display string)

View Layout:
┌─────────────────────────────────────┐
│ Header (title + status)             │
├──────────┬──────────────────────────┤
│ Sidebar  │ Content                  │
│ (menu)   │ (viewport)               │
├──────────┴──────────────────────────┤
│ Footer (help text)                  │
└─────────────────────────────────────┘
```

## Related Examples

- [viewport](../../intermediate/viewport) - Viewport basics
- [todo-list](../../intermediate/todo-list) - State management
- [form](../form) - Multi-field input handling
