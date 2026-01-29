//! SSH server mode for the demo showcase.
//!
//! This module implements the SSH server wrapper that serves the demo showcase
//! TUI application over SSH connections. Each connected user gets their own
//! independent application instance.
//!
//! # Usage
//!
//! ```bash
//! # Start the SSH server (development mode - accepts all connections)
//! demo_showcase ssh --host-key ./host_key --addr :2222 --no-auth
//!
//! # Start with password authentication
//! demo_showcase ssh --host-key ./host_key --password secret123
//!
//! # Start with username + password authentication
//! demo_showcase ssh --host-key ./host_key --username demo --password secret123
//!
//! # Using environment variables
//! DEMO_SSH_PASSWORD=secret123 demo_showcase ssh --host-key ./host_key
//!
//! # Connect from another terminal
//! ssh -p 2222 -o StrictHostKeyChecking=no localhost
//! ```
//!
//! # Authentication Modes
//!
//! - **Password auth**: Set `--password` (and optionally `--username`) or
//!   the `DEMO_SSH_PASSWORD` environment variable.
//! - **No auth (development)**: Use `--no-auth` flag. Only for local development!
//!
//! # Host Key Setup
//!
//! Before running the SSH server, you need to generate a host key:
//!
//! ```bash
//! ssh-keygen -t ed25519 -f ./host_key -N ""
//! chmod 600 ./host_key
//! ```
//!
//! # Session Tracking
//!
//! The server logs session start/end events with duration tracking:
//! - Session number and active session count
//! - Username and connection time
//! - Session duration on disconnect

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use wish::auth::{AcceptAllAuth, CallbackAuth, RateLimitedAuth};
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

/// Authentication mode for the SSH server.
#[derive(Debug, Clone)]
pub enum AuthMode {
    /// Accept all connections (development only - security warning logged).
    AcceptAll,

    /// Require password authentication.
    Password {
        /// Required username (None = any username accepted).
        username: Option<String>,
        /// Required password.
        password: String,
    },
}

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

    /// Authentication mode.
    pub auth_mode: AuthMode,
}

impl SshConfig {
    /// Create SSH config from CLI arguments and runtime config.
    #[must_use]
    pub fn from_args(args: &SshArgs, config: &Config) -> Self {
        // Determine authentication mode
        let auth_mode = if args.no_auth {
            AuthMode::AcceptAll
        } else if let Some(password) = &args.password {
            AuthMode::Password {
                username: args.username.clone(),
                password: password.clone(),
            }
        } else {
            // Default: require DEMO_SSH_PASSWORD to be set, or use AcceptAll with warning
            AuthMode::AcceptAll
        };

        Self {
            addr: normalize_address(&args.addr),
            host_key_path: args.host_key.display().to_string(),
            max_sessions: args.max_sessions,
            theme: config.theme_preset,
            animations: config.use_animations(),
            auth_mode,
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

/// Create a password authentication handler.
fn create_password_auth(
    username: Option<String>,
    password: String,
) -> RateLimitedAuth<CallbackAuth<impl Fn(&wish::auth::AuthContext, &str) -> bool + Send + Sync>> {
    if let Some(user) = &username {
        tracing::info!(username = %user, "SSH password authentication enabled");
    } else {
        tracing::info!("SSH password authentication enabled (any username)");
    }

    // Use RateLimitedAuth to prevent brute-force attacks
    let auth = CallbackAuth::new(move |ctx, pwd| {
        // Check username if required
        if let Some(ref required) = username {
            if ctx.username() != required {
                return false;
            }
        }
        // Check password (CallbackAuth does basic comparison)
        pwd == password
    });

    RateLimitedAuth::new(auth)
}

/// Session statistics tracker.
struct SessionStats {
    /// Total sessions started.
    total_sessions: AtomicU64,
    /// Currently active sessions.
    active_sessions: AtomicU64,
}

impl SessionStats {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            total_sessions: AtomicU64::new(0),
            active_sessions: AtomicU64::new(0),
        })
    }

    fn session_started(&self) -> u64 {
        self.active_sessions.fetch_add(1, Ordering::Relaxed);
        self.total_sessions.fetch_add(1, Ordering::Relaxed) + 1
    }

    fn session_ended(&self) {
        self.active_sessions.fetch_sub(1, Ordering::Relaxed);
    }

    fn active_count(&self) -> u64 {
        self.active_sessions.load(Ordering::Relaxed)
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

    // Log startup with auth mode
    let auth_type = match &ssh_config.auth_mode {
        AuthMode::AcceptAll => "none (INSECURE)",
        AuthMode::Password {
            username: Some(u), ..
        } => {
            tracing::info!(username = %u, "Password auth with specific username");
            "password"
        }
        AuthMode::Password { username: None, .. } => "password",
    };

    tracing::info!(
        addr = %ssh_config.addr,
        host_key = %ssh_config.host_key_path,
        max_sessions = ssh_config.max_sessions,
        auth = auth_type,
        "Starting demo_showcase SSH server"
    );
    tracing::info!(
        "Connect with: ssh -p {} -o StrictHostKeyChecking=no localhost",
        ssh_config.addr.split(':').nth(1).unwrap_or("2222")
    );

    // Run server with appropriate auth handler
    match ssh_config.auth_mode.clone() {
        AuthMode::AcceptAll => {
            tracing::warn!(
                "SSH server running with AcceptAll authentication - \
                 NOT FOR PRODUCTION. Set DEMO_SSH_PASSWORD or use --password."
            );
            run_server_with_auth(ssh_config, AcceptAllAuth::new()).await
        }
        AuthMode::Password { username, password } => {
            let auth = create_password_auth(username, password);
            run_server_with_auth(ssh_config, auth).await
        }
    }
}

/// Run the SSH server with a specific auth handler.
async fn run_server_with_auth<H: wish::auth::AuthHandler + 'static>(
    ssh_config: SshConfig,
    auth_handler: H,
) -> Result<()> {
    // Session statistics for tracking
    let stats = SessionStats::new();
    let stats_for_session = Arc::clone(&stats);

    // Capture config values for the closure
    let theme = ssh_config.theme;
    let animations = ssh_config.animations;

    // Build the server
    let server = ServerBuilder::new()
        .address(&ssh_config.addr)
        .host_key_path(&ssh_config.host_key_path)
        .version("SSH-2.0-CharmedShowcase")
        .banner("Welcome to the Charmed Control Center!\r\n")
        .auth_handler(auth_handler)
        // Add logging middleware for connection tracking
        .with_middleware(logging::middleware())
        // Add BubbleTea middleware - creates a new App for each session
        .with_middleware(wish::tea::middleware(move |session: &Session| {
            let session_num = stats_for_session.session_started();
            let active = stats_for_session.active_count();
            let start_time = Instant::now();
            let user = session.user().to_string();

            tracing::info!(
                user = %user,
                session_num,
                active_sessions = active,
                "Session started"
            );

            // Create a guard to log session duration on drop
            let stats_clone = Arc::clone(&stats_for_session);
            let _guard = SessionGuard {
                user,
                start_time,
                stats: stats_clone,
            };

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

/// Guard that logs session duration when dropped.
struct SessionGuard {
    user: String,
    start_time: Instant,
    stats: Arc<SessionStats>,
}

impl Drop for SessionGuard {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        self.stats.session_ended();
        let active = self.stats.active_count();

        tracing::info!(
            user = %self.user,
            duration_secs = duration.as_secs_f64(),
            active_sessions = active,
            "Session ended"
        );
    }
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
            auth_mode: AuthMode::AcceptAll,
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SshError::HostKeyNotFound(_)));
    }

    #[test]
    fn auth_mode_accept_all() {
        let mode = AuthMode::AcceptAll;
        assert!(matches!(mode, AuthMode::AcceptAll));
    }

    #[test]
    fn auth_mode_password_with_username() {
        let mode = AuthMode::Password {
            username: Some("demo".to_string()),
            password: "secret".to_string(),
        };
        match mode {
            AuthMode::Password { username, password } => {
                assert_eq!(username, Some("demo".to_string()));
                assert_eq!(password, "secret");
            }
            _ => panic!("Expected Password mode"),
        }
    }

    #[test]
    fn auth_mode_password_any_user() {
        let mode = AuthMode::Password {
            username: None,
            password: "secret".to_string(),
        };
        match mode {
            AuthMode::Password { username, password } => {
                assert!(username.is_none());
                assert_eq!(password, "secret");
            }
            _ => panic!("Expected Password mode"),
        }
    }

    #[test]
    fn session_stats_tracking() {
        let stats = SessionStats::new();

        assert_eq!(stats.active_count(), 0);

        let num1 = stats.session_started();
        assert_eq!(num1, 1);
        assert_eq!(stats.active_count(), 1);

        let num2 = stats.session_started();
        assert_eq!(num2, 2);
        assert_eq!(stats.active_count(), 2);

        stats.session_ended();
        assert_eq!(stats.active_count(), 1);

        stats.session_ended();
        assert_eq!(stats.active_count(), 0);
    }
}
