//! Main application model and routing.
//!
//! The `App` struct is the top-level model that handles:
//! - Global state (theme, toggles, current page)
//! - Message routing to page models
//! - App chrome rendering (header, sidebar, footer)

use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, WindowSizeMsg, batch, quit};
use lipgloss::{Position, Style};

use crate::messages::{AppMsg, Page};
use crate::pages::Pages;
use crate::theme::{Theme, ThemePreset};

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
        }
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
        let sidebar_width: u16 = 12;

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

        let header_content = format!("{title}  {status}");

        #[expect(clippy::cast_possible_truncation)]
        let width_u16 = self.width as u16;

        Style::new()
            .background(self.theme.bg_subtle)
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

        self.theme.muted_style().width(width_u16).render(&hints)
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

        // Top border
        lines.push(format!(
            "{}{}",
            " ".repeat(start_x),
            self.theme
                .box_focused_style()
                .width(box_width_u16)
                .render(&format!("{}Help{}", " ".repeat(14), " ".repeat(14)))
        ));

        // Content
        for text in &help_text {
            let padded = format!("{:width$}", text, width = box_width - 4);
            lines.push(format!(
                "{}{}",
                " ".repeat(start_x),
                Style::new()
                    .foreground(self.theme.text)
                    .render(&format!("  {padded}  "))
            ));
        }

        lines.join("\n")
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for App {
    fn init(&self) -> Option<Cmd> {
        Some(bubbletea::window_size())
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

        // Calculate content area
        let header_height = 1;
        let footer_height = 1;
        let content_height = self.height.saturating_sub(header_height + footer_height);

        let (sidebar, content_width) = if self.sidebar_visible {
            let sidebar = self.render_sidebar(content_height);
            let sidebar_width = 12;
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

        lipgloss::join_vertical(Position::Left, &[&header, &main_area, &footer])
    }
}
