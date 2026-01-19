//! Benchmarks for lipgloss style rendering.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use lipgloss::{Border, Color, Position, Style};

fn benchmark_style_creation(c: &mut Criterion) {
    c.bench_function("Style::new", |b| {
        b.iter(|| black_box(Style::new()));
    });
}

fn benchmark_style_with_properties(c: &mut Criterion) {
    c.bench_function("Style with color and bold", |b| {
        b.iter(|| black_box(Style::new().foreground("#ff00ff").bold()));
    });

    c.bench_function("Style with full properties", |b| {
        b.iter(|| {
            black_box(
                Style::new()
                    .foreground("#ff00ff")
                    .background("#1a1a1a")
                    .bold()
                    .padding((1, 2))
                    .border(Border::rounded())
                    .align(Position::Center),
            )
        });
    });
}

fn benchmark_style_render(c: &mut Criterion) {
    let style = Style::new().foreground("#ff00ff").bold();

    c.bench_function("Style::render short text", |b| {
        b.iter(|| black_box(style.render("Hello")));
    });

    let long_text = "x".repeat(1000);
    c.bench_function("Style::render long text", |b| {
        b.iter(|| black_box(style.render(&long_text)));
    });
}

fn benchmark_style_render_with_border(c: &mut Criterion) {
    let style = Style::new()
        .foreground("#ff00ff")
        .background("#1a1a1a")
        .padding((1, 2))
        .border(Border::rounded());

    c.bench_function("Style::render with border", |b| {
        b.iter(|| black_box(style.render("Hello, World!")));
    });
}

fn benchmark_color_parsing(c: &mut Criterion) {
    c.bench_function("Color::from hex", |b| {
        b.iter(|| black_box(Color::from("#ff00ff")));
    });

    c.bench_function("Color::from ansi", |b| {
        b.iter(|| black_box(Color::from("196")));
    });
}

fn benchmark_style_clone(c: &mut Criterion) {
    let style = Style::new()
        .foreground("#ff00ff")
        .background("#1a1a1a")
        .bold()
        .padding((1, 2))
        .border(Border::rounded());

    c.bench_function("Style::clone", |b| {
        b.iter(|| black_box(style.clone()));
    });
}

fn benchmark_render_throughput(c: &mut Criterion) {
    let style = Style::new().foreground("#ff00ff").bold();

    let mut group = c.benchmark_group("render_throughput");
    for size in [10, 100, 1000, 10000].iter() {
        let text = "x".repeat(*size);
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_function(format!("{} chars", size), |b| {
            b.iter(|| black_box(style.render(&text)));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    benchmark_style_creation,
    benchmark_style_with_properties,
    benchmark_style_render,
    benchmark_style_render_with_border,
    benchmark_color_parsing,
    benchmark_style_clone,
    benchmark_render_throughput,
);

criterion_main!(benches);
