//! Themed Counter Example
//!
//! A simple counter application that demonstrates theming best practices:
//! - Using ThemeContext for runtime theme management
//! - Creating ThemedStyle instances that auto-update
//! - Organizing styles in a dedicated struct
//!
//! Run with: `cargo run -p example-themed-counter`

#![forbid(unsafe_code)]

use std::sync::Arc;

use bubbletea::{Cmd, KeyMsg, KeyType, Message, Program, quit};
use lipgloss::{Border, ColorSlot, ThemeContext, ThemePreset, ThemedStyle};

/// Application styles organized in one place.
///
/// This pattern makes it easy to see all styles at a glance and ensures
/// they all share the same ThemeContext for automatic updates.
struct Styles {
    title: ThemedStyle,
    counter: ThemedStyle,
    counter_positive: ThemedStyle,
    counter_negative: ThemedStyle,
    help: ThemedStyle,
    container: ThemedStyle,
}

impl Styles {
    fn new(ctx: Arc<ThemeContext>) -> Self {
        Self {
            title: ThemedStyle::new(ctx.clone())
                .foreground(ColorSlot::Primary)
                .bold(),
            counter: ThemedStyle::new(ctx.clone())
                .foreground(ColorSlot::Foreground)
                .width(20)
                .align_horizontal(lipgloss::Position::Center),
            counter_positive: ThemedStyle::new(ctx.clone())
                .foreground(ColorSlot::Success)
                .bold()
                .width(20)
                .align_horizontal(lipgloss::Position::Center),
            counter_negative: ThemedStyle::new(ctx.clone())
                .foreground(ColorSlot::Error)
                .bold()
                .width(20)
                .align_horizontal(lipgloss::Position::Center),
            help: ThemedStyle::new(ctx.clone())
                .foreground(ColorSlot::TextMuted)
                .italic(),
            container: ThemedStyle::new(ctx)
                .border(Border::rounded())
                .border_foreground(ColorSlot::Border)
                .padding((1, 3)),
        }
    }
}

/// The counter application model.
#[derive(bubbletea::Model)]
struct Counter {
    ctx: Arc<ThemeContext>,
    styles: Styles,
    count: i32,
}

impl Counter {
    fn new() -> Self {
        // Create a shared theme context - this is the source of truth for colors
        let ctx = Arc::new(ThemeContext::from_preset(ThemePreset::Dracula));
        let styles = Styles::new(ctx.clone());

        Self {
            ctx,
            styles,
            count: 0,
        }
    }

    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::Runes => {
                    if let Some(&ch) = key.runes.first() {
                        match ch {
                            '+' | '=' | 'k' => self.count += 1,
                            '-' | '_' | 'j' => self.count -= 1,
                            'r' | 'R' => self.count = 0,
                            't' | 'T' => self.cycle_theme(),
                            'q' | 'Q' => return Some(quit()),
                            _ => {}
                        }
                    }
                }
                KeyType::Up => self.count += 1,
                KeyType::Down => self.count -= 1,
                KeyType::Tab => self.cycle_theme(),
                KeyType::CtrlC | KeyType::Esc => return Some(quit()),
                _ => {}
            }
        }
        None
    }

    fn cycle_theme(&mut self) {
        // Simple dark/light toggle - in a real app you'd track a theme index
        let current_is_dark = self.ctx.current().is_dark();
        let next = if current_is_dark {
            ThemePreset::Light
        } else {
            ThemePreset::Dracula
        };

        self.ctx.set_preset(next);
        // No need to recreate styles - ThemedStyle automatically uses new colors!
    }

    fn view(&self) -> String {
        let title = self.styles.title.render("Themed Counter");

        // Choose counter style based on value
        let counter_style = if self.count > 0 {
            &self.styles.counter_positive
        } else if self.count < 0 {
            &self.styles.counter_negative
        } else {
            &self.styles.counter
        };

        let counter_display = counter_style.render(&self.count.to_string());

        let help = self
            .styles
            .help
            .render("[+/-] change | [r] reset | [t/Tab] theme | [q] quit");

        // Combine content and wrap in container
        let content = format!("{}\n\n{}\n\n{}", title, counter_display, help);
        let boxed = self.styles.container.render(&content);

        format!("\n{}\n", boxed)
    }
}

fn main() -> anyhow::Result<()> {
    let final_model = Program::new(Counter::new()).with_alt_screen().run()?;
    println!("Final count: {}", final_model.count);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_char(ch: char) -> Message {
        Message::new(KeyMsg {
            key_type: KeyType::Runes,
            runes: vec![ch],
            alt: false,
            paste: false,
        })
    }

    #[test]
    fn test_counter_starts_at_zero() {
        let counter = Counter::new();
        assert_eq!(counter.count, 0);
    }

    #[test]
    fn test_increment() {
        let mut counter = Counter::new();
        counter.update(key_char('+'));
        assert_eq!(counter.count, 1);
    }

    #[test]
    fn test_decrement() {
        let mut counter = Counter::new();
        counter.update(key_char('-'));
        assert_eq!(counter.count, -1);
    }

    #[test]
    fn test_reset() {
        let mut counter = Counter::new();
        counter.count = 42;
        counter.update(key_char('r'));
        assert_eq!(counter.count, 0);
    }

    #[test]
    fn test_view_contains_count() {
        let mut counter = Counter::new();
        counter.count = 42;
        let view = counter.view();
        assert!(view.contains("42"));
    }

    #[test]
    fn test_theme_switching_preserves_count() {
        let mut counter = Counter::new();
        counter.count = 10;
        counter.cycle_theme();
        assert_eq!(counter.count, 10);
    }
}
