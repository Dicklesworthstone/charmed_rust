//! Docs page - markdown viewer with glamour rendering.
//!
//! This page displays documentation rendered with glamour (markdown to terminal)
//! inside a scrollable viewport. It supports:
//!
//! - Beautiful markdown rendering with theme-aware styling
//! - Smooth scrolling through content
//! - Multiple documentation pages with navigation
//! - Responsive resize handling with content caching
//!
//! Uses `RwLock` for thread-safe interior mutability, enabling SSH mode.

use std::sync::RwLock;

use bubbles::viewport::Viewport;
use bubbletea::{Cmd, KeyMsg, KeyType, Message};
use glamour::{Style as GlamourStyle, TermRenderer};

use super::PageModel;
use crate::assets::docs;
use crate::messages::Page;
use crate::theme::Theme;

// =============================================================================
// Documentation State
// =============================================================================

/// A documentation page with title and content.
#[derive(Debug, Clone)]
struct DocEntry {
    /// Display title for navigation.
    title: &'static str,
    /// Raw markdown content.
    content: &'static str,
}

/// Docs page showing markdown documentation with glamour rendering.
pub struct DocsPage {
    /// The viewport for scrollable content (`RwLock` for thread-safe interior mutability).
    viewport: RwLock<Viewport>,
    /// Available documentation pages.
    entries: Vec<DocEntry>,
    /// Currently selected document index.
    current_index: usize,
    /// Cached rendered content (`RwLock` for thread-safe interior mutability).
    rendered_content: RwLock<String>,
    /// Whether content needs to be re-rendered.
    needs_render: RwLock<bool>,
    /// Last known dimensions (for detecting resize).
    last_dims: RwLock<(usize, usize)>,
    /// Last known theme preset (for detecting theme changes).
    last_theme: RwLock<String>,
}

impl DocsPage {
    /// Create a new docs page.
    #[must_use]
    pub fn new() -> Self {
        // Load documentation entries from assets
        let entries: Vec<DocEntry> = docs::ALL
            .iter()
            .map(|(title, content)| DocEntry { title, content })
            .collect();

        // Initialize viewport with mouse support
        let mut viewport = Viewport::new(80, 24);
        viewport.mouse_wheel_enabled = true;
        viewport.mouse_wheel_delta = 3;

        Self {
            viewport: RwLock::new(viewport),
            entries,
            current_index: 0,
            rendered_content: RwLock::new(String::new()),
            needs_render: RwLock::new(true),
            last_dims: RwLock::new((0, 0)),
            last_theme: RwLock::new(String::new()),
        }
    }

    /// Get the current document entry.
    fn current_entry(&self) -> Option<&DocEntry> {
        self.entries.get(self.current_index)
    }

    /// Navigate to the next document.
    fn next_doc(&mut self) {
        if !self.entries.is_empty() {
            self.current_index = (self.current_index + 1) % self.entries.len();
            *self.needs_render.write().unwrap() = true;
            // Reset scroll position for new doc
            self.viewport.write().unwrap().goto_top();
        }
    }

    /// Navigate to the previous document.
    fn prev_doc(&mut self) {
        if !self.entries.is_empty() {
            if self.current_index == 0 {
                self.current_index = self.entries.len() - 1;
            } else {
                self.current_index -= 1;
            }
            *self.needs_render.write().unwrap() = true;
            // Reset scroll position for new doc
            self.viewport.write().unwrap().goto_top();
        }
    }

    /// Render markdown content with glamour.
    fn render_markdown(&self, theme: &Theme, width: usize) -> String {
        let Some(entry) = self.current_entry() else {
            return String::from("No documentation available.");
        };

        // Choose glamour style based on theme
        let glamour_style = if theme.preset.name() == "Light" {
            GlamourStyle::Light
        } else {
            GlamourStyle::Dark
        };

        // Create renderer with appropriate settings
        let renderer = TermRenderer::new()
            .with_style(glamour_style)
            .with_word_wrap(width.saturating_sub(2)); // Leave margin

        renderer.render(entry.content)
    }

    /// Render the document navigation tabs.
    fn render_tabs(&self, theme: &Theme, width: usize) -> String {
        if self.entries.len() <= 1 {
            // No tabs needed for single document
            return String::new();
        }

        let mut tabs = Vec::new();
        for (i, entry) in self.entries.iter().enumerate() {
            let is_active = i == self.current_index;
            let style = if is_active {
                theme.selected_style()
            } else {
                theme.muted_style()
            };
            let tab = format!(" {} ", entry.title);
            tabs.push(style.render(&tab));
        }

        let tab_line = tabs.join("  ");
        let tab_len = lipgloss::visible_width(&tab_line);
        let padding = width.saturating_sub(tab_len);

        format!("{tab_line}{}", " ".repeat(padding))
    }

    /// Render scroll position indicator.
    fn render_scroll_indicator(&self, theme: &Theme, width: usize) -> String {
        let viewport = self.viewport.read().unwrap();
        let total = viewport.total_line_count();
        let visible = viewport.height;
        let offset = viewport.y_offset();

        // Calculate percentage
        let percent = if total <= visible {
            100
        } else {
            ((offset as f64 / (total - visible) as f64) * 100.0) as usize
        };

        let indicator = format!("{}% ({}/{})", percent, offset + 1, total);

        let style = theme.muted_style();

        let indicator_len = indicator.len();
        let padding = width.saturating_sub(indicator_len);

        format!("{}{}", " ".repeat(padding), style.render(&indicator))
    }
}

impl Default for DocsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl PageModel for DocsPage {
    fn update(&mut self, msg: &Message) -> Option<Cmd> {
        // Handle keyboard navigation
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                // Document navigation
                KeyType::Left | KeyType::ShiftTab => {
                    self.prev_doc();
                    return None;
                }
                KeyType::Right | KeyType::Tab => {
                    self.next_doc();
                    return None;
                }

                // Ctrl+D/U for half-page scrolling
                KeyType::CtrlD => {
                    self.viewport.write().unwrap().half_page_down();
                    return None;
                }
                KeyType::CtrlU => {
                    self.viewport.write().unwrap().half_page_up();
                    return None;
                }

                // Vim-style scrolling
                KeyType::Runes => match key.runes.as_slice() {
                    ['j'] => {
                        self.viewport.write().unwrap().scroll_down(1);
                        return None;
                    }
                    ['k'] => {
                        self.viewport.write().unwrap().scroll_up(1);
                        return None;
                    }
                    ['g'] => {
                        self.viewport.write().unwrap().goto_top();
                        return None;
                    }
                    ['G'] => {
                        self.viewport.write().unwrap().goto_bottom();
                        return None;
                    }
                    _ => {}
                },

                // Page navigation
                KeyType::PgUp => {
                    self.viewport.write().unwrap().page_up();
                    return None;
                }
                KeyType::PgDown => {
                    self.viewport.write().unwrap().page_down();
                    return None;
                }
                KeyType::Home => {
                    self.viewport.write().unwrap().goto_top();
                    return None;
                }
                KeyType::End => {
                    self.viewport.write().unwrap().goto_bottom();
                    return None;
                }

                _ => {}
            }
        }

        // Delegate to viewport for mouse wheel handling
        self.viewport.write().unwrap().update(msg);

        None
    }

    fn view(&self, width: usize, height: usize, theme: &Theme) -> String {
        // Reserve space for tabs and scroll indicator
        let has_tabs = self.entries.len() > 1;
        let tab_height = if has_tabs { 2 } else { 0 }; // tabs + separator
        let indicator_height = 1;
        let content_height = height.saturating_sub(tab_height + indicator_height);

        // Check if dimensions or theme changed
        let last_dims = *self.last_dims.read().unwrap();
        let needs_resize = last_dims.0 != width || last_dims.1 != content_height;

        let theme_name = theme.preset.name().to_string();
        let last_theme = self.last_theme.read().unwrap().clone();
        let theme_changed = theme_name != last_theme;

        let needs_render = *self.needs_render.read().unwrap();

        if needs_resize || theme_changed || needs_render {
            // Update viewport dimensions
            let mut viewport = self.viewport.write().unwrap();
            viewport.width = width;
            viewport.height = content_height;

            // Render markdown with glamour
            let rendered = self.render_markdown(theme, width);
            viewport.set_content(&rendered);
            *self.rendered_content.write().unwrap() = rendered;

            // Update cache state
            *self.needs_render.write().unwrap() = false;
            *self.last_dims.write().unwrap() = (width, content_height);
            *self.last_theme.write().unwrap() = theme_name;
        }

        // Build output
        let mut output = String::new();

        // Render tabs if multiple documents
        if has_tabs {
            let tabs = self.render_tabs(theme, width);
            let separator = theme.muted_style().render(&"─".repeat(width));
            output.push_str(&tabs);
            output.push('\n');
            output.push_str(&separator);
            output.push('\n');
        }

        // Render viewport content
        let content = self.viewport.read().unwrap().view();
        output.push_str(&content);
        output.push('\n');

        // Render scroll indicator
        let indicator = self.render_scroll_indicator(theme, width);
        output.push_str(&indicator);

        output
    }

    fn page(&self) -> Page {
        Page::Docs
    }

    fn hints(&self) -> &'static str {
        "j/k scroll  g/G top/bottom  ←/→ docs  PgUp/Dn page"
    }

    fn on_enter(&mut self) -> Option<Cmd> {
        // Mark content for re-rendering when page becomes active
        *self.needs_render.write().unwrap() = true;
        None
    }

    fn on_leave(&mut self) -> Option<Cmd> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docs_page_creates_with_entries() {
        let page = DocsPage::new();
        assert!(
            !page.entries.is_empty(),
            "Should have documentation entries"
        );
        assert_eq!(page.current_index, 0, "Should start at first doc");
    }

    #[test]
    fn docs_page_navigation() {
        let mut page = DocsPage::new();
        let initial = page.current_index;

        if page.entries.len() > 1 {
            page.next_doc();
            assert_eq!(page.current_index, initial + 1);

            page.prev_doc();
            assert_eq!(page.current_index, initial);
        }
    }

    #[test]
    fn docs_page_navigation_wraps() {
        let mut page = DocsPage::new();

        if page.entries.len() > 1 {
            // Go to first
            page.current_index = 0;

            // Previous should wrap to last
            page.prev_doc();
            assert_eq!(page.current_index, page.entries.len() - 1);

            // Next should wrap to first
            page.next_doc();
            assert_eq!(page.current_index, 0);
        }
    }

    #[test]
    fn docs_page_type() {
        let page = DocsPage::new();
        assert_eq!(page.page(), Page::Docs);
    }

    #[test]
    fn docs_page_hints() {
        let page = DocsPage::new();
        let hints = page.hints();
        assert!(hints.contains("scroll"), "Hints should mention scrolling");
        assert!(hints.contains("j/k"), "Hints should mention vim keys");
    }
}
