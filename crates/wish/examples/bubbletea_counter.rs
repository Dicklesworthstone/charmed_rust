//! Interactive counter app served over SSH.
//!
//! This example demonstrates serving a `BubbleTea` TUI application over SSH.
//! Each connected user gets their own independent counter instance.
//!
//! ## Running
//!
//! ```bash
//! cargo run --example bubbletea_counter
//! ```
//!
//! Then connect with:
//!
//! ```bash
//! ssh -p 2222 -o StrictHostKeyChecking=no localhost
//! ```
//!
//! ## Controls
//!
//! - `+` or `k`: Increment counter
//! - `-` or `j`: Decrement counter
//! - `r`: Reset counter
//! - `q` or `Ctrl+C`: Quit

use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model};
use wish::middleware::logging;
use wish::{ServerBuilder, Session};

/// Counter model that tracks the count and connected user.
struct Counter {
    count: i32,
    user: String,
}

impl Counter {
    const fn new(user: String) -> Self {
        Self { count: 0, user }
    }
}

impl Model for Counter {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::Runes => {
                    if key.runes == vec!['+'] || key.runes == vec!['k'] {
                        self.count += 1;
                    } else if key.runes == vec!['-'] || key.runes == vec!['j'] {
                        self.count -= 1;
                    } else if key.runes == vec!['r'] {
                        self.count = 0;
                    } else if key.runes == vec!['q'] {
                        return Some(bubbletea::quit());
                    }
                }
                KeyType::CtrlC => {
                    return Some(bubbletea::quit());
                }
                _ => {}
            }
        }
        None
    }

    fn view(&self) -> String {
        format!(
            "\n  Welcome, {}!\n\n  Count: {}\n\n  Controls:\n    [+/k] increment    [-/j] decrement\n    [r]   reset        [q]   quit\n",
            self.user, self.count
        )
    }
}

#[tokio::main]
async fn main() -> Result<(), wish::Error> {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Starting BubbleTea Counter SSH server...");
    tracing::info!("Connect with: ssh -p 2222 -o StrictHostKeyChecking=no localhost");

    // Build the SSH server with BubbleTea middleware
    let server = ServerBuilder::new()
        .address("127.0.0.1:2222")
        .version("SSH-2.0-WishCounter")
        .banner("Welcome to the Wish Counter Demo!")
        // Add logging middleware
        .with_middleware(logging::middleware())
        // Add BubbleTea middleware - creates a new Counter for each session
        .with_middleware(wish::tea::middleware(|session: &Session| {
            Counter::new(session.user().to_string())
        }))
        .build()?;

    // Run the server
    server.listen().await
}
