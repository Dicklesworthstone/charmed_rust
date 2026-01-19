//! Terminal color types and color profile handling.
//!
//! This module provides various color types for terminal styling:
//! - [`NoColor`] - Absence of color
//! - [`Color`] - Hex or ANSI color string
//! - [`AnsiColor`] - ANSI color by number
//! - [`AdaptiveColor`] - Light/dark background adaptive colors
//! - [`CompleteColor`] - Explicit colors for each profile
//!
//! # Example
//!
//! ```rust
//! use lipgloss::{Color, AdaptiveColor, ColorProfile};
//!
//! // Simple hex color
//! let blue = Color::from("#0000ff");
//!
//! // Adaptive color that changes based on background
//! let adaptive = AdaptiveColor {
//!     light: "#000000".into(),
//!     dark: "#ffffff".into(),
//! };
//! ```

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{self, MapAccess, Visitor};

/// Color profile indicating terminal color capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorProfile {
    /// No color support (1-bit).
    Ascii,
    /// 16 ANSI colors (4-bit).
    Ansi,
    /// 256 colors (8-bit).
    Ansi256,
    /// True color / 16 million colors (24-bit).
    #[default]
    TrueColor,
}

impl ColorProfile {
    /// Returns true if this profile supports the given color depth.
    pub fn supports(&self, other: ColorProfile) -> bool {
        use ColorProfile::*;
        match (self, other) {
            (TrueColor, _) => true,
            (Ansi256, Ansi256 | Ansi | Ascii) => true,
            (Ansi, Ansi | Ascii) => true,
            (Ascii, Ascii) => true,
            _ => false,
        }
    }
}

/// Trait for types that can be rendered as terminal colors.
pub trait TerminalColor: fmt::Debug + Send + Sync {
    /// Convert this color to an ANSI escape sequence for the given profile.
    fn to_ansi_fg(&self, profile: ColorProfile, dark_bg: bool) -> String;

    /// Convert this color to an ANSI escape sequence for background.
    fn to_ansi_bg(&self, profile: ColorProfile, dark_bg: bool) -> String;

    /// Clone this color into a boxed trait object.
    fn clone_box(&self) -> Box<dyn TerminalColor>;
}

impl Clone for Box<dyn TerminalColor> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// No color - uses terminal's default colors.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoColor;

impl TerminalColor for NoColor {
    fn to_ansi_fg(&self, _profile: ColorProfile, _dark_bg: bool) -> String {
        String::new()
    }

    fn to_ansi_bg(&self, _profile: ColorProfile, _dark_bg: bool) -> String {
        String::new()
    }

    fn clone_box(&self) -> Box<dyn TerminalColor> {
        Box::new(*self)
    }
}

/// A color specified by hex string or ANSI number.
///
/// # Examples
///
/// ```rust
/// use lipgloss::Color;
///
/// let hex = Color::from("#ff0000");
/// let ansi = Color::from("196");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Color(pub String);

impl Color {
    /// Create a new color from a string.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Parse as RGB if this is a hex color.
    pub fn as_rgb(&self) -> Option<(u8, u8, u8)> {
        let raw = self.0.trim();
        let s = raw.trim_start_matches('#');
        let has_hash = raw.starts_with('#');
        let has_hex_alpha = s
            .chars()
            .any(|c| c.is_ascii_hexdigit() && !c.is_ascii_digit());

        if !has_hash && !has_hex_alpha {
            return None;
        }
        if s.len() == 6 {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            Some((r, g, b))
        } else if s.len() == 3 {
            let r = u8::from_str_radix(&s[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&s[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&s[2..3], 16).ok()? * 17;
            Some((r, g, b))
        } else {
            None
        }
    }

    /// Parse as ANSI color number.
    pub fn as_ansi(&self) -> Option<u8> {
        self.0.parse::<u8>().ok()
    }

    /// Returns true if this color is a valid ANSI or hex value.
    pub fn is_valid(&self) -> bool {
        if self.0.trim().is_empty() {
            return false;
        }
        self.as_rgb().is_some() || self.as_ansi().is_some()
    }
}

impl From<&str> for Color {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for Color {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl Serialize for Color {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(ColorVisitor)
    }
}

struct ColorVisitor;

impl<'de> Visitor<'de> for ColorVisitor {
    type Value = Color;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a hex string, ANSI number, or RGB map")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        parse_color_str(v).map_err(E::custom)
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
        parse_color_str(&v).map_err(E::custom)
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
        let value = u8::try_from(v)
            .map_err(|_| E::custom(format!("ANSI color must be 0-255, got {v}")))?;
        Ok(Color::new(value.to_string()))
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
        if (0..=i64::from(u8::MAX)).contains(&v) {
            Ok(Color::new(v.to_string()))
        } else {
            Err(E::custom(format!("ANSI color must be 0-255, got {v}")))
        }
    }

    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let mut r: Option<u8> = None;
        let mut g: Option<u8> = None;
        let mut b: Option<u8> = None;
        let mut a: Option<u8> = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "r" | "red" => r = Some(map.next_value()?),
                "g" | "green" => g = Some(map.next_value()?),
                "b" | "blue" => b = Some(map.next_value()?),
                "a" | "alpha" => a = Some(map.next_value()?),
                _ => {
                    let _ = map.next_value::<de::IgnoredAny>()?;
                }
            }
        }

        match (r, g, b) {
            (Some(r), Some(g), Some(b)) => {
                let _ = a;
                Ok(Color::new(format!("#{:02x}{:02x}{:02x}", r, g, b)))
            }
            _ => Err(de::Error::custom("RGB color requires r, g, b fields")),
        }
    }
}

fn parse_color_str(s: &str) -> Result<Color, String> {
    let raw = s.trim();
    if raw.is_empty() {
        return Err("color string is empty".to_string());
    }

    let has_hash = raw.starts_with('#');
    let has_hex_alpha = raw
        .chars()
        .any(|c| matches!(c, 'a'..='f' | 'A'..='F'));

    if has_hash || has_hex_alpha {
        let hex = raw.trim_start_matches('#');
        let is_hex = hex.chars().all(|c| c.is_ascii_hexdigit());
        if !is_hex || !(hex.len() == 3 || hex.len() == 6) {
            return Err(format!("invalid hex color '{raw}'"));
        }
        let normalized = if has_hash {
            raw.to_string()
        } else {
            format!("#{hex}")
        };
        return Ok(Color::new(normalized));
    }

    if raw.chars().all(|c| c.is_ascii_digit()) {
        let value: u16 = raw
            .parse()
            .map_err(|_| format!("invalid ANSI color '{raw}'"))?;
        let value = u8::try_from(value)
            .map_err(|_| format!("ANSI color must be 0-255, got {value}"))?;
        return Ok(Color::new(value.to_string()));
    }

    Err(format!("invalid color '{raw}'"))
}

impl TerminalColor for Color {
    fn to_ansi_fg(&self, profile: ColorProfile, _dark_bg: bool) -> String {
        match profile {
            ColorProfile::Ascii => String::new(),
            ColorProfile::TrueColor => {
                if let Some((r, g, b)) = self.as_rgb() {
                    format!("\x1b[38;2;{r};{g};{b}m")
                } else if let Some(n) = self.as_ansi() {
                    format!("\x1b[38;5;{n}m")
                } else {
                    String::new()
                }
            }
            ColorProfile::Ansi256 => {
                if let Some((r, g, b)) = self.as_rgb() {
                    let n = rgb_to_ansi256(r, g, b);
                    format!("\x1b[38;5;{n}m")
                } else if let Some(n) = self.as_ansi() {
                    format!("\x1b[38;5;{n}m")
                } else {
                    String::new()
                }
            }
            ColorProfile::Ansi => {
                if let Some((r, g, b)) = self.as_rgb() {
                    let n = rgb_to_ansi16(r, g, b);
                    if n < 8 {
                        format!("\x1b[{}m", 30 + n)
                    } else {
                        format!("\x1b[{}m", 90 + n - 8)
                    }
                } else if let Some(n) = self.as_ansi() {
                    if n < 8 {
                        format!("\x1b[{}m", 30 + n)
                    } else if n < 16 {
                        format!("\x1b[{}m", 90 + n - 8)
                    } else {
                        // Map 256 color to 16 color
                        let (r, g, b) = ansi256_to_rgb(n);
                        let n16 = rgb_to_ansi16(r, g, b);
                        if n16 < 8 {
                            format!("\x1b[{}m", 30 + n16)
                        } else {
                            format!("\x1b[{}m", 90 + n16 - 8)
                        }
                    }
                } else {
                    String::new()
                }
            }
        }
    }

    fn to_ansi_bg(&self, profile: ColorProfile, _dark_bg: bool) -> String {
        match profile {
            ColorProfile::Ascii => String::new(),
            ColorProfile::TrueColor => {
                if let Some((r, g, b)) = self.as_rgb() {
                    format!("\x1b[48;2;{r};{g};{b}m")
                } else if let Some(n) = self.as_ansi() {
                    format!("\x1b[48;5;{n}m")
                } else {
                    String::new()
                }
            }
            ColorProfile::Ansi256 => {
                if let Some((r, g, b)) = self.as_rgb() {
                    let n = rgb_to_ansi256(r, g, b);
                    format!("\x1b[48;5;{n}m")
                } else if let Some(n) = self.as_ansi() {
                    format!("\x1b[48;5;{n}m")
                } else {
                    String::new()
                }
            }
            ColorProfile::Ansi => {
                if let Some((r, g, b)) = self.as_rgb() {
                    let n = rgb_to_ansi16(r, g, b);
                    if n < 8 {
                        format!("\x1b[{}m", 40 + n)
                    } else {
                        format!("\x1b[{}m", 100 + n - 8)
                    }
                } else if let Some(n) = self.as_ansi() {
                    if n < 8 {
                        format!("\x1b[{}m", 40 + n)
                    } else if n < 16 {
                        format!("\x1b[{}m", 100 + n - 8)
                    } else {
                        let (r, g, b) = ansi256_to_rgb(n);
                        let n16 = rgb_to_ansi16(r, g, b);
                        if n16 < 8 {
                            format!("\x1b[{}m", 40 + n16)
                        } else {
                            format!("\x1b[{}m", 100 + n16 - 8)
                        }
                    }
                } else {
                    String::new()
                }
            }
        }
    }

    fn clone_box(&self) -> Box<dyn TerminalColor> {
        Box::new(self.clone())
    }
}

/// An ANSI color by number (0-255).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnsiColor(pub u8);

impl From<u8> for AnsiColor {
    fn from(n: u8) -> Self {
        Self(n)
    }
}

impl TerminalColor for AnsiColor {
    fn to_ansi_fg(&self, profile: ColorProfile, dark_bg: bool) -> String {
        Color(self.0.to_string()).to_ansi_fg(profile, dark_bg)
    }

    fn to_ansi_bg(&self, profile: ColorProfile, dark_bg: bool) -> String {
        Color(self.0.to_string()).to_ansi_bg(profile, dark_bg)
    }

    fn clone_box(&self) -> Box<dyn TerminalColor> {
        Box::new(*self)
    }
}

/// An RGB color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl From<(u8, u8, u8)> for RgbColor {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self { r, g, b }
    }
}

impl TerminalColor for RgbColor {
    fn to_ansi_fg(&self, profile: ColorProfile, _dark_bg: bool) -> String {
        match profile {
            ColorProfile::Ascii => String::new(),
            ColorProfile::TrueColor => {
                format!("\x1b[38;2;{};{};{}m", self.r, self.g, self.b)
            }
            ColorProfile::Ansi256 => {
                let n = rgb_to_ansi256(self.r, self.g, self.b);
                format!("\x1b[38;5;{n}m")
            }
            ColorProfile::Ansi => {
                let n = rgb_to_ansi16(self.r, self.g, self.b);
                if n < 8 {
                    format!("\x1b[{}m", 30 + n)
                } else {
                    format!("\x1b[{}m", 90 + n - 8)
                }
            }
        }
    }

    fn to_ansi_bg(&self, profile: ColorProfile, _dark_bg: bool) -> String {
        match profile {
            ColorProfile::Ascii => String::new(),
            ColorProfile::TrueColor => {
                format!("\x1b[48;2;{};{};{}m", self.r, self.g, self.b)
            }
            ColorProfile::Ansi256 => {
                let n = rgb_to_ansi256(self.r, self.g, self.b);
                format!("\x1b[48;5;{n}m")
            }
            ColorProfile::Ansi => {
                let n = rgb_to_ansi16(self.r, self.g, self.b);
                if n < 8 {
                    format!("\x1b[{}m", 40 + n)
                } else {
                    format!("\x1b[{}m", 100 + n - 8)
                }
            }
        }
    }

    fn clone_box(&self) -> Box<dyn TerminalColor> {
        Box::new(*self)
    }
}

/// A color that adapts based on terminal background.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdaptiveColor {
    /// Color to use on light backgrounds.
    pub light: Color,
    /// Color to use on dark backgrounds.
    pub dark: Color,
}

impl TerminalColor for AdaptiveColor {
    fn to_ansi_fg(&self, profile: ColorProfile, dark_bg: bool) -> String {
        if dark_bg {
            self.dark.to_ansi_fg(profile, dark_bg)
        } else {
            self.light.to_ansi_fg(profile, dark_bg)
        }
    }

    fn to_ansi_bg(&self, profile: ColorProfile, dark_bg: bool) -> String {
        if dark_bg {
            self.dark.to_ansi_bg(profile, dark_bg)
        } else {
            self.light.to_ansi_bg(profile, dark_bg)
        }
    }

    fn clone_box(&self) -> Box<dyn TerminalColor> {
        Box::new(self.clone())
    }
}

/// A color with explicit values for each color profile.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompleteColor {
    /// True color (24-bit) value.
    pub truecolor: Option<Color>,
    /// ANSI 256 (8-bit) value.
    pub ansi256: Option<Color>,
    /// ANSI 16 (4-bit) value.
    pub ansi: Option<Color>,
}

impl TerminalColor for CompleteColor {
    fn to_ansi_fg(&self, profile: ColorProfile, dark_bg: bool) -> String {
        match profile {
            ColorProfile::TrueColor => self
                .truecolor
                .as_ref()
                .map(|c| c.to_ansi_fg(profile, dark_bg))
                .unwrap_or_default(),
            ColorProfile::Ansi256 => self
                .ansi256
                .as_ref()
                .map(|c| c.to_ansi_fg(profile, dark_bg))
                .unwrap_or_default(),
            ColorProfile::Ansi => self
                .ansi
                .as_ref()
                .map(|c| c.to_ansi_fg(profile, dark_bg))
                .unwrap_or_default(),
            ColorProfile::Ascii => String::new(),
        }
    }

    fn to_ansi_bg(&self, profile: ColorProfile, dark_bg: bool) -> String {
        match profile {
            ColorProfile::TrueColor => self
                .truecolor
                .as_ref()
                .map(|c| c.to_ansi_bg(profile, dark_bg))
                .unwrap_or_default(),
            ColorProfile::Ansi256 => self
                .ansi256
                .as_ref()
                .map(|c| c.to_ansi_bg(profile, dark_bg))
                .unwrap_or_default(),
            ColorProfile::Ansi => self
                .ansi
                .as_ref()
                .map(|c| c.to_ansi_bg(profile, dark_bg))
                .unwrap_or_default(),
            ColorProfile::Ascii => String::new(),
        }
    }

    fn clone_box(&self) -> Box<dyn TerminalColor> {
        Box::new(self.clone())
    }
}

/// A complete color with adaptive light/dark variants.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompleteAdaptiveColor {
    /// Color for light backgrounds.
    pub light: CompleteColor,
    /// Color for dark backgrounds.
    pub dark: CompleteColor,
}

impl TerminalColor for CompleteAdaptiveColor {
    fn to_ansi_fg(&self, profile: ColorProfile, dark_bg: bool) -> String {
        if dark_bg {
            self.dark.to_ansi_fg(profile, dark_bg)
        } else {
            self.light.to_ansi_fg(profile, dark_bg)
        }
    }

    fn to_ansi_bg(&self, profile: ColorProfile, dark_bg: bool) -> String {
        if dark_bg {
            self.dark.to_ansi_bg(profile, dark_bg)
        } else {
            self.light.to_ansi_bg(profile, dark_bg)
        }
    }

    fn clone_box(&self) -> Box<dyn TerminalColor> {
        Box::new(self.clone())
    }
}

// Color conversion helpers

/// Convert RGB to ANSI 256 color.
pub fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    // Check for grayscale
    if r == g && g == b {
        if r < 8 {
            return 16;
        }
        if r > 248 {
            return 231;
        }
        return ((r as f64 - 8.0) / 247.0 * 24.0).round() as u8 + 232;
    }

    // Convert to 6x6x6 color cube
    let r_idx = (r as f64 / 255.0 * 5.0).round() as u8;
    let g_idx = (g as f64 / 255.0 * 5.0).round() as u8;
    let b_idx = (b as f64 / 255.0 * 5.0).round() as u8;

    16 + 36 * r_idx + 6 * g_idx + b_idx
}

/// Convert ANSI 256 to RGB.
pub fn ansi256_to_rgb(n: u8) -> (u8, u8, u8) {
    if n < 16 {
        // Standard ANSI colors
        return ANSI_COLORS[n as usize];
    }

    if n >= 232 {
        // Grayscale
        let gray = (n - 232) * 10 + 8;
        return (gray, gray, gray);
    }

    // 6x6x6 color cube
    let n = n - 16;
    let r = (n / 36) * 51;
    let g = ((n % 36) / 6) * 51;
    let b = (n % 6) * 51;

    (r, g, b)
}

/// Convert RGB to ANSI 16 color.
pub fn rgb_to_ansi16(r: u8, g: u8, b: u8) -> u8 {
    // Simple algorithm: find closest color by distance
    let mut best = 0u8;
    let mut best_dist = u32::MAX;

    for (i, &(ar, ag, ab)) in ANSI_COLORS.iter().enumerate() {
        let dr = (r as i32 - ar as i32).unsigned_abs();
        let dg = (g as i32 - ag as i32).unsigned_abs();
        let db = (b as i32 - ab as i32).unsigned_abs();
        let dist = dr * dr + dg * dg + db * db;

        if dist < best_dist {
            best_dist = dist;
            best = i as u8;
        }
    }

    best
}

/// Standard ANSI 16 colors as RGB.
const ANSI_COLORS: [(u8, u8, u8); 16] = [
    (0, 0, 0),       // Black
    (128, 0, 0),     // Red
    (0, 128, 0),     // Green
    (128, 128, 0),   // Yellow
    (0, 0, 128),     // Blue
    (128, 0, 128),   // Magenta
    (0, 128, 128),   // Cyan
    (192, 192, 192), // White
    (128, 128, 128), // Bright Black
    (255, 0, 0),     // Bright Red
    (0, 255, 0),     // Bright Green
    (255, 255, 0),   // Bright Yellow
    (0, 0, 255),     // Bright Blue
    (255, 0, 255),   // Bright Magenta
    (0, 255, 255),   // Bright Cyan
    (255, 255, 255), // Bright White
];

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_color_from_hex() {
        let c = Color::from("#ff0000");
        assert_eq!(c.as_rgb(), Some((255, 0, 0)));

        let c = Color::from("#0f0");
        assert_eq!(c.as_rgb(), Some((0, 255, 0)));
    }

    #[test]
    fn test_color_from_ansi() {
        let c = Color::from("196");
        assert_eq!(c.as_ansi(), Some(196));
    }

    #[test]
    fn test_rgb_to_ansi256() {
        // Pure red should map to 196
        let n = rgb_to_ansi256(255, 0, 0);
        assert!((196..=197).contains(&n));

        // Gray should map to grayscale range
        let n = rgb_to_ansi256(128, 128, 128);
        assert!(n >= 232);
    }

    #[test]
    fn test_ansi256_to_rgb() {
        // Black
        assert_eq!(ansi256_to_rgb(0), (0, 0, 0));
        // White
        assert_eq!(ansi256_to_rgb(15), (255, 255, 255));
    }

    #[test]
    fn test_color_profile_supports() {
        assert!(ColorProfile::TrueColor.supports(ColorProfile::Ansi256));
        assert!(ColorProfile::Ansi256.supports(ColorProfile::Ansi));
        assert!(!ColorProfile::Ansi.supports(ColorProfile::TrueColor));
    }

    #[test]
    fn test_color_serde_number() {
        let c: Color = serde_json::from_str("196").expect("parse ANSI number");
        assert_eq!(c.as_ansi(), Some(196));
        assert!(c.as_rgb().is_none());
    }

    #[test]
    fn test_color_serde_hex_without_hash() {
        let c: Color = serde_json::from_str("\"ff00ff\"").expect("parse hex string");
        assert_eq!(c.0, "#ff00ff");
        assert_eq!(c.as_rgb(), Some((255, 0, 255)));
    }

    #[test]
    fn test_color_serde_rgb_map() {
        let c: Color = serde_json::from_str("{\"r\":255,\"g\":0,\"b\":128}")
            .expect("parse rgb map");
        assert_eq!(c.0, "#ff0080");
    }
}
