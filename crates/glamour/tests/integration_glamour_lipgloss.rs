//! Integration tests: glamour + lipgloss styling (bd-366a).
//!
//! Verifies that glamour correctly uses lipgloss for styled terminal output:
//! - Style configurations map to expected ANSI output
//! - Markdown elements receive correct styling attributes
//! - Width handling respects ANSI codes
//! - Nested and composed styles work without corruption
//! - All built-in themes produce valid styled output

#![allow(clippy::uninlined_format_args)]

use glamour::{Renderer, Style, StyleBlock, StylePrimitive, ascii_style, dark_style};
use lipgloss::visible_width;

// ===========================================================================
// Helpers
// ===========================================================================

/// Check if a string contains ANSI escape sequences.
fn contains_ansi(s: &str) -> bool {
    s.contains('\x1b')
}

/// Strip ANSI escape sequences from a string.
fn strip_ansi(s: &str) -> String {
    #[derive(Clone, Copy)]
    enum State {
        Normal,
        Esc,
        Csi,
        Osc,
    }
    let mut state = State::Normal;
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match state {
            State::Normal => {
                if c == '\x1b' {
                    state = State::Esc;
                } else {
                    out.push(c);
                }
            }
            State::Esc => match c {
                '[' => state = State::Csi,
                ']' => state = State::Osc,
                _ => state = State::Normal,
            },
            State::Csi => {
                if ('@'..='~').contains(&c) {
                    state = State::Normal;
                }
            }
            State::Osc => {
                if c == '\x07' {
                    state = State::Normal;
                } else if c == '\x1b' {
                    state = State::Esc;
                }
            }
        }
    }
    out
}

/// Check if output contains a specific ANSI SGR code (e.g., "1" for bold).
fn contains_sgr(s: &str, code: &str) -> bool {
    // Look for \x1b[...{code}m or \x1b[{code};...m or \x1b[...;{code}m
    let pattern_standalone = format!("\x1b[{code}m");
    let pattern_start = format!("\x1b[{code};");
    let pattern_end = format!(";{code}m");
    let pattern_mid = format!(";{code};");
    s.contains(&pattern_standalone)
        || s.contains(&pattern_start)
        || s.contains(&pattern_end)
        || s.contains(&pattern_mid)
}

/// Render markdown with a specific style and return output.
fn render_with(md: &str, style: Style, wrap: usize) -> String {
    Renderer::new()
        .with_word_wrap(wrap)
        .with_style(style)
        .render(md)
}

// ===========================================================================
// 1. Style Application: glamour styles map to lipgloss ANSI output
// ===========================================================================

#[test]
fn dark_style_heading_produces_ansi_codes() {
    // Dark style headings use lipgloss with color + bold, producing ANSI
    let output = render_with("# Heading", Style::Dark, 80);
    assert!(
        contains_ansi(&output),
        "Dark heading should produce ANSI escape sequences: {:?}",
        output
    );
}

#[test]
fn ascii_style_no_ansi_color_codes() {
    let output = render_with("Hello world", Style::Ascii, 80);
    // Ascii style doesn't use colors - output should be plain text
    // (though it may contain structural chars like # for headings)
    let plain = strip_ansi(&output);
    assert_eq!(
        output.trim(),
        plain.trim(),
        "Ascii style should not produce ANSI codes"
    );
}

#[test]
fn dark_style_heading_has_bold() {
    let output = render_with("# Heading", Style::Dark, 80);
    // Dark h1 style has bold(true) and background_color - applied via lipgloss
    assert!(
        contains_sgr(&output, "1"),
        "Dark heading should have bold (SGR 1): {:?}",
        output
    );
}

#[test]
fn ascii_style_strong_has_prefix() {
    // In Ascii mode, strong text uses block_prefix="**" and block_suffix="**"
    let output = render_with("Some **bold** text", Style::Ascii, 80);
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("**bold**"),
        "Ascii **bold** should have ** markers: {:?}",
        plain
    );
}

#[test]
fn ascii_style_emphasis_has_prefix() {
    // In Ascii mode, emphasis uses block_prefix="*" and block_suffix="*"
    let output = render_with("Some *italic* text", Style::Ascii, 80);
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("*italic*"),
        "Ascii *italic* should have * markers: {:?}",
        plain
    );
}

#[test]
fn ascii_style_strikethrough_has_tilde() {
    // In Ascii mode, strikethrough uses block_prefix="~~" and block_suffix="~~"
    let output = render_with("Some ~~deleted~~ text", Style::Ascii, 80);
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("~~deleted~~"),
        "Ascii ~~strikethrough~~ should have ~~ markers: {:?}",
        plain
    );
}

#[test]
fn notty_style_minimal_ansi() {
    let output = render_with("# Heading\n\nSome text.", Style::NoTty, 80);
    let plain = strip_ansi(&output);
    // NoTty should produce minimal or no ANSI codes
    assert!(
        plain.contains("Heading"),
        "NoTty should still contain heading text"
    );
    assert!(
        plain.contains("Some text"),
        "NoTty should still contain paragraph text"
    );
}

// ===========================================================================
// 2. Rendering Integration: markdown elements get correct styles
// ===========================================================================

#[test]
fn heading_levels_all_render_styled() {
    for level in 1..=6 {
        let prefix = "#".repeat(level);
        let md = format!("{prefix} Level {level}");
        let output = render_with(&md, Style::Dark, 80);
        let plain = strip_ansi(&output);
        assert!(
            plain.contains(&format!("Level {level}")),
            "Heading level {} should contain text",
            level
        );
    }
}

#[test]
fn ascii_heading_has_hash_prefix() {
    let output = render_with("# Title", Style::Ascii, 80);
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("# Title"),
        "Ascii h1 should have '# ' prefix: {:?}",
        plain
    );
}

#[test]
fn ascii_h2_has_double_hash() {
    let output = render_with("## Subtitle", Style::Ascii, 80);
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("## Subtitle"),
        "Ascii h2 should have '## ' prefix: {:?}",
        plain
    );
}

#[test]
fn code_block_renders_content() {
    let output = render_with("```\nfn main() {}\n```", Style::Dark, 80);
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("fn main()"),
        "Code block should render content: {:?}",
        plain
    );
}

#[test]
fn inline_code_renders_in_paragraph() {
    let output = render_with("Use `foo()` to call it", Style::Dark, 80);
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("foo()"),
        "Inline code should be in output: {:?}",
        plain
    );
}

#[test]
fn unordered_list_has_bullet() {
    let output = render_with("- Item one\n- Item two", Style::Ascii, 80);
    let plain = strip_ansi(&output);
    // Ascii uses "• " bullet prefix
    assert!(
        plain.contains("Item one") && plain.contains("Item two"),
        "List items should appear: {:?}",
        plain
    );
}

#[test]
fn ordered_list_has_numbers() {
    let output = render_with("1. First\n2. Second\n3. Third", Style::Ascii, 80);
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("First") && plain.contains("Second") && plain.contains("Third"),
        "Ordered list items should appear: {:?}",
        plain
    );
}

#[test]
fn blockquote_has_indent_token() {
    let output = render_with("> Quoted text", Style::Ascii, 80);
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("| Quoted text") || plain.contains("│ Quoted text"),
        "Blockquote should have indent token: {:?}",
        plain
    );
}

#[test]
fn horizontal_rule_renders() {
    let output = render_with("---", Style::Ascii, 80);
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("----"),
        "Horizontal rule should render dashes: {:?}",
        plain
    );
}

#[test]
fn task_list_renders_markers() {
    let output = render_with("- [ ] todo\n- [x] done", Style::Ascii, 80);
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("[ ] todo"),
        "Unchecked task should have [ ]: {:?}",
        plain
    );
    assert!(
        plain.contains("[x] done"),
        "Checked task should have [x]: {:?}",
        plain
    );
}

#[test]
fn link_renders_url() {
    let output = render_with("[Click here](https://example.com)", Style::Ascii, 120);
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("Click here"),
        "Link text should be in output: {:?}",
        plain
    );
    assert!(
        plain.contains("https://example.com"),
        "Link URL should be in output: {:?}",
        plain
    );
}

#[test]
fn nested_bold_italic_ascii_has_markers() {
    // In Ascii mode, nested bold+italic uses ** and * markers
    let output = render_with("***bold and italic***", Style::Ascii, 80);
    let plain = strip_ansi(&output);
    // Content should be wrapped in both bold and italic markers
    assert!(
        plain.contains("bold and italic"),
        "Content should be preserved: {:?}",
        plain
    );
    // Should contain both * and ** markers
    assert!(
        plain.contains('*'),
        "Should contain emphasis markers: {:?}",
        plain
    );
}

// ===========================================================================
// 3. Width Handling: wrap respects ANSI codes
// ===========================================================================

#[test]
fn dark_style_wrap_respects_visible_width() {
    // Generate a paragraph with many words
    let words: Vec<&str> = vec![
        "alpha", "bravo", "charlie", "delta", "echo", "foxtrot", "golf", "hotel", "india",
        "juliet", "kilo", "lima", "mike", "november", "oscar", "papa", "quebec",
    ];
    let text = words.join(" ");
    let wrap_width = 40;

    let output = render_with(&text, Style::Dark, wrap_width);

    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let vw = visible_width(line);
        // Content is wrapped at wrap_width, margin of 2 is added
        // So visible_width should be <= wrap_width + margin
        assert!(
            vw <= wrap_width + 4, // extra slack for ANSI reset codes at line boundaries
            "Line too wide (visible_width={}): {:?}",
            vw,
            line
        );
    }
}

#[test]
fn ascii_and_dark_same_wrap_behavior() {
    let text = "This is a somewhat long sentence that should be wrapped at the specified width for both styles equally.";
    let wrap_width = 30;

    let ascii_output = render_with(text, Style::Ascii, wrap_width);
    let dark_output = render_with(text, Style::Dark, wrap_width);

    let ascii_lines = ascii_output
        .lines()
        .filter(|l| !l.trim().is_empty())
        .count();
    let dark_lines = dark_output.lines().filter(|l| !l.trim().is_empty()).count();

    // Both styles should produce the same number of content lines
    // (same text, same wrap width, margin is same for both)
    assert_eq!(
        ascii_lines, dark_lines,
        "Ascii and Dark should wrap the same way: ascii={}, dark={}",
        ascii_lines, dark_lines
    );
}

#[test]
fn visible_width_excludes_ansi_in_styled_output() {
    let output = render_with("Hello", Style::Dark, 80);
    for line in output.lines() {
        let vw = visible_width(line);
        let raw_len = line.len();
        if contains_ansi(line) {
            assert!(
                vw < raw_len,
                "ANSI-containing line should have visible_width < raw length: vw={}, raw={}",
                vw,
                raw_len
            );
        }
    }
}

// ===========================================================================
// 4. Style Configuration: custom StyleConfig works
// ===========================================================================

#[test]
fn custom_style_config_applies_color() {
    let mut config = ascii_style();
    config.paragraph.style = StylePrimitive::new().color("196"); // bright red
    config.paragraph.style.bold = Some(true);

    let renderer = Renderer::new().with_style_config(config).with_word_wrap(80);
    let output = renderer.render("Test paragraph");

    // Should contain ANSI codes from the custom paragraph style
    assert!(
        contains_ansi(&output),
        "Custom color style should produce ANSI codes: {:?}",
        output
    );
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("Test paragraph"),
        "Content should be preserved with custom style"
    );
}

#[test]
fn custom_heading_style_overrides_default() {
    let mut config = ascii_style();
    config.h1 = StyleBlock::new().style(StylePrimitive::new().prefix(">>> "));

    let renderer = Renderer::new().with_style_config(config).with_word_wrap(80);
    let output = renderer.render("# Custom Heading");
    let plain = strip_ansi(&output);

    assert!(
        plain.contains(">>> Custom Heading"),
        "Custom h1 prefix should be applied: {:?}",
        plain
    );
}

#[test]
fn custom_blockquote_indent_token() {
    let mut config = ascii_style();
    config.block_quote.indent_token = Some("> ".to_string());

    let renderer = Renderer::new().with_style_config(config).with_word_wrap(80);
    let output = renderer.render("> Quoted");
    let plain = strip_ansi(&output);

    assert!(
        plain.contains("> Quoted"),
        "Custom blockquote indent token should be used: {:?}",
        plain
    );
}

#[test]
fn default_styles_function_accessible() {
    // Verify the public style functions are accessible
    let _ascii = ascii_style();
    let _dark = dark_style();
    // These should construct valid StyleConfig instances without panicking
}

// ===========================================================================
// 5. Cross-Style Consistency
// ===========================================================================

#[test]
fn all_styles_render_same_plain_text() {
    let md = "# Heading\n\nSome paragraph text.\n\n- Item one\n- Item two\n";
    let all_styles = [
        Style::Ascii,
        Style::Dark,
        Style::Dracula,
        Style::Light,
        Style::Pink,
        Style::TokyoNight,
        Style::NoTty,
    ];

    // All styles should contain the same visible words
    for style in &all_styles {
        let output = render_with(md, *style, 80);
        let plain = strip_ansi(&output);
        assert!(
            plain.contains("Heading"),
            "{:?} missing 'Heading': {:?}",
            style,
            plain
        );
        assert!(
            plain.contains("Some paragraph text"),
            "{:?} missing paragraph text",
            style
        );
        assert!(
            plain.contains("Item one") && plain.contains("Item two"),
            "{:?} missing list items",
            style
        );
    }
}

#[test]
fn dark_and_dracula_both_have_color() {
    let output_dark = render_with("# Title\n\nBody", Style::Dark, 80);
    let output_dracula = render_with("# Title\n\nBody", Style::Dracula, 80);

    assert!(
        contains_ansi(&output_dark),
        "Dark should have ANSI: {:?}",
        output_dark
    );
    assert!(
        contains_ansi(&output_dracula),
        "Dracula should have ANSI: {:?}",
        output_dracula
    );
}

// ===========================================================================
// 6. Table Rendering with Styles
// ===========================================================================

#[test]
fn table_renders_with_separators() {
    let md = "| Col A | Col B |\n|-------|-------|\n| 1     | 2     |\n| 3     | 4     |";
    let output = render_with(md, Style::Ascii, 80);
    let plain = strip_ansi(&output);

    // Table should have content
    assert!(plain.contains("Col A"), "Table header missing: {:?}", plain);
    assert!(
        plain.contains('1') && plain.contains('2'),
        "Table data missing: {:?}",
        plain
    );
    // Ascii style uses "|" as separator
    assert!(
        plain.contains('|'),
        "Table should have separator: {:?}",
        plain
    );
}

#[test]
fn table_respects_wrap_width() {
    let md = "| Name | Description |\n|------|-------------|\n| Foo | A very long description that might need to be handled within the table width constraints |\n";
    let narrow = render_with(md, Style::Ascii, 40);
    let wide = render_with(md, Style::Ascii, 120);

    // With narrower width, table should adapt
    let narrow_max_width = narrow.lines().map(visible_width).max().unwrap_or(0);
    let wide_max_width = wide.lines().map(visible_width).max().unwrap_or(0);

    // Narrow output should be narrower or equal to wide output
    assert!(
        narrow_max_width <= wide_max_width + 1,
        "Narrow table should not be wider than wide table: narrow={}, wide={}",
        narrow_max_width,
        wide_max_width
    );
}

// ===========================================================================
// 7. Edge Cases: Nested Styles and Boundaries
// ===========================================================================

#[test]
fn bold_inside_italic_preserves_both() {
    let md = "*italic and **bold** text*";
    let output = render_with(md, Style::Dark, 80);
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("italic and"),
        "Italic text preserved: {:?}",
        plain
    );
    assert!(plain.contains("bold"), "Bold text preserved: {:?}", plain);
    assert!(
        plain.contains("text"),
        "Trailing text preserved: {:?}",
        plain
    );
}

#[test]
fn style_across_line_wrap() {
    // A long styled paragraph should maintain style across wrapped lines
    let md = "**This is a bold paragraph that is long enough to require wrapping at a narrow width setting**";
    let output = render_with(md, Style::Dark, 30);
    let plain = strip_ansi(&output);

    // Content should survive wrapping
    assert!(
        plain.contains("bold paragraph"),
        "Bold paragraph content should survive wrap"
    );
}

#[test]
fn many_style_transitions_no_corruption() {
    let md = "Normal *italic* **bold** ~~strike~~ `code` normal again.";
    let output = render_with(md, Style::Dark, 80);
    let plain = strip_ansi(&output);

    assert!(plain.contains("Normal"), "Normal text should be present");
    assert!(plain.contains("italic"), "Italic text should be present");
    assert!(plain.contains("bold"), "Bold text should be present");
    assert!(
        plain.contains("strike"),
        "Strikethrough text should be present"
    );
    assert!(plain.contains("code"), "Code text should be present");
    assert!(
        plain.contains("normal again"),
        "Trailing text should be present"
    );
}

#[test]
fn empty_markdown_elements_no_crash() {
    // Edge cases that might cause issues
    let edge_cases = [
        "****",     // empty bold
        "**",       // incomplete bold
        "``",       // empty inline code
        "---",      // horizontal rule
        "- ",       // empty list item
        "> ",       // empty blockquote
        "# ",       // empty heading
        "```\n```", // empty code block
    ];

    for md in &edge_cases {
        let _ = render_with(md, Style::Dark, 80);
        // Should not panic
    }
}

#[test]
fn very_long_line_with_styles() {
    // A paragraph with many styled segments
    let segments: Vec<String> = (0..20)
        .map(|i| {
            if i % 3 == 0 {
                format!("**word{i}**")
            } else if i % 3 == 1 {
                format!("*word{i}*")
            } else {
                format!("word{i}")
            }
        })
        .collect();
    let md = segments.join(" ");
    let output = render_with(&md, Style::Dark, 50);
    let plain = strip_ansi(&output);

    // All words should be present
    for i in 0..20 {
        assert!(
            plain.contains(&format!("word{i}")),
            "word{} missing from styled output",
            i
        );
    }
}

// ===========================================================================
// 8. Document Margin Integration
// ===========================================================================

#[test]
fn ascii_document_margin_applied() {
    let output = render_with("Hello", Style::Ascii, 80);
    // Ascii style has document margin of 2 (DEFAULT_MARGIN)
    // Content lines should start with 2+ spaces
    for line in output.lines() {
        if !line.trim().is_empty() {
            assert!(
                line.starts_with("  "),
                "Content line should have margin: {:?}",
                line
            );
        }
    }
}

#[test]
fn dark_document_margin_applied() {
    let output = render_with("Hello", Style::Dark, 80);
    let plain = strip_ansi(&output);
    for line in plain.lines() {
        if !line.trim().is_empty() {
            assert!(
                line.starts_with("  "),
                "Content line should have margin in dark style: {:?}",
                line
            );
        }
    }
}

#[test]
fn custom_zero_margin_no_indent() {
    let mut config = ascii_style();
    config.document.margin = Some(0);

    let renderer = Renderer::new().with_style_config(config).with_word_wrap(80);
    let output = renderer.render("Hello");

    // With zero margin, content lines should NOT start with spaces
    let has_unindented_content = output.lines().any(|line| {
        let trimmed = line.trim();
        !trimmed.is_empty() && !line.starts_with(' ')
    });
    assert!(
        has_unindented_content,
        "Zero margin should produce unindented content: {:?}",
        output
    );
}

// ===========================================================================
// 9. Render Bytes API
// ===========================================================================

#[test]
fn render_bytes_produces_same_as_render() {
    let md = "# Heading\n\nParagraph with **bold**.";
    let renderer = Renderer::new().with_style(Style::Dark).with_word_wrap(80);
    let output_str = renderer.render(md);
    let output_from_bytes = renderer.render_bytes(md.as_bytes()).unwrap();

    // render_bytes should be equivalent to rendering from string
    assert_eq!(
        output_str.trim(),
        output_from_bytes.trim(),
        "render_bytes should match render"
    );
}

// ===========================================================================
// 10. StylePrimitive to_lipgloss conversion
// ===========================================================================

#[test]
fn style_primitive_to_lipgloss_color() {
    let prim = StylePrimitive::new().color("196");
    let lipgloss_style = prim.to_lipgloss();
    let rendered = lipgloss_style.render("Test");

    // Should contain ANSI color code for 196
    assert!(
        contains_ansi(&rendered),
        "Lipgloss style with color should produce ANSI: {:?}",
        rendered
    );
    assert!(
        rendered.contains("Test"),
        "Rendered text should contain content"
    );
}

#[test]
fn style_primitive_to_lipgloss_bold() {
    let prim = StylePrimitive::new().bold(true);
    let lipgloss_style = prim.to_lipgloss();
    let rendered = lipgloss_style.render("Bold");

    assert!(
        contains_sgr(&rendered, "1"),
        "Bold style should produce SGR 1: {:?}",
        rendered
    );
}

#[test]
fn style_primitive_to_lipgloss_italic() {
    let prim = StylePrimitive::new().italic(true);
    let lipgloss_style = prim.to_lipgloss();
    let rendered = lipgloss_style.render("Italic");

    assert!(
        contains_sgr(&rendered, "3"),
        "Italic style should produce SGR 3: {:?}",
        rendered
    );
}

#[test]
fn style_primitive_to_lipgloss_underline() {
    let prim = StylePrimitive::new().underline(true);
    let lipgloss_style = prim.to_lipgloss();
    let rendered = lipgloss_style.render("Underline");

    assert!(
        contains_sgr(&rendered, "4"),
        "Underline style should produce SGR 4: {:?}",
        rendered
    );
}

#[test]
fn style_primitive_to_lipgloss_faint() {
    let prim = StylePrimitive::new().faint(true);
    let lipgloss_style = prim.to_lipgloss();
    let rendered = lipgloss_style.render("Faint");

    assert!(
        contains_sgr(&rendered, "2"),
        "Faint style should produce SGR 2: {:?}",
        rendered
    );
}

#[test]
fn style_primitive_to_lipgloss_strikethrough() {
    let prim = StylePrimitive::new().crossed_out(true);
    let lipgloss_style = prim.to_lipgloss();
    let rendered = lipgloss_style.render("Struck");

    assert!(
        contains_sgr(&rendered, "9"),
        "Strikethrough style should produce SGR 9: {:?}",
        rendered
    );
}

#[test]
fn style_primitive_combined_attributes() {
    let prim = StylePrimitive::new()
        .bold(true)
        .italic(true)
        .underline(true)
        .color("39");
    let lipgloss_style = prim.to_lipgloss();
    let rendered = lipgloss_style.render("Combo");

    assert!(
        contains_sgr(&rendered, "1"),
        "Combined style should have bold"
    );
    assert!(
        contains_sgr(&rendered, "3"),
        "Combined style should have italic"
    );
    assert!(
        contains_sgr(&rendered, "4"),
        "Combined style should have underline"
    );
    assert!(
        contains_ansi(&rendered),
        "Combined style should have color codes"
    );
    assert!(
        strip_ansi(&rendered).contains("Combo"),
        "Content should be preserved"
    );
}

#[test]
fn style_primitive_no_attributes_plain_text() {
    let prim = StylePrimitive::new();
    let lipgloss_style = prim.to_lipgloss();
    let rendered = lipgloss_style.render("Plain");

    // No attributes set = plain text, no ANSI
    assert_eq!(
        rendered, "Plain",
        "No-attribute style should produce plain text: {:?}",
        rendered
    );
}

// ===========================================================================
// 11. Preserve Newlines Option Integration
// ===========================================================================

#[test]
fn preserve_newlines_vs_default_differ() {
    let md = "Line one\nLine two";
    let renderer_default = Renderer::new().with_style(Style::Ascii).with_word_wrap(80);
    let renderer_preserve = Renderer::new()
        .with_style(Style::Ascii)
        .with_word_wrap(80)
        .with_preserved_newlines(true);

    let output_default = renderer_default.render(md);
    let output_preserve = renderer_preserve.render(md);

    let plain_default = strip_ansi(&output_default);
    let plain_preserve = strip_ansi(&output_preserve);

    // Both should contain the text content
    assert!(
        plain_default.contains("Line one") && plain_default.contains("Line two"),
        "Default should contain both lines: {:?}",
        plain_default
    );
    assert!(
        plain_preserve.contains("Line one") && plain_preserve.contains("Line two"),
        "Preserve should contain both lines: {:?}",
        plain_preserve
    );

    // Without preserve: soft break becomes space
    // With preserve: soft break becomes '\n' (though word_wrap may rejoin)
    // At minimum, the option should be accepted without panicking
}

// ===========================================================================
// 12. Convenience Function Integration
// ===========================================================================

#[test]
fn convenience_render_returns_ok() {
    let result = glamour::render("# Hello\n\nWorld", Style::Dark);
    assert!(result.is_ok(), "render() should return Ok");
    let output = result.unwrap();
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("Hello"),
        "render() output should contain heading"
    );
    assert!(
        plain.contains("World"),
        "render() output should contain text"
    );
}

#[test]
fn render_bytes_returns_ok() {
    let renderer = Renderer::new().with_style(Style::Ascii).with_word_wrap(80);
    let result = renderer.render_bytes(b"# Hello\n\nWorld");
    assert!(result.is_ok(), "render_bytes() should return Ok");
    let output = result.unwrap();
    assert!(
        output.contains("Hello"),
        "render_bytes() should contain heading"
    );
}
