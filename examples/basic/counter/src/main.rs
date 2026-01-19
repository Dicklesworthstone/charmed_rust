//! Simple Counter Example
//!
//! This is the most basic Bubble Tea application - a counter that you can
//! increment and decrement using keyboard keys.
//!
//! Run with: `cargo run -p example-counter`

#![forbid(unsafe_code)]

use bubbletea::{Cmd, KeyMsg, KeyType, Message, Program, quit};

/// The application model using the derive macro.
///
/// The `#[derive(Model)]` macro generates the `impl Model for Counter` that
/// delegates to the inherent methods `init`, `update`, and `view`.
#[derive(bubbletea::Model)]
struct Counter {
    count: i32,
}

impl Counter {
    /// Create a new counter starting at zero.
    fn new() -> Self {
        Self { count: 0 }
    }

    /// Initialize the model - no startup commands needed.
    fn init(&self) -> Option<Cmd> {
        None
    }

    /// Handle messages and update the model.
    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle keyboard input
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::Runes => {
                    if let Some(&ch) = key.runes.first() {
                        match ch {
                            '+' | '=' | 'k' => self.count += 1,
                            '-' | '_' | 'j' => self.count -= 1,
                            'q' | 'Q' => return Some(quit()),
                            _ => {}
                        }
                    }
                }
                KeyType::Up => self.count += 1,
                KeyType::Down => self.count -= 1,
                KeyType::CtrlC | KeyType::Esc => return Some(quit()),
                _ => {}
            }
        }
        None
    }

    /// Render the view as a string.
    fn view(&self) -> String {
        format!(
            "\n  Counter: {}\n\n  [+/-] or [k/j] to change\n  [q] or [Esc] to quit\n",
            self.count
        )
    }
}

fn main() -> anyhow::Result<()> {
    // Create and run the program
    let final_model = Program::new(Counter::new()).with_alt_screen().run()?;

    println!("Final count: {}", final_model.count);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a key message for a character
    fn key_char(ch: char) -> Message {
        Message::new(KeyMsg {
            key_type: KeyType::Runes,
            runes: vec![ch],
            alt: false,
            paste: false,
        })
    }

    /// Create a key message for a special key
    fn key_type(kt: KeyType) -> Message {
        Message::new(KeyMsg {
            key_type: kt,
            runes: vec![],
            alt: false,
            paste: false,
        })
    }

    #[test]
    fn test_counter_initial_state() {
        let counter = Counter::new();
        assert_eq!(counter.count, 0);
    }

    #[test]
    fn test_counter_increment_plus() {
        let mut counter = Counter::new();
        counter.update(key_char('+'));
        assert_eq!(counter.count, 1);
    }

    #[test]
    fn test_counter_increment_equals() {
        let mut counter = Counter::new();
        counter.update(key_char('='));
        assert_eq!(counter.count, 1);
    }

    #[test]
    fn test_counter_increment_k() {
        let mut counter = Counter::new();
        counter.update(key_char('k'));
        assert_eq!(counter.count, 1);
    }

    #[test]
    fn test_counter_increment_up_arrow() {
        let mut counter = Counter::new();
        counter.update(key_type(KeyType::Up));
        assert_eq!(counter.count, 1);
    }

    #[test]
    fn test_counter_decrement_minus() {
        let mut counter = Counter { count: 5 };
        counter.update(key_char('-'));
        assert_eq!(counter.count, 4);
    }

    #[test]
    fn test_counter_decrement_underscore() {
        let mut counter = Counter { count: 5 };
        counter.update(key_char('_'));
        assert_eq!(counter.count, 4);
    }

    #[test]
    fn test_counter_decrement_j() {
        let mut counter = Counter { count: 5 };
        counter.update(key_char('j'));
        assert_eq!(counter.count, 4);
    }

    #[test]
    fn test_counter_decrement_down_arrow() {
        let mut counter = Counter { count: 5 };
        counter.update(key_type(KeyType::Down));
        assert_eq!(counter.count, 4);
    }

    #[test]
    fn test_counter_decrement_below_zero() {
        let mut counter = Counter { count: 0 };
        counter.update(key_char('-'));
        assert_eq!(counter.count, -1);
    }

    #[test]
    fn test_counter_multiple_operations() {
        let mut counter = Counter::new();
        counter.update(key_char('+'));
        counter.update(key_char('+'));
        counter.update(key_char('-'));
        assert_eq!(counter.count, 1);
    }

    #[test]
    fn test_view_contains_count() {
        let counter = Counter { count: 42 };
        let view = counter.view();
        assert!(view.contains("42"), "View should contain count: {}", view);
    }

    #[test]
    fn test_view_contains_help_text() {
        let counter = Counter::new();
        let view = counter.view();
        assert!(view.contains("[+/-]"), "View should contain help text");
        assert!(view.contains("[q]"), "View should contain quit hint");
    }

    #[test]
    fn test_quit_returns_command() {
        let mut counter = Counter::new();
        let cmd = counter.update(key_char('q'));
        assert!(cmd.is_some(), "Quit should return a command");
    }

    #[test]
    fn test_init_returns_none() {
        let counter = Counter::new();
        assert!(counter.init().is_none(), "Init should return None");
    }
}
