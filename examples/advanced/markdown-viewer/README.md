# Markdown Viewer Example

Demonstrates rendering markdown content using glamour with scrollable viewport navigation.

## Running

```bash
cargo run -p example-markdown-viewer
```

## Key Concepts

### Markdown Rendering

Use glamour to render markdown with terminal styling:

```rust
use glamour::render;

let markdown = "# Hello\n**Bold** and *italic* text";
let rendered = render(markdown)?;
```

### Viewport Integration

Combine rendered markdown with scrollable viewport:

```rust
let mut viewport = Viewport::new(80, 20);
let content = glamour::render(markdown_source)?;
viewport.set_content(&content);
```

### Style Configuration

Customize markdown appearance:

```rust
use glamour::StyleConfig;

let config = StyleConfig::dark_theme();
let rendered = render_with_config(markdown, &config)?;
```

## Controls

| Key | Action |
|-----|--------|
| `j` / `↓` | Scroll down |
| `k` / `↑` | Scroll up |
| `f` / `PgDn` | Page down |
| `b` / `PgUp` | Page up |
| `g` | Go to top |
| `G` | Go to bottom |
| `q` | Quit |

## Supported Markdown

- Headers (h1-h6)
- Bold, italic, strikethrough
- Code blocks with syntax highlighting
- Lists (ordered and unordered)
- Blockquotes
- Horizontal rules
- Links (displayed inline)
- Tables

## Related Examples

- [viewport](../../intermediate/viewport) - Scrollable content basics
- [multi-component](../multi-component) - Multiple viewport use
