//! Theme definitions for `demo_showcase`.
//!
//! Provides semantic color tokens and style helpers for consistent theming.
//! See `docs/demo_showcase/VISUAL_DESIGN.md` for the design specification.

#![allow(dead_code)] // Style helpers will be used as pages are implemented

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
        assert!(spacing::XS < spacing::SM);
        assert!(spacing::SM < spacing::MD);
        assert!(spacing::MD < spacing::LG);
        assert!(spacing::LG < spacing::XL);
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
}
