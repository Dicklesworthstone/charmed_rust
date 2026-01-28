//! SSH server mode for the demo showcase.
//!
//! This module implements the SSH server wrapper that serves the demo showcase
//! TUI application over SSH connections. Each connected user gets their own
//! independent application instance.
//!
//! # Usage
//!
//! ```bash
//! # Start the SSH server
//! demo_showcase ssh --host-key ./host_key --addr :2222
//!
//! # Connect from another terminal
//! ssh -p 2222 localhost
//! ```
//!
//! # Host Key Setup
//!
//! Before running the SSH server, you need to generate a host key:
//!
//! ```bash
//! ssh-keygen -t ed25519 -f ./host_key -N ""
//! chmod 600 ./host_key
//! ```

use std::path::Path;

use wish::middleware::logging;
use wish::{ServerBuilder, Session};

use crate::app::{App, AppConfig};
use crate::cli::SshArgs;
use crate::config::Config;
use crate::theme::ThemePreset;

/// Errors that can occur when running the SSH server.
#[derive(Debug, thiserror::Error)]
pub enum SshError {
    /// Host key file not found.
    #[error("Host key file not found: {0}")]
    HostKeyNotFound(String),

    /// Host key file not readable.
    #[error("Cannot read host key file: {0}")]
    HostKeyNotReadable(String),

    /// Failed to bind to address.
    #[error("Failed to bind to address '{0}': {1}")]
    BindFailed(String, String),

    /// SSH server error.
    #[error("SSH server error: {0}")]
    ServerError(#[from] wish::Error),
}

/// Result type for SSH operations.
pub type Result<T> = std::result::Result<T, SshError>;

/// Configuration for the SSH server.
#[derive(Debug, Clone)]
pub struct SshConfig {
    /// Address to listen on (e.g., ":2222" or "0.0.0.0:2222").
    pub addr: String,

    /// Path to the host key file.
    pub host_key_path: String,

    /// Maximum concurrent sessions.
    pub max_sessions: usize,

    /// Application theme preset.
    pub theme: ThemePreset,

    /// Whether animations are enabled.
    pub animations: bool,
}

impl SshConfig {
    /// Create SSH config from CLI arguments and runtime config.
    #[must_use]
    pub fn from_args(args: &SshArgs, config: &Config) -> Self {
        Self {
            addr: normalize_address(&args.addr),
            host_key_path: args.host_key.display().to_string(),
            max_sessions: args.max_sessions,
            theme: config.theme_preset,
            animations: config.use_animations(),
        }
    }

    /// Validate the SSH configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails.
    pub fn validate(&self) -> Result<()> {
        let path = Path::new(&self.host_key_path);

        if !path.exists() {
            return Err(SshError::HostKeyNotFound(self.host_key_path.clone()));
        }

        // Check if file is readable
        if std::fs::metadata(path).is_err() {
            return Err(SshError::HostKeyNotReadable(self.host_key_path.clone()));
        }

        Ok(())
    }
}

/// Normalize address string.
///
/// Handles addresses like ":2222" by prepending "0.0.0.0".
fn normalize_address(addr: &str) -> String {
    if addr.starts_with(':') {
        format!("0.0.0.0{addr}")
    } else {
        addr.to_string()
    }
}

/// Run the SSH server with the given configuration.
///
/// This function blocks until the server is shut down.
///
/// # Errors
///
/// Returns an error if:
/// - The host key file cannot be loaded
/// - The server fails to bind to the address
/// - A critical server error occurs
pub async fn run_ssh_server(ssh_config: SshConfig) -> Result<()> {
    // Validate configuration
    ssh_config.validate()?;

    // Log startup
    tracing::info!(
        addr = %ssh_config.addr,
        host_key = %ssh_config.host_key_path,
        max_sessions = ssh_config.max_sessions,
        "Starting demo_showcase SSH server"
    );
    tracing::info!(
        "Connect with: ssh -p {} -o StrictHostKeyChecking=no localhost",
        ssh_config.addr.split(':').nth(1).unwrap_or("2222")
    );

    // Capture config values for the closure
    let theme = ssh_config.theme;
    let animations = ssh_config.animations;

    // Build the server
    let server = ServerBuilder::new()
        .address(&ssh_config.addr)
        .host_key_path(&ssh_config.host_key_path)
        .version("SSH-2.0-CharmedShowcase")
        .banner("Welcome to the Charmed Control Center!")
        // Add logging middleware for connection tracking
        .with_middleware(logging::middleware())
        // Add BubbleTea middleware - creates a new App for each session
        .with_middleware(wish::tea::middleware(move |session: &Session| {
            tracing::info!(user = %session.user(), "New session started");

            // Create app config for this session
            let app_config = AppConfig {
                theme,
                animations,
                mouse: true, // Enable mouse for SSH sessions
            };

            App::with_config(app_config)
        }))
        .build()
        .map_err(|e| {
            // Provide helpful error messages
            let msg = e.to_string();
            if msg.contains("Address already in use") || msg.contains("address in use") {
                SshError::BindFailed(
                    ssh_config.addr.clone(),
                    "Address already in use. Is another server running?".to_string(),
                )
            } else if msg.contains("Permission denied") {
                SshError::BindFailed(
                    ssh_config.addr.clone(),
                    "Permission denied. Try a port above 1024 or run with elevated privileges."
                        .to_string(),
                )
            } else {
                SshError::ServerError(e)
            }
        })?;

    // Run the server
    server.listen().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_address_with_colon() {
        assert_eq!(normalize_address(":2222"), "0.0.0.0:2222");
        assert_eq!(normalize_address(":22"), "0.0.0.0:22");
    }

    #[test]
    fn normalize_address_full() {
        assert_eq!(normalize_address("127.0.0.1:2222"), "127.0.0.1:2222");
        assert_eq!(normalize_address("0.0.0.0:2222"), "0.0.0.0:2222");
    }

    #[test]
    fn ssh_config_validate_missing_key() {
        let config = SshConfig {
            addr: ":2222".to_string(),
            host_key_path: "/nonexistent/path/host_key".to_string(),
            max_sessions: 10,
            theme: ThemePreset::Dark,
            animations: true,
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SshError::HostKeyNotFound(_)));
    }
}
