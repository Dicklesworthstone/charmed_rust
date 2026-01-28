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
