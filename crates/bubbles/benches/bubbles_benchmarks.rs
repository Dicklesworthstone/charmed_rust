//! Benchmarks for bubbles TUI components.

use bubbles::list::{DefaultDelegate, Item, List};
use bubbles::paginator::{Paginator, Type as PaginatorType};
use bubbles::spinner::{spinners, SpinnerModel};
use bubbles::textinput::TextInput;
use bubbles::viewport::Viewport;
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

/// Simple item for benchmarking.
#[derive(Clone)]
struct BenchItem {
    title: String,
}

impl Item for BenchItem {
    fn filter_value(&self) -> &str {
        &self.title
    }
}

fn benchmark_list_creation(c: &mut Criterion) {
    let items: Vec<BenchItem> = (0..100)
        .map(|i| BenchItem {
            title: format!("Item {}", i),
        })
        .collect();

    c.bench_function("List::new with 100 items", |b| {
        b.iter(|| black_box(List::new(items.clone(), DefaultDelegate::new(), 80, 20)));
    });
}

fn benchmark_list_view(c: &mut Criterion) {
    let items: Vec<BenchItem> = (0..100)
        .map(|i| BenchItem {
            title: format!("Item {}", i),
        })
        .collect();
    let list = List::new(items, DefaultDelegate::new(), 80, 20);

    c.bench_function("List::view", |b| {
        b.iter(|| black_box(list.view()));
    });
}

fn benchmark_viewport_creation(c: &mut Criterion) {
    c.bench_function("Viewport::new", |b| {
        b.iter(|| black_box(Viewport::new(80, 24)));
    });
}

fn benchmark_viewport_view(c: &mut Criterion) {
    let content = (0..100)
        .map(|i| format!("Line {}: Some content here", i))
        .collect::<Vec<_>>()
        .join("\n");

    let mut viewport = Viewport::new(80, 24);
    viewport.set_content(&content);

    c.bench_function("Viewport::view", |b| {
        b.iter(|| black_box(viewport.view()));
    });
}

fn benchmark_viewport_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("viewport_throughput");

    for lines in [100, 500, 1000, 5000].iter() {
        let content = (0..*lines)
            .map(|i| format!("Line {}: Some content here with more text", i))
            .collect::<Vec<_>>()
            .join("\n");

        let mut viewport = Viewport::new(80, 24);
        viewport.set_content(&content);

        group.throughput(Throughput::Elements(*lines as u64));
        group.bench_function(format!("{} lines", lines), |b| {
            b.iter(|| black_box(viewport.view()));
        });
    }
    group.finish();
}

fn benchmark_textinput_creation(c: &mut Criterion) {
    c.bench_function("TextInput::new", |b| {
        b.iter(|| black_box(TextInput::new()));
    });
}

fn benchmark_textinput_view(c: &mut Criterion) {
    let mut input = TextInput::new();
    input.set_value("Hello, World!");
    input.focus();

    c.bench_function("TextInput::view", |b| {
        b.iter(|| black_box(input.view()));
    });
}

fn benchmark_paginator_view(c: &mut Criterion) {
    let paginator = Paginator::new().total_pages(100).per_page(10);

    c.bench_function("Paginator::view arabic", |b| {
        b.iter(|| black_box(paginator.view()));
    });

    let dots_paginator = Paginator::new()
        .display_type(PaginatorType::Dots)
        .total_pages(10);

    c.bench_function("Paginator::view dots", |b| {
        b.iter(|| black_box(dots_paginator.view()));
    });
}

fn benchmark_spinner_view(c: &mut Criterion) {
    let spinner = SpinnerModel::with_spinner(spinners::dot());

    c.bench_function("SpinnerModel::view", |b| {
        b.iter(|| black_box(spinner.view()));
    });
}

criterion_group!(
    benches,
    benchmark_list_creation,
    benchmark_list_view,
    benchmark_viewport_creation,
    benchmark_viewport_view,
    benchmark_viewport_throughput,
    benchmark_textinput_creation,
    benchmark_textinput_view,
    benchmark_paginator_view,
    benchmark_spinner_view,
);

criterion_main!(benches);
