# Error Type Audit - charmed_rust

**Date:** 2026-01-19
**Bead:** charmed_rust-4l4
**Status:** Complete

## Executive Summary

This audit inventories all error types across charmed_rust crates, categorizes their patterns, and proposes a migration plan to standardize on `thiserror`.

**Key Finding:** Only `wish::Error` uses `thiserror`. All other crates use manual implementations that should be migrated.

---

## Error Type Inventory

### 1. wish (Reference Pattern - DO NOT CHANGE)

**File:** `crates/wish/src/lib.rs:83-119`

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    #[error("ssh error: {0}")]
    Ssh(String),

    #[error("russh error: {0}")]
    Russh(#[from] russh::Error),

    #[error("key error: {0}")]
    Key(String),

    #[error("key loading error: {0}")]
    KeyLoad(#[from] russh_keys::Error),

    #[error("authentication failed")]
    AuthenticationFailed,

    #[error("configuration error: {0}")]
    Configuration(String),

    #[error("session error: {0}")]
    Session(String),

    #[error("address parse error: {0}")]
    AddrParse(#[from] std::net::AddrParseError),
}

pub type Result<T> = std::result::Result<T, Error>;
```

**Pattern Features:**
- Uses `thiserror::Error` derive macro
- `#[error(...)]` for Display implementation
- `#[from]` for automatic `From` implementations
- Type alias `Result<T>` for ergonomic use

---

### 2. bubbletea::Error (NEEDS MIGRATION)

**File:** `crates/bubbletea/src/program.rs:41-60`

**Current Pattern:** Manual `Display`, `Error`, and `From` implementations

```rust
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
}

impl std::fmt::Display for Error { ... }
impl std::error::Error for Error {}
impl From<io::Error> for Error { ... }
```

**Migration Target:**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}
```

---

### 3. bubbletea::MouseParseError (NEEDS MIGRATION)

**File:** `crates/bubbletea/src/mouse.rs:220-243`

**Current Pattern:** Manual `Display` and `Error` implementations

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseParseError {
    UnsupportedSequence,
    InvalidFormat,
    InvalidNumber,
    CoordinateUnderflow,
}
```

**Migration Target:**
```rust
use thiserror::Error;

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseParseError {
    #[error("unsupported mouse sequence")]
    UnsupportedSequence,
    #[error("invalid mouse sequence format")]
    InvalidFormat,
    #[error("invalid numeric value in mouse sequence")]
    InvalidNumber,
    #[error("mouse coordinates underflowed")]
    CoordinateUnderflow,
}
```

---

### 4. huh::FormError (NEEDS MIGRATION)

**File:** `crates/huh/src/lib.rs:74-96`

**Current Pattern:** Manual `Display` and `Error` implementations

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormError {
    UserAborted,
    Timeout,
    Validation(String),
    Io(String),  // Note: stores String, not io::Error
}
```

**Migration Target:**
```rust
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum FormError {
    #[error("user aborted")]
    UserAborted,
    #[error("timeout")]
    Timeout,
    #[error("validation error: {0}")]
    Validation(String),
    #[error("io error: {0}")]
    Io(String),
}
```

**Note:** `Io(String)` pattern is intentional (Clone + PartialEq requirements). Consider if this should be `Io(Arc<io::Error>)` for better error chaining.

---

### 5. glow::ParseError (NEEDS MIGRATION)

**File:** `crates/glow/src/github.rs:109-125`

**Current Pattern:** Manual `Display` and `Error` implementations

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    InvalidFormat,
    MissingOwnerOrRepo,
}
```

**Migration Target:**
```rust
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseError {
    #[error("invalid repository format")]
    InvalidFormat,
    #[error("missing owner or repository name")]
    MissingOwnerOrRepo,
}
```

---

### 6. glow::FetchError (NEEDS MIGRATION)

**File:** `crates/glow/src/github.rs:138-164`

**Current Pattern:** Manual `Display` and `Error` implementations

```rust
#[derive(Debug)]
pub enum FetchError {
    Request(reqwest::Error),
    ApiError { status: u16, message: String },
    DecodeError(String),
    RateLimited { reset_at: Option<u64> },
    CacheError(io::Error),
}
```

**Migration Target:**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FetchError {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("API error ({status}): {message}")]
    ApiError { status: u16, message: String },
    #[error("decode error: {0}")]
    DecodeError(String),
    #[error("rate limited{}", .reset_at.map(|ts| format!(", resets at timestamp {ts}")).unwrap_or_default())]
    RateLimited { reset_at: Option<u64> },
    #[error("cache error: {0}")]
    CacheError(#[from] io::Error),
}
```

---

### 7. charmed_log::ParseLevelError (NEEDS MIGRATION)

**File:** `crates/charmed_log/src/lib.rs:115-123`

**Current Pattern:** Manual `Display` and `Error` implementations

```rust
#[derive(Debug, Clone)]
pub struct ParseLevelError(String);
```

**Migration Target:**
```rust
use thiserror::Error;

#[derive(Error, Debug, Clone)]
#[error("invalid level: {0:?}")]
pub struct ParseLevelError(String);
```

---

### 8. bubbletea_macros::MacroError (SPECIAL CASE)

**File:** `crates/bubbletea-macros/src/error.rs:15-119`

**Current Pattern:** Manual implementations with `to_compile_error()` method

```rust
#[derive(Debug)]
pub enum MacroError {
    Parse(syn::Error),
    MissingAttribute { name: &'static str, span: proc_macro2::Span },
    InvalidAttribute { message: String, span: proc_macro2::Span },
    UnsupportedItem { expected: &'static str, span: proc_macro2::Span },
}
```

**Recommendation:** Keep manual implementation. Proc-macro error types need special handling:
- `to_compile_error()` method is essential
- `syn::Error` integration is custom
- Adding thiserror adds a dependency to the proc-macro crate

---

## Crates Without Error Types

The following crates have no custom error types:

| Crate | Notes |
|-------|-------|
| **lipgloss** | Pure styling, operations don't fail |
| **bubbles** | Uses `bubbletea::Error` for TUI operations |
| **harmonica** | Pure math, operations don't fail |
| **glamour** | Returns `Result<String, std::str::Utf8Error>` for UTF-8 |

---

## Pattern Summary

| Crate | Error Type | Current Pattern | Uses thiserror | Priority |
|-------|------------|-----------------|----------------|----------|
| wish | Error | thiserror | Yes | N/A (Reference) |
| bubbletea | Error | Manual | No | P1 (Core) |
| bubbletea | MouseParseError | Manual | No | P2 |
| huh | FormError | Manual | No | P2 |
| glow | ParseError | Manual | No | P3 |
| glow | FetchError | Manual | No | P3 |
| charmed_log | ParseLevelError | Manual | No | P3 |
| bubbletea-macros | MacroError | Manual | No | N/A (Special) |

---

## Migration Plan

### Phase 1: Core Framework (P1)
1. **bubbletea::Error** - Most used error type across the ecosystem
   - Add thiserror to bubbletea Cargo.toml
   - Migrate Error enum
   - Update re-exports in lib.rs
   - Breaking change: None (API compatible)

### Phase 2: Secondary Crates (P2)
2. **bubbletea::MouseParseError** - Part of bubbletea, do together
3. **huh::FormError** - User-facing forms library
   - Add thiserror to huh Cargo.toml
   - Breaking change: None (API compatible)

### Phase 3: Applications (P3)
4. **glow::ParseError** - Application-specific
5. **glow::FetchError** - Application-specific
6. **charmed_log::ParseLevelError** - Simple struct

### Not Migrating
- **bubbletea_macros::MacroError** - Proc-macro special handling required

---

## Compatibility Notes

### Breaking Changes: NONE

All migrations preserve:
- Enum/struct names and variants
- Public method signatures
- `Display` output format (can verify with tests)
- `From` implementations

### New Capabilities After Migration
- `#[source]` attribute for error chaining
- Consistent `Error::source()` implementation
- Reduced boilerplate (~60% less code per error type)

---

## Dependencies to Add

| Crate | Add to Cargo.toml |
|-------|-------------------|
| bubbletea | `thiserror = "2.0"` |
| huh | `thiserror = "2.0"` |
| glow | `thiserror = "2.0"` |
| charmed_log | `thiserror = "2.0"` |

Note: `thiserror = "2.0"` is already in the workspace `Cargo.toml`.

---

## Testing Strategy

For each migration:
1. Run existing tests (behavior unchanged)
2. Add tests verifying `Display` output matches previous
3. Verify `From` implementations work identically
4. Run `cargo doc` to check documentation

---

## Acceptance Criteria Status

- [x] All errors inventoried (8 error types across 6 crates)
- [x] Patterns categorized (thiserror vs manual vs special)
- [x] Migration plan created (3 phases + exclusions)
- [x] Reviewed (verified 2026-01-19)
