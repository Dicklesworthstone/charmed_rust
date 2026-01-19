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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a key message for a character.
    fn key_char(ch: char) -> Message {
        Message::new(KeyMsg {
            key_type: KeyType::Runes,
            runes: vec![ch],
            alt: false,
            paste: false,
        })
    }

    /// Create a key message for a special key.
    fn key_type(kt: KeyType) -> Message {
        Message::new(KeyMsg {
            key_type: kt,
            runes: vec![],
            alt: false,
            paste: false,
        })
    }

    #[test]
    fn test_initial_state() {
        let app = App::new();
        assert_eq!(app.current_style, CurrentStyle::Dark);
        assert!(app.viewport.total_line_count() > 0);
    }

    #[test]
    fn test_init_returns_none() {
        let app = App::new();
        assert!(app.init().is_none());
    }

    #[test]
    fn test_style_next_cycles() {
        let style = CurrentStyle::Dark;
        assert_eq!(style.next(), CurrentStyle::Light);
        assert_eq!(style.next().next(), CurrentStyle::Pink);
        assert_eq!(style.next().next().next(), CurrentStyle::Ascii);
        assert_eq!(style.next().next().next().next(), CurrentStyle::Dark);
    }

    #[test]
    fn test_style_name() {
        assert_eq!(CurrentStyle::Dark.name(), "Dark");
        assert_eq!(CurrentStyle::Light.name(), "Light");
        assert_eq!(CurrentStyle::Pink.name(), "Pink");
        assert_eq!(CurrentStyle::Ascii.name(), "ASCII");
    }

    #[test]
    fn test_style_to_glamour() {
        assert_eq!(CurrentStyle::Dark.to_glamour(), GlamourStyle::Dark);
        assert_eq!(CurrentStyle::Light.to_glamour(), GlamourStyle::Light);
        assert_eq!(CurrentStyle::Pink.to_glamour(), GlamourStyle::Pink);
        assert_eq!(CurrentStyle::Ascii.to_glamour(), GlamourStyle::Ascii);
    }

    #[test]
    fn test_style_switch_with_s() {
        let mut app = App::new();
        assert_eq!(app.current_style, CurrentStyle::Dark);

        app.update(key_char('s'));
        assert_eq!(app.current_style, CurrentStyle::Light);

        app.update(key_char('s'));
        assert_eq!(app.current_style, CurrentStyle::Pink);
    }

    #[test]
    fn test_style_switch_with_capital_s() {
        let mut app = App::new();
        assert_eq!(app.current_style, CurrentStyle::Dark);

        app.update(key_char('S'));
        assert_eq!(app.current_style, CurrentStyle::Light);
    }

    #[test]
    fn test_quit_q() {
        let mut app = App::new();
        let cmd = app.update(key_char('q'));
        assert!(cmd.is_some());
    }

    #[test]
    fn test_quit_capital_q() {
        let mut app = App::new();
        let cmd = app.update(key_char('Q'));
        assert!(cmd.is_some());
    }

    #[test]
    fn test_quit_ctrl_c() {
        let mut app = App::new();
        let cmd = app.update(key_type(KeyType::CtrlC));
        assert!(cmd.is_some());
    }

    #[test]
    fn test_quit_esc() {
        let mut app = App::new();
        let cmd = app.update(key_type(KeyType::Esc));
        assert!(cmd.is_some());
    }

    #[test]
    fn test_scroll_down_forwarded() {
        let mut app = App::new();
        let initial = app.viewport.y_offset();

        app.update(key_char('j'));
        assert_eq!(app.viewport.y_offset(), initial + 1);
    }

    #[test]
    fn test_scroll_up_forwarded() {
        let mut app = App::new();
        // First scroll down
        app.update(key_char('j'));
        app.update(key_char('j'));
        let after_down = app.viewport.y_offset();

        app.update(key_char('k'));
        assert_eq!(app.viewport.y_offset(), after_down - 1);
    }

    #[test]
    fn test_view_contains_header() {
        let app = App::new();
        let view = app.view();
        assert!(view.contains("Markdown Viewer"));
    }

    #[test]
    fn test_view_contains_style_name() {
        let app = App::new();
        let view = app.view();
        assert!(view.contains("Dark")); // Default style
    }

    #[test]
    fn test_view_contains_scroll_indicator() {
        let app = App::new();
        let view = app.view();
        assert!(view.contains("Scroll:"));
    }

    #[test]
    fn test_view_contains_help_text() {
        let app = App::new();
        let view = app.view();
        assert!(view.contains("scroll"));
        assert!(view.contains("style"));
        assert!(view.contains("quit"));
    }

    #[test]
    fn test_update_content_refreshes_viewport() {
        let mut app = App::new();
        let initial_content = app.content.clone();

        app.current_style = CurrentStyle::Ascii;
        app.update_content();

        // Content should be different with ASCII style
        assert_ne!(app.content, initial_content);
    }

    #[test]
    fn test_render_markdown() {
        let content = render_markdown(CurrentStyle::Dark);
        assert!(content.contains("Glamour")); // Part of sample content
    }

    #[test]
    fn test_regular_input_returns_none() {
        let mut app = App::new();
        let cmd = app.update(key_char('j'));
        assert!(cmd.is_none());
    }
}
