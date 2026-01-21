# Authentication Guide

This guide covers authentication methods and best practices for Wish SSH servers.

## Overview

Wish supports three SSH authentication methods:
- **Password**: Username/password credentials
- **Public Key**: SSH key-based authentication
- **Keyboard-Interactive**: Challenge-response authentication

## Authentication Handlers

### AuthHandler Trait

All authentication is handled through the `AuthHandler` trait:

```rust
#[async_trait]
pub trait AuthHandler: Send + Sync {
    /// Authenticate using password.
    async fn auth_password(&self, ctx: &AuthContext, password: &str) -> AuthResult {
        AuthResult::Reject
    }

    /// Authenticate using public key.
    async fn auth_publickey(&self, ctx: &AuthContext, key: &PublicKey) -> AuthResult {
        AuthResult::Reject
    }

    /// Authenticate using keyboard-interactive.
    async fn auth_keyboard_interactive(&self, ctx: &AuthContext, response: &str) -> AuthResult {
        AuthResult::Reject
    }

    /// Return supported authentication methods.
    fn supported_methods(&self) -> Vec<AuthMethod> {
        vec![]
    }
}
```

### AuthContext

The `AuthContext` provides information about the authentication attempt:

```rust
pub struct AuthContext {
    username: String,
    remote_addr: SocketAddr,
    session_id: SessionId,
}

impl AuthContext {
    pub fn username(&self) -> &str { ... }
    pub fn remote_addr(&self) -> SocketAddr { ... }
    pub fn session_id(&self) -> SessionId { ... }
}
```

### AuthResult

Authentication results can be:

```rust
pub enum AuthResult {
    /// Authentication successful
    Accept,
    /// Authentication failed
    Reject,
    /// Partial success - continue with additional methods
    Partial { next_methods: Vec<AuthMethod> },
}
```

## Built-in Authentication Handlers

### AcceptAllAuth (Development Only)

Accepts all authentication attempts. **Never use in production!**

```rust
use wish::auth::AcceptAllAuth;

let server = ServerBuilder::new()
    .auth_handler(AcceptAllAuth::new())
    .build()?;
```

### PasswordAuth

Password-based authentication with a callback:

```rust
use wish::auth::{PasswordAuth, AuthContext};

// Simple callback
let auth = PasswordAuth::new(|ctx: &AuthContext, password: &str| {
    ctx.username() == "admin" && password == "secret123"
});

// With user database
let users: HashMap<String, String> = /* load from DB */;
let auth = PasswordAuth::new(move |ctx, pw| {
    users.get(ctx.username()).map(|p| p == pw).unwrap_or(false)
});
```

### AuthorizedKeysAuth

Public key authentication using OpenSSH `authorized_keys` format:

```rust
use wish::auth::AuthorizedKeysAuth;

// From file
let auth = AuthorizedKeysAuth::new("~/.ssh/authorized_keys")?;

// From string
let keys = "ssh-ed25519 AAAAC3... user@host";
let auth = AuthorizedKeysAuth::from_string(keys)?;
```

### PublicKeyAuth

Custom public key authentication:

```rust
use wish::auth::{PublicKeyAuth, PublicKey, AuthContext};

let auth = PublicKeyAuth::new(|ctx: &AuthContext, key: &PublicKey| {
    // Verify key against your database
    check_key_in_database(ctx.username(), key)
});
```

### AsyncCallbackAuth

For async authentication (e.g., database lookups):

```rust
use wish::auth::AsyncCallbackAuth;

let auth = AsyncCallbackAuth::new(|ctx, password| {
    Box::pin(async move {
        // Async database lookup
        let result = db.verify_user(ctx.username(), password).await;
        result.is_ok()
    })
});
```

### CompositeAuth

Combine multiple authentication methods:

```rust
use wish::auth::{CompositeAuth, AuthorizedKeysAuth, PasswordAuth};

let auth = CompositeAuth::new()
    .add(AuthorizedKeysAuth::new("~/.ssh/authorized_keys")?)
    .add(PasswordAuth::new(|ctx, pw| {
        // Fallback to password for guest user
        ctx.username() == "guest" && pw == "guest"
    }));
```

The composite handler tries each method in order until one accepts.

### RateLimitedAuth

Wrap any auth handler with rate limiting:

```rust
use wish::auth::RateLimitedAuth;

let inner = PasswordAuth::new(/* ... */);
let auth = RateLimitedAuth::new(inner)
    .with_rejection_delay(200)   // 200ms delay on failed auth
    .with_max_attempts(3);        // Max 3 attempts
```

## Custom Authentication

### Implementing AuthHandler

```rust
use wish::auth::{AuthHandler, AuthContext, AuthResult, AuthMethod};
use async_trait::async_trait;

struct MyDatabaseAuth {
    db_pool: DatabasePool,
}

#[async_trait]
impl AuthHandler for MyDatabaseAuth {
    async fn auth_password(&self, ctx: &AuthContext, password: &str) -> AuthResult {
        // Query database
        match self.db_pool.verify_password(ctx.username(), password).await {
            Ok(true) => AuthResult::Accept,
            Ok(false) => AuthResult::Reject,
            Err(e) => {
                tracing::error!("Database error: {}", e);
                AuthResult::Reject
            }
        }
    }

    async fn auth_publickey(&self, ctx: &AuthContext, key: &PublicKey) -> AuthResult {
        // Query database for user's keys
        match self.db_pool.get_user_keys(ctx.username()).await {
            Ok(keys) if keys.contains(key) => AuthResult::Accept,
            _ => AuthResult::Reject,
        }
    }

    fn supported_methods(&self) -> Vec<AuthMethod> {
        vec![AuthMethod::Password, AuthMethod::PublicKey]
    }
}
```

### Multi-Factor Authentication

Use `AuthResult::Partial` for multi-factor flows:

```rust
#[async_trait]
impl AuthHandler for MfaAuth {
    async fn auth_password(&self, ctx: &AuthContext, password: &str) -> AuthResult {
        if verify_password(ctx.username(), password) {
            // Password OK, require public key next
            AuthResult::Partial {
                next_methods: vec![AuthMethod::PublicKey],
            }
        } else {
            AuthResult::Reject
        }
    }

    async fn auth_publickey(&self, ctx: &AuthContext, key: &PublicKey) -> AuthResult {
        // Only called after password success
        if verify_key(ctx.username(), key) {
            AuthResult::Accept
        } else {
            AuthResult::Reject
        }
    }
}
```

## Security Best Practices

### 1. Never Use AcceptAllAuth in Production

```rust
// BAD - accepts anyone!
.auth_handler(AcceptAllAuth::new())

// GOOD - require authentication
.auth_handler(AuthorizedKeysAuth::new("~/.ssh/authorized_keys")?)
```

### 2. Use Rate Limiting

Prevent brute-force attacks:

```rust
let auth = RateLimitedAuth::new(my_auth)
    .with_rejection_delay(100)  // Timing attack mitigation
    .with_max_attempts(6);       // Limit attempts

let server = ServerBuilder::new()
    .auth_handler(auth)
    .max_auth_attempts(6)
    .auth_rejection_delay(100)
    .build()?;
```

### 3. Prefer Public Key Authentication

Public keys are more secure than passwords:

```rust
// Best: public key only
let auth = AuthorizedKeysAuth::new("~/.ssh/authorized_keys")?;

// OK: composite with public key preferred
let auth = CompositeAuth::new()
    .add(AuthorizedKeysAuth::new("~/.ssh/authorized_keys")?)
    .add(PasswordAuth::new(/* fallback */));
```

### 4. Log Authentication Events

```rust
#[async_trait]
impl AuthHandler for MyAuth {
    async fn auth_password(&self, ctx: &AuthContext, password: &str) -> AuthResult {
        let result = self.verify(ctx, password);

        match result {
            AuthResult::Accept => {
                tracing::info!(
                    user = %ctx.username(),
                    addr = %ctx.remote_addr(),
                    "Authentication successful"
                );
            }
            AuthResult::Reject => {
                tracing::warn!(
                    user = %ctx.username(),
                    addr = %ctx.remote_addr(),
                    "Authentication failed"
                );
            }
            _ => {}
        }

        result
    }
}
```

### 5. Secure Password Storage

Never store plaintext passwords. Use proper hashing:

```rust
use argon2::{Argon2, PasswordHash, PasswordVerifier};

fn verify_password(username: &str, password: &str) -> bool {
    let stored_hash = get_hash_from_db(username);
    let parsed_hash = PasswordHash::new(&stored_hash).ok();

    parsed_hash
        .map(|h| Argon2::default().verify_password(password.as_bytes(), &h).is_ok())
        .unwrap_or(false)
}
```

## Troubleshooting

### "Authentication failed" for valid credentials

Check:
1. Username matches exactly (case-sensitive)
2. Password/key format is correct
3. `supported_methods()` includes the method being used

### Public key not working

Verify:
1. Key format is correct (OpenSSH format)
2. Key is in `authorized_keys` file
3. File permissions are correct (600)
4. Key type is supported

### Rate limiting too aggressive

Adjust settings:
```rust
.with_rejection_delay(50)   // Reduce delay
.with_max_attempts(10)       // Allow more attempts
```
