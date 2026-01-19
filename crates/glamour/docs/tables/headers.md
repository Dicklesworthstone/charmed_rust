# Header Styling

Header styling is controlled by `HeaderStyle`, which can be applied when
rendering a table. This allows you to bold, italicize, underline, recolor,
or transform the header text.

## Text Transformations

`TextTransform` supports:

- `None` - no change
- `Uppercase` - convert to upper-case
- `Lowercase` - convert to lower-case
- `Capitalize` - capitalize each word

## Example

```rust
use glamour::table::{HeaderStyle, TextTransform};

let style = HeaderStyle::new()
    .bold()
    .underline()
    .transform(TextTransform::Uppercase)
    .foreground("#ffffff")
    .background("#333333");

let _ = style;
```

## Notes

- Header styling is optional; if omitted, the header renders like any row.
- Styles are translated into a `lipgloss::Style` via `build_style()`.
