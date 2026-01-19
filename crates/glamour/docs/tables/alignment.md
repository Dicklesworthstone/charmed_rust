# Column Alignment

Glamour uses pulldown-cmark's table alignment markers. Alignment is defined
per column in the separator row and applied to every cell in that column.

## Markdown Alignment Markers

| Marker | Alignment |
|--------|-----------|
| `:---` | Left      |
| `:---:` | Center   |
| `---:` | Right     |
| `---`  | Default (Left) |

## Example

```text
| Left | Center | Right |
|:-----|:------:|------:|
| A    |   B    |     C |
```

## API Notes

The parsed alignment is stored in `ParsedTable.alignments`. Each `TableCell`
captures the alignment at parse time so rendering code does not need to
recompute it.
