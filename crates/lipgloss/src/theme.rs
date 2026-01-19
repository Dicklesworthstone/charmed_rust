//! Theme system with semantic color slots.
//!
//! The [`Theme`] struct provides semantic color slots that components can reference
//! for consistent styling across an application. Themes support light/dark variants
//! and can be serialized for user configuration.
//!
//! # Example
//!
//! ```rust
//! use lipgloss::theme::{Theme, ThemeColors};
//!
//! // Use the default dark theme
//! let theme = Theme::dark();
//!
//! // Create a style using theme colors
//! let style = theme.style()
//!     .foreground_color(theme.colors().primary.clone())
//!     .background_color(theme.colors().background.clone());
//! ```

use crate::color::{AdaptiveColor, Color};
use crate::style::Style;

/// A complete theme with semantic color slots.
///
/// Themes provide a consistent color palette that components can reference
/// by semantic meaning (e.g., "primary", "error") rather than raw color values.
/// This enables easy theme switching and ensures visual consistency.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Human-readable name for this theme.
    name: String,

    /// Whether this is a dark theme (affects adaptive color selection).
    is_dark: bool,

    /// The color palette.
    colors: ThemeColors,
}

/// Semantic color slots for a theme.
///
/// Each slot represents a semantic purpose rather than a specific color.
/// This allows the same code to work with different themes while maintaining
/// appropriate visual meaning.
#[derive(Debug, Clone)]
pub struct ThemeColors {
    // ========================
    // Primary Palette
    // ========================
    /// Primary brand/accent color. Used for primary actions, links, and emphasis.
    pub primary: Color,

    /// Secondary color. Used for secondary actions and less prominent elements.
    pub secondary: Color,

    /// Accent color. Used for highlights, indicators, and visual interest.
    pub accent: Color,

    // ========================
    // Background Colors
    // ========================
    /// Main background color.
    pub background: Color,

    /// Elevated surface color (cards, dialogs, popups).
    pub surface: Color,

    /// Alternative surface for visual layering.
    pub surface_alt: Color,

    // ========================
    // Text Colors
    // ========================
    /// Primary text color (high contrast, main content).
    pub text: Color,

    /// Muted text color (secondary content, descriptions).
    pub text_muted: Color,

    /// Disabled text color (inactive elements).
    pub text_disabled: Color,

    // ========================
    // Semantic Colors
    // ========================
    /// Success/positive color (confirmations, success states).
    pub success: Color,

    /// Warning color (cautions, alerts).
    pub warning: Color,

    /// Error/danger color (errors, destructive actions).
    pub error: Color,

    /// Info color (informational messages, neutral highlights).
    pub info: Color,

    // ========================
    // UI Element Colors
    // ========================
    /// Border color for UI elements.
    pub border: Color,

    /// Subtle border color (dividers, separators).
    pub border_muted: Color,

    /// Separator/divider color.
    pub separator: Color,

    // ========================
    // Interactive States
    // ========================
    /// Focus indicator color.
    pub focus: Color,

    /// Selection/highlight background color.
    pub selection: Color,

    /// Hover state color.
    pub hover: Color,

    // ========================
    // Code/Syntax Colors
    // ========================
    /// Code/syntax: Keywords (if, else, fn, etc.)
    pub code_keyword: Color,

    /// Code/syntax: Strings
    pub code_string: Color,

    /// Code/syntax: Numbers
    pub code_number: Color,

    /// Code/syntax: Comments
    pub code_comment: Color,

    /// Code/syntax: Function names
    pub code_function: Color,

    /// Code/syntax: Types/classes
    pub code_type: Color,

    /// Code/syntax: Variables
    pub code_variable: Color,

    /// Code/syntax: Operators
    pub code_operator: Color,
}

impl Theme {
    /// Creates a new theme with the given name, dark mode flag, and colors.
    pub fn new(name: impl Into<String>, is_dark: bool, colors: ThemeColors) -> Self {
        Self {
            name: name.into(),
            is_dark,
            colors,
        }
    }

    /// Returns the theme name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns true if this is a dark theme.
    pub fn is_dark(&self) -> bool {
        self.is_dark
    }

    /// Returns the theme's color palette.
    pub fn colors(&self) -> &ThemeColors {
        &self.colors
    }

    /// Returns a mutable reference to the theme's color palette.
    pub fn colors_mut(&mut self) -> &mut ThemeColors {
        &mut self.colors
    }

    /// Creates a new Style configured to use this theme.
    ///
    /// The returned style has no properties set but is configured to use
    /// this theme's renderer settings.
    pub fn style(&self) -> Style {
        Style::new()
    }

    // ========================
    // Builder Methods
    // ========================

    /// Sets the theme name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets whether this is a dark theme.
    pub fn with_dark(mut self, is_dark: bool) -> Self {
        self.is_dark = is_dark;
        self
    }

    /// Replaces the color palette.
    pub fn with_colors(mut self, colors: ThemeColors) -> Self {
        self.colors = colors;
        self
    }

    // ========================
    // Default Themes
    // ========================

    /// Returns the default dark theme.
    ///
    /// This theme uses colors suitable for dark terminal backgrounds.
    pub fn dark() -> Self {
        Self::new("Dark", true, ThemeColors::dark())
    }

    /// Returns the default light theme.
    ///
    /// This theme uses colors suitable for light terminal backgrounds.
    pub fn light() -> Self {
        Self::new("Light", false, ThemeColors::light())
    }

    /// Returns the Dracula theme.
    ///
    /// A popular dark theme with purple accents.
    /// <https://draculatheme.com>
    pub fn dracula() -> Self {
        Self::new("Dracula", true, ThemeColors::dracula())
    }

    /// Returns the Nord theme.
    ///
    /// An arctic, north-bluish color palette.
    /// <https://www.nordtheme.com>
    pub fn nord() -> Self {
        Self::new("Nord", true, ThemeColors::nord())
    }

    /// Returns the Catppuccin Mocha theme.
    ///
    /// A soothing pastel theme with warm tones.
    /// <https://catppuccin.com>
    pub fn catppuccin_mocha() -> Self {
        Self::new("Catppuccin Mocha", true, ThemeColors::catppuccin_mocha())
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl ThemeColors {
    /// Creates a new `ThemeColors` with all slots set to the same color.
    ///
    /// Useful as a starting point for building custom themes.
    pub fn uniform(color: impl Into<Color>) -> Self {
        let c = color.into();
        Self {
            primary: c.clone(),
            secondary: c.clone(),
            accent: c.clone(),
            background: c.clone(),
            surface: c.clone(),
            surface_alt: c.clone(),
            text: c.clone(),
            text_muted: c.clone(),
            text_disabled: c.clone(),
            success: c.clone(),
            warning: c.clone(),
            error: c.clone(),
            info: c.clone(),
            border: c.clone(),
            border_muted: c.clone(),
            separator: c.clone(),
            focus: c.clone(),
            selection: c.clone(),
            hover: c.clone(),
            code_keyword: c.clone(),
            code_string: c.clone(),
            code_number: c.clone(),
            code_comment: c.clone(),
            code_function: c.clone(),
            code_type: c.clone(),
            code_variable: c.clone(),
            code_operator: c,
        }
    }

    /// Returns the default dark color palette.
    pub fn dark() -> Self {
        Self {
            // Primary palette
            primary: Color::from("#7c3aed"),   // Violet
            secondary: Color::from("#6366f1"), // Indigo
            accent: Color::from("#22d3ee"),    // Cyan

            // Backgrounds
            background: Color::from("#0f0f0f"),  // Near black
            surface: Color::from("#1a1a1a"),     // Dark gray
            surface_alt: Color::from("#262626"), // Slightly lighter

            // Text
            text: Color::from("#fafafa"),          // Near white
            text_muted: Color::from("#a1a1aa"),    // Gray
            text_disabled: Color::from("#52525b"), // Darker gray

            // Semantic
            success: Color::from("#22c55e"), // Green
            warning: Color::from("#f59e0b"), // Amber
            error: Color::from("#ef4444"),   // Red
            info: Color::from("#3b82f6"),    // Blue

            // UI elements
            border: Color::from("#3f3f46"),       // Zinc-700
            border_muted: Color::from("#27272a"), // Zinc-800
            separator: Color::from("#27272a"),    // Same as border_muted

            // Interactive
            focus: Color::from("#7c3aed"),     // Same as primary
            selection: Color::from("#4c1d95"), // Dark violet
            hover: Color::from("#27272a"),     // Subtle highlight

            // Code/syntax (based on popular dark themes)
            code_keyword: Color::from("#c678dd"),  // Purple
            code_string: Color::from("#98c379"),   // Green
            code_number: Color::from("#d19a66"),   // Orange
            code_comment: Color::from("#5c6370"),  // Gray
            code_function: Color::from("#61afef"), // Blue
            code_type: Color::from("#e5c07b"),     // Yellow
            code_variable: Color::from("#e06c75"), // Red/pink
            code_operator: Color::from("#56b6c2"), // Cyan
        }
    }

    /// Returns the default light color palette.
    pub fn light() -> Self {
        Self {
            // Primary palette
            primary: Color::from("#7c3aed"),   // Violet
            secondary: Color::from("#4f46e5"), // Indigo
            accent: Color::from("#0891b2"),    // Cyan (darker for light bg)

            // Backgrounds
            background: Color::from("#ffffff"),  // White
            surface: Color::from("#f4f4f5"),     // Zinc-100
            surface_alt: Color::from("#e4e4e7"), // Zinc-200

            // Text
            text: Color::from("#18181b"),          // Zinc-900
            text_muted: Color::from("#71717a"),    // Zinc-500
            text_disabled: Color::from("#a1a1aa"), // Zinc-400

            // Semantic
            success: Color::from("#16a34a"), // Green-600
            warning: Color::from("#d97706"), // Amber-600
            error: Color::from("#dc2626"),   // Red-600
            info: Color::from("#2563eb"),    // Blue-600

            // UI elements
            border: Color::from("#d4d4d8"),       // Zinc-300
            border_muted: Color::from("#e4e4e7"), // Zinc-200
            separator: Color::from("#e4e4e7"),    // Same as border_muted

            // Interactive
            focus: Color::from("#7c3aed"),     // Same as primary
            selection: Color::from("#ddd6fe"), // Light violet
            hover: Color::from("#f4f4f5"),     // Subtle highlight

            // Code/syntax (based on popular light themes)
            code_keyword: Color::from("#a626a4"),  // Purple
            code_string: Color::from("#50a14f"),   // Green
            code_number: Color::from("#986801"),   // Orange/brown
            code_comment: Color::from("#a0a1a7"),  // Gray
            code_function: Color::from("#4078f2"), // Blue
            code_type: Color::from("#c18401"),     // Yellow/gold
            code_variable: Color::from("#e45649"), // Red
            code_operator: Color::from("#0184bc"), // Cyan
        }
    }

    /// Returns the Dracula color palette.
    pub fn dracula() -> Self {
        // Dracula theme colors from https://draculatheme.com
        Self {
            primary: Color::from("#bd93f9"),   // Purple
            secondary: Color::from("#ff79c6"), // Pink
            accent: Color::from("#8be9fd"),    // Cyan

            background: Color::from("#282a36"),  // Background
            surface: Color::from("#44475a"),     // Current Line
            surface_alt: Color::from("#6272a4"), // Comment

            text: Color::from("#f8f8f2"),          // Foreground
            text_muted: Color::from("#6272a4"),    // Comment
            text_disabled: Color::from("#44475a"), // Current Line

            success: Color::from("#50fa7b"), // Green
            warning: Color::from("#ffb86c"), // Orange
            error: Color::from("#ff5555"),   // Red
            info: Color::from("#8be9fd"),    // Cyan

            border: Color::from("#44475a"),       // Current Line
            border_muted: Color::from("#282a36"), // Background
            separator: Color::from("#44475a"),    // Current Line

            focus: Color::from("#bd93f9"),     // Purple
            selection: Color::from("#44475a"), // Current Line
            hover: Color::from("#44475a"),     // Current Line

            code_keyword: Color::from("#ff79c6"),  // Pink
            code_string: Color::from("#f1fa8c"),   // Yellow
            code_number: Color::from("#bd93f9"),   // Purple
            code_comment: Color::from("#6272a4"),  // Comment
            code_function: Color::from("#50fa7b"), // Green
            code_type: Color::from("#8be9fd"),     // Cyan
            code_variable: Color::from("#f8f8f2"), // Foreground
            code_operator: Color::from("#ff79c6"), // Pink
        }
    }

    /// Returns the Nord color palette.
    pub fn nord() -> Self {
        // Nord theme colors from https://www.nordtheme.com
        Self {
            primary: Color::from("#88c0d0"),   // Nord8 (cyan)
            secondary: Color::from("#81a1c1"), // Nord9 (blue)
            accent: Color::from("#b48ead"),    // Nord15 (purple)

            background: Color::from("#2e3440"),  // Nord0
            surface: Color::from("#3b4252"),     // Nord1
            surface_alt: Color::from("#434c5e"), // Nord2

            text: Color::from("#eceff4"),          // Nord6
            text_muted: Color::from("#d8dee9"),    // Nord4
            text_disabled: Color::from("#4c566a"), // Nord3

            success: Color::from("#a3be8c"), // Nord14 (green)
            warning: Color::from("#ebcb8b"), // Nord13 (yellow)
            error: Color::from("#bf616a"),   // Nord11 (red)
            info: Color::from("#5e81ac"),    // Nord10 (blue)

            border: Color::from("#4c566a"),       // Nord3
            border_muted: Color::from("#3b4252"), // Nord1
            separator: Color::from("#3b4252"),    // Nord1

            focus: Color::from("#88c0d0"),     // Nord8
            selection: Color::from("#434c5e"), // Nord2
            hover: Color::from("#3b4252"),     // Nord1

            code_keyword: Color::from("#81a1c1"),  // Nord9
            code_string: Color::from("#a3be8c"),   // Nord14
            code_number: Color::from("#b48ead"),   // Nord15
            code_comment: Color::from("#616e88"),  // Muted Nord
            code_function: Color::from("#88c0d0"), // Nord8
            code_type: Color::from("#8fbcbb"),     // Nord7
            code_variable: Color::from("#d8dee9"), // Nord4
            code_operator: Color::from("#81a1c1"), // Nord9
        }
    }

    /// Returns the Catppuccin Mocha color palette.
    pub fn catppuccin_mocha() -> Self {
        // Catppuccin Mocha colors from https://catppuccin.com
        Self {
            primary: Color::from("#cba6f7"),   // Mauve
            secondary: Color::from("#89b4fa"), // Blue
            accent: Color::from("#f5c2e7"),    // Pink

            background: Color::from("#1e1e2e"),  // Base
            surface: Color::from("#313244"),     // Surface0
            surface_alt: Color::from("#45475a"), // Surface1

            text: Color::from("#cdd6f4"),          // Text
            text_muted: Color::from("#a6adc8"),    // Subtext0
            text_disabled: Color::from("#6c7086"), // Overlay0

            success: Color::from("#a6e3a1"), // Green
            warning: Color::from("#f9e2af"), // Yellow
            error: Color::from("#f38ba8"),   // Red
            info: Color::from("#89dceb"),    // Sky

            border: Color::from("#45475a"),       // Surface1
            border_muted: Color::from("#313244"), // Surface0
            separator: Color::from("#313244"),    // Surface0

            focus: Color::from("#cba6f7"),     // Mauve
            selection: Color::from("#45475a"), // Surface1
            hover: Color::from("#313244"),     // Surface0

            code_keyword: Color::from("#cba6f7"),  // Mauve
            code_string: Color::from("#a6e3a1"),   // Green
            code_number: Color::from("#fab387"),   // Peach
            code_comment: Color::from("#6c7086"),  // Overlay0
            code_function: Color::from("#89b4fa"), // Blue
            code_type: Color::from("#f9e2af"),     // Yellow
            code_variable: Color::from("#f5c2e7"), // Pink
            code_operator: Color::from("#89dceb"), // Sky
        }
    }
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self::dark()
    }
}

/// Creates an adaptive color from a theme's light and dark colors.
///
/// This is useful for creating colors that work correctly in both
/// light and dark terminal environments.
pub fn adaptive(
    light: &ThemeColors,
    dark: &ThemeColors,
    slot: impl Fn(&ThemeColors) -> &Color,
) -> AdaptiveColor {
    AdaptiveColor {
        light: slot(light).clone(),
        dark: slot(dark).clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_dark_default() {
        let theme = Theme::dark();
        assert!(theme.is_dark());
        assert_eq!(theme.name(), "Dark");
    }

    #[test]
    fn test_theme_light_default() {
        let theme = Theme::light();
        assert!(!theme.is_dark());
        assert_eq!(theme.name(), "Light");
    }

    #[test]
    fn test_theme_dracula() {
        let theme = Theme::dracula();
        assert!(theme.is_dark());
        assert_eq!(theme.name(), "Dracula");
        // Dracula's background is #282a36
        assert_eq!(theme.colors().background.0, "#282a36");
    }

    #[test]
    fn test_theme_nord() {
        let theme = Theme::nord();
        assert!(theme.is_dark());
        assert_eq!(theme.name(), "Nord");
        // Nord's background is #2e3440
        assert_eq!(theme.colors().background.0, "#2e3440");
    }

    #[test]
    fn test_theme_catppuccin() {
        let theme = Theme::catppuccin_mocha();
        assert!(theme.is_dark());
        assert_eq!(theme.name(), "Catppuccin Mocha");
        // Catppuccin Mocha's background is #1e1e2e
        assert_eq!(theme.colors().background.0, "#1e1e2e");
    }

    #[test]
    fn test_theme_builder() {
        let theme = Theme::dark().with_name("Custom Dark").with_dark(true);
        assert_eq!(theme.name(), "Custom Dark");
        assert!(theme.is_dark());
    }

    #[test]
    fn test_theme_colors_uniform() {
        let colors = ThemeColors::uniform("#ff0000");
        assert_eq!(colors.primary.0, "#ff0000");
        assert_eq!(colors.background.0, "#ff0000");
        assert_eq!(colors.text.0, "#ff0000");
    }

    #[test]
    fn test_adaptive_color() {
        let light = ThemeColors::light();
        let dark = ThemeColors::dark();

        let adaptive_text = adaptive(&light, &dark, |c| &c.text);

        // Light theme text is dark, dark theme text is light
        assert_eq!(adaptive_text.light.0, light.text.0);
        assert_eq!(adaptive_text.dark.0, dark.text.0);
    }

    #[test]
    fn test_theme_style() {
        let theme = Theme::dark();
        let style = theme.style();
        // Style should be empty/default
        assert!(style.value().is_empty());
    }

    #[test]
    fn test_color_slots_all_defined() {
        // Ensure all themes have all color slots defined (not empty)
        for theme in [
            Theme::dark(),
            Theme::light(),
            Theme::dracula(),
            Theme::nord(),
            Theme::catppuccin_mocha(),
        ] {
            let c = theme.colors();

            // All colors should have non-empty values
            assert!(!c.primary.0.is_empty(), "{}: primary empty", theme.name());
            assert!(
                !c.secondary.0.is_empty(),
                "{}: secondary empty",
                theme.name()
            );
            assert!(!c.accent.0.is_empty(), "{}: accent empty", theme.name());
            assert!(
                !c.background.0.is_empty(),
                "{}: background empty",
                theme.name()
            );
            assert!(!c.surface.0.is_empty(), "{}: surface empty", theme.name());
            assert!(!c.text.0.is_empty(), "{}: text empty", theme.name());
            assert!(!c.error.0.is_empty(), "{}: error empty", theme.name());
        }
    }
}
