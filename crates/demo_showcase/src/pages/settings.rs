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
//! The About/Diagnostics section (bd-2kp1) provides:
//! - Version + build info for all charmed_rust crates
//! - Active runtime configuration details
//! - Terminal environment info
//! - Feature flag status
//! - "Copy diagnostics" and "Open in pager" actions
//!
//! Changes take effect immediately without restart.

use std::env;

use bubbletea::{Cmd, KeyMsg, KeyType, Message, batch};
use lipgloss::Style;

use super::PageModel;
use crate::config::Config;
use crate::messages::{AppMsg, Notification, NotificationMsg, Page, ShellOutMsg};
use crate::shell_action::open_in_pager;
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
    /// About + Diagnostics section (bd-2kp1).
    About,
}

/// Action items in the About section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AboutAction {
    /// Copy diagnostics to file.
    #[default]
    CopyDiagnostics,
    /// Open diagnostics in external pager.
    OpenInPager,
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
    /// Currently selected action in About section.
    about_action: AboutAction,
    /// Cached runtime config (synced on page enter).
    runtime_config: Option<Config>,
    /// Whether running in headless mode.
    is_headless: bool,
    /// Last known terminal width.
    terminal_width: usize,
    /// Last known terminal height.
    terminal_height: usize,
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
            about_action: AboutAction::CopyDiagnostics,
            runtime_config: None,
            is_headless: false,
            terminal_width: 80,
            terminal_height: 24,
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

    /// Sync runtime configuration for diagnostics display.
    ///
    /// Called on page enter to capture current runtime state.
    pub fn sync_runtime_config(&mut self, config: Config, is_headless: bool) {
        self.runtime_config = Some(config);
        self.is_headless = is_headless;
    }

    /// Update terminal dimensions (called on resize).
    pub fn update_terminal_size(&mut self, width: usize, height: usize) {
        self.terminal_width = width;
        self.terminal_height = height;
    }

    /// Switch to the next section.
    fn next_section(&mut self) {
        self.section = match self.section {
            SettingsSection::Toggles => SettingsSection::Themes,
            SettingsSection::Themes => SettingsSection::Keybindings,
            SettingsSection::Keybindings => SettingsSection::About,
            SettingsSection::About => SettingsSection::Toggles,
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
            SettingsSection::About => {
                // Toggle between the two actions
                self.about_action = match self.about_action {
                    AboutAction::CopyDiagnostics => AboutAction::OpenInPager,
                    AboutAction::OpenInPager => AboutAction::CopyDiagnostics,
                };
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
            SettingsSection::About => {
                // Toggle between the two actions
                self.about_action = match self.about_action {
                    AboutAction::CopyDiagnostics => AboutAction::OpenInPager,
                    AboutAction::OpenInPager => AboutAction::CopyDiagnostics,
                };
            }
        }
    }

    /// Activate the currently selected item (toggle or apply theme).
    fn activate_selected(&mut self) -> Option<Cmd> {
        match self.section {
            SettingsSection::Toggles => self.toggle_selected_toggle(),
            SettingsSection::Themes => self.apply_selected_theme(),
            SettingsSection::Keybindings => None, // Read-only reference section
            SettingsSection::About => self.execute_about_action(),
        }
    }

    /// Execute the currently selected About action.
    fn execute_about_action(&self) -> Option<Cmd> {
        let diagnostics = self.generate_full_diagnostics();

        match self.about_action {
            AboutAction::CopyDiagnostics => {
                // Export to file (similar to logs export)
                Some(Cmd::new(move || {
                    let export_dir = std::env::current_dir()
                        .unwrap_or_else(|_| std::path::PathBuf::from("."))
                        .join("demo_showcase_exports");

                    if let Err(e) = std::fs::create_dir_all(&export_dir) {
                        return NotificationMsg::Show(Notification::error(
                            0,
                            format!("Failed to create export dir: {e}"),
                        ))
                        .into_message();
                    }

                    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                    let filename = format!("diagnostics_{timestamp}.txt");
                    let filepath = export_dir.join(&filename);

                    match std::fs::write(&filepath, &diagnostics) {
                        Ok(()) => NotificationMsg::Show(Notification::success(
                            0,
                            format!("Diagnostics exported to {}", filepath.display()),
                        ))
                        .into_message(),
                        Err(e) => NotificationMsg::Show(Notification::error(
                            0,
                            format!("Export failed: {e}"),
                        ))
                        .into_message(),
                    }
                }))
            }
            AboutAction::OpenInPager => {
                // Open in pager (shell-out flow)
                if self.is_headless {
                    // In headless mode, show notification instead
                    Some(Cmd::new(|| {
                        NotificationMsg::Show(Notification::info(
                            0,
                            "Pager unavailable in headless mode".to_string(),
                        ))
                        .into_message()
                    }))
                } else {
                    open_in_pager(diagnostics, self.is_headless).or_else(|| {
                        Some(Cmd::new(|| {
                            ShellOutMsg::PagerCompleted(None).into_message()
                        }))
                    })
                }
            }
        }
    }

    /// Generate full diagnostics string including all sections.
    fn generate_full_diagnostics(&self) -> String {
        let mut lines = Vec::new();

        // Header
        lines.push("═".repeat(60));
        lines.push("  CHARMED_RUST DEMO SHOWCASE - DIAGNOSTICS REPORT".to_string());
        lines.push("═".repeat(60));
        lines.push(String::new());

        // Version info (all crates use workspace version)
        let workspace_version = env!("CARGO_PKG_VERSION");
        lines.push("─── VERSION INFO ───".to_string());
        lines.push(format!("charmed_rust workspace: {workspace_version}"));
        lines.push(String::new());
        lines.push("Included crates:".to_string());
        lines.push(format!("  demo_showcase     {workspace_version}"));
        lines.push(format!("  bubbletea         {workspace_version}"));
        lines.push(format!("  lipgloss          {workspace_version}"));
        lines.push(format!("  bubbles           {workspace_version}"));
        lines.push(format!("  glamour           {workspace_version}"));
        lines.push(format!("  harmonica         {workspace_version}"));
        lines.push(format!("  huh               {workspace_version}"));
        lines.push(format!("  charmed_log       {workspace_version}"));
        #[cfg(feature = "ssh")]
        lines.push(format!("  wish              {workspace_version}"));
        lines.push(String::new());

        // Terminal info
        lines.push("─── TERMINAL INFO ───".to_string());
        lines.push(format!(
            "TERM:           {}",
            env::var("TERM").unwrap_or_else(|_| "(not set)".to_string())
        ));
        lines.push(format!(
            "COLORTERM:      {}",
            env::var("COLORTERM").unwrap_or_else(|_| "(not set)".to_string())
        ));
        lines.push(format!(
            "Dimensions:     {}x{} (cols x rows)",
            self.terminal_width, self.terminal_height
        ));
        lines.push(format!(
            "NO_COLOR:       {}",
            if env::var("NO_COLOR").is_ok() {
                "set"
            } else {
                "not set"
            }
        ));
        lines.push(format!(
            "REDUCE_MOTION:  {}",
            if env::var("REDUCE_MOTION").is_ok() {
                "set"
            } else {
                "not set"
            }
        ));
        lines.push(String::new());

        // Feature flags
        lines.push("─── FEATURE FLAGS ───".to_string());
        lines.push(format!(
            "syntax-highlighting: {}",
            if cfg!(feature = "syntax-highlighting") {
                "enabled"
            } else {
                "disabled"
            }
        ));
        lines.push(format!(
            "ssh:                 {}",
            if cfg!(feature = "ssh") {
                "enabled"
            } else {
                "disabled"
            }
        ));
        lines.push(format!(
            "async:               {}",
            if cfg!(feature = "async") {
                "enabled"
            } else {
                "disabled"
            }
        ));
        lines.push(String::new());

        // Runtime config
        lines.push("─── RUNTIME CONFIG ───".to_string());
        if let Some(ref config) = self.runtime_config {
            lines.push(config.to_diagnostic_string());
        } else {
            lines.push("(not available)".to_string());
        }
        lines.push(String::new());

        // Current toggles
        lines.push("─── CURRENT TOGGLES ───".to_string());
        lines.push(format!(
            "Mouse:          {}",
            if self.toggle_states[0] { "on" } else { "off" }
        ));
        lines.push(format!(
            "Animations:     {}",
            if self.toggle_states[1] { "on" } else { "off" }
        ));
        lines.push(format!(
            "ASCII Mode:     {}",
            if self.toggle_states[2] { "on" } else { "off" }
        ));
        lines.push(format!(
            "Syntax HL:      {}",
            if self.toggle_states[3] { "on" } else { "off" }
        ));
        lines.push(format!("Theme:          {}", self.current_theme.name()));
        lines.push(String::new());

        // Footer
        lines.push("═".repeat(60));
        lines.push(format!(
            "  Generated: {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));
        lines.push("═".repeat(60));

        lines.join("\n")
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

    /// Render a single keybinding entry row.
    fn render_keybinding_entry(entry: &KeybindingEntry, theme: &Theme) -> String {
        let key_style = theme.info_style().bold();
        let action_style = theme.muted_style();
        let formatted_key = format!("{:>10}", entry.key);

        format!(
            "    {}  {}",
            key_style.render(&formatted_key),
            action_style.render(entry.action)
        )
    }

    /// Render the keybindings reference section.
    fn render_keybindings(&self, width: usize, theme: &Theme) -> Vec<String> {
        let mut lines = Vec::new();

        // Section header
        let keybindings_focused = self.section == SettingsSection::Keybindings;
        let header = if keybindings_focused {
            theme.title_style().render("▸ Keybindings Reference")
        } else {
            theme.muted_style().render("  Keybindings Reference")
        };
        lines.push(header);
        lines.push(String::new());

        // Global keybindings
        lines.push(theme.title_style().render("  Global"));
        for entry in &GLOBAL_KEYS {
            lines.push(Self::render_keybinding_entry(entry, theme));
        }
        lines.push(String::new());

        // Page-specific keybindings
        lines.push(theme.title_style().render("  Page-Specific"));
        for entry in &PAGE_KEYS {
            lines.push(Self::render_keybinding_entry(entry, theme));
        }

        // Note about customization
        lines.push(String::new());
        let note = if width > 60 {
            "  Note: Keybindings are currently read-only. Customization coming soon!"
        } else {
            "  Keybindings are read-only."
        };
        lines.push(theme.muted_style().italic().render(note));

        lines
    }

    /// Render the About/Diagnostics section (bd-2kp1).
    fn render_about(&self, width: usize, theme: &Theme) -> Vec<String> {
        let mut lines = Vec::new();

        // Section header
        let about_focused = self.section == SettingsSection::About;
        let header = if about_focused {
            theme.title_style().render("▸ About + Diagnostics")
        } else {
            theme.muted_style().render("  About + Diagnostics")
        };
        lines.push(header);
        lines.push(String::new());

        // Version info (compact)
        let version = env!("CARGO_PKG_VERSION");
        lines.push(format!(
            "    {}  {}",
            theme.info_style().render("charmed_rust"),
            theme.muted_style().render(&format!("v{version}"))
        ));
        lines.push(String::new());

        // Terminal quick stats
        let term = env::var("TERM").unwrap_or_else(|_| "unknown".to_string());
        let colorterm = env::var("COLORTERM").unwrap_or_else(|_| "-".to_string());
        lines.push(format!(
            "    Terminal: {} ({}) | {}x{}",
            term, colorterm, self.terminal_width, self.terminal_height
        ));

        // Feature flags
        let mut features = Vec::new();
        if cfg!(feature = "syntax-highlighting") {
            features.push("syntax");
        }
        if cfg!(feature = "ssh") {
            features.push("ssh");
        }
        if cfg!(feature = "async") {
            features.push("async");
        }
        let features_str = if features.is_empty() {
            "none".to_string()
        } else {
            features.join(", ")
        };
        lines.push(format!("    Features:  {features_str}"));
        lines.push(String::new());

        // Config summary
        if let Some(ref config) = self.runtime_config {
            lines.push(format!(
                "    Config: theme={:?}, anim={:?}, mouse={}",
                config.theme_preset,
                config.animations,
                if config.mouse { "on" } else { "off" }
            ));
            if let Some(seed) = config.seed {
                lines.push(format!("            seed={seed}"));
            }
        }
        lines.push(String::new());

        // Actions
        lines.push(theme.muted_style().render("    Actions:"));

        let copy_selected = about_focused && self.about_action == AboutAction::CopyDiagnostics;
        let pager_selected = about_focused && self.about_action == AboutAction::OpenInPager;

        let copy_cursor = if copy_selected { ">" } else { " " };
        let copy_style = if copy_selected {
            theme.title_style()
        } else {
            Style::new()
        };

        let pager_cursor = if pager_selected { ">" } else { " " };
        let pager_style = if pager_selected {
            theme.title_style()
        } else {
            Style::new()
        };

        lines.push(format!(
            "      {} {} {}",
            if copy_selected {
                theme.info_style().render(copy_cursor)
            } else {
                theme.muted_style().render(copy_cursor)
            },
            copy_style.render("[d] Copy to file"),
            theme.muted_style().render("(exports diagnostics)")
        ));

        let pager_hint = if self.is_headless {
            "(unavailable in headless)"
        } else {
            "(opens in $PAGER)"
        };
        lines.push(format!(
            "      {} {} {}",
            if pager_selected {
                theme.info_style().render(pager_cursor)
            } else {
                theme.muted_style().render(pager_cursor)
            },
            pager_style.render("[p] Open in pager"),
            theme.muted_style().render(pager_hint)
        ));

        // Separator with total lines for help
        let _ = width; // Suppress unused warning
        lines.push(String::new());
        lines.push(
            theme
                .muted_style()
                .italic()
                .render("    Press Enter to execute, or c/p for shortcuts"),
        );

        lines
    }

    /// Handle About section keyboard shortcuts.
    fn handle_about_shortcut(&mut self, key: char) -> Option<Cmd> {
        match key {
            'c' | 'd' => {
                self.about_action = AboutAction::CopyDiagnostics;
                self.section = SettingsSection::About;
                self.execute_about_action()
            }
            'p' => {
                self.about_action = AboutAction::OpenInPager;
                self.section = SettingsSection::About;
                self.execute_about_action()
            }
            _ => None,
        }
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
                    [c @ ('m' | 'a' | 's')] => return self.handle_toggle_key(*c),
                    // 'c' can be toggle (ASCII) or copy diagnostics
                    ['c'] => {
                        if self.section == SettingsSection::About {
                            return self.handle_about_shortcut('c');
                        }
                        return self.handle_toggle_key('c');
                    }
                    // 'd' is copy diagnostics shortcut (from any section)
                    ['d'] => return self.handle_about_shortcut('c'),
                    // 'p' is pager shortcut (only from About section to avoid conflicts)
                    ['p'] => {
                        if self.section == SettingsSection::About {
                            return self.handle_about_shortcut('p');
                        }
                    }
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

        // Section: Keybindings Reference (bd-3b7o)
        lines.extend(self.render_keybindings(width, theme));

        lines.push(String::new());

        // Section: About + Diagnostics (bd-2kp1)
        lines.extend(self.render_about(width, theme));

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
        match self.section {
            SettingsSection::Toggles => "Tab section  j/k nav  Space/Enter toggle  m/a/c/s direct",
            SettingsSection::Themes => "Tab section  j/k nav  Enter apply theme",
            SettingsSection::Keybindings => "Tab section  (read-only reference)",
            SettingsSection::About => "Tab section  j/k nav  Enter/d copy  p pager",
        }
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
        assert_eq!(page.section, SettingsSection::Keybindings);

        page.next_section();
        assert_eq!(page.section, SettingsSection::About);

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
        let view = page.view(80, 80, &theme);

        // Should have all four sections
        assert!(view.contains("Toggles"));
        assert!(view.contains("Theme"));
        assert!(view.contains("Keybindings Reference"));
        assert!(view.contains("About + Diagnostics"));
    }

    #[test]
    fn settings_page_keybindings_is_readonly() {
        let mut page = SettingsPage::new();
        page.section = SettingsSection::Keybindings;

        // Keybindings section is read-only, so navigation should be no-op
        page.move_down();
        page.move_up();

        // activate_selected should return None for Keybindings
        let cmd = page.activate_selected();
        assert!(cmd.is_none());
    }

    #[test]
    fn settings_page_keybindings_view_contains_keys() {
        let page = SettingsPage::new();
        let theme = Theme::dark();
        let view = page.view(120, 80, &theme);

        // Should contain global keybindings
        assert!(view.contains("Global"));
        assert!(view.contains("Toggle help overlay"));
        assert!(view.contains("Quit application"));

        // Should contain page-specific keybindings
        assert!(view.contains("Page-Specific"));
        assert!(view.contains("Dashboard"));
        assert!(view.contains("Jobs"));
        assert!(view.contains("Logs"));
    }

    #[test]
    fn settings_page_keybindings_focused_style() {
        let mut page = SettingsPage::new();
        let theme = Theme::dark();

        // When not focused, header should be muted
        page.section = SettingsSection::Toggles;
        let view1 = page.view(80, 60, &theme);
        assert!(view1.contains("Keybindings Reference"));

        // When focused, should have arrow indicator
        page.section = SettingsSection::Keybindings;
        let view2 = page.view(80, 60, &theme);
        assert!(view2.contains("▸ Keybindings Reference"));
    }

    // =========================================================================
    // About + Diagnostics section tests (bd-2kp1)
    // =========================================================================

    #[test]
    fn settings_page_section_toggle_includes_about() {
        let mut page = SettingsPage::new();
        assert_eq!(page.section, SettingsSection::Toggles);

        page.next_section();
        assert_eq!(page.section, SettingsSection::Themes);

        page.next_section();
        assert_eq!(page.section, SettingsSection::Keybindings);

        page.next_section();
        assert_eq!(page.section, SettingsSection::About);

        page.next_section();
        assert_eq!(page.section, SettingsSection::Toggles);
    }

    #[test]
    fn settings_page_about_navigation() {
        let mut page = SettingsPage::new();
        page.section = SettingsSection::About;
        assert_eq!(page.about_action, AboutAction::CopyDiagnostics);

        page.move_down();
        assert_eq!(page.about_action, AboutAction::OpenInPager);

        page.move_up();
        assert_eq!(page.about_action, AboutAction::CopyDiagnostics);
    }

    #[test]
    fn settings_page_view_contains_about_section() {
        let page = SettingsPage::new();
        let theme = Theme::dark();
        let view = page.view(80, 80, &theme);

        assert!(view.contains("About + Diagnostics"));
        assert!(view.contains("charmed_rust"));
        assert!(view.contains("Terminal:"));
        assert!(view.contains("Copy to file"));
        assert!(view.contains("Open in pager"));
    }

    #[test]
    fn settings_page_about_focused_style() {
        let mut page = SettingsPage::new();
        let theme = Theme::dark();

        page.section = SettingsSection::About;
        let view = page.view(80, 80, &theme);
        assert!(view.contains("▸ About + Diagnostics"));
    }

    #[test]
    fn settings_page_generate_diagnostics_includes_version() {
        let page = SettingsPage::new();
        let diag = page.generate_full_diagnostics();

        assert!(diag.contains("CHARMED_RUST"));
        assert!(diag.contains("VERSION INFO"));
        assert!(diag.contains("TERMINAL INFO"));
        assert!(diag.contains("FEATURE FLAGS"));
        assert!(diag.contains("RUNTIME CONFIG"));
        assert!(diag.contains("CURRENT TOGGLES"));
    }

    #[test]
    fn settings_page_sync_runtime_config() {
        let mut page = SettingsPage::new();
        let config = Config::default();

        page.sync_runtime_config(config.clone(), false);
        assert!(page.runtime_config.is_some());
        assert!(!page.is_headless);

        page.sync_runtime_config(config, true);
        assert!(page.is_headless);
    }

    #[test]
    fn settings_page_update_terminal_size() {
        let mut page = SettingsPage::new();

        page.update_terminal_size(120, 40);
        assert_eq!(page.terminal_width, 120);
        assert_eq!(page.terminal_height, 40);
    }

    #[test]
    fn settings_page_hints_vary_by_section() {
        let mut page = SettingsPage::new();

        page.section = SettingsSection::Toggles;
        assert!(page.hints().contains("toggle"));

        page.section = SettingsSection::Themes;
        assert!(page.hints().contains("theme"));

        page.section = SettingsSection::Keybindings;
        assert!(page.hints().contains("read-only"));

        page.section = SettingsSection::About;
        assert!(page.hints().contains("pager"));
    }

    #[test]
    fn settings_page_execute_copy_in_headless() {
        let mut page = SettingsPage::new();
        page.is_headless = true;
        page.about_action = AboutAction::CopyDiagnostics;
        page.section = SettingsSection::About;

        // Should return a command (writes to file)
        let cmd = page.execute_about_action();
        assert!(cmd.is_some());
    }

    #[test]
    fn settings_page_execute_pager_in_headless() {
        let mut page = SettingsPage::new();
        page.is_headless = true;
        page.about_action = AboutAction::OpenInPager;
        page.section = SettingsSection::About;

        // Should return a notification about unavailability
        let cmd = page.execute_about_action();
        assert!(cmd.is_some());
    }
}
