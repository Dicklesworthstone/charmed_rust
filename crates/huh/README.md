# Huh

Interactive forms and prompts built on bubbletea and bubbles.

## Role in the charmed_rust (FrankenTUI) stack

Huh provides a higher-level form system for terminal applications. It composes
`bubbles` widgets (inputs, lists, selectors) into multi-step forms with
validation and navigation. It sits above `bubbletea` and `lipgloss`, and is used
by the demo showcase as the canonical example of multi-step interaction flows.

## Crates.io package

Package name: `charmed-huh`  
Library crate name: `huh`

## What it provides

- Multi-step form flows with per-step validation.
- Text, select, confirm, and file-picker prompts.
- Styling hooks that integrate with lipgloss themes.

## Typical usage

```rust
use huh::form::Form;

let mut form = Form::new();
// Configure steps, validators, and fields...
```

## Where to look next

- `crates/huh/src/form.rs`
