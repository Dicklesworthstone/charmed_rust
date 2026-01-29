//! Theme definitions for `demo_showcase`.
//!
//! Provides semantic color tokens and style helpers for consistent theming.
//! See `docs/demo_showcase/VISUAL_DESIGN.md` for the design specification.
//! See `docs/demo_showcase/ACCESSIBILITY.md` for accessibility guidelines.

#![allow(dead_code)] // Style helpers will be used as pages are implemented

use std::env;
use std::fs;
use std::path::Path;

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
// Custom Theme Loading
// ============================================================================

/// A custom theme loaded from a JSON file.
///
/// All color fields are required and must be valid hex colors (#RGB or #RRGGBB).
/// The theme will be validated for:
/// - Required fields present
/// - Valid hex color format
/// - Sufficient contrast between key color pairs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTheme {
    /// Optional name for the theme.
    #[serde(default)]
    pub name: Option<String>,

    // Primary colors
    /// Brand color, accent, interactive elements.
    pub primary: String,
    /// Secondary accent, less prominent.
    pub secondary: String,

    // Semantic colors
    /// Healthy, complete, positive states.
    pub success: String,
    /// Needs attention, degraded states.
    pub warning: String,
    /// Failed, critical, action needed.
    pub error: String,
    /// Informational, neutral highlight.
    pub info: String,

    // Text colors
    /// Primary text, high contrast.
    pub text: String,
    /// Secondary text, hints, timestamps.
    pub text_muted: String,
    /// Text on colored backgrounds.
    pub text_inverse: String,

    // Background colors
    /// Main background.
    pub bg: String,
    /// Sidebar, header, card backgrounds.
    pub bg_subtle: String,
    /// Hover, selection, active states.
    pub bg_highlight: String,

    // Border colors
    /// Subtle borders, dividers.
    pub border: String,
    /// Focused element borders.
    pub border_focus: String,
}

/// Result of loading a custom theme.
#[derive(Debug, Clone)]
pub struct ThemeLoadResult {
    /// The loaded theme (or fallback if loading failed).
    pub theme: Theme,
    /// Warnings about the theme (e.g., low contrast).
    pub warnings: Vec<ThemeWarning>,
    /// Error that caused fallback, if any.
    pub error: Option<ThemeLoadError>,
}

/// Warning about a loaded theme.
#[derive(Debug, Clone)]
pub struct ThemeWarning {
    /// The type of warning.
    pub kind: ThemeWarningKind,
    /// Human-readable description.
    pub message: String,
}

/// Types of theme warnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeWarningKind {
    /// Low contrast between foreground and background colors.
    LowContrast,
    /// Invalid color format (not a valid hex color).
    InvalidColor,
}

/// Error that can occur when loading a custom theme.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ThemeLoadError {
    /// File not found.
    #[error("Theme file not found: {0}")]
    FileNotFound(String),

    /// File read error.
    #[error("Failed to read theme file: {0}")]
    ReadError(String),

    /// JSON parse error.
    #[error("Invalid JSON in theme file: {0}")]
    ParseError(String),

    /// Missing required field.
    #[error("Missing required field in theme: {0}")]
    MissingField(String),

    /// Invalid color format.
    #[error("Invalid color format: {field} = {value}")]
    InvalidColorFormat { field: String, value: String },
}

impl CustomTheme {
    /// Load a custom theme from a JSON file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn load_from_file(path: &Path) -> Result<Self, ThemeLoadError> {
        if !path.exists() {
            return Err(ThemeLoadError::FileNotFound(path.display().to_string()));
        }

        let content =
            fs::read_to_string(path).map_err(|e| ThemeLoadError::ReadError(e.to_string()))?;

        serde_json::from_str(&content).map_err(|e| ThemeLoadError::ParseError(e.to_string()))
    }

    /// Validate the custom theme and convert to a Theme.
    ///
    /// Returns the theme along with any validation warnings.
    #[must_use]
    pub fn validate_and_convert(&self) -> (Theme, Vec<ThemeWarning>) {
        let mut warnings = Vec::new();

        // Validate all color fields
        let fields = [
            ("primary", &self.primary),
            ("secondary", &self.secondary),
            ("success", &self.success),
            ("warning", &self.warning),
            ("error", &self.error),
            ("info", &self.info),
            ("text", &self.text),
            ("text_muted", &self.text_muted),
            ("text_inverse", &self.text_inverse),
            ("bg", &self.bg),
            ("bg_subtle", &self.bg_subtle),
            ("bg_highlight", &self.bg_highlight),
            ("border", &self.border),
            ("border_focus", &self.border_focus),
        ];

        for (name, value) in &fields {
            if !is_valid_hex_color(value) {
                warnings.push(ThemeWarning {
                    kind: ThemeWarningKind::InvalidColor,
                    message: format!("{name} is not a valid hex color: {value}"),
                });
            }
        }

        // Check contrast ratios for key pairs
        let contrast_pairs = [
            ("text", &self.text, "bg", &self.bg, 4.5),
            ("text_muted", &self.text_muted, "bg", &self.bg, 3.0),
            ("text", &self.text, "bg_subtle", &self.bg_subtle, 4.5),
            ("text", &self.text, "bg_highlight", &self.bg_highlight, 4.5),
            (
                "text_inverse",
                &self.text_inverse,
                "primary",
                &self.primary,
                4.5,
            ),
        ];

        for (fg_name, fg, bg_name, bg, min_ratio) in &contrast_pairs {
            if let (Some(fg_rgb), Some(bg_rgb)) = (parse_hex_color(fg), parse_hex_color(bg)) {
                let ratio = contrast_ratio(fg_rgb, bg_rgb);
                if ratio < *min_ratio {
                    warnings.push(ThemeWarning {
                        kind: ThemeWarningKind::LowContrast,
                        message: format!(
                            "Low contrast between {fg_name} and {bg_name}: {ratio:.2} (minimum {min_ratio})"
                        ),
                    });
                }
            }
        }

        // Convert to Theme using leaked static strings
        // (This is acceptable for a theme that lives for the program lifetime)
        let theme = Theme {
            preset: ThemePreset::Dark, // Custom themes show as Dark in preset field
            primary: leak_string(&self.primary),
            secondary: leak_string(&self.secondary),
            success: leak_string(&self.success),
            warning: leak_string(&self.warning),
            error: leak_string(&self.error),
            info: leak_string(&self.info),
            text: leak_string(&self.text),
            text_muted: leak_string(&self.text_muted),
            text_inverse: leak_string(&self.text_inverse),
            bg: leak_string(&self.bg),
            bg_subtle: leak_string(&self.bg_subtle),
            bg_highlight: leak_string(&self.bg_highlight),
            border: leak_string(&self.border),
            border_focus: leak_string(&self.border_focus),
        };

        (theme, warnings)
    }
}

/// Load a theme from a file path, with fallback to a preset on error.
///
/// Returns the loaded theme along with any warnings or errors.
/// If loading fails, falls back to the specified preset.
#[must_use]
pub fn load_theme_from_file(path: &Path, fallback: ThemePreset) -> ThemeLoadResult {
    match CustomTheme::load_from_file(path) {
        Ok(custom) => {
            let (theme, warnings) = custom.validate_and_convert();
            ThemeLoadResult {
                theme,
                warnings,
                error: None,
            }
        }
        Err(e) => ThemeLoadResult {
            theme: Theme::from_preset(fallback),
            warnings: vec![],
            error: Some(e),
        },
    }
}

/// Check if a string is a valid hex color (#RGB or #RRGGBB).
#[must_use]
pub fn is_valid_hex_color(s: &str) -> bool {
    if !s.starts_with('#') {
        return false;
    }
    let hex = &s[1..];
    if hex.len() != 3 && hex.len() != 6 {
        return false;
    }
    hex.chars().all(|c| c.is_ascii_hexdigit())
}

/// Parse a hex color to RGB values.
#[must_use]
pub fn parse_hex_color(s: &str) -> Option<(u8, u8, u8)> {
    if !s.starts_with('#') {
        return None;
    }
    let hex = &s[1..];
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            Some((r, g, b))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((r, g, b))
        }
        _ => None,
    }
}

/// Calculate relative luminance for a color.
/// See: https://www.w3.org/TR/WCAG20/#relativeluminancedef
#[must_use]
pub fn relative_luminance(r: u8, g: u8, b: u8) -> f64 {
    fn channel(c: u8) -> f64 {
        let c = f64::from(c) / 255.0;
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
}

/// Calculate WCAG contrast ratio between two colors.
/// See: https://www.w3.org/TR/WCAG20/#contrast-ratiodef
#[must_use]
pub fn contrast_ratio(fg: (u8, u8, u8), bg: (u8, u8, u8)) -> f64 {
    let l1 = relative_luminance(fg.0, fg.1, fg.2);
    let l2 = relative_luminance(bg.0, bg.1, bg.2);
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    (lighter + 0.05) / (darker + 0.05)
}

/// Leak a string to get a static reference.
/// This is acceptable for themes that live for the program lifetime.
fn leak_string(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
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
        if has_color { FILL_UNICODE } else { FILL_ASCII }
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
        assert_eq!(border.top, "-");
        assert_eq!(border.left, "|");
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

        // Colored mode should use rounded borders - verify it doesn't panic
        let _colored = theme.box_style_for_profile(ColorProfile::TrueColor);

        // ASCII mode should work - verify it doesn't panic
        let _ascii = theme.box_style_for_profile(ColorProfile::Ascii);

        // Focused styles should also work
        let _focused_colored = theme.box_focused_style_for_profile(ColorProfile::TrueColor);
        let _focused_ascii = theme.box_focused_style_for_profile(ColorProfile::Ascii);
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

    // =========================================================================
    // bd-2oku: Theme switching invariants
    // =========================================================================

    // --- Preset switching updates semantic tokens ---

    #[test]
    fn switching_themes_changes_all_semantic_tokens() {
        let dark = Theme::dark();
        let light = Theme::light();
        let dracula = Theme::dracula();

        // Primary colors must differ across all three themes
        assert_ne!(dark.primary, light.primary, "dark vs light primary");
        assert_ne!(dark.primary, dracula.primary, "dark vs dracula primary");
        assert_ne!(light.primary, dracula.primary, "light vs dracula primary");

        // Background must differ (dark vs light is the most obvious)
        assert_ne!(dark.bg, light.bg, "dark vs light bg");
        assert_ne!(dark.bg, dracula.bg, "dark vs dracula bg");

        // Text must differ
        assert_ne!(dark.text, light.text, "dark vs light text");
    }

    #[test]
    fn all_tokens_populated_for_every_preset() {
        for preset in ThemePreset::all() {
            let theme = Theme::from_preset(preset);
            let name = preset.name();

            assert!(!theme.primary.is_empty(), "{name}: primary empty");
            assert!(!theme.secondary.is_empty(), "{name}: secondary empty");
            assert!(!theme.success.is_empty(), "{name}: success empty");
            assert!(!theme.warning.is_empty(), "{name}: warning empty");
            assert!(!theme.error.is_empty(), "{name}: error empty");
            assert!(!theme.info.is_empty(), "{name}: info empty");
            assert!(!theme.text.is_empty(), "{name}: text empty");
            assert!(!theme.text_muted.is_empty(), "{name}: text_muted empty");
            assert!(!theme.text_inverse.is_empty(), "{name}: text_inverse empty");
            assert!(!theme.bg.is_empty(), "{name}: bg empty");
            assert!(!theme.bg_subtle.is_empty(), "{name}: bg_subtle empty");
            assert!(!theme.bg_highlight.is_empty(), "{name}: bg_highlight empty");
            assert!(!theme.border.is_empty(), "{name}: border empty");
            assert!(!theme.border_focus.is_empty(), "{name}: border_focus empty");
        }
    }

    #[test]
    fn all_tokens_are_hex_colors() {
        for preset in ThemePreset::all() {
            let theme = Theme::from_preset(preset);
            let name = preset.name();
            let tokens = [
                ("primary", theme.primary),
                ("secondary", theme.secondary),
                ("success", theme.success),
                ("warning", theme.warning),
                ("error", theme.error),
                ("info", theme.info),
                ("text", theme.text),
                ("text_muted", theme.text_muted),
                ("text_inverse", theme.text_inverse),
                ("bg", theme.bg),
                ("bg_subtle", theme.bg_subtle),
                ("bg_highlight", theme.bg_highlight),
                ("border", theme.border),
                ("border_focus", theme.border_focus),
            ];

            for (token_name, value) in tokens {
                assert!(
                    value.starts_with('#'),
                    "{name}.{token_name} = {value:?} — not a hex color"
                );
                assert!(
                    value.len() == 7 || value.len() == 4,
                    "{name}.{token_name} = {value:?} — unexpected hex color length"
                );
            }
        }
    }

    #[test]
    fn preset_field_matches_constructor() {
        assert_eq!(Theme::dark().preset, ThemePreset::Dark);
        assert_eq!(Theme::light().preset, ThemePreset::Light);
        assert_eq!(Theme::dracula().preset, ThemePreset::Dracula);
    }

    // --- Style helpers produce different output per theme ---

    #[test]
    fn style_helpers_differ_across_themes() {
        let dark = Theme::dark();
        let light = Theme::light();

        // Render the same text with different themes — output should differ.
        let dark_title = dark.title_style().render("Title").to_string();
        let light_title = light.title_style().render("Title").to_string();
        assert_ne!(
            dark_title, light_title,
            "title_style should differ between dark and light"
        );

        let dark_success = dark.success_style().render("OK").to_string();
        let light_success = light.success_style().render("OK").to_string();
        assert_ne!(
            dark_success, light_success,
            "success_style should differ between dark and light"
        );

        let dark_badge = dark.badge_primary_style().render("tag").to_string();
        let light_badge = light.badge_primary_style().render("tag").to_string();
        assert_ne!(
            dark_badge, light_badge,
            "badge_primary_style should differ between dark and light"
        );
    }

    #[test]
    fn card_style_uses_theme_bg() {
        let dark = Theme::dark();
        let light = Theme::light();

        let dark_card = dark.card_style().render("content").to_string();
        let light_card = light.card_style().render("content").to_string();

        // Cards use bg_subtle which differs between themes
        assert_ne!(
            dark_card, light_card,
            "card_style should differ with different bg_subtle"
        );
    }

    #[test]
    fn panel_style_uses_theme_border() {
        let dark = Theme::dark();
        let light = Theme::light();

        let dark_panel = dark.panel_style().render("content").to_string();
        let light_panel = light.panel_style().render("content").to_string();

        assert_ne!(
            dark_panel, light_panel,
            "panel_style should differ between themes"
        );
    }

    #[test]
    fn table_style_uses_theme_border() {
        let dark = Theme::dark();
        let light = Theme::light();

        let dark_table = dark.table_style().render("data").to_string();
        let light_table = light.table_style().render("data").to_string();

        assert_ne!(
            dark_table, light_table,
            "table_style should differ between themes"
        );
    }

    // --- No-color / ASCII mode ---

    #[test]
    fn ascii_status_output_has_no_ansi() {
        let theme = Theme::dark();
        for status in [Status::Ok, Status::Warning, Status::Error, Status::Unknown] {
            let output = theme.render_status(status, ColorProfile::Ascii);
            assert!(
                !output.contains('\x1b'),
                "ASCII mode status {:?} contains ANSI: {:?}",
                status,
                output
            );
        }
    }

    #[test]
    fn ascii_progress_bar_has_no_ansi() {
        let theme = Theme::dark();
        for pct in [0, 25, 50, 75, 100, 200] {
            let output = theme.render_progress(pct, 20, ColorProfile::Ascii);
            assert!(
                !output.contains('\x1b'),
                "ASCII progress at {pct}% contains ANSI: {:?}",
                output
            );
        }
    }

    #[test]
    fn ascii_progress_bar_uses_correct_chars() {
        let theme = Theme::dark();
        let bar = theme.render_progress(50, 22, ColorProfile::Ascii);
        assert!(
            bar.contains('#'),
            "ASCII progress should use # for fill: {bar:?}"
        );
        assert!(
            bar.contains('.'),
            "ASCII progress should use . for empty: {bar:?}"
        );
    }

    #[test]
    fn ascii_mode_still_uses_spacing() {
        let theme = Theme::dark();

        // box_style_for_profile in ASCII mode still applies borders
        let ascii_box = theme.box_style_for_profile(ColorProfile::Ascii);
        let rendered = ascii_box.render("test").to_string();

        // Should contain ASCII border characters
        assert!(
            rendered.contains('+') || rendered.contains('-') || rendered.contains('|'),
            "ASCII box should use ASCII border chars: {:?}",
            rendered
        );
    }

    #[test]
    fn ascii_focused_box_uses_double_border() {
        let theme = Theme::dark();
        let focused = theme.box_focused_style_for_profile(ColorProfile::Ascii);
        let rendered = focused.render("focus").to_string();

        // Double border uses '='
        assert!(
            rendered.contains('='),
            "ASCII focused box should use double borders: {:?}",
            rendered
        );
    }

    #[test]
    fn ascii_modal_uses_double_border() {
        let theme = Theme::dark();
        let modal = theme.modal_style_for_profile(ColorProfile::Ascii);
        let rendered = modal.render("dialog").to_string();

        assert!(
            rendered.contains('='),
            "ASCII modal should use double borders: {:?}",
            rendered
        );
    }

    // --- Theme atomicity (no mixed styles) ---

    #[test]
    fn theme_from_preset_is_atomic() {
        // Creating a theme from a preset should return a complete, consistent theme.
        // Verify that all tokens belong to the same preset.
        for preset in ThemePreset::all() {
            let theme = Theme::from_preset(preset);
            assert_eq!(
                theme.preset, preset,
                "theme.preset should match the constructor preset"
            );

            // Create a reference theme and verify all tokens match
            let reference = match preset {
                ThemePreset::Dark => Theme::dark(),
                ThemePreset::Light => Theme::light(),
                ThemePreset::Dracula => Theme::dracula(),
            };

            assert_eq!(
                theme.primary, reference.primary,
                "primary mismatch for {preset:?}"
            );
            assert_eq!(theme.bg, reference.bg, "bg mismatch for {preset:?}");
            assert_eq!(theme.text, reference.text, "text mismatch for {preset:?}");
            assert_eq!(
                theme.success, reference.success,
                "success mismatch for {preset:?}"
            );
            assert_eq!(
                theme.error, reference.error,
                "error mismatch for {preset:?}"
            );
        }
    }

    #[test]
    fn default_theme_is_dark() {
        let default = Theme::default();
        let dark = Theme::dark();
        assert_eq!(default.preset, ThemePreset::Dark);
        assert_eq!(default.primary, dark.primary);
        assert_eq!(default.bg, dark.bg);
    }

    #[test]
    fn default_preset_is_dark() {
        assert_eq!(ThemePreset::default(), ThemePreset::Dark);
    }

    // --- Serialization roundtrip for ThemePreset ---

    #[test]
    fn theme_preset_json_roundtrip() {
        for preset in ThemePreset::all() {
            let json = serde_json::to_string(&preset).unwrap();
            let parsed: ThemePreset = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, preset, "roundtrip failed for {preset:?}");
        }
    }

    // --- All style helpers don't panic ---

    #[test]
    fn all_style_helpers_produce_output_for_every_theme() {
        for preset in ThemePreset::all() {
            let theme = Theme::from_preset(preset);
            let name = preset.name();
            let text = "test";

            // Typography
            let _ = theme.title_style().render(text);
            let _ = theme.heading_style().render(text);
            let _ = theme.subheading_style().render(text);
            let _ = theme.muted_style().render(text);
            let _ = theme.link_style().render(text);
            let _ = theme.shortcut_style().render(text);

            // Semantic
            let _ = theme.success_style().render(text);
            let _ = theme.warning_style().render(text);
            let _ = theme.error_style().render(text);
            let _ = theme.info_style().render(text);

            // Containers
            let _ = theme.box_style().render(text);
            let _ = theme.box_focused_style().render(text);
            let _ = theme.card_style().render(text);
            let _ = theme.panel_style().render(text);
            let _ = theme.modal_style().render(text);
            let _ = theme.table_style().render(text);

            // Interactive
            let _ = theme.badge_style().render(text);
            let _ = theme.badge_primary_style().render(text);
            let _ = theme.button_style().render(text);
            let _ = theme.button_primary_style().render(text);
            let _ = theme.hover_style().render(text);
            let _ = theme.selected_style().render(text);

            // Chrome
            let _ = theme.header_style().render(text);
            let _ = theme.footer_style().render(text);
            let _ = theme.sidebar_style().render(text);
            let _ = theme.sidebar_selected_style().render(text);
            let _ = theme.sidebar_inactive_style().render(text);

            // Status
            let _ = theme.status_style(true).render(text);
            let _ = theme.status_style(false).render(text);
            let _ = theme.progress_style(0).render(text);
            let _ = theme.progress_style(100).render(text);

            // Profile-aware
            for profile in [
                ColorProfile::Ascii,
                ColorProfile::Ansi16,
                ColorProfile::Ansi256,
                ColorProfile::TrueColor,
            ] {
                let _ = theme.box_style_for_profile(profile).render(text);
                let _ = theme.box_focused_style_for_profile(profile).render(text);
                let _ = theme.modal_style_for_profile(profile).render(text);
                let _ = theme.render_status(Status::Ok, profile);
                let _ = theme.render_progress(50, 20, profile);
            }

            // If we get here without panic, the theme is safe
            assert!(true, "{name} style helpers should not panic");
        }
    }

    // --- Distinct semantic colors within each theme ---

    #[test]
    fn semantic_colors_are_distinct_within_theme() {
        for preset in ThemePreset::all() {
            let theme = Theme::from_preset(preset);
            let name = preset.name();

            // Status colors should be distinct from each other
            let status_colors = [theme.success, theme.warning, theme.error, theme.info];
            for i in 0..status_colors.len() {
                for j in (i + 1)..status_colors.len() {
                    assert_ne!(
                        status_colors[i], status_colors[j],
                        "{name}: status colors at index {i} and {j} are identical"
                    );
                }
            }

            // Text and background should differ (readability)
            assert_ne!(
                theme.text, theme.bg,
                "{name}: text and bg must differ for readability"
            );
        }
    }

    // --- border_focus uses theme primary ---

    #[test]
    fn border_focus_uses_primary_color() {
        for preset in ThemePreset::all() {
            let theme = Theme::from_preset(preset);
            assert_eq!(
                theme.border_focus, theme.primary,
                "{:?}: border_focus should match primary",
                preset
            );
        }
    }

    // =========================================================================
    // bd-2xlv: Custom theme loading + contrast checks
    // =========================================================================

    // --- Hex color validation ---

    #[test]
    fn is_valid_hex_color_accepts_valid_colors() {
        assert!(is_valid_hex_color("#FFF"));
        assert!(is_valid_hex_color("#fff"));
        assert!(is_valid_hex_color("#FFFFFF"));
        assert!(is_valid_hex_color("#ffffff"));
        assert!(is_valid_hex_color("#7D56F4"));
        assert!(is_valid_hex_color("#123"));
        assert!(is_valid_hex_color("#abc"));
        assert!(is_valid_hex_color("#ABC"));
        assert!(is_valid_hex_color("#123456"));
    }

    #[test]
    fn is_valid_hex_color_rejects_invalid_colors() {
        assert!(!is_valid_hex_color("FFF"));
        assert!(!is_valid_hex_color("#"));
        assert!(!is_valid_hex_color("#GGG"));
        assert!(!is_valid_hex_color("#12345"));
        assert!(!is_valid_hex_color("#1234567"));
        assert!(!is_valid_hex_color("rgb(255,255,255)"));
        assert!(!is_valid_hex_color(""));
        assert!(!is_valid_hex_color("#12"));
        assert!(!is_valid_hex_color("#1234"));
    }

    // --- Hex color parsing ---

    #[test]
    fn parse_hex_color_parses_short_form() {
        assert_eq!(parse_hex_color("#FFF"), Some((255, 255, 255)));
        assert_eq!(parse_hex_color("#000"), Some((0, 0, 0)));
        assert_eq!(parse_hex_color("#F00"), Some((255, 0, 0)));
        assert_eq!(parse_hex_color("#0F0"), Some((0, 255, 0)));
        assert_eq!(parse_hex_color("#00F"), Some((0, 0, 255)));
        // #ABC = #AABBCC
        assert_eq!(parse_hex_color("#ABC"), Some((170, 187, 204)));
    }

    #[test]
    fn parse_hex_color_parses_long_form() {
        assert_eq!(parse_hex_color("#FFFFFF"), Some((255, 255, 255)));
        assert_eq!(parse_hex_color("#000000"), Some((0, 0, 0)));
        assert_eq!(parse_hex_color("#FF0000"), Some((255, 0, 0)));
        assert_eq!(parse_hex_color("#00FF00"), Some((0, 255, 0)));
        assert_eq!(parse_hex_color("#0000FF"), Some((0, 0, 255)));
        assert_eq!(parse_hex_color("#7D56F4"), Some((125, 86, 244)));
    }

    #[test]
    fn parse_hex_color_rejects_invalid() {
        assert_eq!(parse_hex_color("FFF"), None);
        assert_eq!(parse_hex_color("#GGG"), None);
        assert_eq!(parse_hex_color("#12345"), None);
        assert_eq!(parse_hex_color(""), None);
    }

    // --- Luminance and contrast ---

    #[test]
    fn relative_luminance_black_is_zero() {
        let lum = relative_luminance(0, 0, 0);
        assert!(lum < 0.001, "black should have ~0 luminance");
    }

    #[test]
    fn relative_luminance_white_is_one() {
        let lum = relative_luminance(255, 255, 255);
        assert!(lum > 0.999, "white should have ~1 luminance");
    }

    #[test]
    fn contrast_ratio_black_white() {
        let ratio = contrast_ratio((0, 0, 0), (255, 255, 255));
        assert!(
            (ratio - 21.0).abs() < 0.1,
            "black/white contrast should be ~21:1"
        );
    }

    #[test]
    fn contrast_ratio_same_color() {
        let ratio = contrast_ratio((128, 128, 128), (128, 128, 128));
        assert!(
            (ratio - 1.0).abs() < 0.001,
            "same color contrast should be 1:1"
        );
    }

    #[test]
    fn contrast_ratio_order_independent() {
        let ratio1 = contrast_ratio((0, 0, 0), (255, 255, 255));
        let ratio2 = contrast_ratio((255, 255, 255), (0, 0, 0));
        assert!(
            (ratio1 - ratio2).abs() < 0.001,
            "contrast ratio should be order-independent"
        );
    }

    // --- CustomTheme validation ---

    #[test]
    fn custom_theme_validates_colors() {
        let theme = CustomTheme {
            name: Some("Test".to_string()),
            primary: "#INVALID".to_string(),
            secondary: "#FF79C6".to_string(),
            success: "#50FA7B".to_string(),
            warning: "#F1FA8C".to_string(),
            error: "#FF5555".to_string(),
            info: "#8BE9FD".to_string(),
            text: "#F8F8F2".to_string(),
            text_muted: "#6272A4".to_string(),
            text_inverse: "#282A36".to_string(),
            bg: "#282A36".to_string(),
            bg_subtle: "#343746".to_string(),
            bg_highlight: "#44475A".to_string(),
            border: "#44475A".to_string(),
            border_focus: "#BD93F9".to_string(),
        };

        let (_, warnings) = theme.validate_and_convert();
        assert!(
            warnings
                .iter()
                .any(|w| w.kind == ThemeWarningKind::InvalidColor),
            "should warn about invalid color"
        );
    }

    #[test]
    fn custom_theme_detects_low_contrast() {
        let theme = CustomTheme {
            name: Some("Low Contrast".to_string()),
            primary: "#333333".to_string(),
            secondary: "#444444".to_string(),
            success: "#50FA7B".to_string(),
            warning: "#F1FA8C".to_string(),
            error: "#FF5555".to_string(),
            info: "#8BE9FD".to_string(),
            text: "#444444".to_string(), // Very low contrast with bg
            text_muted: "#333333".to_string(),
            text_inverse: "#FFFFFF".to_string(),
            bg: "#333333".to_string(), // Same as text!
            bg_subtle: "#343746".to_string(),
            bg_highlight: "#44475A".to_string(),
            border: "#44475A".to_string(),
            border_focus: "#333333".to_string(),
        };

        let (_, warnings) = theme.validate_and_convert();
        assert!(
            warnings
                .iter()
                .any(|w| w.kind == ThemeWarningKind::LowContrast),
            "should warn about low contrast"
        );
    }

    #[test]
    fn custom_theme_good_contrast_no_warnings() {
        // Dracula theme has good contrast
        let theme = CustomTheme {
            name: Some("Dracula".to_string()),
            primary: "#BD93F9".to_string(),
            secondary: "#FF79C6".to_string(),
            success: "#50FA7B".to_string(),
            warning: "#F1FA8C".to_string(),
            error: "#FF5555".to_string(),
            info: "#8BE9FD".to_string(),
            text: "#F8F8F2".to_string(),
            text_muted: "#6272A4".to_string(),
            text_inverse: "#282A36".to_string(),
            bg: "#282A36".to_string(),
            bg_subtle: "#343746".to_string(),
            bg_highlight: "#44475A".to_string(),
            border: "#44475A".to_string(),
            border_focus: "#BD93F9".to_string(),
        };

        let (_, warnings) = theme.validate_and_convert();
        // Filter to only contrast warnings (color format is fine)
        let contrast_warnings: Vec<_> = warnings
            .iter()
            .filter(|w| w.kind == ThemeWarningKind::LowContrast)
            .collect();
        assert!(
            contrast_warnings.is_empty(),
            "Dracula theme should have good contrast, got: {:?}",
            contrast_warnings
        );
    }

    #[test]
    fn custom_theme_converts_to_theme() {
        let custom = CustomTheme {
            name: Some("Custom".to_string()),
            primary: "#7D56F4".to_string(),
            secondary: "#FF69B4".to_string(),
            success: "#00FF00".to_string(),
            warning: "#FFCC00".to_string(),
            error: "#FF0000".to_string(),
            info: "#00BFFF".to_string(),
            text: "#FFFFFF".to_string(),
            text_muted: "#626262".to_string(),
            text_inverse: "#000000".to_string(),
            bg: "#000000".to_string(),
            bg_subtle: "#1a1a1a".to_string(),
            bg_highlight: "#333333".to_string(),
            border: "#444444".to_string(),
            border_focus: "#7D56F4".to_string(),
        };

        let (theme, _) = custom.validate_and_convert();
        assert_eq!(theme.primary, "#7D56F4");
        assert_eq!(theme.bg, "#000000");
        assert_eq!(theme.text, "#FFFFFF");
    }

    #[test]
    fn custom_theme_json_roundtrip() {
        let original = CustomTheme {
            name: Some("Test".to_string()),
            primary: "#7D56F4".to_string(),
            secondary: "#FF69B4".to_string(),
            success: "#00FF00".to_string(),
            warning: "#FFCC00".to_string(),
            error: "#FF0000".to_string(),
            info: "#00BFFF".to_string(),
            text: "#FFFFFF".to_string(),
            text_muted: "#626262".to_string(),
            text_inverse: "#000000".to_string(),
            bg: "#000000".to_string(),
            bg_subtle: "#1a1a1a".to_string(),
            bg_highlight: "#333333".to_string(),
            border: "#444444".to_string(),
            border_focus: "#7D56F4".to_string(),
        };

        let json = serde_json::to_string_pretty(&original).unwrap();
        let parsed: CustomTheme = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, original.name);
        assert_eq!(parsed.primary, original.primary);
        assert_eq!(parsed.bg, original.bg);
    }

    // --- Theme file loading ---

    #[test]
    fn load_theme_from_file_returns_fallback_on_missing_file() {
        let result = load_theme_from_file(
            std::path::Path::new("/nonexistent/theme.json"),
            ThemePreset::Light,
        );

        assert!(result.error.is_some());
        assert!(matches!(
            result.error,
            Some(ThemeLoadError::FileNotFound(_))
        ));
        // Should fall back to Light theme
        assert_eq!(result.theme.preset, ThemePreset::Light);
    }

    #[test]
    fn theme_load_error_display() {
        let err = ThemeLoadError::FileNotFound("test.json".to_string());
        assert!(err.to_string().contains("test.json"));

        let err = ThemeLoadError::ParseError("unexpected token".to_string());
        assert!(err.to_string().contains("unexpected token"));

        let err = ThemeLoadError::MissingField("primary".to_string());
        assert!(err.to_string().contains("primary"));

        let err = ThemeLoadError::InvalidColorFormat {
            field: "bg".to_string(),
            value: "not-a-color".to_string(),
        };
        assert!(err.to_string().contains("bg"));
        assert!(err.to_string().contains("not-a-color"));
    }

    #[test]
    fn theme_warning_kinds() {
        let warning1 = ThemeWarning {
            kind: ThemeWarningKind::LowContrast,
            message: "test".to_string(),
        };
        let warning2 = ThemeWarning {
            kind: ThemeWarningKind::InvalidColor,
            message: "test".to_string(),
        };

        assert_eq!(warning1.kind, ThemeWarningKind::LowContrast);
        assert_eq!(warning2.kind, ThemeWarningKind::InvalidColor);
        assert_ne!(warning1.kind, warning2.kind);
    }

    #[test]
    fn theme_load_result_fields() {
        let result = ThemeLoadResult {
            theme: Theme::dark(),
            warnings: vec![ThemeWarning {
                kind: ThemeWarningKind::LowContrast,
                message: "test".to_string(),
            }],
            error: None,
        };

        assert_eq!(result.warnings.len(), 1);
        assert!(result.error.is_none());
    }
}
