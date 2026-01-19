# Theme and Styling Guidance

Glamour table rendering uses two layers of styling:

1. **High-level renderer styles** via `StyleConfig` and `StyleTable`.
2. **Low-level table rendering** via `TableRenderConfig` and `HeaderStyle`.

## Renderer StyleConfig

When rendering Markdown with `Renderer`, table separators are controlled by
`StyleConfig::table` (`StyleTable`). This is the simplest way to keep table
style consistent with the rest of the Markdown output.

```rust
use glamour::{Renderer, StyleConfig};

let mut config = StyleConfig::default();
config.table = config.table.separators("│", "│", "─");

let renderer = Renderer::new().with_style_config(config);
let _ = renderer;
```

## Low-level Table Styling

When rendering a `ParsedTable` directly, use `TableRenderConfig` and
`HeaderStyle`:

```rust
use glamour::table::{TableRenderConfig, HeaderStyle, ROUNDED_BORDER};

let config = TableRenderConfig::new().border(ROUNDED_BORDER);
let header = HeaderStyle::new().bold().foreground("#ffffff");

let _ = (config, header);
```

## Consistency Tips

- Use the same border style (ASCII vs Unicode) across your output.
- Keep padding consistent between table cells and other layout blocks.
- If you theme the header, ensure the text contrast remains readable.
