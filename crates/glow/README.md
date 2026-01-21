# Glow

A terminal-based markdown reader and browser, powered by `glamour`.

Glow makes it easy to read and browse markdown files directly in the terminal,
with beautiful rendering and an intuitive pager interface.

## Features

- **Markdown Rendering**: Beautiful terminal rendering via `glamour`
- **Multiple Styles**: Dark, light, ASCII, pink, and auto themes
- **Configurable Width**: Word wrap for any terminal width
- **Pager Mode**: Scroll through long documents
- **File Browser**: Browse local markdown files
- **Document Stash**: Save and organize frequently accessed files
- **GitHub Integration**: Fetch READMEs from GitHub (with `github` feature)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
glow = { version = "0.1", path = "../glow" }
```

For GitHub README fetching:

```toml
[dependencies]
glow = { version = "0.1", path = "../glow", features = ["github"] }
```

## Quick Start

### Library Usage

```rust
use glow::{Config, Reader};

fn main() -> std::io::Result<()> {
    // Create a reader with custom configuration
    let config = Config::new()
        .style("dark")
        .width(80)
        .pager(true);

    let reader = Reader::new(config);

    // Render a markdown file
    let output = reader.read_file("README.md")?;
    println!("{output}");

    Ok(())
}
```

### CLI Usage

```bash
# Render a markdown file
glow README.md

# Use a different style
glow --style light README.md

# Set custom width
glow --width 80 README.md

# Disable pager
glow --no-pager README.md

# Read from stdin
cat README.md | glow -

# Browse local files
glow --local
```

## Configuration

### Config Builder

```rust
use glow::Config;

let config = Config::new()
    .pager(true)        // Enable pager (default: true)
    .width(100)         // Set wrap width (default: terminal width)
    .style("dark");     // Set theme (default: "dark")
```

### Available Styles

| Style | Description |
|-------|-------------|
| `dark` | Dark terminal theme (default) |
| `light` | Light terminal theme |
| `ascii` | ASCII-only output |
| `pink` | Pink accent theme |
| `auto` | Detect from terminal |
| `no-tty` | Plain output without styling |

### Configuration File

Create `~/.config/glow/config.yml`:

```yaml
style: dark
width: 100
pager: true
mouse: true
local_only: false
```

## Key Bindings

| Key | Action |
|-----|--------|
| `j` / `Down` | Scroll down |
| `k` / `Up` | Scroll up |
| `d` / `Page Down` | Page down |
| `u` / `Page Up` | Page up |
| `g` / `Home` | Go to top |
| `G` / `End` | Go to bottom |
| `/` | Search |
| `n` | Next search result |
| `N` | Previous search result |
| `q` / `Esc` | Quit |

## API Reference

### Reader

The main type for reading and rendering markdown:

```rust
use glow::{Config, Reader};

let reader = Reader::new(Config::default());

// Read from file
let output = reader.read_file("README.md")?;

// Render markdown string
let output = reader.render_markdown("# Hello World")?;
```

### Stash

Document stash for saving frequently accessed files:

```rust
use glow::Stash;

let mut stash = Stash::new();
stash.add("README.md");
stash.add("docs/guide.md");

for doc in stash.documents() {
    println!("{doc}");
}
```

### File Browser

Browse local markdown files with `bubbletea` TUI:

```rust
use glow::browser::{FileBrowser, BrowserConfig};

let config = BrowserConfig::new("/path/to/docs");
let browser = FileBrowser::new(config);
```

## Examples

See the `examples/` directory for complete examples:

- Basic markdown rendering
- File browser TUI
- Configuration patterns
- GitHub README fetching

## Architecture

```text
┌─────────────────────────────────────────────────────────┐
│                      glow                               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
│  │  Config  │→ │  Reader  │→ │ glamour  │              │
│  └──────────┘  └──────────┘  └──────────┘              │
│       │                            ↓                    │
│       │        ┌──────────────────────────┐            │
│       └───────→│  FileBrowser (bubbletea) │            │
│                └──────────────────────────┘            │
└─────────────────────────────────────────────────────────┘
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `github` | Enable GitHub README fetching |

## License

MIT
