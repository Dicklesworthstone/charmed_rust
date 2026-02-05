# Glow

A terminal-based Markdown reader and browser, powered by `glamour` and `bubbletea`.

## TL;DR

**The Problem:** Reading Markdown in the terminal is painful without styling and
navigation.

**The Solution:** Glow renders Markdown with themes, wrapping, and a pager UI.

**Why Glow**

- **Beautiful rendering** via `glamour`.
- **Interactive**: scroll, search, and browse files.
- **Scriptable**: read from stdin or local files.

## Role in the charmed_rust (FrankenTUI) stack

Glow is the flagship CLI app that demonstrates how `bubbletea`, `bubbles`, and
`glamour` work together. It’s also a real end-user tool you can install.

## Crates.io package

Package name: `charmed-glow`  
Library crate name: `glow`  
Binary name: `glow`

## Installation

```toml
[dependencies]
glow = { package = "charmed-glow", version = "0.1.2" }
```

Install the CLI:

```bash
cargo install charmed-glow
```

## CLI Usage

```bash
# Render a file
glow README.md

# From stdin
cat README.md | glow -

# Theme override
glow --style dracula README.md

# Set wrap width
glow --width 80 README.md
```

## Configuration

Create `~/.config/glow/config.yml`:

```yaml
style: dark
width: 100
pager: true
mouse: true
local_only: false
```

## Library Usage

```rust
use glow::{Config, Reader};

let config = Config::new().style("dark").width(80).pager(true);
let reader = Reader::new(config);
let output = reader.read_file("README.md")?;
println!("{output}");
```

## Feature Flags

- `github`: enable GitHub README fetching.
- `syntax-highlighting`: enabled by default via `glamour`.

```toml
glow = { package = "charmed-glow", version = "0.1.2", features = ["github"] }
```

## Key Bindings

- `j` / `Down`: scroll down
- `k` / `Up`: scroll up
- `q`: quit

## Troubleshooting

- **No colors**: set `COLORTERM=truecolor` or use `--style ascii`.
- **Pager won’t scroll**: ensure you’re in a tty and not piping output.

## Limitations

- GitHub fetching requires the `github` feature.
- Rendering is terminal-only (no HTML export).

## FAQ

**Can I embed Glow in my own app?**  
Yes. Use the `Reader` API from the library.

**Does it support custom themes?**  
Yes, via glamour style configuration.

## About Contributions

Please don't take this the wrong way, but I do not accept outside contributions for any of my projects. I simply don't have the mental bandwidth to review anything, and it's my name on the thing, so I'm responsible for any problems it causes; thus, the risk-reward is highly asymmetric from my perspective. I'd also have to worry about other "stakeholders," which seems unwise for tools I mostly make for myself for free. Feel free to submit issues, and even PRs if you want to illustrate a proposed fix, but know I won't merge them directly. Instead, I'll have Claude or Codex review submissions via `gh` and independently decide whether and how to address them. Bug reports in particular are welcome. Sorry if this offends, but I want to avoid wasted time and hurt feelings. I understand this isn't in sync with the prevailing open-source ethos that seeks community contributions, but it's the only way I can move at this velocity and keep my sanity.

## License

MIT. See `LICENSE` at the repository root.
