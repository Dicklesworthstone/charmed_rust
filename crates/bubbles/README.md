# Bubbles

Pre-built TUI components for bubbletea: inputs, lists, tables, viewports, and
more.

## Role in the charmed_rust (FrankenTUI) stack

Bubbles sits directly above `bubbletea` and provides reusable components that
implement the `Model` trait. These components are styled using `lipgloss` and
are composed together by higher-level crates such as `huh` (forms) and `glow`
(Markdown reader). The demo showcase uses bubbles extensively to demonstrate
the full component catalog.

## Crates.io package

Package name: `charmed-bubbles`  
Library crate name: `bubbles`

## Component catalog

- `textinput` for text entry.
- `list` for selectable lists.
- `table` for tabular data.
- `spinner` and `progress` for loading and progress.
- `viewport` for scrollable content.
- `filepicker` for filesystem navigation.
- `paginator`, `timer`, and `stopwatch` utilities.

## Typical usage

```rust
use bubbles::textinput::TextInput;

let mut input = TextInput::new();
input.set_placeholder("Search...");
input.focus();

// In update():
// input.update(msg);
```

## Where to look next

- `crates/bubbles/src/textinput.rs`
- `crates/bubbles/src/list.rs`
