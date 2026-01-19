//! Benchmarks for glamour markdown parsing and rendering.

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use glamour::{Renderer, Style, StyleBlock, StyleConfig, StylePrimitive};
use pulldown_cmark::Parser;
use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};
use std::alloc::System;
use std::fmt::Write;
use std::time::Instant;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

const SMALL_DOC: &str = include_str!("fixtures/small.md");
const MEDIUM_DOC: &str = include_str!("fixtures/medium.md");
const LARGE_DOC: &str = include_str!("fixtures/large.md");

fn custom_style_config() -> StyleConfig {
    let mut config = Style::Dark.config();
    config.h1 =
        StyleBlock::new().style(StylePrimitive::new().prefix("## ").color("196").bold(true));
    config.code = StyleBlock::new().style(
        StylePrimitive::new()
            .prefix(" ")
            .suffix(" ")
            .color("45")
            .background_color("236"),
    );
    config
}

fn benchmark_parsing(c: &mut Criterion) {
    let large = LARGE_DOC.repeat(8);
    let docs = [
        ("small", SMALL_DOC),
        ("medium", MEDIUM_DOC),
        ("large", large.as_str()),
    ];

    let mut group = c.benchmark_group("glamour/parsing");
    for (name, doc) in docs {
        group.throughput(Throughput::Bytes(doc.len() as u64));
        group.bench_with_input(BenchmarkId::new("parse", name), doc, |b, doc| {
            b.iter(|| black_box(Parser::new(doc).count()));
        });
    }
    group.finish();
}

fn benchmark_full_render(c: &mut Criterion) {
    let large = LARGE_DOC.repeat(8);
    let docs = [
        ("small", SMALL_DOC),
        ("medium", MEDIUM_DOC),
        ("large", large.as_str()),
    ];

    let mut group = c.benchmark_group("glamour/render");
    for (name, doc) in docs {
        group.throughput(Throughput::Bytes(doc.len() as u64));
        group.bench_with_input(BenchmarkId::new("full", name), doc, |b, doc| {
            let renderer = Renderer::new().with_style(Style::Dark);
            b.iter(|| black_box(renderer.render(doc)));
        });
    }
    group.finish();
}

fn benchmark_elements(c: &mut Criterion) {
    let mut group = c.benchmark_group("glamour/elements");

    let mut headers_base = String::new();
    for n in 1..=6 {
        let _ = write!(&mut headers_base, "{} Header Level {n}\n\n", "#".repeat(n));
    }
    let headers = headers_base.repeat(100);
    group.bench_function("headers", |b| {
        let renderer = Renderer::new().with_style(Style::Dark);
        b.iter(|| black_box(renderer.render(&headers)));
    });

    let mut list = String::new();
    for i in 0..100 {
        let _ = writeln!(&mut list, "- Item {i}");
    }
    group.bench_function("unordered_list_100", |b| {
        let renderer = Renderer::new().with_style(Style::Dark);
        b.iter(|| black_box(renderer.render(&list)));
    });

    let mut nested_list = String::new();
    for i in 0..50 {
        let _ = writeln!(&mut nested_list, "- Item {i}");
        let _ = writeln!(&mut nested_list, "  - Nested {i}");
        let _ = writeln!(&mut nested_list, "    - Deep {i}");
    }
    group.bench_function("nested_list", |b| {
        let renderer = Renderer::new().with_style(Style::Dark);
        b.iter(|| black_box(renderer.render(&nested_list)));
    });

    let code_blocks = r#"
```rust
fn main() {
    println!("Hello");
}
```
"#
    .repeat(50);
    group.bench_function("code_blocks_50", |b| {
        let renderer = Renderer::new().with_style(Style::Dark);
        b.iter(|| black_box(renderer.render(&code_blocks)));
    });

    let mut links = String::new();
    for i in 0..100 {
        let _ = writeln!(
            &mut links,
            "[Link {i}](https://example.com/{i}) and **bold** and *italic*"
        );
    }
    group.bench_function("links_emphasis_100", |b| {
        let renderer = Renderer::new().with_style(Style::Dark);
        b.iter(|| black_box(renderer.render(&links)));
    });

    let table = r"
| Col 1 | Col 2 | Col 3 |
|-------|-------|-------|
| A | B | C |
"
    .repeat(50);
    group.bench_function("tables_50", |b| {
        let renderer = Renderer::new().with_style(Style::Dark);
        b.iter(|| black_box(renderer.render(&table)));
    });

    group.finish();
}

fn benchmark_config_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("glamour/config");

    group.bench_function("default_dark", |b| {
        let renderer = Renderer::new().with_style(Style::Dark);
        b.iter(|| black_box(renderer.render(MEDIUM_DOC)));
    });

    group.bench_function("light_style", |b| {
        let renderer = Renderer::new().with_style(Style::Light);
        b.iter(|| black_box(renderer.render(MEDIUM_DOC)));
    });

    let custom = custom_style_config();
    group.bench_function("custom_styles", |b| {
        let renderer = Renderer::new().with_style_config(custom.clone());
        b.iter(|| black_box(renderer.render(MEDIUM_DOC)));
    });

    #[cfg(feature = "syntax-highlighting")]
    {
        let mut config = Style::Dark.config();
        config.code_block = config.code_block.clone().theme("base16-ocean.dark");
        let renderer = Renderer::new().with_style_config(config);
        group.bench_function("with_syntax_highlighting", |b| {
            b.iter(|| black_box(renderer.render(MEDIUM_DOC)));
        });
    }

    group.finish();
}

fn benchmark_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("glamour/memory");

    for (name, doc) in [("small", SMALL_DOC), ("medium", MEDIUM_DOC)] {
        group.bench_function(format!("alloc_{name}"), |b| {
            let renderer = Renderer::new().with_style(Style::Dark);
            b.iter_custom(|iters| {
                let start = Instant::now();
                let region = Region::new(GLOBAL);

                for _ in 0..iters {
                    black_box(renderer.render(doc));
                }

                let duration = start.elapsed();
                let stats = region.change();
                let iter_count = iters.max(1);
                let bytes_per_iter = (stats.bytes_allocated as u64) / iter_count;
                let allocs_per_iter = (stats.allocations as u64) / iter_count;

                eprintln!(
                    "glamour/memory {name}: bytes_total={}, allocs_total={}, bytes_per_iter={}, allocs_per_iter={}",
                    stats.bytes_allocated,
                    stats.allocations,
                    bytes_per_iter,
                    allocs_per_iter
                );

                duration
            });
        });
    }

    group.finish();
}

criterion_group!(
    glamour_benches,
    benchmark_parsing,
    benchmark_full_render,
    benchmark_elements,
    benchmark_config_impact,
    benchmark_memory
);
criterion_main!(glamour_benches);
