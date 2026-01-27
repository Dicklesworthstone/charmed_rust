# CHARM_SPEC.md - Charmed Rust Behavioral Specification

This document defines the behavioral contract for the Rust port of Charm's TUI
libraries. It is intentionally compact and focuses on cross-crate invariants and
observable behavior.

If any detail here conflicts with the deeper per-crate algorithm specs in
`EXISTING_CHARM_STRUCTURE_AND_ARCHITECTURE.md`, that document wins for the
low-level behavior. This spec defines the user-facing contract and system-wide
rules that implementations must uphold.

## 0. Authority and Precedence

Order of authority for behavioral requirements:

1. `EXISTING_CHARM_STRUCTURE_AND_ARCHITECTURE.md` (per-crate algorithm specs)
2. `CHARM_SPEC.md` (cross-crate behavioral contract)
3. Conformance fixtures and harness in `tests/conformance`
4. Public API docs and examples (README + crate docs)
5. Implementation details (code)

If conflicts exist, follow the highest-ranked source and update the others to
match.

## 1. Scope and Goals

- Provide a complete Rust port of Charm's Go TUI ecosystem with idiomatic Rust
  APIs, while preserving observable behavior and output.
- Keep the Elm Architecture programming model across the stack.
- Maintain safe Rust guarantees: no unsafe code in any crate.
- Support terminal output that matches Go Charm for the same inputs, modulo
  platform and terminal capability differences.

## 2. Non-Goals

- Backward compatibility shims or wrappers for deprecated APIs.
- 1:1 translation of Go code or Go-specific idioms.
- GUI support; terminal-first behavior only.

## 3. Global Invariants

- Safety: Every crate must compile with `#![forbid(unsafe_code)]`.
- Determinism: Given the same inputs (messages, config, environment), output
  must be deterministic except where the spec explicitly allows randomness or
  time dependence.
- Error handling: Prefer `Result` over panics for recoverable errors. Panics are
  allowed only for programming errors that cannot be handled sensibly.
- Feature flags:
  - `bubbletea`: optional `async` support.
  - `glamour`: optional `syntax-highlighting` support.
  - `glow`: optional `github` support.
- Conformance: For each crate, tests in `tests/conformance` define expected
  output equivalence with Go Charm where applicable.

## 4. Dependency and Integration Graph

Crate dependency rules must follow the workspace graph:

- harmonica: standalone
- lipgloss: standalone
- charmed_log -> lipgloss
- bubbletea -> lipgloss, harmonica
- glamour -> lipgloss
- bubbles -> bubbletea, lipgloss
- huh -> bubbletea, lipgloss, bubbles
- wish -> bubbletea
- glow -> glamour, bubbletea, lipgloss, bubbles
- charmed-wasm -> lipgloss (bindings for web/WASM)
- bubbletea-macros (proc-macro helper for bubbletea; compile-time only)

Cross-crate APIs must stay stable and composable. Components are expected to be
used as bubbletea Models whenever interactive behavior is required.

## 5. Behavioral Contracts by Crate

### 5.1 harmonica

Purpose: Physics-based animation helpers.

Behavioral contract:
- `Spring::new(delta_time, angular_frequency, damping_ratio)` precomputes
  coefficients and must behave as a damped harmonic oscillator consistent with
  the Go implementation (see detailed spec).
- `Spring::update(pos, vel, target)` must be stable, smooth, and deterministic.
- `Projectile` updates must reflect constant acceleration kinematics.
- Constants `GRAVITY` and `TERMINAL_GRAVITY` match the defined coordinate
  systems in the architecture spec.

### 5.2 lipgloss

Purpose: CSS-like terminal styling and layout.

Behavioral contract:
- Styles are built via a fluent builder and are immutable after construction.
- `Style::render(text)` applies formatting, borders, padding, margin, alignment,
  and width constraints consistently with the Go output.
- Color profiles must respect terminal capability detection; when terminal
  capabilities are unknown, safe defaults are used.
- Layout helpers (`join_horizontal`, `join_vertical`, `place`) must compute
  widths and heights using Unicode-aware cell widths.
- Themes and themed styles must resolve to concrete colors at render time based
  on a theme context.

Known limitations (see [FEATURE_PARITY.md](FEATURE_PARITY.md)):
- Partial border edges (e.g., top-bottom only) not fully implemented.

### 5.3 bubbletea

Purpose: Elm Architecture TUI runtime.

Behavioral contract:
- `Model` provides `init`, `update`, `view`. `view` must be pure and free of I/O.
- `Message` is type-erased; `downcast` and `downcast_ref` must be safe and
  deterministic.
- `Cmd` represents side effects and produces at most one message. `batch` and
  `sequence` must run commands concurrently or sequentially, respectively.
- The event loop polls input events, updates the model, executes commands, and
  renders the view with an FPS cap. Rendering must not occur if the view is
  identical to the previous render.
- Quit/interrupt messages (`QuitMsg`, `InterruptMsg`) terminate cleanly and
  restore terminal state.
- Optional async runtime (`run_async`) must preserve the same semantics while
  using tokio for async command execution and event handling.

Known limitations (see [FEATURE_PARITY.md](FEATURE_PARITY.md)):
- Custom I/O mode event injection path is not fully implemented.

### 5.3a bubbletea-macros

Purpose: Derive macro to reduce `Model` boilerplate.

Behavioral contract:
- The generated `Model` impl must delegate to the user's inherent `init`,
  `update`, and `view` methods without semantic changes.
- Optional state tracking (`#[state]`) only affects render scheduling and must
  not mutate or reorder user state.

### 5.4 charmed_log

Purpose: Structured logging with styled terminal output.

Behavioral contract:
- Log levels are ordered and filterable: Debug < Info < Warn < Error < Fatal.
- Default formatter is a human-readable, styled output using lipgloss.
- JSON and logfmt formatters must emit stable, parseable output.
- Timestamp and caller fields must be optional and configurable.

### 5.5 glamour

Purpose: Markdown rendering for terminals.

Behavioral contract:
- Must support headings, lists, blockquotes, tables, code blocks, and inline
  emphasis consistent with Go Glamour output.
- Word wrapping must be Unicode-aware and deterministic for a given width.
- Styles are theme-driven; `Style::Auto` selects light/dark based on terminal
  background when possible.
- Optional syntax highlighting must be consistent for supported languages and
  must degrade gracefully when disabled.
- `GLAMOUR_STYLE` environment variable is honored when rendering with
  environment-based defaults.

Known limitations (see [FEATURE_PARITY.md](FEATURE_PARITY.md)):
- Style presets `notty`, `ascii`, `dracula` have minor behavioral differences
  from Go (backtick handling, heading prefixes).

### 5.6 bubbles

Purpose: Reusable TUI components.

Components included:
- **cursor** - Text cursor with blink modes (Blink, Static, Hide)
- **spinner** - Animated loading indicators (12+ preset styles)
- **timer** - Countdown timer with timeout notifications
- **stopwatch** - Elapsed time tracking
- **progress** - Animated progress bars with spring physics
- **viewport** - Scrollable content viewport
- **paginator** - Page navigation (dots or arabic numerals)
- **textinput** - Single-line text input with suggestions
- **textarea** - Multi-line text editor
- **list** - Filterable list with fuzzy matching
- **table** - Data table with keyboard navigation
- **filepicker** - File system browser
- **help** - Key binding help display
- **key** - Keybinding definitions and matching
- **runeutil** - Input sanitization utilities

Behavioral contract:
- Each interactive component must integrate with bubbletea as a model-like unit
  with `update` and `view` semantics.
- Components must expose predictable state transitions and rendering output for
  a given sequence of messages.
- Components must use lipgloss for styling and bubbletea messages for input.

See [EXISTING_CHARM_STRUCTURE_AND_ARCHITECTURE.md §6](EXISTING_CHARM_STRUCTURE_AND_ARCHITECTURE.md#6-bubbles--tui-components)
for detailed data structures and algorithms.

### 5.7 huh

Purpose: Interactive forms and prompts.

Field types:
- **Input** - Single-line text input with validation
- **Select** - Single selection from options
- **MultiSelect** - Multiple selection from options
- **Confirm** - Yes/No confirmation
- **Text** - Multi-line text input (uses bubbles textarea)
- **Note** - Read-only informational display
- **FilePicker** - File selection (uses bubbles filepicker)

Behavioral contract:
- Forms are composed of fields grouped into pages or sections.
- Validation errors are recoverable and must be surfaced to the user without
  aborting the form unless explicitly configured.
- Users can cancel; cancellation is represented as a typed error or message.
- Completed forms expose values by key with type-safe accessors.

Known limitations (see [FEATURE_PARITY.md](FEATURE_PARITY.md)):
- Textarea field not fully implemented (skipped conformance tests).

### 5.8 wish

Purpose: SSH application framework for TUI apps.

Behavioral contract:
- SSH sessions map to independent bubbletea Programs with their own I/O.
- Authentication mechanisms (public key, password, etc.) are configurable and
  deterministic.
- Middleware can wrap session handling and must preserve session lifecycles.
- Server shutdown should be graceful and avoid leaving terminals in bad states.

Known limitations (see [FEATURE_PARITY.md](FEATURE_PARITY.md)):
- Labeled as "beta" stability level.
- Windows SSH support is untested.

### 5.9 glow

Purpose: Markdown reader CLI.

Behavioral contract:
- Accepts file paths, stdin (`-`), or GitHub references (when enabled).
- Renders markdown via glamour with theme selection and optional pager mode.
- Pager behavior must support scrolling, search, and help overlay.
- CLI flags must be stable and documented in README.

### 5.10 charmed-wasm

Purpose: WebAssembly bindings for lipgloss styling.

Behavioral contract:
- Exposes a stable JS API for style construction, layout helpers, and utility
  functions mirroring lipgloss behavior.
- Must render identically to native lipgloss for equivalent inputs, modulo the
  lack of terminal capability detection in WASM.

## 6. Environment and Configuration

- `COLORTERM`, `TERM`, `NO_COLOR`, `COLORFGBG` influence lipgloss color profile
  detection and background selection.
- `GLAMOUR_STYLE` controls glamour default style when using environment-based
  rendering.
- `GITHUB_TOKEN` (glow, optional feature) enables authenticated README fetches
  to avoid rate limits.

## 7. Conformance Testing

- The conformance harness in `tests/conformance` is the source of truth for
  behavioral comparisons with Go Charm.
- For new features, add or update fixtures before implementation changes.
- Conformance failures should be investigated before modifying expected output.
- Current parity status and known gaps are tracked in
  [FEATURE_PARITY.md](FEATURE_PARITY.md).

Fixture coverage by crate (see FEATURE_PARITY.md for details):
- bubbles: 83 fixtures (cursor, spinner, timer, stopwatch, progress, viewport,
  paginator, textinput, textarea, list, table, filepicker, help, keybinding)
- bubbletea: 168 fixtures
- charmed_log: 67 fixtures
- harmonica: 24 fixtures
- huh: 46 fixtures (42 pass, 4 skip - textarea)
- lipgloss: 58 fixtures (57 pass, 1 skip - partial borders)
- glamour: 84 fixtures (81 pass, 3 skip - style presets)
- integration: 24 fixtures

## 8. Compatibility and Versioning

- The project is pre-1.0; APIs may change. Prefer correctness and clarity over
  backward compatibility.
- Breaking changes must update conformance fixtures and documentation.

## 9. Glossary (Quick Reference)

- Model: Pure state container with `init`, `update`, `view`.
- Message: Type-erased input to `update`.
- Cmd: Lazy side-effect producing a Message.
- Theme: Semantic color palette for consistent styling.
- Renderer: Terminal capability detection and output helper.
