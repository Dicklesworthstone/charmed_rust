# Existing Charm Structure and Architecture

> **THE SPEC:** This document is the authoritative reference for implementation.
> After reading this spec, you should NOT need to consult legacy code.

---

## Table of Contents

1. [Harmonica](#1-harmonica---physics-based-animations)
2. [Lipgloss](#2-lipgloss---terminal-styling)
3. [Bubbletea](#3-bubbletea---tui-framework)
4. [Charmed Log](#4-charmed-log---logging)
5. [Glamour](#5-glamour---markdown-rendering)
6. [Bubbles](#6-bubbles---tui-components)
7. [Huh](#7-huh---forms-and-prompts)
8. [Wish](#8-wish---ssh-apps)
9. [Glow](#9-glow---markdown-cli)
10. [Cross-Library Integration Patterns](#10-cross-library-integration-patterns)

---

## 1. Harmonica — Physics-Based Animations

**Purpose:** Physics-based animation tools for smooth, realistic motion in 2D/3D applications.

**Source:** `legacy_harmonica/` (~200 lines of Go)

### 1.1 Data Structures

#### Spring

A damped harmonic oscillator with precomputed coefficients for efficient frame-by-frame updates.

```rust
/// Precomputed spring coefficients for damped harmonic oscillator.
/// These coefficients are computed once during initialization and reused
/// for all position/velocity updates.
pub struct Spring {
    /// Position-to-position coefficient
    pos_pos_coef: f64,
    /// Velocity-to-position coefficient
    pos_vel_coef: f64,
    /// Position-to-velocity coefficient
    vel_pos_coef: f64,
    /// Velocity-to-velocity coefficient
    vel_vel_coef: f64,
}
```

#### Point

A 3D point representing position in space.

```rust
/// A point in 3D space.
#[derive(Debug, Clone, Copy, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
```

#### Vector

A 3D vector representing magnitude and direction from origin.

```rust
/// A 3D vector for velocity and acceleration.
#[derive(Debug, Clone, Copy, Default)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
```

#### Projectile

Mutable projectile state for kinematics simulation.

```rust
/// A projectile with position, velocity, and acceleration.
pub struct Projectile {
    pos: Point,
    vel: Vector,
    acc: Vector,
    delta_time: f64,
}
```

### 1.2 Constants

```rust
/// Standard gravity vector (origin at bottom-left, Y points up)
pub const GRAVITY: Vector = Vector { x: 0.0, y: -9.81, z: 0.0 };

/// Terminal gravity vector (origin at top-left, Y points down)
pub const TERMINAL_GRAVITY: Vector = Vector { x: 0.0, y: 9.81, z: 0.0 };

/// Machine epsilon for floating point comparisons
const EPSILON: f64 = f64::EPSILON;  // Or calculate as: 1.0_f64.next_after(2.0) - 1.0
```

### 1.3 Functions

#### fps()

Converts frames-per-second to time delta in seconds.

```rust
/// Convert FPS to time delta.
///
/// # Example
/// ```
/// let delta = fps(60);  // Returns 1/60 ≈ 0.01667
/// ```
pub fn fps(n: u32) -> f64 {
    1.0 / n as f64
}
```

**Go original:**
```go
func FPS(n int) float64 {
    return (time.Second / time.Duration(n)).Seconds()
}
```

#### Spring::new()

Creates a new spring with precomputed motion parameters.

**Parameters:**
- `delta_time: f64` — Frame time step (use `fps()` helper)
- `angular_frequency: f64` — Motion speed (typical: 5.0-10.0)
- `damping_ratio: f64` — Oscillation control

**Damping ratios:**
| Value | Type | Behavior |
|-------|------|----------|
| ζ > 1 | Over-damped | No oscillation, slow return |
| ζ = 1 | Critically-damped | Fastest return, no oscillation |
| ζ < 1 | Under-damped | Oscillates with decay |

**Algorithm (Ryan Juckett's Damped Harmonic Motion):**

```rust
impl Spring {
    pub fn new(delta_time: f64, angular_frequency: f64, damping_ratio: f64) -> Self {
        // Clamp to legal range
        let omega = angular_frequency.max(0.0);  // ω
        let zeta = damping_ratio.max(0.0);       // ζ

        // If no angular frequency, spring doesn't move (identity)
        if omega < EPSILON {
            return Spring {
                pos_pos_coef: 1.0,
                pos_vel_coef: 0.0,
                vel_pos_coef: 0.0,
                vel_vel_coef: 1.0,
            };
        }

        if zeta > 1.0 + EPSILON {
            // OVER-DAMPED: ζ > 1
            let za = -omega * zeta;
            let zb = omega * (zeta * zeta - 1.0).sqrt();
            let z1 = za - zb;
            let z2 = za + zb;

            let e1 = (z1 * delta_time).exp();
            let e2 = (z2 * delta_time).exp();

            let inv_two_zb = 1.0 / (2.0 * zb);  // = 1 / (z2 - z1)

            let e1_over_two_zb = e1 * inv_two_zb;
            let e2_over_two_zb = e2 * inv_two_zb;

            let z1e1_over_two_zb = z1 * e1_over_two_zb;
            let z2e2_over_two_zb = z2 * e2_over_two_zb;

            Spring {
                pos_pos_coef: e1_over_two_zb * z2 - z2e2_over_two_zb + e2,
                pos_vel_coef: -e1_over_two_zb + e2_over_two_zb,
                vel_pos_coef: (z1e1_over_two_zb - z2e2_over_two_zb + e2) * z2,
                vel_vel_coef: -z1e1_over_two_zb + z2e2_over_two_zb,
            }
        } else if zeta < 1.0 - EPSILON {
            // UNDER-DAMPED: ζ < 1
            let omega_zeta = omega * zeta;
            let alpha = omega * (1.0 - zeta * zeta).sqrt();

            let exp_term = (-omega_zeta * delta_time).exp();
            let cos_term = (alpha * delta_time).cos();
            let sin_term = (alpha * delta_time).sin();

            let inv_alpha = 1.0 / alpha;

            let exp_sin = exp_term * sin_term;
            let exp_cos = exp_term * cos_term;
            let exp_omega_zeta_sin_over_alpha = exp_term * omega_zeta * sin_term * inv_alpha;

            Spring {
                pos_pos_coef: exp_cos + exp_omega_zeta_sin_over_alpha,
                pos_vel_coef: exp_sin * inv_alpha,
                vel_pos_coef: -exp_sin * alpha - omega_zeta * exp_omega_zeta_sin_over_alpha,
                vel_vel_coef: exp_cos - exp_omega_zeta_sin_over_alpha,
            }
        } else {
            // CRITICALLY-DAMPED: ζ ≈ 1
            let exp_term = (-omega * delta_time).exp();
            let time_exp = delta_time * exp_term;
            let time_exp_freq = time_exp * omega;

            Spring {
                pos_pos_coef: time_exp_freq + exp_term,
                pos_vel_coef: time_exp,
                vel_pos_coef: -omega * time_exp_freq,
                vel_vel_coef: -time_exp_freq + exp_term,
            }
        }
    }
}
```

#### Spring::update()

Updates position and velocity towards an equilibrium (target) position.

```rust
impl Spring {
    /// Update position and velocity towards equilibrium.
    /// Returns (new_position, new_velocity).
    pub fn update(&self, pos: f64, vel: f64, equilibrium_pos: f64) -> (f64, f64) {
        // Transform to equilibrium-relative space
        let old_pos = pos - equilibrium_pos;
        let old_vel = vel;

        // Apply precomputed coefficients
        let new_pos = old_pos * self.pos_pos_coef + old_vel * self.pos_vel_coef + equilibrium_pos;
        let new_vel = old_pos * self.vel_pos_coef + old_vel * self.vel_vel_coef;

        (new_pos, new_vel)
    }
}
```

#### Projectile::new()

Creates a new projectile with initial conditions.

```rust
impl Projectile {
    pub fn new(
        delta_time: f64,
        initial_position: Point,
        initial_velocity: Vector,
        acceleration: Vector,
    ) -> Self {
        Projectile {
            pos: initial_position,
            vel: initial_velocity,
            acc: acceleration,
            delta_time,
        }
    }
}
```

#### Projectile::update()

Applies Euler integration to update position and velocity.

```rust
impl Projectile {
    /// Update projectile state by one time step.
    /// Returns the new position.
    pub fn update(&mut self) -> Point {
        // Update position: p' = p + v * dt
        self.pos.x += self.vel.x * self.delta_time;
        self.pos.y += self.vel.y * self.delta_time;
        self.pos.z += self.vel.z * self.delta_time;

        // Update velocity: v' = v + a * dt
        self.vel.x += self.acc.x * self.delta_time;
        self.vel.y += self.acc.y * self.delta_time;
        self.vel.z += self.acc.z * self.delta_time;

        self.pos
    }

    pub fn position(&self) -> Point { self.pos }
    pub fn velocity(&self) -> Vector { self.vel }
    pub fn acceleration(&self) -> Vector { self.acc }
}
```

### 1.4 Usage Examples

```rust
use harmonica::{fps, Spring, Projectile, Point, Vector, TERMINAL_GRAVITY};

// Spring animation
let spring = Spring::new(fps(60), 6.0, 0.2);
let mut pos = 0.0;
let mut vel = 0.0;
let target = 100.0;

for _ in 0..120 {  // 2 seconds at 60 FPS
    (pos, vel) = spring.update(pos, vel, target);
}
assert!((pos - target).abs() < 5.0);

// Projectile motion
let mut projectile = Projectile::new(
    fps(60),
    Point { x: 0.0, y: 0.0, z: 0.0 },
    Vector { x: 10.0, y: -5.0, z: 0.0 },
    TERMINAL_GRAVITY,
);

for _ in 0..60 {
    let pos = projectile.update();
    println!("Position: ({}, {}, {})", pos.x, pos.y, pos.z);
}
```

### 1.5 Attribution

The spring algorithm is based on Ryan Juckett's damped harmonic motion:
https://www.ryanjuckett.com/damped-springs/

---

## 2. Lipgloss — Terminal Styling

**Purpose:** Declarative terminal styling with colors, borders, padding, margins, and alignment.

**Source:** `legacy_lipgloss/` (~1,500 lines of Go)

### 2.1 Property System

Lipgloss uses a bitfield to track which style properties have been explicitly set.
This enables efficient inheritance and zero-value detection.

#### Property Keys (43 total)

```rust
use bitflags::bitflags;

bitflags! {
    /// Property flags indicating which style properties are set.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct Props: u64 {
        // Boolean properties (first 10 bits)
        const BOLD                    = 1 << 0;
        const ITALIC                  = 1 << 1;
        const UNDERLINE               = 1 << 2;
        const STRIKETHROUGH           = 1 << 3;
        const REVERSE                 = 1 << 4;
        const BLINK                   = 1 << 5;
        const FAINT                   = 1 << 6;
        const UNDERLINE_SPACES        = 1 << 7;
        const STRIKETHROUGH_SPACES    = 1 << 8;
        const COLOR_WHITESPACE        = 1 << 9;

        // Value properties
        const FOREGROUND              = 1 << 10;
        const BACKGROUND              = 1 << 11;
        const WIDTH                   = 1 << 12;
        const HEIGHT                  = 1 << 13;
        const ALIGN_HORIZONTAL        = 1 << 14;
        const ALIGN_VERTICAL          = 1 << 15;

        // Padding
        const PADDING_TOP             = 1 << 16;
        const PADDING_RIGHT           = 1 << 17;
        const PADDING_BOTTOM          = 1 << 18;
        const PADDING_LEFT            = 1 << 19;

        // Margins
        const MARGIN_TOP              = 1 << 20;
        const MARGIN_RIGHT            = 1 << 21;
        const MARGIN_BOTTOM           = 1 << 22;
        const MARGIN_LEFT             = 1 << 23;
        const MARGIN_BACKGROUND       = 1 << 24;

        // Border style
        const BORDER_STYLE            = 1 << 25;

        // Border edges
        const BORDER_TOP              = 1 << 26;
        const BORDER_RIGHT            = 1 << 27;
        const BORDER_BOTTOM           = 1 << 28;
        const BORDER_LEFT             = 1 << 29;

        // Border foreground colors
        const BORDER_TOP_FOREGROUND   = 1 << 30;
        const BORDER_RIGHT_FOREGROUND = 1 << 31;
        const BORDER_BOTTOM_FOREGROUND= 1 << 32;
        const BORDER_LEFT_FOREGROUND  = 1 << 33;

        // Border background colors
        const BORDER_TOP_BACKGROUND   = 1 << 34;
        const BORDER_RIGHT_BACKGROUND = 1 << 35;
        const BORDER_BOTTOM_BACKGROUND= 1 << 36;
        const BORDER_LEFT_BACKGROUND  = 1 << 37;

        // Additional properties
        const INLINE                  = 1 << 38;
        const MAX_WIDTH               = 1 << 39;
        const MAX_HEIGHT              = 1 << 40;
        const TAB_WIDTH               = 1 << 41;
        const TRANSFORM               = 1 << 42;
    }
}
```

### 2.2 Data Structures

#### Color Types

```rust
/// A terminal color.
pub trait TerminalColor: Clone {
    /// Convert to crossterm color based on color profile.
    fn color(&self, renderer: &Renderer) -> crossterm::style::Color;
}

/// No color (terminal default).
#[derive(Debug, Clone, Copy, Default)]
pub struct NoColor;

/// Color specified by hex string or ANSI number string.
/// Examples: "#ff0000", "21", "red"
#[derive(Debug, Clone)]
pub struct Color(pub String);

/// ANSI color by number (0-255).
#[derive(Debug, Clone, Copy)]
pub struct ANSIColor(pub u8);

/// Adaptive color that changes based on background darkness.
#[derive(Debug, Clone)]
pub struct AdaptiveColor {
    pub light: String,
    pub dark: String,
}

/// Complete color with explicit values for each profile.
#[derive(Debug, Clone)]
pub struct CompleteColor {
    pub true_color: String,
    pub ansi256: String,
    pub ansi: String,
}

/// Complete adaptive color for both backgrounds and all profiles.
#[derive(Debug, Clone)]
pub struct CompleteAdaptiveColor {
    pub light: CompleteColor,
    pub dark: CompleteColor,
}
```

#### Border

```rust
/// Border characters for all edges and corners.
#[derive(Debug, Clone, Default)]
pub struct Border {
    pub top: String,
    pub bottom: String,
    pub left: String,
    pub right: String,
    pub top_left: String,
    pub top_right: String,
    pub bottom_left: String,
    pub bottom_right: String,
    pub middle_left: String,
    pub middle_right: String,
    pub middle: String,
    pub middle_top: String,
    pub middle_bottom: String,
}

impl Border {
    /// Get the width contribution of the top border.
    pub fn get_top_size(&self) -> usize { ... }
    /// Get the width contribution of the right border.
    pub fn get_right_size(&self) -> usize { ... }
    /// Get the width contribution of the bottom border.
    pub fn get_bottom_size(&self) -> usize { ... }
    /// Get the width contribution of the left border.
    pub fn get_left_size(&self) -> usize { ... }
}
```

#### Built-in Border Styles

| Function | Characters | Description |
|----------|------------|-------------|
| `normal_border()` | `┌─┐│└─┘` | Standard box-drawing |
| `rounded_border()` | `╭─╮│╰─╯` | Rounded corners |
| `block_border()` | `████████` | Solid block |
| `outer_half_block_border()` | `▀▄▌▐▛▜▙▟` | Half blocks outside |
| `inner_half_block_border()` | `▄▀▐▌▗▖▝▘` | Half blocks inside |
| `thick_border()` | `┏━┓┃┗━┛` | Heavy weight |
| `double_border()` | `╔═╗║╚═╝` | Double lines |
| `hidden_border()` | Spaces | Invisible but takes space |
| `markdown_border()` | `\|---\|` | Markdown table style |
| `ascii_border()` | `+---+\|` | ASCII-only |

**Exact border characters:**

```rust
pub fn normal_border() -> Border {
    Border {
        top: "─".into(), bottom: "─".into(),
        left: "│".into(), right: "│".into(),
        top_left: "┌".into(), top_right: "┐".into(),
        bottom_left: "└".into(), bottom_right: "┘".into(),
        middle_left: "├".into(), middle_right: "┤".into(),
        middle: "┼".into(), middle_top: "┬".into(), middle_bottom: "┴".into(),
    }
}

pub fn rounded_border() -> Border {
    Border {
        top: "─".into(), bottom: "─".into(),
        left: "│".into(), right: "│".into(),
        top_left: "╭".into(), top_right: "╮".into(),
        bottom_left: "╰".into(), bottom_right: "╯".into(),
        middle_left: "├".into(), middle_right: "┤".into(),
        middle: "┼".into(), middle_top: "┬".into(), middle_bottom: "┴".into(),
    }
}

pub fn thick_border() -> Border {
    Border {
        top: "━".into(), bottom: "━".into(),
        left: "┃".into(), right: "┃".into(),
        top_left: "┏".into(), top_right: "┓".into(),
        bottom_left: "┗".into(), bottom_right: "┛".into(),
        middle_left: "┣".into(), middle_right: "┫".into(),
        middle: "╋".into(), middle_top: "┳".into(), middle_bottom: "┻".into(),
    }
}

pub fn double_border() -> Border {
    Border {
        top: "═".into(), bottom: "═".into(),
        left: "║".into(), right: "║".into(),
        top_left: "╔".into(), top_right: "╗".into(),
        bottom_left: "╚".into(), bottom_right: "╝".into(),
        middle_left: "╠".into(), middle_right: "╣".into(),
        middle: "╬".into(), middle_top: "╦".into(), middle_bottom: "╩".into(),
    }
}
```

#### Position

```rust
/// Position along an axis (0.0 = start, 1.0 = end, 0.5 = center).
#[derive(Debug, Clone, Copy, Default)]
pub struct Position(pub f64);

impl Position {
    pub fn value(&self) -> f64 {
        self.0.clamp(0.0, 1.0)
    }
}

// Position constants
pub const TOP: Position = Position(0.0);
pub const BOTTOM: Position = Position(1.0);
pub const CENTER: Position = Position(0.5);
pub const LEFT: Position = Position(0.0);
pub const RIGHT: Position = Position(1.0);
```

#### Style

The main style struct with all properties.

```rust
/// A terminal style definition.
#[derive(Clone)]
pub struct Style {
    renderer: Option<Arc<Renderer>>,
    props: Props,
    value: String,

    // Boolean attributes (stored as bitmask for efficiency)
    attrs: u16,

    // Colors
    fg_color: Option<Box<dyn TerminalColor>>,
    bg_color: Option<Box<dyn TerminalColor>>,

    // Dimensions
    width: i32,
    height: i32,

    // Alignment
    align_horizontal: Position,
    align_vertical: Position,

    // Padding
    padding_top: i32,
    padding_right: i32,
    padding_bottom: i32,
    padding_left: i32,

    // Margins
    margin_top: i32,
    margin_right: i32,
    margin_bottom: i32,
    margin_left: i32,
    margin_bg_color: Option<Box<dyn TerminalColor>>,

    // Border
    border_style: Border,
    border_top_fg_color: Option<Box<dyn TerminalColor>>,
    border_right_fg_color: Option<Box<dyn TerminalColor>>,
    border_bottom_fg_color: Option<Box<dyn TerminalColor>>,
    border_left_fg_color: Option<Box<dyn TerminalColor>>,
    border_top_bg_color: Option<Box<dyn TerminalColor>>,
    border_right_bg_color: Option<Box<dyn TerminalColor>>,
    border_bottom_bg_color: Option<Box<dyn TerminalColor>>,
    border_left_bg_color: Option<Box<dyn TerminalColor>>,

    // Constraints
    max_width: i32,
    max_height: i32,
    tab_width: i32,  // Default: 4, -1 = preserve tabs, 0 = remove tabs

    // Transform
    transform: Option<Box<dyn Fn(&str) -> String>>,
}
```

#### Renderer

Thread-safe renderer with color profile detection.

```rust
use std::sync::{Arc, RwLock, Once};

/// Color profile for the terminal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorProfile {
    Ascii,      // No color (1-bit)
    Ansi,       // 16 colors (4-bit)
    Ansi256,    // 256 colors (8-bit)
    TrueColor,  // 16M colors (24-bit)
}

/// Terminal renderer with cached color profile.
pub struct Renderer {
    output: RwLock<Box<dyn std::io::Write + Send>>,
    color_profile: RwLock<Option<ColorProfile>>,
    has_dark_background: RwLock<Option<bool>>,

    // sync.Once equivalents
    color_profile_init: Once,
    background_init: Once,

    explicit_color_profile: RwLock<bool>,
    explicit_background: RwLock<bool>,
}

impl Renderer {
    pub fn new(output: impl std::io::Write + Send + 'static) -> Self { ... }

    /// Get color profile (auto-detects on first call).
    pub fn color_profile(&self) -> ColorProfile { ... }

    /// Set color profile explicitly.
    pub fn set_color_profile(&self, profile: ColorProfile) { ... }

    /// Check if terminal has dark background (auto-detects on first call).
    pub fn has_dark_background(&self) -> bool { ... }

    /// Set dark background explicitly.
    pub fn set_has_dark_background(&self, dark: bool) { ... }

    /// Create a new style using this renderer.
    pub fn new_style(&self) -> Style { ... }
}
```

### 2.3 Style Builder Methods

All setters return `Self` for chaining. Each setter marks the corresponding property as set.

```rust
impl Style {
    pub fn new() -> Self { ... }

    // Text formatting
    pub fn bold(self, v: bool) -> Self { ... }
    pub fn italic(self, v: bool) -> Self { ... }
    pub fn underline(self, v: bool) -> Self { ... }
    pub fn strikethrough(self, v: bool) -> Self { ... }
    pub fn reverse(self, v: bool) -> Self { ... }
    pub fn blink(self, v: bool) -> Self { ... }
    pub fn faint(self, v: bool) -> Self { ... }

    // Colors
    pub fn foreground(self, c: impl TerminalColor) -> Self { ... }
    pub fn background(self, c: impl TerminalColor) -> Self { ... }

    // Dimensions
    pub fn width(self, w: i32) -> Self { ... }
    pub fn height(self, h: i32) -> Self { ... }
    pub fn max_width(self, w: i32) -> Self { ... }
    pub fn max_height(self, h: i32) -> Self { ... }

    // Alignment
    pub fn align(self, h: Position, v: Position) -> Self { ... }
    pub fn align_horizontal(self, p: Position) -> Self { ... }
    pub fn align_vertical(self, p: Position) -> Self { ... }

    // Padding (shorthand sets all four)
    pub fn padding(self, top: i32, right: i32, bottom: i32, left: i32) -> Self { ... }
    pub fn padding_top(self, v: i32) -> Self { ... }
    pub fn padding_right(self, v: i32) -> Self { ... }
    pub fn padding_bottom(self, v: i32) -> Self { ... }
    pub fn padding_left(self, v: i32) -> Self { ... }

    // Margins
    pub fn margin(self, top: i32, right: i32, bottom: i32, left: i32) -> Self { ... }
    pub fn margin_top(self, v: i32) -> Self { ... }
    pub fn margin_right(self, v: i32) -> Self { ... }
    pub fn margin_bottom(self, v: i32) -> Self { ... }
    pub fn margin_left(self, v: i32) -> Self { ... }
    pub fn margin_background(self, c: impl TerminalColor) -> Self { ... }

    // Borders
    pub fn border(self, b: Border) -> Self { ... }
    pub fn border_top(self, v: bool) -> Self { ... }
    pub fn border_right(self, v: bool) -> Self { ... }
    pub fn border_bottom(self, v: bool) -> Self { ... }
    pub fn border_left(self, v: bool) -> Self { ... }
    pub fn border_foreground(self, c: impl TerminalColor) -> Self { ... }
    pub fn border_background(self, c: impl TerminalColor) -> Self { ... }
    // ... per-edge color setters ...

    // Behavior
    pub fn inline(self, v: bool) -> Self { ... }
    pub fn tab_width(self, w: i32) -> Self { ... }
    pub fn underline_spaces(self, v: bool) -> Self { ... }
    pub fn strikethrough_spaces(self, v: bool) -> Self { ... }
    pub fn color_whitespace(self, v: bool) -> Self { ... }
    pub fn transform(self, f: impl Fn(&str) -> String + 'static) -> Self { ... }

    // Value management
    pub fn set_string(self, s: impl Into<String>) -> Self { ... }
    pub fn value(&self) -> &str { ... }

    // Inheritance
    pub fn inherit(self, other: Style) -> Self { ... }
}
```

### 2.4 Render Algorithm

The `render()` method applies all style properties to produce ANSI-styled text.

**Algorithm (pseudocode):**

```
fn render(&self, strs: &[&str]) -> String {
    1. Prepend self.value if set
    2. Join strings with space
    3. Apply transform if set
    4. If props == 0, just convert tabs and return

    5. Build ANSI style sequences:
       - Apply bold, italic, underline, strikethrough, reverse, blink, faint
       - Apply foreground color
       - Apply background color

    6. Convert tabs according to tab_width:
       - -1: preserve tabs
       - 0: remove tabs
       - N: replace with N spaces (default: 4)

    7. Replace \r\n with \n

    8. If inline mode: remove all newlines

    9. Word wrap if width > 0 and not inline:
       - Wrap at (width - left_padding - right_padding)
       - Use unicode cell width

    10. Render core text:
        - Split by newlines
        - If using space styler: apply different style to spaces vs text
        - Otherwise: apply style to entire line

    11. Apply padding (if not inline):
        - Left: prepend spaces to each line
        - Right: append spaces to each line
        - Top: prepend empty lines
        - Bottom: append empty lines

    12. Apply vertical alignment if height > 0

    13. Apply horizontal alignment:
        - Pad lines to same width
        - Apply alignment position

    14. Apply border (if not inline):
        - Render top edge with corners
        - Render left/right sides for each line
        - Render bottom edge with corners
        - Style border with per-edge colors

    15. Apply margins:
        - Left/right: pad lines with margin background
        - Top/bottom: add empty lines with background

    16. Truncate to max_width if set:
        - Truncate each line preserving ANSI sequences

    17. Truncate to max_height if set:
        - Keep only first N lines

    return result
}
```

### 2.5 Utility Functions

#### getLines()

Splits string into lines and calculates max width.

```rust
/// Split string into lines and return (lines, max_width).
fn get_lines(s: &str) -> (Vec<&str>, usize) {
    let lines: Vec<&str> = s.split('\n').collect();
    let max_width = lines.iter()
        .map(|l| unicode_width::UnicodeWidthStr::width(*l))
        .max()
        .unwrap_or(0);
    (lines, max_width)
}
```

#### JoinHorizontal()

Horizontally joins multi-line strings along a vertical axis.

```rust
/// Join strings horizontally with alignment.
pub fn join_horizontal(pos: Position, strs: &[&str]) -> String {
    if strs.is_empty() { return String::new(); }
    if strs.len() == 1 { return strs[0].to_string(); }

    // 1. Split each string into lines, track max width per block
    // 2. Find tallest block
    // 3. Pad shorter blocks with empty lines based on position
    // 4. Merge lines horizontally, padding to max width
    ...
}
```

#### JoinVertical()

Vertically joins multi-line strings along a horizontal axis.

```rust
/// Join strings vertically with alignment.
pub fn join_vertical(pos: Position, strs: &[&str]) -> String {
    if strs.is_empty() { return String::new(); }
    if strs.len() == 1 { return strs[0].to_string(); }

    // 1. Split each string into lines, find max width
    // 2. Pad each line to max width based on position
    // 3. Concatenate all lines
    ...
}
```

#### Place()

Places a string in a box of given dimensions.

```rust
/// Place string in a box with given dimensions and alignment.
pub fn place(width: i32, height: i32, h_pos: Position, v_pos: Position, s: &str) -> String {
    place_vertical(height, v_pos,
        &place_horizontal(width, h_pos, s))
}

pub fn place_horizontal(width: i32, pos: Position, s: &str) -> String { ... }
pub fn place_vertical(height: i32, pos: Position, s: &str) -> String { ... }
```

### 2.6 Usage Examples

```rust
use lipgloss::{Style, Color, Position, normal_border, CENTER};

// Basic styling
let style = Style::new()
    .foreground(Color("#ff0000".into()))
    .background(Color("21".into()))
    .bold(true)
    .padding(1, 2, 1, 2);

println!("{}", style.render("Hello, World!"));

// With border
let boxed = Style::new()
    .border(normal_border())
    .border_foreground(Color("#00ff00".into()))
    .width(40)
    .align_horizontal(CENTER)
    .render("Centered text in a box");

// Joining blocks
let left = "Line 1\nLine 2\nLine 3";
let right = "A\nB";
let combined = lipgloss::join_horizontal(Position(0.5), &[left, right]);
```

---

## 3. Bubbletea — TUI Framework

**Purpose:** Elm-architecture TUI framework for building interactive terminal applications.

**Source:** `legacy_bubbletea/` (~2,500 lines of Go)

### 3.1 Core Elm Architecture

#### Model Trait

```rust
/// The Model contains the program's state and core functions.
pub trait Model: Sized + Send + 'static {
    /// Init is called first and returns an optional initial command.
    fn init(&self) -> Option<Cmd>;

    /// Update is called when a message is received.
    fn update(&mut self, msg: Msg) -> Option<Cmd>;

    /// View renders the program's UI as a string.
    fn view(&self) -> String;
}
```

#### Message and Command Types

```rust
/// Msg contains data from the result of an IO operation.
pub trait Msg: Send + 'static {}

/// Cmd is an IO operation that returns a message when complete.
pub type Cmd = Box<dyn FnOnce() -> Box<dyn Msg> + Send>;

/// Batch executes commands concurrently with no ordering guarantees.
pub fn batch(cmds: Vec<Cmd>) -> Cmd { ... }

/// Sequence runs commands one at a time in order.
pub fn sequence(cmds: Vec<Cmd>) -> Cmd { ... }
```

### 3.2 Program Structure

```rust
pub struct Program<M: Model> {
    initial_model: M,
    startup_options: StartupOptions,
    startup_title: Option<String>,
    input: Box<dyn io::Read + Send>,
    output: Box<dyn io::Write + Send>,
    msgs: mpsc::UnboundedSender<Box<dyn Msg>>,
    renderer: Box<dyn Renderer>,
    ctx: CancellationToken,
    filter: Option<Box<dyn Fn(&M, Box<dyn Msg>) -> Option<Box<dyn Msg>>>>,
    fps: u32,  // 1-120, default 60
}

bitflags! {
    pub struct StartupOptions: u16 {
        const WITH_ALT_SCREEN = 1 << 0;
        const WITH_MOUSE_CELL_MOTION = 1 << 1;
        const WITH_MOUSE_ALL_MOTION = 1 << 2;
        const WITH_ANSI_COMPRESSOR = 1 << 3;
        const WITHOUT_SIGNAL_HANDLER = 1 << 4;
        const WITHOUT_CATCH_PANICS = 1 << 5;
        const WITHOUT_BRACKETED_PASTE = 1 << 6;
        const WITH_REPORT_FOCUS = 1 << 7;
    }
}
```

### 3.3 Key Input Handling

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    // Control characters
    Null, CtrlA, CtrlB, CtrlC, CtrlD, CtrlE, CtrlF, CtrlG,
    CtrlH, Tab, CtrlJ, CtrlK, CtrlL, Enter, CtrlN, CtrlO,
    CtrlP, CtrlQ, CtrlR, CtrlS, CtrlT, CtrlU, CtrlV, CtrlW,
    CtrlX, CtrlY, CtrlZ, Escape, Backspace,

    // Navigation keys
    Up, Down, Left, Right, Home, End, PageUp, PageDown,
    Delete, Insert, Space,

    // Modifier combinations
    ShiftTab, CtrlUp, CtrlDown, CtrlLeft, CtrlRight,
    ShiftUp, ShiftDown, ShiftLeft, ShiftRight,

    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,

    // Regular characters
    Runes,
}

pub struct KeyMsg {
    pub key_type: KeyType,
    pub runes: Vec<char>,
    pub alt: bool,
    pub paste: bool,
}
```

### 3.4 Mouse Events

```rust
pub struct MouseMsg {
    pub x: i32,
    pub y: i32,
    pub button: MouseButton,
    pub action: MouseAction,
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
}

pub enum MouseButton {
    None, Left, Middle, Right,
    WheelUp, WheelDown, WheelLeft, WheelRight,
    Backward, Forward,
}

pub enum MouseAction {
    Press, Release, Motion,
}
```

### 3.5 Control Messages

```rust
pub struct QuitMsg;           // Graceful quit
pub struct SuspendMsg;        // Suspend program
pub struct ResumeMsg;         // Resume from suspend
pub struct WindowSizeMsg {
    pub width: u32,
    pub height: u32,
}
pub struct FocusMsg;          // Terminal gained focus
pub struct BlurMsg;           // Terminal lost focus

// Commands
pub fn quit() -> Cmd { ... }
pub fn clear_screen() -> Cmd { ... }
pub fn enter_alt_screen() -> Cmd { ... }
pub fn exit_alt_screen() -> Cmd { ... }
pub fn show_cursor() -> Cmd { ... }
pub fn hide_cursor() -> Cmd { ... }
pub fn set_window_title(title: &str) -> Cmd { ... }
```

### 3.6 Program Options

```rust
pub type ProgramOption<M> = Box<dyn FnOnce(&mut Program<M>)>;

pub fn with_alt_screen<M>() -> ProgramOption<M> { ... }
pub fn with_mouse_cell_motion<M>() -> ProgramOption<M> { ... }
pub fn with_mouse_all_motion<M>() -> ProgramOption<M> { ... }
pub fn with_input<M>(r: impl io::Read + Send + 'static) -> ProgramOption<M> { ... }
pub fn with_output<M>(w: impl io::Write + Send + 'static) -> ProgramOption<M> { ... }
pub fn with_fps<M>(fps: u32) -> ProgramOption<M> { ... }
pub fn with_filter<M>(f: impl Fn(&M, Box<dyn Msg>) -> Option<Box<dyn Msg>>) -> ProgramOption<M> { ... }
pub fn without_signal_handler<M>() -> ProgramOption<M> { ... }
pub fn without_catch_panics<M>() -> ProgramOption<M> { ... }
pub fn with_report_focus<M>() -> ProgramOption<M> { ... }
```

### 3.7 Timing Commands

```rust
/// Tick produces a command at a fixed interval.
pub fn tick(duration: Duration, f: impl Fn(SystemTime) -> Box<dyn Msg>) -> Cmd { ... }

/// Every ticks in sync with the system clock.
pub fn every(duration: Duration, f: impl Fn(SystemTime) -> Box<dyn Msg>) -> Cmd { ... }
```

---

## 4. Charmed Log — Logging

**Purpose:** Structured, colorful logging with Lipgloss styling.

**Source:** `legacy_log/` (~800 lines of Go)

### 4.1 Logger Structure

```rust
pub struct Logger {
    writer: Arc<Mutex<Box<dyn io::Write + Send>>>,
    level: AtomicI64,
    is_discard: AtomicBool,
    state: RwLock<LoggerState>,
}

pub struct LoggerState {
    prefix: String,
    time_func: Option<fn(SystemTime) -> SystemTime>,
    time_format: String,           // Default: "%Y/%m/%d %H:%M:%S"
    caller_offset: usize,
    caller_formatter: Option<fn(&str, usize, &str) -> String>,
    formatter: Formatter,
    report_caller: bool,
    report_timestamp: bool,
    fields: Vec<(String, String)>,
    helpers: HashSet<String>,
    styles: Styles,
}
```

### 4.2 Log Levels

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Debug = -4,
    Info = 0,
    Warn = 4,
    Error = 8,
    Fatal = 12,
}
```

### 4.3 Formatters

```rust
pub enum Formatter {
    Text,     // Human-readable with colors
    Json,     // JSON format
    Logfmt,   // key=value format
}

// Reserved keys:
pub const TIMESTAMP_KEY: &str = "time";
pub const MESSAGE_KEY: &str = "msg";
pub const LEVEL_KEY: &str = "level";
pub const CALLER_KEY: &str = "caller";
pub const PREFIX_KEY: &str = "prefix";
```

### 4.4 Styles

```rust
pub struct Styles {
    pub timestamp: Style,
    pub caller: Style,         // Faint
    pub prefix: Style,         // Bold + Faint
    pub message: Style,
    pub key: Style,            // Faint
    pub value: Style,
    pub separator: Style,      // Faint "="
    pub levels: HashMap<Level, Style>,
    pub keys: HashMap<String, Style>,
    pub values: HashMap<String, Style>,
}

// Default level colors:
// Debug: Bold, #63 (Purple)
// Info:  Bold, #86 (Cyan)
// Warn:  Bold, #192 (Yellow)
// Error: Bold, #204 (Red)
// Fatal: Bold, #134 (Magenta)
```

### 4.5 Logger Methods

```rust
impl Logger {
    pub fn new(writer: impl io::Write + Send + 'static) -> Self { ... }

    // Logging methods
    pub fn log(&self, level: Level, msg: &str, keyvals: &[(&str, &str)]) { ... }
    pub fn debug(&self, msg: &str, keyvals: &[(&str, &str)]) { ... }
    pub fn info(&self, msg: &str, keyvals: &[(&str, &str)]) { ... }
    pub fn warn(&self, msg: &str, keyvals: &[(&str, &str)]) { ... }
    pub fn error(&self, msg: &str, keyvals: &[(&str, &str)]) { ... }
    pub fn fatal(&self, msg: &str, keyvals: &[(&str, &str)]) { ... }  // Exits

    // Configuration
    pub fn set_level(&self, level: Level) { ... }
    pub fn set_formatter(&self, f: Formatter) { ... }
    pub fn set_report_timestamp(&self, report: bool) { ... }
    pub fn set_report_caller(&self, report: bool) { ... }
    pub fn set_prefix(&self, prefix: &str) { ... }
    pub fn set_styles(&self, styles: Styles) { ... }

    // Derivation
    pub fn with(&self, keyvals: &[(&str, &str)]) -> Logger { ... }
    pub fn with_prefix(&self, prefix: &str) -> Logger { ... }

    // Helper tracking
    pub fn helper(&self) { ... }
}
```

---

## 5. Glamour — Markdown Rendering

**Purpose:** Markdown to styled ANSI terminal output.

**Source:** `legacy_glamour/` (~1,800 lines of Go)

### 5.1 TermRenderer

```rust
pub struct TermRenderer {
    options: AnsiOptions,
}

pub struct AnsiOptions {
    pub base_url: Option<String>,
    pub word_wrap: usize,              // Default 80
    pub table_wrap: Option<bool>,
    pub inline_table_links: bool,
    pub preserve_new_lines: bool,
    pub color_profile: ColorProfile,
    pub styles: StyleConfig,
    pub chroma_formatter: Option<String>,
}

impl TermRenderer {
    pub fn new(options: Vec<TermRendererOption>) -> Result<Self, Error> { ... }
    pub fn render(&self, markdown: &str) -> Result<String, Error> { ... }
}
```

### 5.2 TermRenderer Options

```rust
pub type TermRendererOption = Box<dyn FnOnce(&mut TermRenderer) -> Result<(), Error>>;

pub fn with_standard_style(name: &str) -> TermRendererOption { ... }
pub fn with_auto_style() -> TermRendererOption { ... }
pub fn with_style_path(path: &str) -> TermRendererOption { ... }
pub fn with_styles(config: StyleConfig) -> TermRendererOption { ... }
pub fn with_word_wrap(width: usize) -> TermRendererOption { ... }
pub fn with_color_profile(profile: ColorProfile) -> TermRendererOption { ... }
pub fn with_emoji() -> TermRendererOption { ... }
pub fn with_preserved_new_lines() -> TermRendererOption { ... }
```

### 5.3 Style Configuration

```rust
pub struct StyleConfig {
    // Block elements
    pub document: StyleBlock,
    pub block_quote: StyleBlock,
    pub paragraph: StyleBlock,
    pub list: StyleList,

    // Headings
    pub heading: StyleBlock,
    pub h1: StyleBlock,
    pub h2: StyleBlock,
    pub h3: StyleBlock,
    pub h4: StyleBlock,
    pub h5: StyleBlock,
    pub h6: StyleBlock,

    // Inline
    pub text: StylePrimitive,
    pub emph: StylePrimitive,          // Italic
    pub strong: StylePrimitive,        // Bold
    pub strikethrough: StylePrimitive,
    pub horizontal_rule: StylePrimitive,

    // Code
    pub code: StyleBlock,              // Inline code
    pub code_block: StyleCodeBlock,    // Block code

    // Links
    pub link: StylePrimitive,
    pub link_text: StylePrimitive,
    pub image: StylePrimitive,
    pub image_text: StylePrimitive,

    // Lists
    pub item: StylePrimitive,
    pub enumeration: StylePrimitive,
    pub task: StyleTask,

    // Tables
    pub table: StyleTable,

    // Definitions
    pub definition_list: StyleBlock,
    pub definition_term: StylePrimitive,
    pub definition_description: StylePrimitive,
}

pub struct StylePrimitive {
    pub block_prefix: Option<String>,
    pub block_suffix: Option<String>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub color: Option<String>,
    pub background_color: Option<String>,
    pub underline: Option<bool>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub crossed_out: Option<bool>,
    pub faint: Option<bool>,
    pub format: Option<String>,  // Template: "{{.text}}"
}

pub struct StyleBlock {
    pub primitive: StylePrimitive,
    pub indent: Option<usize>,
    pub indent_token: Option<String>,
    pub margin: Option<usize>,
}

pub struct StyleCodeBlock {
    pub block: StyleBlock,
    pub theme: String,
    pub chroma: Option<Chroma>,
}

pub struct StyleTable {
    pub block: StyleBlock,
    pub center_separator: Option<String>,
    pub column_separator: Option<String>,
    pub row_separator: Option<String>,
}
```

### 5.4 Built-in Themes

| Theme | Description |
|-------|-------------|
| `"auto"` | Auto-detect dark/light based on terminal background |
| `"dark"` | Dark background theme (default when auto-detect fails) |
| `"light"` | Light background theme |
| `"dracula"` | Dracula color scheme |
| `"tokyo-night"` | Tokyo Night color scheme |
| `"pink"` | Pink accent theme |
| `"ascii"` / `"notty"` | No colors, ASCII only |

**Theme Resolution Algorithm:**
```rust
pub fn resolve_style(style_name: &str) -> StyleConfig {
    match style_name {
        "auto" => {
            if has_dark_background() {
                load_builtin_theme("dark")
            } else {
                load_builtin_theme("light")
            }
        }
        name if Path::new(name).exists() => load_theme_file(name)?,
        name => load_builtin_theme(name),
    }
}
```

### 5.5 Rendering Pipeline

**Step-by-step rendering process:**

```rust
impl TermRenderer {
    pub fn render(&self, markdown: &str) -> Result<String, Error> {
        // 1. Parse markdown with pulldown-cmark
        let parser = Parser::new_ext(markdown, Options::all());

        // 2. Create rendering context
        let mut ctx = RenderContext {
            out: String::new(),
            style_stack: vec![self.options.styles.document.clone()],
            list_depth: 0,
            block_depth: 0,
            footnotes: Vec::new(),
            footnote_refs: Vec::new(),
        };

        // 3. Walk AST and render each element
        for event in parser {
            match event {
                Event::Start(tag) => self.start_tag(&mut ctx, tag),
                Event::End(tag) => self.end_tag(&mut ctx, tag),
                Event::Text(text) => self.render_text(&mut ctx, &text),
                Event::Code(code) => self.render_inline_code(&mut ctx, &code),
                Event::SoftBreak => ctx.out.push(' '),
                Event::HardBreak => ctx.out.push('\n'),
                Event::Rule => self.render_rule(&mut ctx),
                Event::TaskListMarker(checked) => self.render_task(&mut ctx, checked),
            }
        }

        // 4. Append link footnotes if any
        if !ctx.footnotes.is_empty() {
            ctx.out.push_str("\n\n");
            for (i, url) in ctx.footnotes.iter().enumerate() {
                ctx.out.push_str(&format!("[{}]: {}\n", i + 1, url));
            }
        }

        // 5. Apply word wrapping
        if self.options.word_wrap > 0 {
            ctx.out = wrap_text(&ctx.out, self.options.word_wrap);
        }

        Ok(ctx.out)
    }
}
```

**Code Block Rendering with Syntax Highlighting:**
```rust
fn render_code_block(&self, ctx: &mut RenderContext, lang: &str, code: &str) {
    let style = &self.options.styles.code_block;

    // Apply block styling (margin, indent)
    let prefix = " ".repeat(style.block.margin.unwrap_or(0));

    // Syntax highlighting via syntect (if available)
    let highlighted = if let Some(ref chroma) = style.chroma {
        highlight_code(code, lang, chroma)
    } else {
        code.to_string()
    };

    // Apply background and borders
    for line in highlighted.lines() {
        ctx.out.push_str(&prefix);
        ctx.out.push_str(&style.block.primitive.apply(line));
        ctx.out.push('\n');
    }
}

### 5.6 Syntax Highlighting (Chroma)

```rust
pub struct Chroma {
    pub text: StylePrimitive,
    pub error: StylePrimitive,
    pub comment: StylePrimitive,
    pub keyword: StylePrimitive,
    pub keyword_type: StylePrimitive,
    pub operator: StylePrimitive,
    pub punctuation: StylePrimitive,
    pub name: StylePrimitive,
    pub name_function: StylePrimitive,
    pub name_class: StylePrimitive,
    pub literal_string: StylePrimitive,
    pub literal_number: StylePrimitive,
    pub generic_deleted: StylePrimitive,
    pub generic_inserted: StylePrimitive,
    pub background: StylePrimitive,
}
```

---

## 6. Bubbles — TUI Components

**Purpose:** Reusable Bubble Tea components.

**Source:** `legacy_bubbles/` (~4,000 lines of Go)

### 6.1 TextInput

```rust
pub struct TextInput {
    pub value: Vec<char>,
    pub pos: usize,
    pub prompt: String,
    pub placeholder: String,
    pub echo_mode: EchoMode,
    pub echo_character: char,        // Default '*'
    pub char_limit: Option<usize>,
    pub width: Option<usize>,
    pub cursor: Cursor,
    pub suggestions: Vec<String>,
    pub validate: Option<Box<dyn Fn(&str) -> Result<(), String>>>,
    pub keymap: TextInputKeyMap,
    pub styles: TextInputStyles,
}

pub enum EchoMode {
    Normal,
    Password,
    None,
}

// Key bindings:
// ctrl+e: Accept suggestion
// shift+tab: Previous field
// enter/tab: Next field
// left/right: Move cursor
// ctrl+a/ctrl+e: Start/end of line
// ctrl+w: Delete word backward
// ctrl+k: Delete after cursor
// ctrl+u: Delete before cursor
```

### 6.2 TextArea

```rust
pub struct TextArea {
    pub value: Vec<Vec<char>>,       // Grid of lines
    pub row: usize,
    pub col: usize,
    pub prompt: String,
    pub placeholder: String,
    pub show_line_numbers: bool,
    pub char_limit: Option<usize>,
    pub max_height: Option<usize>,
    pub cursor: Cursor,
    pub viewport: Viewport,
    pub keymap: TextAreaKeyMap,
    pub focused_style: TextAreaStyle,
    pub blurred_style: TextAreaStyle,
}

// Key bindings:
// alt+enter/ctrl+j: Insert newline
// ctrl+e: Open external editor
// Arrow keys: Navigation
// ctrl+a/ctrl+e: Line start/end
// alt+</alt+>: Document start/end
```

### 6.3 Spinner

```rust
pub struct Spinner {
    pub frames: Vec<String>,
    pub fps: Duration,
    pub style: Style,
    frame: usize,
}

// Built-in spinners with exact frame sequences:
pub fn line_spinner() -> Spinner {
    // FPS: 10, Frames: ["|", "/", "-", "\\"]
    Spinner::new(vec!["|", "/", "-", "\\"], Duration::from_millis(100))
}

pub fn dot_spinner() -> Spinner {
    // FPS: 10, Frames: Braille animation
    Spinner::new(vec!["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"], Duration::from_millis(100))
}

pub fn mini_dot_spinner() -> Spinner {
    // FPS: 12, Frames: Small dots
    Spinner::new(vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"], Duration::from_millis(83))
}

pub fn jump_spinner() -> Spinner {
    // FPS: 10, Frames: Bouncing ball
    Spinner::new(vec!["⢄", "⢂", "⢁", "⡁", "⡈", "⡐", "⡠"], Duration::from_millis(100))
}

pub fn pulse_spinner() -> Spinner {
    // FPS: 8, Frames: Block gradient
    Spinner::new(vec!["█", "▓", "▒", "░"], Duration::from_millis(125))
}

pub fn points_spinner() -> Spinner {
    // FPS: 7, Frames: Growing dots
    Spinner::new(vec!["∙∙∙", "●∙∙", "∙●∙", "∙∙●"], Duration::from_millis(143))
}

pub fn globe_spinner() -> Spinner {
    // FPS: 4, Frames: Earth rotation
    Spinner::new(vec!["🌍", "🌎", "🌏"], Duration::from_millis(250))
}

pub fn moon_spinner() -> Spinner {
    // FPS: 8, Frames: Moon phases
    Spinner::new(vec!["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"], Duration::from_millis(125))
}

pub fn monkey_spinner() -> Spinner {
    // FPS: 3, Frames: See/Hear/Speak no evil
    Spinner::new(vec!["🙈", "🙉", "🙊"], Duration::from_millis(333))
}

pub fn meter_spinner() -> Spinner {
    // FPS: 7, Frames: Progress bar
    Spinner::new(vec![
        "▱▱▱", "▰▱▱", "▰▰▱", "▰▰▰", "▱▰▰", "▱▱▰", "▱▱▱"
    ], Duration::from_millis(143))
}

pub fn hamburger_spinner() -> Spinner {
    // FPS: 3, Frames: Menu animation
    Spinner::new(vec!["☱", "☲", "☴"], Duration::from_millis(333))
}

pub fn ellipsis_spinner() -> Spinner {
    // FPS: 3, Frames: Typing dots
    Spinner::new(vec!["", ".", "..", "..."], Duration::from_millis(333))
}
```

### 6.4 Progress

```rust
pub struct Progress {
    pub width: usize,
    pub full: char,                     // Default '█'
    pub empty: char,                    // Default '░'
    pub full_color: String,
    pub empty_color: String,
    pub show_percentage: bool,          // Default true
    pub percent_format: String,         // Default " %3.0f%%"
    pub spring: Spring,                 // From harmonica
    percent_shown: f64,
    target_percent: f64,
    velocity: f64,
}

impl Progress {
    pub fn set_percent(&mut self, p: f64) { ... }
    pub fn is_animating(&self) -> bool { ... }
}
```

### 6.5 List

```rust
pub struct List<T: Item> {
    pub title: String,
    pub items: Vec<T>,
    pub filtered_items: Vec<T>,
    pub filter_input: TextInput,
    pub filter_state: FilterState,
    pub paginator: Paginator,
    pub cursor: usize,
    pub delegate: Box<dyn ItemDelegate<T>>,
    pub spinner: Spinner,
    pub help: Help,
    pub styles: ListStyles,
    pub keymap: ListKeyMap,
    pub infinite_scrolling: bool,
    pub filtering_enabled: bool,
}

pub trait Item {
    fn filter_value(&self) -> String;
}

pub trait ItemDelegate<T: Item> {
    fn render(&self, item: &T, index: usize, selected: bool) -> String;
    fn height(&self) -> usize;
    fn spacing(&self) -> usize;
    fn update(&mut self, msg: Msg, items: &mut [T]) -> Option<Cmd>;
}

/// Fuzzy filtering algorithm used by List.
/// Matches items case-insensitively with support for:
/// - Prefix matching (prioritized)
/// - Substring matching
/// - Fuzzy character sequence matching
pub fn fuzzy_filter<T: Item>(items: &[T], query: &str) -> Vec<FilteredItem<T>> {
    if query.is_empty() {
        return items.iter().enumerate()
            .map(|(i, item)| FilteredItem { index: i, rank: 0 })
            .collect();
    }

    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    for (index, item) in items.iter().enumerate() {
        let value = item.filter_value().to_lowercase();

        // Calculate match rank (lower is better)
        let rank = if value.starts_with(&query_lower) {
            0  // Exact prefix match
        } else if value.contains(&query_lower) {
            1  // Substring match
        } else if fuzzy_match(&value, &query_lower) {
            2  // Fuzzy match
        } else {
            continue;  // No match
        };

        results.push(FilteredItem { index, rank });
    }

    // Sort by rank, then by original index
    results.sort_by(|a, b| a.rank.cmp(&b.rank).then(a.index.cmp(&b.index)));
    results
}

fn fuzzy_match(haystack: &str, needle: &str) -> bool {
    let mut needle_chars = needle.chars().peekable();
    for c in haystack.chars() {
        if needle_chars.peek() == Some(&c) {
            needle_chars.next();
        }
        if needle_chars.peek().is_none() {
            return true;
        }
    }
    false
}
```

### 6.6 Table

```rust
pub struct Table {
    pub columns: Vec<Column>,
    pub rows: Vec<Row>,
    pub cursor: usize,
    pub viewport: Viewport,
    pub styles: TableStyles,
    pub keymap: TableKeyMap,
    pub help: Help,
}

pub struct Column {
    pub title: String,
    pub width: usize,
}

pub type Row = Vec<String>;
```

### 6.7 Viewport

```rust
pub struct Viewport {
    pub width: usize,
    pub height: usize,
    pub y_offset: usize,
    pub x_offset: usize,
    pub mouse_wheel_enabled: bool,
    pub mouse_wheel_delta: usize,    // Default 3
    pub style: Style,
    lines: Vec<String>,
}

impl Viewport {
    pub fn set_content(&mut self, content: &str) { ... }
    pub fn scroll_up(&mut self, n: usize) { ... }
    pub fn scroll_down(&mut self, n: usize) { ... }
    pub fn page_up(&mut self) { ... }
    pub fn page_down(&mut self) { ... }
    pub fn half_page_up(&mut self) { ... }
    pub fn half_page_down(&mut self) { ... }
    pub fn goto_top(&mut self) { ... }
    pub fn goto_bottom(&mut self) { ... }
    pub fn at_top(&self) -> bool { ... }
    pub fn at_bottom(&self) -> bool { ... }
    pub fn scroll_percent(&self) -> f64 { ... }
}
```

### 6.8 Paginator

```rust
pub struct Paginator {
    pub page: usize,
    pub per_page: usize,
    pub total_pages: usize,
    pub paginator_type: PaginatorType,
    pub active_dot: String,          // Default "•"
    pub inactive_dot: String,        // Default "○"
    pub arabic_format: String,       // Default "%d/%d"
    pub keymap: PaginatorKeyMap,
}

pub enum PaginatorType {
    Dots,
    Arabic,
}
```

### 6.9 Help

```rust
pub struct Help {
    pub width: usize,
    pub show_all: bool,
    pub short_separator: String,     // Default " • "
    pub full_separator: String,      // Default "    "
    pub ellipsis: String,            // Default "…"
    pub styles: HelpStyles,
}

pub trait KeyMap {
    fn short_help(&self) -> Vec<KeyBinding>;
    fn full_help(&self) -> Vec<Vec<KeyBinding>>;
}
```

### 6.10 FilePicker

```rust
pub struct FilePicker {
    pub current_directory: PathBuf,
    pub path: String,                 // Selected file
    pub allowed_types: Vec<String>,
    pub show_permissions: bool,
    pub show_size: bool,
    pub show_hidden: bool,
    pub dir_allowed: bool,
    pub file_allowed: bool,           // Default true
    pub cursor: String,               // Default ">"
    pub height: usize,
    pub styles: FilePickerStyles,
    pub keymap: FilePickerKeyMap,
}
```

### 6.11 Cursor

```rust
pub struct Cursor {
    pub blink_speed: Duration,       // Default 530ms
    pub style: Style,
    pub text_style: Style,
    pub mode: CursorMode,
    blink: bool,
    char: char,
}

pub enum CursorMode {
    Blink,
    Static,
    Hide,
}
```

---

## 7. Huh — Forms and Prompts

**Purpose:** Interactive form building with validation.

**Source:** `legacy_huh/` (~3,000 lines of Go)

### 7.1 Form Structure

```rust
pub struct Form {
    pub groups: Vec<Group>,
    pub state: FormState,
    pub theme: Theme,
    pub keymap: KeyMap,
    pub width: usize,
    pub height: usize,
    pub accessible: bool,
    current_group_index: usize,
    results: HashMap<String, Box<dyn Any>>,
}

pub enum FormState {
    Normal,
    Completed,
    Aborted,
}

impl Form {
    pub fn new(groups: Vec<Group>) -> Self { ... }
    pub fn with_theme(self, theme: Theme) -> Self { ... }
    pub fn with_width(self, width: usize) -> Self { ... }
    pub fn with_accessible(self, accessible: bool) -> Self { ... }
    pub fn run(&mut self) -> Result<(), FormError> { ... }
    pub fn get<T: 'static>(&self, key: &str) -> Option<&T> { ... }
    pub fn get_string(&self, key: &str) -> Option<String> { ... }
    pub fn get_bool(&self, key: &str) -> Option<bool> { ... }
}
```

### 7.2 Group Structure

```rust
pub struct Group {
    pub fields: Vec<Box<dyn Field>>,
    pub title: String,
    pub description: String,
    pub hide: Option<Box<dyn Fn() -> bool>>,
    current_field_index: usize,
}
```

### 7.3 Field Trait

```rust
pub trait Field: Send + Sync {
    fn init(&mut self);
    fn update(&mut self, msg: Msg) -> Option<Cmd>;
    fn view(&self) -> String;
    fn focus(&mut self) -> Option<Cmd>;
    fn blur(&mut self) -> Option<Cmd>;
    fn error(&self) -> Option<&str>;
    fn key(&self) -> &str;
    fn value(&self) -> Box<dyn Any>;
    fn validate(&self) -> Result<(), String>;
}
```

### 7.4 Input Field

```rust
pub struct Input {
    pub key: String,
    pub title: String,
    pub description: String,
    pub placeholder: String,
    pub prompt: String,
    pub char_limit: Option<usize>,
    pub echo_mode: EchoMode,
    pub suggestions: Vec<String>,
    pub validate: Option<Box<dyn Fn(&str) -> Result<(), String>>>,
    pub inline: bool,
    value: String,
}
```

### 7.5 Select Field

```rust
pub struct Select<T: Clone + PartialEq + 'static> {
    pub key: String,
    pub title: String,
    pub description: String,
    pub options: Vec<SelectOption<T>>,
    pub filtering: bool,
    pub inline: bool,
    pub height: usize,
    pub validate: Option<Box<dyn Fn(&T) -> Result<(), String>>>,
    cursor_index: usize,
    filter_text: String,
}

pub struct SelectOption<T> {
    pub label: String,
    pub value: T,
}
```

### 7.6 MultiSelect Field

```rust
pub struct MultiSelect<T: Clone + PartialEq + 'static> {
    pub key: String,
    pub title: String,
    pub description: String,
    pub options: Vec<SelectOption<T>>,
    pub filterable: bool,
    pub selection_limit: Option<usize>,
    pub height: usize,
    pub validate: Option<Box<dyn Fn(&[T]) -> Result<(), String>>>,
    selected: HashSet<usize>,
    cursor_index: usize,
}
```

### 7.7 Confirm Field

```rust
pub struct Confirm {
    pub key: String,
    pub title: String,
    pub description: String,
    pub affirmative_label: String,   // Default "Yes"
    pub negative_label: String,      // Default "No"
    pub inline: bool,
    pub validate: Option<Box<dyn Fn(bool) -> Result<(), String>>>,
    value: bool,
}
```

### 7.8 Text Field (Multi-line)

```rust
pub struct Text {
    pub key: String,
    pub title: String,
    pub description: String,
    pub placeholder: String,
    pub char_limit: Option<usize>,
    pub show_line_numbers: bool,
    pub external_editor: bool,
    pub editor_command: String,
    pub validate: Option<Box<dyn Fn(&str) -> Result<(), String>>>,
    value: String,
}
```

### 7.9 Note Field

```rust
pub struct Note {
    pub title: String,
    pub description: String,
    pub next_label: String,
    pub show_next_button: bool,
}
```

### 7.10 FilePicker Field

```rust
pub struct FilePickerField {
    pub key: String,
    pub title: String,
    pub description: String,
    pub current_directory: PathBuf,
    pub show_hidden: bool,
    pub file_allowed: bool,
    pub dir_allowed: bool,
    pub allowed_types: Vec<String>,
    pub height: usize,
    pub validate: Option<Box<dyn Fn(&str) -> Result<(), String>>>,
    value: String,
}
```

### 7.11 Theme System

```rust
pub struct Theme {
    pub form: FormStyles,
    pub group: GroupStyles,
    pub focused: FieldStyles,
    pub blurred: FieldStyles,
    pub help: HelpStyles,
}

// Built-in themes:
pub fn charm_theme() -> Theme { ... }       // Default Charm brand colors
pub fn dracula_theme() -> Theme { ... }     // Dracula color scheme
pub fn base16_theme() -> Theme { ... }      // Base16 colors
pub fn catppuccin_theme() -> Theme { ... }  // Catppuccin Mocha

### 7.12 Default Keybindings

**Navigation:**
| Key | Action |
|-----|--------|
| `tab` / `enter` | Next field |
| `shift+tab` | Previous field |
| `ctrl+c` | Abort form |
| `esc` | Abort form (if enabled) |

**Input Field:**
| Key | Action |
|-----|--------|
| `ctrl+e` | Accept suggestion |
| `ctrl+a` | Move to start |
| `ctrl+e` | Move to end |
| `ctrl+k` | Delete to end |
| `ctrl+u` | Delete to start |
| `ctrl+w` | Delete word |
| `left/right` | Move cursor |

**Select/MultiSelect:**
| Key | Action |
|-----|--------|
| `up/k` | Previous option |
| `down/j` | Next option |
| `space` | Toggle selection (MultiSelect) |
| `x` | Toggle selection (MultiSelect) |
| `/` | Start filtering |
| `esc` | Clear filter |

**Confirm:**
| Key | Action |
|-----|--------|
| `left/h` | Toggle to No |
| `right/l` | Toggle to Yes |
| `y` | Select Yes |
| `n` | Select No |

**Text (Multi-line):**
| Key | Action |
|-----|--------|
| `ctrl+j` | Insert newline |
| `alt+enter` | Insert newline |
| `ctrl+e` | Open external editor |

### 7.13 Accessibility Mode

```rust
impl Form {
    /// Enable accessibility mode for screen readers.
    /// - Disables animations
    /// - Uses simpler, linear navigation
    /// - Adds ARIA-like descriptions
    pub fn with_accessible(mut self, accessible: bool) -> Self {
        self.accessible = accessible;
        self
    }
}

// Accessible form provides:
// - Clear field labels read aloud
// - Error messages announced immediately
// - Linear tab navigation (no fancy cursor movement)
// - High-contrast styles
```

---

## 8. Wish — SSH Apps

**Purpose:** SSH server with Bubble Tea TUI middleware.

**Source:** `legacy_wish/` (~1,500 lines of Go)

### 8.1 Server Setup

```rust
pub struct Server {
    pub addr: String,
    pub version: String,
    pub banner: Option<String>,
    pub host_signers: Vec<Signer>,
    pub handler: Handler,
    pub idle_timeout: Option<Duration>,
    pub max_timeout: Option<Duration>,
}

pub type Handler = Box<dyn Fn(&mut Session) + Send + Sync>;
pub type Middleware = fn(next: Handler) -> Handler;

impl Server {
    pub fn new(options: Vec<ServerOption>) -> Result<Self, Error> { ... }
    pub fn listen_and_serve(&self) -> Result<(), Error> { ... }
    pub fn shutdown(&self, ctx: Context) -> Result<(), Error> { ... }
}
```

### 8.2 Server Options

```rust
pub type ServerOption = Box<dyn FnOnce(&mut Server)>;

pub fn with_address(addr: &str) -> ServerOption { ... }
pub fn with_version(v: &str) -> ServerOption { ... }
pub fn with_banner(b: &str) -> ServerOption { ... }
pub fn with_host_key_path(p: &str) -> ServerOption { ... }
pub fn with_host_key_pem(pem: &[u8]) -> ServerOption { ... }
pub fn with_middleware(mw: Vec<Middleware>) -> ServerOption { ... }
pub fn with_idle_timeout(d: Duration) -> ServerOption { ... }
pub fn with_max_timeout(d: Duration) -> ServerOption { ... }
```

### 8.3 Session Interface

```rust
pub trait Session: io::Read + io::Write + Send + Sync {
    fn pty(&self) -> Option<(Pty, Receiver<WindowSize>, bool)>;
    fn emulated_pty(&self) -> bool;
    fn command(&self) -> Vec<String>;
    fn environ(&self) -> Vec<String>;
    fn user(&self) -> String;
    fn public_key(&self) -> Option<&dyn PublicKey>;
    fn context(&self) -> &dyn SessionContext;
    fn stderr(&self) -> Box<dyn io::Write>;
    fn exit(&mut self, code: i32) -> io::Result<()>;
}

pub struct Pty {
    pub term: String,
    pub window: WindowSize,
}

pub struct WindowSize {
    pub width: usize,
    pub height: usize,
}
```

### 8.4 Bubble Tea Middleware

```rust
pub type TeaHandler = fn(&mut Session) -> (Box<dyn Model>, Vec<ProgramOption>);

pub fn bubbletea_middleware(handler: TeaHandler) -> Middleware { ... }
pub fn bubbletea_middleware_with_color_profile(
    handler: TeaHandler,
    profile: ColorProfile
) -> Middleware { ... }

pub fn make_renderer(session: &Session) -> Renderer { ... }
pub fn make_options(session: &Session) -> Vec<ProgramOption> { ... }
```

### 8.5 Access Control Middleware

```rust
pub fn access_control_middleware(allowed_commands: Vec<String>) -> Middleware {
    |next| {
        |session| {
            let cmd = session.command();
            if cmd.is_empty() {
                return next(session);
            }
            if allowed_commands.contains(&cmd[0]) {
                return next(session);
            }
            writeln!(session.stderr(), "Command not allowed: {}", cmd[0]).ok();
            session.exit(1).ok();
        }
    }
}
```

### 8.6 Logging Middleware

```rust
pub fn logging_middleware() -> Middleware { ... }
pub fn logging_middleware_with_logger(logger: &dyn Logger) -> Middleware { ... }
pub fn structured_logging_middleware(logger: &Logger, level: Level) -> Middleware { ... }
```

### 8.7 Rate Limiting Middleware

```rust
pub trait RateLimiter: Send + Sync {
    fn allow(&self, session: &Session) -> Result<(), RateLimitError>;
}

pub fn rate_limiter_middleware(limiter: Box<dyn RateLimiter>) -> Middleware { ... }

pub struct TokenBucketLimiter {
    rate: f64,
    burst: usize,
    max_entries: usize,
}

impl TokenBucketLimiter {
    pub fn new(rate: f64, burst: usize, max_entries: usize) -> Self { ... }
}
```

### 8.8 Panic Recovery Middleware

```rust
pub fn recover_middleware(inner: Vec<Middleware>) -> Middleware { ... }
pub fn recover_middleware_with_logger(
    logger: &dyn Logger,
    inner: Vec<Middleware>
) -> Middleware { ... }
```

### 8.9 Active Terminal Middleware

```rust
pub fn active_term_middleware() -> Middleware {
    |next| {
        |session| {
            if let Some((_, _, true)) = session.pty() {
                return next(session);
            }
            writeln!(session, "Requires an active PTY").ok();
            session.exit(1).ok();
        }
    }
}
```

### 8.10 Color Profile Detection

```rust
/// Detect color profile from SSH session's PTY terminal type.
pub fn detect_profile(term: &str) -> ColorProfile {
    let term_lower = term.to_lowercase();

    // Check for true color support
    if term_lower.contains("truecolor")
        || term_lower.contains("24bit")
        || term_lower.ends_with("-256color")
    {
        return ColorProfile::TrueColor;
    }

    // Check for 256-color support
    if term_lower.contains("256color") || term_lower == "xterm-256color" {
        return ColorProfile::ANSI256;
    }

    // Check for basic ANSI
    if term_lower.starts_with("xterm")
        || term_lower.starts_with("vt100")
        || term_lower.starts_with("screen")
    {
        return ColorProfile::ANSI;
    }

    // Dumb terminals
    if term_lower == "dumb" || term_lower.is_empty() {
        return ColorProfile::Ascii;
    }

    // Default to ANSI for unknown terminals
    ColorProfile::ANSI
}

/// Create a Lipgloss renderer with the detected profile.
pub fn make_renderer(session: &Session) -> Renderer {
    let profile = if let Some((pty, _, _)) = session.pty() {
        detect_profile(&pty.term)
    } else {
        ColorProfile::Ascii
    };

    Renderer::new_with_profile(profile)
}
```

### 8.11 Complete Middleware Stack Example

```rust
use wish::{Server, bubbletea_middleware, logging_middleware,
           rate_limiter_middleware, recover_middleware, active_term_middleware};

fn main() {
    let server = Server::new(vec![
        with_address(":2222"),
        with_host_key_path("./id_ed25519"),
        with_middleware(vec![
            // Order matters: outermost first
            recover_middleware(vec![]),           // Catch panics
            logging_middleware(),                 // Log all sessions
            rate_limiter_middleware(              // Limit connections
                Box::new(TokenBucketLimiter::new(1.0, 10, 1000))
            ),
            active_term_middleware(),             // Require PTY
            bubbletea_middleware(|session| {      // Run TUI
                let renderer = make_renderer(session);
                (Box::new(MyModel::new(renderer)), make_options(session))
            }),
        ]),
    ]).unwrap();

    server.listen_and_serve().unwrap();
}
```

---

## 9. Glow — Markdown CLI

**Purpose:** CLI/TUI markdown reader.

**Source:** `legacy_glow/` (~2,000 lines of Go)

### 9.1 CLI Structure

```rust
// Root command: glow [SOURCE|DIR]
pub struct Args {
    pub source: Option<String>,

    // Flags
    #[arg(short, long)]
    pub pager: bool,

    #[arg(short = 't', long)]
    pub tui: bool,

    #[arg(short, long, default_value = "auto")]
    pub style: String,

    #[arg(short, long, default_value = "0")]
    pub width: u32,

    #[arg(short, long)]
    pub all: bool,

    #[arg(short = 'l', long)]
    pub line_numbers: bool,

    #[arg(short = 'n', long)]
    pub preserve_new_lines: bool,

    #[arg(short, long)]
    pub mouse: bool,

    #[arg(long)]
    pub config: Option<PathBuf>,
}
```

### 9.2 Config File

```yaml
# ~/.config/glow/glow.yml
style: "auto"
mouse: false
pager: false
width: 80
all: false
showLineNumbers: false
preserveNewLines: false
```

### 9.3 Source Resolution

```rust
pub enum Source {
    Stdin,
    File(PathBuf),
    Url(Url),
    GitHub { owner: String, repo: String },
    GitLab { owner: String, repo: String },
    Directory(PathBuf),
}

impl Source {
    pub fn from_arg(arg: &str) -> Result<Self, Error> {
        if arg == "-" { return Ok(Source::Stdin); }
        if arg.starts_with("github://") { ... }
        if arg.starts_with("gitlab://") { ... }
        if arg.starts_with("https://") { ... }
        if Path::new(arg).is_dir() { return Ok(Source::Directory(...)); }
        Ok(Source::File(PathBuf::from(arg)))
    }
}
```

### 9.4 TUI Model

```rust
pub struct Model {
    pub state: AppState,
    pub stash: StashModel,
    pub pager: PagerModel,
    pub cfg: Config,
    pub width: usize,
    pub height: usize,
}

pub enum AppState {
    ShowStash,      // File browser
    ShowDocument,   // Pager view
}

pub struct StashModel {
    pub markdowns: Vec<Markdown>,
    pub filtered_markdowns: Vec<Markdown>,
    pub filter_input: TextInput,
    pub filter_state: FilterState,
    pub paginator: Paginator,
    pub cursor: usize,
    pub spinner: Spinner,
}

pub struct PagerModel {
    pub viewport: Viewport,
    pub current_document: Markdown,
    pub show_help: bool,
}

pub struct Markdown {
    pub local_path: String,
    pub body: String,
    pub note: String,
    pub modtime: SystemTime,
}
```

### 9.5 TUI Key Bindings - File Browser

| Key | Action |
|-----|--------|
| `j`/`↓` | Move down |
| `k`/`↑` | Move up |
| `g`/`home` | Go to top |
| `G`/`end` | Go to bottom |
| `enter` | Open document |
| `/` | Start filter |
| `esc` | Clear filter |
| `e` | Edit in $EDITOR |
| `r` | Refresh |
| `?` | Toggle help |
| `q` | Quit |

### 9.6 TUI Key Bindings - Pager

| Key | Action |
|-----|--------|
| `j`/`↓` | Scroll down |
| `k`/`↑` | Scroll up |
| `d` | Half page down |
| `u` | Half page up |
| `f`/`pgdn` | Page down |
| `b`/`pgup` | Page up |
| `g`/`home` | Go to top |
| `G`/`end` | Go to bottom |
| `c` | Copy to clipboard |
| `e` | Edit in $EDITOR |
| `r` | Reload |
| `?` | Toggle help |
| `q`/`esc` | Back to stash |

### 9.7 GitHub/GitLab URL Resolution

```rust
// GitHub: github://owner/repo → https://api.github.com/repos/{owner}/{repo}/readme
// GitLab: gitlab://owner/repo → https://gitlab.com/api/v4/projects/{projectPath}

pub fn resolve_github_readme(owner: &str, repo: &str) -> Result<String, Error> {
    let api_url = format!("https://api.github.com/repos/{}/{}/readme", owner, repo);
    let response: GitHubReadme = http_get_json(&api_url)?;
    http_get_text(&response.download_url)
}

pub fn resolve_gitlab_readme(owner: &str, repo: &str) -> Result<String, Error> {
    let project = url_encode(&format!("{}/{}", owner, repo));
    let api_url = format!("https://gitlab.com/api/v4/projects/{}", project);
    let response: GitLabProject = http_get_json(&api_url)?;
    let raw_url = response.readme_url.replace("blob", "raw");
    http_get_text(&raw_url)
}
```

### 9.8 Styles and Colors

```rust
// Adaptive colors for light/dark terminals
pub struct AdaptiveColor {
    pub light: String,
    pub dark: String,
}

// Status bar colors
let status_bar_note_fg = AdaptiveColor { light: "#656565", dark: "#7D7D7D" };
let status_bar_bg = AdaptiveColor { light: "#E6E6E6", dark: "#242424" };

// Line number color
let line_number_fg = AdaptiveColor { light: "#656565", dark: "#7D7D7D" };
```

### 9.9 Pager Mode

```rust
/// Simple pager mode (glow --pager FILE.md)
/// Renders markdown and pipes to system pager or built-in viewport.
pub fn run_pager_mode(args: &Args) -> Result<(), Error> {
    let content = read_source(&args.source)?;

    // Render with Glamour
    let renderer = TermRenderer::new(vec![
        with_auto_style(),
        with_word_wrap(args.width.max(terminal_width())),
        if args.preserve_new_lines { with_preserved_new_lines() } else { noop() },
    ])?;

    let rendered = renderer.render(&content)?;

    // Add line numbers if requested
    let output = if args.line_numbers {
        add_line_numbers(&rendered)
    } else {
        rendered
    };

    // Use system pager or print directly
    if let Ok(pager) = std::env::var("PAGER") {
        pipe_to_pager(&pager, &output)
    } else if output.lines().count() > terminal_height() {
        pipe_to_pager("less -R", &output)
    } else {
        println!("{}", output);
        Ok(())
    }
}
```

### 9.10 Glamour Integration

```rust
/// Glow uses Glamour for markdown rendering with these customizations:
impl GlowRenderer {
    pub fn new(config: &Config) -> Result<Self, Error> {
        let style = match config.style.as_str() {
            "auto" => with_auto_style(),
            "dark" => with_standard_style("dark"),
            "light" => with_standard_style("light"),
            path if Path::new(path).exists() => with_style_path(path),
            name => with_standard_style(name),
        };

        let renderer = TermRenderer::new(vec![
            style,
            with_word_wrap(config.width),
            with_color_profile(detect_color_profile()),
            if config.preserve_new_lines {
                with_preserved_new_lines()
            } else {
                noop()
            },
        ])?;

        Ok(Self { renderer, config: config.clone() })
    }
}
```

### 9.11 File Discovery

```rust
/// Find markdown files in a directory for TUI stash view.
pub fn find_markdown_files(dir: &Path, show_hidden: bool) -> Vec<Markdown> {
    let extensions = ["md", "markdown", "mdown", "mkdn", "mkd"];

    walkdir::WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| {
            if !show_hidden {
                !e.file_name().to_str().map(|s| s.starts_with('.')).unwrap_or(false)
            } else {
                true
            }
        })
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| extensions.contains(&ext.to_lowercase().as_str()))
                    .unwrap_or(false)
        })
        .map(|e| Markdown {
            local_path: e.path().to_string_lossy().to_string(),
            body: String::new(),  // Loaded lazily
            note: e.path().file_name().unwrap().to_string_lossy().to_string(),
            modtime: e.metadata().ok().and_then(|m| m.modified().ok()).unwrap_or(SystemTime::UNIX_EPOCH),
        })
        .collect()
}
```

---

## 10. Cross-Library Integration Patterns

This section documents how Charm's libraries compose together to form a complete TUI ecosystem.

### 10.1 Dependency Graph

```
harmonica (standalone)     lipgloss (standalone)
     │                          │
     │                          ├──────────────────────┐
     │                          │                      │
     ▼                          ▼                      ▼
bubbletea ◄────────────── bubbles ◄───────────── charmed_log
     │                     │                          │
     │                     │                          │
     ▼                     ▼                          ▼
   huh ◄──────────────────┘                        wish
     │                                               │
     │                                               │
     ▼                                               ▼
  glamour ◄─────────────────────────────────────── glow
```

### 10.2 Bubbletea + Lipgloss Integration

**Pattern: Styled Model Views**

All Bubbletea models use Lipgloss for rendering:

```rust
// In Model::view()
impl Model for MyModel {
    fn view(&self) -> String {
        let style = Style::new()
            .foreground("#FAFAFA")
            .padding((0, 2));

        style.render(&format!("Count: {}", self.count))
    }
}
```

**Renderer Thread Safety**

Lipgloss uses per-renderer color profiles. Bubbletea creates one Renderer per Program:

```rust
// Renderer is created once per Program
let renderer = Renderer::new(ColorProfile::detect());

// All View() calls use the same renderer
impl Model for MyModel {
    fn view(&self, renderer: &Renderer) -> String {
        renderer.new_style().bold().render("Hello")
    }
}
```

### 10.3 Bubbletea + Harmonica Integration

**Pattern: Animated State Transitions**

Progress bars, spinners, and smooth scrolling use Harmonica springs:

```rust
pub struct AnimatedProgress {
    spring: Spring,
    current: f64,
    target: f64,
    velocity: f64,
}

impl AnimatedProgress {
    fn update(&mut self, msg: Msg) -> Cmd {
        match msg {
            Msg::SetPercent(p) => {
                self.target = p;
                // Return tick command for animation frame
                tick(FRAME_RATE)
            }
            Msg::Tick => {
                // Apply spring physics
                let (pos, vel) = self.spring.update(
                    self.current,
                    self.velocity,
                    self.target
                );
                self.current = pos;
                self.velocity = vel;

                if (self.current - self.target).abs() < 0.001 {
                    Cmd::None
                } else {
                    tick(FRAME_RATE)
                }
            }
        }
    }
}
```

### 10.4 Bubbles + Bubbletea Integration

**Pattern: Composable Components**

Bubbles components are Bubbletea Models themselves:

```rust
pub struct FormModel {
    text_input: textinput::Model,
    spinner: spinner::Model,
    viewport: viewport::Model,
}

impl Model for FormModel {
    fn update(&mut self, msg: Msg) -> Cmd {
        // Delegate to child components
        let (ti, ti_cmd) = self.text_input.update(msg.clone());
        let (sp, sp_cmd) = self.spinner.update(msg.clone());
        let (vp, vp_cmd) = self.viewport.update(msg);

        self.text_input = ti;
        self.spinner = sp;
        self.viewport = vp;

        Cmd::batch(vec![ti_cmd, sp_cmd, vp_cmd])
    }

    fn view(&self) -> String {
        format!(
            "{}\n{}\n{}",
            self.text_input.view(),
            self.spinner.view(),
            self.viewport.view()
        )
    }
}
```

**Pattern: Component Styling**

Each Bubbles component accepts a Styles struct with Lipgloss styles:

```rust
let list = list::Model::new()
    .with_styles(list::Styles {
        title: Style::new().bold().foreground("#FF00FF"),
        selected_item: Style::new().background("#333"),
        normal_item: Style::new().foreground("#888"),
        filter_prompt: Style::new().foreground("#00FF00"),
        ..Default::default()
    });
```

### 10.5 Huh + Bubbles Integration

**Pattern: Field Wrapping**

Huh fields wrap Bubbles components:

```rust
// Huh Input wraps Bubbles textinput
pub struct Input {
    textinput: textinput::Model,  // Bubbles component
    title: String,
    description: String,
    validate: Option<ValidateFn>,
    theme: Theme,
}

// Huh Select wraps Bubbles list (simplified)
pub struct Select<T> {
    viewport: viewport::Model,    // Bubbles viewport for scrolling
    filter: textinput::Model,     // Bubbles textinput for filtering
    options: Vec<Option<T>>,
    selected: usize,
    theme: Theme,
}
```

**Pattern: Theme Propagation**

Huh themes apply Lipgloss styles to all nested Bubbles components:

```rust
impl Theme {
    fn apply_to_input(&self, input: &mut textinput::Model) {
        input.prompt_style = self.focused.text_input.prompt.clone();
        input.text_style = self.focused.text_input.text.clone();
        input.placeholder_style = self.focused.text_input.placeholder.clone();
        input.cursor.style = self.focused.text_input.cursor.clone();
    }
}
```

### 10.6 Wish + Bubbletea Integration

**Pattern: SSH Session to Program**

Wish bridges SSH sessions to Bubbletea Programs:

```rust
// Middleware converts SSH session to Bubbletea Program
pub fn bubbletea_middleware<M: Model>(
    handler: impl Fn(&Session) -> M
) -> Middleware {
    move |next| {
        move |sess: &Session| {
            let model = handler(sess);

            // Get PTY info for window size
            let (pty, window_changes, _) = sess.pty();

            // Create program with SSH I/O
            let program = Program::new(model)
                .with_input(sess.clone())
                .with_output(sess.clone())
                .with_color_profile(detect_profile(&pty.term));

            // Spawn window resize handler
            spawn(async move {
                while let Some(win) = window_changes.recv().await {
                    program.send(WindowSizeMsg {
                        width: win.width as usize,
                        height: win.height as usize,
                    });
                }
            });

            // Run program
            program.run();
        }
    }
}
```

**Pattern: Color Profile Negotiation**

```rust
fn detect_profile(term: &str) -> ColorProfile {
    match term {
        "dumb" => ColorProfile::Ascii,
        t if t.contains("256color") => ColorProfile::Ansi256,
        t if t.contains("truecolor") => ColorProfile::TrueColor,
        _ => {
            // Query terminal via ANSI sequences
            if let Ok(bg) = query_background_color() {
                ColorProfile::TrueColor
            } else {
                ColorProfile::Ansi256
            }
        }
    }
}
```

### 10.7 Glow + Glamour Integration

**Pattern: Markdown Rendering Pipeline**

```rust
pub fn render_markdown(
    content: &str,
    style: &str,
    width: usize,
    base_url: Option<&str>,
) -> Result<String, Error> {
    // Create Glamour renderer with options
    let renderer = TermRenderer::new()
        .with_style_path(style)
        .with_word_wrap(width)
        .with_base_url(base_url.unwrap_or(""));

    renderer.render(content)
}
```

**Pattern: Glamour in Bubbletea Viewport**

```rust
pub struct PagerModel {
    viewport: viewport::Model,
    content: String,
    rendered: String,
    style: String,
}

impl PagerModel {
    fn render_content(&mut self) {
        // Render markdown to ANSI
        let width = self.viewport.width;
        self.rendered = render_markdown(&self.content, &self.style, width, None)
            .unwrap_or_else(|_| self.content.clone());

        // Set viewport content
        self.viewport.set_content(&self.rendered);
    }
}
```

### 10.8 Charmed Log + Lipgloss Integration

**Pattern: Styled Log Output**

```rust
pub struct Logger {
    renderer: Renderer,
    level_styles: HashMap<Level, Style>,
    key_style: Style,
    value_style: Style,
    separator_style: Style,
}

impl Logger {
    fn format_entry(&self, level: Level, msg: &str, fields: &[(&str, &str)]) -> String {
        let mut output = String::new();

        // Level badge
        let level_style = &self.level_styles[&level];
        output.push_str(&level_style.render(&level.to_string()));
        output.push(' ');

        // Message
        output.push_str(msg);

        // Key-value pairs
        for (key, value) in fields {
            output.push(' ');
            output.push_str(&self.key_style.render(key));
            output.push_str(&self.separator_style.render("="));
            output.push_str(&self.value_style.render(value));
        }

        output
    }
}
```

### 10.9 Complete Integration Example

**Full TUI Application Stack**

```rust
// Application using all 9 libraries
use harmonica::Spring;
use lipgloss::{Style, Renderer, ColorProfile};
use bubbletea::{Model, Program, Cmd, Msg};
use charmed_log::Logger;
use glamour::TermRenderer;
use bubbles::{viewport, textinput, list, spinner, progress};
use huh::{Form, Group, Input, Select, Confirm};
use wish::{Server, Middleware};

pub struct App {
    // Core state
    state: AppState,

    // Bubbles components
    viewport: viewport::Model,
    input: textinput::Model,
    list: list::Model<Item>,
    spinner: spinner::Model,
    progress: progress::Model,

    // Animation
    scroll_spring: Spring,

    // Styling
    renderer: Renderer,
    theme: Theme,

    // Logging
    logger: Logger,
}

impl Model for App {
    fn init(&self) -> Cmd {
        Cmd::batch(vec![
            self.spinner.tick(),
            self.load_content(),
        ])
    }

    fn update(&mut self, msg: Msg) -> Cmd {
        match msg {
            Msg::Key(key) => self.handle_key(key),
            Msg::Tick => self.animate(),
            Msg::Loaded(content) => {
                // Render with Glamour
                let md = TermRenderer::new()
                    .with_style_path(&self.theme.glamour_style)
                    .with_word_wrap(self.viewport.width)
                    .render(&content)?;
                self.viewport.set_content(&md);
                Cmd::None
            }
            _ => Cmd::None
        }
    }

    fn view(&self) -> String {
        let title = self.theme.title.render("My TUI App");
        let content = self.viewport.view();
        let status = self.render_status();

        format!("{}\n{}\n{}", title, content, status)
    }
}

// SSH server entry point (Wish)
fn main() {
    let server = Server::new()
        .with_address(":2222")
        .with_middleware(bubbletea_middleware(|sess| {
            App::new(detect_profile(&sess.pty().term))
        }));

    server.listen_and_serve();
}
```

### 10.10 Message Flow Patterns

**Cross-Component Communication**

```
┌─────────────┐      tea.KeyMsg       ┌─────────────┐
│   Program   │ ─────────────────────▶│    Model    │
└─────────────┘                       └──────┬──────┘
                                             │
                    ┌────────────────────────┼────────────────────────┐
                    │                        │                        │
                    ▼                        ▼                        ▼
            ┌──────────────┐        ┌──────────────┐        ┌──────────────┐
            │  TextInput   │        │   Viewport   │        │   Spinner    │
            │   (Bubbles)  │        │   (Bubbles)  │        │   (Bubbles)  │
            └──────┬───────┘        └──────┬───────┘        └──────┬───────┘
                   │                       │                       │
                   │ textinput.Msg         │ viewport.Msg          │ spinner.Msg
                   │                       │                       │
                   └───────────────────────┼───────────────────────┘
                                           │
                                           ▼
                                    ┌──────────────┐
                                    │    Cmd       │
                                    │ (batch all)  │
                                    └──────────────┘
```

### 10.11 Color Profile Cascade

```
┌──────────────────────────────────────────────────────────────────┐
│                      Environment Detection                        │
│  termenv.ColorProfile() → TrueColor / ANSI256 / ANSI / Ascii     │
└───────────────────────────────┬──────────────────────────────────┘
                                │
                                ▼
┌──────────────────────────────────────────────────────────────────┐
│                      Renderer Creation                            │
│  lipgloss.NewRenderer(WithColorProfile(profile))                 │
└───────────────────────────────┬──────────────────────────────────┘
                                │
                    ┌───────────┼───────────┐
                    │           │           │
                    ▼           ▼           ▼
              ┌──────────┐ ┌──────────┐ ┌──────────┐
              │ Lipgloss │ │ Glamour  │ │Charmed   │
              │  Styles  │ │ Renderer │ │  Log     │
              └──────────┘ └──────────┘ └──────────┘
                    │           │           │
                    └───────────┼───────────┘
                                │
                                ▼
                    ┌──────────────────────┐
                    │  Degraded ANSI codes │
                    │  (if needed by env)  │
                    └──────────────────────┘
```

---

*Last updated: 2026-01-17*
