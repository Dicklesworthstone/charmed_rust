# Theming Best Practices

Guidelines for building accessible, performant themed applications with lipgloss.

## Do's

### Use Semantic Color Slots

Always use semantic slots instead of hardcoded colors:

```rust
// Good: Semantic meaning
let error_style = ThemedStyle::new(ctx.clone())
    .foreground(ColorSlot::Error);

// Avoid: Hardcoded color
let error_style = Style::new()
    .foreground(Color::from_hex("#ff0000"));
```

### Test with Multiple Themes

Verify your app looks good with both light and dark themes:

```rust
#[test]
fn test_renders_with_all_presets() {
    let presets = [
        ThemePreset::Dark,
        ThemePreset::Light,
        ThemePreset::Dracula,
        ThemePreset::Nord,
    ];

    for preset in presets {
        let ctx = Arc::new(ThemeContext::from_preset(preset));
        let style = ThemedStyle::new(ctx).foreground(ColorSlot::Primary);
        let output = style.render("Test");
        assert!(!output.is_empty());
    }
}
```

### Check Contrast Ratios

Ensure text is readable:

```rust
fn validate_custom_theme(theme: &Theme) -> Result<(), String> {
    // WCAG AA minimum
    let pairs = [
        (ColorSlot::Foreground, ColorSlot::Background),
        (ColorSlot::Error, ColorSlot::Background),
        (ColorSlot::Success, ColorSlot::Background),
    ];

    for (fg, bg) in pairs {
        if !theme.check_contrast_aa(fg, bg) {
            return Err(format!(
                "{:?} on {:?} has insufficient contrast",
                fg, bg
            ));
        }
    }
    Ok(())
}
```

### Cache Resolved Styles

Use `CachedThemedStyle` for performance in hot paths:

```rust
use lipgloss::CachedThemedStyle;

struct Component {
    // Cache the resolved style
    style: CachedThemedStyle,
}

impl Component {
    fn render(&self, text: &str) -> String {
        // Uses cached style, fast!
        self.style.render(text)
    }
}
```

### Create Styles Once

Initialize styles at startup, not per-render:

```rust
// Good: Create once, reuse
struct App {
    styles: AppStyles,
}

impl App {
    fn render(&self) -> String {
        // Just render, don't recreate styles
        self.styles.title.render("Hello")
    }
}

// Avoid: Creating styles every render
impl App {
    fn render(&self) -> String {
        let style = ThemedStyle::new(self.ctx.clone())
            .foreground(ColorSlot::Primary);
        style.render("Hello")
    }
}
```

### Use the Global Context for Simple Apps

For single-threaded apps without complex state:

```rust
use lipgloss::{set_global_preset, ThemedStyle, ThemePreset};

fn main() {
    set_global_preset(ThemePreset::Dark);

    // Simple, no Arc needed
    let style = ThemedStyle::global().foreground(ColorSlot::Primary);
}
```

## Don'ts

### Don't Mix Themed and Hardcoded Colors

Be consistent within a component:

```rust
// Avoid: Mixing approaches
let box_style = ThemedStyle::new(ctx.clone())
    .foreground(ColorSlot::Primary)
    .background(Color::from_hex("#1a1a1a")); // Hardcoded!

// Good: All themed
let box_style = ThemedStyle::new(ctx.clone())
    .foreground(ColorSlot::Primary)
    .background(ColorSlot::Surface);
```

### Don't Assume Dark Mode

Support both light and dark themes:

```rust
// Avoid: Assuming dark background
let style = Style::new()
    .foreground(Color::from_hex("#ffffff")); // Won't work on light bg

// Good: Let theme decide
let style = ThemedStyle::new(ctx)
    .foreground(ColorSlot::Foreground);
```

### Don't Ignore Accessibility

Low contrast hurts usability for everyone:

```rust
// Avoid: Poor contrast choices
theme.colors_mut().foreground = Color::from_hex("#888888");
theme.colors_mut().background = Color::from_hex("#666666");

// Good: Validate before using
if !custom_theme.check_contrast_aa(ColorSlot::Foreground, ColorSlot::Background) {
    eprintln!("Warning: Theme has poor contrast");
    return Err("Theme failed accessibility check");
}
```

### Don't Store Resolved Colors

Let `ThemedStyle` resolve at render time:

```rust
// Avoid: Storing resolved color
struct BadComponent {
    color: Color, // Stale if theme changes!
}

// Good: Store the ThemedStyle
struct GoodComponent {
    style: ThemedStyle, // Resolves on render
}
```

### Don't Switch Themes in Tight Loops

Theme switching has overhead:

```rust
// Avoid: Switching in hot path
for item in items {
    ctx.set_preset(item.theme); // Expensive!
    render(item);
}

// Good: Batch by theme
let by_theme = items.group_by(|i| i.theme);
for (preset, group) in by_theme {
    ctx.set_preset(preset);
    for item in group {
        render(item);
    }
}
```

### Don't Create Many Theme Contexts

Share a single context:

```rust
// Avoid: Context per component
fn create_component() -> Component {
    let ctx = Arc::new(ThemeContext::from_preset(ThemePreset::Dark));
    Component { ctx }
}

// Good: Share context
fn create_component(ctx: Arc<ThemeContext>) -> Component {
    Component { ctx }
}
```

## Performance Tips

### 1. Batch Style Creation

```rust
struct Styles {
    title: ThemedStyle,
    body: ThemedStyle,
    footer: ThemedStyle,
}

impl Styles {
    fn new(ctx: Arc<ThemeContext>) -> Self {
        // Create all at once
        Self {
            title: ThemedStyle::new(ctx.clone()).foreground(ColorSlot::Primary).bold(),
            body: ThemedStyle::new(ctx.clone()).foreground(ColorSlot::Foreground),
            footer: ThemedStyle::new(ctx).foreground(ColorSlot::Muted),
        }
    }
}
```

### 2. Use CachedThemedStyle for Repeated Renders

```rust
// For styles used many times per frame
let cached = CachedThemedStyle::new(
    ThemedStyle::new(ctx).foreground(ColorSlot::Primary)
);

// Invalidate when theme changes
ctx.on_change(|_| cached.invalidate());
```

### 3. Minimize Theme Context Clones

```rust
// Arc clone is cheap, but avoid in hot paths
let ctx = Arc::new(ThemeContext::from_preset(ThemePreset::Dark));

// Pass reference where possible
fn render_with_ctx(ctx: &ThemeContext) {
    // Use ctx.current() instead of cloning
}
```

### 4. Prefer Presets Over Custom Themes

Built-in presets are optimized and pre-validated:

```rust
// Fast: Built-in preset
let theme = ThemePreset::Dracula.to_theme();

// Slower: Loading from file (I/O + parsing)
let theme = Theme::from_file("custom.toml")?;
```

## Accessibility Checklist

- [ ] Main text passes WCAG AA (4.5:1 contrast)
- [ ] Large text passes WCAG AA (3:1 contrast)
- [ ] Error/success/warning states are distinguishable
- [ ] UI doesn't rely solely on color (use icons, labels)
- [ ] Tested with colorblind simulation tools
- [ ] Light and dark themes both work
- [ ] Custom themes are validated before use

## Common Pitfalls

### Pitfall: Forgetting to Update After Theme Change

```rust
// Problem: Cached value becomes stale
let color = ctx.current().get(ColorSlot::Primary);
// ... later, theme changes, but `color` is old

// Solution: Re-resolve when needed, or use ThemedStyle
let style = ThemedStyle::new(ctx).foreground(ColorSlot::Primary);
// style.render() always uses current theme
```

### Pitfall: Thread Safety with Global Context

```rust
// Problem: Global context modification from multiple threads
std::thread::spawn(|| {
    set_global_preset(ThemePreset::Dark); // Race condition!
});

// Solution: Use explicit context with proper synchronization
let ctx = Arc::new(ThemeContext::from_preset(ThemePreset::Dark));
// Share ctx across threads safely
```

### Pitfall: Inconsistent Borders

```rust
// Problem: Border color doesn't match theme
let style = ThemedStyle::new(ctx.clone())
    .foreground(ColorSlot::Primary)
    .border(Border::rounded());
// Border uses default color, not themed!

// Solution: Set border color explicitly
let style = ThemedStyle::new(ctx.clone())
    .foreground(ColorSlot::Primary)
    .border(Border::rounded())
    .border_foreground(ColorSlot::Border);
```

## Next Steps

- [Theming Tutorial](theming-tutorial.md) - Step-by-step guide
- [Custom Themes](custom-themes.md) - Create your own themes
