//! Middleware demonstration example.
//!
//! This example shows how to use and compose various middleware:
//! - Logging middleware for connection tracking
//! - Active terminal middleware for PTY requirements
//! - Rate limiting middleware for abuse prevention
//! - Custom middleware creation
//!
//! ## Running
//!
//! ```bash
//! cargo run --example middleware_example
//! ```
//!
//! Then connect with:
//!
//! ```bash
//! ssh -p 2222 -o StrictHostKeyChecking=no localhost
//! ```

use std::sync::Arc;
use std::time::Duration;

use wish::middleware::{activeterm, elapsed, logging, ratelimiter};
use wish::{BoxFuture, Handler, Middleware, ServerBuilder, Session, println};

/// Custom middleware that adds a welcome message before the handler runs.
fn welcome_middleware() -> Middleware {
    Arc::new(|next: Handler| {
        Arc::new(move |session: Session| {
            let next = next.clone();
            Box::pin(async move {
                // Run before the handler
                println(&session, "=== Welcome to the Middleware Demo ===\n");

                // Call the next handler in the chain
                next(session).await;
            }) as BoxFuture<'static, ()>
        })
    })
}

/// Custom middleware that adds a goodbye message after the handler runs.
fn goodbye_middleware() -> Middleware {
    Arc::new(|next: Handler| {
        Arc::new(move |session: Session| {
            let next = next.clone();
            Box::pin(async move {
                // Call the next handler first
                next(session.clone()).await;

                // Run after the handler
                println(&session, "\n=== Thanks for visiting! ===");
            }) as BoxFuture<'static, ()>
        })
    })
}

/// Custom middleware that counts and displays the connection number.
fn connection_counter_middleware() -> Middleware {
    use std::sync::atomic::{AtomicU64, Ordering};

    let counter = Arc::new(AtomicU64::new(0));

    Arc::new(move |next: Handler| {
        let counter = counter.clone();
        Arc::new(move |session: Session| {
            let next = next.clone();
            let counter = counter.clone();
            Box::pin(async move {
                let conn_num = counter.fetch_add(1, Ordering::SeqCst) + 1;
                println(&session, format!("You are connection #{conn_num}\n"));
                next(session).await;
            }) as BoxFuture<'static, ()>
        })
    })
}

#[tokio::main]
async fn main() -> Result<(), wish::Error> {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Starting Middleware Demo SSH server...");
    tracing::info!("Connect with: ssh -p 2222 -o StrictHostKeyChecking=no localhost");

    // Create a rate limiter: 2 requests per second, burst of 5
    let limiter = ratelimiter::new_rate_limiter(2.0, 5, 1000);

    // Build the SSH server with multiple middleware layers
    //
    // Middleware execution order (outside-in):
    // 1. Rate limiter - checks rate limits first
    // 2. Logging - logs connection start/end
    // 3. Active terminal - ensures PTY is allocated
    // 4. Welcome - adds welcome message
    // 5. Connection counter - shows connection number
    // 6. Handler - main logic
    // 7. Goodbye - adds goodbye message
    // 8. Elapsed - shows elapsed time
    let server = ServerBuilder::new()
        .address("127.0.0.1:2222")
        .version("SSH-2.0-WishMiddleware")
        .idle_timeout(Duration::from_mins(1))
        // Middleware is applied in order (first added = outermost)
        .with_middleware(ratelimiter::middleware(limiter))
        .with_middleware(logging::middleware())
        .with_middleware(activeterm::middleware())
        .with_middleware(welcome_middleware())
        .with_middleware(connection_counter_middleware())
        .with_middleware(goodbye_middleware())
        .with_middleware(elapsed::middleware())
        // Main handler
        .handler(|session| async move {
            println(&session, "This is the main handler.");
            println(&session, format!("User: {}", session.user()));
            println(&session, format!("Remote: {}", session.remote_addr()));

            if let (Some(pty), true) = session.pty() {
                println(
                    &session,
                    format!(
                        "Terminal: {} ({}x{})",
                        pty.term, pty.window.width, pty.window.height
                    ),
                );
            }

            // Simulate some work
            tokio::time::sleep(Duration::from_millis(100)).await;

            let _ = session.exit(0);
        })
        .build()?;

    // Run the server
    server.listen().await
}
