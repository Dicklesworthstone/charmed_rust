//! Core authentication handler trait and types.

use std::net::SocketAddr;

use async_trait::async_trait;

use super::SessionId;
use crate::PublicKey;

/// Context provided to authentication handlers.
///
/// Contains information about the authentication attempt including
/// the username, remote address, and session identifier.
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// The username attempting authentication.
    pub username: String,
    /// The remote address of the client.
    pub remote_addr: SocketAddr,
    /// The session ID for this connection.
    pub session_id: SessionId,
    /// Number of authentication attempts so far.
    pub attempt_count: u32,
}

impl AuthContext {
    /// Creates a new authentication context.
    pub fn new(username: impl Into<String>, remote_addr: SocketAddr, session_id: SessionId) -> Self {
        Self {
            username: username.into(),
            remote_addr,
            session_id,
            attempt_count: 0,
        }
    }

    /// Creates a context with an incremented attempt count.
    pub fn with_attempt(mut self, count: u32) -> Self {
        self.attempt_count = count;
        self
    }

    /// Returns the username.
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns the remote address.
    pub fn remote_addr(&self) -> SocketAddr {
        self.remote_addr
    }

    /// Returns the session ID.
    pub fn session_id(&self) -> SessionId {
        self.session_id
    }

    /// Returns the current attempt count.
    pub fn attempt_count(&self) -> u32 {
        self.attempt_count
    }
}

/// Authentication methods supported by SSH.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthMethod {
    /// No authentication (anonymous).
    None,
    /// Password authentication.
    Password,
    /// Public key authentication.
    PublicKey,
    /// Keyboard-interactive authentication.
    KeyboardInteractive,
    /// Host-based authentication.
    HostBased,
}

impl std::fmt::Display for AuthMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthMethod::None => write!(f, "none"),
            AuthMethod::Password => write!(f, "password"),
            AuthMethod::PublicKey => write!(f, "publickey"),
            AuthMethod::KeyboardInteractive => write!(f, "keyboard-interactive"),
            AuthMethod::HostBased => write!(f, "hostbased"),
        }
    }
}

/// Result of an authentication attempt.
#[derive(Debug, Clone)]
pub enum AuthResult {
    /// Authentication was successful.
    Accept,
    /// Authentication was rejected.
    Reject,
    /// Authentication partially succeeded, continue with additional methods.
    Partial {
        /// Methods to continue with.
        next_methods: Vec<AuthMethod>,
    },
}

impl AuthResult {
    /// Returns true if the authentication was accepted.
    pub fn is_accepted(&self) -> bool {
        matches!(self, AuthResult::Accept)
    }

    /// Returns true if the authentication was rejected.
    pub fn is_rejected(&self) -> bool {
        matches!(self, AuthResult::Reject)
    }

    /// Returns true if partial authentication is required.
    pub fn is_partial(&self) -> bool {
        matches!(self, AuthResult::Partial { .. })
    }
}

/// Trait for implementing authentication handlers.
///
/// Authentication handlers decide whether to accept or reject
/// authentication attempts based on credentials provided.
///
/// # Example
///
/// ```rust,ignore
/// use wish::auth::{AuthHandler, AuthContext, AuthResult};
/// use async_trait::async_trait;
///
/// struct MyAuth;
///
/// #[async_trait]
/// impl AuthHandler for MyAuth {
///     async fn auth_password(&self, ctx: &AuthContext, password: &str) -> AuthResult {
///         if ctx.username() == "admin" && password == "secret" {
///             AuthResult::Accept
///         } else {
///             AuthResult::Reject
///         }
///     }
/// }
/// ```
#[async_trait]
pub trait AuthHandler: Send + Sync {
    /// Authenticate with password.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The authentication context.
    /// * `password` - The password provided by the client.
    ///
    /// # Returns
    ///
    /// The authentication result.
    async fn auth_password(&self, ctx: &AuthContext, password: &str) -> AuthResult {
        let _ = (ctx, password);
        AuthResult::Reject
    }

    /// Authenticate with public key.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The authentication context.
    /// * `key` - The public key provided by the client.
    ///
    /// # Returns
    ///
    /// The authentication result.
    async fn auth_publickey(&self, ctx: &AuthContext, key: &PublicKey) -> AuthResult {
        let _ = (ctx, key);
        AuthResult::Reject
    }

    /// Authenticate with keyboard-interactive.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The authentication context.
    /// * `response` - The response provided by the client.
    ///
    /// # Returns
    ///
    /// The authentication result.
    async fn auth_keyboard_interactive(
        &self,
        ctx: &AuthContext,
        response: &str,
    ) -> AuthResult {
        let _ = (ctx, response);
        AuthResult::Reject
    }

    /// Check if "none" authentication is allowed.
    ///
    /// By default, returns `Reject`. Override to allow anonymous access.
    async fn auth_none(&self, ctx: &AuthContext) -> AuthResult {
        let _ = ctx;
        AuthResult::Reject
    }

    /// Returns the authentication methods supported by this handler.
    fn supported_methods(&self) -> Vec<AuthMethod> {
        vec![AuthMethod::Password, AuthMethod::PublicKey]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_context() {
        let addr: SocketAddr = "192.168.1.1:12345".parse().unwrap();
        let ctx = AuthContext::new("testuser", addr, SessionId(42));

        assert_eq!(ctx.username(), "testuser");
        assert_eq!(ctx.remote_addr(), addr);
        assert_eq!(ctx.session_id(), SessionId(42));
        assert_eq!(ctx.attempt_count(), 0);

        let ctx = ctx.with_attempt(3);
        assert_eq!(ctx.attempt_count(), 3);
    }

    #[test]
    fn test_auth_method_display() {
        assert_eq!(format!("{}", AuthMethod::None), "none");
        assert_eq!(format!("{}", AuthMethod::Password), "password");
        assert_eq!(format!("{}", AuthMethod::PublicKey), "publickey");
        assert_eq!(format!("{}", AuthMethod::KeyboardInteractive), "keyboard-interactive");
        assert_eq!(format!("{}", AuthMethod::HostBased), "hostbased");
    }

    #[test]
    fn test_auth_result_checks() {
        let accept = AuthResult::Accept;
        assert!(accept.is_accepted());
        assert!(!accept.is_rejected());
        assert!(!accept.is_partial());

        let reject = AuthResult::Reject;
        assert!(!reject.is_accepted());
        assert!(reject.is_rejected());
        assert!(!reject.is_partial());

        let partial = AuthResult::Partial {
            next_methods: vec![AuthMethod::Password],
        };
        assert!(!partial.is_accepted());
        assert!(!partial.is_rejected());
        assert!(partial.is_partial());
    }

    use super::super::SessionId;

    struct RejectAllAuth;

    #[async_trait]
    impl AuthHandler for RejectAllAuth {}

    #[tokio::test]
    async fn test_default_auth_handler_rejects() {
        let handler = RejectAllAuth;
        let addr: SocketAddr = "127.0.0.1:22".parse().unwrap();
        let ctx = AuthContext::new("user", addr, SessionId(1));

        assert!(matches!(handler.auth_password(&ctx, "pass").await, AuthResult::Reject));
        assert!(matches!(handler.auth_none(&ctx).await, AuthResult::Reject));
    }
}
