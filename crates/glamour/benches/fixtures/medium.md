# Medium Fixture Document

This medium fixture simulates a typical README with sections, lists, tables,
and code blocks. It is long enough to measure rendering throughput.

## Overview

Glamour renders markdown into styled terminal output. This fixture stresses:

- Headings
- Paragraphs
- Lists
- Tables
- Links and emphasis
- Code blocks

## Highlights

Here is a numbered list:

1. First item with **bold** emphasis
2. Second item with *italic* emphasis
3. Third item with `inline code`

A task list:

- [x] Done item
- [ ] Pending item
- [ ] Another pending item

A nested list:

- Parent one
  - Child one
  - Child two
    - Grandchild one
- Parent two
  - Child three

## Table

| Feature | Status | Notes |
|---------|--------|-------|
| Parsing | Done   | Uses pulldown-cmark |
| Styling | Done   | Uses lipgloss |
| Themes  | WIP    | Dark, Light, Pink |

## Blockquote

> This is a blockquote that spans multiple lines.
> It should be indented and styled.

## Code

```rust
use glamour::{Renderer, Style};

fn main() {
    let md = "# Title\n\nHello";
    let output = Renderer::new().with_style(Style::Dark).render(md);
    println!("{}", output);
}
```

## Links

- Website: [Charm](https://charm.sh)
- Repository: [charmed_rust](https://github.com/Dicklesworthstone/charmed_rust)

## Repeated Section

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor
incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis
nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.

Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu
fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in
culpa qui officia deserunt mollit anim id est laborum.

## Another Section

- Bullet A
- Bullet B
- Bullet C

```text
Plain text code block to ensure styling works in non-rust blocks.
Line two of the code block.
```
