# Charmed Log

Structured logging for terminal applications, with optional lipgloss styling.

## TL;DR

**The Problem:** Terminal apps need structured logs that are readable in a TUI
and parsable in CI or production.

**The Solution:** Charmed Log provides a structured logger with multiple
formatters (text, JSON, logfmt) and lipgloss-based styling for humans.

**Why Charmed Log**

- **Structured**: key/value fields for every log entry.
- **Readable**: styled output for terminal usage.
- **Flexible**: switch formatters without changing callsites.

## Role in the charmed_rust (FrankenTUI) stack

Charmed Log is used by `wish` (SSH logging), the demo showcase, and any app that
wants consistent, styled logging. It relies on `lipgloss` for color and layout.

## Crates.io package

Package name: `charmed-log`  
Library crate name: `charmed_log`

## Installation

```toml
[dependencies]
charmed_log = { package = "charmed-log", version = "0.1.2" }
```

## Quick Start

```rust
use charmed_log::{Logger, Level};

let logger = Logger::new();
logger.info("Application started", &[("version", "0.1.2")]);
logger.log(Level::Warn, "Slow response", &[("ms", "1200")]);
```

## Formatters

- **Text**: human-readable output (default).
- **JSON**: machine-readable output for structured pipelines.
- **Logfmt**: `key=value` format for log aggregation.

## API Overview

Key types in `crates/charmed_log/src/lib.rs`:

- `Logger`: main entrypoint.
- `Level`: log level enum (`debug`, `info`, `warn`, `error`, `fatal`).
- `Formatter`: output formatter selection.

## Configuration Patterns

- Choose formatter at initialization.
- Adjust log level to reduce noise in production.
- Use structured fields to capture context (user, request_id, etc.).

## Troubleshooting

- **No output**: ensure logger is initialized and level permits the message.
- **Unreadable colors**: switch formatter to JSON or disable styles.

## Limitations

- Not a full tracing system; it focuses on structured log output.
- Async logging requires external buffering if you need non-blocking sinks.

## FAQ

**Does it integrate with `tracing`?**  
It’s a separate structured logger; you can bridge them if needed.

**Can I output JSON?**  
Yes, select the JSON formatter.

## About Contributions

Please don't take this the wrong way, but I do not accept outside contributions for any of my projects. I simply don't have the mental bandwidth to review anything, and it's my name on the thing, so I'm responsible for any problems it causes; thus, the risk-reward is highly asymmetric from my perspective. I'd also have to worry about other "stakeholders," which seems unwise for tools I mostly make for myself for free. Feel free to submit issues, and even PRs if you want to illustrate a proposed fix, but know I won't merge them directly. Instead, I'll have Claude or Codex review submissions via `gh` and independently decide whether and how to address them. Bug reports in particular are welcome. Sorry if this offends, but I want to avoid wasted time and hurt feelings. I understand this isn't in sync with the prevailing open-source ethos that seeks community contributions, but it's the only way I can move at this velocity and keep my sanity.

## License

MIT. See `LICENSE` at the repository root.
