# FEATURE_PARITY.md — Charmed Rust

Last updated: 2026-01-25

This document tracks conformance and parity status between the Rust ports and
the original Charm Go libraries. It is intended to be a single source of truth
for feature gaps, behavioral discrepancies, and infrastructure blockers.

---

## Conformance Test Run (Latest)

Command:

```
CARGO_HOME=target/cargo_home_20260124_201340 \
CARGO_TARGET_DIR=target/conformance_20260124_201340 \
cargo test -p charmed_conformance -- --nocapture
```

Result summary:
- **Passed:** 164
- **Failed:** 1
- **Ignored:** 6
- **Failure cause:** `benchmark_e2e::compilation_tests::test_benchmarks_compile` failed due to
  cargo package cache locking and a compile error in `huh` (missing `file_picker`
  field in `KeyMap` initializer at `crates/huh/src/lib.rs:646`).

### Per-Crate Conformance Results

| Crate        | Tests | Pass | Fail | Skip | Notes |
|--------------|-------|------|------|------|-------|
| bubbles      | 83    | 83   | 0    | 0    | All fixtures pass |
| bubbletea    | 168   | 168  | 0    | 0    | All fixtures pass |
| charmed_log  | 67    | 67   | 0    | 0    | All fixtures pass |
| harmonica    | 24    | 24   | 0    | 0    | All fixtures pass |
| huh          | 46    | 42   | 0    | 4    | Textarea not implemented |
| lipgloss     | 58    | 57   | 0    | 1    | Partial border edges |
| glamour      | 84    | 66   | 0    | 18   | Known rendering differences |
| integration  | 24    | 24   | 0    | 0    | Cross-crate integration OK |

### Benchmark E2E Failures (Infra)

`benchmark_e2e::compilation_tests::test_benchmarks_compile` failed after
encountering cargo package cache locks and a compile error in `huh`.
The other compile checks succeeded in this run:
- `benchmark_e2e::compilation_tests::test_bubbletea_benchmarks_compile` ✅
- `benchmark_e2e::compilation_tests::test_glamour_benchmarks_compile` ✅
- `benchmark_e2e::compilation_tests::test_lipgloss_benchmarks_compile` ✅

Root issue remains: concurrent benchmark compilation jobs contend on cargo’s
package cache and artifact directory locks. The new blocker is the `huh` compile
error (`KeyMap` missing `file_picker` initializer). These are infra/product
correctness blockers, not conformance discrepancies.

---

## Known Parity Gaps (Behavioral Discrepancies)

### Glamour (Markdown Rendering)
Skipped tests indicate known output differences from Go:
- `format_mixed`: extra space before inline code
- Nested lists: `list_nested_unordered`, `list_nested_ordered`, `list_mixed_nested`
- Task list: `list_task_list` (task markers rendered differently)
- Links: `link_inline`, `link_inline_title`, `link_reference`
- Email autolink: `link_autolink_email` (mailto prefix)
- Images: `link_image`, `link_image_title` (arrow glyph `->` vs `→`)
- Blockquotes: `blockquote_multi_paragraph`, `blockquote_nested`
- Style presets: `style_preset_dark`, `style_preset_light`,
  `style_preset_notty`, `style_preset_ascii`, `style_preset_dracula`

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

1. **Fix `huh` KeyMap compile error** blocking benchmark compilation.
2. **Fix benchmark compilation locking** (test infra reliability).
3. **Address Glamour rendering discrepancies** (lists, links, blockquotes, presets).
4. **Implement Lipgloss partial border edges**.
5. **Implement Huh textarea field** and extend fixtures.
6. **Audit Bubbletea custom I/O event injection**.
7. **Run targeted validation** for README limitations (Wish stability, mouse drag, Unicode).

---

## Notes

This file is the authoritative parity status report for the port. Update it
after any conformance run or feature parity change.
