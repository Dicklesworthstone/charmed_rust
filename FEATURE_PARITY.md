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
- **Skipped:** 7 (Glamour 3, Huh 4)
- **Notes:** Benchmark compile tests ran; benchmark execution tests remain ignored.

### Per-Crate Conformance Results

| Crate        | Tests | Pass | Fail | Skip | Notes |
|--------------|-------|------|------|------|-------|
| bubbles      | 83    | 83   | 0    | 0    | All fixtures pass, full component coverage |
| bubbletea    | 168   | 168  | 0    | 0    | All fixtures pass |
| charmed_log  | 67    | 67   | 0    | 0    | All fixtures pass |
| harmonica    | 24    | 24   | 0    | 0    | All fixtures pass |
| huh          | 46    | 42   | 0    | 4    | Textarea not implemented |
| lipgloss     | 58    | 58   | 0    | 0    | All fixtures pass |
| glamour      | 84    | 81   | 0    | 3    | 3 style presets differ (notty, ascii, dracula) |
| glow         | 7     | 7    | 0    | 0    | Basic conformance harness (config, render, styles, stash) |
| charmed-wasm | 47    | 47   | 0    | 0    | WASM smoke tests (style, layout, DOM) |
| integration  | 24    | 24   | 0    | 0    | Cross-crate integration OK |

---

## Known Parity Gaps (Behavioral Discrepancies)

### Glamour (Markdown Rendering)
Current gaps vs Go (from latest run):
- **Skips:** `style_preset_notty`, `style_preset_ascii`, `style_preset_dracula`
- **Note:** All link, blockquote, nested list, and table tests now pass (81/84 = 96%)

### Huh (Forms)
- Textarea field not implemented (skips: `text_basic`, `text_with_lines`,
  `text_placeholder`, `text_char_limit`).

### Bubbletea
- Custom I/O mode event injection path is noted as “not yet implemented fully”.

---

## Known Product Limitations (from README)

These are documented limitations that still need verification or closure:
- Wish SSH: labeled “beta” and Windows SSH untested.
- Mouse drag support: limited.
- Complex Unicode: “basic” support only.

---

## Recommended Next Actions (High Priority)

1. **Address remaining Glamour preset discrepancies** (notty/ascii backtick handling, dracula heading prefix).
2. **Implement Huh textarea field** and extend fixtures.
3. **Audit Bubbletea custom I/O event injection**.
4. **Run targeted validation** for README limitations (Wish stability, mouse drag, Unicode).

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

### Glow (Initial Coverage)
Basic conformance harness created (7 tests):
- **config**: defaults, pager, width, style builder methods
- **render**: basic markdown rendering through glamour
- **styles**: valid style parsing (dark, light, ascii, pink, auto, no-tty)
- **stash**: document organization operations

**Note**: Glow conformance is new. Fixtures from Go runtime capture pending.
Current tests validate core library behavior without fixture-based comparison.

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
