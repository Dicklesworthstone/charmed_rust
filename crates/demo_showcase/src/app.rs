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

use crate::components::{StatusLevel, banner};
use crate::messages::{AppMsg, Notification, NotificationMsg, Page};
use crate::pages::Pages;
use crate::theme::{Theme, ThemePreset, spacing};

/// Application configuration.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields will be used as features are implemented
pub struct AppConfig {
    /// Initial theme preset.
    pub theme: ThemePreset,
    /// Whether animations are enabled.
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
    /// Whether sidebar is visible.
    sidebar_visible: bool,
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
            sidebar_visible: true,
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

    /// Navigate to a new page.
    fn navigate(&mut self, page: Page) -> Option<Cmd> {
        if page == self.current_page {
            return None;
        }

        // Leave current page
        let leave_cmd = self.pages.get_mut(self.current_page).on_leave();

        // Enter new page
        self.current_page = page;
        let enter_cmd = self.pages.get_mut(page).on_enter();

        // Combine commands
        batch(vec![leave_cmd, enter_cmd])
    }

    /// Handle global keyboard shortcuts.
    fn handle_global_key(&mut self, key: &KeyMsg) -> Option<Cmd> {
        match key.key_type {
            KeyType::CtrlC | KeyType::Esc if !self.show_help => return Some(quit()),
            KeyType::Esc if self.show_help => {
                self.show_help = false;
                return None;
            }
            KeyType::Runes => match key.runes.as_slice() {
                ['q'] if !self.show_help => return Some(quit()),
                ['?'] => {
                    self.show_help = !self.show_help;
                    return None;
                }
                ['['] => {
                    self.sidebar_visible = !self.sidebar_visible;
                    return None;
                }
                [c] if !self.show_help => {
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

    /// Render the sidebar.
    fn render_sidebar(&self, height: usize) -> String {
        let sidebar_width = spacing::SIDEBAR_WIDTH;

        let items: Vec<String> = Page::all()
            .iter()
            .map(|&page| {
                let prefix = if page == self.current_page { ">" } else { " " };
                let style = if page == self.current_page {
                    self.theme.sidebar_selected_style()
                } else {
                    self.theme.sidebar_style()
                };
                let label = format!("{} {} {}", prefix, page.icon(), page.name());
                style.width(sidebar_width).render(&label)
            })
            .collect();

        let nav = items.join("\n");

        // Pad to fill height
        let nav_lines = items.len();
        let padding = height.saturating_sub(nav_lines);
        let padding_str = "\n".repeat(padding);

        #[expect(clippy::cast_possible_truncation)]
        let height_u16 = height as u16;

        self.theme
            .sidebar_style()
            .height(height_u16)
            .width(sidebar_width)
            .render(&format!("{nav}{padding_str}"))
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
    fn render_help(&self) -> String {
        let help_text = [
            "",
            "  Keyboard Shortcuts",
            "  ------------------",
            "",
            "  Navigation",
            "  1-7        Jump to page",
            "  [          Toggle sidebar",
            "",
            "  Global",
            "  ?          Toggle this help",
            "  q / Esc    Quit",
            "",
            "  Page-specific shortcuts",
            "  shown in footer when active",
            "",
            "  Press ? or Esc to close",
        ];

        let box_width: usize = 36;
        let box_height = help_text.len() + 2;
        let start_x = self.width.saturating_sub(box_width) / 2;
        let start_y = self.height.saturating_sub(box_height) / 2;

        let mut lines: Vec<String> = Vec::new();

        // Top padding
        for _ in 0..start_y {
            lines.push(String::new());
        }

        #[expect(clippy::cast_possible_truncation)]
        let box_width_u16 = box_width as u16;

        // Use modal style for the help overlay
        let modal_style = self.theme.modal_style();

        // Top border with title
        lines.push(format!(
            "{}{}",
            " ".repeat(start_x),
            modal_style.width(box_width_u16).render(&format!(
                "{}Help{}",
                " ".repeat(14),
                " ".repeat(14)
            ))
        ));

        // Content
        for text in &help_text {
            let padded = format!("{:width$}", text, width = box_width - 4);
            lines.push(format!(
                "{}{}",
                " ".repeat(start_x),
                Style::new()
                    .foreground(self.theme.text)
                    .background(self.theme.bg_highlight)
                    .render(&format!("  {padded}  "))
            ));
        }

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
                    self.config.animations = !self.config.animations;
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
}
