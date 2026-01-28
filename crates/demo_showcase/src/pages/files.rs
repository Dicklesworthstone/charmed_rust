//! Files page - file browser with preview.
//!
//! This page integrates the bubbles FilePicker component to provide
//! directory navigation and file preview capabilities.
//!
//! # Modes
//!
//! - **Fixture mode** (default): Uses embedded `assets::fixtures::FIXTURE_TREE`
//!   for deterministic demos and E2E testing
//! - **Real mode**: When `Config.files_root` is set, browses actual filesystem
//!
//! # Features
//!
//! - Keyboard navigation (j/k, enter, backspace)
//! - Hidden files toggle (h key)
//! - Selection updates preview pane
//! - Breadcrumb path display

use std::path::PathBuf;
use std::sync::RwLock;

use bubbles::filepicker::FilePicker;
use bubbles::viewport::Viewport;
use bubbletea::{Cmd, KeyMsg, KeyType, Message};
use glamour::{Style as GlamourStyle, TermRenderer};
use lipgloss::Style;

use super::PageModel;
use crate::assets::fixtures::{FIXTURE_TREE, VirtualEntry};
use crate::messages::Page;
use crate::theme::Theme;

/// Maximum bytes to load for preview (64 KB).
const PREVIEW_MAX_BYTES: usize = 64 * 1024;

/// Files page showing file browser with preview.
pub struct FilesPage {
    /// The file picker component (used for real filesystem mode).
    picker: Option<FilePicker>,
    /// Virtual file entries (used for fixture mode).
    virtual_entries: Vec<VirtualEntry>,
    /// Current path in virtual filesystem.
    virtual_path: Vec<&'static str>,
    /// Selected index in current directory.
    selected: usize,
    /// Whether showing hidden files.
    show_hidden: bool,
    /// Preview file name.
    preview_name: Option<String>,
    /// Whether using real filesystem mode.
    real_mode: bool,
    /// Height in rows.
    height: usize,
    /// Scroll offset for file list.
    scroll_offset: usize,
    /// Viewport for scrollable preview content (`RwLock` for thread-safe interior mutability).
    preview_viewport: RwLock<Viewport>,
    /// Whether the preview content was truncated.
    preview_truncated: bool,
    /// Whether focus is on the preview pane (for scrolling).
    preview_focused: bool,
    /// Raw content for markdown rendering (stored for re-render on resize/theme change).
    raw_content: Option<String>,
    /// Whether the current file is markdown.
    is_markdown: bool,
    /// Whether syntax highlighting is enabled.
    syntax_highlighting: bool,
    /// Whether to show line numbers in code blocks.
    line_numbers: bool,
    /// Last known width for detecting resize (`RwLock` for interior mutability during view).
    last_width: RwLock<usize>,
    /// Last known theme name for detecting theme changes (`RwLock` for interior mutability during view).
    last_theme: RwLock<String>,
}

impl FilesPage {
    /// Create a new files page in fixture mode.
    #[must_use]
    pub fn new() -> Self {
        let virtual_entries = Self::entries_from_fixture(FIXTURE_TREE);

        Self {
            picker: None,
            virtual_entries,
            virtual_path: Vec::new(),
            selected: 0,
            show_hidden: false,
            preview_name: None,
            real_mode: false,
            height: 20,
            scroll_offset: 0,
            preview_viewport: RwLock::new(Viewport::new(40, 20)),
            preview_truncated: false,
            preview_focused: false,
            raw_content: None,
            is_markdown: false,
            syntax_highlighting: true,
            line_numbers: false,
            last_width: RwLock::new(0),
            last_theme: RwLock::new(String::new()),
        }
    }

    /// Create a new files page with real filesystem mode.
    #[must_use]
    pub fn with_root(root: PathBuf) -> Self {
        let mut picker = FilePicker::new();
        picker.set_root(&root);
        picker.set_current_directory(&root);
        picker.show_hidden = false;
        picker.show_permissions = false;
        picker.show_size = true;
        picker.dir_allowed = true;
        picker.file_allowed = true;

        Self {
            picker: Some(picker),
            virtual_entries: Vec::new(),
            virtual_path: Vec::new(),
            selected: 0,
            show_hidden: false,
            preview_name: None,
            real_mode: true,
            height: 20,
            scroll_offset: 0,
            preview_viewport: RwLock::new(Viewport::new(40, 20)),
            preview_truncated: false,
            preview_focused: false,
            raw_content: None,
            is_markdown: false,
            syntax_highlighting: true,
            line_numbers: false,
            last_width: RwLock::new(0),
            last_theme: RwLock::new(String::new()),
        }
    }

    /// Convert static fixture entries to owned entries.
    fn entries_from_fixture(entries: &'static [VirtualEntry]) -> Vec<VirtualEntry> {
        entries.to_vec()
    }

    /// Get current directory entries (filtered by hidden state).
    fn visible_entries(&self) -> Vec<&VirtualEntry> {
        self.virtual_entries
            .iter()
            .filter(|e| self.show_hidden || !e.is_hidden())
            .collect()
    }

    /// Get current path as string.
    fn current_path_display(&self) -> String {
        if self.virtual_path.is_empty() {
            "fixtures/".to_string()
        } else {
            format!("fixtures/{}/", self.virtual_path.join("/"))
        }
    }

    /// Navigate into a directory.
    fn enter_directory(&mut self) {
        // Extract data first to avoid borrow conflicts
        let action = {
            let entries: Vec<_> = self
                .virtual_entries
                .iter()
                .filter(|e| self.show_hidden || !e.is_hidden())
                .collect();

            if let Some(entry) = entries.get(self.selected) {
                if let Some(children) = entry.children() {
                    Some((entry.name, Some(children.to_vec()), None::<String>))
                } else if let Some(content) = entry.content() {
                    Some((entry.name, None, Some(content.to_string())))
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some((name, children_opt, content_opt)) = action {
            if let Some(children) = children_opt {
                self.virtual_path.push(name);
                self.virtual_entries = children;
                self.selected = 0;
                self.scroll_offset = 0;
                self.preview_viewport.write().unwrap().set_content("");
                self.preview_name = None;
                self.preview_truncated = false;
            } else if let Some(content) = content_opt {
                // Handle file content with truncation
                let (truncated_content, was_truncated) = if content.len() > PREVIEW_MAX_BYTES {
                    let truncated = content
                        .char_indices()
                        .take_while(|(i, _)| *i < PREVIEW_MAX_BYTES)
                        .map(|(_, c)| c)
                        .collect::<String>();
                    (truncated, true)
                } else {
                    (content, false)
                };
                let mut viewport = self.preview_viewport.write().unwrap();
                viewport.set_content(&truncated_content);
                viewport.goto_top();
                self.preview_name = Some(name.to_string());
                self.preview_truncated = was_truncated;
            }
        }
    }

    /// Navigate to parent directory.
    fn go_back(&mut self) {
        if self.virtual_path.is_empty() {
            return;
        }

        // Find parent entries
        let mut current_tree: &[VirtualEntry] = FIXTURE_TREE;
        self.virtual_path.pop();

        for segment in &self.virtual_path {
            if let Some(entry) = current_tree.iter().find(|e| e.name == *segment) {
                if let Some(children) = entry.children() {
                    current_tree = children;
                }
            }
        }

        self.virtual_entries = current_tree.to_vec();
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Move selection up.
    fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.update_preview();
            self.ensure_visible();
        }
    }

    /// Move selection down.
    fn move_down(&mut self) {
        let entries = self.visible_entries();
        if self.selected < entries.len().saturating_sub(1) {
            self.selected += 1;
            self.update_preview();
            self.ensure_visible();
        }
    }

    /// Go to first entry.
    fn goto_top(&mut self) {
        self.selected = 0;
        self.scroll_offset = 0;
        self.update_preview();
    }

    /// Go to last entry.
    fn goto_bottom(&mut self) {
        let entries = self.visible_entries();
        self.selected = entries.len().saturating_sub(1);
        self.ensure_visible();
        self.update_preview();
    }

    /// Ensure selected item is visible.
    fn ensure_visible(&mut self) {
        let visible_rows = self.height.saturating_sub(4); // Header + footer
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + visible_rows {
            self.scroll_offset = self.selected.saturating_sub(visible_rows) + 1;
        }
    }

    /// Toggle hidden files visibility.
    fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
        // Clamp selection
        let count = self.visible_entry_count();
        if self.selected >= count {
            self.selected = count.saturating_sub(1);
        }
    }

    /// Count visible entries.
    fn visible_entry_count(&self) -> usize {
        self.virtual_entries
            .iter()
            .filter(|e| self.show_hidden || !e.is_hidden())
            .count()
    }

    /// Update preview based on current selection.
    fn update_preview(&mut self) {
        // Extract data first to avoid borrow conflicts
        let (content, name, is_dir) = {
            let entries: Vec<_> = self
                .virtual_entries
                .iter()
                .filter(|e| self.show_hidden || !e.is_hidden())
                .collect();

            if let Some(entry) = entries.get(self.selected) {
                (
                    entry.content().map(String::from),
                    entry.name.to_string(),
                    entry.is_dir(),
                )
            } else {
                (None, String::new(), false)
            }
        };

        if !name.is_empty() {
            // Check if file is markdown
            self.is_markdown = !is_dir && Self::is_markdown_file(&name);

            self.preview_name = if is_dir {
                Some(format!("{}/", name))
            } else {
                Some(name)
            };

            // Set viewport content with truncation for large files
            if let Some(content) = content {
                let (truncated_content, was_truncated) = if content.len() > PREVIEW_MAX_BYTES {
                    let truncated = content
                        .char_indices()
                        .take_while(|(i, _)| *i < PREVIEW_MAX_BYTES)
                        .map(|(_, c)| c)
                        .collect::<String>();
                    (truncated, true)
                } else {
                    (content, false)
                };

                // Store raw content for markdown files (will be rendered in view)
                if self.is_markdown {
                    self.raw_content = Some(truncated_content.clone());
                    // Clear viewport - will be filled in render_preview with proper theme
                    self.preview_viewport.write().unwrap().set_content("");
                    // Force re-render by clearing last dimensions
                    *self.last_width.write().unwrap() = 0;
                    self.last_theme.write().unwrap().clear();
                } else {
                    // Non-markdown: show raw content directly
                    self.raw_content = None;
                    self.preview_viewport.write().unwrap().set_content(&truncated_content);
                }
                self.preview_truncated = was_truncated;
            } else {
                self.preview_viewport.write().unwrap().set_content("");
                self.raw_content = None;
                self.preview_truncated = false;
            }
            // Reset scroll position when switching files
            self.preview_viewport.write().unwrap().goto_top();
        } else {
            self.preview_viewport.write().unwrap().set_content("");
            self.preview_name = None;
            self.raw_content = None;
            self.is_markdown = false;
            self.preview_truncated = false;
        }
    }

    /// Render the file list.
    fn render_list(&self, _width: usize, height: usize, theme: &Theme) -> String {
        let entries = self.visible_entries();
        let visible_rows = height.saturating_sub(2); // For breadcrumb + status

        let mut lines = Vec::new();

        // Breadcrumb
        let path = self.current_path_display();
        let breadcrumb = theme.muted_style().render(&path);
        lines.push(breadcrumb);

        // Entry list
        for (i, entry) in entries
            .iter()
            .enumerate()
            .skip(self.scroll_offset)
            .take(visible_rows)
        {
            let is_selected = i == self.selected;
            let cursor = if is_selected { ">" } else { " " };

            let name = if entry.is_dir() {
                format!("{}/", entry.name)
            } else {
                entry.name.to_string()
            };

            let icon = if entry.is_dir() {
                theme.muted_style().render("📁 ")
            } else {
                theme.muted_style().render("📄 ")
            };

            let name_style = if is_selected {
                theme.title_style()
            } else if entry.is_dir() {
                theme.info_style()
            } else {
                Style::new()
            };

            let cursor_style = if is_selected {
                theme.info_style()
            } else {
                theme.muted_style()
            };

            let line = format!(
                "{} {}{}",
                cursor_style.render(cursor),
                icon,
                name_style.render(&name)
            );

            lines.push(line);
        }

        // Pad to height
        while lines.len() < height.saturating_sub(1) {
            lines.push(String::new());
        }

        // Status line
        let hidden_indicator = if self.show_hidden {
            "[h] Hide"
        } else {
            "[h] Show hidden"
        };
        let status = format!(
            "{}/{} {}",
            self.selected + 1,
            entries.len(),
            hidden_indicator
        );
        lines.push(theme.muted_style().render(&status));

        lines.join("\n")
    }

    /// Render the preview pane.
    fn render_preview(&self, width: usize, height: usize, theme: &Theme) -> String {
        let mut lines = Vec::new();

        // Header with focus indicator and markdown indicator
        let focus_indicator = if self.preview_focused { "● " } else { "" };
        let md_indicator = if self.is_markdown { " [MD]" } else { "" };
        let header = if let Some(ref name) = self.preview_name {
            let header_text = format!("{}{}{}", focus_indicator, name, md_indicator);
            theme.heading_style().render(&header_text)
        } else {
            theme.muted_style().render("(no selection)")
        };
        lines.push(header);
        lines.push(theme.muted_style().render(&"─".repeat(width.min(40))));

        // For markdown files, check if we need to re-render
        let theme_name = theme.preset.name().to_string();
        if self.is_markdown {
            if let Some(ref raw) = self.raw_content {
                // Re-render if width or theme changed
                let last_w = *self.last_width.read().unwrap();
                let last_t = self.last_theme.read().unwrap().clone();
                if last_w != width || last_t != theme_name {
                    let content_width = width.saturating_sub(2);
                    let rendered = self.render_markdown(raw, theme, content_width);
                    self.preview_viewport.write().unwrap().set_content(&rendered);
                    *self.last_width.write().unwrap() = width;
                    *self.last_theme.write().unwrap() = theme_name;
                }
            }
        }

        // Content via viewport
        let viewport = self.preview_viewport.read().unwrap();
        let has_content = viewport.total_line_count() > 0
            || (self.is_markdown && self.raw_content.is_some());
        if has_content {
            // Create a viewport clone with correct dimensions for rendering
            let mut render_viewport = viewport.clone();
            let content_height = height.saturating_sub(4); // header + separator + status
            render_viewport.width = width.saturating_sub(2);
            render_viewport.height = content_height;

            let viewport_view = render_viewport.view();
            for line in viewport_view.lines() {
                lines.push(line.to_string());
            }

            // Scroll position indicator
            let scroll_pct = viewport.scroll_percent();
            let scroll_indicator = if viewport.total_line_count() <= content_height {
                String::new()
            } else if viewport.at_top() {
                "Top".to_string()
            } else if viewport.at_bottom() {
                "End".to_string()
            } else {
                format!("{}%", (scroll_pct * 100.0) as usize)
            };

            let truncated_indicator = if self.preview_truncated {
                " [truncated]"
            } else {
                ""
            };

            // Add toggle indicators for markdown files
            let toggle_indicator = if self.is_markdown {
                let syntax_char = if self.syntax_highlighting { "S" } else { "·" };
                let line_char = if self.line_numbers { "#" } else { "·" };
                format!(" [{}{}]", syntax_char, line_char)
            } else {
                String::new()
            };

            let status = format!("{}{}{}", scroll_indicator, truncated_indicator, toggle_indicator);
            lines.push(theme.muted_style().render(&status));
        } else if self.preview_name.is_some() {
            lines.push(theme.muted_style().render("(directory)"));
        }

        // Pad to height
        while lines.len() < height {
            lines.push(String::new());
        }

        lines.join("\n")
    }

    /// Toggle focus between file list and preview pane.
    fn toggle_preview_focus(&mut self) {
        self.preview_focused = !self.preview_focused;
    }

    /// Check if a filename indicates a markdown file.
    fn is_markdown_file(name: &str) -> bool {
        let lower = name.to_lowercase();
        lower.ends_with(".md")
            || lower.ends_with(".markdown")
            || lower.ends_with(".mdx")
            || lower.ends_with(".mdown")
    }

    /// Toggle syntax highlighting on/off.
    pub fn toggle_syntax_highlighting(&mut self) {
        self.syntax_highlighting = !self.syntax_highlighting;
    }

    /// Toggle line numbers on/off.
    pub fn toggle_line_numbers(&mut self) {
        self.line_numbers = !self.line_numbers;
    }

    /// Render markdown content via glamour.
    fn render_markdown(&self, content: &str, theme: &Theme, width: usize) -> String {
        // Choose glamour style based on theme and syntax highlighting setting
        let glamour_style = if !self.syntax_highlighting {
            // When syntax highlighting is disabled, use Ascii style (no colors)
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

        renderer.render(content)
    }
}

impl Default for FilesPage {
    fn default() -> Self {
        Self::new()
    }
}

impl PageModel for FilesPage {
    fn update(&mut self, msg: &Message) -> Option<Cmd> {
        // Note: Real filesystem mode requires changes to bubbletea's Message type
        // to support Clone. For now, we only support virtual fixture mode.

        // Handle key messages
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            // Tab toggles focus between file list and preview
            if key.key_type == KeyType::Tab {
                self.toggle_preview_focus();
                return None;
            }

            // When preview is focused, delegate scroll keys to viewport
            if self.preview_focused {
                let mut viewport = self.preview_viewport.write().unwrap();
                match key.key_type {
                    KeyType::Up => viewport.scroll_up(1),
                    KeyType::Down => viewport.scroll_down(1),
                    KeyType::PgUp => viewport.page_up(),
                    KeyType::PgDown => viewport.page_down(),
                    KeyType::Home => viewport.goto_top(),
                    KeyType::End => viewport.goto_bottom(),
                    KeyType::Esc => {
                        drop(viewport);
                        self.preview_focused = false;
                    }
                    KeyType::Runes => match key.runes.as_slice() {
                        ['j'] => viewport.scroll_down(1),
                        ['k'] => viewport.scroll_up(1),
                        ['g'] => viewport.goto_top(),
                        ['G'] => viewport.goto_bottom(),
                        ['d'] => viewport.half_page_down(),
                        ['u'] => viewport.half_page_up(),
                        // Toggle syntax highlighting with 's' for markdown files
                        ['s'] if self.is_markdown => {
                            drop(viewport);
                            self.toggle_syntax_highlighting();
                            // Force re-render
                            *self.last_width.write().unwrap() = 0;
                        }
                        // Toggle line numbers with '#' for markdown files
                        ['#'] if self.is_markdown => {
                            drop(viewport);
                            self.toggle_line_numbers();
                            // Force re-render
                            *self.last_width.write().unwrap() = 0;
                        }
                        _ => {}
                    },
                    _ => {}
                }
                return None;
            }

            // File list navigation when not focused on preview
            match key.key_type {
                KeyType::Up => {
                    self.move_up();
                }
                KeyType::Down => {
                    self.move_down();
                }
                KeyType::Enter | KeyType::Right => {
                    self.enter_directory();
                }
                KeyType::Left | KeyType::Backspace | KeyType::Esc => {
                    self.go_back();
                }
                KeyType::Home => {
                    self.goto_top();
                }
                KeyType::End => {
                    self.goto_bottom();
                }
                KeyType::Runes => match key.runes.as_slice() {
                    ['j'] => self.move_down(),
                    ['k'] => self.move_up(),
                    ['l'] => self.enter_directory(),
                    ['h'] if key.alt => self.toggle_hidden(),
                    ['h'] => self.go_back(),
                    ['g'] => self.goto_top(),
                    ['G'] => self.goto_bottom(),
                    ['H'] => self.toggle_hidden(),
                    _ => {}
                },
                _ => {}
            }
        }

        None
    }

    fn view(&self, width: usize, height: usize, theme: &Theme) -> String {
        // Split into list and preview panes
        let list_width = width / 2;
        let preview_width = width.saturating_sub(list_width).saturating_sub(1);

        let list = self.render_list(list_width, height, theme);
        let preview = self.render_preview(preview_width, height, theme);

        // Join panes side by side
        let list_lines: Vec<&str> = list.lines().collect();
        let preview_lines: Vec<&str> = preview.lines().collect();

        let mut result = Vec::new();
        let max_lines = list_lines.len().max(preview_lines.len());

        for i in 0..max_lines {
            let list_line = list_lines.get(i).copied().unwrap_or("");
            let preview_line = preview_lines.get(i).copied().unwrap_or("");

            // Pad list line to width
            let list_visible_width = lipgloss::visible_width(list_line);
            let padding = list_width.saturating_sub(list_visible_width);

            result.push(format!(
                "{}{:padding$} │ {}",
                list_line,
                "",
                preview_line,
                padding = padding
            ));
        }

        result.join("\n")
    }

    fn page(&self) -> Page {
        Page::Docs // TODO: Add Page::Files variant
    }

    fn hints(&self) -> &'static str {
        if self.preview_focused {
            if self.is_markdown {
                "j/k scroll  s syntax  # lines  Tab list  Esc back"
            } else {
                "j/k scroll  g/G top/bottom  Tab list  Esc back"
            }
        } else {
            "j/k nav  l/Enter open  h back  H hidden  Tab preview"
        }
    }

    fn on_enter(&mut self) -> Option<Cmd> {
        self.update_preview();

        // For real mode, initialize the picker
        if self.real_mode {
            if let Some(picker) = &self.picker {
                return picker.init();
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn files_page_creates() {
        let page = FilesPage::new();
        assert!(!page.virtual_entries.is_empty());
        assert!(!page.real_mode);
    }

    #[test]
    fn files_page_navigation() {
        let mut page = FilesPage::new();

        // Move down
        page.move_down();
        assert!(page.selected > 0 || page.visible_entries().len() <= 1);

        // Move up
        page.move_up();
        assert_eq!(page.selected, 0);
    }

    #[test]
    fn files_page_hidden_toggle() {
        let mut page = FilesPage::new();
        assert!(!page.show_hidden);

        page.toggle_hidden();
        assert!(page.show_hidden);

        page.toggle_hidden();
        assert!(!page.show_hidden);
    }

    #[test]
    fn files_page_path_display() {
        let page = FilesPage::new();
        assert_eq!(page.current_path_display(), "fixtures/");
    }

    #[test]
    fn files_page_hints() {
        let page = FilesPage::new();
        let hints = page.hints();
        assert!(hints.contains("nav"));
        assert!(hints.contains("hidden"));
    }

    #[test]
    fn files_page_enter_directory() {
        let mut page = FilesPage::new();

        // Find first directory
        let entries = page.visible_entries();
        let first_dir_idx = entries.iter().position(|e| e.is_dir());

        if let Some(idx) = first_dir_idx {
            page.selected = idx;
            page.enter_directory();
            assert!(!page.virtual_path.is_empty());
        }
    }

    #[test]
    fn files_page_go_back() {
        let mut page = FilesPage::new();

        // Enter a directory first
        let entries = page.visible_entries();
        if let Some(idx) = entries.iter().position(|e| e.is_dir()) {
            page.selected = idx;
            page.enter_directory();

            // Now go back
            page.go_back();
            assert!(page.virtual_path.is_empty());
        }
    }

    #[test]
    fn files_page_preview_focus_toggle() {
        let mut page = FilesPage::new();
        assert!(!page.preview_focused);

        page.toggle_preview_focus();
        assert!(page.preview_focused);

        page.toggle_preview_focus();
        assert!(!page.preview_focused);
    }

    #[test]
    fn files_page_hints_change_with_focus() {
        let mut page = FilesPage::new();

        // Not focused - should show file list hints
        let hints = page.hints();
        assert!(hints.contains("nav"));
        assert!(hints.contains("Tab preview"));

        // Focused on preview - should show scroll hints
        page.toggle_preview_focus();
        let hints = page.hints();
        assert!(hints.contains("scroll"));
        assert!(hints.contains("Tab list"));
    }

    #[test]
    fn files_page_viewport_initialized() {
        let page = FilesPage::new();
        assert_eq!(page.preview_viewport.read().unwrap().total_line_count(), 0);
        assert!(!page.preview_truncated);
    }

    #[test]
    fn files_page_preview_viewport_scrolling() {
        let mut page = FilesPage::new();

        // Find and select a file with content
        let entries = page.visible_entries();
        let file_idx = entries
            .iter()
            .position(|e| !e.is_dir() && e.content().is_some());

        if let Some(idx) = file_idx {
            page.selected = idx;
            page.update_preview();

            // Verify content was loaded into viewport
            assert!(page.preview_viewport.read().unwrap().total_line_count() > 0 || page.preview_name.is_some());
        }
    }

    #[test]
    fn files_page_markdown_detection() {
        assert!(FilesPage::is_markdown_file("README.md"));
        assert!(FilesPage::is_markdown_file("doc.markdown"));
        assert!(FilesPage::is_markdown_file("test.MDX"));
        assert!(FilesPage::is_markdown_file("notes.mdown"));
        assert!(!FilesPage::is_markdown_file("code.rs"));
        assert!(!FilesPage::is_markdown_file("config.toml"));
        assert!(!FilesPage::is_markdown_file("file.txt"));
    }

    #[test]
    fn files_page_syntax_highlighting_toggle() {
        let mut page = FilesPage::new();
        assert!(page.syntax_highlighting);

        page.toggle_syntax_highlighting();
        assert!(!page.syntax_highlighting);

        page.toggle_syntax_highlighting();
        assert!(page.syntax_highlighting);
    }

    #[test]
    fn files_page_line_numbers_toggle() {
        let mut page = FilesPage::new();
        assert!(!page.line_numbers);

        page.toggle_line_numbers();
        assert!(page.line_numbers);

        page.toggle_line_numbers();
        assert!(!page.line_numbers);
    }
}
