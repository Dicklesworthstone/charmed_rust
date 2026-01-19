# Text Input Example

Demonstrates user text input with the bubbles TextInput component, including focus management and form submission.

## Running

```bash
cargo run -p example-textinput
```

## Key Concepts

### Text Input Component

The `TextInput` component handles all the complexity of text editing:

- Character input and deletion
- Cursor positioning
- Placeholder text
- Focus state

```rust
let mut input = TextInput::new();
input.set_placeholder("Enter your name...");
input.focus();
```

### Focus Management

Text inputs need to be focused to receive input:

```rust
input.focus();    // Start receiving input
input.blur();     // Stop receiving input
input.focused();  // Check focus state
```

### Form Submission

Handle the Enter key to submit input and transition state:

```rust
KeyType::Enter => {
    if !self.submitted {
        self.name = self.input.value();
        self.submitted = true;
    } else {
        return Some(quit());
    }
}
```

### Message Routing

After handling your own logic, pass messages to the text input so it can process character input:

```rust
if !self.submitted {
    return self.input.update(msg);
}
```

### Styled Output

Use lipgloss to style the submitted name:

```rust
let style = Style::new().foreground("212");
format!("Hello, {}!", style.render(&self.name))
```

## TextInput Methods

Common TextInput methods:

| Method | Description |
|--------|-------------|
| `value()` | Get current text |
| `set_value(s)` | Set text programmatically |
| `set_placeholder(s)` | Shown when empty |
| `focus()` / `blur()` | Manage focus state |
| `set_char_limit(n)` | Maximum characters |

## Related Examples

- [counter](../counter) - Basic Elm architecture
- [spinner](../spinner) - Another bubbles component
