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

mod app;
mod data;
mod messages;
mod pages;
mod theme;

use bubbletea::Program;

use app::{App, AppConfig};

fn main() -> anyhow::Result<()> {
    let config = AppConfig::default();
    let app = App::with_config(config);

    Program::new(app).with_alt_screen().run()?;

    Ok(())
}
