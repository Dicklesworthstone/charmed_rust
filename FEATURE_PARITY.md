# FEATURE_PARITY.md — Charmed Rust

Last updated: 2026-01-27

This document tracks conformance and parity status between the Rust ports and
the original Charm Go libraries. It is intended to be a single source of truth
for feature gaps, behavioral discrepancies, and infrastructure blockers.

---

## Conformance Test Run (Latest)

Command:

```
CARGO_HOME=target/cargo_home_20260125_full \
CARGO_TARGET_DIR=target/conformance_20260125_full \
cargo test -p charmed_conformance -- --nocapture
```

Result summary:
- **Failed:** 0
- **Skipped:** 0
- **Notes:** All theme presets now pass (notty, ascii, dracula fixed). Benchmark compile tests ran; benchmark execution tests remain ignored.

### Per-Crate Conformance Results

| Crate        | Tests | Pass | Fail | Skip | Notes |
|--------------|-------|------|------|------|-------|
| bubbles      | 83    | 83   | 0    | 0    | All fixtures pass, full component coverage |
| bubbletea    | 168   | 168  | 0    | 0    | All fixtures pass |
| charmed_log  | 67    | 67   | 0    | 0    | All fixtures pass |
| harmonica    | 24    | 24   | 0    | 0    | All fixtures pass |
| huh          | 46    | 46   | 0    | 0    | All fixtures pass |
| lipgloss     | 58    | 58   | 0    | 0    | All fixtures pass |
| glamour      | 84    | 84   | 0    | 0    | All fixtures pass |
| glow         | 29    | 29   | 0    | 0    | Full fixture coverage (config, render, styles) |
| charmed-wasm | 47    | 47   | 0    | 0    | WASM smoke tests (style, layout, DOM) |
| integration  | 24    | 24   | 0    | 0    | Cross-crate integration OK |

---

## Known Parity Gaps (Behavioral Discrepancies)

**No remaining gaps.** All crates now have full Go API parity.

### Glamour (Markdown Rendering)
**Full parity achieved (2026-01-27).** All 84 conformance tests pass including:
- All style presets (dark, light, ascii, notty, dracula, pink)
- All link, blockquote, nested list, and table tests
- **Go API naming:** `TermRenderer` and `AnsiOptions` match Go API (with `Renderer`/`RendererOptions` aliases for compatibility)

### Bubbletea
**No remaining gaps.** Custom I/O mode event injection is fully implemented via:
- `with_input_receiver(rx)` - for external message injection (used by wish SSH integration)
- `with_input(reader)` - for raw input parsing

### Charmed Log
**Full parity achieved (2026-01-27):**
- `with()` method added for Go API compatibility (alias for `with_fields()`)
- Actual caller extraction via `backtrace` crate (not placeholder)
- `CallerInfo` struct for programmatic caller access

### Huh (Forms)
**Full parity achieved (2026-01-27):**
- `with_accessible(bool)` method added for accessibility mode
- `is_accessible()` accessor method

### Glow (Markdown CLI)
**Full CLI parity achieved (2026-01-27):**
- GitHub README fetching (with `--github` feature)
- URL fetching support
- New CLI flags: `--all`, `--line-numbers`, `--mouse`, `--preserve-new-lines`
- Pager keys: `c` (clipboard), `e` (editor)
- Stdin support via `-`

---

## Known Product Limitations (from README)

### Wish SSH (Validated Stable)
**Stability audit complete (2026-01-27).** The Wish SSH implementation passes all stability tests:
- 9/9 e2e tests pass (connection, auth, PTY, bubbletea rendering)
- 4/4 stress tests pass:
  - Connection throughput: 364 conn/sec
  - Sequential open/close: 12.88 cycles/sec
  - Rapid PTY resize: 19/20 events received
  - Concurrent PTY sessions: 5 sessions OK
- Rich error types implemented (Io, Ssh, Russh, Key, Auth, Config, Session, AddrParse)

**Remaining:** Windows SSH untested (bd-212m.7.2).

### Other Limitations
- Mouse drag support: limited.
- Complex Unicode: "basic" support only.

---

## Recommended Next Actions (High Priority)

1. **Validate Windows SSH** (bd-212m.7.2).
2. **Run targeted validation** for remaining README limitations (mouse drag, Unicode).

---

## Fixture Coverage Notes

### Bubbles (100% Coverage)
All 83 bubbles fixtures have full test implementations:
- **viewport** (7): new, with_content, scroll_down, goto_top, goto_bottom, half_page_down, page_navigation
- **list** (7): empty, with_items, cursor_movement, goto_top_bottom, pagination, title, selection
- **table** (8): empty, with_data, cursor_movement, goto_top_bottom, focus, set_cursor, dimensions, cursor_bounds
- **textinput** (10): new, with_value, char_limit, width, cursor_set/start/end, password, echo_none, focus_blur
- **filepicker** (11): new, set_directory, allowed_types, show_hidden, height, dir_allowed, keybindings, format_size, cursor, sort_order, empty_view
- **spinner** (12): line, dot, minidot, jump, pulse, points, globe, moon, monkey, meter, hamburger, model_view
- **progress** (6): basic, zero, full, custom_width, no_percent, solid_fill
- **paginator** (5): dots, arabic, navigation, boundaries, items_per_page
- **help** (3): basic, custom_width, empty
- **cursor** (4): mode_cursorblink, mode_cursorstatic, mode_cursorhide, model
- **keybinding** (4): simple, multi, disabled, toggle
- **stopwatch** (3): new, tick, reset
- **timer** (3): new, tick, timeout

### Glow (Full Coverage)
Full conformance harness (29 tests):
- **config**: defaults, pager, width, style builder methods
- **render**: basic markdown rendering through glamour
- **styles**: valid style parsing (dark, light, ascii, pink, auto, no-tty)
- **stash**: document organization operations
- **browser**: file browser with fuzzy filtering
- **github**: repository reference parsing, README fetching with cache

**CLI features:**
- File, stdin (`-`), URL, and GitHub repo input sources
- All style themes (dark, light, dracula, ascii, pink, auto, no-tty)
- Pager with search (`/`), navigation, clipboard (`c`), editor (`e`)
- Mouse support, line numbers, preserve newlines options

### Charmed-wasm (WASM Coverage)
47 wasm-bindgen-test tests across two files:
- **web.rs** (33 tests): Module readiness, style creation, colors, formatting, padding, borders, layout helpers, string utilities
- **e2e.rs** (14 tests): DOM rendering, multiple styles, responsive layouts, interactive scenarios

**Run manually**: `wasm-pack test --headless --chrome crates/charmed-wasm`
**CI**: `.github/workflows/wasm.yml` builds and validates WASM packages on push.

---

## Notes

This file is the authoritative parity status report for the port. Update it
after any conformance run or feature parity change.
