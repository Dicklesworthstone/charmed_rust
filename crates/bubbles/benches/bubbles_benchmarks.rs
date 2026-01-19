#![forbid(unsafe_code)]

//! Benchmarks for bubbles TUI components.

use bubbles::list::{DefaultDelegate, Item, List};
use bubbles::paginator::{Paginator, Type as PaginatorType};
use bubbles::progress::Progress;
use bubbles::spinner::{SpinnerModel, spinners};
use bubbles::table::{Column, Row, Table};
use bubbles::textinput::TextInput;
use bubbles::viewport::Viewport;
use bubbletea::{KeyMsg, KeyType, Message};
use criterion::{
    BatchSize, BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main,
};

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

fn build_items(count: usize) -> Vec<BenchItem> {
    (0..count)
        .map(|i| BenchItem {
            title: format!("Item {i}"),
        })
        .collect()
}

fn build_table_columns() -> Vec<Column> {
    vec![
        Column::new("Name", 18),
        Column::new("Status", 12),
        Column::new("Region", 12),
        Column::new("Score", 8),
    ]
}

fn build_table_rows(count: usize) -> Vec<Row> {
    (0..count)
        .map(|i| {
            vec![
                format!("Person {i}"),
                if i % 2 == 0 { "Online" } else { "Offline" }.to_string(),
                {
                    let zone = i % 8;
                    format!("Zone {zone}")
                },
                {
                    let score = i * 7;
                    format!("{score}")
                },
            ]
        })
        .collect()
}

fn build_viewport_content(lines: usize) -> String {
    (0..lines)
        .map(|i| format!("Line {i}: Some content here with more text"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn bench_list(c: &mut Criterion) {
    let mut group = c.benchmark_group("bubbles/list");

    for count in [10_usize, 100, 1000] {
        let items = build_items(count);
        group.bench_with_input(BenchmarkId::new("create", count), &items, |b, items| {
            b.iter(|| black_box(List::new(items.clone(), DefaultDelegate::new(), 80, 20)));
        });
    }

    let list = List::new(build_items(100), DefaultDelegate::new(), 80, 20);
    group.bench_function("view_100", |b| b.iter(|| black_box(list.view())));

    group.bench_function("navigate_100", |b| {
        b.iter_batched(
            || list.clone(),
            |mut list| {
                for _ in 0..10 {
                    list.cursor_down();
                }
                for _ in 0..5 {
                    list.cursor_up();
                }
                black_box(list.selected_item());
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("filter_100", |b| {
        b.iter_batched(
            || List::new(build_items(100), DefaultDelegate::new(), 80, 20),
            |mut list| {
                list.set_filter_value("Item 5");
                black_box(list.view());
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_table(c: &mut Criterion) {
    let mut group = c.benchmark_group("bubbles/table");
    let columns = build_table_columns();

    for count in [10_usize, 100, 1000] {
        let rows = build_table_rows(count);
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::new("view", count), &rows, |b, rows| {
            let table = Table::new()
                .columns(columns.clone())
                .rows(rows.clone())
                .width(80)
                .height(20)
                .focused(true);
            b.iter(|| black_box(table.view()));
        });
    }

    group.bench_function("navigate", |b| {
        let rows = build_table_rows(200);
        let table = Table::new()
            .columns(columns.clone())
            .rows(rows)
            .width(80)
            .height(20)
            .focused(true);

        b.iter_batched(
            || table.clone(),
            |mut table| {
                table.move_down(10);
                table.move_up(5);
                table.goto_bottom();
                table.goto_top();
                black_box(table.selected_row());
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("set_columns_rows", |b| {
        b.iter_batched(
            || (build_table_columns(), build_table_rows(150)),
            |(columns, rows)| {
                let mut table = Table::new().width(80).height(20);
                table.set_columns(columns);
                table.set_rows(rows);
                black_box(table.view());
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_viewport(c: &mut Criterion) {
    let mut group = c.benchmark_group("bubbles/viewport");

    for lines in [100_usize, 1000, 10_000] {
        let content = build_viewport_content(lines);
        let mut viewport = Viewport::new(80, 24);
        viewport.set_content(&content);

        group.throughput(Throughput::Elements(lines as u64));
        group.bench_with_input(BenchmarkId::new("render", lines), &viewport, |b, vp| {
            b.iter(|| black_box(vp.view()));
        });
    }

    group.bench_function("scroll_ops", |b| {
        b.iter_batched(
            || {
                let content = build_viewport_content(2000);
                let mut viewport = Viewport::new(80, 24);
                viewport.set_content(&content);
                viewport
            },
            |mut viewport| {
                viewport.scroll_down(5);
                viewport.scroll_up(2);
                viewport.page_down();
                viewport.page_up();
                black_box(viewport.view());
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_textinput(c: &mut Criterion) {
    let mut group = c.benchmark_group("bubbles/textinput");

    group.bench_function("create", |b| b.iter(|| black_box(TextInput::new())));

    let mut input = TextInput::new();
    input.set_value("Hello, World!");
    input.focus();

    group.bench_function("view_with_text", |b| b.iter(|| black_box(input.view())));

    group.bench_function("insert_chars", |b| {
        b.iter_batched(
            || {
                let mut input = TextInput::new();
                input.focus();
                input
            },
            |mut input| {
                for c in ['a', 'b', 'c', 'd', 'e'] {
                    let msg = Message::new(KeyMsg::from_char(c));
                    input.update(msg);
                }
                black_box(input.value());
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("cursor_movement", |b| {
        b.iter_batched(
            || {
                let mut input = TextInput::new();
                input.set_value(&"x".repeat(1000));
                input.focus();
                input
            },
            |mut input| {
                input.update(Message::new(KeyMsg::from_type(KeyType::Left)));
                input.update(Message::new(KeyMsg::from_type(KeyType::Right)));
                input.cursor_start();
                input.cursor_end();
                black_box(input.position());
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_paginator(c: &mut Criterion) {
    let mut group = c.benchmark_group("bubbles/paginator");

    let paginator = Paginator::new().total_pages(100).per_page(10);
    group.bench_function("view_arabic", |b| b.iter(|| black_box(paginator.view())));

    let dots_paginator = Paginator::new()
        .display_type(PaginatorType::Dots)
        .total_pages(10);
    group.bench_function("view_dots", |b| b.iter(|| black_box(dots_paginator.view())));

    group.finish();
}

fn bench_spinner_and_progress(c: &mut Criterion) {
    let mut group = c.benchmark_group("bubbles/animated");

    let spinner = SpinnerModel::with_spinner(spinners::dot());
    group.bench_function("spinner_view", |b| b.iter(|| black_box(spinner.view())));

    group.bench_function("spinner_update", |b| {
        b.iter_batched(
            || spinner.clone(),
            |mut spinner| {
                let msg = spinner.tick();
                spinner.update(msg);
                black_box(spinner.view());
            },
            BatchSize::SmallInput,
        );
    });

    let progress = Progress::new().width(40);
    group.bench_function("progress_view_50", |b| {
        b.iter(|| black_box(progress.view_as(0.5)));
    });

    group.bench_function("progress_set_percent", |b| {
        b.iter_batched(
            || Progress::new().width(40),
            |mut progress| {
                progress.set_percent(0.75);
                black_box(progress.view());
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_list,
    bench_table,
    bench_viewport,
    bench_textinput,
    bench_paginator,
    bench_spinner_and_progress,
);
criterion_main!(benches);
