# Lipgloss

CSS-like terminal styling for colors, borders, layout, and text formatting.

## Role in the charmed_rust (FrankenTUI) stack

Lipgloss is the styling foundation for the entire ecosystem. Anything that
renders text in charmed_rust typically does so via lipgloss styles. `bubbletea`
views render strings that are styled with lipgloss, `bubbles` components expose
styling hooks using lipgloss, `glamour` uses lipgloss to theme Markdown output,
`charmed_log` uses it for human-readable log formatting, and the demo showcase
centralizes its theming system around lipgloss.

## Crates.io package

Package name: `charmed-lipgloss`  
Library crate name: `lipgloss`

## What it provides

- `Style` builder for composable styling.
- `Color`, `AdaptiveColor`, and theme presets.
- `Border` presets and layout helpers (`join_horizontal`, `place`, etc.).
- Renderer and color profile helpers for terminal capability handling.

## Typical usage

```rust
use lipgloss::{Border, Position, Style};

let card = Style::new()
    .border(Border::rounded())
    .padding((1, 2))
    .align(Position::Center)
    .foreground("#ff69b4");

println!("{}", card.render("Hello, Lipgloss"));
```

## Where to look next

- `crates/lipgloss/src/style.rs`
- `crates/lipgloss/src/color.rs`
- `crates/lipgloss/src/theme.rs`
