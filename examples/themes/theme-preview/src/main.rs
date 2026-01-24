//! Theme Preview Example
//!
//! Displays a preview of all built-in themes with color swatches.
//! This is a non-interactive example that prints to stdout.
//!
//! Run with: `cargo run -p example-theme-preview`

#![forbid(unsafe_code)]

use lipgloss::{Border, CatppuccinFlavor, ColorSlot, Style, Theme, ThemePreset};

/// All available theme presets
const PRESETS: &[(&str, ThemePreset)] = &[
    ("Dark (Default)", ThemePreset::Dark),
    ("Light", ThemePreset::Light),
    ("Dracula", ThemePreset::Dracula),
    ("Nord", ThemePreset::Nord),
    (
        "Catppuccin Mocha",
        ThemePreset::Catppuccin(CatppuccinFlavor::Mocha),
    ),
    (
        "Catppuccin Macchiato",
        ThemePreset::Catppuccin(CatppuccinFlavor::Macchiato),
    ),
    (
        "Catppuccin Frappe",
        ThemePreset::Catppuccin(CatppuccinFlavor::Frappe),
    ),
    (
        "Catppuccin Latte",
        ThemePreset::Catppuccin(CatppuccinFlavor::Latte),
    ),
];

/// Semantic color slots to display
const COLOR_SLOTS: &[(ColorSlot, &str)] = &[
    (ColorSlot::Primary, "Primary"),
    (ColorSlot::Secondary, "Secondary"),
    (ColorSlot::Success, "Success"),
    (ColorSlot::Warning, "Warning"),
    (ColorSlot::Error, "Error"),
    (ColorSlot::Info, "Info"),
    (ColorSlot::TextMuted, "Muted"),
];

fn main() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║            lipgloss Theme Preview Gallery                 ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    for (name, preset) in PRESETS {
        let theme = preset.to_theme();
        print_theme_preview(name, &theme);
        println!();
    }

    println!("Use ThemePreset::* to access these themes in your code.");
    println!("See docs/custom-themes.md for creating custom themes.\n");
}

fn print_theme_preview(name: &str, theme: &Theme) {
    // Theme header
    let header_style = Style::from_theme(theme, ColorSlot::Primary).bold();
    let variant = if theme.is_dark() { "dark" } else { "light" };

    println!(
        "{} ({})",
        header_style.render(&format!("═══ {} ", name)),
        variant
    );

    // Color swatches - use foreground_slot for each color
    let mut swatch_line = String::new();
    for (slot, label) in COLOR_SLOTS {
        let style = Style::from_theme(theme, *slot);
        swatch_line.push_str(&format!("{} {:<10} ", style.render("██"), label));
    }
    println!("  {}", swatch_line.trim_end());

    // Sample text
    let text_style = Style::from_theme(theme, ColorSlot::Foreground);
    let muted_style = Style::from_theme(theme, ColorSlot::TextMuted);
    println!(
        "  {} {}",
        text_style.render("Sample text"),
        muted_style.render("(muted)")
    );

    // Sample box - use the _slot methods for theme colors
    let box_style = Style::new()
        .border(Border::rounded())
        .border_foreground_slot(theme, ColorSlot::Border)
        .foreground_slot(theme, ColorSlot::Foreground)
        .padding((0, 1));

    let success_style = Style::from_theme(theme, ColorSlot::Success);
    let error_style = Style::from_theme(theme, ColorSlot::Error);

    let box_content = format!(
        "{} | {}",
        success_style.render("✓ OK"),
        error_style.render("✗ Error")
    );

    println!("  {}", box_style.render(&box_content));
}
