# Building a Themed Terminal Application

This tutorial walks through creating a terminal application with full theming support using lipgloss's theme system.

## Overview

The lipgloss theme system provides:

- **Built-in presets**: Popular color schemes like Dracula, Nord, and Catppuccin
- **Custom themes**: Define your own color palette with semantic slots
- **Runtime switching**: Change themes dynamically without restarting
- **Auto-updating styles**: `ThemedStyle` automatically reflects theme changes
- **Serialization**: Save and load themes from JSON/TOML/YAML files

## Step 1: Choose a Theme Strategy

You have three main approaches:

| Strategy | Use Case | Complexity |
|----------|----------|------------|
| Static theme | Simple apps, utilities | Low |
| User-selectable | Apps with settings menu | Medium |
| Fully customizable | Power-user tools | High |

## Step 2: Using Built-in Presets

The simplest approach is using a built-in preset:

```rust
use lipgloss::{Style, ThemePreset, ColorSlot};

fn main() {
    // Convert preset to a Theme
    let theme = ThemePreset::Dracula.to_theme();

    // Get colors from semantic slots
    let primary = theme.get(ColorSlot::Primary);
    let error = theme.get(ColorSlot::Error);

    // Create styles with theme colors
    let title_style = Style::new().foreground(primary).bold(true);
    let error_style = Style::new().foreground(error);

    println!("{}", title_style.render("Welcome!"));
    println!("{}", error_style.render("Something went wrong"));
}
```

Available presets:
- `ThemePreset::Dark` - Default dark theme
- `ThemePreset::Light` - Default light theme
- `ThemePreset::Dracula` - Popular purple-tinted dark theme
- `ThemePreset::Nord` - Arctic-inspired palette
- `ThemePreset::Catppuccin(flavor)` - Pastel themes in 4 flavors

## Step 3: Set Up a Theme Context

For runtime theme switching, use `ThemeContext`:

```rust
use lipgloss::{ThemeContext, ThemePreset};
use std::sync::Arc;

struct App {
    theme_ctx: Arc<ThemeContext>,
}

impl App {
    fn new() -> Self {
        let theme_ctx = Arc::new(
            ThemeContext::from_preset(ThemePreset::Dark)
        );
        Self { theme_ctx }
    }

    fn switch_to_light(&self) {
        self.theme_ctx.set_preset(ThemePreset::Light);
    }
}
```

## Step 4: Create Themed Styles

`ThemedStyle` automatically resolves colors from the current theme:

```rust
use lipgloss::{ThemedStyle, ColorSlot};
use std::sync::Arc;

struct AppStyles {
    title: ThemedStyle,
    body: ThemedStyle,
    error: ThemedStyle,
    success: ThemedStyle,
}

impl AppStyles {
    fn new(ctx: Arc<ThemeContext>) -> Self {
        Self {
            title: ThemedStyle::new(ctx.clone())
                .foreground(ColorSlot::Primary)
                .bold(),
            body: ThemedStyle::new(ctx.clone())
                .foreground(ColorSlot::Foreground),
            error: ThemedStyle::new(ctx.clone())
                .foreground(ColorSlot::Error)
                .bold(),
            success: ThemedStyle::new(ctx)
                .foreground(ColorSlot::Success),
        }
    }
}
```

When the theme changes, `ThemedStyle` automatically uses the new colors on the next render.

## Step 5: Implement Theme Switching

```rust
impl App {
    fn cycle_theme(&self) {
        let presets = [
            ThemePreset::Dark,
            ThemePreset::Light,
            ThemePreset::Dracula,
            ThemePreset::Nord,
        ];

        // Simple cycling through themes
        static CURRENT: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);

        let next = CURRENT.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        self.theme_ctx.set_preset(presets[next % presets.len()]);
    }
}
```

## Step 6: Listen for Theme Changes

Register callbacks to respond to theme changes:

```rust
use lipgloss::ThemeContext;

fn setup_theme_listener(ctx: &ThemeContext) {
    let listener_id = ctx.on_change(|theme| {
        eprintln!("Theme changed to: {}", theme.name());
        // Trigger re-render, update UI, etc.
    });

    // Later, remove the listener if needed
    // ctx.remove_listener(listener_id);
}
```

## Step 7: Use the Global Theme Context

For simpler apps, use the global context:

```rust
use lipgloss::{set_global_preset, ThemedStyle, ColorSlot, ThemePreset};

fn main() {
    // Set the global theme
    set_global_preset(ThemePreset::Nord);

    // Create styles that use the global context
    let style = ThemedStyle::global()
        .foreground(ColorSlot::Primary)
        .bold();

    println!("{}", style.render("Hello, Nord!"));
}
```

## Step 8: Load Custom Themes

Load user-defined themes from files:

```rust
use lipgloss::Theme;
use std::path::PathBuf;

fn load_user_theme() -> Option<Theme> {
    let config_path = dirs::config_dir()?
        .join("myapp")
        .join("theme.toml");

    if config_path.exists() {
        Theme::from_file(&config_path).ok()
    } else {
        None
    }
}

fn apply_user_theme(ctx: &ThemeContext) {
    if let Some(theme) = load_user_theme() {
        ctx.set_theme(theme);
    }
}
```

## Step 9: Validate Theme Accessibility

Check contrast ratios for readability:

```rust
use lipgloss::{Theme, ColorSlot};

fn validate_theme(theme: &Theme) -> bool {
    // WCAG AA requires 4.5:1 contrast ratio for normal text
    if !theme.check_contrast_aa(ColorSlot::Foreground, ColorSlot::Background) {
        eprintln!("Warning: Poor contrast for main text");
        return false;
    }

    // Check other important combinations
    if !theme.check_contrast_aa(ColorSlot::Error, ColorSlot::Background) {
        eprintln!("Warning: Error text may be hard to read");
    }

    true
}
```

## Complete Example

Here's a full example combining all concepts:

```rust
use lipgloss::{
    ThemeContext, ThemePreset, ThemedStyle, ColorSlot, Theme, Border,
};
use std::sync::Arc;

struct App {
    ctx: Arc<ThemeContext>,
    styles: Styles,
}

struct Styles {
    header: ThemedStyle,
    menu_item: ThemedStyle,
    menu_selected: ThemedStyle,
    status_ok: ThemedStyle,
    status_error: ThemedStyle,
    border_box: ThemedStyle,
}

impl App {
    fn new() -> Self {
        let ctx = Arc::new(ThemeContext::from_preset(ThemePreset::Dark));
        let styles = Styles::new(ctx.clone());
        Self { ctx, styles }
    }

    fn render(&self, selected: usize) -> String {
        let items = ["Home", "Settings", "Help", "Quit"];

        let menu: String = items.iter().enumerate()
            .map(|(i, item)| {
                if i == selected {
                    self.styles.menu_selected.render(&format!("> {}", item))
                } else {
                    self.styles.menu_item.render(&format!("  {}", item))
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        let header = self.styles.header.render("My Terminal App");
        let status = self.styles.status_ok.render("Ready");

        format!("{}\n\n{}\n\n{}", header, menu, status)
    }

    fn set_theme(&self, preset: ThemePreset) {
        self.ctx.set_preset(preset);
    }
}

impl Styles {
    fn new(ctx: Arc<ThemeContext>) -> Self {
        Self {
            header: ThemedStyle::new(ctx.clone())
                .foreground(ColorSlot::Primary)
                .bold(),
            menu_item: ThemedStyle::new(ctx.clone())
                .foreground(ColorSlot::Muted),
            menu_selected: ThemedStyle::new(ctx.clone())
                .foreground(ColorSlot::Primary)
                .background(ColorSlot::Surface)
                .bold(),
            status_ok: ThemedStyle::new(ctx.clone())
                .foreground(ColorSlot::Success),
            status_error: ThemedStyle::new(ctx.clone())
                .foreground(ColorSlot::Error)
                .bold(),
            border_box: ThemedStyle::new(ctx)
                .border(Border::rounded())
                .border_foreground(ColorSlot::Border)
                .padding((1, 2)),
        }
    }
}

fn main() {
    let app = App::new();

    // Render with dark theme
    println!("{}", app.render(0));

    // Switch to light theme
    app.set_theme(ThemePreset::Light);
    println!("\n--- Light Theme ---\n");
    println!("{}", app.render(0));
}
```

## Next Steps

- [Custom Themes Guide](custom-themes.md) - Create your own theme files
- [Best Practices](theming-best-practices.md) - Tips for accessible, performant themes
- [API Reference](LIPGLOSS.md) - Complete theme API documentation
