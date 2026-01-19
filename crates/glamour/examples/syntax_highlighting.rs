//! Syntax highlighting example
//!
//! Run with: cargo run -p glamour --features syntax-highlighting --example syntax_highlighting

use glamour::{Renderer, Style, StyleConfig, render};

fn main() {
    println!("=== Glamour Syntax Highlighting Demo ===\n");

    // Example markdown with multiple code blocks
    let markdown = r#"
# Code Examples

## Rust Code

```rust
fn main() {
    let message = "Hello, World!";
    println!("{}", message);

    for i in 0..5 {
        println!("Count: {}", i);
    }
}
```

## Python Code

```python
def greet(name: str) -> str:
    """Return a greeting message."""
    return f"Hello, {name}!"

if __name__ == "__main__":
    print(greet("World"))
```

## JavaScript Code

```javascript
const greet = (name) => {
    return `Hello, ${name}!`;
};

// Arrow function example
const numbers = [1, 2, 3, 4, 5];
const doubled = numbers.map(n => n * 2);
console.log(doubled);
```

## JSON Data

```json
{
    "name": "glamour",
    "version": "0.1.0",
    "features": ["syntax-highlighting", "serde"]
}
```
"#;

    // Render with default theme
    println!("--- Default Theme (base16-ocean.dark) ---\n");
    let output = render(markdown, Style::Dark).unwrap();
    println!("{}", output);

    // Render with Solarized theme
    println!("\n--- Solarized (dark) Theme ---\n");
    let config = StyleConfig::default().syntax_theme("Solarized (dark)");
    let renderer = Renderer::new().with_style_config(config);
    let output = renderer.render(markdown);
    println!("{}", output);

    // Render with line numbers
    println!("\n--- With Line Numbers ---\n");
    let config = StyleConfig::default()
        .syntax_theme("base16-eighties.dark")
        .with_line_numbers(true);
    let renderer = Renderer::new().with_style_config(config);
    let output = renderer.render(markdown);
    println!("{}", output);
}
