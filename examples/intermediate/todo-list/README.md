# Todo List Example

Demonstrates complex state management with a fully interactive todo list, including mode switching, keyboard navigation, and CRUD operations.

## Running

```bash
cargo run -p example-todo-list
```

## Key Concepts

### Mode-Based Input Handling

The app switches between browse and add modes:

```rust
enum Mode {
    Browse,  // Navigate and manage items
    Add,     // Text input for new items
}
```

Each mode has its own keyboard handler for clean separation of concerns.

### State Management

The model tracks cursor position, items, and current mode:

```rust
struct App {
    items: Vec<TodoItem>,
    cursor: usize,
    mode: Mode,
    input: String,
}
```

### Keyboard Navigation

Browse mode supports vim-style and arrow key navigation:

```rust
KeyType::Runes => {
    match ch {
        'j' => self.cursor_down(),
        'k' => self.cursor_up(),
        'a' => { self.mode = Mode::Add; }
        'd' => self.delete_current(),
        ' ' => self.toggle_current(),
        'q' => return Some(quit()),
        _ => {}
    }
}
KeyType::Up => self.cursor_up(),
KeyType::Down => self.cursor_down(),
```

### Styled Output

Different styles for selected, completed, and normal items:

```rust
let selected_style = Style::new().foreground("212");
let completed_style = Style::new().foreground("241").strikethrough();
```

## Controls

| Key | Action |
|-----|--------|
| `j` / `↓` | Move cursor down |
| `k` / `↑` | Move cursor up |
| `Space` / `Enter` | Toggle completion |
| `a` | Add new item |
| `d` | Delete current item |
| `q` / `Esc` | Quit |

### Add Mode

| Key | Action |
|-----|--------|
| Type | Add characters |
| `Enter` | Save item |
| `Esc` | Cancel |

## Related Examples

- [counter](../../basic/counter) - Basic state management
- [textinput](../../basic/textinput) - Text input handling
- [viewport](../viewport) - Scrollable content
