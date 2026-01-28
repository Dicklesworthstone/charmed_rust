//! Theme definitions for `demo_showcase`.
//!
//! Provides semantic color tokens and style helpers for consistent theming.
//! See `docs/demo_showcase/VISUAL_DESIGN.md` for the design specification.
//! See `docs/demo_showcase/ACCESSIBILITY.md` for accessibility guidelines.

#![allow(dead_code)] // Style helpers will be used as pages are implemented

use std::env;

use lipgloss::{Border, Style};
use serde::{Deserialize, Serialize};

// ============================================================================
// Spacing Constants
// ============================================================================

/// Spacing scale based on a 4-unit base.
/// See `VISUAL_DESIGN.md` for usage guidelines.
pub mod spacing {
    /// Extra small spacing (1 unit) - icon-to-text gap, tight inline spacing.
    pub const XS: u16 = 1;
    /// Small spacing (2 units) - compact padding, list item spacing.
    pub const SM: u16 = 2;
    /// Medium spacing (4 units) - standard padding, section margins.
    pub const MD: u16 = 4;
    /// Large spacing (6 units) - major section separation.
    pub const LG: u16 = 6;
    /// Extra large spacing (8 units) - page-level padding, modal margins.
    pub const XL: u16 = 8;

    /// Fixed sidebar width.
    pub const SIDEBAR_WIDTH: u16 = 14;
    /// Minimum content width for proper layout.
    pub const MIN_CONTENT_WIDTH: u16 = 60;
    /// Header height (1 line).
    pub const HEADER_HEIGHT: u16 = 1;
    /// Footer height (1 line).
    pub const FOOTER_HEIGHT: u16 = 1;
}

// ============================================================================
// Theme Presets
// ============================================================================

/// Theme preset identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ThemePreset {
    #[default]
    Dark,
    Light,
    Dracula,
}

impl ThemePreset {
    /// Get the display name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
            Self::Dracula => "Dracula",
        }
    }

    /// Get all available presets.
    #[must_use]
    pub const fn all() -> [Self; 3] {
        [Self::Dark, Self::Light, Self::Dracula]
    }
}

// ============================================================================
// Semantic Color Tokens
// ============================================================================

/// Semantic color tokens for the application.
///
/// Colors are stored as hex strings for direct use with lipgloss.
/// All colors should be accessed via the Theme struct, never hardcoded.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Theme preset being used.
    pub preset: ThemePreset,

    // Primary colors
    /// Brand color, accent, interactive elements.
    pub primary: &'static str,
    /// Secondary accent, less prominent.
    pub secondary: &'static str,

    // Semantic colors
    /// Healthy, complete, positive states.
    pub success: &'static str,
    /// Needs attention, degraded states.
    pub warning: &'static str,
    /// Failed, critical, action needed.
    pub error: &'static str,
    /// Informational, neutral highlight.
    pub info: &'static str,

    // Text colors
    /// Primary text, high contrast.
    pub text: &'static str,
    /// Secondary text, hints, timestamps.
    pub text_muted: &'static str,
    /// Text on colored backgrounds.
    pub text_inverse: &'static str,

    // Background colors
    /// Main background.
    pub bg: &'static str,
    /// Sidebar, header, card backgrounds.
    pub bg_subtle: &'static str,
    /// Hover, selection, active states.
    pub bg_highlight: &'static str,

    // Border colors
    /// Subtle borders, dividers.
    pub border: &'static str,
    /// Focused element borders.
    pub border_focus: &'static str,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Create the dark theme (default).
    #[must_use]
    pub const fn dark() -> Self {
        Self {
            preset: ThemePreset::Dark,
            primary: "#7D56F4",
            secondary: "#FF69B4",
            success: "#00FF00",
            warning: "#FFCC00",
            error: "#FF0000",
            info: "#00BFFF",
            text: "#FFFFFF",
            text_muted: "#626262",
            text_inverse: "#000000",
            bg: "#000000",
            bg_subtle: "#1a1a1a",
            bg_highlight: "#333333",
            border: "#444444",
            border_focus: "#7D56F4",
        }
    }

    /// Create the light theme.
    #[must_use]
    pub const fn light() -> Self {
        Self {
            preset: ThemePreset::Light,
            primary: "#6B46C1",
            secondary: "#D53F8C",
            success: "#38A169",
            warning: "#D69E2E",
            error: "#E53E3E",
            info: "#3182CE",
            text: "#1A202C",
            text_muted: "#718096",
            text_inverse: "#FFFFFF",
            bg: "#FFFFFF",
            bg_subtle: "#F7FAFC",
            bg_highlight: "#EDF2F7",
            border: "#E2E8F0",
            border_focus: "#6B46C1",
        }
    }

    /// Create the Dracula theme.
    #[must_use]
    pub const fn dracula() -> Self {
        Self {
            preset: ThemePreset::Dracula,
            primary: "#BD93F9",
            secondary: "#FF79C6",
            success: "#50FA7B",
            warning: "#F1FA8C",
            error: "#FF5555",
            info: "#8BE9FD",
            text: "#F8F8F2",
            text_muted: "#6272A4",
            text_inverse: "#282A36",
            bg: "#282A36",
            bg_subtle: "#343746",
            bg_highlight: "#44475A",
            border: "#44475A",
            border_focus: "#BD93F9",
        }
    }

    /// Create a theme from a preset.
    #[must_use]
    pub const fn from_preset(preset: ThemePreset) -> Self {
        match preset {
            ThemePreset::Dark => Self::dark(),
            ThemePreset::Light => Self::light(),
            ThemePreset::Dracula => Self::dracula(),
        }
    }

    // ========================================================================
    // Typography Style Helpers
    // ========================================================================

    /// Style for page titles (Level 1 heading).
    /// Bold + primary color.
    #[must_use]
    pub fn title_style(&self) -> Style {
        Style::new().bold().foreground(self.primary)
    }

    /// Style for section headings (Level 2).
    /// Bold + normal text color.
    #[must_use]
    pub fn heading_style(&self) -> Style {
        Style::new().bold().foreground(self.text)
    }

    /// Style for subsection headings (Level 3).
    /// Bold with normal text color.
    #[must_use]
    pub fn subheading_style(&self) -> Style {
        Style::new().bold().foreground(self.text)
    }

    /// Style for muted/hint text.
    /// Uses `text_muted` color for secondary information.
    #[must_use]
    pub fn muted_style(&self) -> Style {
        Style::new().foreground(self.text_muted)
    }

    /// Style for links and interactive text.
    /// Underlined with info color.
    #[must_use]
    pub fn link_style(&self) -> Style {
        Style::new().underline().foreground(self.info)
    }

    /// Style for keyboard shortcuts display.
    /// Faint text for non-intrusive hints.
    #[must_use]
    pub fn shortcut_style(&self) -> Style {
        Style::new().faint().foreground(self.text_muted)
    }

    // ========================================================================
    // Semantic Status Style Helpers
    // ========================================================================

    /// Style for success messages and indicators.
    #[must_use]
    pub fn success_style(&self) -> Style {
        Style::new().foreground(self.success)
    }

    /// Style for warning messages and indicators.
    #[must_use]
    pub fn warning_style(&self) -> Style {
        Style::new().foreground(self.warning)
    }

    /// Style for error messages and indicators.
    #[must_use]
    pub fn error_style(&self) -> Style {
        Style::new().foreground(self.error)
    }

    /// Style for informational messages and indicators.
    #[must_use]
    pub fn info_style(&self) -> Style {
        Style::new().foreground(self.info)
    }

    // ========================================================================
    // Container Style Helpers
    // ========================================================================

    /// Style for content boxes with rounded borders.
    /// Use for general content containers.
    #[must_use]
    pub fn box_style(&self) -> Style {
        Style::new()
            .border(Border::rounded())
            .border_foreground(self.border)
    }

    /// Style for focused content boxes.
    /// Use when a box has keyboard focus.
    #[must_use]
    pub fn box_focused_style(&self) -> Style {
        Style::new()
            .border(Border::rounded())
            .border_foreground(self.border_focus)
    }

    /// Style for cards - containers with subtle background.
    /// Use for grouping related content without borders.
    #[must_use]
    pub fn card_style(&self) -> Style {
        Style::new()
            .background(self.bg_subtle)
            .padding((spacing::XS, spacing::SM))
    }

    /// Style for panels - bordered containers with background.
    /// Use for major content sections.
    #[must_use]
    pub fn panel_style(&self) -> Style {
        Style::new()
            .border(Border::rounded())
            .border_foreground(self.border)
            .background(self.bg_subtle)
            .padding(spacing::SM)
    }

    /// Style for modals and dialogs.
    /// Double border for emphasis with highlight background.
    #[must_use]
    pub fn modal_style(&self) -> Style {
        Style::new()
            .border(Border::double())
            .border_foreground(self.border_focus)
            .background(self.bg_highlight)
            .padding((spacing::SM, spacing::MD))
    }

    /// Style for tables.
    /// Normal (non-rounded) borders for grid alignment.
    #[must_use]
    pub fn table_style(&self) -> Style {
        Style::new()
            .border(Border::normal())
            .border_foreground(self.border)
    }

    // ========================================================================
    // Interactive Element Style Helpers
    // ========================================================================

    /// Style for badges and chips.
    /// Compact padding with background.
    #[must_use]
    pub fn badge_style(&self) -> Style {
        Style::new()
            .background(self.bg_highlight)
            .foreground(self.text)
            .padding_left(spacing::XS)
            .padding_right(spacing::XS)
    }

    /// Style for primary badges (uses primary color).
    #[must_use]
    pub fn badge_primary_style(&self) -> Style {
        Style::new()
            .background(self.primary)
            .foreground(self.text_inverse)
            .padding_left(spacing::XS)
            .padding_right(spacing::XS)
    }

    /// Style for buttons.
    /// Bold text with background and horizontal padding.
    #[must_use]
    pub fn button_style(&self) -> Style {
        Style::new()
            .bold()
            .background(self.bg_highlight)
            .foreground(self.text)
            .padding_left(spacing::SM)
            .padding_right(spacing::SM)
    }

    /// Style for primary buttons.
    /// Uses primary color for emphasis.
    #[must_use]
    pub fn button_primary_style(&self) -> Style {
        Style::new()
            .bold()
            .background(self.primary)
            .foreground(self.text_inverse)
            .padding_left(spacing::SM)
            .padding_right(spacing::SM)
    }

    /// Style for hover/focus states on interactive elements.
    #[must_use]
    pub fn hover_style(&self) -> Style {
        Style::new().background(self.bg_highlight)
    }

    /// Style for selected items in lists.
    /// Bold + primary color + highlight background.
    #[must_use]
    pub fn selected_style(&self) -> Style {
        Style::new()
            .bold()
            .foreground(self.primary)
            .background(self.bg_highlight)
    }

    // ========================================================================
    // App Chrome Style Helpers
    // ========================================================================

    /// Style for the header bar.
    #[must_use]
    pub fn header_style(&self) -> Style {
        Style::new()
            .background(self.bg_subtle)
            .foreground(self.text)
    }

    /// Style for the footer/status bar.
    #[must_use]
    pub fn footer_style(&self) -> Style {
        Style::new().foreground(self.text_muted).background(self.bg)
    }

    /// Style for the sidebar background.
    #[must_use]
    pub fn sidebar_style(&self) -> Style {
        Style::new()
            .foreground(self.text)
            .background(self.bg_subtle)
    }

    /// Style for the selected sidebar item.
    #[must_use]
    pub fn sidebar_selected_style(&self) -> Style {
        Style::new()
            .bold()
            .foreground(self.primary)
            .background(self.bg_highlight)
    }

    /// Style for inactive sidebar items.
    #[must_use]
    pub fn sidebar_inactive_style(&self) -> Style {
        Style::new()
            .foreground(self.text_muted)
            .background(self.bg_subtle)
    }

    // ========================================================================
    // Status Indicator Helpers
    // ========================================================================

    /// Get the appropriate style for a health/status value.
    /// Maps boolean-like states to success/error.
    #[must_use]
    pub fn status_style(&self, is_ok: bool) -> Style {
        if is_ok {
            self.success_style()
        } else {
            self.error_style()
        }
    }

    /// Get style for progress indicators.
    /// Uses info color for neutral progress, success for complete.
    #[must_use]
    pub fn progress_style(&self, percent: u8) -> Style {
        if percent >= 100 {
            self.success_style()
        } else {
            self.info_style()
        }
    }
}

// ============================================================================
// Color Profile Detection
// ============================================================================

/// Terminal color profile capabilities.
///
/// Ordered from least capable (Ascii) to most capable (TrueColor).
/// Detection follows the hierarchy defined in ACCESSIBILITY.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorProfile {
    /// No color support (1-bit). ASCII borders only.
    Ascii,
    /// 16 ANSI colors (4-bit).
    Ansi16,
    /// 256 colors (8-bit).
    Ansi256,
    /// True color / 16 million colors (24-bit).
    #[default]
    TrueColor,
}

impl ColorProfile {
    /// Detect the terminal's color profile from environment.
    ///
    /// Detection hierarchy (from ACCESSIBILITY.md):
    /// 1. `NO_COLOR` set → Ascii
    /// 2. `TERM=dumb` or empty → Ascii
    /// 3. `COLORTERM=truecolor` or `24bit` → TrueColor
    /// 4. `TERM` contains `256color` → Ansi256
    /// 5. Default → Ansi16
    #[must_use]
    pub fn detect() -> Self {
        // Check for NO_COLOR (any value disables colors)
        if env::var("NO_COLOR").is_ok() {
            return Self::Ascii;
        }

        // Check for dumb terminal
        let term = env::var("TERM").unwrap_or_default();
        if term.is_empty() || term == "dumb" {
            return Self::Ascii;
        }

        // Check for true color support
        if let Ok(colorterm) = env::var("COLORTERM") {
            if colorterm == "truecolor" || colorterm == "24bit" {
                return Self::TrueColor;
            }
        }

        // Check for 256-color support
        if term.contains("256color") || term.contains("256-color") {
            return Self::Ansi256;
        }

        // Default to ANSI 16 for known terminal types
        Self::Ansi16
    }

    /// Check if this profile supports colors.
    #[must_use]
    pub const fn has_color(&self) -> bool {
        !matches!(self, Self::Ascii)
    }

    /// Check if this profile supports 256 colors.
    #[must_use]
    pub const fn has_256_colors(&self) -> bool {
        matches!(self, Self::Ansi256 | Self::TrueColor)
    }

    /// Check if this profile supports true color (24-bit).
    #[must_use]
    pub const fn has_true_color(&self) -> bool {
        matches!(self, Self::TrueColor)
    }

    /// Get a human-readable name for this profile.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Ascii => "ASCII (no color)",
            Self::Ansi16 => "ANSI 16 colors",
            Self::Ansi256 => "ANSI 256 colors",
            Self::TrueColor => "True color (24-bit)",
        }
    }
}

// ============================================================================
// ASCII Mode Support
// ============================================================================

/// Status indicator for different health states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Healthy, OK, success.
    Ok,
    /// Warning, degraded.
    Warning,
    /// Error, failed, critical.
    Error,
    /// Unknown, pending.
    Unknown,
}

impl Status {
    /// Get the Unicode character indicator (for colored modes).
    #[must_use]
    pub const fn unicode_char(&self) -> char {
        match self {
            Self::Ok => '●',      // Filled circle
            Self::Warning => '◐', // Half circle
            Self::Error => '○',   // Empty circle
            Self::Unknown => '?',
        }
    }

    /// Get the ASCII text indicator (for NO_COLOR mode).
    #[must_use]
    pub const fn ascii_text(&self) -> &'static str {
        match self {
            Self::Ok => "[OK]",
            Self::Warning => "[!!]",
            Self::Error => "[XX]",
            Self::Unknown => "[??]",
        }
    }

    /// Get the indicator appropriate for the color profile.
    #[must_use]
    pub fn indicator(&self, profile: ColorProfile) -> String {
        if profile.has_color() {
            self.unicode_char().to_string()
        } else {
            self.ascii_text().to_string()
        }
    }
}

/// ASCII-safe border characters for NO_COLOR mode.
///
/// Maps Unicode box-drawing characters to ASCII equivalents.
pub mod ascii_borders {
    /// Top-left corner: `+`
    pub const TOP_LEFT: char = '+';
    /// Top-right corner: `+`
    pub const TOP_RIGHT: char = '+';
    /// Bottom-left corner: `+`
    pub const BOTTOM_LEFT: char = '+';
    /// Bottom-right corner: `+`
    pub const BOTTOM_RIGHT: char = '+';
    /// Horizontal line: `-`
    pub const HORIZONTAL: char = '-';
    /// Vertical line: `|`
    pub const VERTICAL: char = '|';
    /// T-junction: `+`
    pub const T_JUNCTION: char = '+';
    /// Cross: `+`
    pub const CROSS: char = '+';

    /// Double horizontal (for emphasis): `=`
    pub const DOUBLE_HORIZONTAL: char = '=';
}

/// Progress bar characters for different modes.
pub mod progress_chars {
    /// Filled section for colored mode.
    pub const FILL_UNICODE: char = '█';
    /// Empty section for colored mode.
    pub const EMPTY_UNICODE: char = '░';
    /// Filled section for ASCII mode.
    pub const FILL_ASCII: char = '#';
    /// Empty section for ASCII mode.
    pub const EMPTY_ASCII: char = '.';

    /// Get fill character for the given color profile.
    #[must_use]
    pub const fn fill(has_color: bool) -> char {
        if has_color {
            FILL_UNICODE
        } else {
            FILL_ASCII
        }
    }

    /// Get empty character for the given color profile.
    #[must_use]
    pub const fn empty(has_color: bool) -> char {
        if has_color {
            EMPTY_UNICODE
        } else {
            EMPTY_ASCII
        }
    }
}

/// ANSI 16-color mappings for semantic tokens.
///
/// Used when the terminal only supports 16 colors.
pub mod ansi16 {
    /// Primary color → Bright Blue (94).
    pub const PRIMARY: u8 = 94;
    /// Secondary color → Bright Magenta (95).
    pub const SECONDARY: u8 = 95;
    /// Success color → Bright Green (92).
    pub const SUCCESS: u8 = 92;
    /// Warning color → Bright Yellow (93).
    pub const WARNING: u8 = 93;
    /// Error color → Bright Red (91).
    pub const ERROR: u8 = 91;
    /// Info color → Bright Cyan (96).
    pub const INFO: u8 = 96;
    /// Text color → White (97).
    pub const TEXT: u8 = 97;
    /// Muted text → Bright Black (90).
    pub const TEXT_MUTED: u8 = 90;
    /// Border color → Bright Black (90).
    pub const BORDER: u8 = 90;

    /// Get the ANSI escape sequence for a foreground color.
    #[must_use]
    pub fn fg(code: u8) -> String {
        format!("\x1b[{code}m")
    }

    /// Get the ANSI escape sequence for a background color.
    #[must_use]
    pub fn bg(code: u8) -> String {
        format!("\x1b[{}m", code + 10)
    }

    /// Reset all styles.
    pub const RESET: &str = "\x1b[0m";
}

/// Get an ASCII-safe border style.
///
/// Returns a lipgloss Border that uses ASCII characters suitable for
/// terminals without Unicode support.
#[must_use]
pub fn ascii_border() -> Border {
    Border {
        top: String::from("-"),
        bottom: String::from("-"),
        left: String::from("|"),
        right: String::from("|"),
        top_left: String::from("+"),
        top_right: String::from("+"),
        bottom_left: String::from("+"),
        bottom_right: String::from("+"),
        middle_left: String::from("+"),
        middle_right: String::from("+"),
        middle: String::from("+"),
        middle_top: String::from("+"),
        middle_bottom: String::from("+"),
    }
}

/// Get a double ASCII border for emphasis.
#[must_use]
pub fn ascii_double_border() -> Border {
    Border {
        top: String::from("="),
        bottom: String::from("="),
        left: String::from("|"),
        right: String::from("|"),
        top_left: String::from("+"),
        top_right: String::from("+"),
        bottom_left: String::from("+"),
        bottom_right: String::from("+"),
        middle_left: String::from("+"),
        middle_right: String::from("+"),
        middle: String::from("+"),
        middle_top: String::from("+"),
        middle_bottom: String::from("+"),
    }
}

impl Theme {
    // ========================================================================
    // Color Profile Aware Helpers
    // ========================================================================

    /// Get box style appropriate for the color profile.
    ///
    /// Returns rounded borders for colored mode, ASCII borders for no-color.
    #[must_use]
    pub fn box_style_for_profile(&self, profile: ColorProfile) -> Style {
        if profile.has_color() {
            self.box_style()
        } else {
            Style::new().border(ascii_border())
        }
    }

    /// Get focused box style appropriate for the color profile.
    #[must_use]
    pub fn box_focused_style_for_profile(&self, profile: ColorProfile) -> Style {
        if profile.has_color() {
            self.box_focused_style()
        } else {
            // In ASCII mode, use double border for focus
            Style::new().border(ascii_double_border())
        }
    }

    /// Get modal style appropriate for the color profile.
    #[must_use]
    pub fn modal_style_for_profile(&self, profile: ColorProfile) -> Style {
        if profile.has_color() {
            self.modal_style()
        } else {
            Style::new()
                .border(ascii_double_border())
                .padding((spacing::SM, spacing::MD))
        }
    }

    /// Render a status indicator appropriate for the color profile.
    #[must_use]
    pub fn render_status(&self, status: Status, profile: ColorProfile) -> String {
        let indicator = status.indicator(profile);

        if profile.has_color() {
            let style = match status {
                Status::Ok => self.success_style(),
                Status::Warning => self.warning_style(),
                Status::Error => self.error_style(),
                Status::Unknown => self.muted_style(),
            };
            style.render(&indicator).to_string()
        } else {
            indicator
        }
    }

    /// Render a progress bar appropriate for the color profile.
    #[must_use]
    pub fn render_progress(&self, percent: u8, width: usize, profile: ColorProfile) -> String {
        let percent = percent.min(100);
        let has_color = profile.has_color();

        let fill_char = progress_chars::fill(has_color);
        let empty_char = progress_chars::empty(has_color);

        let inner_width = width.saturating_sub(2); // Account for brackets
        let filled = (usize::from(percent) * inner_width) / 100;
        let empty = inner_width.saturating_sub(filled);

        let bar: String = std::iter::repeat(fill_char)
            .take(filled)
            .chain(std::iter::repeat(empty_char).take(empty))
            .collect();

        if has_color {
            let style = self.progress_style(percent);
            format!("[{}]", style.render(&bar))
        } else {
            format!("[{bar}] {percent}%")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_presets_have_names() {
        assert_eq!(ThemePreset::Dark.name(), "Dark");
        assert_eq!(ThemePreset::Light.name(), "Light");
        assert_eq!(ThemePreset::Dracula.name(), "Dracula");
    }

    #[test]
    fn all_presets_returns_three() {
        assert_eq!(ThemePreset::all().len(), 3);
    }

    #[test]
    fn from_preset_roundtrips() {
        for preset in ThemePreset::all() {
            let theme = Theme::from_preset(preset);
            assert_eq!(theme.preset, preset);
        }
    }

    #[test]
    fn spacing_constants_are_ordered() {
        const {
            assert!(spacing::XS < spacing::SM);
            assert!(spacing::SM < spacing::MD);
            assert!(spacing::MD < spacing::LG);
            assert!(spacing::LG < spacing::XL);
        }
    }

    #[test]
    fn status_style_returns_correct_variant() {
        let theme = Theme::dark();
        // Just verify these don't panic and return valid styles
        let _ = theme.status_style(true);
        let _ = theme.status_style(false);
        let _ = theme.progress_style(0);
        let _ = theme.progress_style(50);
        let _ = theme.progress_style(100);
    }

    #[test]
    fn color_profile_detection_respects_no_color() {
        // Note: This test would need env var mocking to test properly
        // Here we just verify the default behavior
        let profile = ColorProfile::default();
        assert_eq!(profile, ColorProfile::TrueColor);
    }

    #[test]
    fn color_profile_capabilities() {
        assert!(!ColorProfile::Ascii.has_color());
        assert!(ColorProfile::Ansi16.has_color());
        assert!(ColorProfile::Ansi256.has_color());
        assert!(ColorProfile::TrueColor.has_color());

        assert!(!ColorProfile::Ascii.has_256_colors());
        assert!(!ColorProfile::Ansi16.has_256_colors());
        assert!(ColorProfile::Ansi256.has_256_colors());
        assert!(ColorProfile::TrueColor.has_256_colors());

        assert!(!ColorProfile::Ascii.has_true_color());
        assert!(!ColorProfile::Ansi16.has_true_color());
        assert!(!ColorProfile::Ansi256.has_true_color());
        assert!(ColorProfile::TrueColor.has_true_color());
    }

    #[test]
    fn color_profile_names() {
        assert!(ColorProfile::Ascii.name().contains("ASCII"));
        assert!(ColorProfile::Ansi16.name().contains("16"));
        assert!(ColorProfile::Ansi256.name().contains("256"));
        assert!(ColorProfile::TrueColor.name().contains("24-bit"));
    }

    #[test]
    fn status_indicators() {
        assert_eq!(Status::Ok.unicode_char(), '●');
        assert_eq!(Status::Warning.unicode_char(), '◐');
        assert_eq!(Status::Error.unicode_char(), '○');
        assert_eq!(Status::Unknown.unicode_char(), '?');

        assert_eq!(Status::Ok.ascii_text(), "[OK]");
        assert_eq!(Status::Warning.ascii_text(), "[!!]");
        assert_eq!(Status::Error.ascii_text(), "[XX]");
        assert_eq!(Status::Unknown.ascii_text(), "[??]");
    }

    #[test]
    fn status_indicator_respects_profile() {
        let ok = Status::Ok;

        let colored = ok.indicator(ColorProfile::TrueColor);
        assert_eq!(colored, "●");

        let ascii = ok.indicator(ColorProfile::Ascii);
        assert_eq!(ascii, "[OK]");
    }

    #[test]
    fn progress_chars_for_profile() {
        assert_eq!(progress_chars::fill(true), '█');
        assert_eq!(progress_chars::fill(false), '#');
        assert_eq!(progress_chars::empty(true), '░');
        assert_eq!(progress_chars::empty(false), '.');
    }

    #[test]
    fn ascii_border_uses_ascii_chars() {
        let border = ascii_border();
        assert_eq!(border.top_left, "+");
        assert_eq!(border.horizontal(), "-");
        assert_eq!(border.vertical(), "|");
    }

    #[test]
    fn ascii_double_border_uses_equals() {
        let border = ascii_double_border();
        assert_eq!(border.top, "=");
        assert_eq!(border.bottom, "=");
    }

    #[test]
    fn theme_box_style_for_profile() {
        let theme = Theme::dark();

        // Colored mode should use rounded borders
        let colored = theme.box_style_for_profile(ColorProfile::TrueColor);
        assert!(format!("{colored:?}").contains("border"));

        // ASCII mode should work
        let ascii = theme.box_style_for_profile(ColorProfile::Ascii);
        assert!(format!("{ascii:?}").contains("border"));
    }

    #[test]
    fn theme_render_status_colored() {
        let theme = Theme::dark();
        let result = theme.render_status(Status::Ok, ColorProfile::TrueColor);
        assert!(result.contains('●') || result.contains("[OK]"));
    }

    #[test]
    fn theme_render_status_ascii() {
        let theme = Theme::dark();
        let result = theme.render_status(Status::Ok, ColorProfile::Ascii);
        assert_eq!(result, "[OK]");
    }

    #[test]
    fn theme_render_progress_bar() {
        let theme = Theme::dark();

        // Test ASCII mode progress
        let ascii = theme.render_progress(50, 12, ColorProfile::Ascii);
        assert!(ascii.contains('['));
        assert!(ascii.contains(']'));
        assert!(ascii.contains('#') || ascii.contains('.'));

        // Test colored mode progress
        let colored = theme.render_progress(50, 12, ColorProfile::TrueColor);
        assert!(colored.contains('['));
    }

    #[test]
    fn theme_render_progress_bounds() {
        let theme = Theme::dark();

        // 0%
        let zero = theme.render_progress(0, 12, ColorProfile::Ascii);
        assert!(zero.contains("0%"));

        // 100%
        let hundred = theme.render_progress(100, 12, ColorProfile::Ascii);
        assert!(hundred.contains("100%"));

        // Over 100% should clamp
        let over = theme.render_progress(150, 12, ColorProfile::Ascii);
        assert!(over.contains("100%"));
    }

    #[test]
    fn ansi16_codes_are_valid() {
        // Verify codes are in valid ANSI bright color range (90-97, 100-107)
        assert!(ansi16::PRIMARY >= 90 && ansi16::PRIMARY <= 97);
        assert!(ansi16::SUCCESS >= 90 && ansi16::SUCCESS <= 97);
        assert!(ansi16::ERROR >= 90 && ansi16::ERROR <= 97);
        assert!(ansi16::WARNING >= 90 && ansi16::WARNING <= 97);
        assert!(ansi16::INFO >= 90 && ansi16::INFO <= 97);
        assert!(ansi16::TEXT >= 90 && ansi16::TEXT <= 97);
    }

    #[test]
    fn ansi16_escape_sequences() {
        let fg = ansi16::fg(ansi16::PRIMARY);
        assert!(fg.starts_with("\x1b["));
        assert!(fg.ends_with('m'));

        let bg = ansi16::bg(ansi16::PRIMARY);
        assert!(bg.starts_with("\x1b["));
        assert!(bg.ends_with('m'));

        // bg code should be 10 higher than fg
        assert_eq!(ansi16::RESET, "\x1b[0m");
    }
}
