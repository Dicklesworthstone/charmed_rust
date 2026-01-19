# Unicode Width and Truncation

Glamour uses the `unicode-width` crate to measure display width, which is
critical for correct alignment of wide characters (CJK, emoji, etc.).

## Width Calculation

Column widths are computed from the maximum display width across header and
body cells. The width uses Unicode display width, not byte length.

## Example

```text
| Name | Greeting |
|------|----------|
| 田中 | 你好     |
```

Both "田中" and "你好" are wide characters, and their width is calculated
correctly so the columns stay aligned.

## Truncation

`truncate_content` shortens long content while preserving display width. It
uses an ellipsis ("…") when truncation is required.

```rust
use glamour::table::truncate_content;

assert_eq!(truncate_content("Hello, World!", 5), "Hell…");
```
