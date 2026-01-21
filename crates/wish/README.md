# Wish

SSH apps for Rust, powered by BubbleTea.

Wish makes it easy to build SSH servers that serve interactive terminal applications.
Based on the [Charm wish library](https://github.com/charmbracelet/wish) for Go,
this Rust port provides a middleware-based API for handling SSH connections with full
BubbleTea TUI integration.

## Features

- **SSH Server**: Full SSH server implementation using russh
- **Authentication**: Password, public key, and keyboard-interactive auth
- **BubbleTea Integration**: Serve interactive TUI apps over SSH
- **Middleware Pattern**: Composable request processing pipeline
- **Session Management**: Track and manage connected sessions
- **PTY Support**: Full pseudo-terminal emulation

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
wish = { version = "0.1", path = "../wish" }
bubbletea = { version = "0.1", path = "../bubbletea" }
tokio = { version = "1", features = ["full"] }
```

## Quick Start

### Basic Echo Server

```rust
use wish::{ServerBuilder, println};

#[tokio::main]
async fn main() -> Result<(), wish::Error> {
    let server = ServerBuilder::new()
        .address("0.0.0.0:2222")
        .handler(|session| async move {
            println(&session, "Hello from Wish!");
            println(&session, format!("User: {}", session.user()));
            let _ = session.exit(0);
        })
        .build()?;

    server.listen().await
}
```

Connect with: `ssh -p 2222 localhost`

### Serving a BubbleTea Application

```rust
use bubbletea::{Model, Cmd, tea};
use wish::{ServerBuilder, Session};
use wish::middleware::logging;

struct Counter {
    count: i32,
    user: String,
}

impl Model for Counter {
    type Message = CounterMsg;

    fn update(&mut self, msg: Self::Message) -> Cmd<Self::Message> {
        match msg {
            CounterMsg::Increment => self.count += 1,
            CounterMsg::Decrement => self.count -= 1,
            CounterMsg::Quit => return tea::quit(),
        }
        Cmd::none()
    }

    fn view(&self) -> String {
        format!(
            "Hello, {}!\n\nCount: {}\n\n[+] increment  [-] decrement  [q] quit",
            self.user, self.count
        )
    }
}

#[tokio::main]
async fn main() -> Result<(), wish::Error> {
    let server = ServerBuilder::new()
        .address("0.0.0.0:2222")
        .with_middleware(logging::middleware())
        .with_middleware(wish::tea::middleware(|session: &Session| {
            Counter {
                count: 0,
                user: session.user().to_string(),
            }
        }))
        .build()?;

    server.listen().await
}
```

## Authentication

### Accept All (Development Only)

```rust
use wish::auth::AcceptAllAuth;

let server = ServerBuilder::new()
    .auth_handler(AcceptAllAuth::new())
    // ...
```

### Public Key Authentication

```rust
use wish::auth::AuthorizedKeysAuth;

let auth = AuthorizedKeysAuth::new("~/.ssh/authorized_keys")?;
let server = ServerBuilder::new()
    .auth_handler(auth)
    // ...
```

### Password Authentication

```rust
use wish::auth::{PasswordAuth, AuthContext};

let auth = PasswordAuth::new(|ctx: &AuthContext, password: &str| {
    ctx.username() == "admin" && password == "secret"
});
let server = ServerBuilder::new()
    .auth_handler(auth)
    // ...
```

### Composite Authentication

```rust
use wish::auth::{CompositeAuth, AuthorizedKeysAuth, PasswordAuth};

let auth = CompositeAuth::new()
    .add(AuthorizedKeysAuth::new("~/.ssh/authorized_keys")?)
    .add(PasswordAuth::new(|ctx, pw| ctx.username() == "guest" && pw == "guest"));

let server = ServerBuilder::new()
    .auth_handler(auth)
    // ...
```

## Built-in Middleware

### Logging

```rust
use wish::middleware::logging;

// Basic logging
ServerBuilder::new()
    .with_middleware(logging::middleware())

// Structured logging
ServerBuilder::new()
    .with_middleware(logging::structured_middleware())
```

### Active Terminal Check

```rust
use wish::middleware::activeterm;

// Require PTY allocation
ServerBuilder::new()
    .with_middleware(activeterm::middleware())
```

### Access Control

```rust
use wish::middleware::accesscontrol;

// Restrict allowed commands
ServerBuilder::new()
    .with_middleware(accesscontrol::middleware(vec![
        "git-receive-pack".to_string(),
        "git-upload-pack".to_string(),
    ]))
```

### Rate Limiting

```rust
use wish::middleware::ratelimiter;

// Token-bucket rate limiter
let limiter = ratelimiter::new_rate_limiter(1.0, 10, 1000);
ServerBuilder::new()
    .with_middleware(ratelimiter::middleware(limiter))
```

## Session API

The `Session` object provides access to connection information:

```rust
session.user()          // Username
session.remote_addr()   // Client address
session.local_addr()    // Server address
session.pty()           // PTY info (terminal type, window size)
session.command()       // Command being executed
session.public_key()    // Authentication public key
session.environ()       // Environment variables
session.window()        // Current window dimensions
```

## Output Functions

```rust
use wish::{print, println, error, errorln, fatal};

println(&session, "Hello, world!");           // stdout with newline
print(&session, "No newline");                // stdout without newline
errorln(&session, "Error message");           // stderr with newline
fatal(&session, "Fatal error, exiting...");   // stderr + exit(1)
```

## Server Configuration

```rust
use std::time::Duration;

let server = ServerBuilder::new()
    .address("0.0.0.0:2222")                    // Listen address
    .version("SSH-2.0-MyApp")                   // SSH version string
    .banner("Welcome to MyApp!")               // Auth banner
    .host_key_path("/path/to/host_key")        // Persistent host key
    .idle_timeout(Duration::from_secs(300))    // Connection timeout
    .max_auth_attempts(3)                      // Auth attempt limit
    .auth_rejection_delay(100)                 // Timing attack mitigation
    .build()?;
```

## Examples

See the `examples/` directory for complete examples:

- `echo_server.rs` - Basic SSH server with greeting message
- `bubbletea_counter.rs` - Interactive counter app over SSH
- `authenticated_server.rs` - Server with public key authentication
- `middleware_example.rs` - Custom middleware demonstration

## Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                     SSH Client                              │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                   Wish Server                               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                  │
│  │   Auth   │→ │Middleware│→ │ Handler  │                  │
│  └──────────┘  └──────────┘  └──────────┘                  │
│                                    │                        │
│                                    ▼                        │
│                          ┌──────────────┐                   │
│                          │  BubbleTea   │                   │
│                          │   Program    │                   │
│                          └──────────────┘                   │
└─────────────────────────────────────────────────────────────┘
```

## Security Considerations

- **Never use `AcceptAllAuth` in production** - it accepts any credentials
- Use public key authentication for production deployments
- Consider rate limiting to prevent brute-force attacks
- Set appropriate timeouts to prevent resource exhaustion
- Use persistent host keys for production (prevents host key warnings)

## License

MIT
