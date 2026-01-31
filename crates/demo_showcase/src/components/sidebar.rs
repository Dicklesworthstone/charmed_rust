//! Navigation sidebar component with keyboard navigation and filtering.
//!
//! This component provides a polished sidebar experience with:
//! - j/k keyboard navigation
//! - '/' filter mode with instant filtering
//! - Enter to select, Escape to clear filter
//! - Visual feedback for selected and highlighted items

use bubbletea::{Cmd, KeyMsg, KeyType, Message};
use lipgloss::Style;

use crate::messages::{AppMsg, Page};
use crate::theme::{Theme, spacing};

/// Sidebar focus state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SidebarFocus {
    /// Sidebar is not focused, number keys work for navigation.
    #[default]
    Inactive,
    /// Sidebar is focused, j/k navigation active.
    Active,
    /// Filter input is active.
    Filtering,
}

/// Navigation sidebar with keyboard navigation and filtering.
#[derive(Debug, Clone)]
pub struct Sidebar {
    /// Current page selection.
    current_page: Page,
    /// Highlighted item index (for keyboard nav).
    highlighted: usize,
    /// Focus state.
    focus: SidebarFocus,
    /// Filter text.
    filter: String,
    /// Filtered page indices.
    filtered_indices: Vec<usize>,
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl Sidebar {
    /// Create a new sidebar.
    #[must_use]
    pub fn new() -> Self {
        let all_pages = Page::all();
        Self {
            current_page: Page::Dashboard,
            highlighted: 0,
            focus: SidebarFocus::Inactive,
            filter: String::new(),
            filtered_indices: (0..all_pages.len()).collect(),
        }
    }

    /// Set the current page (updates highlight to match).
    pub fn set_current_page(&mut self, page: Page) {
        self.current_page = page;
        // Update highlight to match current page if not filtering
        if self.focus != SidebarFocus::Filtering
            && let Some(idx) = Self::page_index(page)
        {
            self.highlighted = idx;
        }
    }

    /// Get the focus state.
    #[must_use]
    pub const fn focus(&self) -> SidebarFocus {
        self.focus
    }

    /// Set focus state.
    pub fn set_focus(&mut self, focus: SidebarFocus) {
        self.focus = focus;
        if focus == SidebarFocus::Filtering {
            // Clear filter when entering filter mode
            self.filter.clear();
            self.update_filtered_indices();
        }
    }

    /// Toggle sidebar focus (between Inactive and Active).
    pub const fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            SidebarFocus::Inactive => SidebarFocus::Active,
            SidebarFocus::Active | SidebarFocus::Filtering => SidebarFocus::Inactive,
        };
    }

    /// Check if sidebar is focused (Active or Filtering).
    #[must_use]
    pub const fn is_focused(&self) -> bool {
        matches!(self.focus, SidebarFocus::Active | SidebarFocus::Filtering)
    }

    /// Handle key messages.
    ///
    /// Returns `Some(Cmd)` if navigation should occur.
    pub fn update(&mut self, msg: &Message) -> Option<Cmd> {
        let key = msg.downcast_ref::<KeyMsg>()?;

        match self.focus {
            SidebarFocus::Inactive => None,
            SidebarFocus::Active => self.handle_active_key(key),
            SidebarFocus::Filtering => self.handle_filtering_key(key),
        }
    }

    /// Handle keys when sidebar is active (not filtering).
    fn handle_active_key(&mut self, key: &KeyMsg) -> Option<Cmd> {
        match key.key_type {
            KeyType::Up => {
                self.move_highlight(-1);
                None
            }
            KeyType::Down => {
                self.move_highlight(1);
                None
            }
            KeyType::Enter => self.select_highlighted(),
            KeyType::Esc => {
                self.focus = SidebarFocus::Inactive;
                None
            }
            KeyType::Runes => match key.runes.as_slice() {
                ['j'] => {
                    self.move_highlight(1);
                    None
                }
                ['k'] => {
                    self.move_highlight(-1);
                    None
                }
                ['/'] => {
                    self.focus = SidebarFocus::Filtering;
                    self.filter.clear();
                    self.update_filtered_indices();
                    None
                }
                ['g'] => {
                    self.highlighted = 0;
                    None
                }
                ['G'] => {
                    self.highlighted = self.filtered_indices.len().saturating_sub(1);
                    None
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// Handle keys when filtering.
    fn handle_filtering_key(&mut self, key: &KeyMsg) -> Option<Cmd> {
        match key.key_type {
            KeyType::Esc => {
                self.filter.clear();
                self.update_filtered_indices();
                self.focus = SidebarFocus::Active;
                None
            }
            KeyType::Enter => {
                let cmd = self.select_highlighted();
                self.filter.clear();
                self.update_filtered_indices();
                self.focus = SidebarFocus::Active;
                cmd
            }
            KeyType::Backspace => {
                self.filter.pop();
                self.update_filtered_indices();
                None
            }
            KeyType::Up => {
                self.move_highlight(-1);
                None
            }
            KeyType::Down => {
                self.move_highlight(1);
                None
            }
            KeyType::Runes => {
                for c in &key.runes {
                    if c.is_alphanumeric() || *c == ' ' {
                        self.filter.push(*c);
                    }
                }
                self.update_filtered_indices();
                None
            }
            _ => None,
        }
    }

    /// Move highlight by delta (positive = down, negative = up).
    #[expect(clippy::missing_const_for_fn)] // can't be const: Vec access + mutation
    fn move_highlight(&mut self, delta: i32) {
        if self.filtered_indices.is_empty() {
            return;
        }

        let len = self.filtered_indices.len();
        if delta >= 0 {
            #[expect(clippy::cast_sign_loss)]
            let delta_u = delta as usize;
            self.highlighted = (self.highlighted + delta_u) % len;
        } else {
            #[expect(clippy::cast_sign_loss)]
            let delta_u = (-delta) as usize;
            self.highlighted = (self.highlighted + len - (delta_u % len)) % len;
        }
    }

    /// Select the currently highlighted item.
    fn select_highlighted(&self) -> Option<Cmd> {
        self.filtered_indices.get(self.highlighted).map(|&idx| {
            let page = Page::all()[idx];
            Cmd::new(move || Message::new(AppMsg::Navigate(page)))
        })
    }

    /// Update filtered indices based on current filter.
    fn update_filtered_indices(&mut self) {
        let filter_lower = self.filter.to_lowercase();
        self.filtered_indices = Page::all()
            .iter()
            .enumerate()
            .filter(|(_, page)| {
                filter_lower.is_empty() || page.name().to_lowercase().contains(&filter_lower)
            })
            .map(|(i, _)| i)
            .collect();

        // Ensure highlight is in bounds
        if self.highlighted >= self.filtered_indices.len() {
            self.highlighted = self.filtered_indices.len().saturating_sub(1);
        }
    }

    /// Get the index of a page in the list.
    fn page_index(page: Page) -> Option<usize> {
        Page::all().iter().position(|&p| p == page)
    }

    /// Render the sidebar.
    ///
    /// # Arguments
    ///
    /// * `height` - Height in lines
    /// * `theme` - Theme for styling
    /// * `focus_intensity` - Animation intensity for focus state (0.0 to 1.0)
    ///   Used for smooth focus transitions (bd-30md).
    #[must_use]
    pub fn view(&self, height: usize, theme: &Theme, focus_intensity: f64) -> String {
        let sidebar_width = spacing::SIDEBAR_WIDTH;
        let all_pages = Page::all();

        let mut lines: Vec<String> = Vec::new();

        // Filter input (if filtering)
        if self.focus == SidebarFocus::Filtering {
            let filter_line = format!("/{}_", self.filter);
            let filter_styled = theme.info_style().width(sidebar_width).render(&filter_line);
            lines.push(filter_styled);
        }

        // Navigation items
        for (filtered_idx, &page_idx) in self.filtered_indices.iter().enumerate() {
            let page = all_pages[page_idx];
            let is_current = page == self.current_page;
            let is_highlighted = filtered_idx == self.highlighted && self.is_focused();

            let prefix = if is_current { ">" } else { " " };
            let style = Self::item_style(is_current, is_highlighted, theme);
            let label = format!("{} {} {}", prefix, page.icon(), page.name());
            lines.push(style.width(sidebar_width).render(&label));
        }

        let nav = lines.join("\n");

        // Pad to fill height
        let used_lines = lines.len();
        let padding = height.saturating_sub(used_lines);
        let padding_str = "\n".repeat(padding);

        #[expect(clippy::cast_possible_truncation)]
        let height_u16 = height as u16;

        // Apply focus highlight border when focused (bd-30md)
        // The border_right appears when focus_intensity > 0.5
        let base_style = theme
            .sidebar_style()
            .height(height_u16)
            .width(sidebar_width);
        let styled = if focus_intensity > 0.5 {
            base_style
                .border_right(true)
                .border_foreground(theme.primary)
        } else {
            base_style
        };

        styled.render(&format!("{nav}{padding_str}"))
    }

    /// Get the style for an item.
    fn item_style(is_current: bool, is_highlighted: bool, theme: &Theme) -> Style {
        match (is_current, is_highlighted) {
            (_, true) => {
                // Highlighted has highest priority (keyboard focus)
                theme.sidebar_selected_style()
            }
            (true, false) => {
                // Current page but not highlighted (sidebar unfocused)
                theme.sidebar_selected_style()
            }
            (false, false) => theme.sidebar_style(),
        }
    }

    /// Get key hints for current state.
    #[must_use]
    pub const fn hints(&self) -> &'static str {
        match self.focus {
            SidebarFocus::Inactive => "Tab focus  1-7 pages",
            SidebarFocus::Active => "j/k nav  Enter select  / filter  Esc unfocus",
            SidebarFocus::Filtering => "type to filter  Enter select  Esc cancel",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strip_ansi(input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        let mut in_escape = false;
        let mut in_csi = false;
        for c in input.chars() {
            if c == '\x1b' {
                in_escape = true;
                in_csi = false;
                continue;
            }
            if in_escape {
                if c == '[' {
                    in_csi = true;
                    continue;
                }
                if in_csi {
                    // CSI sequences end with '@' through '~'
                    if ('@'..='~').contains(&c) {
                        in_escape = false;
                        in_csi = false;
                    }
                    continue;
                }
                in_escape = false;
                continue;
            }
            result.push(c);
        }
        result
    }

    #[test]
    fn sidebar_default_state() {
        let sidebar = Sidebar::new();
        assert_eq!(sidebar.current_page, Page::Dashboard);
        assert_eq!(sidebar.focus, SidebarFocus::Inactive);
        assert_eq!(sidebar.highlighted, 0);
    }

    #[test]
    fn sidebar_set_current_page() {
        let mut sidebar = Sidebar::new();
        sidebar.set_current_page(Page::Jobs);
        assert_eq!(sidebar.current_page, Page::Jobs);
    }

    #[test]
    fn sidebar_toggle_focus() {
        let mut sidebar = Sidebar::new();
        assert_eq!(sidebar.focus, SidebarFocus::Inactive);

        sidebar.toggle_focus();
        assert_eq!(sidebar.focus, SidebarFocus::Active);

        sidebar.toggle_focus();
        assert_eq!(sidebar.focus, SidebarFocus::Inactive);
    }

    #[test]
    fn sidebar_move_highlight() {
        let mut sidebar = Sidebar::new();
        sidebar.focus = SidebarFocus::Active;
        assert_eq!(sidebar.highlighted, 0);

        sidebar.move_highlight(1);
        assert_eq!(sidebar.highlighted, 1);

        sidebar.move_highlight(-1);
        assert_eq!(sidebar.highlighted, 0);

        // Wrap around
        sidebar.move_highlight(-1);
        assert_eq!(sidebar.highlighted, Page::all().len() - 1);
    }

    #[test]
    fn sidebar_filter_updates_indices() {
        let mut sidebar = Sidebar::new();
        sidebar.focus = SidebarFocus::Filtering;

        // Initially all pages visible
        assert_eq!(sidebar.filtered_indices.len(), Page::all().len());

        // Filter to "dash"
        sidebar.filter = "dash".to_string();
        sidebar.update_filtered_indices();

        // Should only match Dashboard
        assert_eq!(sidebar.filtered_indices.len(), 1);
        assert_eq!(Page::all()[sidebar.filtered_indices[0]], Page::Dashboard);
    }

    #[test]
    fn sidebar_hints_change_with_focus() {
        let mut sidebar = Sidebar::new();

        let inactive_hints = sidebar.hints();
        assert!(inactive_hints.contains("1-7"));

        sidebar.focus = SidebarFocus::Active;
        let active_hints = sidebar.hints();
        assert!(active_hints.contains("j/k"));

        sidebar.focus = SidebarFocus::Filtering;
        let filter_hints = sidebar.hints();
        assert!(filter_hints.contains("filter"));
    }

    #[test]
    fn debug_sidebar_line_count() {
        let sidebar = Sidebar::new();
        let theme = Theme::dark();

        // Render at height 22 (simulating 80x24 terminal)
        let view = sidebar.view(22, &theme, 0.0);

        // Strip ANSI codes for counting
        let stripped = strip_ansi(&view);
        let line_count = stripped.lines().count();

        eprintln!("Sidebar view line count: {line_count}");
        eprintln!("First 5 lines:");
        for (i, line) in stripped.lines().take(5).enumerate() {
            let width = lipgloss::visible_width(line);
            eprintln!("  Line {i}: {line:?} (width={width})");
        }

        // The sidebar should have exactly 22 lines for height 22
        assert_eq!(line_count, 22, "Sidebar should have exactly height lines");
    }

    #[test]
    fn debug_join_horizontal() {
        use lipgloss::{Position, join_horizontal};

        let sidebar = Sidebar::new();
        let theme = Theme::dark();

        // Render at height 22 (simulating 80x24 terminal)
        let sidebar_view = sidebar.view(22, &theme, 0.0);

        // Check visible widths of sidebar lines
        eprintln!("=== Sidebar item widths ===");
        for (i, line) in sidebar_view.lines().take(10).enumerate() {
            let visible_w = lipgloss::visible_width(line);
            eprintln!("  Line {i}: visible_width={visible_w}");
        }

        // Simple content (simulate page content)
        let mut content_lines = vec![];
        for i in 0..22 {
            content_lines.push(format!("Content line {i:02}"));
        }
        let content = content_lines.join("\n");

        // Join them
        let joined = join_horizontal(Position::Top, &[&sidebar_view, &content]);

        // Strip ANSI codes for counting
        let stripped = strip_ansi(&joined);
        let line_count = stripped.lines().count();

        eprintln!("Joined view line count: {line_count}");
        eprintln!("First 10 lines:");
        for (i, line) in stripped.lines().take(10).enumerate() {
            eprintln!("  Line {i}: {line:?}");
        }

        // The joined output should have exactly 22 lines
        assert_eq!(line_count, 22, "Joined view should have exactly 22 lines");

        // Each line should start with sidebar content, not have sidebar content in middle
        let first_line = stripped.lines().next().unwrap();
        assert!(
            first_line.starts_with("> [] Dashboard"),
            "First line should start with Dashboard, got: {first_line:?}"
        );

        let second_line = stripped.lines().nth(1).unwrap();
        assert!(
            second_line.starts_with("  >_ Services"),
            "Second line should start with Services, got: {second_line:?}"
        );
    }
}
