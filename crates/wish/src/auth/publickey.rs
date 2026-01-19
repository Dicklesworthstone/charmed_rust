//! Public key authentication handlers.

use std::sync::Arc;

use async_trait::async_trait;
use tracing::debug;

use super::handler::{AuthContext, AuthHandler, AuthMethod, AuthResult};
use crate::PublicKey;

/// Callback-based public key authentication handler.
///
/// Uses a user-provided callback function to validate public keys.
///
/// # Example
///
/// ```rust,ignore
/// use wish::auth::PublicKeyCallbackAuth;
///
/// let auth = PublicKeyCallbackAuth::new(|ctx, key| {
///     // Check if key is in allowed list
///     allowed_keys.contains(key)
/// });
/// ```
pub struct PublicKeyCallbackAuth<F>
where
    F: Fn(&AuthContext, &PublicKey) -> bool + Send + Sync,
{
    callback: F,
}

impl<F> PublicKeyCallbackAuth<F>
where
    F: Fn(&AuthContext, &PublicKey) -> bool + Send + Sync,
{
    /// Creates a new callback-based public key auth handler.
    pub fn new(callback: F) -> Self {
        Self { callback }
    }
}

#[async_trait]
impl<F> AuthHandler for PublicKeyCallbackAuth<F>
where
    F: Fn(&AuthContext, &PublicKey) -> bool + Send + Sync + 'static,
{
    async fn auth_publickey(&self, ctx: &AuthContext, key: &PublicKey) -> AuthResult {
        debug!(
            username = %ctx.username(),
            remote_addr = %ctx.remote_addr(),
            key_type = %key.key_type,
            "PublicKeyCallbackAuth: auth attempt"
        );

        if (self.callback)(ctx, key) {
            debug!(username = %ctx.username(), "PublicKeyCallbackAuth: accepted");
            AuthResult::Accept
        } else {
            debug!(username = %ctx.username(), "PublicKeyCallbackAuth: rejected");
            AuthResult::Reject
        }
    }

    fn supported_methods(&self) -> Vec<AuthMethod> {
        vec![AuthMethod::PublicKey]
    }
}

/// Simple public key authentication against a static set of keys.
///
/// Stores public keys and validates against them.
///
/// # Example
///
/// ```rust,ignore
/// use wish::auth::PublicKeyAuth;
/// use wish::PublicKey;
///
/// let key = PublicKey::new("ssh-ed25519", key_data);
/// let auth = PublicKeyAuth::new().add_key(key);
/// ```
#[derive(Default)]
pub struct PublicKeyAuth {
    /// All allowed keys (regardless of user).
    global_keys: Vec<PublicKey>,
    /// Per-user allowed keys.
    user_keys: std::collections::HashMap<String, Vec<PublicKey>>,
}

impl PublicKeyAuth {
    /// Creates a new empty public key auth handler.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a global key that can authenticate any user.
    pub fn add_key(mut self, key: PublicKey) -> Self {
        self.global_keys.push(key);
        self
    }

    /// Adds multiple global keys.
    pub fn add_keys<I>(mut self, keys: I) -> Self
    where
        I: IntoIterator<Item = PublicKey>,
    {
        self.global_keys.extend(keys);
        self
    }

    /// Adds a key that can only authenticate a specific user.
    pub fn add_user_key(mut self, username: impl Into<String>, key: PublicKey) -> Self {
        self.user_keys
            .entry(username.into())
            .or_default()
            .push(key);
        self
    }

    /// Returns the number of global keys.
    pub fn global_key_count(&self) -> usize {
        self.global_keys.len()
    }

    /// Returns the number of keys for a specific user.
    pub fn user_key_count(&self, username: &str) -> usize {
        self.user_keys.get(username).map(|v| v.len()).unwrap_or(0)
    }

    /// Checks if a key is allowed for authentication.
    fn is_key_allowed(&self, username: &str, key: &PublicKey) -> bool {
        // Check global keys
        if self.global_keys.iter().any(|k| k == key) {
            return true;
        }

        // Check user-specific keys
        if let Some(user_keys) = self.user_keys.get(username) {
            if user_keys.iter().any(|k| k == key) {
                return true;
            }
        }

        false
    }
}

#[async_trait]
impl AuthHandler for PublicKeyAuth {
    async fn auth_publickey(&self, ctx: &AuthContext, key: &PublicKey) -> AuthResult {
        debug!(
            username = %ctx.username(),
            remote_addr = %ctx.remote_addr(),
            key_type = %key.key_type,
            "PublicKeyAuth: auth attempt"
        );

        if self.is_key_allowed(ctx.username(), key) {
            debug!(username = %ctx.username(), "PublicKeyAuth: accepted");
            AuthResult::Accept
        } else {
            debug!(username = %ctx.username(), "PublicKeyAuth: rejected");
            AuthResult::Reject
        }
    }

    fn supported_methods(&self) -> Vec<AuthMethod> {
        vec![AuthMethod::PublicKey]
    }
}

/// Async callback-based public key authentication handler.
///
/// Uses a user-provided async callback function to validate public keys.
/// Useful for database lookups or remote authentication services.
#[allow(dead_code)]
pub struct AsyncPublicKeyAuth<F>
where
    F: Fn(
            &AuthContext,
            &PublicKey,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send>>
        + Send
        + Sync,
{
    callback: Arc<F>,
}

impl<F> AsyncPublicKeyAuth<F>
where
    F: Fn(
            &AuthContext,
            &PublicKey,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send>>
        + Send
        + Sync,
{
    /// Creates a new async callback-based public key auth handler.
    #[allow(dead_code)]
    pub fn new(callback: Arc<F>) -> Self {
        Self { callback }
    }
}

#[async_trait]
impl<F> AuthHandler for AsyncPublicKeyAuth<F>
where
    F: Fn(
            &AuthContext,
            &PublicKey,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send>>
        + Send
        + Sync
        + 'static,
{
    async fn auth_publickey(&self, ctx: &AuthContext, key: &PublicKey) -> AuthResult {
        debug!(
            username = %ctx.username(),
            remote_addr = %ctx.remote_addr(),
            key_type = %key.key_type,
            "AsyncPublicKeyAuth: auth attempt"
        );

        if (self.callback)(ctx, key).await {
            debug!(username = %ctx.username(), "AsyncPublicKeyAuth: accepted");
            AuthResult::Accept
        } else {
            debug!(username = %ctx.username(), "AsyncPublicKeyAuth: rejected");
            AuthResult::Reject
        }
    }

    fn supported_methods(&self) -> Vec<AuthMethod> {
        vec![AuthMethod::PublicKey]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use super::super::SessionId;

    fn make_context(username: &str) -> AuthContext {
        let addr: SocketAddr = "127.0.0.1:22".parse().unwrap();
        AuthContext::new(username, addr, SessionId(1))
    }

    fn make_key(key_type: &str, data: &[u8]) -> PublicKey {
        PublicKey::new(key_type, data.to_vec())
    }

    #[tokio::test]
    async fn test_publickey_callback_auth() {
        let auth = PublicKeyCallbackAuth::new(|ctx, key| {
            ctx.username() == "alice" && key.key_type == "ssh-ed25519"
        });

        let ctx = make_context("alice");
        let key = make_key("ssh-ed25519", b"keydata");
        assert!(matches!(auth.auth_publickey(&ctx, &key).await, AuthResult::Accept));

        let key = make_key("ssh-rsa", b"keydata");
        assert!(matches!(auth.auth_publickey(&ctx, &key).await, AuthResult::Reject));

        let ctx = make_context("bob");
        let key = make_key("ssh-ed25519", b"keydata");
        assert!(matches!(auth.auth_publickey(&ctx, &key).await, AuthResult::Reject));
    }

    #[tokio::test]
    async fn test_publickey_auth_global() {
        let key1 = make_key("ssh-ed25519", b"key1");
        let key2 = make_key("ssh-ed25519", b"key2");
        let key3 = make_key("ssh-ed25519", b"key3");

        let auth = PublicKeyAuth::new()
            .add_key(key1.clone())
            .add_key(key2.clone());

        assert_eq!(auth.global_key_count(), 2);

        let ctx = make_context("anyone");
        assert!(matches!(auth.auth_publickey(&ctx, &key1).await, AuthResult::Accept));
        assert!(matches!(auth.auth_publickey(&ctx, &key2).await, AuthResult::Accept));
        assert!(matches!(auth.auth_publickey(&ctx, &key3).await, AuthResult::Reject));
    }

    #[tokio::test]
    async fn test_publickey_auth_per_user() {
        let alice_key = make_key("ssh-ed25519", b"alice_key");
        let bob_key = make_key("ssh-ed25519", b"bob_key");

        let auth = PublicKeyAuth::new()
            .add_user_key("alice", alice_key.clone())
            .add_user_key("bob", bob_key.clone());

        assert_eq!(auth.user_key_count("alice"), 1);
        assert_eq!(auth.user_key_count("bob"), 1);
        assert_eq!(auth.user_key_count("charlie"), 0);

        let ctx = make_context("alice");
        assert!(matches!(auth.auth_publickey(&ctx, &alice_key).await, AuthResult::Accept));
        assert!(matches!(auth.auth_publickey(&ctx, &bob_key).await, AuthResult::Reject));

        let ctx = make_context("bob");
        assert!(matches!(auth.auth_publickey(&ctx, &bob_key).await, AuthResult::Accept));
        assert!(matches!(auth.auth_publickey(&ctx, &alice_key).await, AuthResult::Reject));
    }

    #[tokio::test]
    async fn test_publickey_auth_add_keys() {
        let keys = vec![
            make_key("ssh-ed25519", b"key1"),
            make_key("ssh-ed25519", b"key2"),
        ];
        let auth = PublicKeyAuth::new().add_keys(keys);
        assert_eq!(auth.global_key_count(), 2);
    }
}
