//! Integration tests for Terminal I/O verification: lipgloss ANSI output (bd-97j0).
//!
//! Verifies that `Style::render()` produces correct, well-formed ANSI escape
//! sequences for colors, text attributes, and resets across all color profiles.

use std::sync::Arc;

use lipgloss::color::TerminalColor;
use lipgloss::{
    AdaptiveColor, AnsiColor, Color, ColorProfile, CompleteColor, NoColor, Renderer, RgbColor,
    Style,
};

// ===========================================================================
// Helpers
// ===========================================================================

/// Check if a string contains any ANSI escape sequence (\x1b[...).
fn contains_ansi(s: &str) -> bool {
    s.contains('\x1b')
}

/// Check if a string contains a specific SGR code pattern.
///
/// Matches:
/// - `\x1b[{code}m` (standalone SGR)
/// - `\x1b[{code};...m` (first in compound)
/// - `...;{code}m` (last in compound)
/// - `...;{code};...m` (middle in compound)
fn contains_sgr(s: &str, code: &str) -> bool {
    let standalone = format!("\x1b[{code}m");
    let prefix = format!("\x1b[{code};");
    let suffix = format!(";{code}m");
    let middle = format!(";{code};");
    s.contains(&standalone) || s.contains(&prefix) || s.contains(&suffix) || s.contains(&middle)
}

/// Strip all ANSI escape sequences from a string.
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip ESC and the following sequence
            if let Some(&next) = chars.peek() {
                if next == '[' {
                    chars.next(); // consume '['
                    // CSI: consume until final byte in @-~
                    while let Some(&c2) = chars.peek() {
                        chars.next();
                        if ('@'..='~').contains(&c2) {
                            break;
                        }
                    }
                } else if next == ']' {
                    chars.next(); // consume ']'
                    // OSC: consume until BEL or ST
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

/// Create a renderer with the given color profile and dark background.
fn renderer(profile: ColorProfile, dark_bg: bool) -> Arc<Renderer> {
    let mut r = Renderer::new();
    r.set_color_profile(profile);
    r.set_has_dark_background(dark_bg);
    Arc::new(r)
}

/// Count the number of ANSI reset sequences in a string.
fn count_resets(s: &str) -> usize {
    s.matches("\x1b[0m").count()
}

/// Count lines in a rendered string.
fn count_lines(s: &str) -> usize {
    s.lines().count()
}

// ===========================================================================
// 1. SGR Text Attribute Sequences
// ===========================================================================

#[test]
fn bold_produces_sgr_1() {
    let s = Style::new().bold().render("hello");
    assert!(contains_sgr(&s, "1"), "Bold should produce SGR 1: {s:?}");
    assert!(s.contains("\x1b[0m"), "Should contain reset");
    assert_eq!(strip_ansi(&s), "hello");
}

#[test]
fn faint_produces_sgr_2() {
    let s = Style::new().faint().render("hello");
    assert!(contains_sgr(&s, "2"), "Faint should produce SGR 2: {s:?}");
    assert!(s.contains("\x1b[0m"), "Should contain reset");
}

#[test]
fn italic_produces_sgr_3() {
    let s = Style::new().italic().render("hello");
    assert!(contains_sgr(&s, "3"), "Italic should produce SGR 3: {s:?}");
    assert!(s.contains("\x1b[0m"), "Should contain reset");
}

#[test]
fn underline_produces_sgr_4() {
    let s = Style::new().underline().render("hello");
    assert!(
        contains_sgr(&s, "4"),
        "Underline should produce SGR 4: {s:?}"
    );
    assert!(s.contains("\x1b[0m"), "Should contain reset");
}

#[test]
fn blink_produces_sgr_5() {
    let s = Style::new().blink().render("hello");
    assert!(contains_sgr(&s, "5"), "Blink should produce SGR 5: {s:?}");
    assert!(s.contains("\x1b[0m"), "Should contain reset");
}

#[test]
fn reverse_produces_sgr_7() {
    let s = Style::new().reverse().render("hello");
    assert!(contains_sgr(&s, "7"), "Reverse should produce SGR 7: {s:?}");
    assert!(s.contains("\x1b[0m"), "Should contain reset");
}

#[test]
fn strikethrough_produces_sgr_9() {
    let s = Style::new().strikethrough().render("hello");
    assert!(
        contains_sgr(&s, "9"),
        "Strikethrough should produce SGR 9: {s:?}"
    );
    assert!(s.contains("\x1b[0m"), "Should contain reset");
}

#[test]
fn all_text_attributes_combined() {
    let s = Style::new()
        .bold()
        .italic()
        .underline()
        .strikethrough()
        .render("styled");
    assert!(contains_sgr(&s, "1"), "Should have bold: {s:?}");
    assert!(contains_sgr(&s, "3"), "Should have italic: {s:?}");
    assert!(contains_sgr(&s, "4"), "Should have underline: {s:?}");
    assert!(contains_sgr(&s, "9"), "Should have strikethrough: {s:?}");
    assert_eq!(strip_ansi(&s), "styled");
}

// ===========================================================================
// 2. Foreground Color Sequences
// ===========================================================================

#[test]
fn truecolor_fg_produces_rgb_sequence() {
    let s = Style::new()
        .renderer(renderer(ColorProfile::TrueColor, true))
        .foreground("#ff0000")
        .render("red");
    assert!(
        s.contains("\x1b[38;2;255;0;0m"),
        "TrueColor fg should produce \\x1b[38;2;R;G;Bm: {s:?}"
    );
    assert_eq!(strip_ansi(&s), "red");
}

#[test]
fn ansi256_fg_produces_256_sequence() {
    let s = Style::new()
        .renderer(renderer(ColorProfile::Ansi256, true))
        .foreground("196")
        .render("red");
    assert!(
        s.contains("\x1b[38;5;196m"),
        "Ansi256 fg should produce \\x1b[38;5;Nm: {s:?}"
    );
}

#[test]
fn ansi16_fg_produces_standard_sequence() {
    // ANSI color 1 (red) should produce \x1b[31m
    let s = Style::new()
        .renderer(renderer(ColorProfile::Ansi, true))
        .foreground("1")
        .render("red");
    assert!(
        s.contains("\x1b[31m"),
        "Ansi16 fg color 1 should produce \\x1b[31m: {s:?}"
    );
}

#[test]
fn ansi16_bright_fg_produces_90_range() {
    // ANSI color 9 (bright red) should produce \x1b[91m
    let s = Style::new()
        .renderer(renderer(ColorProfile::Ansi, true))
        .foreground("9")
        .render("bright_red");
    assert!(
        s.contains("\x1b[91m"),
        "Ansi16 bright fg color 9 should produce \\x1b[91m: {s:?}"
    );
}

#[test]
fn ascii_profile_fg_produces_no_color() {
    let s = Style::new()
        .renderer(renderer(ColorProfile::Ascii, true))
        .foreground("#ff0000")
        .render("nocolor");
    assert!(
        !contains_ansi(&s),
        "Ascii profile should produce no ANSI: {s:?}"
    );
    assert_eq!(s, "nocolor");
}

// ===========================================================================
// 3. Background Color Sequences
// ===========================================================================

#[test]
fn truecolor_bg_produces_rgb_sequence() {
    let s = Style::new()
        .renderer(renderer(ColorProfile::TrueColor, true))
        .background("#0000ff")
        .render("blue_bg");
    assert!(
        s.contains("\x1b[48;2;0;0;255m"),
        "TrueColor bg should produce \\x1b[48;2;R;G;Bm: {s:?}"
    );
}

#[test]
fn ansi256_bg_produces_256_sequence() {
    let s = Style::new()
        .renderer(renderer(ColorProfile::Ansi256, true))
        .background("21")
        .render("blue_bg");
    assert!(
        s.contains("\x1b[48;5;21m"),
        "Ansi256 bg should produce \\x1b[48;5;Nm: {s:?}"
    );
}

#[test]
fn ansi16_bg_produces_40_range() {
    // ANSI color 2 (green bg) should produce \x1b[42m
    let s = Style::new()
        .renderer(renderer(ColorProfile::Ansi, true))
        .background("2")
        .render("green_bg");
    assert!(
        s.contains("\x1b[42m"),
        "Ansi16 bg color 2 should produce \\x1b[42m: {s:?}"
    );
}

#[test]
fn ansi16_bright_bg_produces_100_range() {
    // ANSI color 10 (bright green bg) should produce \x1b[102m
    let s = Style::new()
        .renderer(renderer(ColorProfile::Ansi, true))
        .background("10")
        .render("bright_green_bg");
    assert!(
        s.contains("\x1b[102m"),
        "Ansi16 bright bg color 10 should produce \\x1b[102m: {s:?}"
    );
}

#[test]
fn ascii_profile_bg_produces_no_color() {
    let s = Style::new()
        .renderer(renderer(ColorProfile::Ascii, true))
        .background("#0000ff")
        .render("nocolor");
    assert!(!contains_ansi(&s), "Ascii bg should produce no ANSI: {s:?}");
}

// ===========================================================================
// 4. Color Profile Downgrades (Same Color, Different Profiles)
// ===========================================================================

#[test]
fn hex_color_downgrades_through_profiles() {
    let hex = "#00ff00"; // Bright green

    // TrueColor: RGB sequence
    let tc = Style::new()
        .renderer(renderer(ColorProfile::TrueColor, true))
        .foreground(hex)
        .render("g");
    assert!(tc.contains("\x1b[38;2;0;255;0m"), "TrueColor: {tc:?}");

    // Ansi256: 256-color sequence
    let a256 = Style::new()
        .renderer(renderer(ColorProfile::Ansi256, true))
        .foreground(hex)
        .render("g");
    assert!(
        a256.contains("\x1b[38;5;"),
        "Ansi256 should use 38;5;N format: {a256:?}"
    );

    // Ansi: 16-color sequence (30-37 or 90-97 range)
    let a16 = Style::new()
        .renderer(renderer(ColorProfile::Ansi, true))
        .foreground(hex)
        .render("g");
    // Green maps to ANSI color 10 (bright green) → \x1b[92m
    let has_16_color = (30..=37).any(|n| a16.contains(&format!("\x1b[{n}m")))
        || (90..=97).any(|n| a16.contains(&format!("\x1b[{n}m")));
    assert!(has_16_color, "Ansi should use 16-color: {a16:?}");

    // Ascii: no color
    let ascii = Style::new()
        .renderer(renderer(ColorProfile::Ascii, true))
        .foreground(hex)
        .render("g");
    assert!(!contains_ansi(&ascii), "Ascii: {ascii:?}");

    // All should have the same plain text
    assert_eq!(strip_ansi(&tc), "g");
    assert_eq!(strip_ansi(&a256), "g");
    assert_eq!(strip_ansi(&a16), "g");
    assert_eq!(ascii, "g");
}

#[test]
fn background_downgrades_through_profiles() {
    let hex = "#ff00ff"; // Magenta

    let tc = Style::new()
        .renderer(renderer(ColorProfile::TrueColor, true))
        .background(hex)
        .render("m");
    assert!(tc.contains("\x1b[48;2;255;0;255m"), "TrueColor bg: {tc:?}");

    let a256 = Style::new()
        .renderer(renderer(ColorProfile::Ansi256, true))
        .background(hex)
        .render("m");
    assert!(a256.contains("\x1b[48;5;"), "Ansi256 bg: {a256:?}");

    let a16 = Style::new()
        .renderer(renderer(ColorProfile::Ansi, true))
        .background(hex)
        .render("m");
    let has_16_bg = (40..=47).any(|n| a16.contains(&format!("\x1b[{n}m")))
        || (100..=107).any(|n| a16.contains(&format!("\x1b[{n}m")));
    assert!(has_16_bg, "Ansi bg 16-color: {a16:?}");

    let ascii = Style::new()
        .renderer(renderer(ColorProfile::Ascii, true))
        .background(hex)
        .render("m");
    assert!(!contains_ansi(&ascii), "Ascii bg: {ascii:?}");
}

// ===========================================================================
// 5. Combined Attributes + Colors
// ===========================================================================

#[test]
fn bold_with_foreground_produces_both() {
    let s = Style::new()
        .renderer(renderer(ColorProfile::TrueColor, true))
        .bold()
        .foreground("#ff0000")
        .render("boldred");
    assert!(contains_sgr(&s, "1"), "Should have bold: {s:?}");
    assert!(
        s.contains("\x1b[38;2;255;0;0m"),
        "Should have fg color: {s:?}"
    );
    assert!(s.contains("\x1b[0m"), "Should have reset: {s:?}");
    assert_eq!(strip_ansi(&s), "boldred");
}

#[test]
fn fg_and_bg_together() {
    let s = Style::new()
        .renderer(renderer(ColorProfile::TrueColor, true))
        .foreground("#ffffff")
        .background("#000000")
        .render("contrast");
    assert!(
        s.contains("\x1b[38;2;255;255;255m"),
        "Should have fg: {s:?}"
    );
    assert!(s.contains("\x1b[48;2;0;0;0m"), "Should have bg: {s:?}");
    assert!(s.contains("\x1b[0m"), "Should have reset: {s:?}");
}

#[test]
fn all_attributes_plus_colors() {
    let s = Style::new()
        .renderer(renderer(ColorProfile::TrueColor, true))
        .bold()
        .italic()
        .underline()
        .foreground("#ff0000")
        .background("#0000ff")
        .render("full");
    assert!(contains_sgr(&s, "1"), "bold");
    assert!(contains_sgr(&s, "3"), "italic");
    assert!(contains_sgr(&s, "4"), "underline");
    assert!(s.contains("\x1b[38;2;"), "fg color");
    assert!(s.contains("\x1b[48;2;"), "bg color");
    assert!(s.contains("\x1b[0m"), "reset");
    assert_eq!(strip_ansi(&s), "full");
}

// ===========================================================================
// 6. Multiline Style Application
// ===========================================================================

#[test]
fn multiline_each_line_independently_styled() {
    let s = Style::new().bold().render("line1\nline2\nline3");
    let lines: Vec<&str> = s.lines().collect();
    assert_eq!(lines.len(), 3);
    for (i, line) in lines.iter().enumerate() {
        assert!(
            line.starts_with("\x1b[1m"),
            "Line {i} should start with bold: {line:?}"
        );
        assert!(
            line.ends_with("\x1b[0m"),
            "Line {i} should end with reset: {line:?}"
        );
    }
}

#[test]
fn multiline_no_style_bleed() {
    let s = Style::new()
        .foreground("#ff0000")
        .renderer(renderer(ColorProfile::TrueColor, true))
        .render("a\nb");
    let lines: Vec<&str> = s.lines().collect();
    assert_eq!(lines.len(), 2);
    // Each line should contain its own reset
    for line in &lines {
        assert!(line.contains("\x1b[0m"), "Each line needs reset: {line:?}");
    }
    // Content is preserved
    let plain = strip_ansi(&s);
    assert_eq!(plain, "a\nb");
}

#[test]
fn multiline_reset_count_equals_line_count() {
    let input = "line1\nline2\nline3\nline4";
    let s = Style::new().bold().render(input);
    let resets = count_resets(&s);
    let lines = count_lines(&s);
    assert_eq!(
        resets, lines,
        "Reset count ({resets}) should equal line count ({lines})"
    );
}

// ===========================================================================
// 7. Reset Sequence Placement
// ===========================================================================

#[test]
fn every_styled_line_ends_with_reset() {
    let s = Style::new()
        .bold()
        .italic()
        .foreground("#aabbcc")
        .renderer(renderer(ColorProfile::TrueColor, true))
        .render("one\ntwo\nthree");
    for (i, line) in s.lines().enumerate() {
        assert!(
            line.ends_with("\x1b[0m"),
            "Line {i} should end with reset: {line:?}"
        );
    }
}

#[test]
fn no_orphaned_style_at_end() {
    let s = Style::new().bold().render("text");
    // The string should end with reset, not with an unclosed style
    assert!(s.ends_with("\x1b[0m"), "Should end with reset: {s:?}");
}

#[test]
fn unstyled_text_has_no_escapes() {
    let s = Style::new().render("plain");
    assert!(
        !contains_ansi(&s),
        "Unstyled text should have no ANSI: {s:?}"
    );
    assert_eq!(s, "plain");
}

// ===========================================================================
// 8. TerminalColor trait: Color types
// ===========================================================================

#[test]
fn color_hex_to_ansi_fg_truecolor() {
    let c = Color::from("#abcdef");
    let seq = c.to_ansi_fg(ColorProfile::TrueColor, true);
    assert_eq!(seq, "\x1b[38;2;171;205;239m");
}

#[test]
fn color_hex_to_ansi_bg_truecolor() {
    let c = Color::from("#abcdef");
    let seq = c.to_ansi_bg(ColorProfile::TrueColor, true);
    assert_eq!(seq, "\x1b[48;2;171;205;239m");
}

#[test]
fn color_ansi_number_to_fg_256() {
    let c = Color::from("42");
    let seq = c.to_ansi_fg(ColorProfile::Ansi256, true);
    assert_eq!(seq, "\x1b[38;5;42m");
}

#[test]
fn color_ansi_number_to_bg_256() {
    let c = Color::from("42");
    let seq = c.to_ansi_bg(ColorProfile::Ansi256, true);
    assert_eq!(seq, "\x1b[48;5;42m");
}

#[test]
fn color_ascii_produces_empty() {
    let c = Color::from("#ff0000");
    assert_eq!(c.to_ansi_fg(ColorProfile::Ascii, true), "");
    assert_eq!(c.to_ansi_bg(ColorProfile::Ascii, true), "");
}

#[test]
fn nocolor_produces_empty_for_all_profiles() {
    let nc = NoColor;
    for profile in [
        ColorProfile::Ascii,
        ColorProfile::Ansi,
        ColorProfile::Ansi256,
        ColorProfile::TrueColor,
    ] {
        assert_eq!(nc.to_ansi_fg(profile, true), "", "fg {profile:?}");
        assert_eq!(nc.to_ansi_bg(profile, true), "", "bg {profile:?}");
    }
}

#[test]
fn ansi_color_type_delegates_correctly() {
    let ac = AnsiColor(196);
    let fg = ac.to_ansi_fg(ColorProfile::Ansi256, true);
    assert_eq!(fg, "\x1b[38;5;196m");
    let bg = ac.to_ansi_bg(ColorProfile::Ansi256, true);
    assert_eq!(bg, "\x1b[48;5;196m");
}

#[test]
fn rgb_color_type_truecolor() {
    let rc = RgbColor::new(128, 64, 32);
    let fg = rc.to_ansi_fg(ColorProfile::TrueColor, true);
    assert_eq!(fg, "\x1b[38;2;128;64;32m");
    let bg = rc.to_ansi_bg(ColorProfile::TrueColor, true);
    assert_eq!(bg, "\x1b[48;2;128;64;32m");
}

#[test]
fn rgb_color_type_downgrades_to_256() {
    let rc = RgbColor::new(128, 64, 32);
    let fg = rc.to_ansi_fg(ColorProfile::Ansi256, true);
    assert!(fg.starts_with("\x1b[38;5;"), "Should be 256-color: {fg:?}");
    assert!(fg.ends_with('m'));
}

#[test]
fn rgb_color_type_downgrades_to_16() {
    let rc = RgbColor::new(255, 0, 0);
    let fg = rc.to_ansi_fg(ColorProfile::Ansi, true);
    // Should be a standard or bright foreground color code
    let is_16 = (30..=37).any(|n| fg == format!("\x1b[{n}m"))
        || (90..=97).any(|n| fg == format!("\x1b[{n}m"));
    assert!(is_16, "Should be 16-color fg: {fg:?}");
}

#[test]
fn rgb_color_type_ascii_empty() {
    let rc = RgbColor::new(128, 64, 32);
    assert_eq!(rc.to_ansi_fg(ColorProfile::Ascii, true), "");
    assert_eq!(rc.to_ansi_bg(ColorProfile::Ascii, true), "");
}

// ===========================================================================
// 9. AdaptiveColor Selection
// ===========================================================================

#[test]
fn adaptive_color_dark_bg_selects_dark() {
    let ac = AdaptiveColor {
        light: Color::from("#000000"),
        dark: Color::from("#ffffff"),
    };
    let fg_dark = ac.to_ansi_fg(ColorProfile::TrueColor, true);
    assert_eq!(fg_dark, "\x1b[38;2;255;255;255m", "Dark bg → white fg");

    let fg_light = ac.to_ansi_fg(ColorProfile::TrueColor, false);
    assert_eq!(fg_light, "\x1b[38;2;0;0;0m", "Light bg → black fg");
}

#[test]
fn adaptive_color_bg_also_adapts() {
    let ac = AdaptiveColor {
        light: Color::from("#ffffff"),
        dark: Color::from("#000000"),
    };
    let bg_dark = ac.to_ansi_bg(ColorProfile::TrueColor, true);
    assert_eq!(bg_dark, "\x1b[48;2;0;0;0m");

    let bg_light = ac.to_ansi_bg(ColorProfile::TrueColor, false);
    assert_eq!(bg_light, "\x1b[48;2;255;255;255m");
}

// ===========================================================================
// 10. CompleteColor Profile Selection
// ===========================================================================

#[test]
fn complete_color_uses_correct_profile_field() {
    let cc = CompleteColor {
        truecolor: Some(Color::from("#ff0000")),
        ansi256: Some(Color::from("196")),
        ansi: Some(Color::from("1")),
    };

    let tc = cc.to_ansi_fg(ColorProfile::TrueColor, true);
    assert_eq!(tc, "\x1b[38;2;255;0;0m", "TrueColor profile");

    let a256 = cc.to_ansi_fg(ColorProfile::Ansi256, true);
    assert_eq!(a256, "\x1b[38;5;196m", "Ansi256 profile");

    let a16 = cc.to_ansi_fg(ColorProfile::Ansi, true);
    assert_eq!(a16, "\x1b[31m", "Ansi profile (color 1 → \\x1b[31m)");

    let ascii = cc.to_ansi_fg(ColorProfile::Ascii, true);
    assert_eq!(ascii, "", "Ascii profile");
}

#[test]
fn complete_color_missing_field_returns_empty() {
    let cc = CompleteColor {
        truecolor: Some(Color::from("#ff0000")),
        ansi256: None,
        ansi: None,
    };
    assert_eq!(cc.to_ansi_fg(ColorProfile::Ansi256, true), "");
    assert_eq!(cc.to_ansi_fg(ColorProfile::Ansi, true), "");
}

// ===========================================================================
// 11. Renderer Integration
// ===========================================================================

#[test]
fn style_with_custom_renderer_uses_its_profile() {
    let s = Style::new()
        .renderer(renderer(ColorProfile::Ansi256, true))
        .foreground("#ff0000")
        .render("text");
    // Should use 256-color, not truecolor
    assert!(s.contains("\x1b[38;5;"), "Should use 256-color: {s:?}");
    assert!(!s.contains("\x1b[38;2;"), "Should NOT use truecolor: {s:?}");
}

#[test]
fn style_with_ascii_renderer_no_ansi() {
    let s = Style::new()
        .renderer(renderer(ColorProfile::Ascii, true))
        .bold()
        .italic()
        .foreground("#ff0000")
        .background("#0000ff")
        .render("plain");
    // Ascii profile means no ANSI from colors, but SGR attributes are still present
    // because bold/italic are separate from color profile.
    // Actually, looking at the code, the style_start string includes attributes
    // regardless of profile. Only colors check the profile. So bold and italic
    // will still produce \x1b[1m and \x1b[3m even with Ascii profile.
    // Let me verify: colors should be empty, but attributes should be present.
    assert!(contains_sgr(&s, "1"), "Bold SGR still present: {s:?}");
    assert!(contains_sgr(&s, "3"), "Italic SGR still present: {s:?}");
    // But no color sequences
    assert!(!s.contains("\x1b[38;"), "No fg color: {s:?}");
    assert!(!s.contains("\x1b[48;"), "No bg color: {s:?}");
}

#[test]
fn renderer_dark_background_flag_affects_adaptive() {
    let dark_r = renderer(ColorProfile::TrueColor, true);
    let light_r = renderer(ColorProfile::TrueColor, false);

    // Use adaptive color via the style
    // Note: style.foreground() takes a &str, but AdaptiveColor requires the
    // TerminalColor trait. Let's test at the TerminalColor level directly.
    let ac = AdaptiveColor {
        light: Color::from("#111111"),
        dark: Color::from("#eeeeee"),
    };
    let dark_fg = ac.to_ansi_fg(dark_r.color_profile(), dark_r.has_dark_background());
    let light_fg = ac.to_ansi_fg(light_r.color_profile(), light_r.has_dark_background());

    assert_ne!(
        dark_fg, light_fg,
        "Different bg should produce different colors"
    );
    assert!(
        dark_fg.contains("238;238;238"),
        "Dark bg → light text: {dark_fg:?}"
    );
    assert!(
        light_fg.contains("17;17;17"),
        "Light bg → dark text: {light_fg:?}"
    );
}

// ===========================================================================
// 12. ANSI Sequence Well-Formedness
// ===========================================================================

#[test]
fn all_escape_sequences_are_properly_terminated() {
    let s = Style::new()
        .bold()
        .italic()
        .foreground("#abcdef")
        .background("#123456")
        .renderer(renderer(ColorProfile::TrueColor, true))
        .render("test\nline2\nline3");

    // Every \x1b[ should have a matching terminal byte (letter)
    let mut in_seq = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_seq = true;
            continue;
        }
        if in_seq && c == '[' {
            // Now in CSI
            continue;
        }
        if in_seq {
            if ('@'..='~').contains(&c) {
                in_seq = false; // properly terminated
            } else if !c.is_ascii_digit() && c != ';' {
                panic!("Unexpected character in CSI sequence: {c:?} in {s:?}");
            }
        }
    }
    assert!(
        !in_seq,
        "Unterminated escape sequence at end of string: {s:?}"
    );
}

#[test]
fn multiline_with_colors_all_sequences_terminated() {
    let s = Style::new()
        .foreground("#ff8800")
        .background("#001122")
        .bold()
        .underline()
        .renderer(renderer(ColorProfile::TrueColor, true))
        .render("alpha\nbeta\ngamma\ndelta");

    let esc_count = s.matches('\x1b').count();
    let reset_count = count_resets(&s);
    // Each line gets style_start (multiple \x1b) + 1 reset
    assert_eq!(reset_count, 4, "4 lines → 4 resets");
    // Each escape should be part of a complete sequence
    assert!(
        esc_count > reset_count,
        "More escapes than resets (attrs+colors+resets)"
    );
}

// ===========================================================================
// 13. Color Conversion Correctness
// ===========================================================================

#[test]
fn hex_3_digit_shorthand_expands_correctly() {
    let c = Color::from("#f0a");
    let (r, g, b) = c.as_rgb().expect("Should parse 3-digit hex");
    assert_eq!((r, g, b), (255, 0, 170));
    let seq = c.to_ansi_fg(ColorProfile::TrueColor, true);
    assert_eq!(seq, "\x1b[38;2;255;0;170m");
}

#[test]
fn ansi_standard_colors_fg_range_0_to_7() {
    for n in 0..8u8 {
        let c = Color::from(n.to_string().as_str());
        let seq = c.to_ansi_fg(ColorProfile::Ansi, true);
        let expected = format!("\x1b[{}m", 30 + n);
        assert_eq!(seq, expected, "ANSI color {n}");
    }
}

#[test]
fn ansi_bright_colors_fg_range_8_to_15() {
    for n in 8..16u8 {
        let c = Color::from(n.to_string().as_str());
        let seq = c.to_ansi_fg(ColorProfile::Ansi, true);
        let expected = format!("\x1b[{}m", 90 + n - 8);
        assert_eq!(seq, expected, "ANSI bright color {n}");
    }
}

#[test]
fn ansi_standard_colors_bg_range_0_to_7() {
    for n in 0..8u8 {
        let c = Color::from(n.to_string().as_str());
        let seq = c.to_ansi_bg(ColorProfile::Ansi, true);
        let expected = format!("\x1b[{}m", 40 + n);
        assert_eq!(seq, expected, "ANSI bg color {n}");
    }
}

#[test]
fn ansi_bright_colors_bg_range_8_to_15() {
    for n in 8..16u8 {
        let c = Color::from(n.to_string().as_str());
        let seq = c.to_ansi_bg(ColorProfile::Ansi, true);
        let expected = format!("\x1b[{}m", 100 + n - 8);
        assert_eq!(seq, expected, "ANSI bright bg color {n}");
    }
}

#[test]
fn ansi256_high_colors_produce_38_5_sequence() {
    for n in [16u8, 100, 196, 231, 232, 255] {
        let c = Color::from(n.to_string().as_str());
        let seq = c.to_ansi_fg(ColorProfile::Ansi256, true);
        let expected = format!("\x1b[38;5;{n}m");
        assert_eq!(seq, expected, "256-color {n}");
    }
}

// ===========================================================================
// 14. Visible Width with ANSI Codes
// ===========================================================================

#[test]
fn visible_width_ignores_sgr_codes() {
    let plain = "hello";
    let styled = Style::new()
        .bold()
        .foreground("#ff0000")
        .renderer(renderer(ColorProfile::TrueColor, true))
        .render(plain);
    assert_eq!(lipgloss::visible_width(&styled), plain.len());
}

#[test]
fn visible_width_handles_multiline_styled() {
    let styled = Style::new().italic().render("abc\ndef");
    // visible_width counts the widest line
    assert_eq!(lipgloss::width(&styled), 3);
}

#[test]
fn visible_width_empty_string() {
    assert_eq!(lipgloss::visible_width(""), 0);
}

#[test]
fn visible_width_only_ansi() {
    assert_eq!(lipgloss::visible_width("\x1b[1m\x1b[0m"), 0);
}

#[test]
fn visible_width_unicode_with_ansi() {
    let styled = Style::new().bold().render("こんにちは");
    // Each CJK character is 2 cells wide → 5 chars × 2 = 10
    assert_eq!(lipgloss::visible_width(&styled), 10);
}

// ===========================================================================
// 15. Edge Cases
// ===========================================================================

#[test]
fn empty_text_styled() {
    let s = Style::new().bold().render("");
    // Empty text has zero lines from .lines(), so no ANSI is emitted
    assert_eq!(s, "", "Empty text renders as empty even with style: {s:?}");
}

#[test]
fn style_with_no_attrs_is_passthrough() {
    let s = Style::new().render("passthrough");
    assert_eq!(s, "passthrough");
}

#[test]
fn unicode_content_preserved_with_styling() {
    let s = Style::new().bold().render("日本語テスト 🦀");
    let plain = strip_ansi(&s);
    assert_eq!(plain, "日本語テスト 🦀");
}

#[test]
fn very_long_text_styled() {
    let long = "x".repeat(10_000);
    let s = Style::new().bold().render(&long);
    assert_eq!(strip_ansi(&s), long);
    assert!(s.starts_with("\x1b[1m"));
    assert!(s.ends_with("\x1b[0m"));
}

#[test]
fn newlines_only_text_styled() {
    let s = Style::new().bold().render("\n\n\n");
    let lines: Vec<&str> = s.lines().collect();
    // lines() skips trailing empty strings from split
    // "\n\n\n" has 3 lines of content (all empty)
    for line in &lines {
        if !line.is_empty() {
            assert!(
                line.contains("\x1b[1m"),
                "Non-empty line should have bold: {line:?}"
            );
        }
    }
}

#[test]
fn tab_in_content_preserved() {
    let s = Style::new().render("a\tb");
    // Default tab width is 4 spaces
    assert!(s.contains("    ") || s.contains('\t'), "Tab handled: {s:?}");
}

// ===========================================================================
// 16. Style Composition Patterns
// ===========================================================================

#[test]
fn cloned_style_independent_rendering() {
    let base = Style::new()
        .renderer(renderer(ColorProfile::TrueColor, true))
        .bold();
    let red = base.clone().foreground("#ff0000");
    let blue = base.foreground("#0000ff");

    let r = red.render("red");
    let b = blue.render("blue");

    assert!(r.contains("\x1b[38;2;255;0;0m"), "Red fg: {r:?}");
    assert!(b.contains("\x1b[38;2;0;0;255m"), "Blue fg: {b:?}");
    // Both have bold
    assert!(contains_sgr(&r, "1"), "Red has bold");
    assert!(contains_sgr(&b, "1"), "Blue has bold");
}

#[test]
fn nested_render_output_is_valid() {
    // Render inner text first, then wrap in outer style
    let inner = Style::new().italic().render("inner");
    let outer = Style::new().bold().render(&inner);
    // outer should wrap the already-styled inner text
    assert!(outer.contains("\x1b[1m"), "Outer bold");
    assert!(outer.contains("\x1b[3m"), "Inner italic preserved");
    let plain = strip_ansi(&outer);
    assert_eq!(plain, "inner");
}

// ===========================================================================
// 17. Color Conversion Helpers
// ===========================================================================

#[test]
fn rgb_to_ansi256_roundtrip_grayscale() {
    // Pure gray values should map to grayscale range (232-255) or near it
    let n = lipgloss::color::rgb_to_ansi256(128, 128, 128);
    assert!(
        n >= 232 || n == 16,
        "Gray should be in grayscale range or cube: {n}"
    );
}

#[test]
fn ansi256_to_rgb_standard_colors() {
    // Color 0 (black) → (0, 0, 0)
    assert_eq!(lipgloss::color::ansi256_to_rgb(0), (0, 0, 0));
    // Color 15 (bright white) → (255, 255, 255)
    assert_eq!(lipgloss::color::ansi256_to_rgb(15), (255, 255, 255));
}

#[test]
fn ansi256_to_rgb_grayscale_range() {
    // Grayscale range 232-255
    let (r, g, b) = lipgloss::color::ansi256_to_rgb(232);
    assert_eq!(r, g);
    assert_eq!(g, b);
    assert_eq!(r, 8); // First grayscale entry is 8

    let (r, g, b) = lipgloss::color::ansi256_to_rgb(255);
    assert_eq!(r, g);
    assert_eq!(g, b);
    assert_eq!(r, 238); // Last grayscale entry is 238
}

#[test]
fn rgb_to_ansi16_maps_to_valid_range() {
    let n = lipgloss::color::rgb_to_ansi16(255, 0, 0);
    assert!(n < 16, "Should be in 0-15 range: {n}");
    // Bright red (255,0,0) should map to 9 (bright red)
    assert_eq!(n, 9);
}

// ===========================================================================
// 18. Sequence Order Verification
// ===========================================================================

#[test]
fn attributes_come_before_colors_in_sequence() {
    let s = Style::new()
        .renderer(renderer(ColorProfile::TrueColor, true))
        .bold()
        .foreground("#ff0000")
        .render("order");
    // The bold SGR should appear before the color SGR
    let bold_pos = s.find("\x1b[1m").expect("Should have bold");
    let color_pos = s.find("\x1b[38;2;").expect("Should have fg color");
    assert!(
        bold_pos < color_pos,
        "Bold ({bold_pos}) should come before color ({color_pos})"
    );
}

#[test]
fn fg_comes_before_bg_in_sequence() {
    let s = Style::new()
        .renderer(renderer(ColorProfile::TrueColor, true))
        .foreground("#ff0000")
        .background("#0000ff")
        .render("order");
    let fg_pos = s.find("\x1b[38;2;").expect("Should have fg");
    let bg_pos = s.find("\x1b[48;2;").expect("Should have bg");
    assert!(
        fg_pos < bg_pos,
        "Fg ({fg_pos}) should come before bg ({bg_pos})"
    );
}

#[test]
fn reset_comes_after_content() {
    let s = Style::new().bold().render("content");
    let content_start = s.find("content").expect("Should have content");
    let reset_pos = s.rfind("\x1b[0m").expect("Should have reset");
    assert!(
        content_start < reset_pos,
        "Content ({content_start}) before reset ({reset_pos})"
    );
}
