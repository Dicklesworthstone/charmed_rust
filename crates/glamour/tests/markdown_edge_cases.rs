#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]

//! Tests for glamour markdown rendering edge cases, API surface,
//! and markdown feature coverage gaps.

use glamour::{available_styles, render, Renderer, Style};

// =============================================================================
// API surface tests
// =============================================================================

#[test]
fn available_styles_contains_all_variants() {
    let styles = available_styles();
    assert!(styles.contains_key("ascii"));
    assert!(styles.contains_key("dark"));
    assert!(styles.contains_key("dracula"));
    assert!(styles.contains_key("light"));
    assert!(styles.contains_key("pink"));
    assert!(styles.contains_key("notty"));
    assert!(styles.contains_key("auto"));
    assert_eq!(styles.len(), 7);
}

#[test]
fn render_convenience_function() {
    let result = render("# Hello", Style::Dark);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Hello"));
}

#[test]
fn renderer_builder_chain() {
    let output = Renderer::new()
        .with_style(Style::Ascii)
        .with_word_wrap(40)
        .render("# Test\n\nParagraph.");
    assert!(output.contains("Test"));
    assert!(output.contains("Paragraph"));
}

#[test]
fn render_bytes_valid_utf8() {
    let renderer = Renderer::new().with_style(Style::Ascii);
    let result = renderer.render_bytes(b"# Hello");
    assert!(result.is_ok());
    assert!(result.unwrap().contains("Hello"));
}

#[test]
fn render_bytes_invalid_utf8() {
    let renderer = Renderer::new().with_style(Style::Ascii);
    let result = renderer.render_bytes(&[0xFF, 0xFE]);
    assert!(result.is_err());
}

// =============================================================================
// Markdown element coverage
// =============================================================================

#[test]
fn render_headings_all_levels() {
    let md = "# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6\n";
    let output = Renderer::new().with_style(Style::Ascii).render(md);
    assert!(output.contains("H1"));
    assert!(output.contains("H2"));
    assert!(output.contains("H3"));
    assert!(output.contains("H4"));
    assert!(output.contains("H5"));
    assert!(output.contains("H6"));
}

#[test]
fn render_bold_and_italic() {
    let md = "This is **bold** and *italic* and ***both***.";
    let output = Renderer::new().with_style(Style::Dark).render(md);
    assert!(output.contains("bold"));
    assert!(output.contains("italic"));
    assert!(output.contains("both"));
}

#[test]
fn render_strikethrough() {
    let md = "This is ~~deleted~~ text.";
    let output = Renderer::new().with_style(Style::Dark).render(md);
    assert!(output.contains("deleted"));
}

#[test]
fn render_unordered_list() {
    let md = "- Item 1\n- Item 2\n- Item 3\n";
    let output = Renderer::new().with_style(Style::Dark).render(md);
    assert!(output.contains("Item 1"));
    assert!(output.contains("Item 2"));
    assert!(output.contains("Item 3"));
}

#[test]
fn render_ordered_list() {
    let md = "1. First\n2. Second\n3. Third\n";
    let output = Renderer::new().with_style(Style::Dark).render(md);
    assert!(output.contains("First"));
    assert!(output.contains("Second"));
    assert!(output.contains("Third"));
}

#[test]
fn render_nested_list() {
    let md = "- Parent\n  - Child\n    - Grandchild\n";
    let output = Renderer::new().with_style(Style::Dark).render(md);
    assert!(output.contains("Parent"));
    assert!(output.contains("Child"));
    assert!(output.contains("Grandchild"));
}

#[test]
fn render_blockquote() {
    let md = "> This is a quote\n> with two lines\n";
    let output = Renderer::new().with_style(Style::Dark).render(md);
    assert!(output.contains("This is a quote"));
}

#[test]
fn render_nested_blockquote() {
    let md = "> Outer\n>> Inner\n";
    let output = Renderer::new().with_style(Style::Dark).render(md);
    assert!(output.contains("Outer"));
    assert!(output.contains("Inner"));
}

#[test]
fn render_code_inline() {
    let md = "Use `println!` to print.";
    let output = Renderer::new().with_style(Style::Dark).render(md);
    assert!(output.contains("println!"));
}

#[test]
fn render_code_block() {
    let md = "```rust\nfn main() {}\n```\n";
    let output = Renderer::new().with_style(Style::Dark).render(md);
    // Syntax highlighting may insert ANSI escapes between tokens
    assert!(output.contains("main"));
}

#[test]
fn render_code_block_no_language() {
    let md = "```\nplain code\n```\n";
    let output = Renderer::new().with_style(Style::Dark).render(md);
    assert!(output.contains("plain code"));
}

#[test]
fn render_horizontal_rule() {
    let md = "Above\n\n---\n\nBelow\n";
    let output = Renderer::new().with_style(Style::Dark).render(md);
    assert!(output.contains("Above"));
    assert!(output.contains("Below"));
}

#[test]
fn render_link() {
    let md = "[Click here](https://example.com)";
    let output = Renderer::new().with_style(Style::Dark).render(md);
    assert!(output.contains("Click here"));
}

#[test]
fn render_image() {
    let md = "![Alt text](image.png)";
    let output = Renderer::new().with_style(Style::Dark).render(md);
    // Image should be represented somehow (alt text or format string)
    assert!(output.contains("Alt text") || output.contains("image.png"));
}

#[test]
fn render_task_list() {
    let md = "- [ ] Unchecked\n- [x] Checked\n";
    let output = Renderer::new().with_style(Style::Dark).render(md);
    assert!(output.contains("Unchecked"));
    assert!(output.contains("Checked"));
}

#[test]
fn render_table() {
    let md = "| Name | Value |\n|------|-------|\n| A    | 1     |\n| B    | 2     |\n";
    let output = Renderer::new().with_style(Style::Dark).render(md);
    assert!(output.contains('A'));
    assert!(output.contains('B'));
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn render_empty_input() {
    let output = Renderer::new().with_style(Style::Dark).render("");
    // Should not panic, may produce whitespace
    let _ = output;
}

#[test]
fn render_only_whitespace() {
    let output = Renderer::new().with_style(Style::Dark).render("   \n\n  \n");
    let _ = output;
}

#[test]
fn render_very_long_line() {
    let long_line = "x".repeat(10_000);
    let output = Renderer::new()
        .with_style(Style::Dark)
        .with_word_wrap(80)
        .render(&long_line);
    assert!(output.contains('x'));
}

#[test]
fn render_unicode_content() {
    let md = "# 日本語テスト\n\nこんにちは世界。**太字**と*斜体*。\n";
    let output = Renderer::new().with_style(Style::Dark).render(md);
    assert!(output.contains("日本語"));
    assert!(output.contains("太字"));
}

#[test]
fn render_emoji_content() {
    let md = "# 🎉 Party\n\nHave some 🍕 pizza!\n";
    let output = Renderer::new().with_style(Style::Dark).render(md);
    assert!(output.contains("Party"));
    assert!(output.contains("pizza"));
}

#[test]
fn render_deeply_nested_lists() {
    let md = "- L1\n  - L2\n    - L3\n      - L4\n        - L5\n";
    let output = Renderer::new().with_style(Style::Dark).render(md);
    assert!(output.contains("L1"));
    assert!(output.contains("L5"));
}

#[test]
fn render_multiple_paragraphs() {
    let md = "Para 1.\n\nPara 2.\n\nPara 3.\n";
    let output = Renderer::new().with_style(Style::Dark).render(md);
    assert!(output.contains("Para 1"));
    assert!(output.contains("Para 2"));
    assert!(output.contains("Para 3"));
}

// =============================================================================
// Style consistency
// =============================================================================

#[test]
fn all_styles_render_without_panic() {
    let md = "# Heading\n\n**Bold** *italic* `code`\n\n- item\n\n> quote\n\n```\ncode\n```\n";
    let styles = [
        Style::Ascii,
        Style::Dark,
        Style::Dracula,
        Style::Light,
        Style::Pink,
        Style::TokyoNight,
        Style::NoTty,
        Style::Auto,
    ];
    for style in &styles {
        let output = Renderer::new().with_style(*style).render(md);
        assert!(
            output.contains("Heading"),
            "Style {style:?} should preserve content"
        );
        assert!(output.contains("Bold"));
        assert!(output.contains("code"));
    }
}

#[test]
fn ascii_style_no_ansi_escapes() {
    let md = "# Hello\n\n**Bold** text.\n";
    let output = Renderer::new().with_style(Style::Ascii).render(md);
    // Ascii style should not contain ANSI escape sequences
    assert!(
        !output.contains('\x1b'),
        "Ascii style should not produce ANSI escapes"
    );
}

#[test]
fn notty_style_no_ansi_escapes() {
    let md = "# Hello\n\n**Bold** text.\n";
    let output = Renderer::new().with_style(Style::NoTty).render(md);
    assert!(
        !output.contains('\x1b'),
        "NoTty style should not produce ANSI escapes"
    );
}

#[test]
fn word_wrap_respects_width() {
    let md = "This is a sentence that should wrap at a narrow width.";
    let width = 20;
    let output = Renderer::new()
        .with_style(Style::Ascii)
        .with_word_wrap(width)
        .render(md);
    // Each line should not exceed the wrap width (plus some margin)
    for line in output.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            assert!(
                trimmed.len() <= width + 10,
                "Line too long ({} chars): '{trimmed}'",
                trimmed.len()
            );
        }
    }
}

#[test]
fn preserved_newlines_option() {
    let md = "Line 1\nLine 2\nLine 3\n";
    let with = Renderer::new()
        .with_style(Style::Ascii)
        .with_preserved_newlines(true)
        .render(md);
    let without = Renderer::new()
        .with_style(Style::Ascii)
        .with_preserved_newlines(false)
        .render(md);
    // Both should contain the content
    assert!(with.contains("Line 1"));
    assert!(without.contains("Line 1"));
}

// =============================================================================
// Determinism
// =============================================================================

#[test]
fn render_is_deterministic() {
    let md = "# Hello\n\n**Bold** and *italic*.\n\n- Item 1\n- Item 2\n";
    let r = Renderer::new().with_style(Style::Dark);
    let output1 = r.render(md);
    let output2 = r.render(md);
    assert_eq!(output1, output2, "Rendering should be deterministic");
}
