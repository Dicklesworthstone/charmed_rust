//! Benchmarks for glamour markdown parsing and rendering.

// These casts are safe in benchmarks where values are small/bounded
#![expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::ref_as_ptr,
    clippy::too_many_lines
)]

use std::hint::black_box;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
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

// === LRU Cache Benchmarks (bd-3h5r) ===

#[cfg(feature = "syntax-highlighting")]
fn benchmark_lru_cache(c: &mut Criterion) {
    use glamour::syntax::StyleCache;

    let mut group = c.benchmark_group("glamour/lru_cache");

    // Benchmark 1: Cache hit performance
    // Target: <100ns per hit (O(1) operation)
    group.bench_function("cache_hit", |b| {
        use syntect::highlighting::{
            Color as SynColor, FontStyle as SynFontStyle, Style as SynStyle,
        };

        let mut cache = StyleCache::new();
        let style = SynStyle {
            foreground: SynColor {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            },
            background: SynColor {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
            font_style: SynFontStyle::BOLD,
        };

        // Warm up
        let _ = cache.get_or_convert(style);

        b.iter(|| {
            let result = cache.get_or_convert(style);
            black_box(result as *const _)
        });
    });

    // Benchmark 2: Cache miss (conversion + storage)
    group.bench_function("cache_miss", |b| {
        use syntect::highlighting::{
            Color as SynColor, FontStyle as SynFontStyle, Style as SynStyle,
        };

        b.iter_custom(|iters| {
            let mut total = std::time::Duration::ZERO;

            for i in 0..iters {
                let mut cache = StyleCache::new();
                let style = SynStyle {
                    foreground: SynColor {
                        r: (i % 256) as u8,
                        g: ((i * 7) % 256) as u8,
                        b: ((i * 13) % 256) as u8,
                        a: 255,
                    },
                    background: SynColor {
                        r: 0,
                        g: 0,
                        b: 0,
                        a: 0,
                    },
                    font_style: SynFontStyle::empty(),
                };

                let start = Instant::now();
                let _ = black_box(cache.get_or_convert(style));
                total += start.elapsed();
            }

            total
        });
    });

    // Benchmark 3: Heavy style mixing (round-robin access pattern)
    // Simulates real workload: 20 distinct styles accessed repeatedly
    group.bench_function("heavy_mixing_20_styles", |b| {
        use syntect::highlighting::{
            Color as SynColor, FontStyle as SynFontStyle, Style as SynStyle,
        };

        let mut cache = StyleCache::with_capacity(50);

        // Create 20 distinct styles
        let styles: Vec<_> = (0..20)
            .map(|i| SynStyle {
                foreground: SynColor {
                    r: (i * 10) as u8,
                    g: ((i * 7) % 255) as u8,
                    b: ((i * 13) % 255) as u8,
                    a: 255,
                },
                background: SynColor {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 0,
                },
                font_style: SynFontStyle::empty(),
            })
            .collect();

        // Warm up
        for style in &styles {
            let _ = cache.get_or_convert(*style);
        }

        let mut idx = 0usize;
        b.iter(|| {
            let style = styles[idx % styles.len()];
            idx = idx.wrapping_add(1);
            let result = cache.get_or_convert(style);
            black_box(result as *const _)
        });
    });

    // Benchmark 4: LRU promotion (accessing existing entries)
    // This is the key O(1) vs O(n) improvement
    group.bench_function("lru_promotion", |b| {
        use syntect::highlighting::{
            Color as SynColor, FontStyle as SynFontStyle, Style as SynStyle,
        };

        let mut cache = StyleCache::with_capacity(256);

        // Fill cache to capacity
        for i in 0..256 {
            let style = SynStyle {
                foreground: SynColor {
                    r: i as u8,
                    g: 0,
                    b: 0,
                    a: 255,
                },
                background: SynColor {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 0,
                },
                font_style: SynFontStyle::empty(),
            };
            let _ = cache.get_or_convert(style);
        }

        // Access the oldest entry (would be O(n) with Vec::remove, O(1) with LRU)
        let oldest_style = SynStyle {
            foreground: SynColor {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            },
            background: SynColor {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
            font_style: SynFontStyle::empty(),
        };

        b.iter(|| {
            let result = cache.get_or_convert(oldest_style);
            // Convert reference to pointer to avoid lifetime escape
            black_box(result as *const _)
        });
    });

    // Benchmark 5: Eviction under pressure
    // Test performance when cache is at capacity and must evict
    group.bench_function("eviction_pressure", |b| {
        use syntect::highlighting::{
            Color as SynColor, FontStyle as SynFontStyle, Style as SynStyle,
        };

        let mut cache = StyleCache::with_capacity(100);

        // Fill to capacity
        for i in 0..100 {
            let style = SynStyle {
                foreground: SynColor {
                    r: i as u8,
                    g: 0,
                    b: 0,
                    a: 255,
                },
                background: SynColor {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 0,
                },
                font_style: SynFontStyle::empty(),
            };
            let _ = cache.get_or_convert(style);
        }

        let mut idx = 100u16;
        b.iter(|| {
            let style = SynStyle {
                foreground: SynColor {
                    r: (idx % 256) as u8,
                    g: ((idx >> 8) % 256) as u8,
                    b: 255,
                    a: 255,
                },
                background: SynColor {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 0,
                },
                font_style: SynFontStyle::empty(),
            };
            idx = idx.wrapping_add(1);
            let result = cache.get_or_convert(style);
            black_box(result as *const _)
        });
    });

    group.finish();
}

#[cfg(not(feature = "syntax-highlighting"))]
fn benchmark_lru_cache(_c: &mut Criterion) {
    // LRU cache benchmarks require syntax-highlighting feature
}

criterion_group!(
    glamour_benches,
    benchmark_parsing,
    benchmark_full_render,
    benchmark_elements,
    benchmark_config_impact,
    benchmark_memory,
    benchmark_lru_cache
);
criterion_main!(glamour_benches);
