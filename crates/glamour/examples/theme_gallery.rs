//! Theme gallery - preview all available syntax highlighting themes
//!
//! Run with: cargo run -p glamour --features syntax-highlighting --example theme_gallery

use glamour::{Renderer, StyleConfig};

#[cfg(feature = "syntax-highlighting")]
use glamour::syntax::SyntaxTheme;

fn main() {
    println!("=== Glamour Syntax Highlighting Theme Gallery ===\n");

    // Sample code to demonstrate highlighting
    let sample_code = r#"
```rust
// A sample Rust function to showcase syntax highlighting
fn calculate_fibonacci(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => calculate_fibonacci(n - 1) + calculate_fibonacci(n - 2),
    }
}

fn main() {
    let result = calculate_fibonacci(10);
    println!("Fibonacci(10) = {}", result);
}
```
"#;

    #[cfg(feature = "syntax-highlighting")]
    {
        // Get all available themes
        let themes = SyntaxTheme::available_themes();

        println!("Available themes: {}\n", themes.len());
        println!("{}\n", "=".repeat(60));

        for theme_name in themes {
            println!("Theme: {}", theme_name);
            println!("{}", "-".repeat(40));

            let config = StyleConfig::default().syntax_theme(theme_name);
            let renderer = Renderer::new().with_style_config(config);
            let output = renderer.render(sample_code);
            println!("{}", output);
            println!();
        }
    }

    #[cfg(not(feature = "syntax-highlighting"))]
    {
        println!("This example requires the syntax-highlighting feature.");
        println!(
            "Run with: cargo run -p glamour --features syntax-highlighting --example theme_gallery"
        );
    }
}
