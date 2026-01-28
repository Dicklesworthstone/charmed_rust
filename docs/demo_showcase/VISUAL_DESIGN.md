# Visual Design System

This document defines the visual language for Charmed Control Center, ensuring the application feels cohesive and polished across all pages and components.

## Design Principles

1. **Clarity First**: Information hierarchy is immediately visible
2. **Consistent Density**: Comfortable information density without feeling cramped
3. **Accessible by Default**: Works in 16-color terminals, gracefully degrades
4. **Theme Flexibility**: All colors come from semantic tokens, never hardcoded

---

## Spacing Scale

Based on a 4-unit base (where 1 unit = 1 terminal cell for width, 1 line for height).

| Token | Value | Usage |
|-------|-------|-------|
| `xs`  | 1     | Icon-to-text gap, tight inline spacing |
| `sm`  | 2     | Compact padding, list item spacing |
| `md`  | 4     | Standard padding, section margins |
| `lg`  | 6     | Major section separation |
| `xl`  | 8     | Page-level padding, modal margins |

### Application Rules

- **Padding inside boxes**: `md` (1 line vertical, 2 chars horizontal)
- **Gap between components**: `sm` vertical, `md` horizontal
- **Sidebar width**: 12-14 chars (fixed)
- **Header/Footer height**: 1 line each
- **Minimum content width**: 60 chars

---

## Border Styles

### Rounded Borders (`Border::rounded()`)
Use for:
- Content boxes and cards
- Modal dialogs
- Help overlays
- Focus indicators

### Normal Borders (`Border::normal()`)
Use for:
- Tables and grids
- Code blocks
- Terminal output

### Double Borders (`Border::double()`)
Use for:
- Critical alerts/warnings
- Modal dialogs requiring attention
- Confirmation dialogs

### No Border
Use for:
- Inline elements
- Status indicators
- Dense lists

### Border Colors
- **Default**: `border` token (subtle, low contrast)
- **Focused/Active**: `border_focus` token (primary color)
- **Error state**: `error` token
- **Success state**: `success` token

---

## Color Semantics

All colors are referenced via semantic tokens, never raw hex values in components.

### Semantic Token Mapping

| Token | Purpose | Dark | Light | Dracula |
|-------|---------|------|-------|---------|
| `primary` | Brand color, accent, interactive elements | `#7D56F4` | `#6B46C1` | `#BD93F9` |
| `secondary` | Secondary accent, less prominent | `#FF69B4` | `#D53F8C` | `#FF79C6` |
| `success` | Healthy, complete, positive | `#00FF00` | `#38A169` | `#50FA7B` |
| `warning` | Needs attention, degraded | `#FFCC00` | `#D69E2E` | `#F1FA8C` |
| `error` | Failed, critical, action needed | `#FF0000` | `#E53E3E` | `#FF5555` |
| `info` | Informational, neutral highlight | `#00BFFF` | `#3182CE` | `#8BE9FD` |
| `text` | Primary text, high contrast | `#FFFFFF` | `#1A202C` | `#F8F8F2` |
| `text_muted` | Secondary text, hints, timestamps | `#626262` | `#718096` | `#6272A4` |
| `text_inverse` | Text on colored backgrounds | `#000000` | `#FFFFFF` | `#282A36` |
| `bg` | Main background | `#000000` | `#FFFFFF` | `#282A36` |
| `bg_subtle` | Sidebar, header, card backgrounds | `#1A1A1A` | `#F7FAFC` | `#343746` |
| `bg_highlight` | Hover, selection, active states | `#333333` | `#EDF2F7` | `#44475A` |
| `border` | Subtle borders, dividers | `#444444` | `#E2E8F0` | `#44475A` |
| `border_focus` | Focused element borders | `#7D56F4` | `#6B46C1` | `#BD93F9` |

### Usage Guidelines

1. **Never mix semantic tokens**: A success indicator should only use `success`, not a mix of green shades
2. **Background layering**: `bg` < `bg_subtle` < `bg_highlight` (increasing prominence)
3. **Text contrast**: Always pair `text` with `bg`, `text_muted` with `bg_subtle`

---

## Typography

Terminal typography is limited, but we use these techniques:

### Weight/Emphasis

| Style | Method | Usage |
|-------|--------|-------|
| **Bold** | `Style::bold()` | Titles, headings, selected items, important values |
| *Faint* | `Style::faint()` | Timestamps, IDs, secondary info |
| Underline | `Style::underline()` | Links, keyboard shortcuts |
| Italic | `Style::italic()` | Descriptions, quotes (limited terminal support) |
| Reverse | `Style::reverse()` | Critical alerts, selection indicators |

### Hierarchy Rules

1. **Level 1 (Page Title)**: Bold + Primary color
2. **Level 2 (Section Header)**: Bold + Text color
3. **Level 3 (Subsection)**: Bold only
4. **Body**: Normal weight + Text color
5. **Meta/Hint**: Faint or Muted color

### Text Alignment

- **Labels**: Right-aligned in forms, left-aligned in lists
- **Values**: Left-aligned (numbers optionally right-aligned)
- **Actions/Buttons**: Centered

---

## Theme Presets

### Dark (Default)
- Optimized for dark terminals (most common)
- High-contrast text on pure black
- Vibrant accent colors for visibility

### Light
- Clean, professional appearance
- Reduced contrast for eye comfort
- Softer semantic colors (greens, yellows)

### Dracula
- Popular community theme
- Purple primary, pink secondary
- Muted background with vibrant accents

### Nord (Future)
- Arctic, bluish color palette
- Calm, low-contrast design
- Primary: `#88C0D0`, Background: `#2E3440`

### Catppuccin Mocha (Future)
- Warm, pastel aesthetic
- Primary: `#CBA6F7`, Background: `#1E1E2E`
- Rounded, friendly feel

### ASCII/NoColor (Accessibility)
- No ANSI colors, pure ASCII
- Uses `+`, `-`, `|` for borders
- Bold and underline only for emphasis
- For terminals without color support

---

## Component Patterns

### Status Indicators

```
[●] Healthy    (success color, filled circle)
[◐] Degraded   (warning color, half circle)
[○] Unhealthy  (error color, empty circle)
[?] Unknown    (muted color, question mark)
```

### Progress Bars

```
[####------] 40%   (fill with #, empty with -)
```

Width: 10-20 chars depending on context

### Tables

```
╭──────────┬────────┬──────────╮
│ Service  │ Status │ Version  │
├──────────┼────────┼──────────┤
│ api      │   ●    │ 1.2.3    │
│ worker   │   ◐    │ 1.2.2    │
╰──────────┴────────┴──────────╯
```

Use rounded corners, center status columns.

### Lists (Selectable)

```
   Dashboard
 > Services    (selected: prefix with >, bold, primary color)
   Jobs
   Logs
```

### Keyboard Hints

```
↑/↓ navigate  Enter select  q quit  ? help
```

Style: Muted color, displayed in footer.

---

## Animation Guidelines

1. **Spinners**: Use standard spinner styles from bubbles (dot, line, etc.)
2. **Transitions**: No animation between pages (instant switch)
3. **Progress**: Smooth updates, not jumpy
4. **Tick Rate**: 100ms for spinners, 500ms for status updates

---

## Accessibility Fallbacks

### 16-Color Mode
When `TERM` indicates limited color support:
- Map semantic colors to ANSI 16 palette
- Primary → Blue/Cyan
- Success → Green
- Warning → Yellow
- Error → Red

### No-Color Mode
When `NO_COLOR` is set or `TERM=dumb`:
- Disable all ANSI sequences
- Use ASCII box drawing (`+`, `-`, `|`)
- Rely on Bold, Underline, spacing for hierarchy
- Status indicators: `[OK]`, `[WARN]`, `[ERR]`, `[??]`

---

## Implementation Notes

### Style Helper Methods

The `Theme` struct should provide these methods:

```rust
// Layout
fn box_style(&self) -> Style        // Rounded border, border color
fn box_focused_style(&self) -> Style // Rounded border, focus color
fn card_style(&self) -> Style        // Subtle background, no border

// Text
fn title_style(&self) -> Style       // Bold + primary
fn heading_style(&self) -> Style     // Bold
fn muted_style(&self) -> Style       // text_muted color
fn link_style(&self) -> Style        // Underline + info color

// Status
fn success_style(&self) -> Style
fn warning_style(&self) -> Style
fn error_style(&self) -> Style
fn info_style(&self) -> Style

// Interactive
fn selected_style(&self) -> Style    // Bold + primary + highlight bg
fn hover_style(&self) -> Style       // Highlight background
```

### Spacing Constants

```rust
pub const SPACE_XS: u16 = 1;
pub const SPACE_SM: u16 = 2;
pub const SPACE_MD: u16 = 4;
pub const SPACE_LG: u16 = 6;
pub const SPACE_XL: u16 = 8;

pub const SIDEBAR_WIDTH: u16 = 14;
pub const MIN_CONTENT_WIDTH: u16 = 60;
```

---

## Versioning

This design system is versioned alongside the application:
- Major version changes may update the visual language
- Theme presets are additive (new presets don't remove existing ones)
- Semantic tokens are stable (renaming requires migration)
