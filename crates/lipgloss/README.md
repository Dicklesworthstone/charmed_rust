# Lipgloss

CSS-like terminal styling for colors, borders, layout, and text formatting.

Lipgloss is the styling foundation of charmed_rust. It provides a fluent `Style`
API, layout helpers, and theme-aware color systems that make terminal UIs feel
intentional instead of improvised.

## TL;DR

**The Problem:** Styling terminals by hand (ANSI codes, manual padding, ad-hoc
wrapping) is brittle and unreadable.

**The Solution:** Lipgloss gives you a declarative, composable styling system
that feels like CSS: borders, margins, alignment, and theme-aware colors.

**Why Lipgloss**

- **Compositional**: styles are pure values you can clone and tweak.
- **Theme-aware**: semantic color slots for consistent theming.
- **Layout helpers**: alignment, joining, and placement built in.
- **Portable**: native terminal support plus optional WASM bindings.

## Role in the charmed_rust (FrankenTUI) stack

Lipgloss is the visual layer for the entire ecosystem. `bubbletea` renders its
views using lipgloss, `bubbles` components expose lipgloss styling hooks,
`glamour` uses lipgloss for Markdown themes, and `charmed_log` uses it for
human-readable logging output. The demo showcase centralizes all theming through
lipgloss.

## Crates.io package

Package name: `charmed-lipgloss`  
Library crate name: `lipgloss`

## Installation

```toml
[dependencies]
lipgloss = { package = "charmed-lipgloss", version = "0.1.2" }
```

## Quick Example

```rust
use lipgloss::{Border, Position, Style};

let card = Style::new()
    .border(Border::rounded())
    .padding((1, 2))
    .align(Position::Center)
    .foreground("#ff69b4");

println!("{}", card.render("Hello, Lipgloss"));
```

## Core Concepts

- **Style**: immutable style value with a fluent builder.
- **Color / AdaptiveColor**: supports hex, ANSI 256, and adaptive colors.
- **Border**: preset border styles (rounded, double, ascii, etc.).
- **Position**: alignment helpers for layout and placement.
- **Theme**: semantic color slots for consistent styling across an app.

## API Overview

- `Style` in `crates/lipgloss/src/style.rs`
- Colors in `crates/lipgloss/src/color.rs`
- Borders in `crates/lipgloss/src/border.rs`
- Layout helpers in `crates/lipgloss/src/position.rs`
- Rendering and terminal detection in `crates/lipgloss/src/renderer.rs`
- Theming in `crates/lipgloss/src/theme.rs`

## Theming

Lipgloss supports semantic theming with preset palettes and custom themes. Use
semantic slots (primary, error, muted, etc.) instead of hard-coded colors to
make theme switching easy.

```rust
use lipgloss::{ThemeContext, ThemePreset, ThemedStyle, ColorSlot};
use std::sync::Arc;

let ctx = Arc::new(ThemeContext::from_preset(ThemePreset::Dracula));
let title = ThemedStyle::new(ctx.clone()).foreground(ColorSlot::Primary).bold();
```

## Layout Helpers

- `join_horizontal` and `join_vertical` to compose blocks.
- `place` to center or align content inside a container.

```rust
use lipgloss::{join_horizontal, Position};

let left = "Left";
let right = "Right";
let row = join_horizontal(Position::Top, &[left, right]);
```

## Feature Flags

- `native` (default): crossterm + colored for terminal output.
- `yaml`: YAML serialization for themes.
- `tokio`: async helpers for theme updates.
- `wasm`: WebAssembly bindings.

```toml
lipgloss = { package = "charmed-lipgloss", version = "0.1.2", features = ["yaml"] }
```

## Troubleshooting

- **Colors look wrong**: verify terminal supports truecolor or switch to ANSI.
- **Layout width is off**: ensure your content is UTF-8 and use `string_width`.
- **Theme doesn’t apply**: use `ThemedStyle` with a shared `ThemeContext`.

## Limitations

- Not a full layout engine (no flex/grid).
- Terminal rendering depends on the host terminal’s capabilities.

## FAQ

**Can I use lipgloss without bubbletea?**  
Yes, lipgloss is completely standalone.

**Does it support no-tty output?**  
Yes, you can render plain text by disabling styling.

**Is it WASM-ready?**  
Yes, enable the `wasm` feature and use the `lipgloss::wasm` bindings.

## About Contributions

Please don't take this the wrong way, but I do not accept outside contributions for any of my projects. I simply don't have the mental bandwidth to review anything, and it's my name on the thing, so I'm responsible for any problems it causes; thus, the risk-reward is highly asymmetric from my perspective. I'd also have to worry about other "stakeholders," which seems unwise for tools I mostly make for myself for free. Feel free to submit issues, and even PRs if you want to illustrate a proposed fix, but know I won't merge them directly. Instead, I'll have Claude or Codex review submissions via `gh` and independently decide whether and how to address them. Bug reports in particular are welcome. Sorry if this offends, but I want to avoid wasted time and hurt feelings. I understand this isn't in sync with the prevailing open-source ethos that seeks community contributions, but it's the only way I can move at this velocity and keep my sanity.

## License

MIT. See `LICENSE` at the repository root.
