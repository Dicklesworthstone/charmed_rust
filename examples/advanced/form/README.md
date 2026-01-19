# Form Example

Demonstrates building interactive multi-step forms using the huh crate with validation, multiple field types, and confirmation steps.

## Running

```bash
cargo run -p example-form
```

## Key Concepts

### Form Structure

Forms are built from groups containing fields:

```rust
let form = Form::new(vec![
    Group::new(vec![
        Box::new(Input::new().key("name").title("Name")),
        Box::new(Input::new().key("email").title("Email")),
    ]).title("Personal Info"),
    Group::new(vec![
        Box::new(Confirm::new().key("submit")),
    ]).title("Confirmation"),
]);
```

### Field Types

huh provides several field types:

- **Input**: Text input with optional validation
- **Select**: Single-choice dropdown
- **MultiSelect**: Multiple-choice checkboxes
- **Confirm**: Yes/No confirmation

### Validation

Add validation functions to fields:

```rust
Input::new()
    .key("email")
    .validate(|s| {
        if !s.contains('@') {
            Some("Invalid email".to_string())
        } else {
            None
        }
    })
```

### Running Forms

Forms implement Model so they work directly with Program:

```rust
Program::new(form).with_alt_screen().run()?;
```

## Controls

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Navigate fields |
| `Enter` | Submit field / Confirm |
| `↑` / `↓` | Navigate options |
| `Space` | Toggle selection |
| `Esc` | Cancel |

## Related Examples

- [textinput](../../basic/textinput) - Simple text input
- [todo-list](../../intermediate/todo-list) - Mode switching
