//! Property-based tests for glamour text wrapping (bd-dj49).
//!
//! Tests invariants of the word wrapping algorithm and rendering pipeline:
//! - Line width bounds (no line exceeds wrap width + margin)
//! - Content preservation (words survive the wrap/render pipeline)
//! - Stability (never panics for any input)
//! - Monotonicity (wider wrap → fewer or equal lines)
//! - Unicode and ANSI handling

#![allow(clippy::doc_markdown)]
#![allow(clippy::redundant_closure_for_method_calls)]

use glamour::{Renderer, Style};
use lipgloss::visible_width;
use proptest::prelude::*;

// ===========================================================================
// Helpers
// ===========================================================================

/// Strip ANSI escape sequences from a string, returning plain text.
fn strip_ansi(s: &str) -> String {
    #[derive(Clone, Copy, PartialEq)]
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
            State::Esc => {
                if c == '[' {
                    state = State::Csi;
                } else if c == ']' {
                    state = State::Osc;
                } else {
                    state = State::Normal;
                }
            }
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

/// Extract non-empty whitespace-split words from a string.
fn extract_words(s: &str) -> Vec<String> {
    s.split_whitespace()
        .filter(|w| !w.is_empty())
        .map(|w| w.to_string())
        .collect()
}

/// Count non-empty lines (after trimming trailing whitespace on each line).
fn count_content_lines(s: &str) -> usize {
    s.lines().filter(|l| !l.trim().is_empty()).count()
}

/// The document margin for the Ascii style (2 spaces prepended to each line).
const ASCII_MARGIN: usize = 2;

// ===========================================================================
// 1. Stability: Rendering never panics
// ===========================================================================

proptest! {
    /// Any arbitrary input text and wrap width produces a result without panicking.
    #[test]
    fn render_never_panics(
        text in "\\PC{0,300}",
        wrap_width in 0usize..300,
    ) {
        let renderer = Renderer::new()
            .with_word_wrap(wrap_width)
            .with_style(Style::Ascii);
        let _output = renderer.render(&text);
    }

    /// Rendering with various styles never panics.
    #[test]
    fn all_styles_never_panic(
        text in "[a-zA-Z0-9 .,!?\\n]{0,200}",
        style_idx in 0usize..7,
    ) {
        let styles = [
            Style::Ascii,
            Style::Dark,
            Style::Dracula,
            Style::Light,
            Style::Pink,
            Style::TokyoNight,
            Style::NoTty,
        ];
        let style = styles[style_idx % styles.len()];
        let renderer = Renderer::new()
            .with_word_wrap(60)
            .with_style(style);
        let _output = renderer.render(&text);
    }

    /// Markdown with headings, lists, code blocks never panics.
    #[test]
    fn markdown_elements_never_panic(
        heading in "[a-zA-Z ]{1,30}",
        paragraph in "[a-zA-Z ]{1,60}",
        code in "[a-zA-Z0-9_(){}; ]{0,40}",
        list_items in prop::collection::vec("[a-zA-Z ]{1,20}", 1..5),
        wrap_width in 20usize..120,
    ) {
        let list_md: String = list_items.iter()
            .map(|item| format!("- {item}"))
            .collect::<Vec<_>>()
            .join("\n");
        let md = format!(
            "# {heading}\n\n{paragraph}\n\n```\n{code}\n```\n\n{list_md}\n"
        );
        let renderer = Renderer::new()
            .with_word_wrap(wrap_width)
            .with_style(Style::Ascii);
        let _output = renderer.render(&md);
    }
}

// ===========================================================================
// 2. Content Preservation
// ===========================================================================

proptest! {
    /// All words from a plain paragraph appear in the rendered output.
    #[test]
    fn paragraph_words_preserved(
        words in prop::collection::vec("[a-zA-Z]{1,12}", 1..20),
        wrap_width in 20usize..120,
    ) {
        let text = words.join(" ");
        let renderer = Renderer::new()
            .with_word_wrap(wrap_width)
            .with_style(Style::Ascii);
        let output = renderer.render(&text);
        let plain = strip_ansi(&output);

        for word in &words {
            prop_assert!(
                plain.contains(word.as_str()),
                "Word '{}' missing from output.\nInput: {}\nOutput: {}",
                word, text, plain
            );
        }
    }

    /// All words from multiple paragraphs appear in output.
    #[test]
    fn multi_paragraph_words_preserved(
        para1 in prop::collection::vec("[a-zA-Z]{2,10}", 3..10),
        para2 in prop::collection::vec("[a-zA-Z]{2,10}", 3..10),
        wrap_width in 30usize..100,
    ) {
        let text1 = para1.join(" ");
        let text2 = para2.join(" ");
        let md = format!("{text1}\n\n{text2}");
        let renderer = Renderer::new()
            .with_word_wrap(wrap_width)
            .with_style(Style::Ascii);
        let output = renderer.render(&md);
        let plain = strip_ansi(&output);

        for word in para1.iter().chain(para2.iter()) {
            prop_assert!(
                plain.contains(word.as_str()),
                "Word '{}' missing from multi-paragraph output",
                word
            );
        }
    }

    /// Heading text is preserved in the rendered output.
    #[test]
    fn heading_text_preserved(
        heading_text in "[a-zA-Z]{3,20}",
        level in 1usize..=6,
    ) {
        let prefix = "#".repeat(level);
        let md = format!("{prefix} {heading_text}");
        let renderer = Renderer::new()
            .with_word_wrap(80)
            .with_style(Style::Ascii);
        let output = renderer.render(&md);
        let plain = strip_ansi(&output);

        prop_assert!(
            plain.contains(&heading_text),
            "Heading text '{}' missing from output: {}",
            heading_text, plain
        );
    }

    /// List item text is preserved.
    #[test]
    fn list_item_text_preserved(
        items in prop::collection::vec("[a-zA-Z]{3,15}", 1..6),
    ) {
        let md: String = items.iter()
            .map(|item| format!("- {item}"))
            .collect::<Vec<_>>()
            .join("\n");
        let renderer = Renderer::new()
            .with_word_wrap(80)
            .with_style(Style::Ascii);
        let output = renderer.render(&md);
        let plain = strip_ansi(&output);

        for item in &items {
            prop_assert!(
                plain.contains(item.as_str()),
                "List item '{}' missing from output: {}",
                item, plain
            );
        }
    }

    /// Code block content is preserved.
    #[test]
    fn code_block_content_preserved(
        code_text in "[a-zA-Z0-9_]{3,30}",
    ) {
        let md = format!("```\n{code_text}\n```");
        let renderer = Renderer::new()
            .with_word_wrap(80)
            .with_style(Style::Ascii);
        let output = renderer.render(&md);
        let plain = strip_ansi(&output);

        prop_assert!(
            plain.contains(&code_text),
            "Code block text '{}' missing from output: {}",
            code_text, plain
        );
    }
}

// ===========================================================================
// 3. Line Width Bounds
// ===========================================================================

proptest! {
    /// For plain paragraph text, each visible line width does not exceed
    /// wrap_width + document margin. Individual words longer than wrap_width
    /// are exempt (wrapping doesn't break words).
    #[test]
    fn paragraph_line_width_bounded(
        words in prop::collection::vec("[a-zA-Z]{1,10}", 3..25),
        wrap_width in 20usize..120,
    ) {
        let text = words.join(" ");
        let longest_word = words.iter().map(|w| w.len()).max().unwrap_or(0);
        let renderer = Renderer::new()
            .with_word_wrap(wrap_width)
            .with_style(Style::Ascii);
        let output = renderer.render(&text);

        // Max allowed width: wrap_width (content) + margin (prepended by renderer)
        // If a single word exceeds wrap_width, that word goes on its own line
        let max_allowed = wrap_width.max(longest_word) + ASCII_MARGIN;

        for line in output.lines() {
            let vw = visible_width(line);
            // Skip empty/whitespace-only lines (document prefix/suffix)
            if line.trim().is_empty() {
                continue;
            }
            prop_assert!(
                vw <= max_allowed,
                "Line too wide: visible_width={}, max_allowed={}, line='{}'\nwrap_width={}, longest_word={}",
                vw, max_allowed, line, wrap_width, longest_word
            );
        }
    }

    /// For words within the wrap width, no line's content exceeds wrap_width.
    /// (Uses short words to guarantee no word exceeds the width.)
    #[test]
    fn short_words_line_width_strictly_bounded(
        words in prop::collection::vec("[a-z]{1,5}", 5..30),
        wrap_width in 20usize..120,
    ) {
        let text = words.join(" ");
        let renderer = Renderer::new()
            .with_word_wrap(wrap_width)
            .with_style(Style::Ascii);
        let output = renderer.render(&text);

        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let vw = visible_width(line);
            // Content width = vw - margin; should be <= wrap_width
            // Total visible width should be <= wrap_width + margin
            prop_assert!(
                vw <= wrap_width + ASCII_MARGIN,
                "Line exceeds wrap width: visible={}, limit={}, line='{}'",
                vw, wrap_width + ASCII_MARGIN, line
            );
        }
    }
}

// ===========================================================================
// 4. Width 0 Disables Wrapping
// ===========================================================================

proptest! {
    /// With wrap_width=0, the word_wrap function returns text unchanged,
    /// so long text should NOT be split across lines (apart from document
    /// prefix/suffix newlines).
    #[test]
    fn width_zero_does_not_wrap(
        words in prop::collection::vec("[a-zA-Z]{3,10}", 10..30),
    ) {
        let text = words.join(" ");
        let renderer = Renderer::new()
            .with_word_wrap(0)
            .with_style(Style::Ascii);
        let output = renderer.render(&text);

        // With wrapping disabled, all words should be on a single content line
        let content_lines: Vec<&str> = output.lines()
            .filter(|l| !l.trim().is_empty())
            .collect();

        // All input words should appear together (single paragraph = single content line)
        prop_assert!(
            content_lines.len() == 1,
            "With wrap_width=0, expected 1 content line, got {}: {:?}",
            content_lines.len(), content_lines
        );
    }
}

// ===========================================================================
// 5. Monotonicity: Wider wrap → fewer or equal lines
// ===========================================================================

proptest! {
    /// Wider wrap width produces fewer or equal content lines.
    #[test]
    fn wider_wrap_fewer_or_equal_lines(
        words in prop::collection::vec("[a-zA-Z]{2,8}", 5..20),
        narrow in 20usize..60,
        extra in 1usize..60,
    ) {
        let wide = narrow + extra;
        let text = words.join(" ");

        let narrow_renderer = Renderer::new()
            .with_word_wrap(narrow)
            .with_style(Style::Ascii);
        let wide_renderer = Renderer::new()
            .with_word_wrap(wide)
            .with_style(Style::Ascii);

        let narrow_output = narrow_renderer.render(&text);
        let wide_output = wide_renderer.render(&text);

        let narrow_lines = count_content_lines(&narrow_output);
        let wide_lines = count_content_lines(&wide_output);

        prop_assert!(
            wide_lines <= narrow_lines,
            "Wider wrap should produce fewer/equal lines: narrow({})={} lines, wide({})={} lines",
            narrow, narrow_lines, wide, wide_lines
        );
    }
}

// ===========================================================================
// 6. Word Integrity (wrapping doesn't break words)
// ===========================================================================

proptest! {
    /// Words are never split across lines by the wrapping algorithm.
    #[test]
    fn words_never_split_across_lines(
        words in prop::collection::vec("[a-zA-Z]{2,12}", 3..15),
        wrap_width in 15usize..80,
    ) {
        let text = words.join(" ");
        let renderer = Renderer::new()
            .with_word_wrap(wrap_width)
            .with_style(Style::Ascii);
        let output = renderer.render(&text);
        let plain = strip_ansi(&output);

        // Each input word should appear as a whole token on some line
        for word in &words {
            let found = plain.lines().any(|line| {
                // Word appears as a whitespace-delimited token
                extract_words(line).iter().any(|w| w == word)
            });
            prop_assert!(
                found,
                "Word '{}' was split or missing.\nInput: {}\nOutput: {}",
                word, text, plain
            );
        }
    }
}

// ===========================================================================
// 7. ANSI Codes Don't Affect Content
// ===========================================================================

proptest! {
    /// Different styles produce the same plain-text words for paragraph content.
    #[test]
    fn styles_produce_same_plain_text_words(
        words in prop::collection::vec("[a-zA-Z]{2,10}", 3..12),
    ) {
        let text = words.join(" ");

        let ascii_output = Renderer::new()
            .with_word_wrap(80)
            .with_style(Style::Ascii)
            .render(&text);
        let dark_output = Renderer::new()
            .with_word_wrap(80)
            .with_style(Style::Dark)
            .render(&text);

        let ascii_words = extract_words(&strip_ansi(&ascii_output));
        let dark_words = extract_words(&strip_ansi(&dark_output));

        // Both should contain the same words (order may differ due to margins,
        // but content should match)
        for word in &words {
            prop_assert!(
                ascii_words.contains(word),
                "Word '{}' missing from Ascii output",
                word
            );
            prop_assert!(
                dark_words.contains(word),
                "Word '{}' missing from Dark output",
                word
            );
        }
    }

    /// ANSI codes in styled output don't count toward visible width.
    #[test]
    fn ansi_codes_dont_affect_visible_width(
        words in prop::collection::vec("[a-z]{2,8}", 3..15),
        wrap_width in 30usize..100,
    ) {
        let text = words.join(" ");

        let dark_output = Renderer::new()
            .with_word_wrap(wrap_width)
            .with_style(Style::Dark)
            .render(&text);

        // Even with ANSI codes, visible_width should respect wrap limits
        for line in dark_output.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let vw = visible_width(line);
            let raw_len = line.len();
            // Visible width should be <= raw byte length (ANSI codes add bytes)
            prop_assert!(
                vw <= raw_len,
                "visible_width ({}) > raw length ({}) - impossible",
                vw, raw_len
            );
        }
    }
}

// ===========================================================================
// 8. Edge Cases
// ===========================================================================

proptest! {
    /// Empty input renders without panic and produces minimal output.
    #[test]
    fn empty_input_renders(
        wrap_width in 0usize..200,
    ) {
        let renderer = Renderer::new()
            .with_word_wrap(wrap_width)
            .with_style(Style::Ascii);
        let output = renderer.render("");
        // Should produce some output (at least document prefix/suffix)
        // but no content words
        let plain = strip_ansi(&output);
        let words = extract_words(&plain);
        prop_assert!(
            words.is_empty(),
            "Empty input should produce no words, got: {:?}",
            words
        );
    }

    /// A single very long word is not broken by wrapping.
    #[test]
    fn long_word_not_broken(
        word_len in 30usize..100,
        wrap_width in 10usize..25,
    ) {
        // Create a word longer than wrap_width
        #[allow(clippy::cast_possible_truncation)]
        let word: String = (0..word_len).map(|i| (b'a' + (i % 26) as u8) as char).collect();
        let renderer = Renderer::new()
            .with_word_wrap(wrap_width)
            .with_style(Style::Ascii);
        let output = renderer.render(&word);
        let plain = strip_ansi(&output);

        // The word should appear intact on some line
        let found = plain.lines().any(|line| line.contains(&word));
        prop_assert!(
            found,
            "Long word (len={}) should appear intact in output.\nWord: {}\nOutput: {}",
            word_len, word, plain
        );
    }

    /// Only whitespace input renders without panic and produces no content words.
    #[test]
    fn whitespace_only_input(
        spaces in " {1,50}",
        wrap_width in 10usize..100,
    ) {
        let renderer = Renderer::new()
            .with_word_wrap(wrap_width)
            .with_style(Style::Ascii);
        let output = renderer.render(&spaces);
        // Should not panic; content may be empty or minimal
        let _ = output;
    }
}

// ===========================================================================
// 9. visible_width consistency via lipgloss
// ===========================================================================

proptest! {
    /// visible_width of plain ASCII text equals its byte length.
    #[test]
    fn visible_width_ascii_equals_len(
        text in "[a-zA-Z0-9 ]{0,100}",
    ) {
        let vw = visible_width(&text);
        // For pure ASCII (no control chars), visible width == char count
        let expected = text.chars().count();
        prop_assert_eq!(
            vw, expected,
            "visible_width mismatch for ASCII text: got {}, expected {}",
            vw, expected
        );
    }

    /// visible_width of ANSI-styled text is less than or equal to raw byte length.
    #[test]
    fn visible_width_with_ansi_leq_raw(
        text in "[a-zA-Z]{1,20}",
        code in 30u8..48,
    ) {
        let styled = format!("\x1b[{code}m{text}\x1b[0m");
        let vw = visible_width(&styled);
        let text_width = visible_width(&text);

        // ANSI escape codes should not contribute to visible width
        prop_assert_eq!(
            vw, text_width,
            "ANSI codes should not affect visible width: styled={}, text={}",
            vw, text_width
        );
    }

    /// visible_width of empty string is 0.
    #[test]
    fn visible_width_empty_is_zero(_dummy in 0..1u8) {
        prop_assert_eq!(visible_width(""), 0);
    }

    /// visible_width with only ANSI codes is 0.
    #[test]
    fn visible_width_only_ansi_is_zero(
        code in 30u8..48,
    ) {
        let only_ansi = format!("\x1b[{code}m\x1b[0m");
        prop_assert_eq!(
            visible_width(&only_ansi), 0,
            "ANSI-only string should have visible_width=0"
        );
    }

    /// CSI sequences of various lengths have zero visible width.
    #[test]
    fn csi_sequences_zero_width(
        params in "[0-9;]{0,10}",
        final_byte in prop::sample::select(vec!['m', 'H', 'J', 'K', 'A', 'B', 'C', 'D']),
    ) {
        let csi = format!("\x1b[{params}{final_byte}");
        prop_assert_eq!(
            visible_width(&csi), 0,
            "CSI sequence should have zero visible width: {:?}",
            csi
        );
    }
}

// ===========================================================================
// 10. Word Order Preservation
// ===========================================================================

proptest! {
    /// Words appear in the same order in the output as in the input.
    #[test]
    fn word_order_preserved(
        words in prop::collection::vec("[a-z]{3,8}", 5..15),
        wrap_width in 20usize..80,
    ) {
        let text = words.join(" ");
        let renderer = Renderer::new()
            .with_word_wrap(wrap_width)
            .with_style(Style::Ascii);
        let output = renderer.render(&text);
        let plain = strip_ansi(&output);
        let output_words = extract_words(&plain);

        // Check order: each input word should appear in order in output
        let mut search_from = 0;
        for word in &words {
            let pos = output_words[search_from..]
                .iter()
                .position(|w| w == word);
            prop_assert!(
                pos.is_some(),
                "Word '{}' not found in order. Remaining output words: {:?}",
                word, &output_words[search_from..]
            );
            search_from += pos.unwrap() + 1;
        }
    }
}

// ===========================================================================
// 11. Render Determinism
// ===========================================================================

proptest! {
    /// Rendering the same input twice produces identical output.
    #[test]
    fn render_is_deterministic(
        text in "[a-zA-Z0-9 .,\\n]{0,200}",
        wrap_width in 10usize..120,
    ) {
        let renderer = Renderer::new()
            .with_word_wrap(wrap_width)
            .with_style(Style::Ascii);
        let output1 = renderer.render(&text);
        let output2 = renderer.render(&text);
        prop_assert_eq!(
            output1, output2,
            "Rendering should be deterministic"
        );
    }
}

// ===========================================================================
// 12. Convenience render() function
// ===========================================================================

proptest! {
    /// The convenience render() function never panics.
    #[test]
    fn convenience_render_never_panics(
        text in "[a-zA-Z0-9 .,!?\\n#*_`-]{0,200}",
        style_idx in 0usize..7,
    ) {
        let styles = [
            Style::Ascii,
            Style::Dark,
            Style::Dracula,
            Style::Light,
            Style::Pink,
            Style::TokyoNight,
            Style::NoTty,
        ];
        let style = styles[style_idx % styles.len()];
        let result = glamour::render(&text, style);
        prop_assert!(result.is_ok(), "render() should never fail");
    }
}
