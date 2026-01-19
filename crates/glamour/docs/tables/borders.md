# Border Styles

Table rendering supports multiple border character sets via `TableBorder`.
Use the built-in constants or provide your own character set.

## Built-in Borders

| Constant | Description |
|----------|-------------|
| `ASCII_BORDER` | `+`, `-`, `|` for maximum compatibility |
| `NORMAL_BORDER` | Unicode box drawing (sharp corners) |
| `ROUNDED_BORDER` | Unicode box drawing (rounded corners) |
| `DOUBLE_BORDER` | Unicode double-line borders |
| `NO_BORDER` | Empty strings (no visible borders) |

## Example

```rust
use glamour::table::{TableRenderConfig, ROUNDED_BORDER, render_table, ParsedTable};

let config = TableRenderConfig::new().border(ROUNDED_BORDER);
let _ = (config, render_table, ParsedTable::new());
```

## Separators

- `header_separator` inserts a divider between the header and body.
- `row_separator` inserts dividers between body rows.

These options are controlled via `TableRenderConfig`.
