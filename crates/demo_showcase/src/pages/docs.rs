//! Docs page - split-view markdown documentation browser.
//!
//! This page displays documentation with a split-view layout:
//! - Left pane: document list with selection
//! - Right pane: rendered markdown content with scrollable viewport
//!
//! Features:
//! - Beautiful markdown rendering with glamour
//! - Theme-aware styling (dark/light)
//! - Vim-style navigation (j/k for list, scrolling in content)
//! - Focus switching between list and content (Tab)
//! - Per-document scroll position preservation
//! - Responsive resize handling with content caching
//!
//! Uses `RwLock` for thread-safe interior mutability, enabling SSH mode.

use std::collections::HashMap;
use std::sync::RwLock;

use bubbles::viewport::Viewport;
use bubbletea::{Cmd, KeyMsg, KeyType, Message};
use glamour::{Style as GlamourStyle, TermRenderer};
use lipgloss::{Border, Position, Style};

use super::PageModel;
use crate::assets::docs;
use crate::messages::Page;
use crate::theme::Theme;

// =============================================================================
// Constants
// =============================================================================

/// Width of the document list panel (in characters).
const LIST_WIDTH: usize = 24;

/// Minimum width for the content panel.
const MIN_CONTENT_WIDTH: usize = 40;

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

/// Focus state for the docs page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum DocsFocus {
    /// Document list is focused (default).
    #[default]
    List,
    /// Content viewport is focused.
    Content,
}

/// Docs page showing markdown documentation with split-view layout.
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
    /// Current focus state.
    focus: DocsFocus,
    /// Saved scroll positions per document index.
    scroll_positions: HashMap<usize, usize>,
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
            focus: DocsFocus::List,
            scroll_positions: HashMap::new(),
        }
    }

    /// Get the current document entry.
    fn current_entry(&self) -> Option<&DocEntry> {
        self.entries.get(self.current_index)
    }

    /// Save the current scroll position for the current document.
    fn save_scroll_position(&mut self) {
        let offset = self.viewport.read().unwrap().y_offset();
        self.scroll_positions.insert(self.current_index, offset);
    }

    /// Restore the scroll position for the current document.
    fn restore_scroll_position(&self) {
        if let Some(&offset) = self.scroll_positions.get(&self.current_index) {
            self.viewport.write().unwrap().set_y_offset(offset);
        } else {
            self.viewport.write().unwrap().goto_top();
        }
    }

    /// Select a document by index.
    fn select_doc(&mut self, index: usize) {
        if index < self.entries.len() && index != self.current_index {
            // Save current scroll position
            self.save_scroll_position();
            // Change document
            self.current_index = index;
            *self.needs_render.write().unwrap() = true;
        }
    }

    /// Navigate to the next document in the list.
    fn next_doc(&mut self) {
        if !self.entries.is_empty() {
            self.save_scroll_position();
            self.current_index = (self.current_index + 1) % self.entries.len();
            *self.needs_render.write().unwrap() = true;
            // Restore position for new document (or reset to top)
            self.restore_scroll_position();
        }
    }

    /// Navigate to the previous document in the list.
    fn prev_doc(&mut self) {
        if !self.entries.is_empty() {
            self.save_scroll_position();
            if self.current_index == 0 {
                self.current_index = self.entries.len() - 1;
            } else {
                self.current_index -= 1;
            }
            *self.needs_render.write().unwrap() = true;
            // Restore position for new document (or reset to top)
            self.restore_scroll_position();
        }
    }

    /// Toggle focus between list and content.
    fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            DocsFocus::List => DocsFocus::Content,
            DocsFocus::Content => DocsFocus::List,
        };
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
            .with_word_wrap(width.saturating_sub(4)); // Leave margin for borders

        renderer.render(entry.content)
    }

    /// Render the document list panel.
    fn render_list(&self, theme: &Theme, height: usize) -> String {
        let is_focused = self.focus == DocsFocus::List;

        // Build list items
        let mut lines = Vec::new();

        // Header
        let header_style = if is_focused {
            theme.heading_style()
        } else {
            theme.muted_style()
        };
        lines.push(header_style.render(&format!(
            "{:^width$}",
            "Documents",
            width = LIST_WIDTH - 2
        )));
        lines.push(theme.muted_style().render(&"─".repeat(LIST_WIDTH - 2)));

        // Document entries
        for (i, entry) in self.entries.iter().enumerate() {
            let is_selected = i == self.current_index;

            // Truncate title if needed
            let max_title_len = LIST_WIDTH - 5; // Space for " > " prefix and padding
            let title = if entry.title.len() > max_title_len {
                format!("{}…", &entry.title[..max_title_len - 1])
            } else {
                entry.title.to_string()
            };

            let line = if is_selected {
                let style = if is_focused {
                    theme.selected_style()
                } else {
                    // Selected but not focused - dimmer highlight
                    Style::new()
                        .foreground(theme.text)
                        .background(theme.bg_subtle)
                };
                style.render(&format!(" › {title:<width$}", width = LIST_WIDTH - 5))
            } else {
                let style = theme.muted_style();
                style.render(&format!("   {title:<width$}", width = LIST_WIDTH - 5))
            };

            lines.push(line);
        }

        // Pad remaining height
        let content_lines = lines.len();
        for _ in content_lines..height {
            lines.push(" ".repeat(LIST_WIDTH - 2));
        }

        // Apply border
        let border_style = if is_focused {
            Style::new()
                .foreground(theme.primary)
                .border(Border::rounded())
                .border_foreground(theme.primary)
        } else {
            Style::new()
                .foreground(theme.border)
                .border(Border::rounded())
                .border_foreground(theme.border)
        };

        let content = lines.join("\n");
        #[expect(clippy::cast_possible_truncation)]
        border_style
            .width(LIST_WIDTH as u16)
            .height(height as u16)
            .render(&content)
    }

    /// Render the content panel.
    fn render_content(&self, theme: &Theme, width: usize, height: usize) -> String {
        let is_focused = self.focus == DocsFocus::Content;

        // Get viewport content
        let viewport_content = self.viewport.read().unwrap().view();

        // Render scroll indicator
        let viewport = self.viewport.read().unwrap();
        let total = viewport.total_line_count();
        let visible = viewport.height;
        let offset = viewport.y_offset();
        drop(viewport);

        let percent = if total <= visible {
            100
        } else {
            ((offset as f64 / (total - visible).max(1) as f64) * 100.0) as usize
        };

        // Build header with title and scroll position
        let title = self
            .current_entry()
            .map(|e| e.title)
            .unwrap_or("Documentation");
        let scroll_info = format!("{}%", percent);
        let title_width = width.saturating_sub(scroll_info.len() + 4);
        let truncated_title = if title.len() > title_width {
            format!("{}…", &title[..title_width.saturating_sub(1)])
        } else {
            title.to_string()
        };

        let header_style = if is_focused {
            theme.heading_style()
        } else {
            theme.muted_style()
        };
        let header = format!(
            "{} {}",
            header_style.render(&truncated_title),
            theme.muted_style().render(&scroll_info)
        );

        // Build content
        let separator = theme
            .muted_style()
            .render(&"─".repeat(width.saturating_sub(2)));
        let mut content_lines = Vec::new();
        content_lines.push(header);
        content_lines.push(separator);

        // Add viewport content (already height-limited by viewport)
        for line in viewport_content.lines() {
            content_lines.push(line.to_string());
        }

        // Pad to fill height
        let used_lines = content_lines.len();
        let needed_lines = height.saturating_sub(2); // Account for border
        for _ in used_lines..needed_lines {
            content_lines.push(String::new());
        }

        // Apply border
        let border_style = if is_focused {
            Style::new()
                .border(Border::rounded())
                .border_foreground(theme.primary)
        } else {
            Style::new()
                .border(Border::rounded())
                .border_foreground(theme.border)
        };

        let content = content_lines.join("\n");
        #[expect(clippy::cast_possible_truncation)]
        border_style
            .width(width as u16)
            .height(height as u16)
            .render(&content)
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
                // Tab to switch focus
                KeyType::Tab => {
                    self.toggle_focus();
                    return None;
                }

                // Enter to focus content when in list
                KeyType::Enter if self.focus == DocsFocus::List => {
                    self.focus = DocsFocus::Content;
                    return None;
                }

                // Escape to return to list
                KeyType::Esc if self.focus == DocsFocus::Content => {
                    self.focus = DocsFocus::List;
                    return None;
                }

                // Ctrl+D/U for half-page scrolling (content focused)
                KeyType::CtrlD if self.focus == DocsFocus::Content => {
                    self.viewport.write().unwrap().half_page_down();
                    return None;
                }
                KeyType::CtrlU if self.focus == DocsFocus::Content => {
                    self.viewport.write().unwrap().half_page_up();
                    return None;
                }

                // Vim-style navigation
                KeyType::Runes => {
                    match key.runes.as_slice() {
                        ['j'] => {
                            if self.focus == DocsFocus::List {
                                self.next_doc();
                            } else {
                                self.viewport.write().unwrap().scroll_down(1);
                            }
                            return None;
                        }
                        ['k'] => {
                            if self.focus == DocsFocus::List {
                                self.prev_doc();
                            } else {
                                self.viewport.write().unwrap().scroll_up(1);
                            }
                            return None;
                        }
                        ['g'] if self.focus == DocsFocus::Content => {
                            self.viewport.write().unwrap().goto_top();
                            return None;
                        }
                        ['G'] if self.focus == DocsFocus::Content => {
                            self.viewport.write().unwrap().goto_bottom();
                            return None;
                        }
                        ['l'] | ['h'] if self.focus == DocsFocus::List => {
                            // l/h to switch focus in list mode
                            self.toggle_focus();
                            return None;
                        }
                        _ => {}
                    }
                }

                // Arrow keys
                KeyType::Down => {
                    if self.focus == DocsFocus::List {
                        self.next_doc();
                    } else {
                        self.viewport.write().unwrap().scroll_down(1);
                    }
                    return None;
                }
                KeyType::Up => {
                    if self.focus == DocsFocus::List {
                        self.prev_doc();
                    } else {
                        self.viewport.write().unwrap().scroll_up(1);
                    }
                    return None;
                }
                KeyType::Left if self.focus == DocsFocus::Content => {
                    self.focus = DocsFocus::List;
                    return None;
                }
                KeyType::Right if self.focus == DocsFocus::List => {
                    self.focus = DocsFocus::Content;
                    return None;
                }

                // Page navigation (content only)
                KeyType::PgUp if self.focus == DocsFocus::Content => {
                    self.viewport.write().unwrap().page_up();
                    return None;
                }
                KeyType::PgDown if self.focus == DocsFocus::Content => {
                    self.viewport.write().unwrap().page_down();
                    return None;
                }
                KeyType::Home if self.focus == DocsFocus::Content => {
                    self.viewport.write().unwrap().goto_top();
                    return None;
                }
                KeyType::End if self.focus == DocsFocus::Content => {
                    self.viewport.write().unwrap().goto_bottom();
                    return None;
                }

                _ => {}
            }
        }

        // Delegate to viewport for mouse wheel handling (when content focused)
        if self.focus == DocsFocus::Content {
            self.viewport.write().unwrap().update(msg);
        }

        None
    }

    fn view(&self, width: usize, height: usize, theme: &Theme) -> String {
        // Calculate panel widths
        let list_width = LIST_WIDTH;
        let gap = 1; // Space between panels
        let content_width = width
            .saturating_sub(list_width + gap)
            .max(MIN_CONTENT_WIDTH);
        let actual_content_width = content_width.saturating_sub(2); // Account for borders

        // Calculate content height (account for borders)
        let content_height = height.saturating_sub(2);

        // Check if dimensions or theme changed
        let last_dims = *self.last_dims.read().unwrap();
        let needs_resize = last_dims.0 != actual_content_width || last_dims.1 != content_height;

        let theme_name = theme.preset.name().to_string();
        let last_theme = self.last_theme.read().unwrap().clone();
        let theme_changed = theme_name != last_theme;

        let needs_render = *self.needs_render.read().unwrap();

        if needs_resize || theme_changed || needs_render {
            // Update viewport dimensions (account for header and separator)
            let viewport_height = content_height.saturating_sub(2);
            let mut viewport = self.viewport.write().unwrap();
            viewport.width = actual_content_width;
            viewport.height = viewport_height;

            // Render markdown with glamour
            let rendered = self.render_markdown(theme, actual_content_width);
            viewport.set_content(&rendered);
            *self.rendered_content.write().unwrap() = rendered;

            // Restore scroll position after re-render
            drop(viewport);
            self.restore_scroll_position();

            // Update cache state
            *self.needs_render.write().unwrap() = false;
            *self.last_dims.write().unwrap() = (actual_content_width, content_height);
            *self.last_theme.write().unwrap() = theme_name;
        }

        // Render panels
        let list_panel = self.render_list(theme, height);
        let content_panel = self.render_content(theme, content_width, height);

        // Join horizontally with gap
        lipgloss::join_horizontal(Position::Top, &[&list_panel, " ", &content_panel])
    }

    fn page(&self) -> Page {
        Page::Docs
    }

    fn hints(&self) -> &'static str {
        "j/k nav  Tab focus  g/G top/btm  Enter select"
    }

    fn on_enter(&mut self) -> Option<Cmd> {
        // Mark content for re-rendering when page becomes active
        *self.needs_render.write().unwrap() = true;
        self.focus = DocsFocus::List;
        None
    }

    fn on_leave(&mut self) -> Option<Cmd> {
        // Save scroll position when leaving
        self.save_scroll_position();
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
        assert!(hints.contains("nav"), "Hints should mention navigation");
        assert!(hints.contains("Tab"), "Hints should mention Tab for focus");
    }

    #[test]
    fn docs_page_focus_toggle() {
        let mut page = DocsPage::new();
        assert_eq!(page.focus, DocsFocus::List, "Should start focused on list");

        page.toggle_focus();
        assert_eq!(page.focus, DocsFocus::Content, "Should toggle to content");

        page.toggle_focus();
        assert_eq!(page.focus, DocsFocus::List, "Should toggle back to list");
    }

    #[test]
    fn docs_page_scroll_position_preserved() {
        let mut page = DocsPage::new();

        if page.entries.len() > 1 {
            // Set some content in viewport so y_offset can be set
            // (viewport clamps y_offset to max_y_offset which depends on content)
            let content = (0..100)
                .map(|i| format!("Line {i}"))
                .collect::<Vec<_>>()
                .join("\n");
            page.viewport.write().unwrap().set_content(&content);

            // Set viewport y_offset and save it - this simulates scrolling down
            page.viewport.write().unwrap().set_y_offset(5);
            page.save_scroll_position();
            assert_eq!(page.scroll_positions.get(&0), Some(&5));

            // Navigate away to doc 1 - this saves current position then switches
            page.next_doc();
            assert_eq!(page.current_index, 1);
            // Doc 1 should restore to 0 (no saved position)
            assert_eq!(page.viewport.read().unwrap().y_offset(), 0);

            // Navigate back to doc 0
            page.prev_doc();
            assert_eq!(page.current_index, 0);

            // The saved position for doc 0 should still be 5
            assert_eq!(page.scroll_positions.get(&0), Some(&5));

            // Check scroll position is restored
            assert_eq!(
                page.viewport.read().unwrap().y_offset(),
                5,
                "Scroll position should be restored"
            );
        }
    }

    #[test]
    fn docs_page_select_doc() {
        let mut page = DocsPage::new();

        if page.entries.len() > 1 {
            page.select_doc(1);
            assert_eq!(page.current_index, 1);

            // Selecting same doc should not change anything
            let needs_render = *page.needs_render.read().unwrap();
            page.select_doc(1);
            // Note: needs_render won't change if index is same
        }
    }
}
