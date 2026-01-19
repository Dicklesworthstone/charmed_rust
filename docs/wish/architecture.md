# Wish SSH Server Architecture

> Research document for porting Go Wish to Rust using russh.

---

## Overview

This document analyzes the russh crate API and SSH protocol patterns to guide the implementation of the Wish SSH server. It maps Go Wish features to russh equivalents and identifies any gaps or limitations.

---

## 1. russh Handler Trait Analysis

The `russh::server::Handler` trait is the core interface for implementing SSH server behavior. It uses async-trait for non-blocking operations.

### Required Type

```rust
type Error: From<russh::Error> + Send;
```

The associated error type must be convertible from `russh::Error`.

### Authentication Methods

| Method | Signature | Purpose |
|--------|-----------|---------|
| `auth_none` | `async fn auth_none(&mut self, user: &str) -> Result<Auth, Self::Error>` | "none" authentication method |
| `auth_password` | `async fn auth_password(&mut self, user: &str, password: &str) -> Result<Auth, Self::Error>` | Password authentication |
| `auth_publickey` | `async fn auth_publickey(&mut self, user: &str, public_key: &PublicKey) -> Result<Auth, Self::Error>` | Public key authentication |
| `auth_keyboard_interactive` | `async fn auth_keyboard_interactive(&mut self, user: &str, submethods: &str, response: Option<Response>) -> Result<Auth, Self::Error>` | Keyboard-interactive auth |
| `auth_openssh_certificate` | `async fn auth_openssh_certificate(&mut self, user: &str, certificate: &Certificate) -> Result<Auth, Self::Error>` | OpenSSH certificate auth |

**Return Type: `Auth`**
```rust
pub enum Auth {
    /// Accept authentication
    Accept,
    /// Reject authentication (constant-time rejection enforced by russh)
    Reject { proceed_with_methods: Option<MethodSet> },
    /// Partial success (for multi-factor auth)
    Partial { name: Cow<'static, str>, instructions: Cow<'static, str>, prompts: Cow<'static, [Prompt]> },
    /// User does not exist (helps hide valid usernames)
    UnsupportedMethod,
}
```

### Channel Lifecycle Methods

| Method | Signature | Purpose |
|--------|-----------|---------|
| `channel_open_session` | `async fn channel_open_session(&mut self, channel: Channel<Msg>, session: &mut Session) -> Result<bool, Self::Error>` | New session channel |
| `channel_open_direct_tcpip` | `async fn channel_open_direct_tcpip(...)` | TCP/IP port forwarding |
| `channel_close` | `async fn channel_close(&mut self, channel: ChannelId, session: &mut Session) -> Result<(), Self::Error>` | Channel closed by client |
| `channel_eof` | `async fn channel_eof(&mut self, channel: ChannelId, session: &mut Session) -> Result<(), Self::Error>` | Client sent EOF |

### PTY and Shell Methods

| Method | Signature | Purpose |
|--------|-----------|---------|
| `pty_request` | `async fn pty_request(&mut self, channel: ChannelId, term: &str, col_width: u32, row_height: u32, pix_width: u32, pix_height: u32, modes: &[(Pty, u32)], session: &mut Session) -> Result<(), Self::Error>` | PTY allocation |
| `shell_request` | `async fn shell_request(&mut self, channel: ChannelId, session: &mut Session) -> Result<(), Self::Error>` | Shell request |
| `exec_request` | `async fn exec_request(&mut self, channel: ChannelId, data: &[u8], session: &mut Session) -> Result<(), Self::Error>` | Execute command |
| `subsystem_request` | `async fn subsystem_request(&mut self, channel: ChannelId, name: &str, session: &mut Session) -> Result<(), Self::Error>` | Subsystem (e.g., sftp) |
| `window_change_request` | `async fn window_change_request(&mut self, channel: ChannelId, col_width: u32, row_height: u32, pix_width: u32, pix_height: u32, session: &mut Session) -> Result<(), Self::Error>` | Window resize |

### Data Transfer Methods

| Method | Signature | Purpose |
|--------|-----------|---------|
| `data` | `async fn data(&mut self, channel: ChannelId, data: &[u8], session: &mut Session) -> Result<(), Self::Error>` | Data from client |
| `extended_data` | `async fn extended_data(&mut self, channel: ChannelId, code: u32, data: &[u8], session: &mut Session) -> Result<(), Self::Error>` | Extended data (stderr) |

### Session Methods for Response

The `Session` object provides methods to respond to the client:

```rust
// Send data to client
session.data(channel_id, CryptoVec::from(data));

// Send success/failure for channel requests
session.channel_success(channel_id);
session.channel_failure(channel_id);

// Send EOF and close
session.eof(channel_id);
session.close(channel_id);

// Exit status
session.exit_status_request(channel_id, exit_code);
```

---

## 2. Server Configuration

### russh::server::Config

```rust
pub struct Config {
    /// Server version string (default: "SSH-2.0-Russh-0.1")
    pub server_id: SshId,

    /// Server authentication methods
    pub methods: MethodSet,

    /// Algorithms preferences
    pub preferred: Preferred,

    /// Private keys for host authentication
    pub keys: Vec<PrivateKey>,

    /// Connection timeout (initial auth)
    pub connection_timeout: Option<Duration>,

    /// Inactivity timeout
    pub inactivity_timeout: Option<Duration>,

    /// Time to wait before rejecting auth (constant-time defense)
    pub auth_rejection_time: Duration,

    /// Initial auth rejection delay
    pub auth_rejection_time_initial: Option<Duration>,

    /// Max auth attempts
    pub auth_max_attempts: usize,

    /// Window size for flow control
    pub window_size: u32,

    /// Maximum packet size
    pub maximum_packet_size: u32,

    /// Limits for parallel requests
    pub limits: Limits,
}
```

### Key Generation

```rust
use russh_keys::{key::PrivateKey, Algorithm};
use rand::rngs::OsRng;

// Generate ephemeral key
let key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519)?;

// Load from file
let key = russh_keys::load_secret_key(path, passphrase)?;

// Load from PEM string
let key = russh_keys::decode_secret_key(pem_data, passphrase)?;
```

---

## 3. Go Wish Feature Mapping

### Core Features

| Go Wish Feature | russh Equivalent | Notes |
|-----------------|------------------|-------|
| `Server.ListenAndServe()` | `russh::server::run_on_socket()` | Both use async accept loop |
| `WithAddress(addr)` | `TcpListener::bind(addr)` | Manual binding required |
| `WithHostKeyPath(path)` | `russh_keys::load_secret_key()` | Key loading separate from config |
| `WithPasswordAuth(fn)` | `Handler::auth_password()` | Trait method |
| `WithPublicKeyAuth(fn)` | `Handler::auth_publickey()` | Trait method |
| `WithKeyboardInteractiveAuth(fn)` | `Handler::auth_keyboard_interactive()` | Trait method |
| `WithIdleTimeout(d)` | `Config::inactivity_timeout` | Direct mapping |
| `WithMaxTimeout(d)` | `Config::connection_timeout` | Direct mapping |

### Session/Context

| Go Wish Type | Rust Equivalent | Notes |
|--------------|-----------------|-------|
| `wish.Session` | Custom struct wrapping russh types | Need to create |
| `wish.Context` | Custom context struct | Already exists in wish crate |
| `wish.Pty` | Custom Pty struct | Already exists |
| `ssh.PublicKey` | `russh_keys::key::PublicKey` | Use russh-keys type |

### Middleware Pattern

| Go Wish Pattern | Rust Implementation |
|-----------------|---------------------|
| `Middleware = func(Handler) Handler` | `type Middleware = Arc<dyn Fn(Handler) -> Handler>` |
| Middleware composition (LIFO) | Same pattern works |
| `activeterm.Middleware()` | Exists in wish crate |
| `logging.Middleware()` | Exists in wish crate |
| `accesscontrol.Middleware()` | Exists in wish crate |

### BubbleTea Integration

| Go Wish Pattern | Rust Implementation |
|-----------------|---------------------|
| `bm.Handler(func(*tea.Program))` | `tea::middleware()` exists |
| PTY → Window size → tea.Program | Need to wire russh → bubbletea |
| WindowSizeMsg on resize | Map `window_change_request` → WindowSizeMsg |

---

## 4. Implementation Architecture

### Connection Handler Flow

```
1. TcpListener::accept() → TcpStream
2. russh::server::run_on_socket(config, stream, handler)
3. SSH handshake (automatic)
4. Authentication callbacks
5. Channel open callbacks
6. PTY/shell/exec requests
7. Data flow
8. Channel close
```

### Proposed Structure

```
crates/wish/
├── src/
│   ├── lib.rs           # Re-exports, prelude
│   ├── server.rs        # Server, ServerBuilder (existing)
│   ├── config.rs        # ServerConfig, ServerOptions (enhance)
│   ├── error.rs         # Error types (existing)
│   ├── session.rs       # Session wrapper (existing, enhance)
│   ├── handler.rs       # NEW: russh Handler implementation
│   ├── connection.rs    # NEW: Per-connection state management
│   ├── middleware.rs    # Middleware module (existing)
│   └── tea.rs           # BubbleTea integration (existing, enhance)
```

### New Handler Implementation

```rust
use russh::server::{Auth, Handler as RusshHandler, Session as RusshSession};

pub struct WishHandler {
    /// Connection-specific state
    connection_id: u64,

    /// User after authentication
    user: Option<String>,

    /// Public key if auth'd via key
    public_key: Option<russh_keys::key::PublicKey>,

    /// PTY info if allocated
    pty: Option<Pty>,

    /// Window dimensions
    window: Window,

    /// Server-level shared state
    server_state: Arc<ServerState>,

    /// Active channels
    channels: HashMap<ChannelId, ChannelState>,
}

struct ChannelState {
    /// The wish Session for this channel
    session: crate::Session,

    /// Data sender to handler
    input_tx: mpsc::Sender<Vec<u8>>,

    /// Whether shell/exec has started
    started: bool,
}

#[async_trait]
impl RusshHandler for WishHandler {
    type Error = crate::Error;

    async fn auth_publickey(
        &mut self,
        user: &str,
        public_key: &russh_keys::key::PublicKey,
    ) -> Result<Auth, Self::Error> {
        // Delegate to ServerState's auth handler
        if let Some(handler) = &self.server_state.public_key_handler {
            let ctx = self.make_context(user);
            let pk = convert_public_key(public_key);
            if handler(&ctx, &pk) {
                self.user = Some(user.to_string());
                self.public_key = Some(public_key.clone());
                return Ok(Auth::Accept);
            }
        }
        Ok(Auth::Reject { proceed_with_methods: None })
    }

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        session: &mut RusshSession,
    ) -> Result<bool, Self::Error> {
        // Create channel state and wish Session
        let (tx, rx) = mpsc::channel(1024);
        let wish_session = crate::Session::new(self.make_context(&self.user.clone().unwrap_or_default()));

        self.channels.insert(channel.id(), ChannelState {
            session: wish_session,
            input_tx: tx,
            started: false,
        });

        Ok(true)
    }

    async fn pty_request(
        &mut self,
        channel: ChannelId,
        term: &str,
        col_width: u32,
        row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        _modes: &[(russh::Pty, u32)],
        session: &mut RusshSession,
    ) -> Result<(), Self::Error> {
        let pty = Pty {
            term: term.to_string(),
            window: Window { width: col_width, height: row_height },
        };
        self.pty = Some(pty.clone());

        if let Some(state) = self.channels.get_mut(&channel) {
            state.session = state.session.clone().with_pty(pty);
        }

        session.channel_success(channel)?;
        Ok(())
    }

    async fn shell_request(
        &mut self,
        channel: ChannelId,
        session: &mut RusshSession,
    ) -> Result<(), Self::Error> {
        // Start the handler for this channel
        if let Some(state) = self.channels.get_mut(&channel) {
            state.started = true;

            // Spawn handler task
            let session_clone = state.session.clone();
            let handler = self.server_state.handler.clone();

            tokio::spawn(async move {
                if let Some(h) = handler {
                    h(session_clone).await;
                }
            });
        }

        session.channel_success(channel)?;
        Ok(())
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        _session: &mut RusshSession,
    ) -> Result<(), Self::Error> {
        // Forward data to channel handler
        if let Some(state) = self.channels.get(&channel) {
            let _ = state.input_tx.send(data.to_vec()).await;
        }
        Ok(())
    }

    async fn window_change_request(
        &mut self,
        channel: ChannelId,
        col_width: u32,
        row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        _session: &mut RusshSession,
    ) -> Result<(), Self::Error> {
        self.window = Window { width: col_width, height: row_height };

        // Notify handler via channel
        if let Some(state) = self.channels.get(&channel) {
            // Send window resize message
            // This would be handled by bubbletea integration
        }
        Ok(())
    }
}
```

---

## 5. Identified Gaps and Limitations

### russh Limitations

1. **No built-in session abstraction** - Must create our own Session type wrapping russh primitives
2. **Error type requirements** - Must implement `From<russh::Error>` for our error type
3. **Handler is per-connection** - Need factory pattern for creating handlers

### Implementation Challenges

1. **Async handler composition** - Our middleware pattern is sync; need to adapt for async russh
2. **Input/Output bridging** - Need to connect russh channels to wish Session I/O
3. **BubbleTea integration** - Program expects synchronous I/O; need async adapter

### Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| Middleware async adaptation | Medium | Wrap in spawn, use channels |
| Session I/O bridging | Low | Use tokio channels and async wrappers |
| BubbleTea input handling | Medium | Create async input stream adapter |
| Error handling | Low | Implement From traits, use thiserror |

---

## 6. Implementation Recommendations

### Phase 1: Basic Server

1. Implement `WishHandler` struct with minimal auth (accept all)
2. Implement `channel_open_session`, `pty_request`, `shell_request`
3. Implement basic `data` forwarding
4. Create `Server::listen()` using `russh::server::run_on_socket()`

### Phase 2: Authentication

1. Wire public key auth to existing `PublicKeyHandler`
2. Wire password auth to existing `PasswordHandler`
3. Add keyboard-interactive support
4. Implement constant-time rejection (automatic via russh)

### Phase 3: BubbleTea Integration

1. Create async input stream adapter
2. Handle `window_change_request` → `WindowSizeMsg`
3. Bridge Session stdout to russh channel data
4. Test with simple TUI app

### Phase 4: Middleware Enhancement

1. Adapt middleware for async context
2. Ensure proper ordering (LIFO composition)
3. Add connection lifecycle hooks

---

## References

- [russh crate documentation](https://docs.rs/russh)
- [russh GitHub repository](https://github.com/Eugeny/russh)
- [russh Handler trait](https://docs.rs/russh/latest/russh/server/trait.Handler.html)
- [Go Wish repository](https://github.com/charmbracelet/wish)
- SSH Protocol RFCs: 4251, 4252, 4253, 4254
