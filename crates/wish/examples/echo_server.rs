//! Simple SSH echo server example.
//!
//! This example demonstrates a basic SSH server that accepts all connections
//! and echoes a greeting message.
//!
//! ## Running
//!
//! ```bash
//! cargo run --example echo_server
//! ```
//!
//! Then connect with:
//!
//! ```bash
//! ssh -p 2222 -o StrictHostKeyChecking=no localhost
//! ```

use std::time::Duration;

use wish::{ServerBuilder, println};

#[tokio::main]
async fn main() -> Result<(), wish::Error> {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Build the SSH server
    let server = ServerBuilder::new()
        .address("127.0.0.1:2222")
        .version("SSH-2.0-WishExample")
        .banner("Welcome to the Wish Echo Server!")
        .idle_timeout(Duration::from_secs(300))
        // Accept all connections (no authentication)
        .handler(|session| async move {
            // Print a greeting
            println(&session, "Hello from Wish SSH server!");
            println(&session, &format!("You are: {}", session.user()));
            println(&session, &format!("Connected from: {}", session.remote_addr()));

            if let (Some(pty), true) = session.pty() {
                println(&session, &format!("Terminal: {} ({}x{})",
                    pty.term, pty.window.width, pty.window.height));
            }

            println(&session, "");
            println(&session, "This is a demo server. Goodbye!");

            // Exit cleanly
            let _ = session.exit(0);
        })
        .build()?;

    tracing::info!("Starting Wish SSH server...");
    tracing::info!("Connect with: ssh -p 2222 -o StrictHostKeyChecking=no localhost");

    // Run the server
    server.listen().await
}
