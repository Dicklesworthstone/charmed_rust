# Dependency Upgrade Log

**Date:** 2026-02-19  |  **Project:** charmed_rust  |  **Language:** Rust

## Summary
- **Updated:** 11  |  **Skipped:** 0  |  **Failed:** 0  |  **Needs attention:** 0

## Updates

### Phase 1: Semver-Compatible Patch Bumps (via `cargo update`)

#### bitflags: 2.10.0 → 2.11.0
- **Breaking:** None (patch)
- **Tests:** Passed

#### syn: 2.0.114 → 2.0.116
- **Breaking:** None (patch)
- **Tests:** Passed

#### trybuild: 1.0.115 → 1.0.116
- **Breaking:** None (patch)
- **Tests:** Passed

#### tempfile: 3.24.0 → 3.25.0
- **Breaking:** None (patch)
- **Tests:** Passed

#### predicates: 3.1.3 → 3.1.4
- **Breaking:** None (patch)
- **Tests:** Passed

### Phase 2: Version Spec Changes

#### unicode-width: 0.1 → 0.2 (lipgloss, glamour)
- **Breaking:** Different width values for some Unicode characters (UAX#11 updates)
- **Migration:** Changed local `unicode-width = "0.1"` to `unicode-width.workspace = true` in lipgloss and glamour Cargo.toml
- **Tests:** 805 tests passed across both crates

#### darling: 0.20 → 0.23 (bubbletea-macros)
- **Breaking:** MSRV bump to 1.88, `fnv` → `std::collections::HashSet` in `darling::usage` types (not used by this project)
- **Migration:** Version bump only, no code changes needed
- **Tests:** 62 tests passed (50 unit + 1 trybuild + 11 integration)

#### portable-pty: 0.8 → 0.9 (demo_showcase dev-deps)
- **Breaking:** Minor API changes
- **Migration:** Version bump only, no code changes needed
- **Tests:** 995 tests passed

#### vt100: 0.15 → 0.16 (demo_showcase dev-deps)
- **Breaking:** Minor API changes
- **Migration:** Version bump only, no code changes needed
- **Tests:** 995 tests passed (tested with portable-pty)

### Phase 3: Major Version Bumps

#### criterion: 0.5 → 0.8 (workspace, conformance)
- **Breaking:** `criterion::black_box` deprecated (use `std::hint::black_box`), `async-std` runtime removed, MSRV bump to 1.86
- **Migration:** Replaced `criterion::black_box` with `std::hint::black_box` in 4 benchmark files (lipgloss, glamour, bubbletea, bubbles)
- **Tests:** 231 conformance tests passed, full workspace clean

#### colored: 2.x → 3.1.1 (workspace, lipgloss)
- **Breaking:** MSRV bump to 1.80 only. Zero API changes.
- **Migration:** Version bump only, no code changes needed
- **Tests:** 487 lipgloss tests passed

## Additional Changes

### Snapshots Updated
- `snapshot_tests__settings_80x24.snap` and `snapshot_tests__settings_120x40.snap` were updated via `cargo insta accept` to reflect the v0.1.2 → v0.2.0 version number change (pre-existing mismatch, not caused by dependency updates)

### Also Updated via `cargo update` (Transitive)
- clap: 4.5.57 → 4.5.60
- futures: 0.3.31 → 0.3.32
- libc: 0.2.180 → 0.2.182
- itertools: 0.10.5 → 0.13.0 (pulled in by criterion 0.8)
- Various other transitive dependency patches

## Final Validation
- `cargo check --workspace --all-targets`: Clean
- `cargo clippy --workspace --all-targets -- -D warnings`: Zero warnings
- `cargo test --workspace`: All tests pass (3,700+ tests)
