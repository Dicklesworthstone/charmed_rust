# Unified Error Pattern Guide - charmed_rust

**Date:** 2026-01-19
**Bead:** charmed_rust-ea9
**Status:** Complete

## Overview

This guide documents the unified error handling pattern for all charmed_rust crates. The pattern is based on `wish::Error` and uses `thiserror` for ergonomic, consistent error types across the monorepo.

---

## The Standard Error Pattern

### Template

Every crate with fallible operations should define its error type following this pattern:

```rust
use std::io;
use thiserror::Error;

/// Errors that can occur in [crate_name].
#[derive(Error, Debug)]
pub enum Error {
    // === External Errors (with #[from] for automatic conversion) ===

    /// I/O error.
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    // === Domain-Specific Errors (with context) ===

    /// Configuration error.
    #[error("configuration error: {0}")]
    Configuration(String),

    // === Structured Errors (with named fields) ===

    /// Operation failed with context.
    #[error("{operation} failed: {reason}")]
    OperationFailed {
        operation: &'static str,
        reason: String,
    },

    // === Errors from Other Charmed Crates (with #[source]) ===

    /// Underlying bubbletea error.
    #[error("bubbletea error")]
    Bubbletea(#[source] bubbletea::Error),

    // === Sentinel Errors (no data) ===

    /// User cancelled the operation.
    #[error("user cancelled")]
    Cancelled,
}

/// Result type alias for [crate_name] operations.
pub type Result<T> = std::result::Result<T, Error>;
```

---

## Pattern Elements

### 1. Derive Macros

Always use:
```rust
#[derive(Error, Debug)]
```

For simple enums that need `Clone` or `Copy`:
```rust
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
```

### 2. The `#[from]` Attribute

Use `#[from]` for automatic `From<T>` implementation:

```rust
// This:
#[error("io error: {0}")]
Io(#[from] io::Error),

// Generates:
impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}
```

**When to use `#[from]`:**
- External library errors (io::Error, reqwest::Error, etc.)
- Standard library errors (ParseIntError, Utf8Error, etc.)
- Direct wrapping without additional context

**When NOT to use `#[from]`:**
- When you need to add context
- When the source error type appears in multiple variants
- For errors from other charmed crates (use `#[source]` instead)

### 3. The `#[source]` Attribute

Use `#[source]` to implement `Error::source()` without automatic `From`:

```rust
/// Underlying error from bubbletea.
#[error("bubbletea error: {0}")]
Bubbletea(#[source] bubbletea::Error),
```

**When to use `#[source]`:**
- Errors from other charmed_rust crates
- When you want error chaining but need manual conversion
- When you want to add context to the wrapped error

### 4. Error Message Format

Use the `#[error(...)]` attribute for `Display` implementation:

```rust
// Simple message with field
#[error("configuration error: {0}")]
Configuration(String),

// Named fields
#[error("{operation} failed: {reason}")]
OperationFailed { operation: &'static str, reason: String },

// Conditional formatting
#[error("rate limited{}", .reset_at.map(|t| format!(", resets at {t}")).unwrap_or_default())]
RateLimited { reset_at: Option<u64> },

// No fields (sentinel)
#[error("user cancelled")]
Cancelled,
```

### 5. Result Type Alias

Always provide a type alias:

```rust
/// Result type for [crate_name] operations.
pub type Result<T> = std::result::Result<T, Error>;
```

This allows:
```rust
// Instead of:
fn process() -> std::result::Result<Data, Error> { ... }

// Write:
fn process() -> Result<Data> { ... }
```

---

## Naming Conventions

### Error Type Names

| Type | Convention | Example |
|------|------------|---------|
| Main crate error | `Error` | `pub enum Error` |
| Domain-specific | `{Domain}Error` | `pub enum ParseError` |
| Struct errors | `{Noun}Error` | `pub struct ParseLevelError` |

### Variant Names

| Category | Convention | Example |
|----------|------------|---------|
| External errors | Source type name | `Io`, `Russh`, `Reqwest` |
| Domain errors | Action or noun | `Configuration`, `Session` |
| Sentinel errors | Past tense or state | `Cancelled`, `Timeout`, `AuthenticationFailed` |
| Structured errors | Action + Failed | `OperationFailed`, `ValidationFailed` |

### Message Style

1. **Use lowercase** (Rust convention):
   - Good: `"configuration error: {0}"`
   - Bad: `"Configuration Error: {0}"`

2. **Be concise but informative:**
   - Good: `"io error: {0}"`
   - Bad: `"An I/O error occurred while performing the operation: {0}"`

3. **Include context in structured variants:**
   - Good: `"{operation} failed: {reason}"`
   - Bad: `"error: {reason}"`

4. **No trailing punctuation:**
   - Good: `"user cancelled"`
   - Bad: `"User cancelled."`

---

## Error Variant Categories

### 1. External Library Errors

Wrap external errors with `#[from]`:

```rust
#[error("io error: {0}")]
Io(#[from] io::Error),

#[error("russh error: {0}")]
Russh(#[from] russh::Error),

#[error("request error: {0}")]
Request(#[from] reqwest::Error),
```

### 2. String-Context Errors

For errors that need dynamic context:

```rust
#[error("configuration error: {0}")]
Configuration(String),

#[error("key error: {0}")]
Key(String),

#[error("session error: {0}")]
Session(String),
```

Create these with:
```rust
return Err(Error::Configuration("missing required field 'host'".into()));
```

### 3. Structured Errors

For errors with multiple pieces of context:

```rust
#[error("API error ({status}): {message}")]
ApiError {
    status: u16,
    message: String,
},

#[error("{operation} failed: {reason}")]
OperationFailed {
    operation: &'static str,
    reason: String,
},
```

### 4. Sentinel Errors

For errors that are self-describing:

```rust
#[error("authentication failed")]
AuthenticationFailed,

#[error("user cancelled")]
Cancelled,

#[error("timeout")]
Timeout,
```

---

## Cross-Crate Error Handling

### Propagating Errors Between Crates

When one charmed crate uses another, wrap errors with `#[source]`:

```rust
// In huh/src/lib.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FormError {
    /// Error from the underlying bubbletea program.
    #[error("program error")]
    Program(#[source] bubbletea::Error),

    // ... other variants
}
```

### Converting Between Error Types

For explicit conversion with context:

```rust
impl From<bubbletea::Error> for FormError {
    fn from(e: bubbletea::Error) -> Self {
        FormError::Program(e)
    }
}
```

Or use `.map_err()` for inline conversion:

```rust
program.run().map_err(FormError::Program)?;
```

---

## Special Cases

### Clone/Copy Requirements

When errors need `Clone` (e.g., for `PartialEq` in tests):

```rust
// Option 1: Store String instead of io::Error
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum FormError {
    #[error("io error: {0}")]
    Io(String),  // Store message, not error
}

// Option 2: Use Arc for shared ownership
use std::sync::Arc;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("io error: {0}")]
    Io(Arc<io::Error>),
}
```

### Proc-Macro Errors

Proc-macro crates should NOT use thiserror. They need special handling:

```rust
// In bubbletea-macros/src/error.rs
#[derive(Debug)]
pub enum MacroError {
    Parse(syn::Error),
    // ...
}

impl MacroError {
    pub fn to_compile_error(&self) -> proc_macro2::TokenStream {
        // Convert to compile_error!() call
    }
}
```

### Simple Struct Errors

For single-value errors:

```rust
#[derive(Error, Debug, Clone)]
#[error("invalid level: {0:?}")]
pub struct ParseLevelError(String);
```

---

## Migration Checklist

When migrating an existing error type:

1. [ ] Add `thiserror = "2.0"` to `Cargo.toml` (workspace dependency)
2. [ ] Replace manual derives with `#[derive(Error, Debug)]`
3. [ ] Add `#[error(...)]` to each variant
4. [ ] Add `#[from]` where `From` was manually implemented
5. [ ] Add `#[source]` for error chaining
6. [ ] Remove manual `Display`, `Error`, and `From` implementations
7. [ ] Add `pub type Result<T>` alias if not present
8. [ ] Verify `Display` output matches previous (add tests if needed)
9. [ ] Run `cargo test` and `cargo clippy`
10. [ ] Update documentation

---

## Example: Complete Error Type

Here's `wish::Error` as the reference implementation:

```rust
use std::io;
use thiserror::Error;

/// Errors that can occur in the wish library.
#[derive(Error, Debug)]
pub enum Error {
    /// I/O error.
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    /// SSH protocol error.
    #[error("ssh error: {0}")]
    Ssh(String),

    /// russh error.
    #[error("russh error: {0}")]
    Russh(#[from] russh::Error),

    /// Key generation or loading error.
    #[error("key error: {0}")]
    Key(String),

    /// Key loading error from russh-keys.
    #[error("key loading error: {0}")]
    KeyLoad(#[from] russh_keys::Error),

    /// Authentication failed.
    #[error("authentication failed")]
    AuthenticationFailed,

    /// Server configuration error.
    #[error("configuration error: {0}")]
    Configuration(String),

    /// Session error.
    #[error("session error: {0}")]
    Session(String),

    /// Address parse error.
    #[error("address parse error: {0}")]
    AddrParse(#[from] std::net::AddrParseError),
}

/// Result type for wish operations.
pub type Result<T> = std::result::Result<T, Error>;
```

---

## Logging Integration

### Error Logging Levels

| Error Type | Log Level | When |
|------------|-----------|------|
| Recoverable | WARN | Error occurred but handled |
| Fatal | ERROR | Error will cause failure |
| External | DEBUG | Wrapped external error (details in source) |

### Tracing Integration

```rust
use tracing::{error, warn, instrument};

#[instrument(skip(config))]
fn load_config(path: &str) -> Result<Config> {
    let data = std::fs::read_to_string(path).map_err(|e| {
        error!(%e, path, "failed to read config file");
        Error::Io(e)
    })?;

    // ...
}
```

---

## Future Considerations

### `#[non_exhaustive]`

Consider adding `#[non_exhaustive]` to allow adding variants in minor versions:

```rust
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    // ...
}
```

**Trade-off:** Prevents exhaustive matching by downstream crates.

### Error Context with `anyhow`

For application code (not library code), consider `anyhow` for ad-hoc context:

```rust
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let config = load_config("app.toml")
        .context("failed to load application config")?;
    // ...
}
```

---

## Summary

| Element | Pattern |
|---------|---------|
| Derive | `#[derive(Error, Debug)]` |
| Messages | `#[error("lowercase message: {0}")]` |
| External errors | `#[from]` for automatic From |
| Crate errors | `#[source]` for error chaining |
| Type alias | `pub type Result<T> = std::result::Result<T, Error>` |
| Naming | PascalCase variants, lowercase messages |

**Reference:** `crates/wish/src/lib.rs:83-122`
