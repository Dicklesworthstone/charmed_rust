#![forbid(unsafe_code)]

//! # Demo Showcase
//!
//! Flagship demonstration of all `charmed_rust` TUI capabilities.
//!
//! This showcase serves as both a feature demonstration and
//! a reference implementation for building complex TUI applications.
//!
//! ## Features Demonstrated
//!
//! - **bubbletea**: Elm architecture, event loop, commands
//! - **lipgloss**: Styling, colors, borders, layout
//! - **bubbles**: Components (viewport, list, textinput, spinner, etc.)
//! - **glamour**: Markdown rendering
//! - **harmonica**: Spring animations
//! - **huh**: Interactive forms
//! - **`charmed_log`**: Structured logging
//!
//! ## Usage
//!
//! ```bash
//! cargo run -p demo_showcase
//! ```

use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, Program, WindowSizeMsg, quit};
use lipgloss::{Border, Position, Style};

/// Application state.
struct App {
    /// Current window dimensions.
    width: usize,
    height: usize,
    /// Whether the app has received initial window size.
    ready: bool,
    /// Title style.
    title_style: Style,
    /// Box style for main content.
    box_style: Style,
    /// Hint style for footer.
    hint_style: Style,
}

impl Default for App {
    fn default() -> Self {
        Self {
            width: 80,
            height: 24,
            ready: false,
            title_style: Style::new()
                .bold()
                .foreground("#FF69B4")
                .padding_left(1)
                .padding_right(1),
            box_style: Style::new()
                .border(Border::rounded())
                .border_foreground("#7D56F4")
                .padding((1, 2)),
            hint_style: Style::new().foreground("#626262").italic(),
        }
    }
}

impl Model for App {
    fn init(&self) -> Option<Cmd> {
        Some(bubbletea::window_size())
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(size) = msg.downcast_ref::<WindowSizeMsg>() {
            self.width = size.width as usize;
            self.height = size.height as usize;
            self.ready = true;
            return None;
        }

        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::CtrlC | KeyType::Esc => return Some(quit()),
                KeyType::Runes if key.runes == ['q'] => return Some(quit()),
                _ => {}
            }
        }

        None
    }

    fn view(&self) -> String {
        if !self.ready {
            return "Loading...".to_string();
        }

        let title = self.title_style.render("Charmed Control Center");

        let content = [
            "",
            "Welcome to the Charmed Rust Demo Showcase!",
            "",
            "This application demonstrates all capabilities of the",
            "charmed_rust TUI framework - a Rust port of Charm's Go libraries.",
            "",
            "Features:",
            "  - Elm Architecture (bubbletea)",
            "  - CSS-like Styling (lipgloss)",
            "  - Pre-built Components (bubbles)",
            "  - Markdown Rendering (glamour)",
            "  - Spring Animations (harmonica)",
            "  - Interactive Forms (huh)",
            "",
            "Coming soon: Dashboard, Logs, Docs, Settings, and more!",
            "",
        ]
        .join("\n");

        let boxed_content = self.box_style.render(&content);

        let hints = self.hint_style.render("  q/Esc quit");

        let layout = lipgloss::join_vertical(Position::Center, &[&title, &boxed_content, &hints]);

        lipgloss::place(
            self.width,
            self.height,
            Position::Center,
            Position::Center,
            &layout,
        )
    }
}

fn main() -> anyhow::Result<()> {
    let app = App::default();
    Program::new(app).with_alt_screen().run()?;
    Ok(())
}
