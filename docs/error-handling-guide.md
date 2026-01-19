# Error Handling Guide - charmed_rust

**Date:** 2026-01-19
**Bead:** charmed_rust-ea9
**Status:** Design Document

## Overview

This guide defines the unified error handling pattern for all charmed_rust crates. The pattern is based on the `wish::Error` implementation, which demonstrates idiomatic Rust error handling using `thiserror`.

---

## Core Pattern

All error types in charmed_rust should follow this template:

```rust
use thiserror::Error;
use std::io;

/// Errors that can occur in [crate_name].
#[derive(Error, Debug)]
pub enum Error {
    // 1. External errors with #[from] for automatic conversion
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    // 2. Domain-specific errors with context string
    #[error("operation failed: {0}")]
    Operation(String),

    // 3. Structured errors with named fields
    #[error("{operation} failed: {reason}")]
    OperationContext {
        operation: &'static str,
        reason: String,
    },

    // 4. Errors from other charmed crates with #[source]
    #[error("render error")]
    Render(#[source] lipgloss::Error),

    // 5. Simple sentinel errors (no data)
    #[error("user cancelled operation")]
    Cancelled,
}

/// Result type alias for ergonomic use.
pub type Result<T> = std::result::Result<T, Error>;
```

---

## Reference Implementation: wish::Error

The `wish` crate demonstrates the canonical pattern (see `crates/wish/src/lib.rs:82-122`):

```rust
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

**Key Features:**
- Uses `thiserror::Error` derive macro
- Doc comments on every variant
- `#[from]` for automatic `From` implementations
- Mix of wrapped errors and domain-specific variants
- Result type alias for ergonomic use

---

## Error Design Guidelines

### 1. Variant Naming

| Pattern | Example | Use Case |
|---------|---------|----------|
| **Wrapped External** | `Io(io::Error)` | Standard library or external crate errors |
| **Domain String** | `Configuration(String)` | Domain errors with dynamic message |
| **Sentinel** | `AuthenticationFailed` | Fixed errors with no additional data |
| **Structured** | `ApiError { status: u16, message: String }` | Errors needing multiple context fields |

**Naming Rules:**
- Use PascalCase for variant names
- Name after the error cause, not the context (e.g., `Io` not `FileReadFailed`)
- Avoid generic names like `Other`, `Unknown`, or `Generic`
- Group related errors logically in the enum

### 2. Error Messages

**Formatting Rules:**
- Use lowercase for error messages (Rust convention)
- Be concise but informative
- Include context via interpolation `{0}` or named fields
- End messages without punctuation

```rust
// Good
#[error("io error: {0}")]
#[error("parse failed at line {line}")]
#[error("authentication failed")]

// Bad
#[error("IO Error: {0}.")]     // Uppercase, trailing punctuation
#[error("Something went wrong")]  // Too vague
#[error("")]  // Empty message
```

### 3. Automatic Conversions with #[from]

Use `#[from]` when:
- The error type appears in the crate's public API
- Conversion should be automatic via `?` operator
- There's a 1:1 mapping from source to variant

```rust
#[error("io error: {0}")]
Io(#[from] io::Error),

#[error("russh error: {0}")]
Russh(#[from] russh::Error),
```

### 4. Source Chaining with #[source]

Use `#[source]` when:
- You want to preserve the error chain but add context
- The variant wraps another charmed_rust error type
- You need the error available via `Error::source()`

```rust
#[error("render failed")]
Render(#[source] lipgloss::Error),

#[error("form submission failed")]
Form(#[source] huh::FormError),
```

### 5. Result Type Alias

Always define a crate-local Result type alias:

```rust
/// Result type for [crate_name] operations.
pub type Result<T> = std::result::Result<T, Error>;
```

This enables:
- `fn process() -> Result<()>` instead of `fn process() -> Result<(), Error>`
- Consistent API across all charmed_rust crates
- Clear ownership of the error type

---

## Migration Checklist

When migrating a crate to thiserror:

1. **Add Dependency**
   ```toml
   # In crate's Cargo.toml
   thiserror.workspace = true
   ```

2. **Update Error Type**
   ```rust
   // Before
   #[derive(Debug)]
   pub enum Error {
       Io(io::Error),
   }

   impl std::fmt::Display for Error {
       fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
           match self {
               Self::Io(e) => write!(f, "io error: {e}"),
           }
       }
   }

   impl std::error::Error for Error {}

   impl From<io::Error> for Error {
       fn from(e: io::Error) -> Self {
           Self::Io(e)
       }
   }

   // After
   use thiserror::Error;

   #[derive(Error, Debug)]
   pub enum Error {
       #[error("io error: {0}")]
       Io(#[from] io::Error),
   }
   ```

3. **Add Result Alias**
   ```rust
   pub type Result<T> = std::result::Result<T, Error>;
   ```

4. **Add Doc Comments**
   ```rust
   /// Errors that can occur in [crate_name].
   #[derive(Error, Debug)]
   pub enum Error {
       /// I/O error from file operations.
       #[error("io error: {0}")]
       Io(#[from] io::Error),
   }
   ```

5. **Verify**
   - Run `cargo test` (behavior unchanged)
   - Run `cargo clippy` (no new warnings)
   - Check `Display` output matches previous

---

## Advanced Patterns

### Non-Exhaustive Errors

For library crates where variants may be added:

```rust
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    // Future variants won't break downstream code
}
```

### Clone + PartialEq Requirements

Some error types need `Clone` and `PartialEq` (e.g., for testing). Since `io::Error` doesn't implement these:

```rust
// Option 1: Store string representation
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum FormError {
    #[error("io error: {0}")]
    Io(String),  // String instead of io::Error
}

// Option 2: Use Arc for shared ownership
use std::sync::Arc;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("io error: {0}")]
    Io(Arc<io::Error>),
}
```

### Structured Errors with Context

When errors need multiple pieces of information:

```rust
#[derive(Error, Debug)]
pub enum FetchError {
    #[error("API error ({status}): {message}")]
    ApiError {
        status: u16,
        message: String,
    },

    #[error("rate limited{}", .reset_at.map(|ts| format!(", resets at {ts}")).unwrap_or_default())]
    RateLimited {
        reset_at: Option<u64>,
    },
}
```

---

## Crate-Specific Patterns

### bubbletea

```rust
#[derive(Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    #[error("terminal error: {0}")]
    Terminal(String),

    #[error("program already running")]
    AlreadyRunning,
}
```

### huh

```rust
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum FormError {
    #[error("user aborted")]
    UserAborted,

    #[error("timeout")]
    Timeout,

    #[error("validation error: {0}")]
    Validation(String),

    #[error("io error: {0}")]
    Io(String),  // String for Clone + PartialEq
}
```

### glamour

```rust
#[derive(Error, Debug)]
pub enum RenderError {
    #[error("utf-8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("style error: {0}")]
    Style(String),
}
```

---

## Integration with anyhow

For binary crates (like `glow`), combine thiserror errors with `anyhow`:

```rust
use anyhow::{Context, Result};
use glow::FetchError;

fn main() -> Result<()> {
    let content = fetch_readme("owner/repo")
        .context("failed to fetch README")?;
    Ok(())
}
```

The thiserror `#[source]` attribute ensures proper error chaining with anyhow's context.

---

## Testing Errors

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_format() {
        let err = Error::Io(io::Error::new(io::ErrorKind::NotFound, "file missing"));
        assert!(err.to_string().starts_with("io error:"));
    }

    #[test]
    fn error_from_conversion() {
        let io_err = io::Error::new(io::ErrorKind::Other, "test");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }

    #[test]
    fn error_source_chain() {
        let inner = io::Error::new(io::ErrorKind::Other, "inner");
        let err = Error::Io(inner);
        assert!(std::error::Error::source(&err).is_some());
    }
}
```

---

## Summary

| Requirement | Implementation |
|-------------|----------------|
| Derive macro | `#[derive(Error, Debug)]` |
| Display messages | `#[error("message: {0}")]` |
| Auto From impl | `#[from]` on variant field |
| Error chaining | `#[source]` for wrapped errors |
| Result alias | `pub type Result<T> = std::result::Result<T, Error>` |
| Documentation | Doc comments on enum and variants |

---

## Related Documents

- [Error Type Audit](./error-audit.md) - Inventory of existing error types
- [wish::Error source](../crates/wish/src/lib.rs:82-122) - Reference implementation
