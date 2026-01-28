//! Page models for `demo_showcase`.
//!
//! Each page implements the `PageModel` trait, providing a consistent
//! interface for the router to delegate update and view calls.

mod dashboard;
mod jobs;
mod placeholder;

pub use dashboard::DashboardPage;
pub use jobs::JobsPage;
pub use placeholder::PlaceholderPage;

use bubbletea::{Cmd, Message};

use crate::messages::Page;
use crate::theme::Theme;

/// Trait for page models that can be routed to.
///
/// This trait provides a consistent interface for the App router
/// to delegate update and view calls to individual pages.
pub trait PageModel {
    /// Handle a message, returning an optional command.
    fn update(&mut self, msg: &Message) -> Option<Cmd>;

    /// Render the page content.
    ///
    /// The width and height are the available content area
    /// (excluding app chrome like header/sidebar/footer).
    fn view(&self, width: usize, height: usize, theme: &Theme) -> String;

    /// Get the page identifier.
    #[allow(dead_code)] // Will be used for routing/debugging
    fn page(&self) -> Page;

    /// Get context-sensitive key hints for the footer.
    fn hints(&self) -> &'static str {
        "j/k navigate  Enter select"
    }

    /// Called when the page becomes active (navigated to).
    fn on_enter(&mut self) -> Option<Cmd> {
        None
    }

    /// Called when leaving the page (navigating away).
    fn on_leave(&mut self) -> Option<Cmd> {
        None
    }
}

/// Container for all page models.
///
/// This allows the router to hold all pages and delegate to the active one.
pub struct Pages {
    pub dashboard: DashboardPage,
    pub services: PlaceholderPage,
    pub jobs: JobsPage,
    pub logs: PlaceholderPage,
    pub docs: PlaceholderPage,
    pub wizard: PlaceholderPage,
    pub settings: PlaceholderPage,
}

impl Default for Pages {
    fn default() -> Self {
        Self {
            dashboard: DashboardPage::new(),
            services: PlaceholderPage::new(Page::Services),
            jobs: JobsPage::new(),
            logs: PlaceholderPage::new(Page::Logs),
            docs: PlaceholderPage::new(Page::Docs),
            wizard: PlaceholderPage::new(Page::Wizard),
            settings: PlaceholderPage::new(Page::Settings),
        }
    }
}

impl Pages {
    /// Get a reference to the active page model.
    pub fn get(&self, page: Page) -> &dyn PageModel {
        match page {
            Page::Dashboard => &self.dashboard,
            Page::Services => &self.services,
            Page::Jobs => &self.jobs,
            Page::Logs => &self.logs,
            Page::Docs => &self.docs,
            Page::Wizard => &self.wizard,
            Page::Settings => &self.settings,
        }
    }

    /// Get a mutable reference to the active page model.
    pub fn get_mut(&mut self, page: Page) -> &mut dyn PageModel {
        match page {
            Page::Dashboard => &mut self.dashboard,
            Page::Services => &mut self.services,
            Page::Jobs => &mut self.jobs,
            Page::Logs => &mut self.logs,
            Page::Docs => &mut self.docs,
            Page::Wizard => &mut self.wizard,
            Page::Settings => &mut self.settings,
        }
    }
}
