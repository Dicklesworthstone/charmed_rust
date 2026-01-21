//! Output backend abstraction for rendering styles to different targets.
//!
//! This module provides an abstraction layer that allows lipgloss to render
//! to different backends:
//! - **ANSI**: Terminal output with ANSI escape codes (default)
//! - **HTML**: HTML/CSS output for web rendering (WASM)
//! - **Plain**: Raw text without any styling
//!
//! # Example
//!
//! ```rust
//! use lipgloss::backend::{OutputBackend, AnsiBackend};
//!
//! let backend = AnsiBackend;
//! let styled = backend.apply_bold("Hello");
//! ```

use crate::color::{ColorProfile, TerminalColor};

/// Trait for output rendering backends.
///
/// Backends are responsible for converting style attributes into their
/// native representation (ANSI escape codes, HTML/CSS, etc.).
pub trait OutputBackend: Send + Sync {
    /// Apply bold styling to content.
    fn apply_bold(&self, content: &str) -> String;

    /// Apply faint/dim styling to content.
    fn apply_faint(&self, content: &str) -> String;

    /// Apply italic styling to content.
    fn apply_italic(&self, content: &str) -> String;

    /// Apply underline styling to content.
    fn apply_underline(&self, content: &str) -> String;

    /// Apply blink styling to content.
    fn apply_blink(&self, content: &str) -> String;

    /// Apply reverse/inverse styling to content.
    fn apply_reverse(&self, content: &str) -> String;

    /// Apply strikethrough styling to content.
    fn apply_strikethrough(&self, content: &str) -> String;

    /// Apply foreground color to content.
    fn apply_foreground(
        &self,
        content: &str,
        color: &dyn TerminalColor,
        profile: ColorProfile,
        dark_bg: bool,
    ) -> String;

    /// Apply background color to content.
    fn apply_background(
        &self,
        content: &str,
        color: &dyn TerminalColor,
        profile: ColorProfile,
        dark_bg: bool,
    ) -> String;

    /// Get the reset sequence for this backend.
    fn reset(&self) -> &str;

    /// Check if this backend supports the given color profile.
    fn supports_color(&self, profile: ColorProfile) -> bool;

    /// Get the newline representation for this backend.
    fn newline(&self) -> &str;

    /// Measure the display width of content (ignoring markup/escape codes).
    fn measure_width(&self, content: &str) -> usize;

    /// Strip any backend-specific markup from content, returning plain text.
    fn strip_markup(&self, content: &str) -> String;
}

/// ANSI terminal backend - renders using ANSI escape codes.
///
/// This is the default backend for terminal applications.
#[derive(Debug, Clone, Copy, Default)]
pub struct AnsiBackend;

impl AnsiBackend {
    /// ANSI escape code constants.
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const FAINT: &str = "\x1b[2m";
    pub const ITALIC: &str = "\x1b[3m";
    pub const UNDERLINE: &str = "\x1b[4m";
    pub const BLINK: &str = "\x1b[5m";
    pub const REVERSE: &str = "\x1b[7m";
    pub const STRIKETHROUGH: &str = "\x1b[9m";

    /// Create a new ANSI backend.
    pub fn new() -> Self {
        Self
    }
}

impl OutputBackend for AnsiBackend {
    fn apply_bold(&self, content: &str) -> String {
        format!("{}{}{}", Self::BOLD, content, Self::RESET)
    }

    fn apply_faint(&self, content: &str) -> String {
        format!("{}{}{}", Self::FAINT, content, Self::RESET)
    }

    fn apply_italic(&self, content: &str) -> String {
        format!("{}{}{}", Self::ITALIC, content, Self::RESET)
    }

    fn apply_underline(&self, content: &str) -> String {
        format!("{}{}{}", Self::UNDERLINE, content, Self::RESET)
    }

    fn apply_blink(&self, content: &str) -> String {
        format!("{}{}{}", Self::BLINK, content, Self::RESET)
    }

    fn apply_reverse(&self, content: &str) -> String {
        format!("{}{}{}", Self::REVERSE, content, Self::RESET)
    }

    fn apply_strikethrough(&self, content: &str) -> String {
        format!("{}{}{}", Self::STRIKETHROUGH, content, Self::RESET)
    }

    fn apply_foreground(
        &self,
        content: &str,
        color: &dyn TerminalColor,
        profile: ColorProfile,
        dark_bg: bool,
    ) -> String {
        let fg_code = color.to_ansi_fg(profile, dark_bg);
        format!("{}{}{}", fg_code, content, Self::RESET)
    }

    fn apply_background(
        &self,
        content: &str,
        color: &dyn TerminalColor,
        profile: ColorProfile,
        dark_bg: bool,
    ) -> String {
        let bg_code = color.to_ansi_bg(profile, dark_bg);
        format!("{}{}{}", bg_code, content, Self::RESET)
    }

    fn reset(&self) -> &str {
        Self::RESET
    }

    fn supports_color(&self, _profile: ColorProfile) -> bool {
        // ANSI backend supports all color profiles
        true
    }

    fn newline(&self) -> &str {
        "\n"
    }

    fn measure_width(&self, content: &str) -> usize {
        visible_width(content)
    }

    fn strip_markup(&self, content: &str) -> String {
        strip_ansi(content)
    }
}

/// Plain text backend - no styling, just raw text.
///
/// Useful for piping output or generating plain text.
#[derive(Debug, Clone, Copy, Default)]
pub struct PlainBackend;

impl PlainBackend {
    /// Create a new plain text backend.
    pub fn new() -> Self {
        Self
    }
}

impl OutputBackend for PlainBackend {
    fn apply_bold(&self, content: &str) -> String {
        content.to_string()
    }

    fn apply_faint(&self, content: &str) -> String {
        content.to_string()
    }

    fn apply_italic(&self, content: &str) -> String {
        content.to_string()
    }

    fn apply_underline(&self, content: &str) -> String {
        content.to_string()
    }

    fn apply_blink(&self, content: &str) -> String {
        content.to_string()
    }

    fn apply_reverse(&self, content: &str) -> String {
        content.to_string()
    }

    fn apply_strikethrough(&self, content: &str) -> String {
        content.to_string()
    }

    fn apply_foreground(
        &self,
        content: &str,
        _color: &dyn TerminalColor,
        _profile: ColorProfile,
        _dark_bg: bool,
    ) -> String {
        content.to_string()
    }

    fn apply_background(
        &self,
        content: &str,
        _color: &dyn TerminalColor,
        _profile: ColorProfile,
        _dark_bg: bool,
    ) -> String {
        content.to_string()
    }

    fn reset(&self) -> &str {
        ""
    }

    fn supports_color(&self, _profile: ColorProfile) -> bool {
        false
    }

    fn newline(&self) -> &str {
        "\n"
    }

    fn measure_width(&self, content: &str) -> usize {
        // Plain backend has no markup, so just measure Unicode width
        content
            .chars()
            .map(|c| unicode_width::UnicodeWidthChar::width(c).unwrap_or(0))
            .sum()
    }

    fn strip_markup(&self, content: &str) -> String {
        content.to_string()
    }
}

/// Calculate the visible width of a string (excluding ANSI escapes).
fn visible_width(s: &str) -> usize {
    let mut width = 0;
    let mut in_escape = false;

    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
            continue;
        }
        if in_escape {
            if c == 'm' {
                in_escape = false;
            }
            continue;
        }
        width += unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
    }

    width
}

/// Strip ANSI escape codes from a string.
fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_escape = false;

    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
            continue;
        }
        if in_escape {
            if c == 'm' {
                in_escape = false;
            }
            continue;
        }
        result.push(c);
    }

    result
}

// Backend selection based on target architecture

/// The default backend type for the current platform.
///
/// - On native targets: [`AnsiBackend`]
/// - On WASM targets: [`PlainBackend`] (can be overridden with HTML backend)
#[cfg(not(target_arch = "wasm32"))]
pub type DefaultBackend = AnsiBackend;

#[cfg(target_arch = "wasm32")]
pub type DefaultBackend = PlainBackend;

/// Get the default backend for the current platform.
pub fn default_backend() -> DefaultBackend {
    DefaultBackend::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;

    #[test]
    fn test_ansi_backend_bold() {
        let backend = AnsiBackend;
        let result = backend.apply_bold("test");
        assert_eq!(result, "\x1b[1mtest\x1b[0m");
    }

    #[test]
    fn test_ansi_backend_measure_width() {
        let backend = AnsiBackend;
        // Plain text
        assert_eq!(backend.measure_width("hello"), 5);
        // With ANSI codes
        assert_eq!(backend.measure_width("\x1b[1mhello\x1b[0m"), 5);
        // Unicode
        assert_eq!(backend.measure_width("你好"), 4); // 2 chars * 2 width each
    }

    #[test]
    fn test_ansi_backend_strip_markup() {
        let backend = AnsiBackend;
        let styled = "\x1b[1m\x1b[31mhello\x1b[0m";
        assert_eq!(backend.strip_markup(styled), "hello");
    }

    #[test]
    fn test_plain_backend_no_styling() {
        let backend = PlainBackend;
        assert_eq!(backend.apply_bold("test"), "test");
        assert_eq!(backend.apply_italic("test"), "test");
        assert_eq!(backend.reset(), "");
    }

    #[test]
    fn test_plain_backend_measure_width() {
        let backend = PlainBackend;
        assert_eq!(backend.measure_width("hello"), 5);
        assert_eq!(backend.measure_width("你好"), 4);
    }

    #[test]
    fn test_strip_ansi() {
        assert_eq!(strip_ansi("\x1b[1mhello\x1b[0m"), "hello");
        assert_eq!(strip_ansi("\x1b[38;5;196mred\x1b[0m"), "red");
        assert_eq!(strip_ansi("plain text"), "plain text");
    }

    #[test]
    fn test_visible_width() {
        assert_eq!(visible_width("hello"), 5);
        assert_eq!(visible_width("\x1b[1mhello\x1b[0m"), 5);
        assert_eq!(visible_width("你好世界"), 8); // 4 chars * 2 width each
    }
}
