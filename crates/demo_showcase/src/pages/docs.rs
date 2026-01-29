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
    /// Search input is focused.
    Search,
}

/// A search match with line number and character offset.
#[derive(Debug, Clone)]
struct SearchMatch {
    /// Line number in the content (0-indexed).
    line: usize,
    /// Character offset within the line.
    #[allow(dead_code)] // Reserved for future highlight overlay
    offset: usize,
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
    /// Search query string.
    search_query: String,
    /// Search matches in current document.
    search_matches: Vec<SearchMatch>,
    /// Current match index (0-indexed).
    current_match: usize,
    /// Previous focus state (to restore when exiting search).
    prev_focus: DocsFocus,
    /// Whether syntax highlighting is enabled.
    syntax_highlighting: bool,
    /// Whether to show line numbers in code blocks.
    line_numbers: bool,
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
            search_query: String::new(),
            search_matches: Vec::new(),
            current_match: 0,
            prev_focus: DocsFocus::List,
            syntax_highlighting: true, // Enabled by default (respects compile-time feature)
            line_numbers: false,       // Disabled by default for cleaner look
        }
    }

    /// Toggle syntax highlighting on/off.
    pub fn toggle_syntax_highlighting(&mut self) {
        self.syntax_highlighting = !self.syntax_highlighting;
        *self.needs_render.write().unwrap() = true;
    }

    /// Toggle line numbers on/off.
    pub fn toggle_line_numbers(&mut self) {
        self.line_numbers = !self.line_numbers;
        *self.needs_render.write().unwrap() = true;
    }

    /// Check if syntax highlighting is enabled.
    #[must_use]
    pub const fn syntax_highlighting_enabled(&self) -> bool {
        self.syntax_highlighting
    }

    /// Check if line numbers are enabled.
    #[must_use]
    pub const fn line_numbers_enabled(&self) -> bool {
        self.line_numbers
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
    #[allow(dead_code)]
    fn select_doc(&mut self, index: usize) {
        if index < self.entries.len() && index != self.current_index {
            // Save current scroll position
            self.save_scroll_position();
            // Change document
            self.current_index = index;
            *self.needs_render.write().unwrap() = true;
            // Clear search state when changing documents
            self.search_query.clear();
            self.search_matches.clear();
            self.current_match = 0;
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
            DocsFocus::Search => DocsFocus::Content, // Exit search to content
        };
    }

    /// Enter search mode.
    fn enter_search(&mut self) {
        self.prev_focus = self.focus;
        self.focus = DocsFocus::Search;
        self.search_query.clear();
        self.search_matches.clear();
        self.current_match = 0;
    }

    /// Exit search mode.
    fn exit_search(&mut self) {
        self.focus = self.prev_focus;
        // Keep search query and matches visible for n/N navigation
    }

    /// Update search matches based on current query.
    fn update_search_matches(&mut self) {
        self.search_matches.clear();
        self.current_match = 0;

        let query = self.search_query.to_lowercase();
        if query.is_empty() {
            return;
        }

        // Search in raw markdown content
        let Some(entry) = self.current_entry() else {
            return;
        };

        let content_lower = entry.content.to_lowercase();
        for (line_idx, line) in content_lower.lines().enumerate() {
            let mut offset = 0;
            while let Some(pos) = line[offset..].find(&query) {
                self.search_matches.push(SearchMatch {
                    line: line_idx,
                    offset: offset + pos,
                });
                offset += pos + query.len();
            }
        }
    }

    /// Navigate to the current search match.
    fn goto_current_match(&self) {
        if let Some(m) = self.search_matches.get(self.current_match) {
            // Scroll viewport to show the match line
            // Center the match in the viewport if possible
            let viewport = self.viewport.read().unwrap();
            let visible_lines = viewport.height;
            drop(viewport);

            let target_offset = if m.line > visible_lines / 2 {
                m.line.saturating_sub(visible_lines / 2)
            } else {
                0
            };
            self.viewport.write().unwrap().set_y_offset(target_offset);
        }
    }

    /// Go to the next search match.
    fn next_match(&mut self) {
        if !self.search_matches.is_empty() {
            self.current_match = (self.current_match + 1) % self.search_matches.len();
            self.goto_current_match();
        }
    }

    /// Go to the previous search match.
    fn prev_match(&mut self) {
        if !self.search_matches.is_empty() {
            if self.current_match == 0 {
                self.current_match = self.search_matches.len() - 1;
            } else {
                self.current_match -= 1;
            }
            self.goto_current_match();
        }
    }

    /// Get search status string.
    fn search_status(&self) -> String {
        if self.search_query.is_empty() {
            String::new()
        } else if self.search_matches.is_empty() {
            "No matches".to_string()
        } else {
            format!("{}/{}", self.current_match + 1, self.search_matches.len())
        }
    }

    /// Render markdown content with glamour.
    fn render_markdown(&self, theme: &Theme, width: usize) -> String {
        let Some(entry) = self.current_entry() else {
            return String::from("No documentation available.");
        };

        // Choose glamour style based on theme and syntax highlighting setting
        let glamour_style = if !self.syntax_highlighting {
            // When syntax highlighting is disabled, use Ascii style
            GlamourStyle::Ascii
        } else if theme.preset.name() == "Light" {
            GlamourStyle::Light
        } else {
            GlamourStyle::Dark
        };

        // Create renderer with appropriate settings
        let mut renderer = TermRenderer::new()
            .with_style(glamour_style)
            .with_word_wrap(width.saturating_sub(4)); // Leave margin for borders

        // Add line numbers if enabled (only available with syntax-highlighting feature)
        #[cfg(feature = "syntax-highlighting")]
        if self.line_numbers {
            renderer.set_line_numbers(true);
        }

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
        let is_focused = self.focus == DocsFocus::Content || self.focus == DocsFocus::Search;
        let is_searching = self.focus == DocsFocus::Search;

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

        // Build header with title, scroll position, and optional search info
        let title = self
            .current_entry()
            .map(|e| e.title)
            .unwrap_or("Documentation");
        let scroll_info = format!("{}%", percent);

        // Build toggle status indicators
        let syntax_indicator = if self.syntax_highlighting { "S" } else { "·" };
        let lines_indicator = if self.line_numbers { "#" } else { "·" };
        let toggles = format!("[{}{}]", syntax_indicator, lines_indicator);

        // Add search status if we have a search query
        let search_status = self.search_status();
        let right_info = if search_status.is_empty() {
            format!("{} {}", toggles, scroll_info)
        } else {
            format!("{} | {} {}", search_status, toggles, scroll_info)
        };

        let title_width = width.saturating_sub(right_info.len() + 4);
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
            theme.muted_style().render(&right_info)
        );

        // Build content
        let separator = theme
            .muted_style()
            .render(&"─".repeat(width.saturating_sub(2)));
        let mut content_lines = Vec::new();
        content_lines.push(header);
        content_lines.push(separator.clone());

        // Reserve space for search bar at bottom if in search mode
        let search_bar_height = if is_searching { 2 } else { 0 };

        // Add viewport content (already height-limited by viewport)
        for line in viewport_content.lines() {
            content_lines.push(line.to_string());
        }

        // Pad to fill height, leaving room for search bar
        let used_lines = content_lines.len();
        let needed_lines = height.saturating_sub(2 + search_bar_height); // Account for border + search
        for _ in used_lines..needed_lines {
            content_lines.push(String::new());
        }

        // Add search bar if in search mode
        if is_searching {
            content_lines.push(separator);
            let prompt = theme.info_style().render("/");
            let query = if self.search_query.is_empty() {
                theme.muted_style().render("type to search...")
            } else {
                Style::new()
                    .foreground(theme.text)
                    .render(&format!("{}_", self.search_query))
            };
            content_lines.push(format!("{prompt} {query}"));
        }

        // Apply border
        let border_color = if is_searching {
            theme.info // Highlight border when searching
        } else if is_focused {
            theme.primary
        } else {
            theme.border
        };

        let border_style = Style::new()
            .border(Border::rounded())
            .border_foreground(border_color);

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
            // Search mode handling
            if self.focus == DocsFocus::Search {
                match key.key_type {
                    KeyType::Esc => {
                        self.exit_search();
                        return None;
                    }
                    KeyType::Enter => {
                        // Exit search and go to first match
                        self.exit_search();
                        if !self.search_matches.is_empty() {
                            self.goto_current_match();
                        }
                        return None;
                    }
                    KeyType::Backspace => {
                        self.search_query.pop();
                        self.update_search_matches();
                        if !self.search_matches.is_empty() {
                            self.goto_current_match();
                        }
                        return None;
                    }
                    KeyType::Runes => {
                        // Add typed characters to search query
                        for c in &key.runes {
                            if c.is_alphanumeric()
                                || c.is_whitespace()
                                || matches!(c, '-' | '_' | '.' | ':')
                            {
                                self.search_query.push(*c);
                            }
                        }
                        self.update_search_matches();
                        if !self.search_matches.is_empty() {
                            self.goto_current_match();
                        }
                        return None;
                    }
                    _ => return None,
                }
            }

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
                        // Search: / to enter search mode
                        ['/'] if self.focus == DocsFocus::Content => {
                            self.enter_search();
                            return None;
                        }
                        // Search navigation: n/N for next/prev match
                        ['n']
                            if self.focus == DocsFocus::Content
                                && !self.search_matches.is_empty() =>
                        {
                            self.next_match();
                            return None;
                        }
                        ['N']
                            if self.focus == DocsFocus::Content
                                && !self.search_matches.is_empty() =>
                        {
                            self.prev_match();
                            return None;
                        }
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
                        // Toggle syntax highlighting with 's'
                        ['s'] if self.focus == DocsFocus::Content => {
                            self.toggle_syntax_highlighting();
                            return None;
                        }
                        // Toggle line numbers with '#'
                        ['#'] if self.focus == DocsFocus::Content => {
                            self.toggle_line_numbers();
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
        match self.focus {
            DocsFocus::List => "j/k nav  Tab focus  Enter select",
            DocsFocus::Content => {
                if !self.search_matches.is_empty() {
                    "j/k scroll  / search  n/N match  s syntax  # lines"
                } else {
                    "j/k scroll  / search  s syntax  # lines  Tab list"
                }
            }
            DocsFocus::Search => "type to search  Enter confirm  Esc cancel",
        }
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
            let _needs_render = *page.needs_render.read().unwrap();
            page.select_doc(1);
            // Note: needs_render won't change if index is same
        }
    }

    #[test]
    fn docs_search_enter_exit() {
        let mut page = DocsPage::new();
        assert_eq!(page.focus, DocsFocus::List);

        // Enter search mode
        page.enter_search();
        assert_eq!(page.focus, DocsFocus::Search);
        assert!(page.search_query.is_empty());

        // Exit search mode
        page.exit_search();
        assert_eq!(page.focus, DocsFocus::List);
    }

    #[test]
    fn docs_search_finds_matches() {
        let mut page = DocsPage::new();

        // The first doc (Welcome) should contain common words
        page.search_query = "the".to_string();
        page.update_search_matches();

        // Should find matches (markdown typically contains "the")
        assert!(
            !page.search_matches.is_empty(),
            "Should find matches for common word 'the'"
        );
    }

    #[test]
    fn docs_search_no_matches() {
        let mut page = DocsPage::new();

        page.search_query = "xyzzy_nonexistent_12345".to_string();
        page.update_search_matches();

        assert!(
            page.search_matches.is_empty(),
            "Should not find matches for nonsense string"
        );
        assert_eq!(page.search_status(), "No matches");
    }

    #[test]
    fn docs_search_navigation() {
        let mut page = DocsPage::new();

        // Search for a common word
        page.search_query = "the".to_string();
        page.update_search_matches();

        if page.search_matches.len() > 1 {
            let initial = page.current_match;

            page.next_match();
            assert_eq!(page.current_match, initial + 1);

            page.prev_match();
            assert_eq!(page.current_match, initial);

            // Test wrap around
            page.current_match = 0;
            page.prev_match();
            assert_eq!(
                page.current_match,
                page.search_matches.len() - 1,
                "Should wrap to last match"
            );

            page.next_match();
            assert_eq!(page.current_match, 0, "Should wrap to first match");
        }
    }

    #[test]
    fn docs_search_cleared_on_doc_change() {
        let mut page = DocsPage::new();

        if page.entries.len() > 1 {
            page.search_query = "test".to_string();
            page.update_search_matches();

            page.select_doc(1);

            assert!(
                page.search_query.is_empty(),
                "Search query should be cleared"
            );
            assert!(
                page.search_matches.is_empty(),
                "Search matches should be cleared"
            );
        }
    }

    #[test]
    fn docs_search_status_formatting() {
        let mut page = DocsPage::new();

        // Empty query
        assert_eq!(page.search_status(), "");

        // No matches
        page.search_query = "xyzzy_nonexistent".to_string();
        page.update_search_matches();
        assert_eq!(page.search_status(), "No matches");

        // With matches
        page.search_query = "the".to_string();
        page.update_search_matches();
        if !page.search_matches.is_empty() {
            let status = page.search_status();
            assert!(
                status.contains('/'),
                "Status should show current/total format"
            );
        }
    }

    #[test]
    fn docs_search_hints_update() {
        let mut page = DocsPage::new();

        // List focus hints
        page.focus = DocsFocus::List;
        let list_hints = page.hints();
        assert!(
            list_hints.contains("nav"),
            "List hints should mention navigation"
        );

        // Content focus hints without search
        page.focus = DocsFocus::Content;
        let content_hints = page.hints();
        assert!(
            content_hints.contains("search"),
            "Content hints should mention search"
        );

        // Content focus hints with search matches
        page.search_query = "the".to_string();
        page.update_search_matches();
        if !page.search_matches.is_empty() {
            let search_hints = page.hints();
            assert!(
                search_hints.contains("n/N"),
                "Should show n/N for match navigation"
            );
        }

        // Search mode hints
        page.focus = DocsFocus::Search;
        let search_mode_hints = page.hints();
        assert!(
            search_mode_hints.contains("Esc"),
            "Search hints should mention Esc"
        );
    }

    // =========================================================================
    // Edge Case Tests (for bd-3eru)
    // =========================================================================

    #[test]
    fn empty_search_query_shows_no_matches() {
        let mut page = DocsPage::new();

        page.search_query = String::new();
        page.update_search_matches();
        assert!(
            page.search_matches.is_empty(),
            "Empty query should have no matches"
        );
    }

    #[test]
    fn unicode_search_does_not_panic() {
        let mut page = DocsPage::new();

        // Unicode characters in query should not panic
        page.search_query = "日本語テスト".to_string();
        page.update_search_matches();
        // Should complete without panicking
        assert!(page.search_matches.len() <= 10_000); // Sanity bound
    }

    #[test]
    fn emoji_search_does_not_panic() {
        let mut page = DocsPage::new();

        // Emoji in query should not panic
        page.search_query = "🚀 deployment 🎉".to_string();
        page.update_search_matches();
        // Should complete without panicking
    }

    #[test]
    fn very_long_search_does_not_panic() {
        let mut page = DocsPage::new();

        // Very long query should not panic or cause memory issues
        page.search_query = "a".repeat(10_000);
        page.update_search_matches();
        // Should complete without panicking (likely no matches)
        assert!(page.search_matches.is_empty() || page.search_matches.len() < 10_000);
    }

    #[test]
    fn whitespace_only_search() {
        let mut page = DocsPage::new();

        // Whitespace-only query
        page.search_query = "   ".to_string();
        page.update_search_matches();
        // Should not crash; whitespace may or may not match
    }

    #[test]
    fn newline_in_search_does_not_panic() {
        let mut page = DocsPage::new();

        // Paste-like input with newlines (though our input filter blocks some chars)
        page.search_query = "hello\nworld".to_string();
        page.update_search_matches();
        // Should complete without panicking
    }

    #[test]
    fn search_is_case_insensitive() {
        let mut page = DocsPage::new();

        // Same query in different cases should find same number of matches
        page.search_query = "THE".to_string();
        page.update_search_matches();
        let upper_count = page.search_matches.len();

        page.search_query = "the".to_string();
        page.update_search_matches();
        let lower_count = page.search_matches.len();

        page.search_query = "The".to_string();
        page.update_search_matches();
        let mixed_count = page.search_matches.len();

        assert_eq!(
            upper_count, lower_count,
            "Case should not affect match count"
        );
        assert_eq!(
            lower_count, mixed_count,
            "Case should not affect match count"
        );
    }

    #[test]
    fn search_is_idempotent() {
        let mut page = DocsPage::new();

        page.search_query = "the".to_string();
        page.update_search_matches();
        let first_count = page.search_matches.len();
        let first_current = page.current_match;

        // Apply again - should get same result
        page.update_search_matches();
        let second_count = page.search_matches.len();
        let second_current = page.current_match;

        assert_eq!(first_count, second_count, "Search should be idempotent");
        assert_eq!(
            first_current, second_current,
            "Current match should reset to 0"
        );
    }

    #[test]
    fn search_match_navigation_at_boundaries() {
        let mut page = DocsPage::new();

        page.search_query = "the".to_string();
        page.update_search_matches();

        if page.search_matches.len() > 1 {
            // Start at first match
            assert_eq!(page.current_match, 0);

            // Go to last match
            page.current_match = page.search_matches.len() - 1;

            // Next should wrap to first
            page.next_match();
            assert_eq!(page.current_match, 0, "Should wrap from last to first");

            // Prev should wrap to last
            page.prev_match();
            assert_eq!(
                page.current_match,
                page.search_matches.len() - 1,
                "Should wrap from first to last"
            );
        }
    }

    #[test]
    fn special_characters_in_search() {
        let mut page = DocsPage::new();

        // Characters that might break regex or matching
        page.search_query = "[test]".to_string();
        page.update_search_matches();
        // Should not panic

        page.search_query = "(foo)".to_string();
        page.update_search_matches();
        // Should not panic

        page.search_query = "*.rs".to_string();
        page.update_search_matches();
        // Should not panic

        page.search_query = "foo/bar".to_string();
        page.update_search_matches();
        // Should not panic
    }

    // =========================================================================
    // Syntax Highlighting & Line Numbers Toggle Tests (for bd-3d87)
    // =========================================================================

    #[test]
    fn syntax_highlighting_default() {
        let page = DocsPage::new();
        assert!(
            page.syntax_highlighting_enabled(),
            "Syntax highlighting should be on by default"
        );
    }

    #[test]
    fn line_numbers_default() {
        let page = DocsPage::new();
        assert!(
            !page.line_numbers_enabled(),
            "Line numbers should be off by default"
        );
    }

    #[test]
    fn toggle_syntax_highlighting() {
        let mut page = DocsPage::new();
        assert!(page.syntax_highlighting);

        page.toggle_syntax_highlighting();
        assert!(!page.syntax_highlighting);
        assert!(
            *page.needs_render.read().unwrap(),
            "Should need re-render after toggle"
        );

        *page.needs_render.write().unwrap() = false;
        page.toggle_syntax_highlighting();
        assert!(page.syntax_highlighting);
        assert!(
            *page.needs_render.read().unwrap(),
            "Should need re-render after toggle"
        );
    }

    #[test]
    fn toggle_line_numbers() {
        let mut page = DocsPage::new();
        assert!(!page.line_numbers);

        page.toggle_line_numbers();
        assert!(page.line_numbers);
        assert!(
            *page.needs_render.read().unwrap(),
            "Should need re-render after toggle"
        );

        *page.needs_render.write().unwrap() = false;
        page.toggle_line_numbers();
        assert!(!page.line_numbers);
        assert!(
            *page.needs_render.read().unwrap(),
            "Should need re-render after toggle"
        );
    }
}
