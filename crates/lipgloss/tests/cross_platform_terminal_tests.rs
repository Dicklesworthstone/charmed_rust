//! Integration Tests: Cross-platform terminal support (bd-2zkx).
//!
//! Verifies that lipgloss renders correctly across different terminal
//! environments by testing color profile detection, graceful degradation,
//! backend abstraction, and environment variable handling.

use std::sync::Arc;

use lipgloss::backend::{AnsiBackend, HtmlBackend, OutputBackend, PlainBackend};
use lipgloss::color::TerminalColor;
use lipgloss::{
    AdaptiveColor, AnsiColor, Color, ColorProfile, CompleteAdaptiveColor, CompleteColor, NoColor,
    Renderer, RgbColor, Style,
};

// ===========================================================================
// Helpers
// ===========================================================================

fn renderer(profile: ColorProfile, dark_bg: bool) -> Arc<Renderer> {
    let mut r = Renderer::new();
    r.set_color_profile(profile);
    r.set_has_dark_background(dark_bg);
    Arc::new(r)
}

fn contains_ansi(s: &str) -> bool {
    s.contains('\x1b')
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                while let Some(&c2) = chars.peek() {
                    chars.next();
                    if ('@'..='~').contains(&c2) {
                        break;
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// All four color profiles in degradation order.
const ALL_PROFILES: [ColorProfile; 4] = [
    ColorProfile::TrueColor,
    ColorProfile::Ansi256,
    ColorProfile::Ansi,
    ColorProfile::Ascii,
];

// ===========================================================================
// 1. Color Profile Detection & Hierarchy
// ===========================================================================

#[test]
fn color_profile_supports_hierarchy() {
    // TrueColor supports everything
    assert!(ColorProfile::TrueColor.supports(ColorProfile::TrueColor));
    assert!(ColorProfile::TrueColor.supports(ColorProfile::Ansi256));
    assert!(ColorProfile::TrueColor.supports(ColorProfile::Ansi));
    assert!(ColorProfile::TrueColor.supports(ColorProfile::Ascii));

    // Ansi256 supports itself, Ansi, and Ascii
    assert!(!ColorProfile::Ansi256.supports(ColorProfile::TrueColor));
    assert!(ColorProfile::Ansi256.supports(ColorProfile::Ansi256));
    assert!(ColorProfile::Ansi256.supports(ColorProfile::Ansi));
    assert!(ColorProfile::Ansi256.supports(ColorProfile::Ascii));

    // Ansi supports itself and Ascii only
    assert!(!ColorProfile::Ansi.supports(ColorProfile::TrueColor));
    assert!(!ColorProfile::Ansi.supports(ColorProfile::Ansi256));
    assert!(ColorProfile::Ansi.supports(ColorProfile::Ansi));
    assert!(ColorProfile::Ansi.supports(ColorProfile::Ascii));

    // Ascii only supports Ascii
    assert!(!ColorProfile::Ascii.supports(ColorProfile::TrueColor));
    assert!(!ColorProfile::Ascii.supports(ColorProfile::Ansi256));
    assert!(!ColorProfile::Ascii.supports(ColorProfile::Ansi));
    assert!(ColorProfile::Ascii.supports(ColorProfile::Ascii));
}

#[test]
fn color_profile_default_is_truecolor() {
    assert_eq!(ColorProfile::default(), ColorProfile::TrueColor);
}

#[test]
fn renderer_default_is_truecolor_dark() {
    let r = Renderer::new();
    assert_eq!(r.color_profile(), ColorProfile::TrueColor);
    assert!(r.has_dark_background());
}

#[test]
fn renderer_detect_returns_valid_profile() {
    let r = Renderer::detect();
    // Whatever the environment, the profile should be one of the four
    let valid = matches!(
        r.color_profile(),
        ColorProfile::TrueColor | ColorProfile::Ansi256 | ColorProfile::Ansi | ColorProfile::Ascii
    );
    assert!(
        valid,
        "Detected profile should be valid: {:?}",
        r.color_profile()
    );
}

#[test]
fn renderer_for_writer_returns_valid_profile() {
    let buf = Vec::new();
    let r = Renderer::for_writer(buf);
    let valid = matches!(
        r.color_profile(),
        ColorProfile::TrueColor | ColorProfile::Ansi256 | ColorProfile::Ansi | ColorProfile::Ascii
    );
    assert!(valid);
}

// ===========================================================================
// 2. Graceful Degradation: Color Rendering Across Profiles
// ===========================================================================

#[test]
fn hex_color_degrades_correctly_across_all_profiles() {
    let color = Color::from("#ff8800");

    // TrueColor: full RGB
    let tc = color.to_ansi_fg(ColorProfile::TrueColor, true);
    assert!(tc.contains("38;2;255;136;0"), "TrueColor RGB: {tc:?}");

    // Ansi256: 256-color index
    let a256 = color.to_ansi_fg(ColorProfile::Ansi256, true);
    assert!(a256.starts_with("\x1b[38;5;"), "Ansi256 format: {a256:?}");
    assert!(a256.ends_with('m'));

    // Ansi: 16-color code
    let a16 = color.to_ansi_fg(ColorProfile::Ansi, true);
    let is_valid_16 = (30..=37).any(|n| a16 == format!("\x1b[{n}m"))
        || (90..=97).any(|n| a16 == format!("\x1b[{n}m"));
    assert!(is_valid_16, "Ansi 16-color: {a16:?}");

    // Ascii: empty
    let ascii = color.to_ansi_fg(ColorProfile::Ascii, true);
    assert!(ascii.is_empty(), "Ascii produces no color: {ascii:?}");
}

#[test]
fn hex_color_bg_degrades_correctly_across_all_profiles() {
    let color = Color::from("#00cc88");

    let tc = color.to_ansi_bg(ColorProfile::TrueColor, true);
    assert!(tc.contains("48;2;0;204;136"), "TrueColor bg: {tc:?}");

    let a256 = color.to_ansi_bg(ColorProfile::Ansi256, true);
    assert!(a256.starts_with("\x1b[48;5;"), "Ansi256 bg: {a256:?}");

    let a16 = color.to_ansi_bg(ColorProfile::Ansi, true);
    let is_valid_16 = (40..=47).any(|n| a16 == format!("\x1b[{n}m"))
        || (100..=107).any(|n| a16 == format!("\x1b[{n}m"));
    assert!(is_valid_16, "Ansi 16 bg: {a16:?}");

    let ascii = color.to_ansi_bg(ColorProfile::Ascii, true);
    assert!(ascii.is_empty());
}

#[test]
fn rgb_color_degrades_correctly() {
    let color = RgbColor::new(128, 64, 255);

    let tc = color.to_ansi_fg(ColorProfile::TrueColor, true);
    assert_eq!(tc, "\x1b[38;2;128;64;255m");

    let a256 = color.to_ansi_fg(ColorProfile::Ansi256, true);
    assert!(a256.starts_with("\x1b[38;5;"));

    let a16 = color.to_ansi_fg(ColorProfile::Ansi, true);
    let is_16 = (30..=37).any(|n| a16 == format!("\x1b[{n}m"))
        || (90..=97).any(|n| a16 == format!("\x1b[{n}m"));
    assert!(is_16, "Ansi 16: {a16:?}");

    assert!(color.to_ansi_fg(ColorProfile::Ascii, true).is_empty());
    assert!(color.to_ansi_bg(ColorProfile::Ascii, true).is_empty());
}

#[test]
fn ansi_color_type_degrades_correctly() {
    let color = AnsiColor(196);

    let a256 = color.to_ansi_fg(ColorProfile::Ansi256, true);
    assert_eq!(a256, "\x1b[38;5;196m");

    let a16 = color.to_ansi_fg(ColorProfile::Ansi, true);
    // 196 is in the 256-color range; should map down to a 16-color code
    let is_16 = (30..=37).any(|n| a16 == format!("\x1b[{n}m"))
        || (90..=97).any(|n| a16 == format!("\x1b[{n}m"));
    assert!(is_16, "AnsiColor 196 degrades to 16: {a16:?}");

    assert!(color.to_ansi_fg(ColorProfile::Ascii, true).is_empty());
}

#[test]
fn nocolor_produces_empty_on_all_profiles() {
    let nc = NoColor;
    for profile in ALL_PROFILES {
        assert_eq!(nc.to_ansi_fg(profile, true), "", "fg {profile:?}");
        assert_eq!(nc.to_ansi_fg(profile, false), "", "fg light {profile:?}");
        assert_eq!(nc.to_ansi_bg(profile, true), "", "bg {profile:?}");
        assert_eq!(nc.to_ansi_bg(profile, false), "", "bg light {profile:?}");
    }
}

// ===========================================================================
// 3. ASCII-Only / Dumb Terminal Fallback
// ===========================================================================

#[test]
fn ascii_profile_style_produces_no_color_codes() {
    let s = Style::new()
        .renderer(renderer(ColorProfile::Ascii, true))
        .foreground("#ff0000")
        .background("#0000ff")
        .render("text");
    // No color sequences
    assert!(!s.contains("\x1b[38;"), "No fg color: {s:?}");
    assert!(!s.contains("\x1b[48;"), "No bg color: {s:?}");
}

#[test]
fn ascii_profile_preserves_text_content() {
    let s = Style::new()
        .renderer(renderer(ColorProfile::Ascii, true))
        .foreground("#ff0000")
        .render("hello world");
    let plain = strip_ansi(&s);
    assert_eq!(plain, "hello world");
}

#[test]
fn ascii_profile_text_attributes_still_work() {
    // Bold/italic are SGR attributes, separate from color profile.
    // They should still be emitted even with Ascii profile.
    let s = Style::new()
        .renderer(renderer(ColorProfile::Ascii, true))
        .bold()
        .render("bold");
    // Bold is SGR 1 — should still be present
    assert!(
        s.contains("\x1b[1m"),
        "Bold SGR still present with Ascii profile: {s:?}"
    );
}

#[test]
fn ascii_profile_multiline_content_preserved() {
    let s = Style::new()
        .renderer(renderer(ColorProfile::Ascii, true))
        .foreground("#ff0000")
        .render("line1\nline2\nline3");
    let plain = strip_ansi(&s);
    assert_eq!(plain, "line1\nline2\nline3");
}

// ===========================================================================
// 4. 16-Color Fallback
// ===========================================================================

#[test]
fn ansi16_fg_codes_in_valid_range() {
    // Standard colors (0-7) produce codes 30-37
    for n in 0..8u8 {
        let c = Color::from(n.to_string().as_str());
        let seq = c.to_ansi_fg(ColorProfile::Ansi, true);
        assert_eq!(seq, format!("\x1b[{}m", 30 + n), "Std fg {n}");
    }
    // Bright colors (8-15) produce codes 90-97
    for n in 8..16u8 {
        let c = Color::from(n.to_string().as_str());
        let seq = c.to_ansi_fg(ColorProfile::Ansi, true);
        assert_eq!(seq, format!("\x1b[{}m", 90 + n - 8), "Bright fg {n}");
    }
}

#[test]
fn ansi16_bg_codes_in_valid_range() {
    // Standard backgrounds (0-7) produce codes 40-47
    for n in 0..8u8 {
        let c = Color::from(n.to_string().as_str());
        let seq = c.to_ansi_bg(ColorProfile::Ansi, true);
        assert_eq!(seq, format!("\x1b[{}m", 40 + n), "Std bg {n}");
    }
    // Bright backgrounds (8-15) produce codes 100-107
    for n in 8..16u8 {
        let c = Color::from(n.to_string().as_str());
        let seq = c.to_ansi_bg(ColorProfile::Ansi, true);
        assert_eq!(seq, format!("\x1b[{}m", 100 + n - 8), "Bright bg {n}");
    }
}

#[test]
fn ansi16_hex_color_always_maps_to_valid_16_range() {
    // Various hex colors should all map to valid 16-color codes
    let hex_colors = [
        "#ff0000", "#00ff00", "#0000ff", "#ffffff", "#000000", "#ffff00", "#ff00ff", "#00ffff",
        "#808080", "#c0c0c0",
    ];
    for hex in hex_colors {
        let c = Color::from(hex);
        let fg = c.to_ansi_fg(ColorProfile::Ansi, true);
        let valid_fg = (30..=37).any(|n| fg == format!("\x1b[{n}m"))
            || (90..=97).any(|n| fg == format!("\x1b[{n}m"));
        assert!(valid_fg, "Hex {hex} → valid 16-color fg: {fg:?}");

        let bg = c.to_ansi_bg(ColorProfile::Ansi, true);
        let bg_ok = (40..=47).any(|n| bg == format!("\x1b[{n}m"))
            || (100..=107).any(|n| bg == format!("\x1b[{n}m"));
        assert!(bg_ok, "Hex {hex} → valid 16-color bg: {bg:?}");
    }
}

#[test]
fn ansi256_color_downgrades_to_ansi16() {
    // A 256-color value (e.g. 196) should map to a valid 16-color when forced
    let c = Color::from("196");
    let seq = c.to_ansi_fg(ColorProfile::Ansi, true);
    let is_16 = (30..=37).any(|n| seq == format!("\x1b[{n}m"))
        || (90..=97).any(|n| seq == format!("\x1b[{n}m"));
    assert!(is_16, "256-color 196 maps to valid 16: {seq:?}");
}

// ===========================================================================
// 5. Adaptive Color: Dark vs Light Background
// ===========================================================================

#[test]
fn adaptive_color_selects_dark_on_dark_bg() {
    let ac = AdaptiveColor {
        light: Color::from("#000000"),
        dark: Color::from("#ffffff"),
    };
    let fg = ac.to_ansi_fg(ColorProfile::TrueColor, true);
    assert_eq!(fg, "\x1b[38;2;255;255;255m", "Dark bg → white");
}

#[test]
fn adaptive_color_selects_light_on_light_bg() {
    let ac = AdaptiveColor {
        light: Color::from("#000000"),
        dark: Color::from("#ffffff"),
    };
    let fg = ac.to_ansi_fg(ColorProfile::TrueColor, false);
    assert_eq!(fg, "\x1b[38;2;0;0;0m", "Light bg → black");
}

#[test]
fn adaptive_color_respects_color_profile() {
    let ac = AdaptiveColor {
        light: Color::from("#ff0000"),
        dark: Color::from("#00ff00"),
    };

    // Ascii → empty regardless of bg
    assert!(ac.to_ansi_fg(ColorProfile::Ascii, true).is_empty());
    assert!(ac.to_ansi_fg(ColorProfile::Ascii, false).is_empty());

    // Ansi → 16-color
    let dark_ansi = ac.to_ansi_fg(ColorProfile::Ansi, true);
    let is_16 = (30..=37).any(|n| dark_ansi == format!("\x1b[{n}m"))
        || (90..=97).any(|n| dark_ansi == format!("\x1b[{n}m"));
    assert!(is_16, "Adaptive Ansi: {dark_ansi:?}");
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
// 6. CompleteColor: Profile-Specific Colors
// ===========================================================================

#[test]
fn complete_color_selects_correct_profile_field() {
    let cc = CompleteColor {
        truecolor: Some(Color::from("#abcdef")),
        ansi256: Some(Color::from("42")),
        ansi: Some(Color::from("2")),
    };

    let tc = cc.to_ansi_fg(ColorProfile::TrueColor, true);
    assert_eq!(tc, "\x1b[38;2;171;205;239m");

    let a256 = cc.to_ansi_fg(ColorProfile::Ansi256, true);
    assert_eq!(a256, "\x1b[38;5;42m");

    let a16 = cc.to_ansi_fg(ColorProfile::Ansi, true);
    assert_eq!(a16, "\x1b[32m"); // color 2 = green = \x1b[32m

    let ascii = cc.to_ansi_fg(ColorProfile::Ascii, true);
    assert!(ascii.is_empty());
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
    assert!(
        cc.to_ansi_fg(ColorProfile::TrueColor, true)
            .contains("38;2;255;0;0")
    );
}

#[test]
fn complete_color_bg_selects_correctly() {
    let cc = CompleteColor {
        truecolor: Some(Color::from("#112233")),
        ansi256: Some(Color::from("100")),
        ansi: Some(Color::from("4")),
    };

    let tc = cc.to_ansi_bg(ColorProfile::TrueColor, true);
    assert!(tc.contains("48;2;17;34;51"), "TC bg: {tc:?}");

    let a256 = cc.to_ansi_bg(ColorProfile::Ansi256, true);
    assert_eq!(a256, "\x1b[48;5;100m");

    let a16 = cc.to_ansi_bg(ColorProfile::Ansi, true);
    assert_eq!(a16, "\x1b[44m"); // color 4 bg
}

// ===========================================================================
// 7. CompleteAdaptiveColor: Profile + Background Sensitivity
// ===========================================================================

#[test]
fn complete_adaptive_color_combines_profile_and_bg() {
    let cac = CompleteAdaptiveColor {
        light: CompleteColor {
            truecolor: Some(Color::from("#111111")),
            ansi256: Some(Color::from("16")),
            ansi: Some(Color::from("0")),
        },
        dark: CompleteColor {
            truecolor: Some(Color::from("#eeeeee")),
            ansi256: Some(Color::from("255")),
            ansi: Some(Color::from("15")),
        },
    };

    // Dark bg → uses dark CompleteColor → TrueColor field
    let dark_tc = cac.to_ansi_fg(ColorProfile::TrueColor, true);
    assert!(
        dark_tc.contains("238;238;238"),
        "Dark TrueColor: {dark_tc:?}"
    );

    // Light bg → uses light CompleteColor → TrueColor field
    let light_tc = cac.to_ansi_fg(ColorProfile::TrueColor, false);
    assert!(
        light_tc.contains("17;17;17"),
        "Light TrueColor: {light_tc:?}"
    );

    // Dark bg → uses dark CompleteColor → Ansi256 field
    let dark_256 = cac.to_ansi_fg(ColorProfile::Ansi256, true);
    assert_eq!(dark_256, "\x1b[38;5;255m");

    // Light bg → uses light CompleteColor → Ansi field
    let light_16 = cac.to_ansi_fg(ColorProfile::Ansi, false);
    assert_eq!(light_16, "\x1b[30m"); // color 0 = black

    // Ascii → always empty
    assert!(cac.to_ansi_fg(ColorProfile::Ascii, true).is_empty());
    assert!(cac.to_ansi_fg(ColorProfile::Ascii, false).is_empty());
}

// ===========================================================================
// 8. Backend Abstraction: ANSI vs Plain vs HTML
// ===========================================================================

#[test]
fn ansi_backend_produces_sgr_codes() {
    let b = AnsiBackend::new();
    assert_eq!(b.apply_bold("x"), "\x1b[1mx\x1b[0m");
    assert_eq!(b.apply_italic("x"), "\x1b[3mx\x1b[0m");
    assert_eq!(b.apply_underline("x"), "\x1b[4mx\x1b[0m");
    assert_eq!(b.apply_faint("x"), "\x1b[2mx\x1b[0m");
    assert_eq!(b.apply_blink("x"), "\x1b[5mx\x1b[0m");
    assert_eq!(b.apply_reverse("x"), "\x1b[7mx\x1b[0m");
    assert_eq!(b.apply_strikethrough("x"), "\x1b[9mx\x1b[0m");
}

#[test]
fn plain_backend_strips_all_styling() {
    let b = PlainBackend::new();
    assert_eq!(b.apply_bold("text"), "text");
    assert_eq!(b.apply_italic("text"), "text");
    assert_eq!(b.apply_underline("text"), "text");
    assert_eq!(b.apply_faint("text"), "text");
    assert_eq!(b.apply_blink("text"), "text");
    assert_eq!(b.apply_reverse("text"), "text");
    assert_eq!(b.apply_strikethrough("text"), "text");
}

#[test]
fn html_backend_produces_css_styling() {
    let b = HtmlBackend::new();
    let bold = b.apply_bold("text");
    assert!(bold.contains("<span"), "HTML span: {bold:?}");
    assert!(bold.contains("font-weight: bold"), "CSS bold: {bold:?}");
    assert!(bold.contains("text"), "Content preserved");

    let italic = b.apply_italic("text");
    assert!(
        italic.contains("font-style: italic"),
        "CSS italic: {italic:?}"
    );

    let underline = b.apply_underline("text");
    assert!(
        underline.contains("text-decoration: underline"),
        "CSS underline: {underline:?}"
    );
}

#[test]
fn all_backends_preserve_text_content() {
    let ansi = AnsiBackend::new();
    let plain = PlainBackend::new();
    let html = HtmlBackend::new();

    let text = "Hello, World!";

    let ansi_out = ansi.apply_bold(text);
    let plain_out = plain.apply_bold(text);
    let html_out = html.apply_bold(text);

    assert_eq!(ansi.strip_markup(&ansi_out), text);
    assert_eq!(plain.strip_markup(&plain_out), text);
    assert_eq!(html.strip_markup(&html_out), text);
}

#[test]
fn ansi_backend_reset_is_sgr0() {
    let b = AnsiBackend::new();
    assert_eq!(b.reset(), "\x1b[0m");
}

#[test]
fn plain_backend_reset_is_empty() {
    let b = PlainBackend::new();
    assert_eq!(b.reset(), "");
}

#[test]
fn html_backend_reset_is_empty() {
    let b = HtmlBackend::new();
    assert_eq!(b.reset(), "");
}

#[test]
fn ansi_backend_supports_all_profiles() {
    let b = AnsiBackend::new();
    for profile in ALL_PROFILES {
        assert!(b.supports_color(profile), "ANSI supports {profile:?}");
    }
}

#[test]
fn plain_backend_supports_no_profiles() {
    let b = PlainBackend::new();
    for profile in ALL_PROFILES {
        assert!(
            !b.supports_color(profile),
            "Plain doesn't support {profile:?}"
        );
    }
}

#[test]
fn html_backend_supports_all_profiles() {
    let b = HtmlBackend::new();
    for profile in ALL_PROFILES {
        assert!(b.supports_color(profile), "HTML supports {profile:?}");
    }
}

// ===========================================================================
// 9. Backend Width Measurement
// ===========================================================================

#[test]
fn ansi_backend_measure_width_ignores_escape_codes() {
    let b = AnsiBackend::new();
    assert_eq!(b.measure_width("hello"), 5);
    assert_eq!(b.measure_width("\x1b[1mhello\x1b[0m"), 5);
    assert_eq!(b.measure_width("\x1b[38;2;255;0;0mred\x1b[0m"), 3);
}

#[test]
fn plain_backend_measure_width_counts_unicode() {
    let b = PlainBackend::new();
    assert_eq!(b.measure_width("hello"), 5);
    assert_eq!(b.measure_width(""), 0);
    // CJK characters are 2 cells wide
    assert_eq!(b.measure_width("你好"), 4);
}

#[test]
fn html_backend_measure_width_strips_tags() {
    let b = HtmlBackend::new();
    let html = r#"<span style="font-weight: bold">hello</span>"#;
    assert_eq!(b.measure_width(html), 5);
}

#[test]
fn all_backends_agree_on_plain_text_width() {
    let ansi = AnsiBackend::new();
    let plain = PlainBackend::new();
    let html = HtmlBackend::new();

    let text = "Hello World";
    assert_eq!(ansi.measure_width(text), 11);
    assert_eq!(plain.measure_width(text), 11);
    assert_eq!(html.measure_width(text), 11);
}

#[test]
fn all_backends_agree_on_unicode_width() {
    let ansi = AnsiBackend::new();
    let plain = PlainBackend::new();
    let html = HtmlBackend::new();

    let text = "日本語";
    // 3 CJK characters * 2 width = 6
    assert_eq!(ansi.measure_width(text), 6);
    assert_eq!(plain.measure_width(text), 6);
    assert_eq!(html.measure_width(text), 6);
}

// ===========================================================================
// 10. Backend Newline Handling
// ===========================================================================

#[test]
fn ansi_backend_newline_is_lf() {
    assert_eq!(AnsiBackend::new().newline(), "\n");
}

#[test]
fn plain_backend_newline_is_lf() {
    assert_eq!(PlainBackend::new().newline(), "\n");
}

#[test]
fn html_backend_newline_is_br() {
    assert_eq!(HtmlBackend::new().newline(), "<br>");
}

// ===========================================================================
// 11. Backend Strip Markup
// ===========================================================================

#[test]
fn ansi_backend_strip_removes_escape_codes() {
    let b = AnsiBackend::new();
    let styled = "\x1b[1m\x1b[38;2;255;0;0mhello\x1b[0m";
    assert_eq!(b.strip_markup(styled), "hello");
}

#[test]
fn plain_backend_strip_is_identity() {
    let b = PlainBackend::new();
    assert_eq!(b.strip_markup("hello"), "hello");
    assert_eq!(b.strip_markup(""), "");
}

#[test]
fn html_backend_strip_removes_tags() {
    let b = HtmlBackend::new();
    let html = r#"<span style="color: red">hello</span>"#;
    assert_eq!(b.strip_markup(html), "hello");
}

// ===========================================================================
// 12. HTML Backend Specifics
// ===========================================================================

#[test]
fn html_backend_inline_styles_by_default() {
    let b = HtmlBackend::new();
    assert!(b.use_inline_styles);
    let result = b.apply_bold("text");
    assert!(result.contains("style="), "Inline style: {result:?}");
    assert!(
        !result.contains("class="),
        "No class when inline: {result:?}"
    );
}

#[test]
fn html_backend_css_classes_mode() {
    let mut b = HtmlBackend::new();
    b.use_inline_styles = false;
    let result = b.apply_bold("text");
    assert!(result.contains("class="), "CSS class: {result:?}");
    assert!(!result.contains("style="), "No inline style: {result:?}");
}

#[test]
fn html_backend_escapes_special_chars() {
    let b = HtmlBackend::new();
    let style = Style::new().bold();
    let result = b.render("<script>alert('xss')</script>", &style);
    assert!(!result.contains("<script>"), "HTML escaped: {result:?}");
    assert!(result.contains("&lt;"), "< escaped: {result:?}");
    assert!(result.contains("&gt;"), "> escaped: {result:?}");
}

#[test]
fn html_backend_br_for_line_breaks() {
    let b = HtmlBackend::new();
    let style = Style::new();
    let result = b.render("line1\nline2", &style);
    assert!(result.contains("<br>"), "Newline → <br>: {result:?}");
}

// ===========================================================================
// 13. Renderer Integration with Styles
// ===========================================================================

#[test]
fn style_uses_renderer_profile_for_color_degradation() {
    let text = "test";
    let hex = "#ff0000";

    // TrueColor renderer → RGB sequence
    let tc_style = Style::new()
        .renderer(renderer(ColorProfile::TrueColor, true))
        .foreground(hex)
        .render(text);
    assert!(tc_style.contains("\x1b[38;2;"), "TrueColor: {tc_style:?}");

    // Ansi256 renderer → 256-color sequence
    let a256_style = Style::new()
        .renderer(renderer(ColorProfile::Ansi256, true))
        .foreground(hex)
        .render(text);
    assert!(a256_style.contains("\x1b[38;5;"), "Ansi256: {a256_style:?}");
    assert!(
        !a256_style.contains("\x1b[38;2;"),
        "Not TrueColor: {a256_style:?}"
    );

    // Ansi renderer → 16-color
    let a16_style = Style::new()
        .renderer(renderer(ColorProfile::Ansi, true))
        .foreground(hex)
        .render(text);
    assert!(!a16_style.contains("\x1b[38;5;"), "Not 256: {a16_style:?}");
    assert!(
        !a16_style.contains("\x1b[38;2;"),
        "Not TrueColor: {a16_style:?}"
    );
    // Should be in 30-37 or 90-97 range
    let has_16 = (30..=37).any(|n| a16_style.contains(&format!("\x1b[{n}m")))
        || (90..=97).any(|n| a16_style.contains(&format!("\x1b[{n}m")));
    assert!(has_16, "Ansi 16: {a16_style:?}");
}

#[test]
fn style_renderer_dark_bg_affects_adaptive_color_selection() {
    let dark_r = renderer(ColorProfile::TrueColor, true);
    let light_r = renderer(ColorProfile::TrueColor, false);

    let ac = AdaptiveColor {
        light: Color::from("#111111"),
        dark: Color::from("#eeeeee"),
    };

    let dark_fg = ac.to_ansi_fg(dark_r.color_profile(), dark_r.has_dark_background());
    let light_fg = ac.to_ansi_fg(light_r.color_profile(), light_r.has_dark_background());

    assert_ne!(dark_fg, light_fg);
    assert!(
        dark_fg.contains("238;238;238"),
        "Dark → light color: {dark_fg:?}"
    );
    assert!(
        light_fg.contains("17;17;17"),
        "Light → dark color: {light_fg:?}"
    );
}

#[test]
fn renderer_clone_is_independent() {
    let mut r1 = Renderer::new();
    r1.set_color_profile(ColorProfile::Ansi256);
    r1.set_has_dark_background(false);

    let r2 = r1.clone();
    r1.set_color_profile(ColorProfile::Ascii);

    assert_eq!(
        r2.color_profile(),
        ColorProfile::Ansi256,
        "Clone not affected"
    );
    assert_eq!(r1.color_profile(), ColorProfile::Ascii, "Original changed");
}

// ===========================================================================
// 14. Color Conversion Consistency
// ===========================================================================

#[test]
fn rgb_to_ansi256_pure_colors() {
    let red = lipgloss::color::rgb_to_ansi256(255, 0, 0);
    assert!((196..=197).contains(&red), "Pure red ~196: {red}");

    let green = lipgloss::color::rgb_to_ansi256(0, 255, 0);
    assert!((46..=47).contains(&green), "Pure green ~46: {green}");

    let blue = lipgloss::color::rgb_to_ansi256(0, 0, 255);
    assert!((21..=22).contains(&blue), "Pure blue ~21: {blue}");
}

#[test]
fn rgb_to_ansi256_grayscale() {
    // Black
    let black = lipgloss::color::rgb_to_ansi256(0, 0, 0);
    assert_eq!(black, 16, "Black → 16");

    // White
    let white = lipgloss::color::rgb_to_ansi256(255, 255, 255);
    assert_eq!(white, 231, "White → 231");

    // Mid-gray
    let gray = lipgloss::color::rgb_to_ansi256(128, 128, 128);
    assert!(gray >= 232, "Mid-gray in grayscale range: {gray}");
}

#[test]
fn ansi256_to_rgb_standard_colors_roundtrip() {
    // Standard 16 colors have well-defined RGB values
    assert_eq!(lipgloss::color::ansi256_to_rgb(0), (0, 0, 0)); // Black
    assert_eq!(lipgloss::color::ansi256_to_rgb(1), (128, 0, 0)); // Red
    assert_eq!(lipgloss::color::ansi256_to_rgb(2), (0, 128, 0)); // Green
    assert_eq!(lipgloss::color::ansi256_to_rgb(7), (192, 192, 192)); // White
    assert_eq!(lipgloss::color::ansi256_to_rgb(9), (255, 0, 0)); // Bright Red
    assert_eq!(lipgloss::color::ansi256_to_rgb(15), (255, 255, 255)); // Bright White
}

#[test]
fn ansi256_to_rgb_grayscale_ramp() {
    // Grayscale range 232-255 should be monotonically increasing
    let mut prev = 0u8;
    for n in 232..=255u8 {
        let (r, g, b) = lipgloss::color::ansi256_to_rgb(n);
        assert_eq!(r, g, "Grayscale: r==g for {n}");
        assert_eq!(g, b, "Grayscale: g==b for {n}");
        assert!(r >= prev, "Monotonic: {n} ({r} >= {prev})");
        prev = r;
    }
}

#[test]
fn rgb_to_ansi16_always_in_range() {
    // Test a variety of RGB values
    let test_colors: [(u8, u8, u8); 8] = [
        (0, 0, 0),
        (255, 255, 255),
        (255, 0, 0),
        (0, 255, 0),
        (0, 0, 255),
        (128, 128, 128),
        (255, 128, 0),
        (64, 32, 128),
    ];
    for (r, g, b) in test_colors {
        let n = lipgloss::color::rgb_to_ansi16(r, g, b);
        assert!(n < 16, "({r},{g},{b}) → {n} should be < 16");
    }
}

// ===========================================================================
// 15. Cross-Profile Content Preservation
// ===========================================================================

#[test]
fn same_content_across_all_profiles() {
    let text = "Hello, Terminal World! 🦀";
    for profile in ALL_PROFILES {
        let s = Style::new()
            .renderer(renderer(profile, true))
            .bold()
            .foreground("#ff0000")
            .render(text);
        let plain = strip_ansi(&s);
        assert_eq!(plain, text, "Content preserved for {profile:?}");
    }
}

#[test]
fn multiline_content_across_all_profiles() {
    let text = "line1\nline2\nline3";
    for profile in ALL_PROFILES {
        let s = Style::new()
            .renderer(renderer(profile, true))
            .foreground("#00ff00")
            .render(text);
        let plain = strip_ansi(&s);
        assert_eq!(plain, text, "Multiline preserved for {profile:?}");
    }
}

#[test]
fn unicode_content_across_all_profiles() {
    let text = "日本語テスト café résumé";
    for profile in ALL_PROFILES {
        let s = Style::new()
            .renderer(renderer(profile, true))
            .italic()
            .render(text);
        let plain = strip_ansi(&s);
        assert_eq!(plain, text, "Unicode preserved for {profile:?}");
    }
}

// ===========================================================================
// 16. Simulated Terminal Environments
// ===========================================================================

/// Simulate "xterm-256color" by using Ansi256 profile.
#[test]
fn simulated_xterm_256color() {
    let r = renderer(ColorProfile::Ansi256, true);
    let s = Style::new()
        .renderer(r)
        .foreground("#ff8800")
        .bold()
        .render("xterm-256color test");

    // Should use 256-color, not truecolor
    assert!(s.contains("\x1b[38;5;"), "256-color format");
    assert!(!s.contains("\x1b[38;2;"), "Not truecolor");
    // Bold SGR still present
    assert!(s.contains("\x1b[1m"), "Bold present");
    assert_eq!(strip_ansi(&s), "xterm-256color test");
}

/// Simulate "xterm" (basic 16 colors) by using Ansi profile.
#[test]
fn simulated_xterm_16color() {
    let r = renderer(ColorProfile::Ansi, true);
    let s = Style::new()
        .renderer(r)
        .foreground("#ff0000")
        .background("#00ff00")
        .render("xterm test");

    // Should only have 16-color codes
    assert!(!s.contains("\x1b[38;2;"), "No truecolor fg");
    assert!(!s.contains("\x1b[38;5;"), "No 256 fg");
    assert!(!s.contains("\x1b[48;2;"), "No truecolor bg");
    assert!(!s.contains("\x1b[48;5;"), "No 256 bg");
    // Should have some color code
    let has_fg = (30..=37).any(|n| s.contains(&format!("\x1b[{n}m")))
        || (90..=97).any(|n| s.contains(&format!("\x1b[{n}m")));
    assert!(has_fg, "16-color fg present");
}

/// Simulate "dumb" terminal by using Ascii profile.
#[test]
fn simulated_dumb_terminal() {
    let r = renderer(ColorProfile::Ascii, true);
    let s = Style::new()
        .renderer(r)
        .foreground("#ff0000")
        .background("#0000ff")
        .render("dumb terminal");

    // No color sequences at all
    assert!(!s.contains("\x1b[38;"), "No fg color codes");
    assert!(!s.contains("\x1b[48;"), "No bg color codes");
    assert_eq!(strip_ansi(&s), "dumb terminal");
}

/// Simulate light-background terminal (e.g., macOS Terminal.app default).
#[test]
fn simulated_light_background_terminal() {
    let r = renderer(ColorProfile::TrueColor, false);
    let ac = AdaptiveColor {
        light: Color::from("#333333"), // dark text for light bg
        dark: Color::from("#cccccc"),  // light text for dark bg
    };

    let fg = ac.to_ansi_fg(r.color_profile(), r.has_dark_background());
    // Should pick the light variant (dark text)
    assert!(fg.contains("51;51;51"), "Light bg → dark text: {fg:?}");
}

/// Simulate dark-background terminal (e.g., most modern terminals).
#[test]
fn simulated_dark_background_terminal() {
    let r = renderer(ColorProfile::TrueColor, true);
    let ac = AdaptiveColor {
        light: Color::from("#333333"),
        dark: Color::from("#cccccc"),
    };

    let fg = ac.to_ansi_fg(r.color_profile(), r.has_dark_background());
    assert!(fg.contains("204;204;204"), "Dark bg → light text: {fg:?}");
}

/// Simulate tmux/screen (typically "screen-256color").
#[test]
fn simulated_tmux_screen() {
    // tmux usually advertises 256 colors
    let r = renderer(ColorProfile::Ansi256, true);
    let s = Style::new()
        .renderer(r)
        .foreground("#abcdef")
        .render("tmux session");

    assert!(s.contains("\x1b[38;5;"), "tmux 256-color");
    assert!(!s.contains("\x1b[38;2;"), "Not truecolor in tmux");
}

/// Simulate SSH session (color capability depends on client).
#[test]
fn simulated_ssh_with_256color() {
    let r = renderer(ColorProfile::Ansi256, true);
    let s = Style::new()
        .renderer(r)
        .foreground("#ff0000")
        .bold()
        .underline()
        .render("SSH remote");

    assert!(s.contains("\x1b[38;5;"), "SSH 256-color");
    assert!(s.contains("\x1b[1m"), "Bold over SSH");
    assert!(s.contains("\x1b[4m"), "Underline over SSH");
    assert_eq!(strip_ansi(&s), "SSH remote");
}

// ===========================================================================
// 17. NO_COLOR Compliance
// ===========================================================================

#[test]
fn no_color_simulation_ascii_profile_suppresses_all_colors() {
    // NO_COLOR spec: when set, programs should not emit color escape codes
    let r = renderer(ColorProfile::Ascii, true);

    // Test with various color types
    let colors: Vec<Box<dyn TerminalColor>> = vec![
        Box::new(Color::from("#ff0000")),
        Box::new(AnsiColor(196)),
        Box::new(RgbColor::new(255, 0, 0)),
        Box::new(NoColor),
        Box::new(AdaptiveColor {
            light: Color::from("#000000"),
            dark: Color::from("#ffffff"),
        }),
    ];

    for color in &colors {
        let fg = color.to_ansi_fg(r.color_profile(), r.has_dark_background());
        assert!(fg.is_empty(), "NO_COLOR: fg should be empty for {color:?}");
        let bg = color.to_ansi_bg(r.color_profile(), r.has_dark_background());
        assert!(bg.is_empty(), "NO_COLOR: bg should be empty for {color:?}");
    }
}

#[test]
fn no_color_styled_content_has_no_color_sequences() {
    let r = renderer(ColorProfile::Ascii, true);
    let style = Style::new()
        .renderer(r)
        .foreground("#ff0000")
        .background("#00ff00");

    let output = style.render("no color output");
    assert!(!output.contains("\x1b[38;"), "No fg sequences");
    assert!(!output.contains("\x1b[48;"), "No bg sequences");
}

// ===========================================================================
// 18. Style Composition Across Profiles
// ===========================================================================

#[test]
fn cloned_style_works_with_different_renderers() {
    let base = Style::new().bold();

    let tc = base
        .clone()
        .renderer(renderer(ColorProfile::TrueColor, true))
        .foreground("#ff0000")
        .render("true");

    let a256 = base
        .clone()
        .renderer(renderer(ColorProfile::Ansi256, true))
        .foreground("#ff0000")
        .render("256");

    let a16 = base
        .clone()
        .renderer(renderer(ColorProfile::Ansi, true))
        .foreground("#ff0000")
        .render("16");

    let ascii = base
        .renderer(renderer(ColorProfile::Ascii, true))
        .foreground("#ff0000")
        .render("ascii");

    // All have bold
    for (label, s) in [("tc", &tc), ("256", &a256), ("16", &a16), ("ascii", &ascii)] {
        assert!(s.contains("\x1b[1m"), "{label} has bold");
    }

    // Different color formats
    assert!(tc.contains("\x1b[38;2;"), "TC has RGB");
    assert!(a256.contains("\x1b[38;5;"), "256 has indexed");
    assert!(
        !a16.contains("\x1b[38;2;") && !a16.contains("\x1b[38;5;"),
        "16 has basic code"
    );
    assert!(!ascii.contains("\x1b[38;"), "Ascii has no color");
}

// ===========================================================================
// 19. Color Parsing Edge Cases
// ===========================================================================

#[test]
fn color_from_3digit_hex() {
    let c = Color::from("#f0a");
    let (r, g, b) = c.as_rgb().expect("3-digit hex");
    assert_eq!((r, g, b), (255, 0, 170));
}

#[test]
fn color_from_6digit_hex() {
    let c = Color::from("#aabbcc");
    let (r, g, b) = c.as_rgb().expect("6-digit hex");
    assert_eq!((r, g, b), (170, 187, 204));
}

#[test]
fn color_from_ansi_number_string() {
    let c = Color::from("42");
    assert_eq!(c.as_ansi(), Some(42));
    assert!(c.as_rgb().is_none());
}

#[test]
fn color_invalid_string_produces_empty() {
    let c = Color::from("not_a_color");
    let seq = c.to_ansi_fg(ColorProfile::TrueColor, true);
    assert!(seq.is_empty(), "Invalid color → empty: {seq:?}");
}

#[test]
fn color_empty_string_produces_empty() {
    let c = Color::new("");
    let seq = c.to_ansi_fg(ColorProfile::TrueColor, true);
    assert!(seq.is_empty());
}

// ===========================================================================
// 20. ANSI Sequence Well-Formedness Across Profiles
// ===========================================================================

#[test]
fn all_profiles_produce_well_formed_sequences() {
    for profile in ALL_PROFILES {
        let s = Style::new()
            .renderer(renderer(profile, true))
            .bold()
            .italic()
            .foreground("#abcdef")
            .background("#fedcba")
            .render("test\nline2");

        // Verify all escape sequences are properly terminated
        let mut in_seq = false;
        let mut after_esc = false;
        for c in s.chars() {
            if c == '\x1b' {
                after_esc = true;
                continue;
            }
            if after_esc {
                if c == '[' {
                    in_seq = true;
                    after_esc = false;
                    continue;
                }
                after_esc = false;
            }
            if in_seq {
                if ('@'..='~').contains(&c) {
                    in_seq = false;
                } else {
                    assert!(
                        c.is_ascii_digit() || c == ';',
                        "Valid CSI char for {profile:?}: {c:?}"
                    );
                }
            }
        }
        assert!(!in_seq, "No unterminated sequence for {profile:?}");
    }
}

#[test]
fn every_styled_line_ends_with_reset_across_profiles() {
    for profile in ALL_PROFILES {
        let s = Style::new()
            .renderer(renderer(profile, true))
            .bold()
            .foreground("#ff0000")
            .render("a\nb\nc");

        for (i, line) in s.lines().enumerate() {
            if contains_ansi(line) {
                assert!(
                    line.ends_with("\x1b[0m"),
                    "Line {i} ends with reset for {profile:?}: {line:?}"
                );
            }
        }
    }
}
