use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use lipgloss::{
    AdaptiveColor, AnsiColor, Border, Color, ColorProfile, Position, RgbColor, Style,
    TerminalColor, join_horizontal, join_vertical, place,
};

const SAMPLE_LINE: &str = "The quick brown fox jumps over the lazy dog.";
const SAMPLE_PARAGRAPH: &str =
    "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt.";

fn bench_style_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("lipgloss/style_creation");

    group.bench_function("Style::new", |b| {
        b.iter(|| black_box(Style::new()));
    });

    group.bench_function("Style::new_with_all_props", |b| {
        b.iter(|| {
            black_box(
                Style::new()
                    .foreground_color(RgbColor::new(255, 0, 0))
                    .background_color(RgbColor::new(0, 0, 255))
                    .bold()
                    .italic()
                    .underline()
                    .padding((1u16, 2u16, 1u16, 2u16))
                    .margin((1u16, 1u16, 1u16, 1u16))
                    .border(Border::rounded()),
            )
        });
    });

    group.finish();
}

fn bench_colors(c: &mut Criterion) {
    let mut group = c.benchmark_group("lipgloss/colors");

    group.bench_function("AnsiColor::from", |b| {
        b.iter(|| black_box(AnsiColor::from(196u8)));
    });

    group.bench_function("RgbColor::new", |b| {
        b.iter(|| black_box(RgbColor::new(255, 128, 64)));
    });

    group.bench_function("Color::hex_parse", |b| {
        b.iter(|| {
            let color = Color::from("#FF8040");
            black_box(color.as_rgb())
        });
    });

    group.bench_function("Color::ansi_parse", |b| {
        b.iter(|| {
            let color = Color::from("196");
            black_box(color.as_ansi())
        });
    });

    group.bench_function("AdaptiveColor::to_ansi_fg", |b| {
        let adaptive = AdaptiveColor {
            light: Color::from("#000000"),
            dark: Color::from("#ffffff"),
        };
        b.iter(|| black_box(adaptive.to_ansi_fg(ColorProfile::TrueColor, true)));
    });

    group.finish();
}

fn bench_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("lipgloss/rendering");

    let simple_style = Style::new().foreground("#ff0000");
    let complex_style = Style::new()
        .foreground("#ff0000")
        .background("#0000ff")
        .bold()
        .padding((1u16, 2u16))
        .border(Border::rounded());

    group.bench_function("render/short/simple", |b| {
        b.iter(|| black_box(simple_style.render(SAMPLE_LINE)));
    });

    group.bench_function("render/short/complex", |b| {
        b.iter(|| black_box(complex_style.render(SAMPLE_LINE)));
    });

    let medium = format!("{SAMPLE_PARAGRAPH}\n{SAMPLE_PARAGRAPH}\n{SAMPLE_LINE}");
    group.throughput(Throughput::Bytes(medium.len() as u64));
    group.bench_function("render/medium/simple", |b| {
        b.iter(|| black_box(simple_style.render(medium.as_str())));
    });

    let long = (0..80)
        .map(|_| SAMPLE_PARAGRAPH)
        .collect::<Vec<&str>>()
        .join("\n");
    group.throughput(Throughput::Bytes(long.len() as u64));
    group.bench_function("render/long/simple", |b| {
        b.iter(|| black_box(simple_style.render(long.as_str())));
    });

    group.finish();
}

fn bench_layout(c: &mut Criterion) {
    let mut group = c.benchmark_group("lipgloss/layout");

    let items: Vec<String> = (0..10)
        .map(|i| format!("Item {i}: {SAMPLE_LINE}"))
        .collect();
    let item_refs: Vec<&str> = items.iter().map(String::as_str).collect();

    group.bench_function("join_horizontal/10", |b| {
        b.iter(|| black_box(join_horizontal(Position::Top, &item_refs)));
    });

    group.bench_function("join_vertical/10", |b| {
        b.iter(|| black_box(join_vertical(Position::Left, &item_refs)));
    });

    group.bench_function("place", |b| {
        b.iter(|| {
            black_box(place(
                80,
                24,
                Position::Center,
                Position::Center,
                SAMPLE_LINE,
            ))
        });
    });

    group.finish();
}

fn bench_borders(c: &mut Criterion) {
    let mut group = c.benchmark_group("lipgloss/borders");

    let content = format!("{SAMPLE_LINE}\n{SAMPLE_PARAGRAPH}\n{SAMPLE_LINE}");

    let none = Style::new();
    let normal = Style::new().border(Border::normal());
    let rounded = Style::new().border(Border::rounded());
    let double = Style::new().border(Border::double());

    group.bench_function("border/none", |b| {
        b.iter(|| black_box(none.render(content.as_str())));
    });

    group.bench_function("border/normal", |b| {
        b.iter(|| black_box(normal.render(content.as_str())));
    });

    group.bench_function("border/rounded", |b| {
        b.iter(|| black_box(rounded.render(content.as_str())));
    });

    group.bench_function("border/double", |b| {
        b.iter(|| black_box(double.render(content.as_str())));
    });

    group.finish();
}

criterion_group!(
    lipgloss_benches,
    bench_style_creation,
    bench_colors,
    bench_rendering,
    bench_layout,
    bench_borders
);
criterion_main!(lipgloss_benches);
