# FEATURE_PARITY.md — Charmed Rust

Last updated: 2026-01-25

This document tracks conformance and parity status between the Rust ports and
the original Charm Go libraries. It is intended to be a single source of truth
for feature gaps, behavioral discrepancies, and infrastructure blockers.

---

## Conformance Test Run (Latest)

Command:

```
CARGO_HOME=target/cargo_home_20260124_210554 \
CARGO_TARGET_DIR=target/conformance_20260124_210554 \
cargo test -p charmed_conformance -- --nocapture
```

Result summary:
- **Failed:** 4 (Glamour blockquote semantic mismatches)
- **Skipped:** 13 (Glamour 8, Huh 4, Lipgloss 1)
- **Notes:** Benchmark compile tests emitted “running over 60 seconds” warnings
  during this run; no cargo lock errors observed. See log
  `target/conformance_20260124_210554.log`.

### Per-Crate Conformance Results

| Crate        | Tests | Pass | Fail | Skip | Notes |
|--------------|-------|------|------|------|-------|
| bubbles      | 83    | 83   | 0    | 0    | All fixtures pass |
| bubbletea    | 168   | 168  | 0    | 0    | All fixtures pass |
| charmed_log  | 67    | 67   | 0    | 0    | All fixtures pass |
| harmonica    | 24    | 24   | 0    | 0    | All fixtures pass |
| huh          | 46    | 42   | 0    | 4    | Textarea not implemented |
| lipgloss     | 58    | 57   | 0    | 1    | Partial border edges |
| glamour      | 84    | 72   | 4    | 8    | Blockquote mismatches + remaining preset/link gaps |
| integration  | 24    | 24   | 0    | 0    | Cross-crate integration OK |

### Benchmark E2E Failures (Infra)

Benchmark compile tests no longer show cargo lock errors after the mutex change,
but did emit “running over 60 seconds” warnings in this run. Re-run after Glamour
fixes to confirm completion timing. See `target/conformance_20260124_210554.log`.

---

## Known Parity Gaps (Behavioral Discrepancies)

### Glamour (Markdown Rendering)
Current gaps vs Go (from latest run):
- **Fails (blockquotes):** `blockquote_single_line`, `blockquote_multi_line`,
  `blockquote_multi_paragraph`, `blockquote_with_formatting`
  - Actual output includes trailing right border `│` and drops styles (bold/italic/fg).
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

1. **Fix Glamour blockquote rendering** (now failing, not skipped).
2. **Address remaining Glamour discrepancies** (task list marker, email autolink, image glyph, presets).
3. **Implement Lipgloss partial border edges**.
4. **Implement Huh textarea field** and extend fixtures.
5. **Audit Bubbletea custom I/O event injection**.
6. **Run targeted validation** for README limitations (Wish stability, mouse drag, Unicode).

---

## Notes

This file is the authoritative parity status report for the port. Update it
after any conformance run or feature parity change.
