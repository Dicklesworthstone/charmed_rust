//! Command palette for quick action access (bd-3mtt).
//!
//! This module provides a VS Code/Sublime-style command palette that allows
//! users to quickly search and execute commands without remembering keybindings.
//!
//! # Usage
//!
//! The command palette is triggered by pressing `/` in the main app. It shows
//! a search input at the top and a filtered list of commands below.
//!
//! # Features
//!
//! - Fuzzy-ish matching over command names and keywords
//! - Navigation commands (jump to pages)
//! - Toggle commands (theme, animations, sidebar)
//! - Utility commands (export, diagnostics)
//! - Shows keybinding hints for discoverability

use bubbletea::{Cmd, KeyMsg, KeyType, Message};
use lipgloss::Style;

use crate::messages::{AppMsg, Page};
use crate::theme::Theme;

// =============================================================================
// COMMAND TYPES
// =============================================================================

/// A command that can be executed from the palette.
#[derive(Debug, Clone)]
pub struct Command {
    /// Unique identifier for the command.
    pub id: &'static str,
    /// Display title shown in the palette.
    pub title: &'static str,
    /// Short description of what the command does.
    pub description: &'static str,
    /// Category for grouping.
    pub category: CommandCategory,
    /// The action to execute when selected.
    pub action: CommandAction,
    /// Optional keybinding hint (e.g., "1", "?", "t").
    pub keybinding: Option<&'static str>,
    /// Additional keywords for fuzzy matching.
    pub keywords: &'static [&'static str],
}

/// Command categories for organization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandCategory {
    /// Navigation commands (go to pages).
    Navigation,
    /// Toggle/settings commands.
    Settings,
    /// View/display commands.
    View,
    /// Utility commands.
    Utility,
}

impl CommandCategory {
    /// Get display name for the category.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Navigation => "Navigation",
            Self::Settings => "Settings",
            Self::View => "View",
            Self::Utility => "Utility",
        }
    }

    /// Get icon for the category.
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Navigation => "→",
            Self::Settings => "⚙",
            Self::View => "◉",
            Self::Utility => "◆",
        }
    }
}

/// Actions that can be executed from the command palette.
#[derive(Debug, Clone)]
pub enum CommandAction {
    /// Navigate to a specific page.
    Navigate(Page),
    /// Toggle sidebar visibility.
    ToggleSidebar,
    /// Toggle animations.
    ToggleAnimations,
    /// Toggle mouse input.
    ToggleMouse,
    /// Toggle syntax highlighting.
    ToggleSyntax,
    /// Cycle to next theme.
    CycleTheme,
    /// Show help overlay.
    ShowHelp,
    /// Quit the application.
    Quit,
}

impl CommandAction {
    /// Convert this action to an AppMsg.
    pub fn to_app_msg(self) -> AppMsg {
        match self {
            Self::Navigate(page) => AppMsg::Navigate(page),
            Self::ToggleSidebar => AppMsg::ToggleSidebar,
            Self::ToggleAnimations => AppMsg::ToggleAnimations,
            Self::ToggleMouse => AppMsg::ToggleMouse,
            Self::ToggleSyntax => AppMsg::ToggleSyntax,
            Self::CycleTheme => AppMsg::CycleTheme,
            Self::ShowHelp => AppMsg::ShowHelp,
            Self::Quit => AppMsg::Quit,
        }
    }
}

// =============================================================================
// COMMAND REGISTRY
// =============================================================================

/// All available commands in the palette.
pub const COMMANDS: &[Command] = &[
    // Navigation commands
    Command {
        id: "nav-dashboard",
        title: "Go to Dashboard",
        description: "Platform health overview",
        category: CommandCategory::Navigation,
        action: CommandAction::Navigate(Page::Dashboard),
        keybinding: Some("1"),
        keywords: &["home", "overview", "metrics", "status"],
    },
    Command {
        id: "nav-services",
        title: "Go to Services",
        description: "Service catalog and status",
        category: CommandCategory::Navigation,
        action: CommandAction::Navigate(Page::Services),
        keybinding: Some("2"),
        keywords: &["service", "catalog", "microservice"],
    },
    Command {
        id: "nav-jobs",
        title: "Go to Jobs",
        description: "Background task monitoring",
        category: CommandCategory::Navigation,
        action: CommandAction::Navigate(Page::Jobs),
        keybinding: Some("3"),
        keywords: &["task", "background", "queue", "worker"],
    },
    Command {
        id: "nav-logs",
        title: "Go to Logs",
        description: "Aggregated log viewer",
        category: CommandCategory::Navigation,
        action: CommandAction::Navigate(Page::Logs),
        keybinding: Some("4"),
        keywords: &["log", "output", "debug", "trace"],
    },
    Command {
        id: "nav-docs",
        title: "Go to Docs",
        description: "Documentation browser",
        category: CommandCategory::Navigation,
        action: CommandAction::Navigate(Page::Docs),
        keybinding: Some("5"),
        keywords: &["documentation", "help", "readme", "guide"],
    },
    Command {
        id: "nav-files",
        title: "Go to Files",
        description: "File browser with preview",
        category: CommandCategory::Navigation,
        action: CommandAction::Navigate(Page::Files),
        keybinding: Some("6"),
        keywords: &["file", "browser", "explorer", "directory"],
    },
    Command {
        id: "nav-wizard",
        title: "Go to Wizard",
        description: "Multi-step workflows",
        category: CommandCategory::Navigation,
        action: CommandAction::Navigate(Page::Wizard),
        keybinding: Some("7"),
        keywords: &["form", "workflow", "setup", "configure"],
    },
    Command {
        id: "nav-settings",
        title: "Go to Settings",
        description: "Preferences and about",
        category: CommandCategory::Navigation,
        action: CommandAction::Navigate(Page::Settings),
        keybinding: Some("8"),
        keywords: &["preference", "config", "option", "about"],
    },
    // Settings commands
    Command {
        id: "toggle-sidebar",
        title: "Toggle Sidebar",
        description: "Show/hide navigation sidebar",
        category: CommandCategory::Settings,
        action: CommandAction::ToggleSidebar,
        keybinding: Some("["),
        keywords: &["sidebar", "nav", "panel", "hide", "show"],
    },
    Command {
        id: "toggle-animations",
        title: "Toggle Animations",
        description: "Enable/disable UI animations",
        category: CommandCategory::Settings,
        action: CommandAction::ToggleAnimations,
        keybinding: None,
        keywords: &["animation", "motion", "reduce", "performance"],
    },
    Command {
        id: "toggle-mouse",
        title: "Toggle Mouse Input",
        description: "Enable/disable mouse support",
        category: CommandCategory::Settings,
        action: CommandAction::ToggleMouse,
        keybinding: None,
        keywords: &["mouse", "click", "scroll", "pointer"],
    },
    Command {
        id: "toggle-syntax",
        title: "Toggle Syntax Highlighting",
        description: "Enable/disable code highlighting",
        category: CommandCategory::Settings,
        action: CommandAction::ToggleSyntax,
        keybinding: None,
        keywords: &["syntax", "highlight", "code", "color"],
    },
    Command {
        id: "cycle-theme",
        title: "Cycle Theme",
        description: "Switch to the next theme preset",
        category: CommandCategory::Settings,
        action: CommandAction::CycleTheme,
        keybinding: Some("t"),
        keywords: &["theme", "dark", "light", "color", "dracula", "gruvbox"],
    },
    // View commands
    Command {
        id: "show-help",
        title: "Show Help",
        description: "Display keyboard shortcuts",
        category: CommandCategory::View,
        action: CommandAction::ShowHelp,
        keybinding: Some("?"),
        keywords: &["help", "keyboard", "shortcut", "keys"],
    },
    // Utility commands
    Command {
        id: "quit",
        title: "Quit Application",
        description: "Exit the demo showcase",
        category: CommandCategory::Utility,
        action: CommandAction::Quit,
        keybinding: Some("q"),
        keywords: &["exit", "close", "bye"],
    },
];

// =============================================================================
// COMMAND PALETTE COMPONENT
// =============================================================================

/// Messages for the command palette.
#[derive(Debug, Clone)]
pub enum CommandPaletteMsg {
    /// User typed a character in the search input.
    SearchInput(char),
    /// User pressed backspace.
    Backspace,
    /// User navigated to next command.
    SelectNext,
    /// User navigated to previous command.
    SelectPrev,
    /// User confirmed selection.
    Execute,
    /// Close the palette.
    Close,
}

impl CommandPaletteMsg {
    /// Convert to a bubbletea Message.
    pub fn into_message(self) -> Message {
        Message::new(self)
    }
}

/// The command palette component.
#[derive(Debug, Clone)]
pub struct CommandPalette {
    /// Current search query.
    query: String,
    /// Filtered commands based on query.
    filtered_commands: Vec<usize>,
    /// Currently selected command index in filtered list.
    selected: usize,
    /// Whether the palette is visible.
    pub visible: bool,
}

impl CommandPalette {
    /// Create a new command palette.
    #[must_use]
    pub fn new() -> Self {
        let filtered_commands: Vec<usize> = (0..COMMANDS.len()).collect();
        Self {
            query: String::new(),
            filtered_commands,
            selected: 0,
            visible: false,
        }
    }

    /// Show the palette and reset state.
    pub fn show(&mut self) {
        self.visible = true;
        self.query.clear();
        self.filtered_commands = (0..COMMANDS.len()).collect();
        self.selected = 0;
    }

    /// Hide the palette.
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Get the currently selected command, if any.
    #[must_use]
    pub fn selected_command(&self) -> Option<&Command> {
        self.filtered_commands
            .get(self.selected)
            .and_then(|&idx| COMMANDS.get(idx))
    }

    /// Handle a key event and return an optional command.
    ///
    /// Returns `Some(Cmd)` if the palette produced an action.
    pub fn handle_key(&mut self, key: &KeyMsg) -> Option<Cmd> {
        match key.key_type {
            KeyType::Esc => {
                self.hide();
                None
            }
            KeyType::Enter => {
                if let Some(cmd) = self.selected_command() {
                    let action = cmd.action.clone();
                    self.hide();
                    Some(Cmd::new(move || action.to_app_msg().into_message()))
                } else {
                    None
                }
            }
            KeyType::Up => {
                self.select_prev();
                None
            }
            KeyType::Down => {
                self.select_next();
                None
            }
            KeyType::Backspace => {
                if !self.query.is_empty() {
                    self.query.pop();
                    self.update_filter();
                }
                None
            }
            KeyType::Runes if !key.runes.is_empty() => {
                // Handle Ctrl+N / Ctrl+P for navigation
                if key.runes == ['n'] && key.alt {
                    self.select_next();
                    return None;
                }
                if key.runes == ['p'] && key.alt {
                    self.select_prev();
                    return None;
                }

                // Add characters to query
                for c in &key.runes {
                    self.query.push(*c);
                }
                self.update_filter();
                None
            }
            _ => None,
        }
    }

    /// Move selection up.
    fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        } else if !self.filtered_commands.is_empty() {
            self.selected = self.filtered_commands.len() - 1;
        }
    }

    /// Move selection down.
    fn select_next(&mut self) {
        if !self.filtered_commands.is_empty() {
            self.selected = (self.selected + 1) % self.filtered_commands.len();
        }
    }

    /// Update the filtered command list based on current query.
    fn update_filter(&mut self) {
        let query = self.query.to_lowercase();

        if query.is_empty() {
            self.filtered_commands = (0..COMMANDS.len()).collect();
        } else {
            self.filtered_commands = COMMANDS
                .iter()
                .enumerate()
                .filter(|(_, cmd)| self.matches_query(cmd, &query))
                .map(|(idx, _)| idx)
                .collect();
        }

        // Reset selection if out of bounds
        if self.selected >= self.filtered_commands.len() {
            self.selected = 0;
        }
    }

    /// Check if a command matches the query (fuzzy-ish).
    fn matches_query(&self, cmd: &Command, query: &str) -> bool {
        let title_lower = cmd.title.to_lowercase();
        let desc_lower = cmd.description.to_lowercase();

        // Exact substring match
        if title_lower.contains(query) || desc_lower.contains(query) {
            return true;
        }

        // Check keywords
        for keyword in cmd.keywords {
            if keyword.to_lowercase().contains(query) {
                return true;
            }
        }

        // Simple fuzzy: all query chars appear in order in title
        let mut title_chars = title_lower.chars();
        for qc in query.chars() {
            if !title_chars.any(|tc| tc == qc) {
                return false;
            }
        }

        true
    }

    /// Render the command palette.
    #[must_use]
    pub fn view(&self, width: usize, height: usize, theme: &Theme) -> String {
        if !self.visible {
            return String::new();
        }

        let mut lines = Vec::new();

        // Calculate palette dimensions
        let palette_width = width.min(70).max(40);
        let palette_height = height.min(20).max(10);
        let left_pad = (width.saturating_sub(palette_width)) / 2;
        let top_pad = (height.saturating_sub(palette_height)) / 3;

        // Create padding
        let pad = " ".repeat(left_pad);

        // Top padding (dimmed background effect)
        for _ in 0..top_pad {
            lines.push(String::new());
        }

        // Border style
        let border_style = theme.info_style();

        // Top border
        let top_border = format!(
            "{}{}",
            pad,
            border_style.render(&format!("╭{}╮", "─".repeat(palette_width - 2)))
        );
        lines.push(top_border);

        // Title bar
        let title = " Command Palette ";
        let title_pad = (palette_width - 2 - title.len()) / 2;
        let title_line = format!(
            "{}{}",
            pad,
            border_style.render(&format!(
                "│{}{}{}│",
                " ".repeat(title_pad),
                theme.heading_style().render(title),
                " ".repeat(palette_width - 2 - title_pad - title.len())
            ))
        );
        lines.push(title_line);

        // Search input
        let search_prompt = "› ";
        let cursor = "│";
        let query_display = if self.query.is_empty() {
            theme.muted_style().render("Type to search...")
        } else {
            Style::new().foreground(theme.text).render(&self.query)
        };
        let search_line = format!(
            "{}{}",
            pad,
            border_style.render(&format!(
                "│ {}{}{} {}│",
                theme.info_style().render(search_prompt),
                query_display,
                theme.info_style().render(cursor),
                " ".repeat(
                    palette_width
                        .saturating_sub(6)
                        .saturating_sub(self.query.len())
                        .saturating_sub(if self.query.is_empty() { 17 } else { 0 })
                )
            ))
        );
        lines.push(search_line);

        // Separator
        let sep_line = format!(
            "{}{}",
            pad,
            border_style.render(&format!("├{}┤", "─".repeat(palette_width - 2)))
        );
        lines.push(sep_line.clone());

        // Command list
        let list_height = palette_height.saturating_sub(6);
        let visible_start = self.selected.saturating_sub(list_height / 2);
        let visible_end = (visible_start + list_height).min(self.filtered_commands.len());

        if self.filtered_commands.is_empty() {
            let no_results = theme.muted_style().render("No matching commands");
            let no_results_line = format!(
                "{}{}",
                pad,
                border_style.render(&format!(
                    "│ {} {}│",
                    no_results,
                    " ".repeat(palette_width.saturating_sub(4 + 21))
                ))
            );
            lines.push(no_results_line);
        } else {
            for (display_idx, &cmd_idx) in self
                .filtered_commands
                .iter()
                .enumerate()
                .skip(visible_start)
                .take(list_height)
            {
                let cmd = &COMMANDS[cmd_idx];
                let is_selected = display_idx == self.selected;

                let line = self.render_command_line(cmd, is_selected, palette_width - 4, theme);
                let cmd_line = format!(
                    "{}{}",
                    pad,
                    border_style.render(&format!("│ {} │", line))
                );
                lines.push(cmd_line);
            }

            // Fill remaining space
            for _ in 0..(list_height.saturating_sub(visible_end - visible_start)) {
                let empty_line = format!(
                    "{}{}",
                    pad,
                    border_style.render(&format!("│{}│", " ".repeat(palette_width - 2)))
                );
                lines.push(empty_line);
            }
        }

        // Separator
        lines.push(sep_line.clone());

        // Hints bar
        let hints = format!(
            "↑↓ navigate  Enter select  Esc close  ({}/{})",
            self.selected + 1,
            self.filtered_commands.len()
        );
        let hints_styled = theme.muted_style().render(&hints);
        let hints_line = format!(
            "{}{}",
            pad,
            border_style.render(&format!(
                "│ {} {}│",
                hints_styled,
                " ".repeat(palette_width.saturating_sub(4 + hints.len()))
            ))
        );
        lines.push(hints_line);

        // Bottom border
        let bottom_border = format!(
            "{}{}",
            pad,
            border_style.render(&format!("╰{}╯", "─".repeat(palette_width - 2)))
        );
        lines.push(bottom_border);

        lines.join("\n")
    }

    /// Render a single command line.
    fn render_command_line(
        &self,
        cmd: &Command,
        selected: bool,
        width: usize,
        theme: &Theme,
    ) -> String {
        let icon = cmd.category.icon();
        let keybind = cmd.keybinding.map_or(String::new(), |k| format!("[{}]", k));

        let title_width = width.saturating_sub(icon.len() + 2 + keybind.len() + 1);
        let title = if cmd.title.len() > title_width {
            format!("{}...", &cmd.title[..title_width.saturating_sub(3)])
        } else {
            cmd.title.to_string()
        };

        let (icon_style, title_style, key_style) = if selected {
            (
                theme.info_style().bold(),
                theme.selected_style(),
                theme.info_style(),
            )
        } else {
            (
                theme.muted_style(),
                Style::new().foreground(theme.text),
                theme.muted_style(),
            )
        };

        let padding = width.saturating_sub(icon.len() + 1 + title.len() + keybind.len());

        format!(
            "{} {}{}{}",
            icon_style.render(icon),
            title_style.render(&title),
            " ".repeat(padding),
            key_style.render(&keybind)
        )
    }
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_palette_starts_hidden() {
        let palette = CommandPalette::new();
        assert!(!palette.visible);
    }

    #[test]
    fn command_palette_show_resets_state() {
        let mut palette = CommandPalette::new();
        palette.query = "test".to_string();
        palette.selected = 5;
        palette.show();
        assert!(palette.visible);
        assert!(palette.query.is_empty());
        assert_eq!(palette.selected, 0);
    }

    #[test]
    fn command_palette_hide_works() {
        let mut palette = CommandPalette::new();
        palette.show();
        palette.hide();
        assert!(!palette.visible);
    }

    #[test]
    fn command_palette_initial_commands() {
        let palette = CommandPalette::new();
        assert_eq!(palette.filtered_commands.len(), COMMANDS.len());
    }

    #[test]
    fn command_palette_filter_by_query() {
        let mut palette = CommandPalette::new();
        palette.query = "dashboard".to_string();
        palette.update_filter();
        assert!(!palette.filtered_commands.is_empty());
        // Should include the dashboard command
        assert!(palette
            .filtered_commands
            .iter()
            .any(|&idx| COMMANDS[idx].id == "nav-dashboard"));
    }

    #[test]
    fn command_palette_filter_by_keyword() {
        let mut palette = CommandPalette::new();
        palette.query = "metrics".to_string();
        palette.update_filter();
        assert!(!palette.filtered_commands.is_empty());
        // "metrics" is a keyword for dashboard
        assert!(palette
            .filtered_commands
            .iter()
            .any(|&idx| COMMANDS[idx].id == "nav-dashboard"));
    }

    #[test]
    fn command_palette_empty_query_shows_all() {
        let mut palette = CommandPalette::new();
        palette.query = "test".to_string();
        palette.update_filter();
        let filtered_count = palette.filtered_commands.len();

        palette.query.clear();
        palette.update_filter();
        assert_eq!(palette.filtered_commands.len(), COMMANDS.len());
        assert!(palette.filtered_commands.len() >= filtered_count);
    }

    #[test]
    fn command_palette_select_next() {
        let mut palette = CommandPalette::new();
        palette.show();
        assert_eq!(palette.selected, 0);
        palette.select_next();
        assert_eq!(palette.selected, 1);
    }

    #[test]
    fn command_palette_select_prev() {
        let mut palette = CommandPalette::new();
        palette.show();
        palette.selected = 3;
        palette.select_prev();
        assert_eq!(palette.selected, 2);
    }

    #[test]
    fn command_palette_select_wraps() {
        let mut palette = CommandPalette::new();
        palette.show();
        palette.selected = palette.filtered_commands.len() - 1;
        palette.select_next();
        assert_eq!(palette.selected, 0);
    }

    #[test]
    fn command_palette_select_prev_wraps() {
        let mut palette = CommandPalette::new();
        palette.show();
        assert_eq!(palette.selected, 0);
        palette.select_prev();
        assert_eq!(palette.selected, palette.filtered_commands.len() - 1);
    }

    #[test]
    fn command_palette_selected_command() {
        let palette = CommandPalette::new();
        let cmd = palette.selected_command();
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().id, COMMANDS[0].id);
    }

    #[test]
    fn command_action_to_app_msg() {
        let action = CommandAction::Navigate(Page::Dashboard);
        let msg = action.to_app_msg();
        match msg {
            AppMsg::Navigate(Page::Dashboard) => {}
            _ => panic!("Expected Navigate(Dashboard)"),
        }
    }

    #[test]
    fn command_categories_have_names() {
        assert_eq!(CommandCategory::Navigation.name(), "Navigation");
        assert_eq!(CommandCategory::Settings.name(), "Settings");
        assert_eq!(CommandCategory::View.name(), "View");
        assert_eq!(CommandCategory::Utility.name(), "Utility");
    }

    #[test]
    fn command_categories_have_icons() {
        assert!(!CommandCategory::Navigation.icon().is_empty());
        assert!(!CommandCategory::Settings.icon().is_empty());
        assert!(!CommandCategory::View.icon().is_empty());
        assert!(!CommandCategory::Utility.icon().is_empty());
    }

    #[test]
    fn all_commands_have_required_fields() {
        for cmd in COMMANDS {
            assert!(!cmd.id.is_empty());
            assert!(!cmd.title.is_empty());
            assert!(!cmd.description.is_empty());
        }
    }

    #[test]
    fn command_palette_view_when_hidden() {
        let palette = CommandPalette::new();
        let theme = Theme::dark();
        let view = palette.view(80, 24, &theme);
        assert!(view.is_empty());
    }

    #[test]
    fn command_palette_view_when_visible() {
        let mut palette = CommandPalette::new();
        palette.show();
        let theme = Theme::dark();
        let view = palette.view(80, 24, &theme);
        assert!(!view.is_empty());
        assert!(view.contains("Command Palette"));
    }
}
