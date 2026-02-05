# Huh

Interactive forms and prompts built on bubbletea and bubbles.

## TL;DR

**The Problem:** Building multi-step terminal forms is tedious and repetitive.

**The Solution:** Huh provides a composable form system with validation,
navigation, and consistent styling.

**Why Huh**

- **Multi-step forms**: build wizards and onboarding flows.
- **Reusable fields**: text, select, confirm, and file prompts.
- **Bubbletea-native**: integrates with existing TUIs.

## Role in the charmed_rust (FrankenTUI) stack

Huh is a higher-level layer built on `bubbletea`, `bubbles`, and `lipgloss`.
It provides the form system used in the demo showcase and is intended as the
fastest way to build interactive terminal workflows.

## Crates.io package

Package name: `charmed-huh`  
Library crate name: `huh`

## Installation

```toml
[dependencies]
huh = { package = "charmed-huh", version = "0.1.2" }
```

## Quick Start

```rust
use huh::form::Form;

let mut form = Form::new();
// add steps, fields, and validation
```

## Core Concepts

- **Form**: a sequence of steps with validation.
- **Step**: a single prompt or group of prompts.
- **Field**: text inputs, selects, confirmations, and file pickers.
- **Validation**: per-field validation hooks.

## Integration Pattern

Huh models are standard `bubbletea` models. Embed them in your own model and
forward messages just like any other component.

## Troubleshooting

- **Inputs not updating**: make sure the form receives messages.
- **Validation never passes**: check your validation functions for strictness.

## Limitations

- Focused on terminal UI, not a general-purpose form engine.
- Complex custom widgets require dropping down to `bubbles`.

## FAQ

**Can I mix Huh forms with custom bubbletea views?**  
Yes. Treat the form as a child model.

**Can I theme Huh?**  
Yes, via lipgloss styling hooks.

## About Contributions

Please don't take this the wrong way, but I do not accept outside contributions for any of my projects. I simply don't have the mental bandwidth to review anything, and it's my name on the thing, so I'm responsible for any problems it causes; thus, the risk-reward is highly asymmetric from my perspective. I'd also have to worry about other "stakeholders," which seems unwise for tools I mostly make for myself for free. Feel free to submit issues, and even PRs if you want to illustrate a proposed fix, but know I won't merge them directly. Instead, I'll have Claude or Codex review submissions via `gh` and independently decide whether and how to address them. Bug reports in particular are welcome. Sorry if this offends, but I want to avoid wasted time and hurt feelings. I understand this isn't in sync with the prevailing open-source ethos that seeks community contributions, but it's the only way I can move at this velocity and keep my sanity.

## License

MIT. See `LICENSE` at the repository root.
