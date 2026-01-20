//! Password authentication handlers.

use std::sync::Arc;

use async_trait::async_trait;
use tracing::{debug, warn};

use super::handler::{AuthContext, AuthHandler, AuthMethod, AuthResult};
use crate::PublicKey;

/// Authentication handler that accepts all authentication attempts.
///
/// **WARNING**: This should only be used for development and testing.
/// Using this in production is a serious security risk.
///
/// # Example
///
/// ```rust,ignore
/// use wish::auth::AcceptAllAuth;
///
/// let auth = AcceptAllAuth::new();
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct AcceptAllAuth {
    _private: (),
}

impl AcceptAllAuth {
    /// Creates a new AcceptAllAuth handler.
    pub fn new() -> Self {
        warn!("AcceptAllAuth in use - NOT FOR PRODUCTION");
        Self { _private: () }
    }
}

#[async_trait]
impl AuthHandler for AcceptAllAuth {
    async fn auth_password(&self, ctx: &AuthContext, _password: &str) -> AuthResult {
        warn!(
            username = %ctx.username(),
            remote_addr = %ctx.remote_addr(),
            "AcceptAllAuth: accepting password auth"
        );
        AuthResult::Accept
    }

    async fn auth_publickey(&self, ctx: &AuthContext, _key: &PublicKey) -> AuthResult {
        warn!(
            username = %ctx.username(),
            remote_addr = %ctx.remote_addr(),
            "AcceptAllAuth: accepting public key auth"
        );
        AuthResult::Accept
    }

    async fn auth_keyboard_interactive(&self, ctx: &AuthContext, _response: &str) -> AuthResult {
        warn!(
            username = %ctx.username(),
            remote_addr = %ctx.remote_addr(),
            "AcceptAllAuth: accepting keyboard-interactive auth"
        );
        AuthResult::Accept
    }

    async fn auth_none(&self, ctx: &AuthContext) -> AuthResult {
        warn!(
            username = %ctx.username(),
            remote_addr = %ctx.remote_addr(),
            "AcceptAllAuth: accepting none auth"
        );
        AuthResult::Accept
    }

    fn supported_methods(&self) -> Vec<AuthMethod> {
        vec![
            AuthMethod::None,
            AuthMethod::Password,
            AuthMethod::PublicKey,
            AuthMethod::KeyboardInteractive,
        ]
    }
}

/// Callback-based password authentication handler.
///
/// Uses a user-provided callback function to validate passwords.
///
/// # Example
///
/// ```rust,ignore
/// use wish::auth::CallbackAuth;
///
/// let auth = CallbackAuth::new(|ctx, password| {
///     ctx.username() == "admin" && password == "secret"
/// });
/// ```
pub struct CallbackAuth<F>
where
    F: Fn(&AuthContext, &str) -> bool + Send + Sync,
{
    callback: F,
}

impl<F> CallbackAuth<F>
where
    F: Fn(&AuthContext, &str) -> bool + Send + Sync,
{
    /// Creates a new callback-based auth handler.
    pub fn new(callback: F) -> Self {
        Self { callback }
    }
}

#[async_trait]
impl<F> AuthHandler for CallbackAuth<F>
where
    F: Fn(&AuthContext, &str) -> bool + Send + Sync + 'static,
{
    async fn auth_password(&self, ctx: &AuthContext, password: &str) -> AuthResult {
        debug!(
            username = %ctx.username(),
            remote_addr = %ctx.remote_addr(),
            "CallbackAuth: password auth attempt"
        );

        if (self.callback)(ctx, password) {
            debug!(username = %ctx.username(), "CallbackAuth: password accepted");
            AuthResult::Accept
        } else {
            debug!(username = %ctx.username(), "CallbackAuth: password rejected");
            AuthResult::Reject
        }
    }

    fn supported_methods(&self) -> Vec<AuthMethod> {
        vec![AuthMethod::Password]
    }
}

/// Simple password authentication against a static map.
///
/// Stores username/password pairs and validates against them.
///
/// # Example
///
/// ```rust,ignore
/// use wish::auth::PasswordAuth;
///
/// let auth = PasswordAuth::new()
///     .add_user("alice", "password123")
///     .add_user("bob", "secret456");
/// ```
pub struct PasswordAuth {
    users: std::collections::HashMap<String, String>,
}

impl PasswordAuth {
    /// Creates a new empty password auth handler.
    pub fn new() -> Self {
        Self {
            users: std::collections::HashMap::new(),
        }
    }

    /// Adds a user with the given password.
    pub fn add_user(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        self.users.insert(username.into(), password.into());
        self
    }

    /// Adds multiple users from an iterator.
    pub fn add_users<I, U, P>(mut self, users: I) -> Self
    where
        I: IntoIterator<Item = (U, P)>,
        U: Into<String>,
        P: Into<String>,
    {
        for (username, password) in users {
            self.users.insert(username.into(), password.into());
        }
        self
    }

    /// Returns the number of registered users.
    pub fn user_count(&self) -> usize {
        self.users.len()
    }

    /// Checks if a user exists.
    pub fn has_user(&self, username: &str) -> bool {
        self.users.contains_key(username)
    }
}

impl Default for PasswordAuth {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuthHandler for PasswordAuth {
    async fn auth_password(&self, ctx: &AuthContext, password: &str) -> AuthResult {
        debug!(
            username = %ctx.username(),
            remote_addr = %ctx.remote_addr(),
            "PasswordAuth: auth attempt"
        );

        match self.users.get(ctx.username()) {
            Some(stored_password) if stored_password == password => {
                debug!(username = %ctx.username(), "PasswordAuth: accepted");
                AuthResult::Accept
            }
            Some(_) => {
                debug!(username = %ctx.username(), "PasswordAuth: wrong password");
                AuthResult::Reject
            }
            None => {
                debug!(username = %ctx.username(), "PasswordAuth: unknown user");
                AuthResult::Reject
            }
        }
    }

    fn supported_methods(&self) -> Vec<AuthMethod> {
        vec![AuthMethod::Password]
    }
}

/// Async callback-based password authentication handler.
///
/// Uses a user-provided async callback function to validate passwords.
/// Useful for database lookups or remote authentication services.
///
/// # Example
///
/// ```rust,ignore
/// use wish::auth::AsyncCallbackAuth;
/// use std::sync::Arc;
///
/// let auth = AsyncCallbackAuth::new(Arc::new(|ctx, password| {
///     Box::pin(async move {
///         // Async database lookup
///         database_check(ctx.username(), password).await
///     })
/// }));
/// ```
#[allow(dead_code)]
pub struct AsyncCallbackAuth<F>
where
    F: Fn(&AuthContext, &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send>>
        + Send
        + Sync,
{
    callback: Arc<F>,
}

impl<F> AsyncCallbackAuth<F>
where
    F: Fn(&AuthContext, &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send>>
        + Send
        + Sync,
{
    /// Creates a new async callback-based auth handler.
    #[allow(dead_code)]
    pub fn new(callback: Arc<F>) -> Self {
        Self { callback }
    }
}

#[async_trait]
impl<F> AuthHandler for AsyncCallbackAuth<F>
where
    F: Fn(&AuthContext, &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send>>
        + Send
        + Sync
        + 'static,
{
    async fn auth_password(&self, ctx: &AuthContext, password: &str) -> AuthResult {
        debug!(
            username = %ctx.username(),
            remote_addr = %ctx.remote_addr(),
            "AsyncCallbackAuth: password auth attempt"
        );

        if (self.callback)(ctx, password).await {
            debug!(username = %ctx.username(), "AsyncCallbackAuth: password accepted");
            AuthResult::Accept
        } else {
            debug!(username = %ctx.username(), "AsyncCallbackAuth: password rejected");
            AuthResult::Reject
        }
    }

    fn supported_methods(&self) -> Vec<AuthMethod> {
        vec![AuthMethod::Password]
    }
}

#[cfg(test)]
mod tests {
    use super::super::SessionId;
    use super::*;
    use std::net::SocketAddr;
    use std::sync::Arc;

    fn make_context(username: &str) -> AuthContext {
        let addr: SocketAddr = "127.0.0.1:22".parse().unwrap();
        AuthContext::new(username, addr, SessionId(1))
    }

    #[tokio::test]
    async fn test_accept_all_auth() {
        let auth = AcceptAllAuth::new();
        let ctx = make_context("anyone");

        assert!(matches!(
            auth.auth_password(&ctx, "anything").await,
            AuthResult::Accept
        ));
        assert!(matches!(auth.auth_none(&ctx).await, AuthResult::Accept));
    }

    #[tokio::test]
    async fn test_callback_auth() {
        let auth =
            CallbackAuth::new(|ctx, password| ctx.username() == "admin" && password == "secret");

        let ctx = make_context("admin");
        assert!(matches!(
            auth.auth_password(&ctx, "secret").await,
            AuthResult::Accept
        ));
        assert!(matches!(
            auth.auth_password(&ctx, "wrong").await,
            AuthResult::Reject
        ));

        let ctx = make_context("user");
        assert!(matches!(
            auth.auth_password(&ctx, "secret").await,
            AuthResult::Reject
        ));
    }

    #[tokio::test]
    async fn test_password_auth() {
        let auth = PasswordAuth::new()
            .add_user("alice", "password123")
            .add_user("bob", "secret456");

        assert_eq!(auth.user_count(), 2);
        assert!(auth.has_user("alice"));
        assert!(!auth.has_user("charlie"));

        let ctx = make_context("alice");
        assert!(matches!(
            auth.auth_password(&ctx, "password123").await,
            AuthResult::Accept
        ));
        assert!(matches!(
            auth.auth_password(&ctx, "wrong").await,
            AuthResult::Reject
        ));

        let ctx = make_context("charlie");
        assert!(matches!(
            auth.auth_password(&ctx, "any").await,
            AuthResult::Reject
        ));
    }

    #[test]
    fn test_password_auth_add_users() {
        let users = vec![("user1", "pass1"), ("user2", "pass2")];
        let auth = PasswordAuth::new().add_users(users);
        assert_eq!(auth.user_count(), 2);
        assert!(auth.has_user("user1"));
        assert!(auth.has_user("user2"));
    }

    #[tokio::test]
    async fn test_async_callback_auth() {
        let auth = AsyncCallbackAuth::new(Arc::new(|ctx: &AuthContext, password: &str| {
            let username = ctx.username().to_string();
            let password = password.to_string();
            let fut: std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send>> =
                Box::pin(async move { username == "admin" && password == "secret" });
            fut
        }));

        let ctx = make_context("admin");
        assert!(matches!(
            auth.auth_password(&ctx, "secret").await,
            AuthResult::Accept
        ));
        assert!(matches!(
            auth.auth_password(&ctx, "wrong").await,
            AuthResult::Reject
        ));
    }
}
