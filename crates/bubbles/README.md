# Bubbles

Pre-built TUI components for bubbletea: inputs, lists, tables, viewports, and more.

## TL;DR

**The Problem:** Building a robust TUI requires a lot of reusable widgets.

**The Solution:** Bubbles provides a curated component suite that already
implements `Model` and integrates with `bubbletea` and `lipgloss`.

**Why Bubbles**

- **Composable**: components are models you can embed and update.
- **Consistent**: shared styling and event handling.
- **Productive**: avoids rewriting common UI widgets.

## Role in the charmed_rust (FrankenTUI) stack

Bubbles sits directly above `bubbletea` and provides reusable components. It is
used by `huh` (forms), `glow` (markdown reader), and the demo showcase.

## Crates.io package

Package name: `charmed-bubbles`  
Library crate name: `bubbles`

## Installation

```toml
[dependencies]
bubbles = { package = "charmed-bubbles", version = "0.1.2" }
```

## Component Catalog

- `textinput`: single-line text entry.
- `textarea`: multi-line text editing.
- `list`: selectable lists with optional filtering.
- `table`: fixed-width tabular rendering.
- `spinner`: loading indicators.
- `progress`: progress bars.
- `viewport`: scrollable content area.
- `filepicker`: filesystem navigation.
- `paginator`: pagination control.
- `timer`: countdown timers.
- `stopwatch`: elapsed time counters.
- `help`: keybinding hints.
- `cursor`: shared cursor utilities.

## Quick Start (Text Input)

```rust
use bubbles::textinput::TextInput;

let mut input = TextInput::new();
input.set_placeholder("Search...");
input.focus();

// in update:
// input.update(msg);

// in view:
// input.view()
```

## Integration Pattern

Each component implements a bubbletea-compatible update/view pattern. You
usually wire them into a parent model and forward messages:

```rust
fn update(&mut self, msg: Message) -> Cmd {
    self.input.update(msg.clone());
    self.list.update(msg);
    Cmd::none()
}
```

## Styling

Components expose styling hooks via `lipgloss`. Use `Style` to customize colors,
padding, borders, and layout.

## API Overview

- `crates/bubbles/src/textinput.rs`
- `crates/bubbles/src/textarea.rs`
- `crates/bubbles/src/list.rs`
- `crates/bubbles/src/table.rs`
- `crates/bubbles/src/viewport.rs`
- `crates/bubbles/src/spinner.rs`
- `crates/bubbles/src/progress.rs`
- `crates/bubbles/src/filepicker.rs`

## Troubleshooting

- **Component not receiving input**: ensure you forward `Message` to it.
- **Weird layout**: check component width/height settings and terminal size.
- **Styling not applied**: verify you’re using `lipgloss::Style` helpers.

## Limitations

- Components assume terminal I/O and bubbletea’s message loop.
- Some components require explicit sizing to render correctly.

## FAQ

**Can I use a component outside bubbletea?**  
They’re designed for bubbletea’s update/view model; you can adapt if needed.

**Are all components themeable?**  
Yes, through lipgloss styles and configuration hooks.

## About Contributions

Please don't take this the wrong way, but I do not accept outside contributions for any of my projects. I simply don't have the mental bandwidth to review anything, and it's my name on the thing, so I'm responsible for any problems it causes; thus, the risk-reward is highly asymmetric from my perspective. I'd also have to worry about other "stakeholders," which seems unwise for tools I mostly make for myself for free. Feel free to submit issues, and even PRs if you want to illustrate a proposed fix, but know I won't merge them directly. Instead, I'll have Claude or Codex review submissions via `gh` and independently decide whether and how to address them. Bug reports in particular are welcome. Sorry if this offends, but I want to avoid wasted time and hurt feelings. I understand this isn't in sync with the prevailing open-source ethos that seeks community contributions, but it's the only way I can move at this velocity and keep my sanity.

## License

MIT. See `LICENSE` at the repository root.
