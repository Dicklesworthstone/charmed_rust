//! Progress Bar Example
//!
//! This example demonstrates:
//! - Using the bubbles Progress component
//! - Simulating async operations with tick commands
//! - Progress bar updates with visual feedback
//! - Cancellation with Escape
//!
//! Run with: `cargo run -p example-progress`

#![forbid(unsafe_code)]

use bubbles::progress::Progress;
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Program, quit, tick};
use lipgloss::Style;
use std::time::{Duration, Instant};

/// A tick message for progress updates.
struct TickMsg(#[allow(dead_code)] Instant);

impl TickMsg {
    fn msg(instant: Instant) -> Message {
        Message::new(Self(instant))
    }
}

/// Progress state for the simulated operation.
#[derive(Debug, PartialEq, Eq)]
enum State {
    /// Ready to start.
    Ready,
    /// Operation in progress.
    Running,
    /// Operation completed successfully.
    Done,
    /// Operation was cancelled.
    Cancelled,
}

/// The main application model.
#[derive(bubbletea::Model)]
struct App {
    progress: Progress,
    percent: f64,
    state: State,
}

impl App {
    /// Create a new app with a styled progress bar.
    fn new() -> Self {
        let progress = Progress::new().width(40);

        Self {
            progress,
            percent: 0.0,
            state: State::Ready,
        }
    }

    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle tick messages for progress updates
        if msg.downcast_ref::<TickMsg>().is_some() {
            if self.state == State::Running {
                self.percent += 2.0; // Increment by 2% each tick

                if self.percent >= 100.0 {
                    self.percent = 100.0;
                    self.state = State::Done;
                    return None;
                }

                // Continue ticking
                return Some(tick(Duration::from_millis(50), TickMsg::msg));
            }
            return None;
        }

        // Handle keyboard input
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::Enter | KeyType::Space => {
                    if self.state == State::Ready {
                        self.state = State::Running;
                        self.percent = 0.0;
                        // Start ticking
                        return Some(tick(Duration::from_millis(50), TickMsg::msg));
                    }
                }
                KeyType::Runes => {
                    if let Some(&ch) = key.runes.first() {
                        match ch {
                            'r' | 'R' => {
                                // Reset
                                self.state = State::Ready;
                                self.percent = 0.0;
                            }
                            'q' | 'Q' => return Some(quit()),
                            _ => {}
                        }
                    }
                }
                KeyType::Esc => {
                    if self.state == State::Running {
                        self.state = State::Cancelled;
                    } else {
                        return Some(quit());
                    }
                }
                KeyType::CtrlC => return Some(quit()),
                _ => {}
            }
        }

        None
    }

    fn view(&self) -> String {
        let mut output = String::new();

        // Title
        let title_style = Style::new().bold();
        output.push_str(&format!(
            "\n  {}\n\n",
            title_style.render("Progress Example")
        ));

        // Progress bar
        output.push_str(&format!(
            "  {}\n\n",
            self.progress.view_as(self.percent / 100.0)
        ));

        // Percentage
        let pct_style = Style::new().foreground("212");
        output.push_str(&format!(
            "  {} {:.0}%\n\n",
            pct_style.render("Progress:"),
            self.percent
        ));

        // Status message
        let status_style = match self.state {
            State::Ready => Style::new().foreground("39"),
            State::Running => Style::new().foreground("214"),
            State::Done => Style::new().foreground("82"),
            State::Cancelled => Style::new().foreground("196"),
        };

        let status_text = match self.state {
            State::Ready => "Ready to start",
            State::Running => "Processing...",
            State::Done => "Complete!",
            State::Cancelled => "Cancelled",
        };

        output.push_str(&format!("  {}\n\n", status_style.render(status_text)));

        // Help text
        let help_style = Style::new().foreground("241");
        let help = match self.state {
            State::Ready => "Press Enter/Space to start, q to quit",
            State::Running => "Press Esc to cancel",
            State::Done | State::Cancelled => "Press 'r' to restart, q to quit",
        };
        output.push_str(&format!("  {}\n", help_style.render(help)));

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
        assert_eq!(app.state, State::Ready);
        assert_eq!(app.percent, 0.0);
    }

    #[test]
    fn test_init_returns_none() {
        let app = App::new();
        assert!(app.init().is_none());
    }

    #[test]
    fn test_start_with_enter() {
        let mut app = App::new();
        assert_eq!(app.state, State::Ready);

        let cmd = app.update(key_type(KeyType::Enter));
        assert_eq!(app.state, State::Running);
        assert!(cmd.is_some()); // Returns tick command
    }

    #[test]
    fn test_start_with_space() {
        let mut app = App::new();
        assert_eq!(app.state, State::Ready);

        let cmd = app.update(key_type(KeyType::Space));
        assert_eq!(app.state, State::Running);
        assert!(cmd.is_some()); // Returns tick command
    }

    #[test]
    fn test_cancel_with_esc() {
        let mut app = App::new();
        app.state = State::Running;

        app.update(key_type(KeyType::Esc));
        assert_eq!(app.state, State::Cancelled);
    }

    #[test]
    fn test_quit_with_esc_when_not_running() {
        let mut app = App::new();
        assert_eq!(app.state, State::Ready);

        let cmd = app.update(key_type(KeyType::Esc));
        assert!(cmd.is_some()); // Returns quit command
    }

    #[test]
    fn test_quit_with_q() {
        let mut app = App::new();
        let cmd = app.update(key_char('q'));
        assert!(cmd.is_some());
    }

    #[test]
    fn test_quit_with_capital_q() {
        let mut app = App::new();
        let cmd = app.update(key_char('Q'));
        assert!(cmd.is_some());
    }

    #[test]
    fn test_quit_with_ctrl_c() {
        let mut app = App::new();
        let cmd = app.update(key_type(KeyType::CtrlC));
        assert!(cmd.is_some());
    }

    #[test]
    fn test_reset_with_r() {
        let mut app = App::new();
        app.state = State::Done;
        app.percent = 100.0;

        app.update(key_char('r'));
        assert_eq!(app.state, State::Ready);
        assert_eq!(app.percent, 0.0);
    }

    #[test]
    fn test_reset_with_capital_r() {
        let mut app = App::new();
        app.state = State::Cancelled;
        app.percent = 50.0;

        app.update(key_char('R'));
        assert_eq!(app.state, State::Ready);
        assert_eq!(app.percent, 0.0);
    }

    #[test]
    fn test_tick_increments_progress() {
        let mut app = App::new();
        app.state = State::Running;
        app.percent = 0.0;

        let tick_msg = TickMsg::msg(Instant::now());
        app.update(tick_msg);

        assert_eq!(app.percent, 2.0);
        assert_eq!(app.state, State::Running);
    }

    #[test]
    fn test_tick_completes_at_100() {
        let mut app = App::new();
        app.state = State::Running;
        app.percent = 99.0;

        let tick_msg = TickMsg::msg(Instant::now());
        let cmd = app.update(tick_msg);

        assert_eq!(app.percent, 100.0);
        assert_eq!(app.state, State::Done);
        assert!(cmd.is_none()); // No more ticks
    }

    #[test]
    fn test_tick_ignored_when_not_running() {
        let mut app = App::new();
        app.state = State::Ready;
        app.percent = 0.0;

        let tick_msg = TickMsg::msg(Instant::now());
        let cmd = app.update(tick_msg);

        assert_eq!(app.percent, 0.0);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_view_contains_title() {
        let app = App::new();
        let view = app.view();
        assert!(view.contains("Progress Example"));
    }

    #[test]
    fn test_view_contains_ready_status() {
        let app = App::new();
        let view = app.view();
        assert!(view.contains("Ready to start"));
    }

    #[test]
    fn test_view_contains_running_status() {
        let mut app = App::new();
        app.state = State::Running;
        let view = app.view();
        assert!(view.contains("Processing"));
    }

    #[test]
    fn test_view_contains_done_status() {
        let mut app = App::new();
        app.state = State::Done;
        let view = app.view();
        assert!(view.contains("Complete"));
    }

    #[test]
    fn test_view_contains_cancelled_status() {
        let mut app = App::new();
        app.state = State::Cancelled;
        let view = app.view();
        assert!(view.contains("Cancelled"));
    }

    #[test]
    fn test_view_contains_progress_percentage() {
        let mut app = App::new();
        app.percent = 50.0;
        let view = app.view();
        assert!(view.contains("50%"));
    }

    #[test]
    fn test_enter_during_running_does_nothing() {
        let mut app = App::new();
        app.state = State::Running;
        app.percent = 30.0;

        let cmd = app.update(key_type(KeyType::Enter));
        assert_eq!(app.state, State::Running);
        assert_eq!(app.percent, 30.0);
        assert!(cmd.is_none());
    }
}
