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

use crate::border::Border;
use crate::color::{ColorProfile, TerminalColor, ansi256_to_rgb};
use crate::style::{Attrs, Style};
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::RwLock;

/// Trait for output rendering backends.
///
/// Backends are responsible for converting style attributes into their
/// native representation (ANSI escape codes, HTML/CSS, etc.).
pub trait OutputBackend: Send + Sync {
    /// Render the given content with the provided style.
    fn render(&self, content: &str, style: &Style) -> String;

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
    fn reset(&self) -> &'static str;

    /// Check if this backend supports the given color profile.
    fn supports_color(&self, profile: ColorProfile) -> bool;

    /// Get the newline representation for this backend.
    fn newline(&self) -> &'static str;

    /// Measure the display width of content (ignoring markup/escape codes).
    fn measure_width(&self, content: &str) -> usize;

    /// Strip any backend-specific markup from content, returning plain text.
    fn strip_markup(&self, content: &str) -> String;

    /// Join multiple rendered segments with a separator.
    fn join(&self, segments: &[String], separator: &str) -> String {
        segments.join(separator)
    }
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
    fn render(&self, content: &str, style: &Style) -> String {
        style.render(content)
    }
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

    fn reset(&self) -> &'static str {
        Self::RESET
    }

    fn supports_color(&self, _profile: ColorProfile) -> bool {
        // ANSI backend supports all color profiles
        true
    }

    fn newline(&self) -> &'static str {
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
    fn render(&self, content: &str, style: &Style) -> String {
        strip_ansi(&style.render(content))
    }
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

    fn reset(&self) -> &'static str {
        ""
    }

    fn supports_color(&self, _profile: ColorProfile) -> bool {
        false
    }

    fn newline(&self) -> &'static str {
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

/// HTML backend - renders styled output as HTML/CSS.
#[derive(Debug)]
pub struct HtmlBackend {
    /// Whether to use inline styles instead of CSS classes.
    pub use_inline_styles: bool,
    /// Prefix for generated CSS classes.
    pub class_prefix: String,
    /// Treat the background as dark when resolving adaptive colors.
    pub dark_background: bool,
    classes: RwLock<HashMap<String, String>>,
}

impl Default for HtmlBackend {
    fn default() -> Self {
        Self {
            use_inline_styles: true,
            class_prefix: "charmed".to_string(),
            dark_background: true,
            classes: RwLock::new(HashMap::new()),
        }
    }
}

impl HtmlBackend {
    /// Create a new HTML backend with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Return a snapshot of generated classes (class -> css).
    pub fn classes(&self) -> BTreeMap<String, String> {
        self.classes
            .read()
            .map(|map| map.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default()
    }

    fn register_class(&self, css: &str) -> String {
        let mut hasher = DefaultHasher::new();
        css.hash(&mut hasher);
        let hash = hasher.finish();
        let class = format!("{}-{hash:x}", self.class_prefix);
        if let Ok(mut map) = self.classes.write() {
            map.entry(class.clone()).or_insert_with(|| css.to_string());
        }
        class
    }

    fn wrap(&self, content: &str, css: &str) -> String {
        if css.is_empty() {
            return content.to_string();
        }
        if self.use_inline_styles {
            format!(r#"<span style="{css}">{content}</span>"#)
        } else {
            let class = self.register_class(css);
            format!(r#"<span class="{class}">{content}</span>"#)
        }
    }

    fn prepare_content(style: &Style, content: &str) -> String {
        let mut text = if style.value().is_empty() {
            content.to_string()
        } else {
            format!("{} {}", style.value(), content)
        };

        if let Some(transform) = style.transform_ref() {
            text = transform(&text);
        }

        let tab_width = if style.has_custom_tab_width() {
            style.get_tab_width()
        } else {
            4
        };
        text = match tab_width {
            -1 => text,
            0 => text.replace('\t', ""),
            n => text.replace('\t', &" ".repeat(n as usize)),
        };

        text = text.replace("\r\n", "\n");

        if style.attrs().contains(Attrs::INLINE) {
            text = text.replace('\n', "");
        }

        text
    }

    fn style_to_css(&self, style: &Style) -> String {
        let mut parts: Vec<String> = Vec::new();

        let mut fg = style
            .foreground_color_ref()
            .and_then(|c| Self::color_to_css(c, self.dark_background));
        let mut bg = style
            .background_color_ref()
            .and_then(|c| Self::color_to_css(c, self.dark_background));

        let attrs = style.attrs();
        if attrs.contains(Attrs::REVERSE) {
            std::mem::swap(&mut fg, &mut bg);
        }

        if let Some(fg) = fg {
            parts.push(format!("color: {fg}"));
        }
        if let Some(bg) = bg {
            parts.push(format!("background-color: {bg}"));
        }

        if attrs.contains(Attrs::BOLD) {
            parts.push("font-weight: bold".to_string());
        }
        if attrs.contains(Attrs::ITALIC) {
            parts.push("font-style: italic".to_string());
        }
        if attrs.contains(Attrs::FAINT) {
            parts.push("opacity: 0.7".to_string());
        }

        let underline = attrs.contains(Attrs::UNDERLINE);
        let strike = attrs.contains(Attrs::STRIKETHROUGH);
        if underline || strike {
            let mut deco = Vec::new();
            if underline {
                deco.push("underline");
            }
            if strike {
                deco.push("line-through");
            }
            parts.push(format!("text-decoration: {}", deco.join(" ")));
        }

        let padding = style.get_padding();
        if padding.top > 0 || padding.right > 0 || padding.bottom > 0 || padding.left > 0 {
            let top = padding.top as f32 * 1.2;
            let bottom = padding.bottom as f32 * 1.2;
            parts.push(format!(
                "padding: {top:.2}em {}ch {bottom:.2}em {}ch",
                padding.right, padding.left
            ));
        }

        let margin = style.get_margin();
        if margin.top > 0 || margin.right > 0 || margin.bottom > 0 || margin.left > 0 {
            let top = margin.top as f32 * 1.2;
            let bottom = margin.bottom as f32 * 1.2;
            parts.push(format!(
                "margin: {top:.2}em {}ch {bottom:.2}em {}ch",
                margin.right, margin.left
            ));
        }

        if let Some(width) = style.get_width() {
            parts.push("display: inline-block".to_string());
            parts.push(format!("width: {width}ch"));
        }
        if let Some(height) = style.get_height() {
            parts.push("display: inline-block".to_string());
            parts.push(format!("height: {:.2}em", height as f32 * 1.2));
        }

        match style.get_align_horizontal() {
            crate::position::Position::Center => parts.push("text-align: center".to_string()),
            crate::position::Position::Right | crate::position::Position::Bottom => {
                parts.push("text-align: right".to_string());
            }
            _ => {}
        }

        self.border_to_css(style, &mut parts);

        parts.join("; ")
    }

    fn border_to_css(&self, style: &Style, parts: &mut Vec<String>) {
        let edges = style.effective_border_edges();
        let border = style.border_style_ref();
        if !edges.any() || border.is_empty() {
            return;
        }

        let (border_style, border_width, border_radius) = border_css_hint(border);

        let default_color = "currentColor".to_string();
        let top_color = style
            .border_fg_ref(0)
            .and_then(|c| Self::color_to_css(c, self.dark_background))
            .unwrap_or_else(|| default_color.clone());
        let right_color = style
            .border_fg_ref(1)
            .and_then(|c| Self::color_to_css(c, self.dark_background))
            .unwrap_or_else(|| default_color.clone());
        let bottom_color = style
            .border_fg_ref(2)
            .and_then(|c| Self::color_to_css(c, self.dark_background))
            .unwrap_or_else(|| default_color.clone());
        let left_color = style
            .border_fg_ref(3)
            .and_then(|c| Self::color_to_css(c, self.dark_background))
            .unwrap_or_else(|| default_color.clone());

        if edges.is_all()
            && top_color == right_color
            && top_color == bottom_color
            && top_color == left_color
        {
            parts.push(format!("border: {border_width} {border_style} {top_color}"));
        } else {
            if edges.top {
                parts.push(format!(
                    "border-top: {border_width} {border_style} {top_color}"
                ));
            }
            if edges.right {
                parts.push(format!(
                    "border-right: {border_width} {border_style} {right_color}"
                ));
            }
            if edges.bottom {
                parts.push(format!(
                    "border-bottom: {border_width} {border_style} {bottom_color}"
                ));
            }
            if edges.left {
                parts.push(format!(
                    "border-left: {border_width} {border_style} {left_color}"
                ));
            }
        }

        if let Some(radius) = border_radius {
            parts.push(format!("border-radius: {radius}"));
        }
    }

    fn color_to_css(color: &dyn TerminalColor, dark_bg: bool) -> Option<String> {
        let seq = color.to_ansi_fg(ColorProfile::TrueColor, dark_bg);
        parse_ansi_color_sequence(&seq)
    }
}

impl OutputBackend for HtmlBackend {
    fn render(&self, content: &str, style: &Style) -> String {
        let prepared = Self::prepare_content(style, content);
        let escaped = html_escape(&prepared);
        let css = self.style_to_css(style);
        self.wrap(&escaped, &css)
    }

    fn apply_bold(&self, content: &str) -> String {
        self.wrap(content, "font-weight: bold")
    }

    fn apply_faint(&self, content: &str) -> String {
        self.wrap(content, "opacity: 0.7")
    }

    fn apply_italic(&self, content: &str) -> String {
        self.wrap(content, "font-style: italic")
    }

    fn apply_underline(&self, content: &str) -> String {
        self.wrap(content, "text-decoration: underline")
    }

    fn apply_blink(&self, content: &str) -> String {
        self.wrap(content, "text-decoration: blink")
    }

    fn apply_reverse(&self, content: &str) -> String {
        self.wrap(content, "filter: invert(1)")
    }

    fn apply_strikethrough(&self, content: &str) -> String {
        self.wrap(content, "text-decoration: line-through")
    }

    fn apply_foreground(
        &self,
        content: &str,
        color: &dyn TerminalColor,
        _profile: ColorProfile,
        dark_bg: bool,
    ) -> String {
        let css = Self::color_to_css(color, dark_bg)
            .map(|c| format!("color: {c}"))
            .unwrap_or_default();
        self.wrap(content, &css)
    }

    fn apply_background(
        &self,
        content: &str,
        color: &dyn TerminalColor,
        _profile: ColorProfile,
        dark_bg: bool,
    ) -> String {
        let css = Self::color_to_css(color, dark_bg)
            .map(|c| format!("background-color: {c}"))
            .unwrap_or_default();
        self.wrap(content, &css)
    }

    fn reset(&self) -> &'static str {
        ""
    }

    fn supports_color(&self, _profile: ColorProfile) -> bool {
        true
    }

    fn newline(&self) -> &'static str {
        "<br>"
    }

    fn measure_width(&self, content: &str) -> usize {
        let plain = strip_html_tags(content);
        plain
            .chars()
            .map(|c| unicode_width::UnicodeWidthChar::width(c).unwrap_or(0))
            .sum()
    }

    fn strip_markup(&self, content: &str) -> String {
        strip_html_tags(content)
    }

    fn join(&self, segments: &[String], separator: &str) -> String {
        let sep = html_escape(separator);
        segments.join(&sep)
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

// HTML helpers

fn html_escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            '\n' => out.push_str("<br>"),
            '\t' => out.push_str("&nbsp;&nbsp;&nbsp;&nbsp;"),
            ' ' => out.push_str("&nbsp;"),
            _ => out.push(ch),
        }
    }
    out
}

fn strip_html_tags(input: &str) -> String {
    let normalized = input
        .replace("<br />", "\n")
        .replace("<br/>", "\n")
        .replace("<br>", "\n");
    let mut out = String::new();
    let mut in_tag = false;
    let mut in_entity = false;
    let mut entity = String::new();

    for ch in normalized.chars() {
        if in_tag {
            if ch == '>' {
                in_tag = false;
            }
            continue;
        }

        if in_entity {
            if ch == ';' {
                if let Some(decoded) = decode_entity(&entity) {
                    out.push(decoded);
                }
                entity.clear();
                in_entity = false;
            } else {
                entity.push(ch);
            }
            continue;
        }

        match ch {
            '<' => in_tag = true,
            '&' => in_entity = true,
            _ => out.push(ch),
        }
    }

    out
}

fn decode_entity(entity: &str) -> Option<char> {
    match entity {
        "nbsp" => Some(' '),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "amp" => Some('&'),
        "quot" => Some('"'),
        "#39" => Some('\''),
        _ => {
            if let Some(rest) = entity.strip_prefix("#x") {
                u32::from_str_radix(rest, 16).ok().and_then(char::from_u32)
            } else if let Some(rest) = entity.strip_prefix('#') {
                rest.parse::<u32>().ok().and_then(char::from_u32)
            } else {
                None
            }
        }
    }
}

fn parse_ansi_color_sequence(seq: &str) -> Option<String> {
    let seq = seq.strip_prefix("\x1b[")?;
    let seq = seq.strip_suffix('m')?;
    let parts: Vec<&str> = seq.split(';').collect();

    if parts.len() >= 5 && (parts[0] == "38" || parts[0] == "48") && parts[1] == "2" {
        let r = parts[2].parse::<u8>().ok()?;
        let g = parts[3].parse::<u8>().ok()?;
        let b = parts[4].parse::<u8>().ok()?;
        return Some(format!("#{:02x}{:02x}{:02x}", r, g, b));
    }

    if parts.len() >= 3 && (parts[0] == "38" || parts[0] == "48") && parts[1] == "5" {
        let n = parts[2].parse::<u8>().ok()?;
        let (r, g, b) = ansi256_to_rgb(n);
        return Some(format!("#{:02x}{:02x}{:02x}", r, g, b));
    }

    if parts.len() == 1 {
        let code = parts[0].parse::<u8>().ok()?;
        let idx = match code {
            30..=37 => code - 30,
            90..=97 => 8 + (code - 90),
            _ => return None,
        };
        let (r, g, b) = ansi256_to_rgb(idx);
        return Some(format!("#{:02x}{:02x}{:02x}", r, g, b));
    }

    None
}

fn border_css_hint(border: &Border) -> (&'static str, &'static str, Option<&'static str>) {
    let parts = [
        border.top.as_str(),
        border.bottom.as_str(),
        border.left.as_str(),
        border.right.as_str(),
        border.top_left.as_str(),
        border.top_right.as_str(),
        border.bottom_left.as_str(),
        border.bottom_right.as_str(),
    ];

    let has_any = parts.iter().any(|s| !s.is_empty());
    if !has_any {
        return ("none", "0", None);
    }

    let is_double = parts
        .iter()
        .any(|s| s.contains('═') || s.contains('║') || s.contains('╔') || s.contains('╗'));
    if is_double {
        return ("double", "3px", None);
    }

    let is_thick = parts.iter().any(|s| {
        s.contains('━')
            || s.contains('┃')
            || s.contains('┏')
            || s.contains('┓')
            || s.contains('┗')
            || s.contains('┛')
    });
    if is_thick {
        return ("solid", "3px", None);
    }

    let is_block = parts
        .iter()
        .any(|s| s.contains('█') || s.contains('▌') || s.contains('▐'));
    if is_block {
        return ("solid", "2px", None);
    }

    let is_rounded = parts
        .iter()
        .any(|s| s.contains('╭') || s.contains('╮') || s.contains('╰'));
    if is_rounded {
        return ("solid", "1px", Some("0.5em"));
    }

    ("solid", "1px", None)
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
