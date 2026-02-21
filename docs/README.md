# Charmed Rust Documentation Guide

This guide is the entry point for `charmed_rust/docs/`.

If you are new to the project, start here and follow one of the learning paths below. The goal is to get you from "I can run something" to "I can ship a polished, accessible, production-quality TUI" without guessing which document to read next.

## What this documentation is for

The docs in this directory are designed to help you:

- Understand the architecture and relationships between crates.
- Build terminal apps with the Elm-style `Model`/`Update`/`View` pattern.
- Compose high-level UI components (`bubbles`) instead of re-implementing primitives.
- Build a consistent visual language using semantic theming and style composition.
- Validate quality with accessibility, error-handling, testing, and performance guidance.

## Pick a learning path

### Path A: Fast onboarding (about 45-60 minutes)

Use this if you want to get an app running quickly with good defaults.

1. Read `SPEC.md` (project scope and architecture map).
2. Read `BUBBLETEA.md` (runtime model and event loop patterns).
3. Read `BUBBLES.md` (component-level quick starts and full examples).
4. Read `LIPGLOSS.md` (styling API and rendering model).
5. Read `theming-tutorial.md` (runtime theme switching).

### Path B: Build a high-quality UI (about 2-3 hours)

Use this if your goal is polish, consistency, and maintainability.

1. Follow Path A.
2. Read `theming-best-practices.md`.
3. Read `custom-themes.md`.
4. Read `demo_showcase/VISUAL_DESIGN.md`.
5. Read `demo_showcase/ACCESSIBILITY.md`.
6. Read `error-handling-guide.md` and `ERROR_PATTERN_GUIDE.md`.

### Path C: Architecture and implementation depth

Use this if you are designing internals, extending frameworks, or reviewing behavior parity.

1. `SPEC.md`
2. `async-architecture.md`
3. `async-migration.md`
4. `property-testing.md`
5. `BENCHMARKS.md`
6. `example-audit.md`

## Document map by outcome

| Outcome | Primary docs | Why these matter |
|---|---|---|
| Build first Bubble Tea app | `BUBBLETEA.md`, `BUBBLES.md` | Core runtime loop plus reusable components |
| Design a cohesive visual system | `LIPGLOSS.md`, `demo_showcase/VISUAL_DESIGN.md` | Style primitives, tokens, spacing, hierarchy |
| Add runtime theme switching | `theming-tutorial.md`, `custom-themes.md` | Theme contexts, presets, serialization |
| Ensure accessibility and fallbacks | `theming-best-practices.md`, `demo_showcase/ACCESSIBILITY.md` | Contrast, keyboard-first UX, no-color behavior |
| Harden reliability and errors | `error-handling-guide.md`, `ERROR_PATTERN_GUIDE.md` | Error taxonomy, structured handling patterns |
| Validate performance and correctness | `BENCHMARKS.md`, `property-testing.md` | Repeatable quality checks and invariants |

## High-quality UI checklist

Use this checklist before calling a feature complete.

- The UI uses semantic colors and avoids hardcoded one-off color values in components.
- Focus is always visible and does not rely on color alone.
- Keyboard-only navigation covers all user actions.
- Contrast checks pass for primary text and state indicators.
- The app remains usable in 16-color and no-color terminal environments.
- Styles are created once and reused; rendering avoids avoidable allocations in hot paths.
- Error states are explicit, actionable, and consistently styled.

## Suggested build order for a new app

1. Define domain state and messages in a `Model`.
2. Build interaction flow in `update` first (without styling).
3. Compose layout and styles using `lipgloss`.
4. Introduce `bubbles` components where behavior is standard (list, viewport, spinner, progress, textinput).
5. Add theme context and semantic color slots.
6. Add accessibility and fallback validation.
7. Add snapshot/property tests and basic benchmark checks.

## Notes about API truth

- The canonical API source is the crate code and crate-level rustdoc in `crates/`.
- The docs in this directory are maintained as practical guides and architecture references.
- If a snippet ever diverges from crate behavior, prefer crate code and update docs immediately.

## Where to go next

- For an end-to-end page architecture example: `demo_showcase/PRODUCT_CONCEPT.md`.
- For wizard workflow design detail: `demo_showcase/WIZARD_DESIGN.md`.
- For web/wasm integration docs: `wasm/README.md`.
