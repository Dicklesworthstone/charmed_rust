//! Placeholder page for pages not yet implemented.

use bubbletea::{Cmd, Message};
use lipgloss::Position;

use super::PageModel;
use crate::messages::Page;
use crate::theme::Theme;

/// Placeholder page for pages not yet implemented.
pub struct PlaceholderPage {
    page: Page,
}

impl PlaceholderPage {
    /// Create a new placeholder page.
    #[must_use]
    pub const fn new(page: Page) -> Self {
        Self { page }
    }
}

impl PageModel for PlaceholderPage {
    fn update(&mut self, _msg: &Message) -> Option<Cmd> {
        None
    }

    fn view(&self, width: usize, height: usize, theme: &Theme) -> String {
        let title = theme.title_style().render(self.page.name());

        let description = match self.page {
            Page::Services => "Service catalog with filtering and quick actions.",
            Page::Jobs => "Background job monitoring with progress tracking.",
            Page::Logs => "Aggregated log viewer with search and filtering.",
            Page::Docs => "Markdown documentation browser.",
            Page::Files => "File browser with preview pane.",
            Page::Wizard => "Multi-step workflow for service deployment.",
            Page::Settings => "Theme selection and application preferences.",
            Page::Dashboard => "Platform health overview.",
        };

        let content = format!(
            "{}\n\n{}\n\n{}",
            title,
            theme.muted_style().render(description),
            theme.muted_style().italic().render("Coming soon...")
        );

        let boxed = theme.box_style().padding(1).render(&content);

        lipgloss::place(width, height, Position::Center, Position::Center, &boxed)
    }

    fn page(&self) -> Page {
        self.page
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bubbletea::Message;

    #[test]
    fn placeholder_returns_correct_page() {
        let page = PlaceholderPage::new(Page::Services);
        assert_eq!(page.page(), Page::Services);
    }

    #[test]
    fn placeholder_view_renders_content() {
        let page = PlaceholderPage::new(Page::Services);
        let theme = Theme::default();
        let view = page.view(80, 24, &theme);

        // View should not be empty
        assert!(!view.is_empty());
        // View should contain page name
        assert!(view.contains("Services") || view.contains("Service"));
    }

    #[test]
    fn placeholder_update_returns_none() {
        let mut page = PlaceholderPage::new(Page::Dashboard);
        // Use a unit type as the message payload
        let msg = Message::new(());
        let cmd = page.update(&msg);
        assert!(cmd.is_none());
    }

    #[test]
    fn placeholder_default_hints() {
        let page = PlaceholderPage::new(Page::Services);
        let hints = page.hints();
        // Default hints from PageModel trait
        assert!(!hints.is_empty());
    }

    #[test]
    fn placeholder_on_enter_returns_none() {
        let mut page = PlaceholderPage::new(Page::Services);
        let cmd = page.on_enter();
        assert!(cmd.is_none());
    }

    #[test]
    fn placeholder_on_leave_returns_none() {
        let mut page = PlaceholderPage::new(Page::Services);
        let cmd = page.on_leave();
        assert!(cmd.is_none());
    }
}
