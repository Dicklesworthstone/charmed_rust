//! Todo List Example
//!
//! This example demonstrates:
//! - Complex state management with multiple items
//! - Input mode switching (browsing vs. adding items)
//! - Keyboard navigation (j/k, arrows)
//! - Toggle, add, and delete operations
//!
//! Run with: `cargo run -p example-todo-list`

#![forbid(unsafe_code)]

use bubbletea::{Cmd, KeyMsg, KeyType, Message, Program, quit};
use lipgloss::Style;

/// A single todo item.
#[derive(Clone)]
struct TodoItem {
    text: String,
    completed: bool,
}

impl TodoItem {
    fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            completed: false,
        }
    }
}

/// Input mode for the application.
#[derive(Debug, PartialEq, Eq)]
enum Mode {
    /// Browsing/navigating the list.
    Browse,
    /// Adding a new item.
    Add,
}

/// The main application model.
#[derive(bubbletea::Model)]
struct App {
    items: Vec<TodoItem>,
    cursor: usize,
    mode: Mode,
    input: String,
}

impl App {
    /// Create a new app with some sample items.
    fn new() -> Self {
        Self {
            items: vec![
                TodoItem::new("Learn Rust"),
                TodoItem::new("Build a TUI app"),
                TodoItem::new("Port Charm libraries"),
            ],
            cursor: 0,
            mode: Mode::Browse,
            input: String::new(),
        }
    }

    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match &self.mode {
                Mode::Browse => return self.update_browse(key),
                Mode::Add => return self.update_add(key),
            }
        }
        None
    }

    /// Handle input while browsing the list.
    fn update_browse(&mut self, key: &KeyMsg) -> Option<Cmd> {
        match key.key_type {
            KeyType::Runes => {
                if let Some(&ch) = key.runes.first() {
                    match ch {
                        'j' => self.cursor_down(),
                        'k' => self.cursor_up(),
                        'a' => {
                            self.mode = Mode::Add;
                            self.input.clear();
                        }
                        'd' => self.delete_current(),
                        ' ' => self.toggle_current(),
                        'q' | 'Q' => return Some(quit()),
                        _ => {}
                    }
                }
            }
            KeyType::Up => self.cursor_up(),
            KeyType::Down => self.cursor_down(),
            KeyType::Enter => self.toggle_current(),
            KeyType::CtrlC | KeyType::Esc => return Some(quit()),
            _ => {}
        }
        None
    }

    /// Handle input while adding a new item.
    fn update_add(&mut self, key: &KeyMsg) -> Option<Cmd> {
        match key.key_type {
            KeyType::Runes => {
                for &ch in &key.runes {
                    self.input.push(ch);
                }
            }
            KeyType::Space => {
                self.input.push(' ');
            }
            KeyType::Backspace => {
                self.input.pop();
            }
            KeyType::Enter => {
                if !self.input.trim().is_empty() {
                    self.items.push(TodoItem::new(self.input.clone()));
                    self.cursor = self.items.len().saturating_sub(1);
                }
                self.mode = Mode::Browse;
                self.input.clear();
            }
            KeyType::Esc => {
                self.mode = Mode::Browse;
                self.input.clear();
            }
            _ => {}
        }
        None
    }

    fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    fn cursor_down(&mut self) {
        if self.cursor < self.items.len().saturating_sub(1) {
            self.cursor += 1;
        }
    }

    fn toggle_current(&mut self) {
        if let Some(item) = self.items.get_mut(self.cursor) {
            item.completed = !item.completed;
        }
    }

    fn delete_current(&mut self) {
        if !self.items.is_empty() {
            self.items.remove(self.cursor);
            if self.cursor >= self.items.len() && self.cursor > 0 {
                self.cursor -= 1;
            }
        }
    }

    fn view(&self) -> String {
        let mut output = String::new();

        // Title
        let title_style = Style::new().bold();
        output.push_str(&format!("\n  {}\n\n", title_style.render("Todo List")));

        // Items
        if self.items.is_empty() {
            let empty_style = Style::new().foreground("241");
            output.push_str(&format!(
                "  {}\n",
                empty_style.render("No items. Press 'a' to add one.")
            ));
        } else {
            let selected_style = Style::new().foreground("212");
            let completed_style = Style::new().foreground("241").strikethrough();
            let normal_style = Style::new();

            for (i, item) in self.items.iter().enumerate() {
                let cursor = if i == self.cursor { ">" } else { " " };
                let checkbox = if item.completed { "[x]" } else { "[ ]" };

                let text = if item.completed {
                    completed_style.render(&item.text)
                } else if i == self.cursor {
                    selected_style.render(&item.text)
                } else {
                    normal_style.render(&item.text)
                };

                output.push_str(&format!("  {} {} {}\n", cursor, checkbox, text));
            }
        }

        output.push('\n');

        // Input or help
        match &self.mode {
            Mode::Add => {
                let prompt_style = Style::new().foreground("212");
                output.push_str(&format!(
                    "  {}: {}_\n",
                    prompt_style.render("New item"),
                    self.input
                ));
                output.push_str("  Press Enter to add, Esc to cancel\n");
            }
            Mode::Browse => {
                let help_style = Style::new().foreground("241");
                output.push_str(&format!(
                    "  {}\n",
                    help_style.render("j/k: move  Space/Enter: toggle  a: add  d: delete  q: quit")
                ));
            }
        }

        output
    }
}

fn main() -> anyhow::Result<()> {
    Program::new(App::new()).with_alt_screen().run()?;

    println!("Goodbye!");
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
    fn test_initial_state() {
        let app = App::new();
        assert_eq!(app.items.len(), 3);
        assert_eq!(app.cursor, 0);
        assert_eq!(app.mode, Mode::Browse);
        assert!(app.input.is_empty());
    }

    #[test]
    fn test_cursor_down_j() {
        let mut app = App::new();
        app.update(key_char('j'));
        assert_eq!(app.cursor, 1);
    }

    #[test]
    fn test_cursor_down_arrow() {
        let mut app = App::new();
        app.update(key_type(KeyType::Down));
        assert_eq!(app.cursor, 1);
    }

    #[test]
    fn test_cursor_up_k() {
        let mut app = App::new();
        app.cursor = 2;
        app.update(key_char('k'));
        assert_eq!(app.cursor, 1);
    }

    #[test]
    fn test_cursor_up_arrow() {
        let mut app = App::new();
        app.cursor = 2;
        app.update(key_type(KeyType::Up));
        assert_eq!(app.cursor, 1);
    }

    #[test]
    fn test_cursor_bounds_top() {
        let mut app = App::new();
        app.cursor = 0;
        app.update(key_char('k')); // Try to go above
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn test_cursor_bounds_bottom() {
        let mut app = App::new();
        app.cursor = 2; // Last item
        app.update(key_char('j')); // Try to go below
        assert_eq!(app.cursor, 2);
    }

    #[test]
    fn test_toggle_completion_space() {
        let mut app = App::new();
        assert!(!app.items[0].completed);
        app.update(key_char(' '));
        assert!(app.items[0].completed);
        app.update(key_char(' '));
        assert!(!app.items[0].completed);
    }

    #[test]
    fn test_toggle_completion_enter() {
        let mut app = App::new();
        assert!(!app.items[0].completed);
        app.update(key_type(KeyType::Enter));
        assert!(app.items[0].completed);
    }

    #[test]
    fn test_enter_add_mode() {
        let mut app = App::new();
        assert_eq!(app.mode, Mode::Browse);
        app.update(key_char('a'));
        assert_eq!(app.mode, Mode::Add);
    }

    #[test]
    fn test_add_item() {
        let mut app = App::new();
        let initial_count = app.items.len();

        // Enter add mode
        app.update(key_char('a'));
        assert_eq!(app.mode, Mode::Add);

        // Type text
        app.update(key_char('T'));
        app.update(key_char('e'));
        app.update(key_char('s'));
        app.update(key_char('t'));
        assert_eq!(app.input, "Test");

        // Submit
        app.update(key_type(KeyType::Enter));
        assert_eq!(app.mode, Mode::Browse);
        assert_eq!(app.items.len(), initial_count + 1);
        assert_eq!(app.items.last().unwrap().text, "Test");
        assert_eq!(app.cursor, app.items.len() - 1);
    }

    #[test]
    fn test_add_empty_item_ignored() {
        let mut app = App::new();
        let initial_count = app.items.len();

        // Enter add mode
        app.update(key_char('a'));

        // Submit without typing (or just spaces)
        app.update(key_type(KeyType::Space));
        app.update(key_type(KeyType::Enter));

        // Should not add empty item
        assert_eq!(app.items.len(), initial_count);
        assert_eq!(app.mode, Mode::Browse);
    }

    #[test]
    fn test_cancel_add_mode() {
        let mut app = App::new();
        let initial_count = app.items.len();

        // Enter add mode
        app.update(key_char('a'));
        app.update(key_char('T'));
        app.update(key_char('e'));
        app.update(key_char('s'));
        app.update(key_char('t'));

        // Cancel with Esc
        app.update(key_type(KeyType::Esc));

        assert_eq!(app.mode, Mode::Browse);
        assert_eq!(app.items.len(), initial_count);
        assert!(app.input.is_empty());
    }

    #[test]
    fn test_backspace_in_add_mode() {
        let mut app = App::new();
        app.update(key_char('a')); // Enter add mode
        app.update(key_char('A'));
        app.update(key_char('B'));
        app.update(key_char('C'));
        assert_eq!(app.input, "ABC");

        app.update(key_type(KeyType::Backspace));
        assert_eq!(app.input, "AB");
    }

    #[test]
    fn test_delete_item() {
        let mut app = App::new();
        assert_eq!(app.items.len(), 3);
        let first_text = app.items[0].text.clone();
        let second_text = app.items[1].text.clone();

        app.update(key_char('d'));

        assert_eq!(app.items.len(), 2);
        assert_eq!(app.items[0].text, second_text);
        assert_ne!(app.items[0].text, first_text);
    }

    #[test]
    fn test_delete_last_item_adjusts_cursor() {
        let mut app = App::new();
        app.cursor = 2; // Last item
        app.update(key_char('d'));
        assert_eq!(app.cursor, 1); // Should adjust to new last item
    }

    #[test]
    fn test_delete_all_items() {
        let mut app = App::new();
        app.update(key_char('d'));
        app.update(key_char('d'));
        app.update(key_char('d'));
        assert!(app.items.is_empty());
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn test_view_contains_items() {
        let app = App::new();
        let view = app.view();
        assert!(view.contains("Learn Rust"));
        assert!(view.contains("Build a TUI app"));
    }

    #[test]
    fn test_view_empty_list() {
        let mut app = App::new();
        app.items.clear();
        let view = app.view();
        assert!(view.contains("No items"));
        assert!(view.contains("Press 'a' to add"));
    }

    #[test]
    fn test_view_add_mode() {
        let mut app = App::new();
        app.update(key_char('a'));
        let view = app.view();
        assert!(view.contains("New item"));
        assert!(view.contains("Press Enter to add"));
    }

    #[test]
    fn test_quit_returns_command() {
        let mut app = App::new();
        let cmd = app.update(key_char('q'));
        assert!(cmd.is_some());
    }

    #[test]
    fn test_init_returns_none() {
        let app = App::new();
        assert!(app.init().is_none());
    }
}
