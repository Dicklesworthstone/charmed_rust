# Plan: Port Charm's Go Libraries to Rust

> **THE RULE:** Extract spec from legacy → implement from spec → never translate line-by-line.

---

## Executive Summary

Charmed Rust is a comprehensive port of **all 9 of Charm's terminal UI libraries** from Go to idiomatic Rust. The goal is to create a unified, high-performance TUI ecosystem that enables building beautiful terminal applications with the Elm Architecture pattern.

**Total Legacy Code:** ~17,000 lines of Go across 9 libraries
**Expected Rust Code:** ~8,000-12,000 lines (typical 40-60% reduction)
**Binary Size:** 10-20x smaller than Go equivalents (no runtime, LTO)
**Startup:** 10-20x faster (native, no GC)

---

## Why Port to Rust?

1. **Performance** — No garbage collector, zero-cost abstractions
2. **Binary Size** — Single static binaries, 1-5MB vs 10-50MB Go
3. **Memory Safety** — Ownership system prevents data races
4. **Ecosystem Integration** — Seamless interop with existing Rust CLI tools
5. **Distribution** — Trivial cross-compilation, no runtime dependencies
6. **Async** — First-class async/await vs goroutine overhead

---

## Libraries to Port (Dependency Order)

### Phase 1: Foundations (No Internal Dependencies)

| Crate | Source | Purpose | Est. LoC |
|-------|--------|---------|----------|
| `harmonica` | `legacy_harmonica/` | Spring animation physics | ~100 |
| `lipgloss` | `legacy_lipgloss/` | Terminal styling (colors, borders, layout) | ~1,200 |

### Phase 2: Core Runtime

| Crate | Source | Purpose | Est. LoC |
|-------|--------|---------|----------|
| `bubbletea` | `legacy_bubbletea/` | Elm-architecture TUI framework | ~1,500 |
| `charmed_log` | `legacy_log/` | Structured logging with lipgloss styling | ~600 |

### Phase 3: Rendering

| Crate | Source | Purpose | Est. LoC |
|-------|--------|---------|----------|
| `glamour` | `legacy_glamour/` | Markdown → ANSI rendering | ~1,200 |

### Phase 4: Components

| Crate | Source | Purpose | Est. LoC |
|-------|--------|---------|----------|
| `bubbles` | `legacy_bubbles/` | Reusable TUI components (list, textinput, spinner, etc.) | ~1,800 |

### Phase 5: Applications

| Crate | Source | Purpose | Est. LoC |
|-------|--------|---------|----------|
| `huh` | `legacy_huh/` | Interactive forms and prompts | ~1,500 |
| `wish` | `legacy_wish/` | SSH app framework | ~1,200 |
| `glow` | `legacy_glow/` | Markdown reader CLI | ~900 |

---

## Dependency Graph

```
harmonica (standalone - spring physics)
lipgloss (standalone - terminal styling)

charmed_log ─► lipgloss

bubbletea ─► lipgloss
          ─► harmonica

glamour ─► lipgloss

bubbles ─► bubbletea
        ─► lipgloss
        ─► harmonica

huh ─► bubbletea
    ─► lipgloss
    ─► bubbles

wish ─► bubbletea
     ─► lipgloss
     ─► charmed_log

glow ─► glamour
     ─► bubbletea
     ─► lipgloss
     ─► bubbles
```

---

## What We're Porting

### harmonica
- [x] Spring struct with damping physics
- [x] Projectile motion (3D)
- [x] FPS-to-delta conversion utility
- [x] Over-damped, critically-damped, under-damped springs

### lipgloss
- [ ] Style struct with 30+ properties (bitfield optimization)
- [ ] Color types (ANSI, ANSI256, TrueColor)
- [ ] Border definitions and rendering
- [ ] Padding, margins, alignment
- [ ] Word wrapping with Unicode cell-width awareness
- [ ] Renderer with color profile detection
- [ ] Join/compose utilities

### bubbletea
- [ ] Model trait (Init, Update, View)
- [ ] Cmd type (async commands)
- [ ] Msg type (messages)
- [ ] Program runtime (event loop)
- [ ] Key events (full keyboard support)
- [ ] Mouse events (click, drag, wheel)
- [ ] Window resize events
- [ ] Batch/Sequence command combinators
- [ ] Alt-screen mode
- [ ] Raw terminal mode
- [ ] Panic recovery with terminal restoration

### charmed_log
- [ ] Logger struct with thread-safe access
- [ ] Log levels (Debug, Info, Warn, Error, Fatal)
- [ ] Text formatter (with lipgloss styling)
- [ ] JSON formatter
- [ ] Logfmt formatter
- [ ] Caller tracking
- [ ] Timestamp handling

### glamour
- [ ] TermRenderer for Markdown → ANSI
- [ ] Style configuration per element type
- [ ] Theme system (dark, light, auto-detect)
- [ ] Code syntax highlighting (via syntect)
- [ ] Word wrapping
- [ ] Link footnotes
- [ ] GFM extensions (tables, strikethrough, task lists)

### bubbles
- [ ] List component (filtering, pagination, delegation)
- [ ] TextInput component (validation, cursor, placeholder)
- [ ] TextArea component (multi-line)
- [ ] Spinner component (multiple styles)
- [ ] Progress component (bar, percentage)
- [ ] Paginator component
- [ ] Table component (sortable)
- [ ] Viewport component (scrollable)
- [ ] Help component (keybinding display)
- [ ] FilePicker component
- [ ] Cursor utilities

### huh
- [ ] Form orchestration
- [ ] Group/page management
- [ ] Input field (single-line with validation)
- [ ] Select field (single-select dropdown)
- [ ] MultiSelect field (checkboxes)
- [ ] Confirm field (yes/no)
- [ ] Text field (multi-line)
- [ ] Note field (display-only)
- [ ] FilePicker field
- [ ] Theme system
- [ ] Accessibility support

### wish
- [ ] SSH server setup
- [ ] Bubble Tea middleware (SSH → TUI bridging)
- [ ] Window resize handling
- [ ] Color profile negotiation
- [ ] Session management
- [ ] Access control middleware
- [ ] Logging middleware
- [ ] Rate limiting middleware
- [ ] Panic recovery middleware

### glow
- [ ] CLI with clap (file, URL, stdin support)
- [ ] TUI browser mode
- [ ] Pager mode
- [ ] Style selection (auto, dark, light, custom)
- [ ] Config file support
- [ ] GitHub/GitLab URL resolution
- [ ] Line number display

---

## What We're NOT Porting

| Feature | Reason |
|---------|--------|
| Go-specific interfaces | Using Rust traits instead |
| Goroutine patterns | Using tokio async/await |
| Channel-based coordination | Using tokio channels where needed |
| Go module system | Using Cargo workspace |
| Go error handling | Using Result<T, E> with thiserror |
| Interface{} dynamic typing | Using enums and generics |
| Reflection-based serialization | Using serde derive |

---

## Reference Projects

These projects demonstrate the Rust patterns and best practices we'll follow:

| Project | Path | Patterns to Copy |
|---------|------|------------------|
| rich_rust | `/data/projects/rich_rust` | Terminal styling, color handling, text rendering |
| beads_rust | `/data/projects/beads_rust` | CLI structure, error handling, SQLite patterns |
| xf | `/data/projects/xf` | Clap CLI, release profile, build.rs |
| cass | `/data/projects/cass` | SQLite, config management, logging |

---

## Key Go → Rust Transformations

| Go Pattern | Rust Equivalent |
|------------|-----------------|
| `interface{}` | `enum` or `Box<dyn Trait>` |
| `error` return | `Result<T, Error>` |
| `struct { ... }` | `struct` with `impl` blocks |
| `interface` | `trait` |
| `goroutine` | `tokio::spawn` / async |
| `channel` | `tokio::sync::mpsc` |
| `defer` | `Drop` trait or scopeguard |
| `nil` | `Option::None` |
| `panic/recover` | `catch_unwind` or Result |
| Property bitfields (int64) | `bitflags` crate |
| JSON struct tags | `#[serde(...)]` attributes |
| Mutex with defer unlock | `parking_lot` or `std::sync::Mutex` |

---

## Workspace Structure

```
charmed_rust/
├── AGENTS.md                              # Agent guidelines
├── PLAN_TO_PORT_CHARM_TO_RUST.md         # This document
├── EXISTING_CHARM_STRUCTURE_AND_ARCHITECTURE.md  # THE SPEC (TBD)
├── PROPOSED_RUST_CHARMED_ARCHITECTURE.md # Rust design (TBD)
├── Cargo.toml                            # Workspace root
├── rust-toolchain.toml                   # Nightly toolchain
├── crates/
│   ├── harmonica/                        # Spring animations
│   ├── lipgloss/                         # Terminal styling
│   ├── bubbletea/                        # TUI framework
│   ├── charmed_log/                      # Logging
│   ├── glamour/                          # Markdown rendering
│   ├── bubbles/                          # TUI components
│   ├── huh/                              # Forms
│   ├── wish/                             # SSH apps
│   └── glow/                             # CLI binary
├── legacy_harmonica/                     # Go reference (gitignored)
├── legacy_lipgloss/                      # Go reference (gitignored)
├── legacy_bubbletea/                     # Go reference (gitignored)
├── legacy_log/                           # Go reference (gitignored)
├── legacy_glamour/                       # Go reference (gitignored)
├── legacy_bubbles/                       # Go reference (gitignored)
├── legacy_huh/                           # Go reference (gitignored)
├── legacy_wish/                          # Go reference (gitignored)
├── legacy_glow/                          # Go reference (gitignored)
├── examples/                             # Example applications
├── benches/                              # Benchmarks
├── tests/                                # Integration tests
└── docs/                                 # Documentation
```

---

## Implementation Phases

### Phase 1: Foundations (harmonica + lipgloss)

**Goal:** Working spring animations and terminal styling

1. Complete harmonica implementation
   - Spring physics with damping
   - Projectile motion
   - Unit tests against Go behavior

2. Implement lipgloss core
   - Color types (ANSI, ANSI256, TrueColor)
   - Style struct with property bitfield
   - Builder pattern for style composition
   - Basic rendering

3. Add lipgloss layout
   - Padding, margins
   - Borders (all built-in styles)
   - Alignment (horizontal, vertical)
   - Word wrapping

4. Add lipgloss utilities
   - Join (horizontal, vertical)
   - Width/height calculation
   - Renderer with color profile detection

**Deliverable:** `cargo test -p harmonica -p lipgloss` passes

### Phase 2: Core Runtime (bubbletea + charmed_log)

**Goal:** Working event loop and logging

1. Define core traits
   - Model trait (Init, Update, View)
   - Msg enum
   - Cmd type

2. Implement Program runtime
   - Event loop
   - Message dispatch
   - Command execution
   - Terminal raw mode

3. Add input handling
   - Key events
   - Mouse events
   - Window resize

4. Add program options
   - Alt-screen mode
   - Mouse tracking
   - Panic recovery

5. Implement charmed_log
   - Logger struct
   - Formatters (text, JSON, logfmt)
   - Level filtering
   - Caller tracking

**Deliverable:** Basic TUI apps work with `cargo run --example basic`

### Phase 3: Rendering (glamour)

**Goal:** Markdown → ANSI rendering

1. Integrate pulldown-cmark parser
2. Implement element renderers
   - Headings, paragraphs
   - Code blocks (with syntect)
   - Lists, tables
   - Links, images

3. Add theme system
   - Style configuration
   - Dark/light detection
   - Custom themes

4. Add utilities
   - Word wrapping
   - Link footnotes

**Deliverable:** `glamour::render("# Hello")` produces styled ANSI

### Phase 4: Components (bubbles)

**Goal:** Reusable TUI widgets

1. Implement core components
   - TextInput
   - Spinner
   - Progress

2. Implement complex components
   - List (with delegate pattern)
   - Table
   - Viewport

3. Add utility components
   - Help
   - Paginator
   - Cursor

**Deliverable:** All major bubbles components working

### Phase 5: Applications (huh, wish, glow)

**Goal:** High-level applications

1. Implement huh
   - Form orchestration
   - Field types
   - Validation
   - Themes

2. Implement wish
   - SSH server (russh)
   - Bubble Tea middleware
   - Session management

3. Implement glow
   - CLI interface
   - TUI browser
   - Config management

**Deliverable:** Full-featured applications

---

## Testing Strategy

### Unit Tests
- Each crate has `tests/` module
- Test public API behavior
- Property-based tests for algorithms (proptest)

### Integration Tests
- Cross-crate interactions in `tests/`
- Example applications that exercise full stack

### Conformance Tests
- Compare output with Go implementations
- Snapshot testing with insta

### Benchmarks
- Rendering performance
- Event loop throughput
- Memory usage

---

## Quality Gates

Before each phase is complete:

```bash
# Must pass
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
cargo test --workspace

# Should run
ubs $(git diff --name-only --cached)  # Before commits
```

---

## Session Protocol

### Starting a Session
1. Read AGENTS.md completely
2. Read this plan document
3. Check `bd ready` for prioritized work
4. Reserve files with Agent Mail

### Ending a Session
1. Run quality gates
2. Update beads status
3. Commit with descriptive message
4. Push to remote
5. Document blockers/questions

---

## Current Status

- [x] Project structure created
- [x] Legacy repos cloned
- [x] Workspace Cargo.toml configured
- [x] Crate scaffolding created
- [x] AGENTS.md written
- [x] Plan document created (this file)
- [ ] EXISTING_CHARM_STRUCTURE_AND_ARCHITECTURE.md (pending deep dives)
- [ ] PROPOSED_RUST_CHARMED_ARCHITECTURE.md (pending)
- [ ] Phase 1 implementation
- [ ] Phase 2 implementation
- [ ] Phase 3 implementation
- [ ] Phase 4 implementation
- [ ] Phase 5 implementation

---

## Next Steps

1. **Deep dive each library** — Create comprehensive spec in EXISTING_CHARM_STRUCTURE_AND_ARCHITECTURE.md
2. **Design Rust architecture** — Synthesize from rich_rust/beads_rust patterns
3. **Implement harmonica** — Simplest crate, establishes patterns
4. **Implement lipgloss** — Core styling, enables all other crates
5. **Continue up the dependency graph**

---

## Open Questions

1. **Async runtime choice** — tokio vs async-std vs smol?
   - **Decision:** tokio (ecosystem dominance, performance)

2. **Terminal library** — crossterm vs termion?
   - **Decision:** crossterm (cross-platform, active development)

3. **SSH library** — russh vs thrussh?
   - **Decision:** russh (better async support, maintained fork)

4. **Markdown parser** — pulldown-cmark vs comrak?
   - **Decision:** pulldown-cmark (pure Rust, well-maintained)

5. **Syntax highlighting** — syntect vs tree-sitter-highlight?
   - **Decision:** syntect (optional feature, simpler integration)

---

## Contributors

- Jeffrey Emanuel — Project owner
- AI Agents — Implementation via Claude Code

---

*Last updated: 2026-01-17*
