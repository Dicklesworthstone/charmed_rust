# FEATURE_PARITY.md — Charmed Rust

Last updated: 2026-01-25

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
- **Skipped:** 13 (Glamour 8, Huh 4, Lipgloss 1)
- **Notes:** Benchmark compile tests ran; benchmark execution tests remain ignored.

### Per-Crate Conformance Results

| Crate        | Tests | Pass | Fail | Skip | Notes |
|--------------|-------|------|------|------|-------|
| bubbles      | 83    | 83   | 0    | 0    | All fixtures pass |
| bubbletea    | 168   | 168  | 0    | 0    | All fixtures pass |
| charmed_log  | 67    | 67   | 0    | 0    | All fixtures pass |
| harmonica    | 24    | 24   | 0    | 0    | All fixtures pass |
| huh          | 46    | 42   | 0    | 4    | Textarea not implemented |
| lipgloss     | 58    | 57   | 0    | 1    | Partial border edges |
| glamour      | 84    | 76   | 0    | 8    | Remaining preset/link/task list/nested quote gaps |
| integration  | 24    | 24   | 0    | 0    | Cross-crate integration OK |

---

## Known Parity Gaps (Behavioral Discrepancies)

### Glamour (Markdown Rendering)
Current gaps vs Go (from latest run):
- **Skips:** `list_task_list`, `link_autolink_email`, `link_image`,
  `link_image_title`, `blockquote_nested`, `style_preset_notty`,
  `style_preset_ascii`, `style_preset_dracula`

### Lipgloss (Terminal Styling)
- `border_partial_top_bottom`: partial border edges not implemented.

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

1. **Address remaining Glamour discrepancies** (task list marker, email autolink, image glyph, presets, nested blockquotes).
2. **Implement Lipgloss partial border edges**.
3. **Implement Huh textarea field** and extend fixtures.
4. **Audit Bubbletea custom I/O event injection**.
5. **Run targeted validation** for README limitations (Wish stability, mouse drag, Unicode).

---

## Notes

This file is the authoritative parity status report for the port. Update it
after any conformance run or feature parity change.
