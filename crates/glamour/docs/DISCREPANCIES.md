# Discrepancies from Go glamour

This document tracks known differences between the Go glamour behavior and the
current Rust implementation.

## Known Differences

### 1. Table Rendering Pipeline

- **Go**: Uses Charm's table rendering via lipgloss components.
- **Rust**: Uses a dedicated `glamour::table` module plus a simpler
  separator-based path inside `Renderer::flush_table`.

Result: The low-level table renderer can produce full borders and header
styles, while the default renderer currently uses separator characters.

### 2. Terminal Capability Detection

- **Go**: Uses `termenv` for terminal detection and color capability.
- **Rust**: Uses lipgloss and crossterm for terminal output and styling.

### 3. Width Calculation

- **Go**: Uses `runewidth`.
- **Rust**: Uses `unicode-width` (compatible behavior for most inputs).

### 4. Logging

- **Go**: Emits some table rendering debug logs in certain builds.
- **Rust**: Table rendering currently emits no logs.

## Compatibility Matrix

| Feature | Go glamour | Rust glamour | Notes |
|---------|------------|--------------|-------|
| Basic tables | ✓ | ✓ | Table parsing works via pulldown-cmark |
| Column alignment | ✓ | ✓ | Alignment uses pulldown-cmark markers |
| Unicode content | ✓ | ✓ | `unicode-width` handles display width |
| Border styles | ✓ | ✓ | Via `glamour::table` renderer |
| Header styling | ✓ | ✓ | Via `HeaderStyle` in `glamour::table` |
| Nested tables | ✗ | ✗ | Not supported by Markdown spec |
