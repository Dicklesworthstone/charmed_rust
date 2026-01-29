//! Settings page - application preferences and toggles.
//!
//! This page exposes runtime toggles for:
//! - Mouse input on/off
//! - Animations on/off
//! - Force ASCII mode (no colors, ASCII borders)
//! - Syntax highlighting on/off
//!
//! It also provides a theme picker with live previews for instant theme switching.
//!
//! Changes take effect immediately without restart.

use bubbletea::{Cmd, KeyMsg, KeyType, Message, batch};
use lipgloss::Style;

use super::PageModel;
use crate::messages::{AppMsg, Notification, NotificationMsg, Page};
use crate::theme::{Theme, ThemePreset};

/// Settings toggle item.
#[derive(Debug, Clone, Copy)]
struct Toggle {
    /// Display label for the toggle.
    label: &'static str,
    /// Description of what the toggle does.
    description: &'static str,
    /// Keyboard shortcut (displayed in hints).
    key: char,
}

const TOGGLES: [Toggle; 4] = [
    Toggle {
        label: "Mouse Input",
        description: "Enable mouse clicks and scrolling",
        key: 'm',
    },
    Toggle {
        label: "Animations",
        description: "Enable smooth transitions and spinners",
        key: 'a',
    },
    Toggle {
        label: "ASCII Mode",
        description: "Use ASCII-only characters (no colors)",
        key: 'c',
    },
    Toggle {
        label: "Syntax Highlighting",
        description: "Highlight code in previews",
        key: 's',
    },
];

/// Which section of the Settings page is focused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsSection {
    /// Toggles section (mouse, animations, etc.)
    #[default]
    Toggles,
    /// Theme picker section.
    Themes,
    /// Keybindings reference section (bd-3b7o).
    Keybindings,
}

/// A keybinding entry for display.
#[derive(Debug, Clone, Copy)]
struct KeybindingEntry {
    /// The key or key combination.
    key: &'static str,
    /// Description of what the key does.
    action: &'static str,
}

/// Global keybindings that work across all pages.
const GLOBAL_KEYS: [KeybindingEntry; 12] = [
    KeybindingEntry {
        key: "?",
        action: "Toggle help overlay",
    },
    KeybindingEntry {
        key: "q",
        action: "Quit application",
    },
    KeybindingEntry {
        key: "Esc",
        action: "Close modal/cancel",
    },
    KeybindingEntry {
        key: "1-8",
        action: "Navigate to page",
    },
    KeybindingEntry {
        key: "Tab",
        action: "Cycle focus/section",
    },
    KeybindingEntry {
        key: "j / ↓",
        action: "Move down",
    },
    KeybindingEntry {
        key: "k / ↑",
        action: "Move up",
    },
    KeybindingEntry {
        key: "g",
        action: "Go to top",
    },
    KeybindingEntry {
        key: "G",
        action: "Go to bottom",
    },
    KeybindingEntry {
        key: "Enter",
        action: "Confirm/activate",
    },
    KeybindingEntry {
        key: "Ctrl+C",
        action: "Copy to clipboard",
    },
    KeybindingEntry {
        key: "/",
        action: "Search (in page)",
    },
];

/// Page-specific keybindings.
const PAGE_KEYS: [KeybindingEntry; 12] = [
    // Dashboard
    KeybindingEntry {
        key: "r",
        action: "Dashboard: Refresh data",
    },
    KeybindingEntry {
        key: "Enter",
        action: "Dashboard: Open details",
    },
    // Jobs
    KeybindingEntry {
        key: "n",
        action: "Jobs: Create new job",
    },
    KeybindingEntry {
        key: "x",
        action: "Jobs: Cancel selected",
    },
    KeybindingEntry {
        key: "Enter",
        action: "Jobs: View details",
    },
    // Logs
    KeybindingEntry {
        key: "f",
        action: "Logs: Toggle follow",
    },
    KeybindingEntry {
        key: "e",
        action: "Logs: Export to file",
    },
    KeybindingEntry {
        key: "c",
        action: "Logs: Clear logs",
    },
    // Docs
    KeybindingEntry {
        key: "n/N",
        action: "Docs: Next/prev match",
    },
    KeybindingEntry {
        key: "s",
        action: "Docs: Toggle syntax",
    },
    // Files
    KeybindingEntry {
        key: "Enter",
        action: "Files: Open preview",
    },
    KeybindingEntry {
        key: "Backspace",
        action: "Files: Go up directory",
    },
];

/// Settings page showing application preferences.
pub struct SettingsPage {
    /// Current focused section.
    section: SettingsSection,
    /// Currently selected toggle index.
    toggle_selected: usize,
    /// Current toggle states (synced from App on enter).
    toggle_states: [bool; 4],
    /// Currently selected theme index.
    theme_selected: usize,
    /// Current active theme preset.
    current_theme: ThemePreset,
}

impl SettingsPage {
    /// Create a new settings page.
    #[must_use]
    pub fn new() -> Self {
        Self {
            section: SettingsSection::Toggles,
            toggle_selected: 0,
            toggle_states: [false, true, false, true], // Default: mouse off, anim on, ascii off, syntax on
            theme_selected: 0,
            current_theme: ThemePreset::Dark,
        }
    }

    /// Update toggle states from app state.
    ///
    /// Called on page enter to sync with current app configuration.
    pub fn sync_states(
        &mut self,
        mouse: bool,
        animations: bool,
        force_ascii: bool,
        syntax: bool,
        current_theme: ThemePreset,
    ) {
        self.toggle_states = [mouse, animations, force_ascii, syntax];
        self.current_theme = current_theme;
        // Find the index of the current theme
        let presets = ThemePreset::all();
        self.theme_selected = presets
            .iter()
            .position(|&p| p == current_theme)
            .unwrap_or(0);
    }

    /// Switch to the next section.
    fn next_section(&mut self) {
        self.section = match self.section {
            SettingsSection::Toggles => SettingsSection::Themes,
            SettingsSection::Themes => SettingsSection::Keybindings,
            SettingsSection::Keybindings => SettingsSection::Toggles,
        };
    }

    /// Move selection up within current section.
    fn move_up(&mut self) {
        match self.section {
            SettingsSection::Toggles => {
                if self.toggle_selected > 0 {
                    self.toggle_selected -= 1;
                }
            }
            SettingsSection::Themes => {
                if self.theme_selected > 0 {
                    self.theme_selected -= 1;
                }
            }
            SettingsSection::Keybindings => {
                // Keybindings section is read-only reference; no selection to move
            }
        }
    }

    /// Move selection down within current section.
    fn move_down(&mut self) {
        match self.section {
            SettingsSection::Toggles => {
                if self.toggle_selected < TOGGLES.len() - 1 {
                    self.toggle_selected += 1;
                }
            }
            SettingsSection::Themes => {
                let presets = ThemePreset::all();
                if self.theme_selected < presets.len() - 1 {
                    self.theme_selected += 1;
                }
            }
            SettingsSection::Keybindings => {
                // Keybindings section is read-only reference; no selection to move
            }
        }
    }

    /// Activate the currently selected item (toggle or apply theme).
    fn activate_selected(&mut self) -> Option<Cmd> {
        match self.section {
            SettingsSection::Toggles => self.toggle_selected_toggle(),
            SettingsSection::Themes => self.apply_selected_theme(),
            SettingsSection::Keybindings => None, // Read-only reference section
        }
    }

    /// Toggle the currently selected toggle item.
    fn toggle_selected_toggle(&mut self) -> Option<Cmd> {
        self.toggle_states[self.toggle_selected] = !self.toggle_states[self.toggle_selected];
        let state = self.toggle_states[self.toggle_selected];
        let idx = self.toggle_selected;
        Some(Cmd::new(move || match idx {
            0 => AppMsg::ToggleMouse.into_message(),
            1 => AppMsg::ToggleAnimations.into_message(),
            2 => AppMsg::ForceAscii(state).into_message(),
            3 => AppMsg::ToggleSyntax.into_message(),
            _ => AppMsg::ToggleMouse.into_message(), // Fallback, shouldn't happen
        }))
    }

    /// Apply the currently selected theme.
    fn apply_selected_theme(&mut self) -> Option<Cmd> {
        let presets = ThemePreset::all();
        let selected_preset = presets[self.theme_selected];

        // Don't do anything if already the current theme
        if selected_preset == self.current_theme {
            return None;
        }

        self.current_theme = selected_preset;
        let theme_name = selected_preset.name().to_string();

        // Return batch: set theme + show notification
        batch(vec![
            Some(Cmd::new(move || {
                AppMsg::SetTheme(selected_preset).into_message()
            })),
            Some(Cmd::new(move || {
                NotificationMsg::Show(Notification::success(
                    0, // App will assign actual ID
                    format!("Theme changed to {theme_name}"),
                ))
                .into_message()
            })),
        ])
    }

    /// Handle a specific toggle key.
    fn handle_toggle_key(&mut self, key: char) -> Option<Cmd> {
        for (i, toggle) in TOGGLES.iter().enumerate() {
            if toggle.key == key {
                self.toggle_selected = i;
                self.section = SettingsSection::Toggles;
                return self.toggle_selected_toggle();
            }
        }
        None
    }

    /// Render a single toggle row.
    fn render_toggle(&self, index: usize, width: usize, theme: &Theme) -> String {
        let toggle = &TOGGLES[index];
        let section_focused = self.section == SettingsSection::Toggles;
        let is_selected = section_focused && index == self.toggle_selected;
        let is_on = self.toggle_states[index];

        let cursor = if is_selected { ">" } else { " " };
        let cursor_style = if is_selected {
            theme.info_style()
        } else {
            theme.muted_style()
        };

        // Toggle indicator
        let indicator = if is_on { "[x]" } else { "[ ]" };
        let indicator_style = if is_on {
            theme.success_style()
        } else {
            theme.muted_style()
        };

        // Label
        let label_style = if is_selected {
            theme.title_style()
        } else {
            Style::new()
        };

        // Key hint
        let key_hint = format!("({})", toggle.key);

        // Build the line
        let label_part = format!(
            "{} {} {} {}",
            cursor_style.render(cursor),
            indicator_style.render(indicator),
            label_style.render(toggle.label),
            theme.muted_style().render(&key_hint),
        );

        // Description on same line if space, otherwise truncate
        let desc_width = width.saturating_sub(40);
        let description = if desc_width > 10 {
            let truncated: String = toggle.description.chars().take(desc_width).collect();
            theme.muted_style().italic().render(&truncated)
        } else {
            String::new()
        };

        format!("{}  {}", label_part, description)
    }

    /// Render a theme preview row.
    fn render_theme_row(
        &self,
        preset: ThemePreset,
        index: usize,
        width: usize,
        theme: &Theme,
    ) -> String {
        let section_focused = self.section == SettingsSection::Themes;
        let is_selected = section_focused && index == self.theme_selected;
        let is_current = preset == self.current_theme;

        // Get the preview theme to show its colors
        let preview_theme = Theme::from_preset(preset);

        let cursor = if is_selected { ">" } else { " " };
        let cursor_style = if is_selected {
            theme.info_style()
        } else {
            theme.muted_style()
        };

        // Current theme indicator
        let current_indicator = if is_current { "●" } else { "○" };
        let current_style = if is_current {
            theme.success_style()
        } else {
            theme.muted_style()
        };

        // Theme name
        let name_style = if is_selected {
            theme.title_style()
        } else if is_current {
            Style::new().bold()
        } else {
            Style::new()
        };

        // Build preview swatches using the preview theme's colors
        let preview = Self::render_theme_preview(&preview_theme, width.saturating_sub(30));

        format!(
            "{} {} {}  {}",
            cursor_style.render(cursor),
            current_style.render(current_indicator),
            name_style.render(preset.name()),
            preview
        )
    }

    /// Render a compact preview of theme colors.
    fn render_theme_preview(preview_theme: &Theme, _max_width: usize) -> String {
        // Create small sample swatches showing the theme's key colors
        let primary = Style::new()
            .foreground(preview_theme.text_inverse)
            .background(preview_theme.primary)
            .render(" Pri ");

        let success = Style::new()
            .foreground(preview_theme.text_inverse)
            .background(preview_theme.success)
            .render(" Ok ");

        let warning = Style::new()
            .foreground(preview_theme.text_inverse)
            .background(preview_theme.warning)
            .render(" !! ");

        let error = Style::new()
            .foreground(preview_theme.text_inverse)
            .background(preview_theme.error)
            .render(" Err ");

        let info = Style::new()
            .foreground(preview_theme.text_inverse)
            .background(preview_theme.info)
            .render(" Inf ");

        format!("{primary}{success}{warning}{error}{info}")
    }
}

impl Default for SettingsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl PageModel for SettingsPage {
    fn update(&mut self, msg: &Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::Tab => {
                    self.next_section();
                    return None;
                }
                KeyType::Up => self.move_up(),
                KeyType::Down => self.move_down(),
                KeyType::Enter => return self.activate_selected(),
                KeyType::Runes => match key.runes.as_slice() {
                    ['j'] => self.move_down(),
                    ['k'] => self.move_up(),
                    [' '] => return self.activate_selected(),
                    // Direct toggle shortcuts only work for toggles
                    [c @ ('m' | 'a' | 'c' | 's')] => return self.handle_toggle_key(*c),
                    _ => {}
                },
                _ => {}
            }
        }
        None
    }

    fn view(&self, width: usize, height: usize, theme: &Theme) -> String {
        let mut lines = Vec::new();

        // Header
        lines.push(theme.heading_style().render("Settings"));
        lines.push(theme.muted_style().render(&"─".repeat(width.min(60))));
        lines.push(String::new());

        // Section: Toggles
        let toggles_focused = self.section == SettingsSection::Toggles;
        let toggles_header = if toggles_focused {
            theme.title_style().render("▸ Toggles")
        } else {
            theme.muted_style().render("  Toggles")
        };
        lines.push(toggles_header);
        lines.push(String::new());

        for i in 0..TOGGLES.len() {
            lines.push(self.render_toggle(i, width, theme));
        }

        lines.push(String::new());

        // Section: Theme Picker
        let themes_focused = self.section == SettingsSection::Themes;
        let themes_header = if themes_focused {
            theme.title_style().render("▸ Theme")
        } else {
            theme.muted_style().render("  Theme")
        };
        lines.push(themes_header);
        lines.push(String::new());

        // Render theme options with previews
        for (i, preset) in ThemePreset::all().iter().enumerate() {
            lines.push(self.render_theme_row(*preset, i, width, theme));
        }

        lines.push(String::new());
        lines.push(theme.muted_style().render(&"─".repeat(width.min(60))));

        // Status summary: current theme + toggles
        let theme_status = format!("Theme: {}", self.current_theme.name());
        let toggle_status: Vec<String> = TOGGLES
            .iter()
            .zip(self.toggle_states.iter())
            .map(|(t, &on)| {
                let indicator = if on { "●" } else { "○" };
                format!("{} {}", indicator, t.label)
            })
            .collect();
        lines.push(theme.muted_style().render(&format!(
            "{}  |  {}",
            theme_status,
            toggle_status.join("  ")
        )));

        // Pad to height
        while lines.len() < height {
            lines.push(String::new());
        }

        lines.join("\n")
    }

    fn page(&self) -> Page {
        Page::Settings
    }

    fn hints(&self) -> &'static str {
        "Tab section  j/k nav  Enter apply  m/a/c/s toggles"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_page_creates() {
        let page = SettingsPage::new();
        assert_eq!(page.toggle_selected, 0);
        assert_eq!(page.theme_selected, 0);
        assert_eq!(page.section, SettingsSection::Toggles);
    }

    #[test]
    fn settings_page_toggle_navigation() {
        let mut page = SettingsPage::new();
        page.section = SettingsSection::Toggles;

        page.move_down();
        assert_eq!(page.toggle_selected, 1);

        page.move_up();
        assert_eq!(page.toggle_selected, 0);

        // Can't go above 0
        page.move_up();
        assert_eq!(page.toggle_selected, 0);
    }

    #[test]
    fn settings_page_theme_navigation() {
        let mut page = SettingsPage::new();
        page.section = SettingsSection::Themes;

        page.move_down();
        assert_eq!(page.theme_selected, 1);

        page.move_up();
        assert_eq!(page.theme_selected, 0);

        // Can't go above 0
        page.move_up();
        assert_eq!(page.theme_selected, 0);
    }

    #[test]
    fn settings_page_section_toggle() {
        let mut page = SettingsPage::new();
        assert_eq!(page.section, SettingsSection::Toggles);

        page.next_section();
        assert_eq!(page.section, SettingsSection::Themes);

        page.next_section();
        assert_eq!(page.section, SettingsSection::Toggles);
    }

    #[test]
    fn settings_page_sync_states() {
        let mut page = SettingsPage::new();
        page.sync_states(true, false, true, false, ThemePreset::Dracula);
        assert_eq!(page.toggle_states, [true, false, true, false]);
        assert_eq!(page.current_theme, ThemePreset::Dracula);
        assert_eq!(page.theme_selected, 2); // Dracula is at index 2
    }

    #[test]
    fn settings_page_toggle_item() {
        let mut page = SettingsPage::new();
        page.section = SettingsSection::Toggles;
        let initial = page.toggle_states[0];

        let _cmd = page.toggle_selected_toggle();
        assert_eq!(page.toggle_states[0], !initial);
    }

    #[test]
    fn settings_page_apply_theme() {
        let mut page = SettingsPage::new();
        page.section = SettingsSection::Themes;
        page.theme_selected = 1; // Light
        page.current_theme = ThemePreset::Dark;

        let cmd = page.apply_selected_theme();
        assert!(cmd.is_some());
        assert_eq!(page.current_theme, ThemePreset::Light);
    }

    #[test]
    fn settings_page_apply_same_theme_is_noop() {
        let mut page = SettingsPage::new();
        page.section = SettingsSection::Themes;
        page.theme_selected = 0; // Dark
        page.current_theme = ThemePreset::Dark;

        let cmd = page.apply_selected_theme();
        assert!(cmd.is_none()); // No command when already on same theme
    }

    #[test]
    fn settings_page_hints() {
        let page = SettingsPage::new();
        let hints = page.hints();
        assert!(hints.contains("Tab"));
        assert!(hints.contains("j/k"));
    }

    #[test]
    fn settings_page_render_theme_preview() {
        let theme = Theme::dark();
        let preview = SettingsPage::render_theme_preview(&theme, 50);
        // Preview should contain styled text (non-empty)
        assert!(!preview.is_empty());
    }

    #[test]
    fn settings_page_view_contains_sections() {
        let page = SettingsPage::new();
        let theme = Theme::dark();
        let view = page.view(80, 24, &theme);

        // Should have both sections
        assert!(view.contains("Toggles"));
        assert!(view.contains("Theme"));
    }
}
