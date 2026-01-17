//! Terminal renderer with color profile detection.

use std::io::Write;
use std::sync::OnceLock;

use crate::color::ColorProfile;

/// Global default renderer.
static DEFAULT_RENDERER: OnceLock<Renderer> = OnceLock::new();

/// Terminal renderer for lipgloss styles.
#[derive(Debug)]
pub struct Renderer {
    color_profile: ColorProfile,
    has_dark_background: bool,
}

impl Renderer {
    /// Default renderer instance.
    pub const DEFAULT: Renderer = Renderer {
        color_profile: ColorProfile::TrueColor,
        has_dark_background: true,
    };

    /// Create a new renderer with default settings.
    pub fn new() -> Self {
        Self::DEFAULT
    }

    /// Create a new renderer for the given writer.
    pub fn for_writer<W: Write>(_w: W) -> Self {
        // In a real implementation, this would detect terminal capabilities
        Self::detect()
    }

    /// Detect terminal capabilities.
    pub fn detect() -> Self {
        let color_profile = detect_color_profile();
        let has_dark_background = detect_dark_background();

        Self {
            color_profile,
            has_dark_background,
        }
    }

    /// Get the color profile.
    pub fn color_profile(&self) -> ColorProfile {
        self.color_profile
    }

    /// Set the color profile.
    pub fn set_color_profile(&mut self, profile: ColorProfile) {
        self.color_profile = profile;
    }

    /// Check if the terminal has a dark background.
    pub fn has_dark_background(&self) -> bool {
        self.has_dark_background
    }

    /// Set the dark background flag.
    pub fn set_has_dark_background(&mut self, dark: bool) {
        self.has_dark_background = dark;
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the default renderer.
pub fn default_renderer() -> &'static Renderer {
    DEFAULT_RENDERER.get_or_init(Renderer::detect)
}

/// Detect the terminal's color profile from environment.
fn detect_color_profile() -> ColorProfile {
    // Check COLORTERM for truecolor support
    if let Ok(colorterm) = std::env::var("COLORTERM") {
        if colorterm == "truecolor" || colorterm == "24bit" {
            return ColorProfile::TrueColor;
        }
    }

    // Check TERM for color support
    if let Ok(term) = std::env::var("TERM") {
        if term.contains("256color") || term.contains("256-color") {
            return ColorProfile::Ansi256;
        }
        if term.contains("color") || term.starts_with("xterm") || term.starts_with("screen") {
            return ColorProfile::Ansi;
        }
        if term == "dumb" {
            return ColorProfile::Ascii;
        }
    }

    // Check NO_COLOR
    if std::env::var("NO_COLOR").is_ok() {
        return ColorProfile::Ascii;
    }

    // Default to TrueColor for modern terminals
    ColorProfile::TrueColor
}

/// Detect if the terminal has a dark background.
fn detect_dark_background() -> bool {
    // Check COLORFGBG environment variable (format: "fg;bg")
    if let Ok(colorfgbg) = std::env::var("COLORFGBG") {
        let parts: Vec<&str> = colorfgbg.split(';').collect();
        if parts.len() >= 2 {
            if let Ok(bg) = parts[1].parse::<u8>() {
                // ANSI colors 0-7 (plus 8) are considered dark
                return bg <= 8 || bg == 0;
            }
        }
    }

    // Default to dark background (most common for terminals)
    true
}

// Public functions for global renderer access

/// Get the current color profile.
pub fn color_profile() -> ColorProfile {
    default_renderer().color_profile()
}

/// Check if the terminal has a dark background.
pub fn has_dark_background() -> bool {
    default_renderer().has_dark_background()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_default() {
        let r = Renderer::new();
        assert_eq!(r.color_profile(), ColorProfile::TrueColor);
        assert!(r.has_dark_background());
    }

    #[test]
    fn test_renderer_setters() {
        let mut r = Renderer::new();
        r.set_color_profile(ColorProfile::Ansi256);
        assert_eq!(r.color_profile(), ColorProfile::Ansi256);

        r.set_has_dark_background(false);
        assert!(!r.has_dark_background());
    }
}
