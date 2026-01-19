# Error Handling in charmed_rust

**Date:** 2026-01-19

This guide covers error handling patterns and best practices for all charmed_rust crates.

## Overview

All charmed_rust crates use `thiserror` for error definitions, providing consistent error handling across the ecosystem. Each crate defines:

- An `Error` enum with descriptive variants
- A `Result<T>` type alias for convenience
- Proper error chaining with `#[from]` and `#[source]`

## Quick Start

### Using the ? Operator

The simplest way to handle errors is with the `?` operator:

```rust
use bubbletea::Result;

fn run_app() -> Result<()> {
    let model = Program::new(MyModel::default()).run()?;
    println!("Final state: {:?}", model);
    Ok(())
}
```

### Converting to anyhow

For applications that use `anyhow`, errors convert automatically:

```rust
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let model = bubbletea::Program::new(my_model)
        .run()
        .context("failed to run TUI program")?;
    Ok(())
}
```

### Pattern Matching

For fine-grained control, match on specific error variants:

```rust
use bubbletea::{Error, Program};

match Program::new(my_model).run() {
    Ok(model) => handle_success(model),
    Err(Error::Io(e)) => handle_io_error(e),
    Err(e) => return Err(e.into()),
}
```

## Error Types by Crate

### bubbletea

The core TUI framework defines `bubbletea::Error`:

| Variant | Description | Recovery |
|---------|-------------|----------|
| `Io` | Terminal I/O failure | Check terminal, retry |

```rust
use bubbletea::{Program, Error};

match Program::new(model).run() {
    Ok(final_model) => { /* success */ }
    Err(Error::Io(e)) => {
        eprintln!("Terminal error: {}", e);
        // Consider non-TUI fallback
    }
}
```

### huh

Interactive forms library defines `huh::FormError`:

| Variant | Description | Recovery |
|---------|-------------|----------|
| `UserAborted` | User pressed Ctrl+C | Normal exit path |
| `Timeout` | Form timed out | Retry or use default |
| `Validation` | Input validation failed | Show error, allow retry |
| `Io` | I/O error | Check terminal |

**Important:** `UserAborted` is not an error condition:

```rust
use huh::{Form, FormError};

match form.run() {
    Ok(()) => println!("Form completed"),
    Err(FormError::UserAborted) => {
        println!("Cancelled");
        return Ok(()); // Not an error!
    }
    Err(FormError::Validation(msg)) => {
        eprintln!("Invalid input: {}", msg);
        // Typically retry the form
    }
    Err(e) => return Err(e.into()),
}
```

### wish

SSH server library defines `wish::Error`:

| Variant | Description | Recovery |
|---------|-------------|----------|
| `Io` | Network I/O error | Check port, permissions |
| `Ssh` | SSH protocol error | Log and continue |
| `Russh` | Underlying russh error | Check compatibility |
| `Key` | Key generation error | Regenerate keys |
| `KeyLoad` | Key loading error | Check file permissions |
| `AuthenticationFailed` | Bad credentials | Expected, log warning |
| `Configuration` | Invalid config | Fix configuration |
| `Session` | Session error | Close session |
| `AddrParse` | Invalid address | Validate input |

**Important:** `AuthenticationFailed` is expected in normal operation:

```rust
use wish::Error;

match server.handle_connection(conn).await {
    Ok(()) => log::info!("Session completed"),
    Err(Error::AuthenticationFailed) => {
        log::warn!("Auth failed for {}", addr);
        // Normal, not a server error
    }
    Err(e) => log::error!("Session error: {}", e),
}
```

### glow

Markdown viewer defines errors in the `github` module:

**`ParseError`** - Repository reference parsing:

| Variant | Description |
|---------|-------------|
| `InvalidFormat` | Unrecognized format |
| `MissingOwnerOrRepo` | Incomplete reference |

**`FetchError`** - GitHub API operations:

| Variant | Description | Recovery |
|---------|-------------|----------|
| `Request` | Network failure | Retry with backoff |
| `ApiError` | HTTP error status | Check status code |
| `DecodeError` | Base64 decode failure | Report bug |
| `RateLimited` | API limit exceeded | Wait for reset |
| `CacheError` | Local cache I/O | Check disk |

```rust
use glow::github::{FetchError, GitHubFetcher};

match fetcher.fetch(&repo) {
    Ok(content) => render(content),
    Err(FetchError::RateLimited { reset_at: Some(ts) }) => {
        eprintln!("Rate limited until {}", ts);
    }
    Err(FetchError::ApiError { status: 404, .. }) => {
        eprintln!("Repository not found");
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

### charmed_log

Logging library defines `ParseLevelError`:

```rust
use charmed_log::Level;
use std::str::FromStr;

match Level::from_str(level_str) {
    Ok(level) => logger.set_level(level),
    Err(e) => {
        eprintln!("Invalid log level: {}", e);
        // Use default level
    }
}
```

## Error Chaining

All charmed_rust errors support the standard error chain via `Error::source()`:

```rust
fn log_error_chain(error: &dyn std::error::Error) {
    eprintln!("Error: {}", error);
    let mut source = error.source();
    while let Some(cause) = source {
        eprintln!("  Caused by: {}", cause);
        source = cause.source();
    }
}
```

With `anyhow`, chains display automatically:

```rust
fn main() -> anyhow::Result<()> {
    run_app()?; // Prints full chain on error
    Ok(())
}
```

## Cross-Crate Error Handling

When using multiple charmed crates together:

```rust
use anyhow::{Context, Result};

async fn main() -> Result<()> {
    // Form input with huh
    let config = huh::Form::new(fields)
        .run()
        .context("failed to get configuration")?;

    // SSH server with wish
    let server = wish::Server::new(handler)
        .await
        .context("failed to create SSH server")?;

    // TUI with bubbletea
    let model = bubbletea::Program::new(my_model)
        .run()
        .context("failed to run TUI")?;

    Ok(())
}
```

## Best Practices

### 1. Use `?` for Propagation

Don't manually match and re-wrap errors:

```rust
// Good
let data = fs::read(path)?;

// Avoid
let data = match fs::read(path) {
    Ok(d) => d,
    Err(e) => return Err(e.into()),
};
```

### 2. Add Context with anyhow

When propagating, add context for better debugging:

```rust
use anyhow::Context;

let config = load_config(path)
    .with_context(|| format!("failed to load config from {}", path))?;
```

### 3. Handle Expected "Errors" as Control Flow

Some errors are normal operation:

```rust
// User abort is not an error
if let Err(FormError::UserAborted) = form.run() {
    return Ok(());
}

// Auth failures are expected
if matches!(err, Error::AuthenticationFailed) {
    log::warn!("Auth failed");
    // Don't treat as server error
}
```

### 4. Log at Appropriate Levels

| Situation | Log Level |
|-----------|-----------|
| Recoverable error | WARN |
| Fatal error | ERROR |
| Expected failure (auth) | WARN or INFO |
| Detailed cause chain | DEBUG |

```rust
match result {
    Err(e) if e.is_recoverable() => {
        tracing::warn!(error = %e, "recoverable error");
    }
    Err(e) => {
        tracing::error!(error = %e, source = ?e.source(), "fatal error");
    }
    Ok(_) => {}
}
```

## Type Aliases

Each crate provides a `Result` type alias:

```rust
// Instead of
fn process() -> std::result::Result<Data, bubbletea::Error>

// Write
fn process() -> bubbletea::Result<Data>
```

For mixed error types, use the full path:

```rust
fn mixed() -> std::result::Result<(), anyhow::Error> {
    let result: bubbletea::Result<()> = program.run();
    result?;
    Ok(())
}
```

## Summary

| Crate | Error Type | Result Alias |
|-------|------------|--------------|
| bubbletea | `Error` | `Result<T>` |
| huh | `FormError` | `Result<T>` |
| wish | `Error` | `Result<T>` |
| glow::github | `ParseError`, `FetchError` | `ParseResult<T>`, `FetchResult<T>` |
| charmed_log | `ParseLevelError` | `ParseResult<T>` |

All error types:
- Use `thiserror` derive macros
- Support error chaining via `source()`
- Have descriptive Display implementations
- Convert to `anyhow::Error` automatically
