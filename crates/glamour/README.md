# Glamour

Markdown rendering for terminal applications, ported from the Charm ecosystem.

Glamour transforms Markdown into styled terminal output with theme support,
word-wrapping, and optional syntax highlighting.

## TL;DR

**The Problem:** Raw Markdown is hard to read in a terminal, especially with
tables and code blocks.

**The Solution:** Glamour renders Markdown into a styled, readable TUI output
using lipgloss-based themes.

**Why Glamour**

- **Beautiful output**: headings, lists, tables, and code blocks are styled.
- **Themeable**: built-in presets plus custom styles.
- **Composable**: use as a library or in CLI apps like `glow`.

## Role in the charmed_rust (FrankenTUI) stack

Glamour is the Markdown renderer used by `glow` and the demo showcase. It builds
on `lipgloss` for theme-aware styling.

## Crates.io package

Package name: `charmed-glamour`  
Library crate name: `glamour`

## Installation

```toml
[dependencies]
glamour = { package = "charmed-glamour", version = "0.1.2" }
```

Enable syntax highlighting:

```toml
glamour = { package = "charmed-glamour", version = "0.1.2", features = ["syntax-highlighting"] }
```

## Quick Start

```rust
use glamour::{render, Style};

let markdown = "# Hello\n\nThis is **bold**.";
let output = render(markdown, Style::Dark).unwrap();
println!("{}", output);
```

## Rendering Modes

- **Quick render**: `render(markdown, style)`.
- **Configurable**: `Renderer::new().with_style(...).with_word_wrap(...)`.

## Themes

Glamour ships with multiple themes (dark, light, ascii, etc.) and allows custom
styles for full control. See `crates/glamour/src/style.rs` and
`crates/glamour/docs/README.md` for advanced theming and table styling.

## Tables

Markdown tables are supported. For advanced table APIs, see:

- `crates/glamour/docs/tables/README.md`
- `crates/glamour/src/table.rs`

## Feature Flags

- `syntax-highlighting`: enables syntect-based code highlighting.

## Troubleshooting

- **No code highlighting**: enable the `syntax-highlighting` feature.
- **Lines wrap oddly**: set an explicit wrap width on the `Renderer`.

## Limitations

- Syntax highlighting increases binary size.
- Rendering is terminal-only; no HTML output.

## FAQ

**Can I use Glamour without bubbletea?**  
Yes, it’s a standalone Markdown renderer.

**Does it support custom themes?**  
Yes, via style configuration and theme presets.

## About Contributions

Please don't take this the wrong way, but I do not accept outside contributions for any of my projects. I simply don't have the mental bandwidth to review anything, and it's my name on the thing, so I'm responsible for any problems it causes; thus, the risk-reward is highly asymmetric from my perspective. I'd also have to worry about other "stakeholders," which seems unwise for tools I mostly make for myself for free. Feel free to submit issues, and even PRs if you want to illustrate a proposed fix, but know I won't merge them directly. Instead, I'll have Claude or Codex review submissions via `gh` and independently decide whether and how to address them. Bug reports in particular are welcome. Sorry if this offends, but I want to avoid wasted time and hurt feelings. I understand this isn't in sync with the prevailing open-source ethos that seeks community contributions, but it's the only way I can move at this velocity and keep my sanity.

## License

MIT. See `LICENSE` at the repository root.
