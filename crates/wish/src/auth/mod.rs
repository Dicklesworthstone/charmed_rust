//! Authentication module for Wish SSH server.
//!
//! This module provides flexible authentication handlers supporting
//! password, public key, and keyboard-interactive authentication methods.
//!
//! # Example
//!
//! ```rust,ignore
//! use wish::auth::{AuthHandler, AcceptAllAuth, AuthorizedKeysAuth};
//!
//! // Development: accept all connections
//! let dev_auth = AcceptAllAuth::new();
//!
//! // Production: use authorized_keys file
//! let prod_auth = AuthorizedKeysAuth::new("~/.ssh/authorized_keys")
//!     .expect("Failed to load authorized_keys");
//! ```

mod authorized_keys;
mod handler;
mod password;
mod publickey;

pub use authorized_keys::{AuthorizedKey, AuthorizedKeysAuth, parse_authorized_keys};
pub use handler::{AuthContext, AuthHandler, AuthMethod, AuthResult};
pub use password::{AcceptAllAuth, AsyncCallbackAuth, CallbackAuth, PasswordAuth};
pub use publickey::{AsyncPublicKeyAuth, PublicKeyAuth, PublicKeyCallbackAuth};

use std::sync::Arc;

use crate::PublicKey;

/// Session ID type for tracking authentication attempts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(pub u64);

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Default authentication rejection delay to mitigate timing attacks.
pub const DEFAULT_AUTH_REJECTION_DELAY_MS: u64 = 100;

/// Default maximum authentication attempts before disconnection.
pub const DEFAULT_MAX_AUTH_ATTEMPTS: u32 = 6;

/// Composite authentication handler that tries multiple handlers in order.
///
/// Returns `Accept` if any handler accepts, `Reject` if all reject.
pub struct CompositeAuth {
    handlers: Vec<Arc<dyn AuthHandler>>,
}

impl CompositeAuth {
    /// Creates a new composite auth handler.
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Adds an authentication handler.
    #[allow(clippy::should_implement_trait)]
    pub fn add<H: AuthHandler + 'static>(mut self, handler: H) -> Self {
        self.handlers.push(Arc::new(handler));
        self
    }
}

impl Default for CompositeAuth {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl AuthHandler for CompositeAuth {
    async fn auth_password(&self, ctx: &AuthContext, password: &str) -> AuthResult {
        for handler in &self.handlers {
            match handler.auth_password(ctx, password).await {
                AuthResult::Accept => return AuthResult::Accept,
                AuthResult::Partial { next_methods } => {
                    return AuthResult::Partial { next_methods };
                }
                AuthResult::Reject => continue,
            }
        }
        AuthResult::Reject
    }

    async fn auth_publickey(&self, ctx: &AuthContext, key: &PublicKey) -> AuthResult {
        for handler in &self.handlers {
            match handler.auth_publickey(ctx, key).await {
                AuthResult::Accept => return AuthResult::Accept,
                AuthResult::Partial { next_methods } => {
                    return AuthResult::Partial { next_methods };
                }
                AuthResult::Reject => continue,
            }
        }
        AuthResult::Reject
    }

    async fn auth_keyboard_interactive(&self, ctx: &AuthContext, response: &str) -> AuthResult {
        for handler in &self.handlers {
            match handler.auth_keyboard_interactive(ctx, response).await {
                AuthResult::Accept => return AuthResult::Accept,
                AuthResult::Partial { next_methods } => {
                    return AuthResult::Partial { next_methods };
                }
                AuthResult::Reject => continue,
            }
        }
        AuthResult::Reject
    }
}

/// Rate-limited authentication wrapper.
///
/// Adds a delay after failed authentication attempts to mitigate
/// brute-force attacks and timing attacks.
pub struct RateLimitedAuth<H> {
    inner: H,
    rejection_delay_ms: u64,
    max_attempts: u32,
}

impl<H: AuthHandler> RateLimitedAuth<H> {
    /// Creates a new rate-limited auth wrapper with default settings.
    pub fn new(inner: H) -> Self {
        Self {
            inner,
            rejection_delay_ms: DEFAULT_AUTH_REJECTION_DELAY_MS,
            max_attempts: DEFAULT_MAX_AUTH_ATTEMPTS,
        }
    }

    /// Sets the rejection delay in milliseconds.
    pub fn with_rejection_delay(mut self, delay_ms: u64) -> Self {
        self.rejection_delay_ms = delay_ms;
        self
    }

    /// Sets the maximum authentication attempts.
    pub fn with_max_attempts(mut self, max: u32) -> Self {
        self.max_attempts = max;
        self
    }

    /// Returns the maximum authentication attempts.
    pub fn max_attempts(&self) -> u32 {
        self.max_attempts
    }

    async fn apply_rejection_delay(&self) {
        if self.rejection_delay_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(self.rejection_delay_ms)).await;
        }
    }
}

#[async_trait::async_trait]
impl<H: AuthHandler + Send + Sync> AuthHandler for RateLimitedAuth<H> {
    async fn auth_password(&self, ctx: &AuthContext, password: &str) -> AuthResult {
        let result = self.inner.auth_password(ctx, password).await;
        if matches!(result, AuthResult::Reject) {
            self.apply_rejection_delay().await;
        }
        result
    }

    async fn auth_publickey(&self, ctx: &AuthContext, key: &PublicKey) -> AuthResult {
        let result = self.inner.auth_publickey(ctx, key).await;
        if matches!(result, AuthResult::Reject) {
            self.apply_rejection_delay().await;
        }
        result
    }

    async fn auth_keyboard_interactive(&self, ctx: &AuthContext, response: &str) -> AuthResult {
        let result = self.inner.auth_keyboard_interactive(ctx, response).await;
        if matches!(result, AuthResult::Reject) {
            self.apply_rejection_delay().await;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    #[test]
    fn test_session_id() {
        let id = SessionId(42);
        assert_eq!(id.0, 42);
        assert_eq!(format!("{}", id), "42");
    }

    #[test]
    fn test_composite_auth_empty() {
        let auth = CompositeAuth::new();
        assert!(auth.handlers.is_empty());
    }

    #[tokio::test]
    async fn test_composite_auth_accepts_first() {
        let auth = CompositeAuth::new().add(AcceptAllAuth::new());

        let addr: SocketAddr = "127.0.0.1:22".parse().unwrap();
        let ctx = AuthContext::new("testuser", addr, SessionId(1));

        let result = auth.auth_password(&ctx, "password").await;
        assert!(matches!(result, AuthResult::Accept));
    }

    #[test]
    fn test_rate_limited_auth_settings() {
        let inner = AcceptAllAuth::new();
        let auth = RateLimitedAuth::new(inner)
            .with_rejection_delay(200)
            .with_max_attempts(3);

        assert_eq!(auth.rejection_delay_ms, 200);
        assert_eq!(auth.max_attempts(), 3);
    }
}
