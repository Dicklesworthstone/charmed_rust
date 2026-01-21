//! WebAssembly bindings for lipgloss.
//!
//! This module provides JavaScript-friendly APIs for lipgloss when compiled
//! to WebAssembly. It exposes styling capabilities through wasm-bindgen.
//!
//! # Example (JavaScript)
//!
//! ```javascript
//! import init, { newStyle, joinVertical, Position } from 'lipgloss';
//!
//! await init();
//!
//! const style = newStyle()
//!     .foreground("#ff00ff")
//!     .background("#1a1a1a")
//!     .bold()
//!     .padding(1, 2, 1, 2);
//!
//! const rendered = style.render("Hello, World!");
//! document.body.innerHTML = rendered;
//! ```

use wasm_bindgen::prelude::*;

use crate::backend::HtmlBackend;
use crate::color::Color;
use crate::position::Position;
use crate::style::Style;
use crate::Border;

/// Initialize the lipgloss WASM module.
///
/// This sets up the panic hook for better error messages in the browser console.
/// Call this once when your application starts.
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Convert a f64 position value to Position enum.
/// 0.0 = Left/Top, 0.5 = Center, 1.0 = Right/Bottom
fn f64_to_position(value: f64) -> Position {
    if value <= 0.25 {
        Position::Left
    } else if value >= 0.75 {
        Position::Right
    } else {
        Position::Center
    }
}

/// Create a new style builder.
///
/// Returns a new `JsStyle` that can be configured with chainable methods.
#[wasm_bindgen(js_name = "newStyle")]
pub fn new_style() -> JsStyle {
    JsStyle::new()
}

/// JavaScript-friendly wrapper for Style.
///
/// This provides a chainable API suitable for JavaScript usage, with methods
/// that return a new JsStyle for method chaining.
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct JsStyle {
    inner: Style,
}

#[wasm_bindgen]
impl JsStyle {
    /// Create a new empty style.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Style::new(),
        }
    }

    /// Set the foreground color using a hex string (e.g., "#ff00ff").
    #[wasm_bindgen]
    pub fn foreground(self, color: &str) -> Self {
        Self {
            inner: self.inner.foreground(color),
        }
    }

    /// Set the background color using a hex string (e.g., "#1a1a1a").
    #[wasm_bindgen]
    pub fn background(self, color: &str) -> Self {
        Self {
            inner: self.inner.background(color),
        }
    }

    /// Enable bold text.
    #[wasm_bindgen]
    pub fn bold(self) -> Self {
        Self {
            inner: self.inner.bold(),
        }
    }

    /// Enable italic text.
    #[wasm_bindgen]
    pub fn italic(self) -> Self {
        Self {
            inner: self.inner.italic(),
        }
    }

    /// Enable underlined text.
    #[wasm_bindgen]
    pub fn underline(self) -> Self {
        Self {
            inner: self.inner.underline(),
        }
    }

    /// Enable strikethrough text.
    #[wasm_bindgen]
    pub fn strikethrough(self) -> Self {
        Self {
            inner: self.inner.strikethrough(),
        }
    }

    /// Enable faint/dim text.
    #[wasm_bindgen]
    pub fn faint(self) -> Self {
        Self {
            inner: self.inner.faint(),
        }
    }

    /// Enable reverse video (swap foreground and background).
    #[wasm_bindgen]
    pub fn reverse(self) -> Self {
        Self {
            inner: self.inner.reverse(),
        }
    }

    /// Set padding on all sides.
    ///
    /// Can be called with:
    /// - 1 argument: all sides
    /// - 2 arguments: vertical, horizontal
    /// - 4 arguments: top, right, bottom, left
    #[wasm_bindgen(js_name = "paddingAll")]
    pub fn padding_all(self, value: u16) -> Self {
        Self {
            inner: self.inner.padding(value),
        }
    }

    /// Set padding with vertical and horizontal values.
    #[wasm_bindgen(js_name = "paddingVH")]
    pub fn padding_vh(self, vertical: u16, horizontal: u16) -> Self {
        Self {
            inner: self.inner.padding((vertical, horizontal)),
        }
    }

    /// Set padding for all four sides individually.
    #[wasm_bindgen]
    pub fn padding(self, top: u16, right: u16, bottom: u16, left: u16) -> Self {
        Self {
            inner: self.inner.padding((top, right, bottom, left)),
        }
    }

    /// Set margin on all sides.
    #[wasm_bindgen(js_name = "marginAll")]
    pub fn margin_all(self, value: u16) -> Self {
        Self {
            inner: self.inner.margin(value),
        }
    }

    /// Set margin with vertical and horizontal values.
    #[wasm_bindgen(js_name = "marginVH")]
    pub fn margin_vh(self, vertical: u16, horizontal: u16) -> Self {
        Self {
            inner: self.inner.margin((vertical, horizontal)),
        }
    }

    /// Set margin for all four sides individually.
    #[wasm_bindgen]
    pub fn margin(self, top: u16, right: u16, bottom: u16, left: u16) -> Self {
        Self {
            inner: self.inner.margin((top, right, bottom, left)),
        }
    }

    /// Set the width of the styled content.
    #[wasm_bindgen]
    pub fn width(self, w: u16) -> Self {
        Self {
            inner: self.inner.width(w),
        }
    }

    /// Set the height of the styled content.
    #[wasm_bindgen]
    pub fn height(self, h: u16) -> Self {
        Self {
            inner: self.inner.height(h),
        }
    }

    /// Set the border style.
    ///
    /// Available styles: "normal", "rounded", "thick", "double", "hidden", "ascii"
    #[wasm_bindgen(js_name = "borderStyle")]
    pub fn border_style(self, style: &str) -> Self {
        let border = match style {
            "normal" => Border::normal(),
            "rounded" => Border::rounded(),
            "thick" => Border::thick(),
            "double" => Border::double(),
            "hidden" => Border::hidden(),
            "ascii" => Border::ascii(),
            _ => Border::normal(),
        };
        Self {
            inner: self.inner.border(border),
        }
    }

    /// Enable border on all sides.
    #[wasm_bindgen(js_name = "borderAll")]
    pub fn border_all(self) -> Self {
        Self {
            inner: self
                .inner
                .border_top(true)
                .border_right(true)
                .border_bottom(true)
                .border_left(true),
        }
    }

    /// Enable border on top.
    #[wasm_bindgen(js_name = "borderTop")]
    pub fn border_top(self) -> Self {
        Self {
            inner: self.inner.border_top(true),
        }
    }

    /// Enable border on bottom.
    #[wasm_bindgen(js_name = "borderBottom")]
    pub fn border_bottom(self) -> Self {
        Self {
            inner: self.inner.border_bottom(true),
        }
    }

    /// Enable border on left.
    #[wasm_bindgen(js_name = "borderLeft")]
    pub fn border_left(self) -> Self {
        Self {
            inner: self.inner.border_left(true),
        }
    }

    /// Enable border on right.
    #[wasm_bindgen(js_name = "borderRight")]
    pub fn border_right(self) -> Self {
        Self {
            inner: self.inner.border_right(true),
        }
    }

    /// Set horizontal alignment.
    ///
    /// Values: 0.0 = left, 0.5 = center, 1.0 = right
    #[wasm_bindgen(js_name = "alignHorizontal")]
    pub fn align_horizontal(self, value: f64) -> Self {
        Self {
            inner: self.inner.align_horizontal(f64_to_position(value)),
        }
    }

    /// Set vertical alignment.
    ///
    /// Values: 0.0 = top, 0.5 = center, 1.0 = bottom
    #[wasm_bindgen(js_name = "alignVertical")]
    pub fn align_vertical(self, value: f64) -> Self {
        Self {
            inner: self.inner.align_vertical(f64_to_position(value)),
        }
    }

    /// Align content to the left.
    #[wasm_bindgen(js_name = "alignLeft")]
    pub fn align_left(self) -> Self {
        Self {
            inner: self.inner.align(Position::Left),
        }
    }

    /// Align content to the center.
    #[wasm_bindgen(js_name = "alignCenter")]
    pub fn align_center(self) -> Self {
        Self {
            inner: self.inner.align(Position::Center),
        }
    }

    /// Align content to the right.
    #[wasm_bindgen(js_name = "alignRight")]
    pub fn align_right(self) -> Self {
        Self {
            inner: self.inner.align(Position::Right),
        }
    }

    /// Render content with this style as HTML.
    ///
    /// Returns an HTML string with inline styles or CSS classes.
    #[wasm_bindgen]
    pub fn render(&self, content: &str) -> String {
        let backend = HtmlBackend::new();
        backend.render(content, &self.inner)
    }

    /// Render content with this style as ANSI escape sequences.
    ///
    /// This is useful for terminal-like displays in web applications.
    #[wasm_bindgen(js_name = "renderAnsi")]
    pub fn render_ansi(&self, content: &str) -> String {
        self.inner.render(content)
    }

    /// Copy this style.
    ///
    /// Creates a new style with the same properties.
    #[wasm_bindgen]
    pub fn copy(&self) -> Self {
        self.clone()
    }
}

impl Default for JsStyle {
    fn default() -> Self {
        Self::new()
    }
}

/// JavaScript-friendly wrapper for Color.
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct JsColor {
    inner: Color,
}

#[wasm_bindgen]
impl JsColor {
    /// Create a color from a hex string (e.g., "#ff00ff" or "ff00ff").
    #[wasm_bindgen(js_name = "fromHex")]
    pub fn from_hex(hex: &str) -> Result<JsColor, JsValue> {
        let color: Color = hex.into();
        Ok(JsColor { inner: color })
    }

    /// Create a color from RGB values.
    #[wasm_bindgen(js_name = "fromRgb")]
    pub fn from_rgb(r: u8, g: u8, b: u8) -> JsColor {
        JsColor {
            inner: Color::new(format!("#{:02x}{:02x}{:02x}", r, g, b)),
        }
    }

    /// Create a color from an ANSI 256-color code.
    #[wasm_bindgen(js_name = "fromAnsi")]
    pub fn from_ansi(code: u8) -> JsColor {
        JsColor {
            inner: Color::new(code.to_string()),
        }
    }

    /// Get the hex representation of this color.
    #[wasm_bindgen(js_name = "toHex")]
    pub fn to_hex(&self) -> String {
        // Try to convert to hex, or return the raw string
        if let Some((r, g, b)) = self.inner.as_rgb() {
            format!("#{:02x}{:02x}{:02x}", r, g, b)
        } else {
            self.inner.0.clone()
        }
    }
}

/// Join multiple strings horizontally.
///
/// The position parameter controls vertical alignment:
/// - 0.0 = top
/// - 0.5 = center
/// - 1.0 = bottom
#[wasm_bindgen(js_name = "joinHorizontal")]
pub fn join_horizontal(position: f64, items: Vec<JsValue>) -> String {
    let strings: Vec<String> = items.iter().filter_map(|v| v.as_string()).collect();
    let refs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
    crate::join_horizontal(f64_to_position(position), &refs)
}

/// Join multiple strings vertically.
///
/// The position parameter controls horizontal alignment:
/// - 0.0 = left
/// - 0.5 = center
/// - 1.0 = right
#[wasm_bindgen(js_name = "joinVertical")]
pub fn join_vertical(position: f64, items: Vec<JsValue>) -> String {
    let strings: Vec<String> = items.iter().filter_map(|v| v.as_string()).collect();
    let refs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
    crate::join_vertical(f64_to_position(position), &refs)
}

/// Place content at a position within a container.
#[wasm_bindgen]
pub fn place(
    width: usize,
    height: usize,
    h_position: f64,
    v_position: f64,
    content: &str,
) -> String {
    crate::place(
        width,
        height,
        f64_to_position(h_position),
        f64_to_position(v_position),
        content,
    )
}

/// Get the visible width of a string (excluding escape codes).
#[wasm_bindgen(js_name = "stringWidth")]
pub fn string_width(s: &str) -> usize {
    crate::width(s)
}

/// Get the height (number of lines) of a string.
#[wasm_bindgen(js_name = "stringHeight")]
pub fn string_height(s: &str) -> usize {
    crate::height(s)
}

/// Available border preset names.
#[wasm_bindgen]
pub fn border_presets() -> Vec<JsValue> {
    vec![
        JsValue::from_str("normal"),
        JsValue::from_str("rounded"),
        JsValue::from_str("thick"),
        JsValue::from_str("double"),
        JsValue::from_str("hidden"),
        JsValue::from_str("ascii"),
    ]
}

// Re-export the OutputBackend trait implementation for HtmlBackend
use crate::backend::OutputBackend;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_style_builder() {
        let style = JsStyle::new()
            .foreground("#ff00ff")
            .bold()
            .padding(1, 2, 1, 2);

        let rendered = style.render("Hello");
        assert!(rendered.contains("Hello"));
    }

    #[test]
    fn test_js_color_from_hex() {
        let color = JsColor::from_hex("#ff0000").unwrap();
        assert_eq!(color.to_hex(), "#ff0000");
    }

    #[test]
    fn test_js_color_from_rgb() {
        let color = JsColor::from_rgb(255, 0, 0);
        assert_eq!(color.to_hex(), "#ff0000");
    }

    #[test]
    fn test_f64_to_position_boundaries() {
        // Test position conversion function
        assert_eq!(f64_to_position(0.0), Position::Left);
        assert_eq!(f64_to_position(0.25), Position::Left);
        assert_eq!(f64_to_position(0.5), Position::Center);
        assert_eq!(f64_to_position(0.75), Position::Right);
        assert_eq!(f64_to_position(1.0), Position::Right);
    }

    #[test]
    fn test_js_style_all_text_formatting() {
        let style = JsStyle::new()
            .bold()
            .italic()
            .underline()
            .strikethrough()
            .faint()
            .reverse();

        let rendered = style.render("Formatted");
        assert!(rendered.contains("Formatted"));
    }

    #[test]
    fn test_js_style_padding_variants() {
        // All sides
        let all = JsStyle::new().padding_all(2);
        assert!(all.render("Test").contains("Test"));

        // Vertical/Horizontal
        let vh = JsStyle::new().padding_vh(1, 2);
        assert!(vh.render("Test").contains("Test"));

        // Individual
        let individual = JsStyle::new().padding(1, 2, 3, 4);
        assert!(individual.render("Test").contains("Test"));
    }

    #[test]
    fn test_js_style_margin_variants() {
        // All sides
        let all = JsStyle::new().margin_all(1);
        assert!(all.render("Test").contains("Test"));

        // Vertical/Horizontal
        let vh = JsStyle::new().margin_vh(1, 2);
        assert!(vh.render("Test").contains("Test"));

        // Individual
        let individual = JsStyle::new().margin(1, 2, 3, 4);
        assert!(individual.render("Test").contains("Test"));
    }

    #[test]
    fn test_js_style_dimensions() {
        let style = JsStyle::new().width(30).height(5);
        let rendered = style.render("Sized");
        assert!(rendered.contains("Sized"));
    }

    #[test]
    fn test_js_style_border_all_sides() {
        let style = JsStyle::new()
            .border_style("rounded")
            .border_all();
        let rendered = style.render("Bordered");
        assert!(rendered.contains("Bordered"));
    }

    #[test]
    fn test_js_style_individual_borders() {
        let style = JsStyle::new()
            .border_style("normal")
            .border_top()
            .border_bottom();
        let rendered = style.render("TB");
        assert!(rendered.contains("TB"));
    }

    #[test]
    fn test_js_style_alignment() {
        let left = JsStyle::new().width(20).align_left();
        assert!(left.render("L").contains("L"));

        let center = JsStyle::new().width(20).align_center();
        assert!(center.render("C").contains("C"));

        let right = JsStyle::new().width(20).align_right();
        assert!(right.render("R").contains("R"));
    }

    #[test]
    fn test_js_style_alignment_numeric() {
        let left = JsStyle::new().width(20).align_horizontal(0.0);
        assert!(left.render("L").contains("L"));

        let center = JsStyle::new().width(20).align_horizontal(0.5);
        assert!(center.render("C").contains("C"));

        let right = JsStyle::new().width(20).align_horizontal(1.0);
        assert!(right.render("R").contains("R"));
    }

    #[test]
    fn test_js_style_render_ansi() {
        let style = JsStyle::new().bold().foreground("#ff0000");
        let ansi = style.render_ansi("ANSI");
        assert!(ansi.contains("ANSI"));
    }

    #[test]
    fn test_js_style_copy() {
        let original = JsStyle::new().bold().foreground("#ff0000");
        let copied = original.copy();

        assert_eq!(original.render("Test"), copied.render("Test"));
    }

    #[test]
    fn test_js_style_default() {
        let default_style = JsStyle::default();
        let new_style = JsStyle::new();
        assert_eq!(default_style.render("X"), new_style.render("X"));
    }

    #[test]
    fn test_js_color_from_ansi() {
        let color = JsColor::from_ansi(196);
        // ANSI 196 is bright red, hex representation depends on mapping
        let hex = color.to_hex();
        assert!(!hex.is_empty());
    }

    #[test]
    fn test_js_color_various_formats() {
        // With hash
        let with_hash = JsColor::from_hex("#abcdef").unwrap();
        assert_eq!(with_hash.to_hex(), "#abcdef");

        // Lowercase
        let lower = JsColor::from_hex("#aabbcc").unwrap();
        assert_eq!(lower.to_hex(), "#aabbcc");
    }

    #[test]
    fn test_border_style_unknown_defaults() {
        // Unknown border style should default to normal
        let style = JsStyle::new()
            .border_style("unknown_style")
            .border_all();
        let rendered = style.render("Test");
        assert!(rendered.contains("Test"));
    }

    #[test]
    fn test_all_border_presets() {
        let presets = ["normal", "rounded", "thick", "double", "hidden", "ascii"];
        for preset in presets {
            let style = JsStyle::new()
                .border_style(preset)
                .border_all();
            let rendered = style.render(&format!("{} border", preset));
            assert!(rendered.contains("border"));
        }
    }
}
