//! Theme definitions for `demo_showcase`.
//!
//! Provides semantic color tokens and style helpers for consistent theming.

use lipgloss::{Border, Style};

/// Theme preset identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemePreset {
    #[default]
    Dark,
    Light,
    Dracula,
}

impl ThemePreset {
    /// Get the display name.
    #[must_use]
    #[allow(dead_code)] // Will be used in settings page
    pub const fn name(self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
            Self::Dracula => "Dracula",
        }
    }

    /// Get all available presets.
    #[must_use]
    #[allow(dead_code)] // Will be used in settings page
    pub const fn all() -> [Self; 3] {
        [Self::Dark, Self::Light, Self::Dracula]
    }
}

/// Semantic color tokens for the application.
///
/// Colors are stored as hex strings for direct use with lipgloss.
#[derive(Debug, Clone)]
#[allow(dead_code)] // All fields will be used as pages are implemented
pub struct Theme {
    /// Theme preset being used.
    pub preset: ThemePreset,

    // Primary colors
    pub primary: &'static str,
    pub secondary: &'static str,

    // Semantic colors
    pub success: &'static str,
    pub warning: &'static str,
    pub error: &'static str,
    pub info: &'static str,

    // Text colors
    pub text: &'static str,
    pub text_muted: &'static str,
    pub text_inverse: &'static str,

    // Background colors
    pub bg: &'static str,
    pub bg_subtle: &'static str,
    pub bg_highlight: &'static str,

    // Border colors
    pub border: &'static str,
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

    // Style helpers

    /// Style for titles/headings.
    #[must_use]
    pub fn title_style(&self) -> Style {
        Style::new().bold().foreground(self.primary)
    }

    /// Style for the sidebar.
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

    /// Style for content boxes.
    #[must_use]
    pub fn box_style(&self) -> Style {
        Style::new()
            .border(Border::rounded())
            .border_foreground(self.border)
    }

    /// Style for focused content boxes.
    #[must_use]
    pub fn box_focused_style(&self) -> Style {
        Style::new()
            .border(Border::rounded())
            .border_foreground(self.border_focus)
    }

    /// Style for muted/hint text.
    #[must_use]
    pub fn muted_style(&self) -> Style {
        Style::new().foreground(self.text_muted)
    }

    /// Style for success messages.
    #[must_use]
    pub fn success_style(&self) -> Style {
        Style::new().foreground(self.success)
    }

    /// Style for warning messages.
    #[must_use]
    pub fn warning_style(&self) -> Style {
        Style::new().foreground(self.warning)
    }

    /// Style for error messages.
    #[must_use]
    #[allow(dead_code)] // Will be used in future implementations
    pub fn error_style(&self) -> Style {
        Style::new().foreground(self.error)
    }
}
