# Large Fixture Base

This fixture is intentionally repeated in benchmarks to simulate a very large
markdown document. It contains a mix of elements that should exercise the
renderer across headings, lists, tables, blockquotes, and code blocks.

## Section 1

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor
incididunt ut labore et dolore magna aliqua.

- Alpha
- Beta
- Gamma

```rust
fn section_one() -> i32 {
    1
}
```

> A blockquote to test indentation and styling.

## Section 2

| Name | Value | Note |
|------|-------|------|
| A    | 1     | One  |
| B    | 2     | Two  |

Text with **bold**, *italic*, and `inline code`.

## Section 3

- Parent
  - Child
    - Grandchild

```text
Plain text code block.
Another line.
```

## Section 4

Paragraph one for section four.
Paragraph two for section four.
Paragraph three for section four.

## Section 5

1. First
2. Second
3. Third

## Section 6

> Another blockquote for coverage.

## Section 7

- Task list:
  - [x] Done
  - [ ] Pending

## Section 8

```rust
fn section_eight() {
    println!("section eight");
}
```

## Section 9

A final paragraph with a link: [example](https://example.com).

## Section 10

End of base fixture.
