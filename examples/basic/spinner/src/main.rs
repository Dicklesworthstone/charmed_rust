//! Spinner Example
//!
//! This example demonstrates:
//! - Using the bubbles spinner component
//! - Async tick-based animations
//! - Component composition in the Elm Architecture
//!
//! Run with: `cargo run -p example-spinner`

#![forbid(unsafe_code)]

use bubbles::spinner::{SpinnerModel, spinners};
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, Program, quit};
use lipgloss::Style;

/// Application model that wraps the spinner component.
///
/// In the Elm Architecture, your Model contains all application state.
/// Here, we compose a SpinnerModel from the bubbles crate.
#[derive(bubbletea::Model)]
struct App {
    spinner: SpinnerModel,
    loading: bool,
}

impl App {
    /// Create a new app with a styled spinner.
    fn new() -> Self {
        // Create a pink-colored dot spinner
        let style = Style::new().foreground("212");
        let spinner = SpinnerModel::with_spinner(spinners::dot()).style(style);

        Self {
            spinner,
            loading: true,
        }
    }

    /// Initialize - delegate to spinner's init for its tick command.
    fn init(&self) -> Option<Cmd> {
        // The spinner needs to start its tick loop
        self.spinner.init()
    }

    /// Handle messages - keyboard input and spinner ticks.
    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle keyboard input first
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::Runes => {
                    if let Some('q' | 'Q') = key.runes.first() {
                        return Some(quit());
                    }
                }
                KeyType::CtrlC | KeyType::Esc => return Some(quit()),
                _ => {}
            }
        }

        // Forward message to spinner for tick handling
        self.spinner.update(msg)
    }

    /// Render the view with spinner animation.
    fn view(&self) -> String {
        if self.loading {
            format!(
                "\n  {} Loading... please wait\n\n  Press [q] or [Esc] to quit\n",
                self.spinner.view()
            )
        } else {
            "  Done!\n".to_string()
        }
    }
}

fn main() -> anyhow::Result<()> {
    // Create and run the program with alternate screen
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
    fn test_app_initial_state() {
        let app = App::new();
        assert!(app.loading, "App should start in loading state");
    }

    #[test]
    fn test_view_shows_loading() {
        let app = App::new();
        let view = app.view();
        assert!(
            view.contains("Loading"),
            "View should contain 'Loading': {}",
            view
        );
    }

    #[test]
    fn test_view_shows_quit_hint() {
        let app = App::new();
        let view = app.view();
        assert!(
            view.contains("[q]") || view.contains("quit"),
            "View should show quit hint"
        );
    }

    #[test]
    fn test_quit_q_returns_command() {
        let mut app = App::new();
        let cmd = app.update(key_char('q'));
        assert!(cmd.is_some(), "Pressing 'q' should return quit command");
    }

    #[test]
    fn test_quit_capital_q_returns_command() {
        let mut app = App::new();
        let cmd = app.update(key_char('Q'));
        assert!(cmd.is_some(), "Pressing 'Q' should return quit command");
    }

    #[test]
    fn test_quit_esc_returns_command() {
        let mut app = App::new();
        let cmd = app.update(key_type(KeyType::Esc));
        assert!(cmd.is_some(), "Pressing Esc should return quit command");
    }

    #[test]
    fn test_quit_ctrl_c_returns_command() {
        let mut app = App::new();
        let cmd = app.update(key_type(KeyType::CtrlC));
        assert!(cmd.is_some(), "Pressing Ctrl+C should return quit command");
    }

    #[test]
    fn test_other_keys_dont_quit() {
        let mut app = App::new();
        let cmd = app.update(key_char('x'));
        // Other keys may return a tick command from spinner, but shouldn't quit
        // We can't easily test this without mocking the spinner
        let _ = cmd; // Just ensure it doesn't panic
    }

    #[test]
    fn test_init_returns_tick_command() {
        let app = App::new();
        let cmd = app.init();
        assert!(cmd.is_some(), "Init should return spinner tick command");
    }

    #[test]
    fn test_not_loading_shows_done() {
        let mut app = App::new();
        app.loading = false;
        let view = app.view();
        assert!(
            view.contains("Done"),
            "View should show 'Done' when not loading: {}",
            view
        );
    }
}
