# Wish Architecture

This document describes the internal architecture of the Wish SSH server library.

## Overview

Wish is built on a middleware-based architecture that allows flexible composition
of request handling logic. It uses the russh library for SSH protocol handling
and integrates with BubbleTea for TUI application support.

## System Architecture

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                          SSH Client                                      │
│                    (e.g., openssh, putty)                               │
└─────────────────────────────────────────────────────────────────────────┘
                                   │
                                   │ SSH Protocol (TCP)
                                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         Wish Server                                      │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                     russh Layer                                    │  │
│  │  - SSH protocol handling                                          │  │
│  │  - Key exchange                                                    │  │
│  │  - Channel management                                              │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                                   │                                      │
│                                   ▼                                      │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                 Authentication Layer                               │  │
│  │  - Password authentication                                         │  │
│  │  - Public key authentication                                       │  │
│  │  - Keyboard-interactive authentication                             │  │
│  │  - Rate limiting & timing attack mitigation                        │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                                   │                                      │
│                                   ▼                                      │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                  Middleware Chain                                  │  │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐              │  │
│  │  │Logging  │→ │RateLimit│→ │ActivePTY│→ │ Custom  │→ ...         │  │
│  │  └─────────┘  └─────────┘  └─────────┘  └─────────┘              │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                                   │                                      │
│                                   ▼                                      │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                    Session Handler                                 │  │
│  │  - Custom handler function                                         │  │
│  │  - BubbleTea program runner                                        │  │
│  │  - Subsystem handlers (e.g., SFTP)                                │  │
│  └───────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

## Core Components

### Server

The `Server` struct is the main entry point. It:
- Manages server configuration (`ServerOptions`)
- Creates russh configuration
- Accepts TCP connections
- Spawns handler tasks for each connection

```rust
pub struct Server {
    options: ServerOptions,
}
```

### Session

The `Session` struct represents a connected SSH client. It provides:
- User information (username, remote address)
- PTY information (terminal type, window size)
- I/O channels for reading/writing data
- Environment variables and command information

```rust
pub struct Session {
    context: Context,
    pty: Option<Pty>,
    command: Vec<String>,
    // ... I/O channels
}
```

### Middleware

Middleware wraps handlers to add cross-cutting concerns:

```rust
pub type Handler = Arc<dyn Fn(Session) -> BoxFuture<'static, ()> + Send + Sync>;
pub type Middleware = Arc<dyn Fn(Handler) -> Handler + Send + Sync>;
```

Middleware execution flows from outer to inner:

```text
Request → MW1 → MW2 → MW3 → Handler → MW3 → MW2 → MW1 → Response
```

### Authentication

The `AuthHandler` trait defines authentication behavior:

```rust
#[async_trait]
pub trait AuthHandler: Send + Sync {
    async fn auth_password(&self, ctx: &AuthContext, password: &str) -> AuthResult;
    async fn auth_publickey(&self, ctx: &AuthContext, key: &PublicKey) -> AuthResult;
    async fn auth_keyboard_interactive(&self, ctx: &AuthContext, response: &str) -> AuthResult;
}
```

## Data Flow

### Connection Lifecycle

1. **Accept**: TCP connection accepted by TcpListener
2. **Handshake**: SSH protocol handshake via russh
3. **Authenticate**: User credentials verified by AuthHandler
4. **Session**: PTY allocated, session created
5. **Handle**: Middleware chain executed, then handler
6. **Close**: Session closed, connection terminated

### I/O Flow

```text
Client Input → SSH Channel → Input Receiver → Session.recv()
                                                    ↓
                                              BubbleTea/Handler
                                                    ↓
Session.write() → Output Sender → SSH Channel → Client Output
```

## Concurrency Model

Wish uses Tokio for async I/O:

- **Main accept loop**: Single task accepts connections
- **Per-connection task**: Each connection gets a dedicated task
- **BubbleTea tasks**: TUI programs run in blocking task pool
- **Shared state**: Thread-safe via `Arc<RwLock<_>>`

```text
┌──────────────────────────────────────────────────────────────┐
│                    Tokio Runtime                              │
│  ┌─────────────────┐                                         │
│  │  Accept Loop    │                                         │
│  │  (single task)  │                                         │
│  └────────┬────────┘                                         │
│           │ spawn                                             │
│           ▼                                                   │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐  │
│  │ Connection 1    │  │ Connection 2    │  │ Connection N │  │
│  │ (task)          │  │ (task)          │  │ (task)       │  │
│  └────────┬────────┘  └─────────────────┘  └──────────────┘  │
│           │ spawn_blocking                                    │
│           ▼                                                   │
│  ┌─────────────────┐                                         │
│  │ BubbleTea       │                                         │
│  │ (blocking task) │                                         │
│  └─────────────────┘                                         │
└──────────────────────────────────────────────────────────────┘
```

## Extension Points

### Custom Middleware

Create middleware by wrapping the next handler:

```rust
fn my_middleware() -> Middleware {
    Arc::new(|next| {
        Arc::new(move |session| {
            let next = next.clone();
            Box::pin(async move {
                // Pre-processing
                next(session).await;
                // Post-processing
            })
        })
    })
}
```

### Custom Authentication

Implement the `AuthHandler` trait:

```rust
struct MyAuth;

#[async_trait]
impl AuthHandler for MyAuth {
    async fn auth_password(&self, ctx: &AuthContext, password: &str) -> AuthResult {
        // Custom logic
        AuthResult::Accept
    }
}
```

### Custom Handlers

Handlers are async functions that receive a Session:

```rust
.handler(|session| async move {
    // Handle the session
    println(&session, "Hello!");
    let _ = session.exit(0);
})
```

## Dependencies

- **russh**: SSH protocol implementation
- **russh-keys**: SSH key handling
- **tokio**: Async runtime
- **bubbletea**: TUI framework (optional)
- **lipgloss**: Terminal styling (optional)

## Thread Safety

All public types are `Send + Sync`:

- `Server` - can be shared across threads
- `Session` - cloneable, thread-safe I/O
- `AuthHandler` - requires `Send + Sync`
- `Middleware` - requires `Send + Sync`

## Performance Considerations

- Connection handling is fully async
- BubbleTea programs use blocking tasks to avoid blocking the runtime
- Rate limiting uses efficient LRU cache
- Session state uses `parking_lot` for fast locks
