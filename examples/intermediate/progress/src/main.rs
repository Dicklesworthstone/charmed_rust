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
#[derive(PartialEq, Eq)]
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
