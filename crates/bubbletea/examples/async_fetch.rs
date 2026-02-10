#![forbid(unsafe_code)]
#![allow(clippy::unused_self)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::needless_pass_by_value)]

//! Async runtime example demonstrating `#[derive(Model)]` with the async feature.
//!
//! This example shows how to:
//! - Use `run_async()` with tokio runtime
//! - Use sync commands which are wrapped with `spawn_blocking`
//! - Benefit from tokio's async executor for better concurrency
//!
//! The async runtime provides:
//! - Non-blocking event loop
//! - Graceful shutdown with task cancellation
//! - Automatic `spawn_blocking` for sync `Cmd` commands
//!
//! Run with: `cargo run -p charmed-bubbletea --example async_fetch --features async`

// Fallback main for when async feature is not enabled
#[cfg(not(feature = "async"))]
fn main() {
    eprintln!("This example requires the 'async' feature.");
    eprintln!("Run with: cargo run -p charmed-bubbletea --example async_fetch --features async");
}

// All async-related code is gated behind the feature
#[cfg(feature = "async")]
mod async_app {
    use bubbletea::{Cmd, KeyMsg, KeyType, Message, quit, tick};
    use lipgloss::Style;
    use std::time::{Duration, Instant};

    /// Timer tick message.
    struct TimerTick(#[allow(dead_code)] Instant);

    /// Simulated fetch completion.
    struct FetchComplete(String);

    /// Application state.
    ///
    /// Uses `#[derive(bubbletea::Model)]` to auto-implement the Model trait,
    /// delegating to the inherent `init`, `update`, and `view` methods.
    #[derive(bubbletea::Model)]
    pub struct App {
        status: Status,
        elapsed_secs: u32,
    }

    #[derive(Clone)]
    enum Status {
        Idle,
        Fetching,
        Done(String),
    }

    impl App {
        pub const fn new() -> Self {
            Self {
                status: Status::Idle,
                elapsed_secs: 0,
            }
        }

        /// Simulate a fetch operation. When running with `run_async()`,
        /// this blocking code runs on tokio's blocking thread pool via
        /// `spawn_blocking`, so it doesn't block the async event loop.
        fn fetch_data() -> Cmd {
            Cmd::new(|| {
                // Simulate network latency (runs on blocking thread pool)
                std::thread::sleep(Duration::from_secs(2));
                Message::new(FetchComplete("Data loaded successfully!".to_string()))
            })
        }

        /// Start a one-second timer tick.
        fn tick() -> Cmd {
            tick(Duration::from_secs(1), |t| Message::new(TimerTick(t)))
        }

        /// Initialize the model. Called once when the program starts.
        fn init(&self) -> Option<Cmd> {
            None
        }

        /// Handle messages and update the model state.
        fn update(&mut self, msg: Message) -> Option<Cmd> {
            // Handle keyboard input
            if let Some(key) = msg.downcast_ref::<KeyMsg>() {
                match key.key_type {
                    KeyType::Runes => {
                        if let Some(&ch) = key.runes.first() {
                            match ch {
                                'f' | 'F' => {
                                    if matches!(self.status, Status::Idle) {
                                        self.status = Status::Fetching;
                                        self.elapsed_secs = 0;
                                        // Start both the fetch and timer concurrently
                                        // With run_async(), these run on tokio's thread pools
                                        return Some(
                                            bubbletea::batch(vec![
                                                Some(Self::fetch_data()),
                                                Some(Self::tick()),
                                            ])
                                            .unwrap(),
                                        );
                                    }
                                }
                                'r' | 'R' => {
                                    // Reset to idle state
                                    self.status = Status::Idle;
                                    self.elapsed_secs = 0;
                                }
                                'q' | 'Q' => return Some(quit()),
                                _ => {}
                            }
                        }
                    }
                    KeyType::CtrlC | KeyType::Esc => return Some(quit()),
                    _ => {}
                }
            }

            // Handle timer ticks
            if msg.is::<TimerTick>() && matches!(self.status, Status::Fetching) {
                self.elapsed_secs += 1;
                // Continue ticking while fetching
                return Some(Self::tick());
            }

            // Handle fetch completion
            if let Some(FetchComplete(data)) = msg.downcast::<FetchComplete>() {
                self.status = Status::Done(data);
            }

            None
        }

        /// Render the model as a string for display.
        fn view(&self) -> String {
            let title_style = Style::new().bold().foreground("#7D56F4");
            let status_style = Style::new().foreground("#FF69B4");
            let help_style = Style::new().faint();

            let title = title_style.render("Async Runtime Example");

            let status = match &self.status {
                Status::Idle => status_style.render("Ready. Press 'f' to fetch data."),
                Status::Fetching => {
                    let dots = ".".repeat((self.elapsed_secs as usize % 4) + 1);
                    status_style.render(&format!("Fetching{dots} ({}s elapsed)", self.elapsed_secs))
                }
                Status::Done(data) => status_style.render(&format!("✓ {data}")),
            };

            let help = help_style.render("f: fetch | r: reset | q: quit");

            format!("{title}\n\n{status}\n\n{help}")
        }
    }
}

/// Using tokio's async runtime with bubbletea.
///
/// The `run_async()` method provides:
/// - Non-blocking input handling
/// - Graceful shutdown (5s timeout for in-flight tasks)
/// - Automatic `spawn_blocking` for sync Cmd commands
#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), bubbletea::Error> {
    use async_app::App;
    use bubbletea::Program;

    let model = App::new();
    Program::new(model).run_async().await?;
    Ok(())
}
