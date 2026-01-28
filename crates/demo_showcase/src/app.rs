//! Main application model and routing.
//!
//! The `App` struct is the top-level model that handles:
//! - Global state (theme, toggles, current page)
//! - Message routing to page models
//! - App chrome rendering (header, sidebar, footer)

use bubbletea::{
    Cmd, KeyMsg, KeyType, Message, Model, WindowSizeMsg, batch, quit, set_window_title,
};
use lipgloss::{Position, Style};

use crate::components::{Sidebar, SidebarFocus, StatusLevel, banner, key_hint};
use crate::keymap::{HELP_SECTIONS, help_total_lines};
use crate::messages::{AppMsg, ExportFormat, ExportMsg, Notification, NotificationMsg, Page};
use crate::pages::Pages;
use crate::theme::{Theme, ThemePreset, spacing};

/// Convert ANSI-styled terminal output to HTML with inline styles.
///
/// This function parses ANSI escape codes and converts them to HTML spans
/// with appropriate CSS styling, preserving colors and text attributes.
#[allow(clippy::too_many_lines, clippy::similar_names, clippy::collapsible_if)]
fn ansi_to_html(input: &str) -> String {
    let mut html = String::with_capacity(input.len() * 2);
    html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
    html.push_str("<meta charset=\"utf-8\">\n");
    html.push_str("<title>Demo Showcase Export</title>\n");
    html.push_str("<style>\n");
    html.push_str("body { background: #1a1a2e; color: #eaeaea; font-family: 'Monaco', 'Menlo', 'Consolas', monospace; font-size: 14px; line-height: 1.4; padding: 20px; white-space: pre; }\n");
    html.push_str(".bold { font-weight: bold; }\n");
    html.push_str(".italic { font-style: italic; }\n");
    html.push_str(".underline { text-decoration: underline; }\n");
    html.push_str(".dim { opacity: 0.6; }\n");
    html.push_str(".strikethrough { text-decoration: line-through; }\n");
    html.push_str("</style>\n</head>\n<body>\n");

    let mut in_escape = false;
    let mut escape_buf = String::new();
    let mut current_styles: Vec<&str> = Vec::new();
    let mut current_fg: Option<String> = None;
    let mut current_bg: Option<String> = None;

    for c in input.chars() {
        if c == '\x1b' {
            in_escape = true;
            escape_buf.clear();
            continue;
        }

        if in_escape {
            escape_buf.push(c);
            if c == 'm' {
                // Parse the escape sequence
                let seq = escape_buf.trim_start_matches('[').trim_end_matches('m');
                for code in seq.split(';') {
                    match code {
                        "0" => {
                            // Reset
                            if !current_styles.is_empty()
                                || current_fg.is_some()
                                || current_bg.is_some()
                            {
                                html.push_str("</span>");
                            }
                            current_styles.clear();
                            current_fg = None;
                            current_bg = None;
                        }
                        "1" => current_styles.push("bold"),
                        "2" => current_styles.push("dim"),
                        "3" => current_styles.push("italic"),
                        "4" => current_styles.push("underline"),
                        "9" => current_styles.push("strikethrough"),
                        // Basic foreground colors (30-37)
                        "30" => current_fg = Some("#000000".to_string()),
                        "31" => current_fg = Some("#cc0000".to_string()),
                        "32" => current_fg = Some("#00cc00".to_string()),
                        "33" => current_fg = Some("#cccc00".to_string()),
                        "34" => current_fg = Some("#0000cc".to_string()),
                        "35" => current_fg = Some("#cc00cc".to_string()),
                        "36" => current_fg = Some("#00cccc".to_string()),
                        "37" => current_fg = Some("#cccccc".to_string()),
                        // Bright foreground colors (90-97)
                        "90" => current_fg = Some("#666666".to_string()),
                        "91" => current_fg = Some("#ff0000".to_string()),
                        "92" => current_fg = Some("#00ff00".to_string()),
                        "93" => current_fg = Some("#ffff00".to_string()),
                        "94" => current_fg = Some("#0000ff".to_string()),
                        "95" => current_fg = Some("#ff00ff".to_string()),
                        "96" => current_fg = Some("#00ffff".to_string()),
                        "97" => current_fg = Some("#ffffff".to_string()),
                        // Basic background colors (40-47)
                        "40" => current_bg = Some("#000000".to_string()),
                        "41" => current_bg = Some("#cc0000".to_string()),
                        "42" => current_bg = Some("#00cc00".to_string()),
                        "43" => current_bg = Some("#cccc00".to_string()),
                        "44" => current_bg = Some("#0000cc".to_string()),
                        "45" => current_bg = Some("#cc00cc".to_string()),
                        "46" => current_bg = Some("#00cccc".to_string()),
                        "47" => current_bg = Some("#cccccc".to_string()),
                        // 256-color and RGB handled via 38;5;N or 38;2;R;G;B
                        _ => {
                            // Handle 256-color: 38;5;N or 48;5;N
                            if let Some(rest) = seq.strip_prefix("38;5;") {
                                if let Ok(n) = rest.parse::<u8>() {
                                    current_fg = Some(ansi256_to_hex(n));
                                }
                            } else if let Some(rest) = seq.strip_prefix("48;5;") {
                                if let Ok(n) = rest.parse::<u8>() {
                                    current_bg = Some(ansi256_to_hex(n));
                                }
                            }
                            // Handle RGB: 38;2;R;G;B or 48;2;R;G;B
                            else if let Some(rest) = seq.strip_prefix("38;2;") {
                                let parts: Vec<&str> = rest.split(';').collect();
                                if parts.len() == 3 {
                                    if let (Ok(r), Ok(g), Ok(b)) = (
                                        parts[0].parse::<u8>(),
                                        parts[1].parse::<u8>(),
                                        parts[2].parse::<u8>(),
                                    ) {
                                        current_fg = Some(format!("#{r:02x}{g:02x}{b:02x}"));
                                    }
                                }
                            } else if let Some(rest) = seq.strip_prefix("48;2;") {
                                let parts: Vec<&str> = rest.split(';').collect();
                                if parts.len() == 3 {
                                    if let (Ok(r), Ok(g), Ok(b)) = (
                                        parts[0].parse::<u8>(),
                                        parts[1].parse::<u8>(),
                                        parts[2].parse::<u8>(),
                                    ) {
                                        current_bg = Some(format!("#{r:02x}{g:02x}{b:02x}"));
                                    }
                                }
                            }
                        }
                    }
                }

                // Open a new span if we have styles
                if !current_styles.is_empty() || current_fg.is_some() || current_bg.is_some() {
                    html.push_str("<span");
                    let mut style_parts = Vec::new();
                    if let Some(ref fg) = current_fg {
                        style_parts.push(format!("color:{fg}"));
                    }
                    if let Some(ref bg) = current_bg {
                        style_parts.push(format!("background:{bg}"));
                    }
                    if !style_parts.is_empty() {
                        html.push_str(&format!(" style=\"{}\"", style_parts.join(";")));
                    }
                    if !current_styles.is_empty() {
                        html.push_str(&format!(" class=\"{}\"", current_styles.join(" ")));
                    }
                    html.push('>');
                }
                in_escape = false;
            }
            continue;
        }

        // Escape HTML special characters
        match c {
            '&' => html.push_str("&amp;"),
            '<' => html.push_str("&lt;"),
            '>' => html.push_str("&gt;"),
            '"' => html.push_str("&quot;"),
            '\n' => html.push('\n'),
            _ => html.push(c),
        }
    }

    // Close any remaining span
    if !current_styles.is_empty() || current_fg.is_some() || current_bg.is_some() {
        html.push_str("</span>");
    }

    html.push_str("\n</body>\n</html>");
    html
}

/// Convert ANSI 256-color index to hex color.
fn ansi256_to_hex(n: u8) -> String {
    match n {
        // Standard colors (0-15)
        0 => "#000000".to_string(),
        1 => "#800000".to_string(),
        2 => "#008000".to_string(),
        3 => "#808000".to_string(),
        4 => "#000080".to_string(),
        5 => "#800080".to_string(),
        6 => "#008080".to_string(),
        7 => "#c0c0c0".to_string(),
        8 => "#808080".to_string(),
        9 => "#ff0000".to_string(),
        10 => "#00ff00".to_string(),
        11 => "#ffff00".to_string(),
        12 => "#0000ff".to_string(),
        13 => "#ff00ff".to_string(),
        14 => "#00ffff".to_string(),
        15 => "#ffffff".to_string(),
        // 216 colors (16-231)
        16..=231 => {
            let n = n - 16;
            let r = (n / 36) * 51;
            let g = ((n % 36) / 6) * 51;
            let b = (n % 6) * 51;
            format!("#{r:02x}{g:02x}{b:02x}")
        }
        // Grayscale (232-255)
        232..=255 => {
            let gray = (n - 232) * 10 + 8;
            format!("#{gray:02x}{gray:02x}{gray:02x}")
        }
    }
}

/// Strip ANSI escape codes from a string.
fn strip_ansi(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut in_escape = false;
    for c in input.chars() {
        if c == '\x1b' {
            in_escape = true;
            continue;
        }
        if in_escape {
            if c == 'm' {
                in_escape = false;
            }
            continue;
        }
        result.push(c);
    }
    result
}

/// Application configuration.
///
/// This struct holds runtime settings that can be toggled during the session.
/// For animation settings, the canonical source of truth is [`App::use_animations()`].
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields will be used as features are implemented
pub struct AppConfig {
    /// Initial theme preset.
    pub theme: ThemePreset,
    /// Whether animations are enabled.
    ///
    /// This controls all motion in the app. When disabled:
    /// - Transitions are instant
    /// - Progress bars don't animate
    /// - Spinners show static state
    ///
    /// Can be toggled at runtime via `AppMsg::ToggleAnimations`.
    /// Query via [`App::use_animations()`].
    pub animations: bool,
    /// Whether mouse support is enabled.
    pub mouse: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemePreset::Dark,
            animations: true,
            mouse: false,
        }
    }
}

/// Maximum number of notifications to display at once.
const MAX_NOTIFICATIONS: usize = 3;

/// Main application state.
pub struct App {
    /// Application configuration.
    config: AppConfig,
    /// Current theme.
    theme: Theme,
    /// Current page.
    current_page: Page,
    /// Page models.
    pages: Pages,
    /// Window dimensions.
    width: usize,
    height: usize,
    /// Whether the app is ready (received window size).
    ready: bool,
    /// Whether help overlay is shown.
    show_help: bool,
    /// Scroll offset for help overlay content.
    help_scroll_offset: usize,
    /// Whether sidebar is visible.
    sidebar_visible: bool,
    /// Sidebar component with navigation and filtering.
    sidebar: Sidebar,
    /// Active notifications (newest at end).
    notifications: Vec<Notification>,
    /// Counter for generating unique notification IDs.
    next_notification_id: u64,
}

impl App {
    /// Create a new application with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(AppConfig::default())
    }

    /// Create a new application with the given configuration.
    #[must_use]
    pub fn with_config(config: AppConfig) -> Self {
        let theme = Theme::from_preset(config.theme);
        Self {
            config,
            theme,
            current_page: Page::Dashboard,
            pages: Pages::default(),
            width: 80,
            height: 24,
            ready: false,
            show_help: false,
            help_scroll_offset: 0,
            sidebar_visible: true,
            sidebar: Sidebar::new(),
            notifications: Vec::new(),
            next_notification_id: 1,
        }
    }

    /// Show a notification to the user.
    ///
    /// This is the primary API for pages to emit notifications.
    /// Notifications are displayed in the footer area and auto-trimmed
    /// if there are too many.
    #[allow(dead_code)] // Will be used by pages
    pub fn notify(&mut self, message: impl Into<String>, level: StatusLevel) {
        let id = self.next_notification_id;
        self.next_notification_id += 1;
        let notification = Notification::new(id, message, level);
        self.notifications.push(notification);

        // Keep only the most recent notifications
        while self.notifications.len() > MAX_NOTIFICATIONS {
            self.notifications.remove(0);
        }
    }

    /// Get the next notification ID (useful for pages that want to track notifications).
    #[must_use]
    #[allow(dead_code)]
    #[allow(clippy::missing_const_for_fn)] // Mutates self.next_notification_id
    pub fn next_id(&mut self) -> u64 {
        let id = self.next_notification_id;
        self.next_notification_id += 1;
        id
    }

    /// Dismiss a notification by ID.
    fn dismiss_notification(&mut self, id: u64) {
        self.notifications.retain(|n| n.id != id);
    }

    /// Dismiss the oldest notification.
    fn dismiss_oldest_notification(&mut self) {
        if !self.notifications.is_empty() {
            self.notifications.remove(0);
        }
    }

    /// Clear all notifications.
    fn clear_notifications(&mut self) {
        self.notifications.clear();
    }

    // =========================================================================
    // Animation Control (bd-2szb)
    // =========================================================================

    /// Check if animations should be used.
    ///
    /// This is the **canonical source of truth** for all animation decisions.
    /// All code that performs animations must consult this method.
    ///
    /// Returns `false` when:
    /// - `--no-animations` CLI flag was passed
    /// - `REDUCE_MOTION` environment variable is set (returns false for full disable)
    /// - User toggled animations off via Settings
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if app.use_animations() {
    ///     // Perform smooth transition
    ///     spring.animate_to(target);
    /// } else {
    ///     // Instant snap to target
    ///     value = target;
    /// }
    /// ```
    #[must_use]
    #[allow(dead_code)] // Will be used by components, pages, and animations
    pub const fn use_animations(&self) -> bool {
        self.config.animations
    }

    /// Toggle animations on/off.
    ///
    /// This is typically called from the Settings page or via keyboard shortcut.
    pub const fn toggle_animations(&mut self) {
        self.config.animations = !self.config.animations;
    }

    /// Set animations enabled state directly.
    ///
    /// Useful for tests that need deterministic rendering.
    #[allow(dead_code)] // Used by tests for deterministic rendering
    pub const fn set_animations(&mut self, enabled: bool) {
        self.config.animations = enabled;
    }

    // =========================================================================
    // Theme Switching (bd-k52c)
    // =========================================================================

    /// Get the current theme.
    #[must_use]
    #[allow(dead_code)] // Used by pages and components
    pub const fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Get the current theme preset.
    #[must_use]
    #[allow(dead_code)] // Used by pages and tests
    pub const fn theme_preset(&self) -> ThemePreset {
        self.theme.preset
    }

    /// Set the application theme.
    ///
    /// This instantly updates the theme across the entire application.
    /// All rendered content will use the new theme colors on next `view()`.
    pub const fn set_theme(&mut self, preset: ThemePreset) {
        self.theme = Theme::from_preset(preset);
        self.config.theme = preset;
    }

    /// Cycle to the next theme preset.
    ///
    /// Useful for quick theme switching via keyboard shortcut.
    pub fn cycle_theme(&mut self) {
        let presets = ThemePreset::all();
        let current_idx = presets
            .iter()
            .position(|&p| p == self.theme.preset)
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % presets.len();
        self.set_theme(presets[next_idx]);
    }

    // =========================================================================
    // Navigation
    // =========================================================================

    /// Navigate to a new page.
    fn navigate(&mut self, page: Page) -> Option<Cmd> {
        if page == self.current_page {
            return None;
        }

        // Leave current page
        let leave_cmd = self.pages.get_mut(self.current_page).on_leave();

        // Enter new page
        self.current_page = page;
        self.sidebar.set_current_page(page);
        let enter_cmd = self.pages.get_mut(page).on_enter();

        // Combine commands
        batch(vec![leave_cmd, enter_cmd])
    }

    /// Handle global keyboard shortcuts.
    fn handle_global_key(&mut self, key: &KeyMsg) -> Option<Cmd> {
        // Handle help overlay scrolling
        if self.show_help {
            return self.handle_help_key(key);
        }

        // Ctrl+C always quits
        if key.key_type == KeyType::CtrlC {
            return Some(quit());
        }

        // Handle sidebar focus toggle with Tab
        if key.key_type == KeyType::Tab && self.sidebar_visible {
            self.sidebar.toggle_focus();
            return None;
        }

        // When sidebar is focused, pass keys to it (except global shortcuts)
        if self.sidebar.is_focused() && self.sidebar_visible {
            // Allow Escape to unfocus sidebar
            if key.key_type == KeyType::Esc {
                self.sidebar.set_focus(SidebarFocus::Inactive);
                return None;
            }
            // Pass to sidebar
            return self.sidebar.update(&Message::new(key.clone()));
        }

        match key.key_type {
            KeyType::Esc => return Some(quit()),
            KeyType::Runes => match key.runes.as_slice() {
                ['q'] => return Some(quit()),
                ['?'] => {
                    self.show_help = true;
                    self.help_scroll_offset = 0;
                    return None;
                }
                ['['] => {
                    self.sidebar_visible = !self.sidebar_visible;
                    return None;
                }
                ['t'] => {
                    // Cycle through themes
                    self.cycle_theme();
                    return None;
                }
                ['e'] => {
                    // Export current view as plain text
                    return Some(Cmd::new(|| {
                        ExportMsg::Export(ExportFormat::PlainText).into_message()
                    }));
                }
                ['E'] => {
                    // Export current view as HTML
                    return Some(Cmd::new(|| {
                        ExportMsg::Export(ExportFormat::Html).into_message()
                    }));
                }
                [c] => {
                    if let Some(page) = Page::from_shortcut(*c) {
                        return self.navigate(page);
                    }
                }
                _ => {}
            },
            _ => {}
        }
        None
    }

    /// Handle keyboard input when help overlay is shown.
    fn handle_help_key(&mut self, key: &KeyMsg) -> Option<Cmd> {
        let total_lines = help_total_lines();
        let visible_lines = self.help_visible_lines();
        let max_scroll = total_lines.saturating_sub(visible_lines);

        match key.key_type {
            KeyType::Esc => {
                self.show_help = false;
                return None;
            }
            KeyType::Up => {
                self.help_scroll_offset = self.help_scroll_offset.saturating_sub(1);
                return None;
            }
            KeyType::Down => {
                self.help_scroll_offset = (self.help_scroll_offset + 1).min(max_scroll);
                return None;
            }
            KeyType::Home => {
                self.help_scroll_offset = 0;
                return None;
            }
            KeyType::End => {
                self.help_scroll_offset = max_scroll;
                return None;
            }
            KeyType::PgUp => {
                self.help_scroll_offset = self
                    .help_scroll_offset
                    .saturating_sub(visible_lines.saturating_sub(2));
                return None;
            }
            KeyType::PgDown => {
                self.help_scroll_offset =
                    (self.help_scroll_offset + visible_lines.saturating_sub(2)).min(max_scroll);
                return None;
            }
            KeyType::CtrlU => {
                self.help_scroll_offset = self.help_scroll_offset.saturating_sub(visible_lines / 2);
                return None;
            }
            KeyType::CtrlD => {
                self.help_scroll_offset =
                    (self.help_scroll_offset + visible_lines / 2).min(max_scroll);
                return None;
            }
            KeyType::Runes => match key.runes.as_slice() {
                ['?' | 'q'] => {
                    self.show_help = false;
                    return None;
                }
                ['j'] => {
                    self.help_scroll_offset = (self.help_scroll_offset + 1).min(max_scroll);
                    return None;
                }
                ['k'] => {
                    self.help_scroll_offset = self.help_scroll_offset.saturating_sub(1);
                    return None;
                }
                ['g'] => {
                    self.help_scroll_offset = 0;
                    return None;
                }
                ['G'] => {
                    self.help_scroll_offset = max_scroll;
                    return None;
                }
                _ => {}
            },
            _ => {}
        }
        None
    }

    /// Calculate the number of visible lines in the help overlay.
    const fn help_visible_lines(&self) -> usize {
        // Help modal uses most of the screen with some padding
        // Header (1) + title bar (1) + footer hint (1) + border padding (4)
        self.height.saturating_sub(8)
    }

    /// Render the sidebar.
    fn render_sidebar(&self, height: usize) -> String {
        self.sidebar.view(height, &self.theme)
    }

    /// Render the header.
    fn render_header(&self) -> String {
        let title = self.theme.title_style().render(" Charmed Control Center ");

        let status = self.theme.success_style().render("Connected");

        // Add theme name indicator
        let theme_name = self
            .theme
            .muted_style()
            .render(&format!("[{}]", self.theme.preset.name()));

        // Calculate spacing to right-align theme name
        let left_content = format!("{title}  {status}");
        let left_len = strip_ansi_len(&left_content);
        let right_len = strip_ansi_len(&theme_name);
        let gap = self.width.saturating_sub(left_len + right_len + 2);
        let spacer = " ".repeat(gap);

        let header_content = format!("{left_content}{spacer}{theme_name} ");

        #[expect(clippy::cast_possible_truncation)]
        let width_u16 = self.width as u16;

        self.theme
            .header_style()
            .width(width_u16)
            .render(&header_content)
    }

    /// Render the footer.
    fn render_footer(&self) -> String {
        let page_hints = self.pages.get(self.current_page).hints();

        let global_hints = "1-7 pages  [ sidebar  ? help  q quit";

        let hints = format!("  {page_hints}  |  {global_hints}");

        #[expect(clippy::cast_possible_truncation)]
        let width_u16 = self.width as u16;

        self.theme.footer_style().width(width_u16).render(&hints)
    }

    /// Render notifications as a stack above the footer.
    fn render_notifications(&self) -> String {
        if self.notifications.is_empty() {
            return String::new();
        }

        self.notifications
            .iter()
            .map(|notif| {
                banner(
                    &self.theme,
                    notif.level,
                    &notif.message,
                    notif.action_hint.as_deref(),
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Render the help overlay.
    #[expect(
        clippy::too_many_lines,
        reason = "Complex render function with detailed formatting"
    )]
    fn render_help(&self) -> String {
        // Calculate dimensions - wider box for better readability
        let box_width: usize = 52.min(self.width.saturating_sub(4));
        let box_height = self.height.saturating_sub(4);
        let start_x = self.width.saturating_sub(box_width) / 2;
        let start_y = 2; // Small top margin

        let content_width = box_width.saturating_sub(6); // Padding on sides
        let visible_lines = self.help_visible_lines();

        // Build all content lines
        let mut content_lines: Vec<String> = Vec::new();

        // Add current page context at top
        let page_name = self.current_page.name();
        let page_hints = self.pages.get(self.current_page).hints();
        content_lines.push(format!("Current Page: {page_name}"));
        content_lines.push(format!("  {page_hints}"));
        content_lines.push(String::new());

        // Add sections from keymap
        for section in HELP_SECTIONS {
            // Section title (bold styling applied in render)
            content_lines.push(format!("[ {} ]", section.title));

            // Entries with aligned columns
            for entry in section.entries {
                let key_col = format!("{:>12}", entry.key);
                let line = format!("  {key_col}  {}", entry.action);
                content_lines.push(line);
            }
            content_lines.push(String::new()); // Blank line after section
        }

        // Calculate total and apply scroll offset
        let total_lines = content_lines.len();
        let max_scroll = total_lines.saturating_sub(visible_lines);
        let skip = self.help_scroll_offset.min(max_scroll);
        let visible_content: Vec<&String> = content_lines
            .iter()
            .skip(skip)
            .take(visible_lines)
            .collect();

        // Build output
        let mut lines: Vec<String> = Vec::new();

        // Top padding
        for _ in 0..start_y {
            lines.push(String::new());
        }

        #[expect(clippy::cast_possible_truncation)]
        let box_width_u16 = box_width as u16;

        // Title bar with modal style
        let title = " Keyboard Shortcuts ";
        let title_padding = (box_width.saturating_sub(title.len())) / 2;
        let title_line = format!(
            "{}{}{}",
            " ".repeat(title_padding),
            title,
            " ".repeat(box_width.saturating_sub(title_padding + title.len()))
        );
        lines.push(format!(
            "{}{}",
            " ".repeat(start_x),
            self.theme
                .modal_style()
                .bold()
                .width(box_width_u16)
                .render(&title_line)
        ));

        // Content area styling
        let content_style = Style::new()
            .foreground(self.theme.text)
            .background(self.theme.bg_highlight);
        let section_style = Style::new()
            .foreground(self.theme.primary)
            .background(self.theme.bg_highlight)
            .bold();

        for line in &visible_content {
            // Truncate long lines gracefully
            let truncated = if line.len() > content_width {
                format!("{}...", &line[..content_width.saturating_sub(3)])
            } else {
                (*line).clone()
            };
            let padded = format!("{truncated:content_width$}");

            // Apply section title styling if this is a section header
            let styled_content = if line.starts_with("[ ") && line.ends_with(" ]") {
                section_style.render(&format!("   {padded}   "))
            } else {
                content_style.render(&format!("   {padded}   "))
            };

            lines.push(format!("{}{}", " ".repeat(start_x), styled_content));
        }

        // Pad to fill box height
        let content_rows = visible_content.len();
        let remaining_height = box_height.saturating_sub(content_rows + 3); // title + footer + spacing
        let empty_line = " ".repeat(box_width);
        for _ in 0..remaining_height {
            lines.push(format!(
                "{}{}",
                " ".repeat(start_x),
                content_style
                    .clone()
                    .width(box_width_u16)
                    .render(&empty_line)
            ));
        }

        // Scroll indicator
        let scroll_info = if total_lines > visible_lines {
            let percent = (skip * 100).checked_div(max_scroll).unwrap_or(100).min(100);
            format!("[{percent:>3}%]")
        } else {
            String::new()
        };

        // Footer with hints and scroll indicator
        let hints_text = key_hint(&self.theme, "j/k", "scroll");
        let close_text = key_hint(&self.theme, "q/?/Esc", "close");
        let footer_hints = format!("{hints_text}  {close_text}  {scroll_info}");
        let footer_padded = format!("{footer_hints:^box_width$}");
        lines.push(format!(
            "{}{}",
            " ".repeat(start_x),
            self.theme
                .footer_style()
                .width(box_width_u16)
                .render(&footer_padded)
        ));

        lines.join("\n")
    }

    /// Get the current content dimensions.
    #[must_use]
    #[allow(dead_code)] // Will be used by pages
    pub fn content_dimensions(&self) -> (usize, usize) {
        let header_height = usize::from(spacing::HEADER_HEIGHT);
        let footer_height = usize::from(spacing::FOOTER_HEIGHT);
        let content_height = self.height.saturating_sub(header_height + footer_height);

        let content_width = if self.sidebar_visible {
            self.width
                .saturating_sub(usize::from(spacing::SIDEBAR_WIDTH))
        } else {
            self.width
        };

        (content_width, content_height)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for App {
    fn init(&self) -> Option<Cmd> {
        // Set window title and request initial window size
        batch(vec![
            Some(set_window_title("Charmed Control Center")),
            Some(bubbletea::window_size()),
        ])
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle window resize
        if let Some(size) = msg.downcast_ref::<WindowSizeMsg>() {
            self.width = size.width as usize;
            self.height = size.height as usize;
            self.ready = true;
            return None;
        }

        // Handle app-level messages
        if let Some(app_msg) = msg.downcast_ref::<AppMsg>() {
            return match app_msg {
                AppMsg::Navigate(page) => self.navigate(*page),
                AppMsg::ToggleSidebar => {
                    self.sidebar_visible = !self.sidebar_visible;
                    None
                }
                AppMsg::ToggleAnimations => {
                    self.toggle_animations();
                    None
                }
                AppMsg::SetTheme(preset) => {
                    self.set_theme(*preset);
                    None
                }
                AppMsg::CycleTheme => {
                    self.cycle_theme();
                    None
                }
                AppMsg::ShowHelp => {
                    self.show_help = true;
                    None
                }
                AppMsg::HideHelp => {
                    self.show_help = false;
                    None
                }
                AppMsg::Quit => Some(quit()),
            };
        }

        // Handle notification messages
        if let Some(notif_msg) = msg.downcast_ref::<NotificationMsg>() {
            match notif_msg {
                NotificationMsg::Show(notification) => {
                    self.notifications.push(notification.clone());
                    while self.notifications.len() > MAX_NOTIFICATIONS {
                        self.notifications.remove(0);
                    }
                }
                NotificationMsg::Dismiss(id) => {
                    self.dismiss_notification(*id);
                }
                NotificationMsg::DismissOldest => {
                    self.dismiss_oldest_notification();
                }
                NotificationMsg::ClearAll => {
                    self.clear_notifications();
                }
            }
            return None;
        }

        // Handle export messages
        if let Some(export_msg) = msg.downcast_ref::<ExportMsg>() {
            match export_msg {
                ExportMsg::Export(format) => {
                    // Render the current view
                    let ansi_content = self.view();

                    // Convert to requested format
                    let (content, ext) = match format {
                        ExportFormat::PlainText => (strip_ansi(&ansi_content), "txt"),
                        ExportFormat::Html => (ansi_to_html(&ansi_content), "html"),
                    };

                    // Generate filename with timestamp
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    let page_name = self.current_page.name().to_lowercase();
                    let filename = format!("demo_{page_name}_{timestamp}.{ext}");

                    // Write to file (blocking I/O)
                    return Some(Cmd::blocking(move || {
                        match std::fs::write(&filename, content) {
                            Ok(()) => ExportMsg::ExportCompleted(filename).into_message(),
                            Err(e) => ExportMsg::ExportFailed(e.to_string()).into_message(),
                        }
                    }));
                }
                ExportMsg::ExportCompleted(filename) => {
                    let id = self.next_notification_id;
                    self.next_notification_id += 1;
                    self.notifications
                        .push(Notification::success(id, format!("Exported to {filename}")));
                    while self.notifications.len() > MAX_NOTIFICATIONS {
                        self.notifications.remove(0);
                    }
                }
                ExportMsg::ExportFailed(error) => {
                    let id = self.next_notification_id;
                    self.next_notification_id += 1;
                    self.notifications
                        .push(Notification::error(id, format!("Export failed: {error}")));
                    while self.notifications.len() > MAX_NOTIFICATIONS {
                        self.notifications.remove(0);
                    }
                }
            }
            return None;
        }

        // Handle keyboard input
        if let Some(key) = msg.downcast_ref::<KeyMsg>()
            && let Some(cmd) = self.handle_global_key(key)
        {
            return Some(cmd);
        }

        // Delegate to current page if not in help mode
        if !self.show_help {
            return self.pages.get_mut(self.current_page).update(&msg);
        }

        None
    }

    fn view(&self) -> String {
        if !self.ready {
            return "Loading...".to_string();
        }

        // If help is shown, render help overlay
        if self.show_help {
            return self.render_help();
        }

        let header = self.render_header();
        let footer = self.render_footer();
        let notifications = self.render_notifications();

        // Calculate content area using spacing constants
        let header_height = usize::from(spacing::HEADER_HEIGHT);
        let footer_height = usize::from(spacing::FOOTER_HEIGHT);
        let notification_height = self.notifications.len();
        let content_height = self
            .height
            .saturating_sub(header_height + footer_height + notification_height);

        let (sidebar, content_width) = if self.sidebar_visible {
            let sidebar = self.render_sidebar(content_height);
            let sidebar_width = usize::from(spacing::SIDEBAR_WIDTH);
            (Some(sidebar), self.width.saturating_sub(sidebar_width))
        } else {
            (None, self.width)
        };

        // Render current page
        let page_content =
            self.pages
                .get(self.current_page)
                .view(content_width, content_height, &self.theme);

        // Compose layout
        let main_area = if let Some(sb) = sidebar {
            lipgloss::join_horizontal(Position::Top, &[&sb, &page_content])
        } else {
            page_content
        };

        // Build final layout: header, content, notifications (if any), footer
        if notifications.is_empty() {
            lipgloss::join_vertical(Position::Left, &[&header, &main_area, &footer])
        } else {
            lipgloss::join_vertical(
                Position::Left,
                &[&header, &main_area, &notifications, &footer],
            )
        }
    }
}

/// Calculate the visible length of a string (excluding ANSI escape sequences).
fn strip_ansi_len(s: &str) -> usize {
    // Simple heuristic: count non-escape characters
    // This is a rough approximation; a full ANSI parser would be more accurate
    let mut len = 0;
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else {
            len += 1;
        }
    }
    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.theme, ThemePreset::Dark);
        assert!(config.animations);
        assert!(!config.mouse);
    }

    #[test]
    fn app_with_config_uses_theme() {
        let config = AppConfig {
            theme: ThemePreset::Dracula,
            animations: false,
            mouse: true,
        };
        let app = App::with_config(config);
        assert_eq!(app.theme.preset, ThemePreset::Dracula);
    }

    #[test]
    fn strip_ansi_len_basic() {
        assert_eq!(strip_ansi_len("hello"), 5);
        assert_eq!(strip_ansi_len("\x1b[31mred\x1b[0m"), 3);
        assert_eq!(strip_ansi_len("no escapes here"), 15);
    }

    #[test]
    fn content_dimensions_with_sidebar() {
        let mut app = App::new();
        app.width = 100;
        app.height = 30;
        app.sidebar_visible = true;
        let (w, h) = app.content_dimensions();
        assert_eq!(w, 100 - usize::from(spacing::SIDEBAR_WIDTH));
        assert_eq!(
            h,
            30 - usize::from(spacing::HEADER_HEIGHT) - usize::from(spacing::FOOTER_HEIGHT)
        );
    }

    #[test]
    fn content_dimensions_without_sidebar() {
        let mut app = App::new();
        app.width = 100;
        app.height = 30;
        app.sidebar_visible = false;
        let (w, h) = app.content_dimensions();
        assert_eq!(w, 100);
        assert_eq!(
            h,
            30 - usize::from(spacing::HEADER_HEIGHT) - usize::from(spacing::FOOTER_HEIGHT)
        );
    }

    #[test]
    fn notify_adds_notification() {
        let mut app = App::new();
        assert!(app.notifications.is_empty());
        app.notify("Test message", StatusLevel::Info);
        assert_eq!(app.notifications.len(), 1);
        assert_eq!(app.notifications[0].message, "Test message");
    }

    #[test]
    fn notify_trims_to_max() {
        let mut app = App::new();
        for i in 0..10 {
            app.notify(format!("Message {i}"), StatusLevel::Info);
        }
        assert_eq!(app.notifications.len(), MAX_NOTIFICATIONS);
        // Should have the most recent notifications
        assert!(app.notifications.last().unwrap().message.contains('9'));
    }

    #[test]
    fn dismiss_notification_removes_by_id() {
        let mut app = App::new();
        app.notify("First", StatusLevel::Info);
        app.notify("Second", StatusLevel::Warning);
        let first_id = app.notifications[0].id;
        app.dismiss_notification(first_id);
        assert_eq!(app.notifications.len(), 1);
        assert_eq!(app.notifications[0].message, "Second");
    }

    #[test]
    fn clear_notifications_removes_all() {
        let mut app = App::new();
        app.notify("One", StatusLevel::Info);
        app.notify("Two", StatusLevel::Success);
        app.clear_notifications();
        assert!(app.notifications.is_empty());
    }

    #[test]
    fn notification_constructors() {
        let notif = Notification::success(1, "Success!");
        assert_eq!(notif.level, StatusLevel::Success);

        let notif = Notification::warning(2, "Warning!");
        assert_eq!(notif.level, StatusLevel::Warning);

        let notif = Notification::error(3, "Error!");
        assert_eq!(notif.level, StatusLevel::Error);

        let notif = Notification::info(4, "Info!").with_action_hint("Press Enter");
        assert_eq!(notif.level, StatusLevel::Info);
        assert_eq!(notif.action_hint, Some("Press Enter".to_string()));
    }

    // =========================================================================
    // Animation Control tests (bd-2szb)
    // =========================================================================

    #[test]
    fn app_use_animations_default_enabled() {
        let app = App::new();
        assert!(app.use_animations());
    }

    #[test]
    fn app_use_animations_respects_config() {
        let config = AppConfig {
            animations: false,
            ..Default::default()
        };
        let app = App::with_config(config);
        assert!(!app.use_animations());
    }

    #[test]
    fn app_toggle_animations() {
        let mut app = App::new();
        assert!(app.use_animations());

        app.toggle_animations();
        assert!(!app.use_animations());

        app.toggle_animations();
        assert!(app.use_animations());
    }

    #[test]
    fn app_set_animations() {
        let mut app = App::new();

        app.set_animations(false);
        assert!(!app.use_animations());

        app.set_animations(true);
        assert!(app.use_animations());
    }

    #[test]
    fn app_animations_for_deterministic_tests() {
        // Tests can disable animations for deterministic rendering
        let config = AppConfig {
            animations: false,
            ..Default::default()
        };
        let app = App::with_config(config);

        // All animation checks should return false
        assert!(!app.use_animations());
        // Layout should still work (not tested here, but this is the contract)
    }

    // =========================================================================
    // Theme Switching tests (bd-k52c)
    // =========================================================================

    #[test]
    fn app_default_theme_is_dark() {
        let app = App::new();
        assert_eq!(app.theme_preset(), ThemePreset::Dark);
    }

    #[test]
    fn app_set_theme_changes_preset() {
        let mut app = App::new();
        assert_eq!(app.theme_preset(), ThemePreset::Dark);

        app.set_theme(ThemePreset::Light);
        assert_eq!(app.theme_preset(), ThemePreset::Light);

        app.set_theme(ThemePreset::Dracula);
        assert_eq!(app.theme_preset(), ThemePreset::Dracula);
    }

    #[test]
    fn app_set_theme_updates_colors() {
        let mut app = App::new();
        let dark_bg = app.theme().bg;

        app.set_theme(ThemePreset::Light);
        let light_bg = app.theme().bg;

        // Background colors should differ between themes
        assert_ne!(dark_bg, light_bg);
    }

    #[test]
    fn app_cycle_theme_cycles_through_presets() {
        let mut app = App::new();
        assert_eq!(app.theme_preset(), ThemePreset::Dark);

        app.cycle_theme();
        assert_eq!(app.theme_preset(), ThemePreset::Light);

        app.cycle_theme();
        assert_eq!(app.theme_preset(), ThemePreset::Dracula);

        app.cycle_theme();
        assert_eq!(app.theme_preset(), ThemePreset::Dark); // Wraps around
    }

    #[test]
    fn app_config_theme_is_updated() {
        let mut app = App::new();
        assert_eq!(app.config.theme, ThemePreset::Dark);

        app.set_theme(ThemePreset::Light);
        assert_eq!(app.config.theme, ThemePreset::Light);
    }

    #[test]
    fn app_with_config_respects_theme() {
        let config = AppConfig {
            theme: ThemePreset::Dracula,
            ..Default::default()
        };
        let app = App::with_config(config);
        assert_eq!(app.theme_preset(), ThemePreset::Dracula);
    }
}
