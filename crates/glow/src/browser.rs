//! Interactive file browser for discovering and selecting markdown files.
//!
//! # Example
//!
//! ```rust,ignore
//! use glow::browser::{FileBrowser, BrowserConfig};
//!
//! let mut browser = FileBrowser::new(BrowserConfig::default());
//! browser.scan_current_directory()?;
//!
//! // Use in TUI with bubbletea Model trait
//! ```

use std::cmp::Ordering;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model};
use lipgloss::Style;

/// File browser configuration.
#[derive(Debug, Clone)]
pub struct BrowserConfig {
    /// Show hidden files (starting with .)
    pub show_hidden: bool,
    /// Recursively scan subdirectories
    pub recursive: bool,
    /// Extensions to include (empty = all markdown)
    pub extensions: Vec<String>,
    /// Maximum directory depth for recursive scan
    pub max_depth: usize,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            show_hidden: false,
            recursive: false,
            extensions: vec![
                "md".to_string(),
                "markdown".to_string(),
                "mdown".to_string(),
                "mkd".to_string(),
            ],
            max_depth: 5,
        }
    }
}

/// Type of file system entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryType {
    /// A file (potentially markdown)
    File { is_markdown: bool },
    /// A directory
    Directory,
    /// A symbolic link
    Symlink,
}

/// A file system entry in the browser.
#[derive(Debug, Clone)]
pub struct Entry {
    /// Display name
    pub name: String,
    /// Full path
    pub path: PathBuf,
    /// Entry type
    pub entry_type: EntryType,
    /// File size in bytes (0 for directories)
    pub size: u64,
    /// Last modification time
    pub modified: Option<SystemTime>,
}

impl Entry {
    /// Creates a new entry from a path.
    pub fn from_path(path: &Path) -> io::Result<Self> {
        // Use symlink_metadata to detect symlinks without following them
        let symlink_metadata = fs::symlink_metadata(path)?;
        let file_type = symlink_metadata.file_type();
        let (size, modified) = if file_type.is_symlink() {
            match fs::metadata(path) {
                Ok(target) => (target.len(), target.modified().ok()),
                Err(_) => (symlink_metadata.len(), symlink_metadata.modified().ok()),
            }
        } else {
            (symlink_metadata.len(), symlink_metadata.modified().ok())
        };
        let name = path
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("")
            .to_string();

        let entry_type = if file_type.is_symlink() {
            EntryType::Symlink
        } else if file_type.is_dir() {
            EntryType::Directory
        } else {
            let is_markdown = is_markdown_file(path);
            EntryType::File { is_markdown }
        };

        Ok(Self {
            name,
            path: path.to_path_buf(),
            entry_type,
            size,
            modified,
        })
    }

    /// Returns true if this entry is a directory.
    pub fn is_directory(&self) -> bool {
        matches!(self.entry_type, EntryType::Directory)
            || (matches!(self.entry_type, EntryType::Symlink) && self.path.is_dir())
    }

    /// Returns true if this entry is a markdown file.
    pub fn is_markdown(&self) -> bool {
        matches!(self.entry_type, EntryType::File { is_markdown: true })
            || (matches!(self.entry_type, EntryType::Symlink) && is_markdown_file(&self.path))
    }

    /// Returns a display string for the file size.
    pub fn size_display(&self) -> String {
        if self.is_directory() {
            return "-".to_string();
        }
        format_size(self.size)
    }
}

/// Interactive file browser model.
#[derive(Debug, Clone)]
pub struct FileBrowser {
    /// Current directory being browsed
    current_dir: PathBuf,
    /// All entries in current directory
    entries: Vec<Entry>,
    /// Currently selected entry index
    selected: usize,
    /// Filter string for fuzzy matching
    filter: String,
    /// Filtered entries (indices into entries)
    filtered_indices: Vec<usize>,
    /// Browser configuration
    config: BrowserConfig,
    /// Whether the browser has focus
    focused: bool,
    /// Viewport height (number of visible entries)
    height: usize,
    /// Viewport scroll offset
    scroll_offset: usize,
    /// Selected file style
    selected_style: Style,
    /// Directory style
    dir_style: Style,
    /// File style
    file_style: Style,
    /// Filter input active
    filter_mode: bool,
}

impl FileBrowser {
    /// Creates a new file browser at the current directory.
    pub fn new(config: BrowserConfig) -> Self {
        Self {
            current_dir: std::env::current_dir().unwrap_or_default(),
            entries: Vec::new(),
            selected: 0,
            filter: String::new(),
            filtered_indices: Vec::new(),
            config,
            focused: true,
            height: 20,
            scroll_offset: 0,
            selected_style: Style::new().background("#7D56F4").foreground("#FFFFFF"),
            dir_style: Style::new().foreground("#7D56F4").bold(),
            file_style: Style::new().foreground("#EEEEEE"),
            filter_mode: false,
        }
    }

    /// Creates a new file browser at a specific directory.
    pub fn with_directory(path: impl AsRef<Path>, config: BrowserConfig) -> io::Result<Self> {
        let path = path.as_ref();
        if !path.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotADirectory,
                format!("{} is not a directory", path.display()),
            ));
        }
        let mut browser = Self::new(config);
        browser.current_dir = path.to_path_buf();
        Ok(browser)
    }

    /// Sets the viewport height.
    pub fn height(mut self, height: usize) -> Self {
        self.height = height;
        self
    }

    /// Sets focused state.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Returns the current directory.
    pub fn current_directory(&self) -> &Path {
        &self.current_dir
    }

    /// Returns the currently selected entry.
    pub fn selected_entry(&self) -> Option<&Entry> {
        if self.filtered_indices.is_empty() {
            return None;
        }
        let idx = self.filtered_indices.get(self.selected)?;
        self.entries.get(*idx)
    }

    /// Returns all entries.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Returns filtered entries.
    pub fn filtered_entries(&self) -> Vec<&Entry> {
        self.filtered_indices
            .iter()
            .filter_map(|&i| self.entries.get(i))
            .collect()
    }

    /// Returns the current filter string.
    pub fn filter(&self) -> &str {
        &self.filter
    }

    /// Returns whether filter mode is active.
    pub fn is_filter_mode(&self) -> bool {
        self.filter_mode
    }

    /// Scans the current directory and populates entries.
    pub fn scan(&mut self) -> io::Result<()> {
        self.entries.clear();
        self.scan_directory(&self.current_dir.clone(), 0)?;
        self.sort_entries();
        self.apply_filter();
        Ok(())
    }

    fn is_markdown_path(&self, path: &Path) -> bool {
        is_markdown_with_extensions(path, &self.config.extensions)
    }

    fn scan_directory(&mut self, dir: &Path, depth: usize) -> io::Result<()> {
        if depth > self.config.max_depth {
            return Ok(());
        }

        let read_dir = match fs::read_dir(dir) {
            Ok(rd) => rd,
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => return Ok(()),
            Err(e) => return Err(e),
        };

        for entry in read_dir.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(OsStr::to_str).unwrap_or("");

            // Skip hidden files unless configured
            if !self.config.show_hidden && name.starts_with('.') {
                continue;
            }

            if let Ok(mut file_entry) = Entry::from_path(&path) {
                let is_symlink = matches!(file_entry.entry_type, EntryType::Symlink);
                let is_markdown = self.is_markdown_path(&path);
                if let EntryType::File { is_markdown: flag } = &mut file_entry.entry_type {
                    *flag = is_markdown;
                }
                // For non-recursive mode, add directories and markdown files
                if !self.config.recursive {
                    if file_entry.is_directory() || is_markdown {
                        self.entries.push(file_entry);
                    }
                } else {
                    // For recursive mode, only show markdown files
                    if is_markdown {
                        self.entries.push(file_entry);
                    } else if file_entry.is_directory() && !is_symlink {
                        self.scan_directory(&path, depth + 1)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn sort_entries(&mut self) {
        self.entries.sort_by(|a, b| {
            // Directories first
            match (&a.entry_type, &b.entry_type) {
                (EntryType::Directory, EntryType::Directory) => a.name.cmp(&b.name),
                (EntryType::Directory, _) => Ordering::Less,
                (_, EntryType::Directory) => Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
    }

    fn apply_filter(&mut self) {
        if self.filter.is_empty() {
            self.filtered_indices = (0..self.entries.len()).collect();
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.filtered_indices = self
                .entries
                .iter()
                .enumerate()
                .filter(|(_, e)| e.name.to_lowercase().contains(&filter_lower))
                .map(|(i, _)| i)
                .collect();
        }

        // Reset selection if out of bounds
        if self.selected >= self.filtered_indices.len() {
            self.selected = self.filtered_indices.len().saturating_sub(1);
        }
        self.update_scroll();
    }

    /// Sets the filter string.
    pub fn set_filter(&mut self, filter: impl Into<String>) {
        self.filter = filter.into();
        self.apply_filter();
    }

    /// Clears the filter.
    pub fn clear_filter(&mut self) {
        self.filter.clear();
        self.apply_filter();
    }

    /// Moves selection up.
    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.update_scroll();
        }
    }

    /// Moves selection down.
    pub fn move_down(&mut self) {
        if self.selected < self.filtered_indices.len().saturating_sub(1) {
            self.selected += 1;
            self.update_scroll();
        }
    }

    /// Moves selection to top.
    pub fn move_to_top(&mut self) {
        self.selected = 0;
        self.update_scroll();
    }

    /// Moves selection to bottom.
    pub fn move_to_bottom(&mut self) {
        self.selected = self.filtered_indices.len().saturating_sub(1);
        self.update_scroll();
    }

    /// Pages up.
    pub fn page_up(&mut self) {
        self.selected = self.selected.saturating_sub(self.height);
        self.update_scroll();
    }

    /// Pages down.
    pub fn page_down(&mut self) {
        let max = self.filtered_indices.len().saturating_sub(1);
        self.selected = (self.selected + self.height).min(max);
        self.update_scroll();
    }

    fn update_scroll(&mut self) {
        // Ensure selected item is visible
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + self.height {
            self.scroll_offset = self.selected - self.height + 1;
        }
    }

    /// Navigates into the selected directory.
    pub fn enter_directory(&mut self) -> io::Result<bool> {
        if let Some(entry) = self.selected_entry()
            && entry.is_directory()
        {
            self.current_dir = entry.path.clone();
            self.selected = 0;
            self.scroll_offset = 0;
            self.clear_filter();
            self.scan()?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Navigates to parent directory.
    pub fn go_parent(&mut self) -> io::Result<bool> {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.selected = 0;
            self.scroll_offset = 0;
            self.clear_filter();
            self.scan()?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Toggles hidden files visibility.
    pub fn toggle_hidden(&mut self) -> io::Result<()> {
        self.config.show_hidden = !self.config.show_hidden;
        self.scan()
    }

    /// Enters filter mode.
    pub fn enter_filter_mode(&mut self) {
        self.filter_mode = true;
    }

    /// Exits filter mode.
    pub fn exit_filter_mode(&mut self) {
        self.filter_mode = false;
    }

    /// Handles a character input for filter.
    pub fn filter_input(&mut self, c: char) {
        self.filter.push(c);
        self.apply_filter();
    }

    /// Handles backspace in filter mode.
    pub fn filter_backspace(&mut self) {
        self.filter.pop();
        self.apply_filter();
    }
}

impl Model for FileBrowser {
    fn init(&self) -> Option<Cmd> {
        // Return a command to scan the directory
        let dir = self.current_dir.clone();
        Some(Cmd::new(move || {
            Message::new(ScanCompleteMsg { path: dir })
        }))
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle scan complete
        if msg.downcast_ref::<ScanCompleteMsg>().is_some() {
            let _ = self.scan();
            return None;
        }

        // Handle key messages
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            // Filter mode handling
            if self.filter_mode {
                match key.key_type {
                    KeyType::Esc => {
                        self.exit_filter_mode();
                        self.clear_filter();
                    }
                    KeyType::Enter => {
                        self.exit_filter_mode();
                    }
                    KeyType::Backspace => {
                        self.filter_backspace();
                    }
                    KeyType::Runes if !key.runes.is_empty() => {
                        for c in &key.runes {
                            self.filter_input(*c);
                        }
                    }
                    _ => {}
                }
                return None;
            }

            // Normal mode handling
            match key.key_type {
                KeyType::Up => self.move_up(),
                KeyType::Down => self.move_down(),
                KeyType::PgUp => self.page_up(),
                KeyType::PgDown => self.page_down(),
                KeyType::Home => self.move_to_top(),
                KeyType::End => self.move_to_bottom(),
                KeyType::Enter => {
                    if let Some(entry) = self.selected_entry() {
                        if entry.is_directory() {
                            let _ = self.enter_directory();
                        } else {
                            // Return the selected file path
                            return Some(Cmd::new({
                                let path = entry.path.clone();
                                move || Message::new(FileSelectedMsg { path })
                            }));
                        }
                    }
                }
                KeyType::Backspace => {
                    let _ = self.go_parent();
                }
                KeyType::Runes if !key.runes.is_empty() => match key.runes[0] {
                    'j' => self.move_down(),
                    'k' => self.move_up(),
                    'g' => self.move_to_top(),
                    'G' => self.move_to_bottom(),
                    '/' => self.enter_filter_mode(),
                    '.' => {
                        let _ = self.toggle_hidden();
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        None
    }

    fn view(&self) -> String {
        let mut lines = Vec::new();

        // Header: current directory
        let header = format!(" {} ", self.current_dir.display());
        let header_style = Style::new().bold().foreground("#7D56F4");
        lines.push(header_style.render(&header));
        lines.push(String::new());

        // Filter bar (if in filter mode or has filter)
        if self.filter_mode || !self.filter.is_empty() {
            let filter_display = if self.filter_mode {
                format!("/{}_", self.filter)
            } else {
                format!("/{}", self.filter)
            };
            let filter_style = Style::new().foreground("#FFCC00");
            lines.push(filter_style.render(&filter_display));
        }

        // Entries
        if self.filtered_indices.is_empty() {
            let empty_msg = if self.filter.is_empty() {
                "No markdown files found"
            } else {
                "No matches"
            };
            let empty_style = Style::new().foreground("#666666").italic();
            lines.push(empty_style.render(empty_msg));
        } else {
            let visible_end = (self.scroll_offset + self.height).min(self.filtered_indices.len());
            for (view_idx, &entry_idx) in self.filtered_indices[self.scroll_offset..visible_end]
                .iter()
                .enumerate()
            {
                if let Some(entry) = self.entries.get(entry_idx) {
                    let is_selected = self.scroll_offset + view_idx == self.selected;
                    let is_dir = entry.is_directory();
                    let is_markdown = self.is_markdown_path(&entry.path);

                    let prefix = if is_dir {
                        "📁 "
                    } else if is_markdown {
                        "📄 "
                    } else {
                        "   "
                    };

                    let line = format!(
                        "{}{:<40} {:>8}",
                        prefix,
                        truncate_string(&entry.name, 40),
                        entry.size_display()
                    );

                    let styled = if is_selected {
                        self.selected_style.render(&line)
                    } else if is_dir {
                        self.dir_style.render(&line)
                    } else {
                        self.file_style.render(&line)
                    };

                    lines.push(styled);
                }
            }
        }

        // Footer with help
        lines.push(String::new());
        let help =
            "↑/k up • ↓/j down • enter open • backspace parent • / filter • . hidden • q quit";
        let help_style = Style::new().foreground("#666666");
        lines.push(help_style.render(help));

        lines.join("\n")
    }
}

/// Message sent when directory scan completes.
#[derive(Debug, Clone)]
pub struct ScanCompleteMsg {
    /// The directory that was scanned
    pub path: PathBuf,
}

/// Message sent when a file is selected.
#[derive(Debug, Clone)]
pub struct FileSelectedMsg {
    /// Path to the selected file
    pub path: PathBuf,
}

/// Checks if a path is a markdown file.
fn is_markdown_file(path: &Path) -> bool {
    is_markdown_with_extensions(path, &[])
}

const DEFAULT_MARKDOWN_EXTENSIONS: [&str; 4] = ["md", "markdown", "mdown", "mkd"];

fn is_markdown_with_extensions(path: &Path, extensions: &[String]) -> bool {
    let ext = match path.extension().and_then(OsStr::to_str) {
        Some(ext) => ext,
        None => return false,
    };
    let ext = ext.to_ascii_lowercase();

    if extensions.is_empty() {
        return DEFAULT_MARKDOWN_EXTENSIONS
            .iter()
            .any(|candidate| ext == *candidate);
    }

    extensions.iter().any(|candidate| {
        let candidate = candidate.trim_start_matches('.').to_ascii_lowercase();
        !candidate.is_empty() && ext == candidate
    })
}

/// Formats a file size in human-readable form.
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1}G", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}M", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}K", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

/// Truncates a string to a maximum width.
fn truncate_string(s: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    if s.chars().count() <= max_width {
        s.to_string()
    } else {
        if max_width == 1 {
            return "…".to_string();
        }
        let truncated: String = s.chars().take(max_width - 1).collect();
        format!("{}…", truncated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    fn setup_test_dir() -> TempDir {
        let dir = TempDir::new().unwrap();

        // Create test files
        File::create(dir.path().join("README.md")).unwrap();
        File::create(dir.path().join("CHANGELOG.md")).unwrap();
        File::create(dir.path().join("notes.txt")).unwrap();
        File::create(dir.path().join(".hidden.md")).unwrap();

        // Create subdirectory
        std::fs::create_dir(dir.path().join("docs")).unwrap();
        File::create(dir.path().join("docs/guide.md")).unwrap();

        dir
    }

    #[test]
    fn test_browser_scan_finds_markdown_files() {
        let dir = setup_test_dir();
        let mut browser =
            FileBrowser::with_directory(dir.path(), BrowserConfig::default()).unwrap();
        browser.scan().unwrap();

        // Should find: README.md, CHANGELOG.md, docs/
        assert_eq!(browser.entries.len(), 3);

        let names: Vec<_> = browser.entries.iter().map(|e| &e.name).collect();
        assert!(names.contains(&&"docs".to_string()));
        assert!(names.contains(&&"CHANGELOG.md".to_string()));
        assert!(names.contains(&&"README.md".to_string()));
    }

    #[test]
    fn test_browser_scan_with_hidden_files() {
        let dir = setup_test_dir();
        let config = BrowserConfig {
            show_hidden: true,
            ..Default::default()
        };
        let mut browser = FileBrowser::with_directory(dir.path(), config).unwrap();
        browser.scan().unwrap();

        let names: Vec<_> = browser.entries.iter().map(|e| &e.name).collect();
        assert!(names.contains(&&".hidden.md".to_string()));
    }

    #[test]
    fn test_browser_scan_with_custom_extensions() {
        let dir = setup_test_dir();
        let config = BrowserConfig {
            extensions: vec!["txt".to_string()],
            ..Default::default()
        };
        let mut browser = FileBrowser::with_directory(dir.path(), config).unwrap();
        browser.scan().unwrap();

        let names: Vec<_> = browser.entries.iter().map(|e| &e.name).collect();
        assert!(names.contains(&&"notes.txt".to_string()));
        assert!(!names.contains(&&"README.md".to_string()));
    }

    #[test]
    fn test_browser_navigation() {
        let dir = setup_test_dir();
        let mut browser =
            FileBrowser::with_directory(dir.path(), BrowserConfig::default()).unwrap();
        browser.scan().unwrap();

        assert_eq!(browser.selected, 0);

        browser.move_down();
        assert_eq!(browser.selected, 1);

        browser.move_up();
        assert_eq!(browser.selected, 0);

        browser.move_to_bottom();
        assert_eq!(browser.selected, browser.filtered_indices.len() - 1);

        browser.move_to_top();
        assert_eq!(browser.selected, 0);
    }

    #[test]
    fn test_browser_filter() {
        let dir = setup_test_dir();
        let mut browser =
            FileBrowser::with_directory(dir.path(), BrowserConfig::default()).unwrap();
        browser.scan().unwrap();

        let initial_count = browser.filtered_indices.len();
        browser.set_filter("READ");

        assert!(browser.filtered_indices.len() < initial_count);
        assert_eq!(browser.filtered_entries().len(), 1);
        assert_eq!(browser.filtered_entries()[0].name, "README.md");
    }

    #[test]
    fn test_entry_from_path() {
        let dir = setup_test_dir();
        let md_path = dir.path().join("README.md");
        let entry = Entry::from_path(&md_path).unwrap();

        assert_eq!(entry.name, "README.md");
        assert!(entry.is_markdown());
        assert!(!entry.is_directory());
    }

    #[test]
    fn test_is_markdown_file() {
        assert!(is_markdown_file(Path::new("test.md")));
        assert!(is_markdown_file(Path::new("test.markdown")));
        assert!(is_markdown_file(Path::new("test.mdown")));
        assert!(is_markdown_file(Path::new("test.mkd")));
        assert!(!is_markdown_file(Path::new("test.txt")));
        assert!(!is_markdown_file(Path::new("test.rs")));
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(100), "100B");
        assert_eq!(format_size(1024), "1.0K");
        assert_eq!(format_size(1024 * 1024), "1.0M");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0G");
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(
            truncate_string("this is a very long string", 10),
            "this is a…"
        );
        assert_eq!(truncate_string("hello", 1), "…");
        assert_eq!(truncate_string("hello", 0), "");
    }

    #[test]
    fn test_model_init_returns_scan_command() {
        let browser = FileBrowser::new(BrowserConfig::default());
        let cmd = browser.init();
        assert!(cmd.is_some());
    }

    #[test]
    fn test_model_view_renders() {
        let dir = setup_test_dir();
        let mut browser =
            FileBrowser::with_directory(dir.path(), BrowserConfig::default()).unwrap();
        browser.scan().unwrap();

        let view = browser.view();
        assert!(!view.is_empty());
        // Should contain help text
        assert!(view.contains("quit"));
    }
}
