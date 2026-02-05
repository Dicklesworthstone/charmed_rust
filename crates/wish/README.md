# Wish

SSH server framework for serving bubbletea-based TUIs.

## TL;DR

**The Problem:** Shipping a TUI over SSH usually requires a lot of protocol and
session management boilerplate.

**The Solution:** Wish provides a middleware-based SSH server API that embeds
bubbletea programs per session.

**Why Wish**

- **SSH-first**: built on `russh`.
- **Middleware pipeline**: composable session handling.
- **Bubbletea integration**: serve TUIs per SSH connection.

## Role in the charmed_rust (FrankenTUI) stack

Wish is the deployment layer for remote TUIs. It uses `bubbletea` for session
programs, `lipgloss` for styling, and `charmed_log` for structured logging.

## Crates.io package

Package name: `charmed-wish`  
Library crate name: `wish`

## Installation

```toml
[dependencies]
wish = { package = "charmed-wish", version = "0.1.2" }
bubbletea = { package = "charmed-bubbletea", version = "0.1.2" }
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use wish::{ServerBuilder, println};

#[tokio::main]
async fn main() -> Result<(), wish::Error> {
    let server = ServerBuilder::new()
        .address("0.0.0.0:2222")
        .handler(|session| async move {
            println(&session, "Hello from Wish!");
            let _ = session.exit(0);
        })
        .build()?;

    server.listen().await
}
```

Connect with:

```bash
ssh -p 2222 localhost
```

## Bubbletea Integration

```rust
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model};
use wish::{ServerBuilder, Session};
use wish::middleware::logging;

struct Counter { count: i32, user: String }

impl Model for Counter {
    fn init(&self) -> Option<Cmd> { None }
    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            if matches!(key.key_type, KeyType::CtrlC) { return Some(bubbletea::quit()); }
        }
        None
    }
    fn view(&self) -> String { format!("Hello {}", self.user) }
}

#[tokio::main]
async fn main() -> Result<(), wish::Error> {
    let server = ServerBuilder::new()
        .address("0.0.0.0:2222")
        .with_middleware(logging::middleware())
        .with_middleware(wish::tea::middleware(|session: &Session| Counter {
            count: 0,
            user: session.user().to_string(),
        }))
        .build()?;

    server.listen().await
}
```

## Authentication

Wish supports multiple auth modes (password, public key, keyboard-interactive).
See `wish::auth` for helpers and policies.

## Troubleshooting

- **Connection refused**: ensure port is open and `address` is correct.
- **Key errors**: confirm host key permissions and file paths.
- **No TUI output**: ensure the session handler runs a bubbletea program.

## Limitations

- SSH depends on `russh` and its evolving API surface.
- Production deployments require robust auth and host key management.

## FAQ

**Can I serve multiple apps?**  
Yes. Route by user or session and spawn different models.

**Does it support PTY?**  
Yes, through the session integration in `russh`.

## About Contributions

Please don't take this the wrong way, but I do not accept outside contributions for any of my projects. I simply don't have the mental bandwidth to review anything, and it's my name on the thing, so I'm responsible for any problems it causes; thus, the risk-reward is highly asymmetric from my perspective. I'd also have to worry about other "stakeholders," which seems unwise for tools I mostly make for myself for free. Feel free to submit issues, and even PRs if you want to illustrate a proposed fix, but know I won't merge them directly. Instead, I'll have Claude or Codex review submissions via `gh` and independently decide whether and how to address them. Bug reports in particular are welcome. Sorry if this offends, but I want to avoid wasted time and hurt feelings. I understand this isn't in sync with the prevailing open-source ethos that seeks community contributions, but it's the only way I can move at this velocity and keep my sanity.

## License

MIT. See `LICENSE` at the repository root.
