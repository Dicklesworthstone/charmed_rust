# Charmed Log

Structured logging for terminal applications, with optional lipgloss styling.

## Role in the charmed_rust (FrankenTUI) stack

Charmed Log is the logging spine for the ecosystem. It provides consistent,
structured log output for TUI applications and infrastructure like `wish` (SSH)
and the demo showcase. When rendered in text mode, it uses `lipgloss` styles to
make severity and key/value data easy to scan in terminals.

## Crates.io package

Package name: `charmed-log`  
Library crate name: `charmed_log`

## What it provides

- Log levels and filtering (`trace`, `debug`, `info`, `warn`, `error`, `fatal`).
- Structured key/value fields.
- Multiple formatters (text, JSON, logfmt).
- Optional colored output via lipgloss.

## Typical usage

```rust
use charmed_log::Logger;

let logger = Logger::new();
logger.info("Application started", &[("version", "0.1.0")]);
```

## Where to look next

- `crates/charmed_log/src/lib.rs`
