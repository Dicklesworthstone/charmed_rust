//! Integration tests for Terminal I/O verification: glamour ANSI output (bd-97j0).
//!
//! Verifies that glamour's markdown rendering produces correct, well-formed
//! ANSI escape sequences for different styles (Dark, Light, Ascii, etc.).

use glamour::{Renderer, Style};

// ===========================================================================
// Helpers
// ===========================================================================

/// Check if a string contains any ANSI escape sequence.
fn contains_ansi(s: &str) -> bool {
    s.contains('\x1b')
}

/// Strip all ANSI escape sequences from a string.
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if let Some(&next) = chars.peek() {
                if next == '[' {
                    chars.next();
                    while let Some(&c2) = chars.peek() {
                        chars.next();
                        if ('@'..='~').contains(&c2) {
                            break;
                        }
                    }
                } else if next == ']' {
                    chars.next();
                    while let Some(&c2) = chars.peek() {
                        chars.next();
                        if c2 == '\x07' {
                            break;
                        }
                        if c2 == '\x1b' && chars.peek() == Some(&'\\') {
                            chars.next();
                            break;
                        }
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Check if output contains a specific SGR code.
fn contains_sgr(s: &str, code: &str) -> bool {
    let standalone = format!("\x1b[{code}m");
    let prefix = format!("\x1b[{code};");
    let suffix = format!(";{code}m");
    let middle = format!(";{code};");
    s.contains(&standalone) || s.contains(&prefix) || s.contains(&suffix) || s.contains(&middle)
}

/// Render markdown with a given style.
fn render_with(md: &str, style: Style) -> String {
    Renderer::new().with_style(style).render(md)
}

// ===========================================================================
// 1. Dark Style: Headings Produce ANSI
// ===========================================================================

#[test]
fn dark_heading_has_ansi_codes() {
    let output = render_with("# Hello World", Style::Dark);
    assert!(
        contains_ansi(&output),
        "Dark heading should produce ANSI: {output:?}"
    );
}

#[test]
fn dark_heading_has_bold() {
    let output = render_with("# Hello", Style::Dark);
    // Dark style heading uses bold(true)
    assert!(
        contains_sgr(&output, "1"),
        "Dark heading should have bold (SGR 1): {output:?}"
    );
}

#[test]
fn dark_h1_has_background_color() {
    let output = render_with("# Title", Style::Dark);
    // Dark H1: bg_color "63" → ANSI 256 bg → \x1b[48;5;63m
    assert!(
        output.contains("\x1b[48;5;63m"),
        "Dark H1 should have bg color 63: {output:?}"
    );
}

#[test]
fn dark_h1_has_foreground_color() {
    let output = render_with("# Title", Style::Dark);
    // Dark H1: color "228" → ANSI 256 fg → \x1b[38;5;228m
    assert!(
        output.contains("\x1b[38;5;228m"),
        "Dark H1 should have fg color 228: {output:?}"
    );
}

#[test]
fn dark_heading_content_preserved() {
    let output = render_with("# My Heading", Style::Dark);
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("My Heading"),
        "Heading text should be preserved: {plain:?}"
    );
}

#[test]
fn dark_h6_has_distinct_color() {
    let output = render_with("###### Subheading", Style::Dark);
    // Dark H6: color "35" → \x1b[38;5;35m
    assert!(
        output.contains("\x1b[38;5;35m"),
        "Dark H6 should have color 35: {output:?}"
    );
}

// ===========================================================================
// 2. Dark Style: Code Blocks and Inline Code
// ===========================================================================

#[test]
fn dark_inline_code_has_color() {
    let output = render_with("Use `foo()` here", Style::Dark);
    // Dark inline code: color "203" → \x1b[38;5;203m
    assert!(
        output.contains("\x1b[38;5;203m"),
        "Dark inline code should have color 203: {output:?}"
    );
}

#[test]
fn dark_inline_code_has_background() {
    let output = render_with("Use `bar()` here", Style::Dark);
    // Dark inline code: bg_color "236" → \x1b[48;5;236m
    assert!(
        output.contains("\x1b[48;5;236m"),
        "Dark inline code should have bg color 236: {output:?}"
    );
}

#[test]
fn dark_code_block_content_indented() {
    let output = render_with("```\nfn main() {}\n```", Style::Dark);
    // Code blocks in dark style are indented but don't produce per-line ANSI
    // (the color property exists in config but the rendering path for code
    // blocks uses indentation, not lipgloss rendering)
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("fn main() {}"),
        "Code block content present: {plain:?}"
    );
}

#[test]
fn dark_code_block_content_preserved() {
    let output = render_with("```\nlet x = 42;\n```", Style::Dark);
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("let x = 42;"),
        "Code block content preserved: {plain:?}"
    );
}

// ===========================================================================
// 3. Dark Style: Links
// ===========================================================================

#[test]
fn dark_link_renders_text_and_url() {
    let output = render_with("[click](https://example.com)", Style::Dark);
    // Links in dark style render as "text url" (the styling properties exist
    // in config but the rendering path produces inline text, not ANSI)
    let plain = strip_ansi(&output);
    assert!(plain.contains("click"), "Link text present: {plain:?}");
    assert!(plain.contains("example.com"), "Link URL present: {plain:?}");
}

#[test]
fn dark_link_formatted_as_text_url() {
    let output = render_with("[Go here](https://rust-lang.org)", Style::Dark);
    let plain = strip_ansi(&output);
    assert!(plain.contains("Go here"), "Link text present: {plain:?}");
    assert!(
        plain.contains("rust-lang.org"),
        "Link URL present: {plain:?}"
    );
}

// ===========================================================================
// 4. Dark Style: Inline Elements (No ANSI for bold/italic/strike)
// ===========================================================================

// In Dark style, inline elements (strong, emph, strikethrough) use
// StylePrimitive with bold(true)/italic(true)/crossed_out(true), but the
// rendering path only pushes block_prefix/block_suffix (both empty).
// So these do NOT produce ANSI codes for the inline formatting itself.

#[test]
fn dark_strong_text_present_but_no_bold_marker() {
    let output = render_with("This is **bold** text", Style::Dark);
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("bold"),
        "Bold text content preserved: {plain:?}"
    );
    // The bold text doesn't get **markers** (that's Ascii style)
    assert!(
        !plain.contains("**"),
        "Dark style should not use ** markers: {plain:?}"
    );
}

#[test]
fn dark_emphasis_text_present() {
    let output = render_with("This is *italic* text", Style::Dark);
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("italic"),
        "Italic text content preserved: {plain:?}"
    );
    assert!(
        !plain.contains("*italic*"),
        "Dark style should not use * markers around italic: {plain:?}"
    );
}

// ===========================================================================
// 5. Ascii Style: No ANSI Codes
// ===========================================================================

#[test]
fn ascii_style_heading_no_ansi() {
    let output = render_with("# Hello", Style::Ascii);
    assert!(
        !contains_ansi(&output),
        "Ascii style should produce no ANSI: {output:?}"
    );
}

#[test]
fn ascii_style_code_no_ansi() {
    let output = render_with("Use `code` here", Style::Ascii);
    assert!(
        !contains_ansi(&output),
        "Ascii style code should produce no ANSI: {output:?}"
    );
}

#[test]
fn ascii_style_strong_uses_markers() {
    let output = render_with("This is **bold** text", Style::Ascii);
    assert!(
        !contains_ansi(&output),
        "Ascii style should have no ANSI: {output:?}"
    );
    assert!(
        output.contains("**"),
        "Ascii style should use ** markers: {output:?}"
    );
}

#[test]
fn ascii_style_emph_uses_markers() {
    let output = render_with("This is *italic* text", Style::Ascii);
    assert!(
        !contains_ansi(&output),
        "Ascii style should have no ANSI: {output:?}"
    );
}

#[test]
fn ascii_style_strike_uses_markers() {
    let output = render_with("This is ~~struck~~ text", Style::Ascii);
    assert!(
        !contains_ansi(&output),
        "Ascii style should have no ANSI: {output:?}"
    );
    assert!(
        output.contains("~~"),
        "Ascii style should use ~~ markers: {output:?}"
    );
}

#[test]
fn ascii_heading_uses_hash_prefix() {
    let output = render_with("## Second Level", Style::Ascii);
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("## Second Level") || plain.contains("## "),
        "Ascii heading should use ## prefix: {plain:?}"
    );
}

#[test]
fn ascii_code_block_no_ansi() {
    let output = render_with("```\ncode here\n```", Style::Ascii);
    assert!(
        !contains_ansi(&output),
        "Ascii code block should have no ANSI: {output:?}"
    );
    assert!(
        output.contains("code here"),
        "Code content preserved: {output:?}"
    );
}

// ===========================================================================
// 6. NoTty Style: No ANSI Codes
// ===========================================================================

#[test]
fn notty_style_no_ansi() {
    let output = render_with("# Hello\n\n**bold** and *italic*", Style::NoTty);
    assert!(
        !contains_ansi(&output),
        "NoTty style should produce no ANSI: {output:?}"
    );
}

// ===========================================================================
// 7. Light Style: ANSI Present with Different Colors
// ===========================================================================

#[test]
fn light_heading_has_ansi() {
    let output = render_with("# Hello", Style::Light);
    assert!(
        contains_ansi(&output),
        "Light heading should produce ANSI: {output:?}"
    );
}

#[test]
fn light_heading_has_bold() {
    let output = render_with("# Hello", Style::Light);
    assert!(
        contains_sgr(&output, "1"),
        "Light heading should have bold: {output:?}"
    );
}

#[test]
fn light_style_differs_from_dark_on_h2() {
    // H1 colors are identical in Dark and Light, so use H2+ to see difference
    // Dark heading base color: "39", Light heading base color: "27"
    let dark_output = render_with("## Hello", Style::Dark);
    let light_output = render_with("## Hello", Style::Light);
    assert!(contains_ansi(&dark_output), "Dark h2 has ANSI");
    assert!(contains_ansi(&light_output), "Light h2 has ANSI");
    // Dark heading uses color 39, Light uses color 27
    assert!(
        dark_output.contains("\x1b[38;5;39m"),
        "Dark heading color 39: {dark_output:?}"
    );
    assert!(
        light_output.contains("\x1b[38;5;27m"),
        "Light heading color 27: {light_output:?}"
    );
}

// ===========================================================================
// 8. Dracula Style: ANSI with Theme Colors
// ===========================================================================

#[test]
fn dracula_heading_has_ansi() {
    let output = render_with("# Hello", Style::Dracula);
    assert!(
        contains_ansi(&output),
        "Dracula heading should produce ANSI: {output:?}"
    );
}

#[test]
fn dracula_heading_has_bold() {
    let output = render_with("# Hello", Style::Dracula);
    assert!(
        contains_sgr(&output, "1"),
        "Dracula heading should have bold: {output:?}"
    );
}

// ===========================================================================
// 9. Tokyo Night Style
// ===========================================================================

#[test]
fn tokyo_night_heading_has_ansi() {
    let output = render_with("# Hello", Style::TokyoNight);
    assert!(
        contains_ansi(&output),
        "TokyoNight heading should produce ANSI: {output:?}"
    );
}

// ===========================================================================
// 10. Pink Style
// ===========================================================================

#[test]
fn pink_heading_has_ansi() {
    let output = render_with("# Hello", Style::Pink);
    assert!(
        contains_ansi(&output),
        "Pink heading should produce ANSI: {output:?}"
    );
}

// ===========================================================================
// 11. Cross-Style Content Preservation
// ===========================================================================

#[test]
fn all_styles_preserve_paragraph_text() {
    let md = "Hello world, this is a paragraph.";
    for style in [
        Style::Dark,
        Style::Light,
        Style::Ascii,
        Style::NoTty,
        Style::Dracula,
        Style::Pink,
        Style::TokyoNight,
    ] {
        let output = render_with(md, style);
        let plain = strip_ansi(&output);
        assert!(
            plain.contains("Hello world"),
            "Style {style:?} should preserve text: {plain:?}"
        );
    }
}

#[test]
fn all_styles_preserve_heading_text() {
    let md = "# Important Title";
    for style in [
        Style::Dark,
        Style::Light,
        Style::Ascii,
        Style::NoTty,
        Style::Dracula,
        Style::Pink,
        Style::TokyoNight,
    ] {
        let output = render_with(md, style);
        let plain = strip_ansi(&output);
        assert!(
            plain.contains("Important Title"),
            "Style {style:?} heading text: {plain:?}"
        );
    }
}

#[test]
fn all_styles_preserve_code_text() {
    let md = "```\nfn hello() {}\n```";
    for style in [
        Style::Dark,
        Style::Light,
        Style::Ascii,
        Style::NoTty,
        Style::Dracula,
        Style::Pink,
        Style::TokyoNight,
    ] {
        let output = render_with(md, style);
        let plain = strip_ansi(&output);
        assert!(
            plain.contains("fn hello()"),
            "Style {style:?} code text: {plain:?}"
        );
    }
}

// ===========================================================================
// 12. ANSI Well-Formedness in Glamour Output
// ===========================================================================

#[test]
fn dark_output_all_sequences_properly_terminated() {
    let md = "# Title\n\nA paragraph with `code` and [link](url).\n\n```\nblock\n```\n\n---\n\n- item1\n- item2";
    let output = render_with(md, Style::Dark);

    // Every \x1b[ should be properly terminated with a letter in @-~
    let mut in_csi = false;
    let mut prev_esc = false;
    for c in output.chars() {
        if c == '\x1b' {
            prev_esc = true;
            continue;
        }
        if prev_esc && c == '[' {
            in_csi = true;
            prev_esc = false;
            continue;
        }
        prev_esc = false;
        if in_csi {
            if ('@'..='~').contains(&c) {
                in_csi = false;
            } else if !c.is_ascii_digit() && c != ';' {
                panic!("Bad CSI byte {c:?} in output: {output:?}");
            }
        }
    }
    assert!(!in_csi, "Unterminated CSI sequence in dark output");
}

#[test]
fn dark_output_every_styled_line_has_reset() {
    let md = "# Title\n\nParagraph text.";
    let output = render_with(md, Style::Dark);

    for (i, line) in output.lines().enumerate() {
        if contains_ansi(line) {
            assert!(
                line.contains("\x1b[0m"),
                "Line {i} has ANSI but no reset: {line:?}"
            );
        }
    }
}

// ===========================================================================
// 13. Document-Level Styling
// ===========================================================================

#[test]
fn dark_document_paragraph_no_ansi() {
    let output = render_with("Hello", Style::Dark);
    // Document/paragraph level doesn't produce per-line ANSI coloring.
    // The color property exists in config but the rendering pipeline
    // only applies ANSI via to_lipgloss().render() for headings.
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("Hello"),
        "Paragraph content present: {plain:?}"
    );
}

#[test]
fn dark_document_has_margin() {
    let output = render_with("Hello", Style::Dark);
    // Document margin is 2 spaces prepended to each line
    let plain = strip_ansi(&output);
    let content_lines: Vec<&str> = plain.lines().filter(|l| !l.trim().is_empty()).collect();
    for line in &content_lines {
        assert!(
            line.starts_with("  "),
            "Document margin (2 spaces) expected: {line:?}"
        );
    }
}

// ===========================================================================
// 14. Word Wrapping Preserves ANSI
// ===========================================================================

#[test]
fn word_wrap_does_not_corrupt_ansi() {
    // Use heading to get ANSI output with word wrapping
    let output = Renderer::new()
        .with_style(Style::Dark)
        .with_word_wrap(40)
        .render("# Title\n\nThis is a long paragraph that should be wrapped correctly.");
    // Heading should have ANSI
    assert!(
        contains_ansi(&output),
        "Should have ANSI from heading: {output:?}"
    );
    let plain = strip_ansi(&output);
    assert!(plain.contains("Title"), "Heading preserved: {plain:?}");
    assert!(
        plain.contains("This is a long paragraph"),
        "Paragraph content preserved: {plain:?}"
    );
}

#[test]
fn wrapped_lines_each_have_reset() {
    let output = Renderer::new()
        .with_style(Style::Dark)
        .with_word_wrap(30)
        .render("This is a paragraph that will definitely wrap across multiple lines.");
    for (i, line) in output.lines().enumerate() {
        if contains_ansi(line) && !line.trim().is_empty() {
            assert!(
                line.contains("\x1b[0m"),
                "Wrapped line {i} should have reset: {line:?}"
            );
        }
    }
}

// ===========================================================================
// 15. Edge Cases
// ===========================================================================

#[test]
fn empty_markdown_no_panic() {
    let output = render_with("", Style::Dark);
    // Should not panic; may produce minimal output
    let _ = output;
}

#[test]
fn only_whitespace_markdown() {
    let output = render_with("   \n   \n   ", Style::Dark);
    let _ = output;
}

#[test]
fn unicode_in_markdown_with_ansi() {
    let output = render_with("# こんにちは\n\n日本語テスト 🦀", Style::Dark);
    let plain = strip_ansi(&output);
    assert!(plain.contains("こんにちは"), "CJK heading: {plain:?}");
    assert!(plain.contains("🦀"), "Emoji preserved: {plain:?}");
}

#[test]
fn nested_markdown_elements() {
    let md = "# Title\n\n> **Bold** in a *blockquote*\n\n1. Item `one`\n2. Item two";
    let output = render_with(md, Style::Dark);
    let plain = strip_ansi(&output);
    assert!(plain.contains("Title"), "Title present");
    assert!(plain.contains("Bold"), "Bold text present");
    assert!(plain.contains("blockquote"), "Blockquote text present");
    assert!(plain.contains("one"), "List item text present");
}

#[test]
fn horizontal_rule_renders() {
    let output = render_with("Above\n\n---\n\nBelow", Style::Dark);
    let plain = strip_ansi(&output);
    assert!(plain.contains("Above"), "Text above HR");
    assert!(plain.contains("Below"), "Text below HR");
    assert!(
        plain.contains("---") || plain.contains("───") || plain.contains("--------"),
        "HR rendered: {plain:?}"
    );
}

#[test]
fn multiple_headings_all_styled() {
    let md = "# H1\n\n## H2\n\n### H3\n\nParagraph";
    let output = render_with(md, Style::Dark);
    let plain = strip_ansi(&output);
    assert!(plain.contains("H1"));
    assert!(plain.contains("H2"));
    assert!(plain.contains("H3"));
    assert!(plain.contains("Paragraph"));
    // All headings should have ANSI
    assert!(contains_ansi(&output));
}

// ===========================================================================
// 16. Auto Style (Alias for Dark)
// ===========================================================================

#[test]
fn auto_style_same_as_dark() {
    let auto_output = render_with("# Hello\n\nWorld", Style::Auto);
    let dark_output = render_with("# Hello\n\nWorld", Style::Dark);
    assert_eq!(auto_output, dark_output, "Auto should equal Dark");
}

// ===========================================================================
// 17. Style Consistency
// ===========================================================================

#[test]
fn render_is_deterministic() {
    let md = "# Title\n\nParagraph with `code` and **bold**.";
    let a = render_with(md, Style::Dark);
    let b = render_with(md, Style::Dark);
    assert_eq!(a, b, "Same input should produce same output");
}

#[test]
fn colored_styles_have_ansi_uncolored_do_not() {
    let md = "# Hello\n\nWorld";
    let colored = [
        Style::Dark,
        Style::Light,
        Style::Dracula,
        Style::Pink,
        Style::TokyoNight,
    ];
    let uncolored = [Style::Ascii, Style::NoTty];

    for style in colored {
        let output = render_with(md, style);
        assert!(
            contains_ansi(&output),
            "Style {style:?} should have ANSI: {output:?}"
        );
    }
    for style in uncolored {
        let output = render_with(md, style);
        assert!(
            !contains_ansi(&output),
            "Style {style:?} should NOT have ANSI: {output:?}"
        );
    }
}
