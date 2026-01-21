# Creating Custom Themes

This guide covers creating and using custom themes in lipgloss.

## Theme File Structure

Themes can be defined in JSON, TOML, or YAML format. All formats support the same schema.

## Minimal Theme

### TOML (Recommended)

```toml
name = "My Theme"
is_dark = true

[colors]
background = "#1a1a2e"
foreground = "#eaeaea"
primary = "#e94560"
secondary = "#0f3460"
success = "#2ecc71"
warning = "#f39c12"
error = "#e74c3c"
info = "#3498db"
muted = "#666666"
border = "#333333"
```

### JSON

```json
{
  "name": "My Theme",
  "is_dark": true,
  "colors": {
    "background": "#1a1a2e",
    "foreground": "#eaeaea",
    "primary": "#e94560",
    "secondary": "#0f3460",
    "success": "#2ecc71",
    "warning": "#f39c12",
    "error": "#e74c3c",
    "info": "#3498db",
    "muted": "#666666",
    "border": "#333333"
  }
}
```

### YAML

```yaml
name: My Theme
is_dark: true
colors:
  background: "#1a1a2e"
  foreground: "#eaeaea"
  primary: "#e94560"
  secondary: "#0f3460"
  success: "#2ecc71"
  warning: "#f39c12"
  error: "#e74c3c"
  info: "#3498db"
  muted: "#666666"
  border: "#333333"
```

## Complete Theme with Metadata

```toml
name = "Cyberpunk"
is_dark = true

[meta]
description = "A neon-inspired dark theme"
author = "Your Name"
version = "1.0.0"

[colors]
# Core colors
background = "#0d0d0d"
foreground = "#00ff9f"

# Accent colors
primary = "#ff00ff"
secondary = "#00ffff"
accent = "#ffff00"

# Semantic colors
success = "#00ff00"
warning = "#ff9900"
error = "#ff0000"
info = "#00ffff"

# UI colors
muted = "#666666"
border = "#333333"
selection = "#1a1a3e"
surface = "#1a1a1a"

# Custom colors (accessible via get_custom)
[colors.custom]
neon_pink = "#ff1493"
neon_blue = "#00bfff"
neon_green = "#39ff14"
```

## Color Format Options

lipgloss accepts colors in multiple formats:

| Format | Example | Notes |
|--------|---------|-------|
| Hex (6-digit) | `"#ff0000"` | Standard hex color |
| Hex (no hash) | `"ff0000"` | Hash is optional |
| Hex (3-digit) | `"#f00"` | Expands to `#ff0000` |
| ANSI 256 | `196` | ANSI 256-color code |
| RGB (TOML) | `{ r = 255, g = 0, b = 0 }` | Object notation |

## Color Slots Reference

| Slot | Purpose | Typical Usage |
|------|---------|---------------|
| `background` | Main background | App background |
| `foreground` | Default text | Body text |
| `primary` | Primary accent | Buttons, links, headings |
| `secondary` | Secondary accent | Secondary actions |
| `accent` | Highlight color | Focus indicators |
| `success` | Success state | Confirmations, checkmarks |
| `warning` | Warning state | Alerts, caution text |
| `error` | Error state | Error messages, validation |
| `info` | Information | Notices, help text |
| `muted` | De-emphasized | Disabled, placeholder |
| `border` | Borders | Dividers, boxes |
| `selection` | Selected items | Highlighted rows |
| `surface` | Elevated surfaces | Cards, modals |

## Loading Custom Themes

### From File

```rust
use lipgloss::Theme;

// Auto-detects format from extension
let theme = Theme::from_file("~/.config/myapp/theme.toml")?;

// Or parse directly
let toml_content = std::fs::read_to_string("theme.toml")?;
let theme = Theme::from_toml(&toml_content)?;
```

### From Embedded String

```rust
use lipgloss::Theme;

const THEME_JSON: &str = r#"{
    "name": "Embedded",
    "is_dark": true,
    "colors": {
        "background": "#000000",
        "foreground": "#ffffff",
        "primary": "#ff0000"
    }
}"#;

let theme = Theme::from_json(THEME_JSON)?;
```

## Saving Themes

```rust
use lipgloss::Theme;

let theme = Theme::dark();

// Save to file (format detected from extension)
theme.to_file("my-theme.toml")?;

// Or serialize directly
let json = theme.to_json()?;
let toml = theme.to_toml()?;
let yaml = theme.to_yaml()?;
```

## Creating Themes Programmatically

```rust
use lipgloss::{Theme, ThemeColors, Color};

let colors = ThemeColors::dark()
    .with_primary(Color::from_hex("#ff0000"))
    .with_secondary(Color::from_hex("#00ff00"));

let theme = Theme::new("Custom", true, colors)
    .with_description("My custom theme")
    .with_author("Me");
```

## Theme Variants

Create light/dark variants of the same theme:

```rust
use lipgloss::{Theme, ThemeColors};

fn create_theme_pair() -> (Theme, Theme) {
    let dark = Theme::new("MyTheme Dark", true, ThemeColors {
        background: "#1a1a1a".into(),
        foreground: "#ffffff".into(),
        primary: "#3498db".into(),
        // ... other colors
        ..ThemeColors::dark()
    });

    let light = Theme::new("MyTheme Light", false, ThemeColors {
        background: "#ffffff".into(),
        foreground: "#1a1a1a".into(),
        primary: "#2980b9".into(),
        // ... other colors
        ..ThemeColors::light()
    });

    (dark, light)
}
```

## Extending Built-in Presets

Start from a preset and customize:

```rust
use lipgloss::{ThemePreset, Color};

let mut theme = ThemePreset::Dracula.to_theme();

// Customize specific colors
theme.colors_mut().primary = Color::from_hex("#ff79c6");
theme.colors_mut().custom_mut().insert(
    "my_color".to_string(),
    Color::from_hex("#bd93f9"),
);
```

## Ensuring Accessibility

### Contrast Checking

```rust
use lipgloss::{Theme, ColorSlot};

let theme = Theme::from_file("my-theme.toml")?;

// WCAG AA: 4.5:1 for normal text, 3:1 for large text
if !theme.check_contrast_aa(ColorSlot::Foreground, ColorSlot::Background) {
    eprintln!("Warning: Text may be hard to read");
}

// WCAG AAA: 7:1 for normal text (higher standard)
if theme.check_contrast_aaa(ColorSlot::Foreground, ColorSlot::Background) {
    println!("Excellent contrast!");
}

// Get exact ratio
let ratio = theme.contrast_ratio(ColorSlot::Foreground, ColorSlot::Background);
println!("Contrast ratio: {:.2}:1", ratio);
```

### Accessibility Guidelines

1. **Minimum contrast**: 4.5:1 for body text
2. **Large text**: 3:1 minimum (18pt+ or 14pt bold)
3. **UI elements**: 3:1 for borders, icons
4. **Error states**: Ensure error text is readable
5. **Don't rely on color alone**: Use icons, patterns, or text

## Theme Validation

```rust
use lipgloss::Theme;

let theme = Theme::from_file("untrusted-theme.toml")?;

match theme.validate() {
    Ok(()) => println!("Theme is valid"),
    Err(e) => eprintln!("Theme validation failed: {}", e),
}
```

## Custom Color Slots

For app-specific colors not covered by standard slots:

```rust
use lipgloss::{Theme, ThemeColors, Color};

// In theme file (TOML):
// [colors.custom]
// syntax_keyword = "#ff79c6"
// syntax_string = "#f1fa8c"

// In code:
let theme = Theme::from_file("theme.toml")?;

if let Some(keyword_color) = theme.colors().get_custom("syntax_keyword") {
    let style = Style::new().foreground(keyword_color.clone());
    // Use the style...
}
```

## Example Theme Files

### Solarized Dark

```toml
name = "Solarized Dark"
is_dark = true

[meta]
author = "Ethan Schoonover"
description = "Precision colors for machines and people"

[colors]
background = "#002b36"
foreground = "#839496"
primary = "#268bd2"
secondary = "#2aa198"
accent = "#b58900"
success = "#859900"
warning = "#cb4b16"
error = "#dc322f"
info = "#268bd2"
muted = "#586e75"
border = "#073642"
selection = "#073642"
surface = "#073642"
```

### High Contrast

```toml
name = "High Contrast"
is_dark = true

[meta]
description = "Accessibility-focused high contrast theme"

[colors]
background = "#000000"
foreground = "#ffffff"
primary = "#ffff00"
secondary = "#00ffff"
accent = "#ff00ff"
success = "#00ff00"
warning = "#ffaa00"
error = "#ff0000"
info = "#00aaff"
muted = "#aaaaaa"
border = "#ffffff"
selection = "#444444"
surface = "#222222"
```

## Next Steps

- [Theming Tutorial](theming-tutorial.md) - Build a themed application
- [Best Practices](theming-best-practices.md) - Tips and common pitfalls
