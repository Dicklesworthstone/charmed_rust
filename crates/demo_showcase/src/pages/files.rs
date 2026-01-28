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

use bubbles::filepicker::FilePicker;
use bubbletea::{Cmd, KeyMsg, KeyType, Message};
use lipgloss::Style;

use super::PageModel;
use crate::assets::fixtures::{FIXTURE_TREE, VirtualEntry};
use crate::messages::Page;
use crate::theme::Theme;

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
    /// Selected file content for preview.
    preview_content: Option<String>,
    /// Preview file name.
    preview_name: Option<String>,
    /// Whether using real filesystem mode.
    real_mode: bool,
    /// Height in rows.
    height: usize,
    /// Scroll offset for file list.
    scroll_offset: usize,
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
            preview_content: None,
            preview_name: None,
            real_mode: false,
            height: 20,
            scroll_offset: 0,
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
            preview_content: None,
            preview_name: None,
            real_mode: true,
            height: 20,
            scroll_offset: 0,
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
                self.preview_content = None;
                self.preview_name = None;
            } else if let Some(content) = content_opt {
                self.preview_content = Some(content);
                self.preview_name = Some(name.to_string());
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
            self.preview_content = content;
            self.preview_name = if is_dir {
                Some(format!("{}/", name))
            } else {
                Some(name)
            };
        } else {
            self.preview_content = None;
            self.preview_name = None;
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

        // Header
        let header = if let Some(ref name) = self.preview_name {
            theme.heading_style().render(name)
        } else {
            theme.muted_style().render("(no selection)")
        };
        lines.push(header);
        lines.push(theme.muted_style().render(&"─".repeat(width.min(40))));

        // Content
        if let Some(ref content) = self.preview_content {
            let content_height = height.saturating_sub(3);
            for (i, line) in content.lines().enumerate() {
                if i >= content_height {
                    lines.push(theme.muted_style().render("..."));
                    break;
                }
                let truncated: String = line.chars().take(width.saturating_sub(2)).collect();
                lines.push(truncated);
            }
        } else if self.preview_name.is_some() {
            lines.push(theme.muted_style().render("(directory)"));
        }

        // Pad to height
        while lines.len() < height {
            lines.push(String::new());
        }

        lines.join("\n")
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
        "j/k nav  l/Enter open  h back  H hidden  g/G top/bottom"
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
}
