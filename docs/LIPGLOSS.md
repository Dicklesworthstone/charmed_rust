# Lipgloss - Terminal Styling Library

## Essence

Lipgloss provides declarative, composable terminal styling with a CSS-like fluent API.
It's the styling foundation for all Charm terminal applications.

## Core Concepts

### Style Properties (bitflags)

The Go version uses `propKey int64` as bitflags to track which properties are set.
Rust will use the `bitflags` crate for type safety.

**Boolean properties:**
- bold, italic, underline, strikethrough, reverse, blink, faint
- underlineSpaces, strikethroughSpaces, colorWhitespace

**Value properties:**
- foreground, background (colors)
- width, height, maxWidth, maxHeight
- alignHorizontal, alignVertical
- padding (top, right, bottom, left)
- margin (top, right, bottom, left) + marginBackground
- border (style, edges, foreground colors, background colors)
- tabWidth, inline, transform

### Color Types

```rust
pub trait TerminalColor {
    fn to_ansi(&self, profile: ColorProfile) -> AnsiColor;
}

pub struct NoColor;
pub struct Color(String);           // "#0000ff" or "21"
pub struct AnsiColor(u8);           // 0-255
pub struct AdaptiveColor { light: String, dark: String }
pub struct CompleteColor { truecolor: String, ansi256: String, ansi: String }
pub struct CompleteAdaptiveColor { light: CompleteColor, dark: CompleteColor }
```

### Color Profiles

```rust
pub enum ColorProfile {
    Ascii,      // No color (1-bit)
    ANSI,       // 16 colors (4-bit)
    ANSI256,    // 256 colors (8-bit)
    TrueColor,  // 16M colors (24-bit)
}
```

### Border Styles

Pre-defined borders:
- NormalBorder (┌─┐│└┘)
- RoundedBorder (╭─╮│╰╯)
- BlockBorder (█)
- ThickBorder (┏━┓┃┗┛)
- DoubleBorder (╔═╗║╚╝)
- HiddenBorder (spaces)
- ASCIIBorder (+-)

### Position

```rust
pub enum Position {
    Top,
    Bottom,
    Left,
    Right,
    Center,
}
```

## Rust API Design

### Style Builder

```rust
#[derive(Clone, Default)]
pub struct Style {
    props: Props,       // Bitflags for which properties are set
    attrs: Attrs,       // Bitflags for boolean attribute values

    fg_color: Option<Box<dyn TerminalColor>>,
    bg_color: Option<Box<dyn TerminalColor>>,

    width: u16,
    height: u16,
    max_width: u16,
    max_height: u16,

    align_horizontal: Position,
    align_vertical: Position,

    padding: Sides<u16>,
    margin: Sides<u16>,
    margin_bg_color: Option<Box<dyn TerminalColor>>,

    border_style: Border,
    border_edges: Edges,
    border_fg: EdgeColors,
    border_bg: EdgeColors,

    tab_width: i8,  // -1 = no conversion
    transform: Option<fn(String) -> String>,

    renderer: RendererRef,
}

impl Style {
    pub fn new() -> Self;

    // Boolean setters (take self, return Self)
    pub fn bold(self) -> Self;
    pub fn italic(self) -> Self;
    pub fn underline(self) -> Self;
    pub fn strikethrough(self) -> Self;
    pub fn reverse(self) -> Self;
    pub fn blink(self) -> Self;
    pub fn faint(self) -> Self;

    // Color setters
    pub fn foreground(self, color: impl Into<Color>) -> Self;
    pub fn background(self, color: impl Into<Color>) -> Self;

    // Dimension setters
    pub fn width(self, n: u16) -> Self;
    pub fn height(self, n: u16) -> Self;
    pub fn max_width(self, n: u16) -> Self;
    pub fn max_height(self, n: u16) -> Self;

    // Alignment
    pub fn align(self, h: Position) -> Self;
    pub fn align_horizontal(self, p: Position) -> Self;
    pub fn align_vertical(self, p: Position) -> Self;

    // Padding (CSS-like shorthand)
    pub fn padding(self, values: impl Into<Sides<u16>>) -> Self;
    pub fn padding_top(self, n: u16) -> Self;
    pub fn padding_right(self, n: u16) -> Self;
    pub fn padding_bottom(self, n: u16) -> Self;
    pub fn padding_left(self, n: u16) -> Self;

    // Margin (CSS-like shorthand)
    pub fn margin(self, values: impl Into<Sides<u16>>) -> Self;
    pub fn margin_top(self, n: u16) -> Self;
    pub fn margin_right(self, n: u16) -> Self;
    pub fn margin_bottom(self, n: u16) -> Self;
    pub fn margin_left(self, n: u16) -> Self;
    pub fn margin_background(self, color: impl Into<Color>) -> Self;

    // Border
    pub fn border(self, border: Border) -> Self;
    pub fn border_style(self, border: Border) -> Self;
    pub fn border_top(self, v: bool) -> Self;
    pub fn border_right(self, v: bool) -> Self;
    pub fn border_bottom(self, v: bool) -> Self;
    pub fn border_left(self, v: bool) -> Self;
    pub fn border_foreground(self, color: impl Into<Color>) -> Self;
    pub fn border_background(self, color: impl Into<Color>) -> Self;

    // Other
    pub fn inline(self) -> Self;
    pub fn tab_width(self, n: i8) -> Self;
    pub fn transform(self, f: fn(String) -> String) -> Self;

    // Rendering
    pub fn render(&self, text: &str) -> String;
    pub fn render_with(&self, text: impl Display) -> String;
}

// Display trait for easy use
impl Display for Style {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result;
}
```

### Helper Types

```rust
/// CSS-like sides specification
#[derive(Clone, Copy, Default)]
pub struct Sides<T> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

impl<T: Copy> From<T> for Sides<T> {
    fn from(all: T) -> Self { /* all sides */ }
}

impl<T: Copy> From<(T, T)> for Sides<T> {
    fn from((v, h): (T, T)) -> Self { /* vertical, horizontal */ }
}

impl<T: Copy> From<(T, T, T, T)> for Sides<T> {
    fn from((t, r, b, l): (T, T, T, T)) -> Self { /* clockwise */ }
}
```

## Implementation Strategy

### Phase 1: Core Types
1. `color.rs` - Color types and TerminalColor trait
2. `border.rs` - Border struct and preset borders
3. `position.rs` - Position enum

### Phase 2: Style Foundation
4. `props.rs` - Property bitflags
5. `style.rs` - Style struct with setters

### Phase 3: Rendering
6. `renderer.rs` - Renderer with color profile
7. `render.rs` - Style.render() implementation

### Phase 4: Layout
8. `align.rs` - Text alignment
9. `whitespace.rs` - Padding/margin application
10. `wrap.rs` - Text wrapping

## Dependencies

```toml
[dependencies]
crossterm = "0.27"      # Terminal capabilities
unicode-width = "0.2"   # Character width calculation
bitflags = "2"          # Property flags
```

## Testing Strategy

1. Unit tests for color conversion
2. Unit tests for style builder (property tracking)
3. Integration tests for rendering (compare output)
4. Visual tests for border rendering
