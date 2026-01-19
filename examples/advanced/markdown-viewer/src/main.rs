//! Markdown Viewer Example
//!
//! This example demonstrates:
//! - Using glamour to render markdown content
//! - Scrollable viewport for navigation
//! - Keyboard controls for scrolling
//! - Different markdown styles (Dark, Light, Pink, ASCII)
//!
//! Run with: `cargo run -p example-markdown-viewer`

#![forbid(unsafe_code)]

use bubbles::viewport::Viewport;
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Program, quit};
use glamour::{Renderer, Style as GlamourStyle};
use lipgloss::Style;

/// Sample markdown content to display.
const SAMPLE_MARKDOWN: &str = r#"
# Glamour Markdown Viewer

Welcome to the **Glamour** markdown viewer example! This demonstrates rendering
rich markdown content in the terminal.

## Features

- **Bold text** and *italic text*
- ~~Strikethrough~~ text
- `inline code` formatting
- Multi-level lists

## Code Blocks

Here's an example Rust code block:

```rust
fn main() {
    println!("Hello from Glamour!");

    let numbers: Vec<i32> = (1..=5).collect();
    for n in numbers {
        println!("Number: {}", n);
    }
}
```

## Lists

### Unordered List

- First item
- Second item with more details
  - Nested item one
  - Nested item two
- Third item

### Ordered List

1. First step
2. Second step
3. Third step

## Blockquotes

> "The only way to do great work is to love what you do."
> — Steve Jobs

## Links and References

Check out the [Charm.sh](https://charm.sh) website for more terminal tools.

## Tables

| Feature | Status | Notes |
|---------|--------|-------|
| Headings | ✓ | H1-H6 supported |
| Bold/Italic | ✓ | Standard markdown |
| Code blocks | ✓ | With syntax highlighting |
| Tables | ✓ | Basic support |

## Conclusion

This demonstrates how Glamour renders markdown with beautiful terminal styling.
Press `s` to cycle through different styles!

---

*Powered by charmed_rust — Charm's TUI libraries for Rust*
"#;

/// Current style being used.
#[derive(Clone, Copy, PartialEq, Eq)]
enum CurrentStyle {
    Dark,
    Light,
    Pink,
    Ascii,
}

impl CurrentStyle {
    fn next(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::Pink,
            Self::Pink => Self::Ascii,
            Self::Ascii => Self::Dark,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
            Self::Pink => "Pink",
            Self::Ascii => "ASCII",
        }
    }

    fn to_glamour(self) -> GlamourStyle {
        match self {
            Self::Dark => GlamourStyle::Dark,
            Self::Light => GlamourStyle::Light,
            Self::Pink => GlamourStyle::Pink,
            Self::Ascii => GlamourStyle::Ascii,
        }
    }
}

/// The main application model.
#[derive(bubbletea::Model)]
struct App {
    viewport: Viewport,
    current_style: CurrentStyle,
    content: String,
}

impl App {
    /// Create a new app with rendered markdown.
    fn new() -> Self {
        let style = CurrentStyle::Dark;
        let content = render_markdown(style);

        let mut viewport = Viewport::new(80, 24);
        viewport.set_content(&content);

        Self {
            viewport,
            current_style: style,
            content,
        }
    }

    /// Re-render markdown with current style.
    fn update_content(&mut self) {
        self.content = render_markdown(self.current_style);
        self.viewport.set_content(&self.content);
    }

    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle keyboard input
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::Runes => {
                    if let Some(&ch) = key.runes.first() {
                        match ch {
                            'q' | 'Q' => return Some(quit()),
                            's' | 'S' => {
                                // Cycle through styles
                                self.current_style = self.current_style.next();
                                self.update_content();
                            }
                            _ => {}
                        }
                    }
                }
                KeyType::CtrlC | KeyType::Esc => return Some(quit()),
                _ => {}
            }
        }

        // Forward to viewport for scrolling
        self.viewport.update(&msg);
        None
    }

    fn view(&self) -> String {
        let mut output = String::new();

        // Header
        let header_style = Style::new().bold().foreground("212");
        output.push_str(&format!(
            "\n  {} (Style: {})\n",
            header_style.render("Markdown Viewer"),
            self.current_style.name()
        ));

        // Scroll indicator
        let indicator_style = Style::new().foreground("241");
        let y_offset = self.viewport.y_offset();
        let total = self.viewport.total_line_count();
        let percent = if total > 0 {
            (y_offset * 100) / total
        } else {
            0
        };
        output.push_str(&format!(
            "  {}\n\n",
            indicator_style.render(&format!("Scroll: {}%", percent))
        ));

        // Viewport content
        let content = self.viewport.view();
        for line in content.lines() {
            output.push_str(&format!("  {line}\n"));
        }

        output.push('\n');

        // Help text
        let help_style = Style::new().foreground("241");
        output.push_str(&format!(
            "  {}\n",
            help_style.render("j/k: scroll  s: change style  q: quit")
        ));

        output
    }
}

/// Render markdown with the given style.
fn render_markdown(style: CurrentStyle) -> String {
    Renderer::new()
        .with_style(style.to_glamour())
        .with_word_wrap(76)
        .render(SAMPLE_MARKDOWN)
}

fn main() -> anyhow::Result<()> {
    Program::new(App::new()).with_alt_screen().run()?;

    println!("Goodbye!");
    Ok(())
}
