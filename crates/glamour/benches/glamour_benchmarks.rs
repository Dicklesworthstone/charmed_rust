//! Benchmarks for glamour markdown rendering.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use glamour::{render, Style};

const SMALL_MARKDOWN: &str = r#"
# Hello World

This is a **bold** and *italic* paragraph.

- Item 1
- Item 2
- Item 3
"#;

const MEDIUM_MARKDOWN: &str = r#"
# Document Title

This is the introduction paragraph with **bold text** and *italic text*.

## Section 1

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor
incididunt ut labore et dolore magna aliqua.

### Subsection 1.1

Here's a list:

- First item with `inline code`
- Second item with [a link](https://example.com)
- Third item with more content

```rust
fn main() {
    println!("Hello, world!");
}
```

## Section 2

> This is a blockquote with some important information.
> It spans multiple lines.

| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
| A        | B        | C        |
| D        | E        | F        |

1. Numbered item one
2. Numbered item two
3. Numbered item three

---

The end.
"#;

fn generate_large_markdown(paragraphs: usize) -> String {
    let mut md = String::from("# Large Document\n\n");
    for i in 0..paragraphs {
        md.push_str(&format!("## Section {}\n\n", i + 1));
        md.push_str("Lorem ipsum dolor sit amet, consectetur adipiscing elit. ");
        md.push_str("Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. ");
        md.push_str("Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.\n\n");
        md.push_str("- Item with **bold**\n");
        md.push_str("- Item with *italic*\n");
        md.push_str("- Item with `code`\n\n");
    }
    md
}

fn benchmark_render_small(c: &mut Criterion) {
    c.bench_function("render small markdown", |b| {
        b.iter(|| black_box(render(SMALL_MARKDOWN, Style::Dark)));
    });
}

fn benchmark_render_medium(c: &mut Criterion) {
    c.bench_function("render medium markdown", |b| {
        b.iter(|| black_box(render(MEDIUM_MARKDOWN, Style::Dark)));
    });
}

fn benchmark_render_large(c: &mut Criterion) {
    let large = generate_large_markdown(100);

    c.bench_function("render large markdown (100 sections)", |b| {
        b.iter(|| black_box(render(&large, Style::Dark)));
    });
}

fn benchmark_style_variants(c: &mut Criterion) {
    c.bench_function("render with dark style", |b| {
        b.iter(|| black_box(render(MEDIUM_MARKDOWN, Style::Dark)));
    });

    c.bench_function("render with light style", |b| {
        b.iter(|| black_box(render(MEDIUM_MARKDOWN, Style::Light)));
    });
}

fn benchmark_render_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("markdown_throughput");
    for sections in [10, 50, 100, 200].iter() {
        let md = generate_large_markdown(*sections);
        let bytes = md.len();
        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_function(format!("{} sections", sections), |b| {
            b.iter(|| black_box(render(&md, Style::Dark)));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    benchmark_render_small,
    benchmark_render_medium,
    benchmark_render_large,
    benchmark_style_variants,
    benchmark_render_throughput,
);

criterion_main!(benches);
