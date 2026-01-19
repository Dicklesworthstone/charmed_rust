//! Multi-Component Dashboard Example
//!
//! This example demonstrates:
//! - Combining multiple bubbles components into one view
//! - Focus switching between components (Tab key)
//! - Coordinated state updates
//! - Layout composition with lipgloss
//!
//! Layout:
//! ┌──────────────────────────────────────────┐
//! │ Dashboard Title              Status: OK  │
//! ├──────────┬───────────────────────────────┤
//! │ Sidebar  │                               │
//! │ • Item 1 │   Main content area           │
//! │ • Item 2 │   (scrollable viewport)       │
//! │ • Item 3 │                               │
//! ├──────────┴───────────────────────────────┤
//! │ Press Tab to switch focus, q to quit     │
//! └──────────────────────────────────────────┘
//!
//! Run with: `cargo run -p example-multi-component`

#![forbid(unsafe_code)]

use bubbles::viewport::Viewport;
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Program, quit};
use lipgloss::Style;

/// Menu items for the sidebar.
const MENU_ITEMS: &[&str] = &["Dashboard", "Analytics", "Reports", "Settings", "Help"];

/// Content for each menu item.
const CONTENT: &[&str] = &[
    // Dashboard
    r#"Welcome to the Dashboard!

This is your central hub for monitoring application status.

Key Metrics:
• Active Users: 1,234
• System Load: 45%
• Memory Usage: 2.1 GB / 8 GB
• Uptime: 7 days, 3 hours

Recent Activity:
• User login from 192.168.1.100
• Config updated by admin
• Backup completed successfully
• New user registered"#,
    // Analytics
    r#"Analytics Overview

Traffic Summary (Last 7 Days):
─────────────────────────────────
Mon: ████████████░░░░░░░░  60%
Tue: ██████████████░░░░░░  70%
Wed: ████████████████████  100%
Thu: ████████████████░░░░  80%
Fri: ██████████████░░░░░░  70%
Sat: ████████░░░░░░░░░░░░  40%
Sun: ██████░░░░░░░░░░░░░░  30%

Top Pages:
1. /home - 45,231 views
2. /products - 23,456 views
3. /about - 12,345 views
4. /contact - 8,901 views"#,
    // Reports
    r#"Available Reports

Generated Reports:
• Monthly Summary - Jan 2026
• Quarterly Review - Q4 2025
• Annual Report - 2025

Scheduled Reports:
• Weekly digest - Every Monday 9:00 AM
• Daily stats - Every day 6:00 AM
• Alert summary - Real-time

Export Options:
• PDF format
• CSV spreadsheet
• JSON data"#,
    // Settings
    r#"Application Settings

Display:
• Theme: Dark
• Language: English
• Timezone: UTC-5

Notifications:
• Email alerts: Enabled
• Push notifications: Disabled
• Weekly digest: Enabled

Security:
• Two-factor auth: Enabled
• Session timeout: 30 minutes
• IP whitelist: Disabled

Press Enter to edit settings"#,
    // Help
    r#"Help & Documentation

Keyboard Shortcuts:
─────────────────────
Tab      - Switch between panels
j/k      - Navigate menu / scroll content
Enter    - Select menu item
Esc/q    - Quit application

Getting Started:
1. Use Tab to switch between the sidebar and content area
2. In the sidebar, use j/k or arrows to navigate
3. Press Enter to view content for the selected item
4. In the content area, scroll with j/k or arrows

Need more help?
Visit https://charm.sh/docs or press F1"#,
];

/// Which panel is currently focused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Sidebar,
    Content,
}

impl Focus {
    fn toggle(self) -> Self {
        match self {
            Self::Sidebar => Self::Content,
            Self::Content => Self::Sidebar,
        }
    }
}

/// The main application model.
#[derive(bubbletea::Model)]
struct App {
    viewport: Viewport,
    focus: Focus,
    selected: usize,
    status: &'static str,
}

impl App {
    /// Create a new dashboard app.
    fn new() -> Self {
        let mut viewport = Viewport::new(50, 15);
        viewport.set_content(CONTENT[0]);

        Self {
            viewport,
            focus: Focus::Sidebar,
            selected: 0,
            status: "OK",
        }
    }

    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::Tab => {
                    self.focus = self.focus.toggle();
                }
                KeyType::Runes => {
                    if let Some(&ch) = key.runes.first() {
                        match ch {
                            'q' | 'Q' => return Some(quit()),
                            'j' if self.focus == Focus::Sidebar => self.move_down(),
                            'k' if self.focus == Focus::Sidebar => self.move_up(),
                            _ => {}
                        }
                    }
                }
                KeyType::Up if self.focus == Focus::Sidebar => self.move_up(),
                KeyType::Down if self.focus == Focus::Sidebar => self.move_down(),
                KeyType::Enter if self.focus == Focus::Sidebar => {
                    self.viewport.set_content(CONTENT[self.selected]);
                    self.viewport.goto_top();
                }
                KeyType::CtrlC | KeyType::Esc => return Some(quit()),
                _ => {}
            }
        }

        // Forward scrolling to viewport when content is focused
        if self.focus == Focus::Content {
            self.viewport.update(&msg);
        }

        None
    }

    fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.viewport.set_content(CONTENT[self.selected]);
            self.viewport.goto_top();
        }
    }

    fn move_down(&mut self) {
        if self.selected < MENU_ITEMS.len() - 1 {
            self.selected += 1;
            self.viewport.set_content(CONTENT[self.selected]);
            self.viewport.goto_top();
        }
    }

    fn view(&self) -> String {
        let mut output = String::new();

        // Header
        let header = self.render_header();
        output.push_str(&header);
        output.push('\n');

        // Main content: sidebar + content
        let sidebar = self.render_sidebar();
        let content = self.render_content();

        // Combine sidebar and content side by side
        let sidebar_lines: Vec<&str> = sidebar.lines().collect();
        let content_lines: Vec<&str> = content.lines().collect();
        let max_lines = sidebar_lines.len().max(content_lines.len());

        for i in 0..max_lines {
            let sidebar_line = sidebar_lines.get(i).unwrap_or(&"");
            let content_line = content_lines.get(i).unwrap_or(&"");
            output.push_str(&format!("  {} │ {}\n", sidebar_line, content_line));
        }

        // Footer
        let footer = self.render_footer();
        output.push_str(&footer);

        output
    }

    fn render_header(&self) -> String {
        let title_style = Style::new().bold().foreground("212");
        let status_style = if self.status == "OK" {
            Style::new().foreground("82")
        } else {
            Style::new().foreground("196")
        };

        format!(
            "\n  {}                    Status: {}\n  {}",
            title_style.render("Dashboard"),
            status_style.render(self.status),
            "─".repeat(60)
        )
    }

    fn render_sidebar(&self) -> String {
        let mut output = String::new();
        let focused = self.focus == Focus::Sidebar;

        let border_style = if focused {
            Style::new().foreground("212")
        } else {
            Style::new().foreground("240")
        };

        let title = if focused { "◆ Menu" } else { "○ Menu" };
        output.push_str(&format!("{}\n", border_style.render(title)));

        for (i, item) in MENU_ITEMS.iter().enumerate() {
            let is_selected = i == self.selected;
            let prefix = if is_selected { "▸ " } else { "  " };

            let item_style = if is_selected && focused {
                Style::new().foreground("212").bold()
            } else if is_selected {
                Style::new().foreground("252")
            } else {
                Style::new().foreground("245")
            };

            output.push_str(&format!("{}{}\n", prefix, item_style.render(item)));
        }

        // Pad to consistent height
        for _ in MENU_ITEMS.len()..12 {
            output.push_str("          \n");
        }

        output
    }

    fn render_content(&self) -> String {
        let mut output = String::new();
        let focused = self.focus == Focus::Content;

        let border_style = if focused {
            Style::new().foreground("212")
        } else {
            Style::new().foreground("240")
        };

        let title = if focused {
            "◆ Content"
        } else {
            "○ Content"
        };
        output.push_str(&format!("{}\n", border_style.render(title)));

        // Render viewport content
        let content = self.viewport.view();
        for line in content.lines() {
            output.push_str(line);
            output.push('\n');
        }

        output
    }

    fn render_footer(&self) -> String {
        let help_style = Style::new().foreground("241");
        let focus_indicator = match self.focus {
            Focus::Sidebar => "[Sidebar]",
            Focus::Content => "[Content]",
        };

        format!(
            "  {}\n  {} Tab: switch focus  j/k: navigate  Enter: select  q: quit\n",
            "─".repeat(60),
            help_style.render(&format!("{} ", focus_indicator))
        )
    }
}

fn main() -> anyhow::Result<()> {
    Program::new(App::new()).with_alt_screen().run()?;

    println!("Goodbye!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a key message for a character.
    fn key_char(ch: char) -> Message {
        Message::new(KeyMsg {
            key_type: KeyType::Runes,
            runes: vec![ch],
            alt: false,
            paste: false,
        })
    }

    /// Create a key message for a special key.
    fn key_type(kt: KeyType) -> Message {
        Message::new(KeyMsg {
            key_type: kt,
            runes: vec![],
            alt: false,
            paste: false,
        })
    }

    #[test]
    fn test_initial_state() {
        let app = App::new();
        assert_eq!(app.focus, Focus::Sidebar);
        assert_eq!(app.selected, 0);
        assert_eq!(app.status, "OK");
    }

    #[test]
    fn test_init_returns_none() {
        let app = App::new();
        assert!(app.init().is_none());
    }

    #[test]
    fn test_focus_toggle() {
        let focus = Focus::Sidebar;
        assert_eq!(focus.toggle(), Focus::Content);
        assert_eq!(focus.toggle().toggle(), Focus::Sidebar);
    }

    #[test]
    fn test_tab_switches_focus() {
        let mut app = App::new();
        assert_eq!(app.focus, Focus::Sidebar);

        app.update(key_type(KeyType::Tab));
        assert_eq!(app.focus, Focus::Content);

        app.update(key_type(KeyType::Tab));
        assert_eq!(app.focus, Focus::Sidebar);
    }

    #[test]
    fn test_move_down_j_in_sidebar() {
        let mut app = App::new();
        assert_eq!(app.focus, Focus::Sidebar);
        assert_eq!(app.selected, 0);

        app.update(key_char('j'));
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn test_move_down_arrow_in_sidebar() {
        let mut app = App::new();
        assert_eq!(app.selected, 0);

        app.update(key_type(KeyType::Down));
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn test_move_up_k_in_sidebar() {
        let mut app = App::new();
        app.selected = 2;

        app.update(key_char('k'));
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn test_move_up_arrow_in_sidebar() {
        let mut app = App::new();
        app.selected = 2;

        app.update(key_type(KeyType::Up));
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn test_navigation_bounded_top() {
        let mut app = App::new();
        assert_eq!(app.selected, 0);

        app.update(key_char('k')); // Try to go above
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_navigation_bounded_bottom() {
        let mut app = App::new();
        app.selected = MENU_ITEMS.len() - 1;

        app.update(key_char('j')); // Try to go below
        assert_eq!(app.selected, MENU_ITEMS.len() - 1);
    }

    #[test]
    fn test_j_ignored_when_content_focused() {
        let mut app = App::new();
        app.focus = Focus::Content;
        let initial_selected = app.selected;

        app.update(key_char('j'));
        assert_eq!(app.selected, initial_selected); // Unchanged
    }

    #[test]
    fn test_enter_updates_viewport_content() {
        let mut app = App::new();
        app.selected = 1; // Analytics

        app.update(key_type(KeyType::Enter));
        // Viewport content should be updated (we can't easily check content,
        // but we can verify y_offset is reset)
        assert_eq!(app.viewport.y_offset(), 0);
    }

    #[test]
    fn test_quit_q() {
        let mut app = App::new();
        let cmd = app.update(key_char('q'));
        assert!(cmd.is_some());
    }

    #[test]
    fn test_quit_capital_q() {
        let mut app = App::new();
        let cmd = app.update(key_char('Q'));
        assert!(cmd.is_some());
    }

    #[test]
    fn test_quit_ctrl_c() {
        let mut app = App::new();
        let cmd = app.update(key_type(KeyType::CtrlC));
        assert!(cmd.is_some());
    }

    #[test]
    fn test_quit_esc() {
        let mut app = App::new();
        let cmd = app.update(key_type(KeyType::Esc));
        assert!(cmd.is_some());
    }

    #[test]
    fn test_view_contains_header() {
        let app = App::new();
        let view = app.view();
        assert!(view.contains("Dashboard"));
    }

    #[test]
    fn test_view_contains_status() {
        let app = App::new();
        let view = app.view();
        assert!(view.contains("OK"));
    }

    #[test]
    fn test_view_contains_menu_items() {
        let app = App::new();
        let view = app.view();
        for item in MENU_ITEMS {
            assert!(view.contains(item), "View should contain menu item: {}", item);
        }
    }

    #[test]
    fn test_view_contains_help_text() {
        let app = App::new();
        let view = app.view();
        assert!(view.contains("Tab"));
        assert!(view.contains("quit"));
    }

    #[test]
    fn test_render_header() {
        let app = App::new();
        let header = app.render_header();
        assert!(header.contains("Dashboard"));
        assert!(header.contains("OK"));
    }

    #[test]
    fn test_render_sidebar_shows_focused_indicator() {
        let mut app = App::new();
        app.focus = Focus::Sidebar;
        let sidebar = app.render_sidebar();
        assert!(sidebar.contains("◆")); // Focused indicator
    }

    #[test]
    fn test_render_sidebar_shows_unfocused_indicator() {
        let mut app = App::new();
        app.focus = Focus::Content;
        let sidebar = app.render_sidebar();
        assert!(sidebar.contains("○")); // Unfocused indicator
    }

    #[test]
    fn test_render_footer_shows_current_focus() {
        let mut app = App::new();
        app.focus = Focus::Sidebar;
        let footer = app.render_footer();
        assert!(footer.contains("[Sidebar]"));

        app.focus = Focus::Content;
        let footer = app.render_footer();
        assert!(footer.contains("[Content]"));
    }
}
