//! SSH server with authentication examples.
//!
//! This example demonstrates various authentication methods:
//! - Password authentication with a callback
//! - Public key authentication from `authorized_keys` file
//! - Composite authentication (multiple methods)
//!
//! ## Running
//!
//! ```bash
//! cargo run --example authenticated_server
//! ```
//!
//! Then connect with password:
//!
//! ```bash
//! ssh -p 2222 -o StrictHostKeyChecking=no admin@localhost
//! # Password: secret
//! ```
//!
//! Or with guest account:
//!
//! ```bash
//! ssh -p 2222 -o StrictHostKeyChecking=no guest@localhost
//! # Password: guest
//! ```
//!
//! Or with public key (if you have keys configured):
//!
//! ```bash
//! ssh -p 2222 -o StrictHostKeyChecking=no -i ~/.ssh/id_ed25519 localhost
//! ```

use std::time::Duration;

use wish::auth::{
    AcceptAllAuth, AuthContext, AuthHandler, AuthResult, CompositeAuth, PasswordAuth,
};
use wish::middleware::logging;
use wish::{ServerBuilder, println};

/// Custom password authentication handler.
///
/// Accepts two users:
/// - admin:secret (full access)
/// - guest:guest (limited access)
struct MyPasswordAuth;

#[async_trait::async_trait]
impl AuthHandler for MyPasswordAuth {
    async fn auth_password(&self, ctx: &AuthContext, password: &str) -> AuthResult {
        let valid = match ctx.username() {
            "admin" => password == "secret",
            "guest" => password == "guest",
            _ => false,
        };

        if valid {
            tracing::info!(user = %ctx.username(), "Authentication successful");
            AuthResult::Accept
        } else {
            tracing::warn!(user = %ctx.username(), "Authentication failed");
            AuthResult::Reject
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), wish::Error> {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Starting Authenticated SSH server...");
    tracing::info!("Connect with: ssh -p 2222 admin@localhost (password: secret)");
    tracing::info!("Or: ssh -p 2222 guest@localhost (password: guest)");

    // Build the SSH server with authentication
    let server = ServerBuilder::new()
        .address("127.0.0.1:2222")
        .version("SSH-2.0-WishAuth")
        .banner("Welcome! Please authenticate.")
        .idle_timeout(Duration::from_secs(300))
        // Set authentication limits
        .max_auth_attempts(3)
        .auth_rejection_delay(100) // 100ms delay on failed auth (timing attack mitigation)
        // Use our custom password authentication
        .auth_handler(MyPasswordAuth)
        // Add logging middleware
        .with_middleware(logging::structured_middleware())
        // Main handler
        .handler(|session| async move {
            let user = session.user();

            println(&session, "\nAuthentication successful!".to_string());
            println(&session, format!("Welcome, {user}!"));
            println(&session, format!("Your IP: {}", session.remote_addr()));

            // Different messages based on user
            if user == "admin" {
                println(&session, "\nYou have admin access.");
                println(&session, "You can run any command.");
            } else if user == "guest" {
                println(&session, "\nYou have guest access.");
                println(&session, "Some features may be restricted.");
            }

            println(&session, "\nGoodbye!");
            let _ = session.exit(0);
        })
        .build()?;

    // Run the server
    server.listen().await
}

/// Example of using composite authentication.
///
/// This would allow both password and public key authentication.
#[allow(dead_code)]
fn example_composite_auth() -> CompositeAuth {
    // Try password first with predefined users, then fall back to accept all (for demo)
    CompositeAuth::new()
        .add(PasswordAuth::new().add_user("admin", "secret"))
        .add(AcceptAllAuth::new()) // Fallback - remove in production!
}
