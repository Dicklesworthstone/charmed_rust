//! Benchmark Validation Tests
//!
//! These tests verify that the operations being benchmarked produce
//! correct results. This catches cases where a benchmark might be
//! measuring a broken or incorrect implementation.
//!
//! Each test validates that:
//! 1. The operation completes without error
//! 2. The output is correct/valid
//! 3. Edge cases are handled properly

#![cfg(test)]

use lipgloss::{Border, Color, Position, RgbColor, Style, join_horizontal, join_vertical, place};

// ============================================================================
// LIPGLOSS BENCHMARK VALIDATION
// ============================================================================

mod lipgloss_validation {
    use super::*;

    #[test]
    fn validate_style_creation() {
        // Validates: bench_style_creation/Style::new
        let style = Style::new();
        // Style should be usable immediately
        let rendered = style.render("test");
        assert!(!rendered.is_empty());
        assert!(rendered.contains("test"));
    }

    #[test]
    fn validate_style_with_all_props() {
        // Validates: bench_style_creation/Style::new_with_all_props
        let style = Style::new()
            .foreground_color(RgbColor::new(255, 0, 0))
            .background_color(RgbColor::new(0, 0, 255))
            .bold()
            .italic()
            .underline()
            .padding((1u16, 2u16, 1u16, 2u16))
            .margin((1u16, 1u16, 1u16, 1u16))
            .border(Border::rounded());

        let rendered = style.render("styled text");
        // Should contain the text
        assert!(rendered.contains("styled text"));
        // Should have ANSI codes for styling
        assert!(rendered.contains('\x1b'));
    }

    #[test]
    fn validate_color_creation() {
        // Validates: bench_colors/AnsiColor::from
        let _ansi = lipgloss::AnsiColor::from(196u8);

        // Validates: bench_colors/RgbColor::new
        let rgb = RgbColor::new(255, 128, 64);
        assert_eq!(rgb.r, 255);
        assert_eq!(rgb.g, 128);
        assert_eq!(rgb.b, 64);

        // Validates: bench_colors/Color::hex_parse
        let color = Color::from("#FF8040");
        let rgb_tuple = color.as_rgb();
        assert!(rgb_tuple.is_some());
        let (r, g, b) = rgb_tuple.unwrap();
        assert_eq!(r, 255);
        assert_eq!(g, 128);
        assert_eq!(b, 64);

        // Validates: bench_colors/Color::ansi_parse
        let color = Color::from("196");
        let ansi = color.as_ansi();
        assert!(ansi.is_some());
    }

    #[test]
    fn validate_rendering_simple() {
        // Validates: bench_rendering/render/short/simple
        let simple_style = Style::new().foreground("#ff0000");
        let rendered = simple_style.render("The quick brown fox");
        assert!(rendered.contains("The quick brown fox"));
    }

    #[test]
    fn validate_rendering_complex() {
        // Validates: bench_rendering/render/short/complex
        let complex_style = Style::new()
            .foreground("#ff0000")
            .background("#0000ff")
            .bold()
            .padding((1u16, 2u16))
            .border(Border::rounded());

        let rendered = complex_style.render("Content");
        assert!(rendered.contains("Content"));
        // Should have border characters
        assert!(rendered.contains('╭') || rendered.contains('│'));
    }

    #[test]
    fn validate_layout_join_horizontal() {
        // Validates: bench_layout/join_horizontal/10
        let items: Vec<String> = (0..3).map(|i| format!("Item {i}")).collect();
        let item_refs: Vec<&str> = items.iter().map(String::as_str).collect();

        let result = join_horizontal(Position::Top, &item_refs);
        assert!(result.contains("Item 0"));
        assert!(result.contains("Item 1"));
        assert!(result.contains("Item 2"));
    }

    #[test]
    fn validate_layout_join_vertical() {
        // Validates: bench_layout/join_vertical/10
        let items: Vec<String> = (0..3).map(|i| format!("Item {i}")).collect();
        let item_refs: Vec<&str> = items.iter().map(String::as_str).collect();

        let result = join_vertical(Position::Left, &item_refs);
        assert!(result.contains("Item 0"));
        assert!(result.contains("Item 1"));
        assert!(result.contains("Item 2"));
        // Vertical join should have newlines
        assert!(result.contains('\n'));
    }

    #[test]
    fn validate_layout_place() {
        // Validates: bench_layout/place
        let content = "Centered";
        let result = place(80, 24, Position::Center, Position::Center, content);
        assert!(result.contains(content));
        // Should have some padding/spacing
        assert!(result.len() > content.len());
    }

    #[test]
    fn validate_borders() {
        // Validates: bench_borders/border/*
        let content = "Content\nMultiline\nText";

        let none = Style::new();
        let none_rendered = none.render(content);
        assert!(none_rendered.contains("Content"));

        let normal = Style::new().border(Border::normal());
        let normal_rendered = normal.render(content);
        assert!(normal_rendered.contains("Content"));
        // Should have border characters
        assert!(
            normal_rendered.contains('─')
                || normal_rendered.contains('│')
                || normal_rendered.contains('+')
        );

        let rounded = Style::new().border(Border::rounded());
        let rounded_rendered = rounded.render(content);
        assert!(rounded_rendered.contains("Content"));
        // Rounded borders use curved characters
        assert!(
            rounded_rendered.contains('╭')
                || rounded_rendered.contains('╮')
                || rounded_rendered.contains('─')
        );

        let double = Style::new().border(Border::double());
        let double_rendered = double.render(content);
        assert!(double_rendered.contains("Content"));
        // Double borders use double-line characters
        assert!(double_rendered.contains('═') || double_rendered.contains('║'));
    }
}

// ============================================================================
// BUBBLETEA BENCHMARK VALIDATION
// ============================================================================

mod bubbletea_validation {
    use bubbletea::{Cmd, Message, Model, batch, parse_sequence, sequence};

    #[derive(Clone, Debug)]
    enum TestMsg {
        Increment,
        Decrement,
    }

    #[derive(Clone, Debug)]
    struct Counter {
        count: i64,
    }

    impl Model for Counter {
        fn init(&self) -> Option<Cmd> {
            None
        }

        fn update(&mut self, msg: Message) -> Option<Cmd> {
            if let Some(msg) = msg.downcast::<TestMsg>() {
                match msg {
                    TestMsg::Increment => self.count += 1,
                    TestMsg::Decrement => self.count -= 1,
                }
            }
            None
        }

        fn view(&self) -> String {
            format!("Count: {}", self.count)
        }
    }

    #[test]
    fn validate_message_dispatch() {
        // Validates: bench_message_dispatch/single_message
        let mut model = Counter { count: 0 };
        model.update(Message::new(TestMsg::Increment));
        assert_eq!(model.count, 1);

        model.update(Message::new(TestMsg::Decrement));
        assert_eq!(model.count, 0);
    }

    #[test]
    fn validate_view_rendering() {
        // Validates: bench_view_rendering/simple_view
        let model = Counter { count: 42 };
        let view = model.view();
        assert_eq!(view, "Count: 42");
    }

    #[test]
    fn validate_key_parsing() {
        // Validates: bench_key_parsing/parse_sequence/*
        let sequences: &[(&str, &[u8])] = &[
            ("arrow_up", b"\x1b[A"),
            ("arrow_down", b"\x1b[B"),
            ("arrow_right", b"\x1b[C"),
            ("arrow_left", b"\x1b[D"),
        ];

        for (name, seq) in sequences {
            let result = parse_sequence(seq);
            // Should return a valid parse result
            assert!(result.is_some() || seq.is_empty(), "Failed to parse {name}");
        }
    }

    #[test]
    fn validate_commands() {
        // Validates: bench_commands/Cmd::none
        let none = Cmd::none();
        // none() should be a no-op command
        drop(none);

        // Validates: bench_commands/Cmd::message
        let cmd = Cmd::new(|| Message::new(TestMsg::Increment));
        let msg = cmd.execute();
        assert!(msg.is_some());

        // Validates: bench_commands/Cmd::batch_10
        let cmds: Vec<Option<Cmd>> = (0..10)
            .map(|_| Some(Cmd::new(|| Message::new(TestMsg::Increment))))
            .collect();
        let batched = batch(cmds);
        assert!(batched.is_some());

        // Validates: bench_commands/Cmd::sequence_10
        let cmds: Vec<Option<Cmd>> = (0..10)
            .map(|_| Some(Cmd::new(|| Message::new(TestMsg::Increment))))
            .collect();
        let sequenced = sequence(cmds);
        assert!(sequenced.is_some());
    }

    #[test]
    fn validate_event_loop_cycle() {
        // Validates: bench_event_loop/frame_cycle
        let mut model = Counter { count: 0 };

        // Simulate one frame cycle
        model.update(Message::new(TestMsg::Increment));
        let view = model.view();

        assert_eq!(model.count, 1);
        assert!(view.contains("1"));
    }
}

// ============================================================================
// BUBBLES BENCHMARK VALIDATION
// ============================================================================

mod bubbles_validation {
    use bubbles::spinner::{SpinnerModel, spinners};

    #[test]
    fn validate_spinner_creation() {
        // Validates: bench_spinner_creation/*
        let spinner = SpinnerModel::new();
        let view = spinner.view();
        // Spinner view should produce output
        assert!(!view.is_empty());

        let spinner_dots = SpinnerModel::with_spinner(spinners::dot());
        let view = spinner_dots.view();
        assert!(!view.is_empty());
    }

    #[test]
    fn validate_spinner_view() {
        // Validates: bench_spinner_tick/*
        let spinner = SpinnerModel::with_spinner(spinners::dot());

        // First view
        let view1 = spinner.view();

        // Views should be non-empty
        assert!(!view1.is_empty());
    }

    #[test]
    fn validate_spinner_types() {
        // Validates all spinner types work correctly
        let spinner_fns: &[fn() -> bubbles::spinner::Spinner] = &[
            spinners::line,
            spinners::dot,
            spinners::mini_dot,
            spinners::jump,
            spinners::pulse,
            spinners::points,
            spinners::globe,
            spinners::moon,
            spinners::monkey,
            spinners::meter,
            spinners::hamburger,
        ];

        for spinner_fn in spinner_fns {
            let spinner = SpinnerModel::with_spinner(spinner_fn());
            let view = spinner.view();
            assert!(!view.is_empty(), "Spinner produced empty view");
        }
    }
}

// ============================================================================
// GLAMOUR BENCHMARK VALIDATION
// ============================================================================

mod glamour_validation {
    use glamour::Renderer;

    #[test]
    fn validate_markdown_rendering() {
        // Validates: bench_render_markdown/*
        let renderer = Renderer::new();

        let simple = "# Hello\n\nWorld";
        let output = renderer.render(simple);
        assert!(output.contains("Hello") || output.contains("World"));
    }

    #[test]
    fn validate_renderer_creation() {
        // Validates: bench_renderer_creation/*
        let _default = Renderer::new();

        // All should create valid renderers
    }

    #[test]
    fn validate_complex_markdown() {
        // Validates rendering of complex markdown structures
        let renderer = Renderer::new();

        let complex = r#"
# Heading 1

Some paragraph text with **bold** and *italic*.

## Heading 2

- List item 1
- List item 2
- List item 3

```rust
fn main() {
    println!("Hello");
}
```

> A blockquote

---

[A link](https://example.com)
"#;

        let output = renderer.render(complex);

        // Should contain various elements
        assert!(output.contains("Heading"));
        assert!(output.contains("List item"));
    }
}

// ============================================================================
// BENCHMARK INFRASTRUCTURE VALIDATION
// ============================================================================

mod infrastructure_validation {
    use crate::harness::{BenchConfig, BenchContext, OutlierRemoval};
    use std::hint::black_box;
    use std::time::Duration;

    #[test]
    fn validate_bench_context_creation() {
        let ctx = BenchContext::new();
        assert!(ctx.results().is_empty());

        let ctx_with_config = BenchContext::with_config(BenchConfig {
            warmup_iterations: 5,
            measure_iterations: 50,
            adaptive_warmup: false,
            outlier_removal: OutlierRemoval::None,
            regression_threshold: 0.10,
        });
        assert!(ctx_with_config.results().is_empty());
    }

    #[test]
    fn validate_bench_execution() {
        let mut ctx = BenchContext::new().warmup(2).iterations(5);

        let result = ctx.bench("test_bench", || {
            black_box(1 + 1);
        });

        assert_eq!(result.name, "test_bench");
        assert_eq!(result.iterations, 5);
        assert!(result.min <= result.mean);
        assert!(result.mean <= result.max);
    }

    #[test]
    fn validate_stats_calculation() {
        let mut ctx = BenchContext::new().iterations(10);

        let result = ctx.bench("stats_test", || {
            std::thread::sleep(Duration::from_micros(100));
        });

        // All stats should be calculated
        assert!(result.min > Duration::ZERO);
        assert!(result.max >= result.min);
        assert!(result.mean >= result.min);
        assert!(result.mean <= result.max);
        assert!(result.p50 >= result.min);
        assert!(result.p95 >= result.p50);
        assert!(result.p99 >= result.p95);
        assert!(result.coefficient_of_variation >= 0.0);
    }

    #[test]
    fn validate_baseline_creation() {
        let mut ctx = BenchContext::new().iterations(5);

        ctx.bench("bench1", || {
            black_box(1);
        });
        ctx.bench("bench2", || {
            black_box(2);
        });

        let baseline = ctx.create_baseline();
        assert!(baseline.results.contains_key("bench1"));
        assert!(baseline.results.contains_key("bench2"));
    }
}
