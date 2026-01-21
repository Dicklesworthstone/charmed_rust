//! Theme Switcher Example
//!
//! Demonstrates runtime theme switching with ThemedStyle and ThemeContext.
//! ThemedStyles automatically use the new colors when the theme changes.
//!
//! Run with: `cargo run -p example-theme-switcher`

#![forbid(unsafe_code)]

use std::sync::Arc;

use bubbletea::{Cmd, KeyMsg, KeyType, Message, Program, quit};
use lipgloss::{
    Border, CatppuccinFlavor, ColorSlot, ThemeContext, ThemePreset, ThemedStyle,
};

/// Available themes for cycling
const THEME_PRESETS: &[(&str, ThemePreset)] = &[
    ("Dark", ThemePreset::Dark),
    ("Light", ThemePreset::Light),
    ("Dracula", ThemePreset::Dracula),
    ("Nord", ThemePreset::Nord),
    ("Catppuccin Mocha", ThemePreset::Catppuccin(CatppuccinFlavor::Mocha)),
    ("Catppuccin Latte", ThemePreset::Catppuccin(CatppuccinFlavor::Latte)),
];

/// Application styles that auto-update when theme changes
struct Styles {
    title: ThemedStyle,
    theme_name: ThemedStyle,
    sample_primary: ThemedStyle,
    sample_success: ThemedStyle,
    sample_warning: ThemedStyle,
    sample_error: ThemedStyle,
    help: ThemedStyle,
    box_style: ThemedStyle,
}

impl Styles {
    fn new(ctx: Arc<ThemeContext>) -> Self {
        Self {
            title: ThemedStyle::new(ctx.clone())
                .foreground(ColorSlot::Primary)
                .bold(),
            theme_name: ThemedStyle::new(ctx.clone())
                .foreground(ColorSlot::Secondary),
            sample_primary: ThemedStyle::new(ctx.clone())
                .foreground(ColorSlot::Primary),
            sample_success: ThemedStyle::new(ctx.clone())
                .foreground(ColorSlot::Success),
            sample_warning: ThemedStyle::new(ctx.clone())
                .foreground(ColorSlot::Warning),
            sample_error: ThemedStyle::new(ctx.clone())
                .foreground(ColorSlot::Error),
            help: ThemedStyle::new(ctx.clone())
                .foreground(ColorSlot::TextMuted)
                .italic(),
            box_style: ThemedStyle::new(ctx)
                .border(Border::rounded())
                .border_foreground(ColorSlot::Border)
                .padding((1, 2)),
        }
    }
}

/// The application model
#[derive(bubbletea::Model)]
struct App {
    ctx: Arc<ThemeContext>,
    styles: Styles,
    current_theme: usize,
}

impl App {
    fn new() -> Self {
        let ctx = Arc::new(ThemeContext::from_preset(ThemePreset::Dark));
        let styles = Styles::new(ctx.clone());
        Self {
            ctx,
            styles,
            current_theme: 0,
        }
    }

    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::Runes => {
                    if let Some(&ch) = key.runes.first() {
                        match ch {
                            'n' | 'N' | ' ' => self.next_theme(),
                            'p' | 'P' => self.prev_theme(),
                            'q' | 'Q' => return Some(quit()),
                            _ => {}
                        }
                    }
                }
                KeyType::Right | KeyType::Tab => self.next_theme(),
                KeyType::Left | KeyType::ShiftTab => self.prev_theme(),
                KeyType::CtrlC | KeyType::Esc => return Some(quit()),
                _ => {}
            }
        }
        None
    }

    fn next_theme(&mut self) {
        self.current_theme = (self.current_theme + 1) % THEME_PRESETS.len();
        self.apply_theme();
    }

    fn prev_theme(&mut self) {
        self.current_theme = (self.current_theme + THEME_PRESETS.len() - 1) % THEME_PRESETS.len();
        self.apply_theme();
    }

    fn apply_theme(&mut self) {
        let (_, preset) = THEME_PRESETS[self.current_theme];
        self.ctx.set_preset(preset);
    }

    fn view(&self) -> String {
        let (theme_name, _) = THEME_PRESETS[self.current_theme];

        // Build content - styles automatically use current theme colors
        let title = self.styles.title.render("Theme Switcher Demo");
        let current = self
            .styles
            .theme_name
            .render(&format!("Current: {}", theme_name));

        let samples = format!(
            "{}\n{}\n{}\n{}",
            self.styles.sample_primary.render("Primary: The quick brown fox"),
            self.styles.sample_success.render("Success: Operation completed"),
            self.styles.sample_warning.render("Warning: Disk space low"),
            self.styles.sample_error.render("Error: Connection failed"),
        );

        let help = self
            .styles
            .help
            .render("[n/space/→] next | [p/←] prev | [q] quit");

        // Combine into a box
        let content = format!("{}\n{}\n\n{}\n\n{}", title, current, samples, help);
        let boxed = self.styles.box_style.render(&content);

        format!("\n{}\n", boxed)
    }
}

fn main() -> anyhow::Result<()> {
    let _final = Program::new(App::new()).with_alt_screen().run()?;
    Ok(())
}
