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
#[derive(PartialEq, Eq)]
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
