//! Page models for `demo_showcase`.
//!
//! Each page implements the `PageModel` trait, providing a consistent
//! interface for the router to delegate update and view calls.
//!
//! # Responsive Resize Handling
//!
//! Pages receive dimensions through [`PageModel::view`] on every render.
//! When content depends on dimensions (e.g., rendered markdown, column widths),
//! pages should:
//!
//! 1. Track last-known dimensions in their state
//! 2. Compare with incoming dimensions in `view()`
//! 3. Invalidate/regenerate cached content when dimensions change
//!
//! ## Example Pattern
//!
//! ```rust,ignore
//! pub struct MyPage {
//!     viewport: RwLock<Viewport>,
//!     cached_content: RwLock<String>,
//!     last_dims: RwLock<(usize, usize)>,
//! }
//!
//! impl PageModel for MyPage {
//!     fn view(&self, width: usize, height: usize, theme: &Theme) -> String {
//!         let last = *self.last_dims.read().unwrap();
//!         let needs_resize = last.0 != width || last.1 != height;
//!
//!         if needs_resize {
//!             // Update viewport dimensions
//!             let mut vp = self.viewport.write().unwrap();
//!             vp.width = width;
//!             vp.height = height;
//!
//!             // Regenerate cached content
//!             *self.cached_content.write().unwrap() = self.render_content(width);
//!             *self.last_dims.write().unwrap() = (width, height);
//!         }
//!
//!         // Use cached content...
//!     }
//! }
//! ```
//!
//! See [`logs::LogsPage`] for a complete example of responsive resize handling.

mod dashboard;
mod docs;
mod files;
mod jobs;
mod logs;
mod placeholder;
mod settings;
mod wizard;

pub use dashboard::DashboardPage;
pub use docs::DocsPage;
pub use files::FilesPage;
pub use jobs::JobsPage;
pub use logs::LogsPage;
pub use placeholder::PlaceholderPage;
pub use settings::SettingsPage;
pub use wizard::WizardPage;

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
    ///
    /// ## Resize Handling
    ///
    /// This method is called on every render frame. When dimensions change
    /// (e.g., terminal resize), pages should:
    ///
    /// - Update viewport/component dimensions
    /// - Invalidate cached rendered content that depends on width
    /// - Re-render content if necessary (e.g., markdown, wrapped text)
    ///
    /// Pages should compare incoming dimensions against cached values to
    /// avoid unnecessary regeneration on every frame.
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
    pub logs: LogsPage,
    pub docs: DocsPage,
    pub files: FilesPage,
    pub wizard: WizardPage,
    pub settings: SettingsPage,
}

impl Default for Pages {
    fn default() -> Self {
        Self {
            dashboard: DashboardPage::new(),
            services: PlaceholderPage::new(Page::Services),
            jobs: JobsPage::new(),
            logs: LogsPage::new(),
            docs: DocsPage::new(),
            files: FilesPage::new(),
            wizard: WizardPage::new(),
            settings: SettingsPage::new(),
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
            Page::Files => &self.files,
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
            Page::Files => &mut self.files,
            Page::Wizard => &mut self.wizard,
            Page::Settings => &mut self.settings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pages_default_creates_all() {
        let pages = Pages::default();
        // All pages should be accessible
        assert_eq!(pages.get(Page::Dashboard).page(), Page::Dashboard);
        assert_eq!(pages.get(Page::Services).page(), Page::Services);
        assert_eq!(pages.get(Page::Jobs).page(), Page::Jobs);
        assert_eq!(pages.get(Page::Logs).page(), Page::Logs);
        assert_eq!(pages.get(Page::Docs).page(), Page::Docs);
        assert_eq!(pages.get(Page::Files).page(), Page::Files);
        assert_eq!(pages.get(Page::Wizard).page(), Page::Wizard);
        assert_eq!(pages.get(Page::Settings).page(), Page::Settings);
    }

    #[test]
    fn pages_get_returns_correct_page() {
        let pages = Pages::default();

        for page_type in Page::all() {
            let page = pages.get(page_type);
            assert_eq!(page.page(), page_type);
        }
    }

    #[test]
    fn pages_get_mut_allows_modification() {
        let mut pages = Pages::default();
        let theme = Theme::default();

        // Should be able to get mutable references and render
        for page_type in Page::all() {
            let page = pages.get_mut(page_type);
            // Verify view() doesn't panic
            let view = page.view(80, 24, &theme);
            assert!(!view.is_empty());
        }
    }

    #[test]
    fn all_pages_have_hints() {
        let pages = Pages::default();

        for page_type in Page::all() {
            let hints = pages.get(page_type).hints();
            // All pages should have some hints
            assert!(!hints.is_empty(), "Page {:?} should have hints", page_type);
        }
    }

    #[test]
    fn all_pages_render_without_panic() {
        let pages = Pages::default();
        let theme = Theme::default();

        // Test various dimensions to catch layout edge cases
        let dimensions = [(80, 24), (120, 40), (40, 10), (200, 60)];

        for (width, height) in dimensions {
            for page_type in Page::all() {
                // This should not panic
                let view = pages.get(page_type).view(width, height, &theme);
                assert!(
                    !view.is_empty(),
                    "Page {:?} rendered empty at {}x{}",
                    page_type,
                    width,
                    height
                );
            }
        }
    }
}
