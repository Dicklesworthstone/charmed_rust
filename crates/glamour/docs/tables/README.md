# Table Rendering

Glamour renders Markdown tables using a two-step pipeline:

1. **Parsing** - pulldown-cmark table events are converted into a `ParsedTable`.
2. **Rendering** - `render_table` (and related helpers) compute column widths,
   apply alignment, and draw borders using lipgloss-friendly strings.

This module is available at `glamour::table`.

## Quick Start

Render a Markdown table with the default Glamour renderer:

```rust
use glamour::{render, Style};

let markdown = "| Name | Age |\n|---|---|\n| Alice | 30 |\n| Bob | 25 |";
let output = render(markdown, Style::Dark).unwrap();
assert!(output.contains("Alice"));
```

If you already have table data and want to render it directly:

```rust
use glamour::table::{ParsedTable, TableCell, TableRenderConfig, render_table, ASCII_BORDER};
use pulldown_cmark::Alignment;

let table = ParsedTable {
    alignments: vec![Alignment::Left, Alignment::Right],
    header: vec![
        TableCell::new("Name", Alignment::Left),
        TableCell::new("Age", Alignment::Right),
    ],
    rows: vec![
        vec![
            TableCell::new("Alice", Alignment::Left),
            TableCell::new("30", Alignment::Right),
        ],
    ],
};

let config = TableRenderConfig::new().border(ASCII_BORDER);
let output = render_table(&table, &config);
assert!(output.contains("+"));
```

## Alignment

Markdown alignment is defined in the separator row:

| Markdown | Alignment |
|---------|-----------|
| `:---`  | Left      |
| `:---:` | Center    |
| `---:`  | Right     |
| `---`   | Default (Left) |

Alignment is applied per column; individual cells inherit their column
alignment when they are parsed.

## Styling

Use `TableRenderConfig` to control borders and separators. For header styling,
use `HeaderStyle` to apply bold/italic/underline, colors, and text transforms.

```rust
use glamour::table::{HeaderStyle, TableRenderConfig, ROUNDED_BORDER, TextTransform};

let config = TableRenderConfig::new()
    .border(ROUNDED_BORDER)
    .row_separator(true);

let header_style = HeaderStyle::new()
    .bold()
    .transform(TextTransform::Uppercase);

let _ = (config, header_style); // configure renderer with these values
```

### TableRenderConfig Options

- `border` - Select a `TableBorder` character set.
- `header_separator` - Draw a separator after the header row.
- `row_separator` - Draw separators between body rows.
- `cell_padding` - Spaces on each side of cell content.

## Behavior Summary

- **Column width** is based on the widest cell content (header included).
- **Unicode width** uses `unicode-width` for display-accurate alignment.
- **Ragged rows** (fewer cells than columns) are padded with empty cells.
- **Empty cells** render as padding-only columns.

## Logging

Table rendering does not currently emit structured logs. If you need visibility
into parsing or width calculations, wrap calls to `TableParser` and the render
helpers with your own tracing.

## See Also

- `alignment.md` - Alignment rules and examples
- `borders.md` - Border styles and separators
- `headers.md` - Header styling
- `themes.md` - Theme and styling guidance
- `unicode.md` - Unicode width and truncation
