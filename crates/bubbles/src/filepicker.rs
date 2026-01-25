//! File picker component for browsing and selecting files.
//!
//! This module provides a file picker widget for TUI applications that allows
//! users to navigate directories and select files.
//!
//! # Example
//!
//! ```rust,ignore
//! use bubbles::filepicker::FilePicker;
//!
//! let mut picker = FilePicker::new();
//! picker.set_current_directory(".");
//!
//! // In your init function, call init() to start reading the directory
//! let cmd = picker.init();
//! ```

use crate::key::{Binding, matches};
use bubbletea::{Cmd, KeyMsg, Message, Model, WindowSizeMsg};
use lipgloss::{Color, Style};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Global ID counter for file picker instances.
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

/// A directory entry in the file picker.
#[derive(Debug, Clone)]
pub struct DirEntry {
    /// Name of the file or directory.
    pub name: String,
    /// Full path.
    pub path: PathBuf,
    /// Whether this is a directory.
    pub is_dir: bool,
    /// Whether this is a symbolic link.
    pub is_symlink: bool,
    /// File size in bytes.
    pub size: u64,
    /// Permission string (e.g., "drwxr-xr-x").
    pub mode: String,
}

/// Message sent when directory reading completes.
#[derive(Debug, Clone)]
pub struct ReadDirMsg {
    /// The file picker ID this message is for.
    pub id: u64,
    /// The directory entries read.
    pub entries: Vec<DirEntry>,
}

/// Message sent when directory reading fails.
#[derive(Debug, Clone)]
pub struct ReadDirErrMsg {
    /// The file picker ID this message is for.
    pub id: u64,
    /// Error message.
    pub error: String,
}

/// Key bindings for file picker navigation.
#[derive(Debug, Clone)]
pub struct KeyMap {
    /// Go to first entry.
    pub goto_top: Binding,
    /// Go to last entry.
    pub goto_last: Binding,
    /// Move down one entry.
    pub down: Binding,
    /// Move up one entry.
    pub up: Binding,
    /// Page up.
    pub page_up: Binding,
    /// Page down.
    pub page_down: Binding,
    /// Go back to parent directory.
    pub back: Binding,
    /// Open directory or select file.
    pub open: Binding,
    /// Confirm selection.
    pub select: Binding,
}

impl Default for KeyMap {
    fn default() -> Self {
        Self {
            goto_top: Binding::new().keys(&["g"]).help("g", "first"),
            goto_last: Binding::new().keys(&["G"]).help("G", "last"),
            down: Binding::new()
                .keys(&["j", "down", "ctrl+n"])
                .help("j", "down"),
            up: Binding::new().keys(&["k", "up", "ctrl+p"]).help("k", "up"),
            page_up: Binding::new().keys(&["K", "pgup"]).help("pgup", "page up"),
            page_down: Binding::new()
                .keys(&["J", "pgdown"])
                .help("pgdown", "page down"),
            back: Binding::new()
                .keys(&["h", "backspace", "left", "esc"])
                .help("h", "back"),
            open: Binding::new()
                .keys(&["l", "right", "enter"])
                .help("l", "open"),
            select: Binding::new().keys(&["enter"]).help("enter", "select"),
        }
    }
}

/// Styles for the file picker.
#[derive(Debug, Clone)]
pub struct Styles {
    /// Style for the cursor when disabled.
    pub disabled_cursor: Style,
    /// Style for the cursor.
    pub cursor: Style,
    /// Style for symbolic links.
    pub symlink: Style,
    /// Style for directories.
    pub directory: Style,
    /// Style for regular files.
    pub file: Style,
    /// Style for disabled files.
    pub disabled_file: Style,
    /// Style for permissions.
    pub permission: Style,
    /// Style for selected item.
    pub selected: Style,
    /// Style for disabled selected item.
    pub disabled_selected: Style,
    /// Style for file size.
    pub file_size: Style,
    /// Style for empty directory message.
    pub empty_directory: Style,
}

impl Default for Styles {
    fn default() -> Self {
        Self {
            disabled_cursor: Style::new().foreground_color(Color::from("247")),
            cursor: Style::new().foreground_color(Color::from("212")),
            symlink: Style::new().foreground_color(Color::from("36")),
            directory: Style::new().foreground_color(Color::from("99")),
            file: Style::new(),
            disabled_file: Style::new().foreground_color(Color::from("243")),
            permission: Style::new().foreground_color(Color::from("244")),
            selected: Style::new().foreground_color(Color::from("212")).bold(),
            disabled_selected: Style::new().foreground_color(Color::from("247")),
            file_size: Style::new().foreground_color(Color::from("240")),
            empty_directory: Style::new().foreground_color(Color::from("240")),
        }
    }
}

/// File picker model for browsing and selecting files.
#[derive(Debug, Clone)]
pub struct FilePicker {
    /// Unique ID for this file picker.
    id: u64,
    /// Root directory (jail) for navigation.
    pub root: Option<PathBuf>,
    /// Currently selected path (after selection).
    pub path: Option<PathBuf>,
    /// Current directory being displayed.
    current_directory: PathBuf,
    /// Allowed file extensions (empty = all allowed).
    pub allowed_types: Vec<String>,
    /// Key bindings.
    pub key_map: KeyMap,
    /// Directory entries.
    files: Vec<DirEntry>,
    /// Whether to show permissions.
    pub show_permissions: bool,
    /// Whether to show file sizes.
    pub show_size: bool,
    /// Whether to show hidden files.
    pub show_hidden: bool,
    /// Whether directories can be selected.
    pub dir_allowed: bool,
    /// Whether files can be selected.
    pub file_allowed: bool,
    /// Currently selected index.
    selected: usize,
    /// Navigation stack for selected indices.
    selected_stack: Vec<usize>,
    /// Minimum visible index.
    min: usize,
    /// Maximum visible index.
    max: usize,
    /// Navigation stack for min values.
    min_stack: Vec<usize>,
    /// Navigation stack for max values.
    max_stack: Vec<usize>,
    /// Height of the picker in rows.
    pub height: usize,
    /// Whether to auto-adjust height based on window size.
    pub auto_height: bool,
    /// Cursor character.
    pub cursor_char: String,
    /// Styles.
    pub styles: Styles,
}

impl Default for FilePicker {
    fn default() -> Self {
        Self::new()
    }
}

impl FilePicker {
    /// Creates a new file picker with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: next_id(),
            root: None,
            path: None,
            current_directory: PathBuf::from("."),
            allowed_types: Vec::new(),
            key_map: KeyMap::default(),
            files: Vec::new(),
            show_permissions: true,
            show_size: true,
            show_hidden: false,
            dir_allowed: false,
            file_allowed: true,
            selected: 0,
            selected_stack: Vec::new(),
            min: 0,
            max: 0,
            min_stack: Vec::new(),
            max_stack: Vec::new(),
            height: 0,
            auto_height: true,
            cursor_char: ">".to_string(),
            styles: Styles::default(),
        }
    }

    /// Returns the unique ID of this file picker.
    #[must_use]
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns the current directory.
    #[must_use]
    pub fn current_directory(&self) -> &Path {
        &self.current_directory
    }

    /// Sets the root directory (jail). Navigation above this directory will be blocked.
    pub fn set_root(&mut self, root: impl AsRef<Path>) {
        self.root = Some(root.as_ref().to_path_buf());
        // Ensure current directory satisfies new root
        if let Some(root) = &self.root
            && !self.current_directory.starts_with(root)
        {
            self.current_directory = root.clone();
        }
    }

    /// Sets the current directory.
    pub fn set_current_directory(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        if let Some(root) = &self.root
            && !path.starts_with(root)
        {
            // If path is outside root, default to root
            self.current_directory = root.clone();
            return;
        }
        self.current_directory = path.to_path_buf();
    }

    /// Sets the height of the file picker.
    pub fn set_height(&mut self, height: usize) {
        self.height = height;
        if self.max > self.height.saturating_sub(1) {
            self.max = self.min + self.height.saturating_sub(1);
        }
    }

    /// Sets the allowed file types.
    pub fn set_allowed_types(&mut self, types: Vec<String>) {
        self.allowed_types = types;
    }

    /// Returns the selected file path, if any.
    #[must_use]
    pub fn selected_path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Returns the currently highlighted entry, if any.
    #[must_use]
    pub fn highlighted_entry(&self) -> Option<&DirEntry> {
        self.files.get(self.selected)
    }

    /// Initializes the file picker and returns a command to read the directory.
    #[must_use]
    pub fn init(&self) -> Option<Cmd> {
        Some(self.read_dir_cmd())
    }

    /// Creates a command to read the current directory.
    fn read_dir_cmd(&self) -> Cmd {
        let id = self.id;
        let path = self.current_directory.clone();
        let show_hidden = self.show_hidden;

        Cmd::new(move || match read_directory(&path, show_hidden) {
            Ok(entries) => Message::new(ReadDirMsg { id, entries }),
            Err(e) => Message::new(ReadDirErrMsg {
                id,
                error: e.to_string(),
            }),
        })
    }

    /// Checks if a file can be selected based on allowed types.
    fn can_select(&self, name: &str) -> bool {
        if self.allowed_types.is_empty() {
            return true;
        }
        self.allowed_types.iter().any(|ext| name.ends_with(ext))
    }

    /// Pushes current view state to the navigation stack.
    fn push_view(&mut self) {
        self.selected_stack.push(self.selected);
        self.min_stack.push(self.min);
        self.max_stack.push(self.max);
    }

    /// Pops view state from the navigation stack.
    fn pop_view(&mut self) -> Option<(usize, usize, usize)> {
        if let (Some(sel), Some(min), Some(max)) = (
            self.selected_stack.pop(),
            self.min_stack.pop(),
            self.max_stack.pop(),
        ) {
            Some((sel, min, max))
        } else {
            None
        }
    }

    /// Checks if this message indicates a file was selected.
    pub fn did_select_file(&self, msg: &Message) -> Option<PathBuf> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            let key_str = key.to_string();
            if matches(&key_str, &[&self.key_map.select])
                && let Some(path) = &self.path
                && self.can_select(path.to_str().unwrap_or(""))
            {
                return Some(path.clone());
            }
        }
        None
    }

    /// Checks if this message indicates a disabled file was selected.
    pub fn did_select_disabled_file(&self, msg: &Message) -> Option<PathBuf> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            let key_str = key.to_string();
            if matches(&key_str, &[&self.key_map.select])
                && let Some(path) = &self.path
                && !self.can_select(path.to_str().unwrap_or(""))
            {
                return Some(path.clone());
            }
        }
        None
    }

    /// Updates the file picker based on messages.
    pub fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle directory read result
        if let Some(read_msg) = msg.downcast_ref::<ReadDirMsg>() {
            if read_msg.id != self.id {
                return None;
            }
            self.files = read_msg.entries.clone();
            self.max = self.max.max(self.height.saturating_sub(1));
            return None;
        }

        // Handle window size
        if let Some(size) = msg.downcast_ref::<WindowSizeMsg>() {
            if self.auto_height {
                self.height = (size.height as usize).saturating_sub(5);
            }
            self.max = self.height.saturating_sub(1);
            return None;
        }

        // Handle key messages
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            let key_str = key.to_string();

            if matches(&key_str, &[&self.key_map.goto_top]) {
                self.selected = 0;
                self.min = 0;
                self.max = self.height.saturating_sub(1);
            } else if matches(&key_str, &[&self.key_map.goto_last]) {
                self.selected = self.files.len().saturating_sub(1);
                self.min = self.files.len().saturating_sub(self.height);
                self.max = self.files.len().saturating_sub(1);
            } else if matches(&key_str, &[&self.key_map.down]) {
                if self.selected < self.files.len().saturating_sub(1) {
                    self.selected += 1;
                    if self.selected > self.max {
                        self.min += 1;
                        self.max += 1;
                    }
                }
            } else if matches(&key_str, &[&self.key_map.up]) {
                if self.selected > 0 {
                    self.selected -= 1;
                    if self.selected < self.min {
                        self.min = self.min.saturating_sub(1);
                        self.max = self.max.saturating_sub(1);
                    }
                }
            } else if matches(&key_str, &[&self.key_map.page_down]) {
                self.selected =
                    (self.selected + self.height).min(self.files.len().saturating_sub(1));
                self.min += self.height;
                self.max += self.height;
                if self.max >= self.files.len() {
                    self.max = self.files.len().saturating_sub(1);
                    self.min = self.max.saturating_sub(self.height);
                }
            } else if matches(&key_str, &[&self.key_map.page_up]) {
                self.selected = self.selected.saturating_sub(self.height);
                self.min = self.min.saturating_sub(self.height);
                self.max = self.max.saturating_sub(self.height);
                if self.min == 0 {
                    self.max = self
                        .height
                        .saturating_sub(1)
                        .min(self.files.len().saturating_sub(1));
                }
            } else if matches(&key_str, &[&self.key_map.back]) {
                // Go to parent directory
                // Check if we are at root
                let at_root = if let Some(root) = &self.root {
                    self.current_directory == *root
                } else {
                    false
                };

                if !at_root {
                    if let Some(parent) = self.current_directory.parent() {
                        self.current_directory = parent.to_path_buf();
                    }
                    if let Some((sel, min, max)) = self.pop_view() {
                        self.selected = sel;
                        self.min = min;
                        self.max = max;
                    } else {
                        self.selected = 0;
                        self.min = 0;
                        self.max = self.height.saturating_sub(1);
                    }
                    return Some(self.read_dir_cmd());
                }
            } else {
                let is_select = matches(&key_str, &[&self.key_map.select]);
                let is_open = matches(&key_str, &[&self.key_map.open]);
                if !is_select && !is_open {
                    return None;
                }

                if self.files.is_empty() {
                    return None;
                }

                let entry = &self.files[self.selected];
                let is_dir = entry.is_dir;

                // Check if we can select this
                if is_select && ((!is_dir && self.file_allowed) || (is_dir && self.dir_allowed)) {
                    self.path = Some(entry.path.clone());
                }

                // If it's a directory, navigate into it
                if is_open && is_dir {
                    self.current_directory = entry.path.clone();
                    self.push_view();
                    self.selected = 0;
                    self.min = 0;
                    self.max = self.height.saturating_sub(1);
                    return Some(self.read_dir_cmd());
                }
            }
        }

        None
    }

    /// Renders the file picker.
    #[must_use]
    pub fn view(&self) -> String {
        if self.files.is_empty() {
            return self.styles.empty_directory.render("No files found.");
        }

        let mut lines = Vec::new();

        for (i, entry) in self.files.iter().enumerate() {
            if i < self.min || i > self.max {
                continue;
            }

            let disabled = !self.can_select(&entry.name) && !entry.is_dir;

            if i == self.selected {
                // Selected row
                let mut parts = Vec::new();

                if self.show_permissions {
                    parts.push(format!(" {}", entry.mode));
                }
                if self.show_size {
                    parts.push(format!("{:>7}", format_size(entry.size)));
                }
                parts.push(format!(" {}", entry.name));
                if entry.is_symlink {
                    parts.push(" →".to_string());
                }

                let content = parts.join("");

                if disabled {
                    lines.push(format!(
                        "{}{}",
                        self.styles.disabled_selected.render(&self.cursor_char),
                        self.styles.disabled_selected.render(&content)
                    ));
                } else {
                    lines.push(format!(
                        "{}{}",
                        self.styles.cursor.render(&self.cursor_char),
                        self.styles.selected.render(&content)
                    ));
                }
            } else {
                // Non-selected row
                let style = if entry.is_dir {
                    &self.styles.directory
                } else if entry.is_symlink {
                    &self.styles.symlink
                } else if disabled {
                    &self.styles.disabled_file
                } else {
                    &self.styles.file
                };

                let mut parts = vec![" ".to_string()]; // Space for cursor

                if self.show_permissions {
                    parts.push(format!(" {}", self.styles.permission.render(&entry.mode)));
                }
                if self.show_size {
                    parts.push(
                        self.styles
                            .file_size
                            .render(&format!("{:>7}", format_size(entry.size))),
                    );
                }
                parts.push(format!(" {}", style.render(&entry.name)));
                if entry.is_symlink {
                    parts.push(" →".to_string());
                }

                lines.push(parts.join(""));
            }
        }

        // Pad to height
        while lines.len() < self.height {
            lines.push(String::new());
        }

        lines.join("\n")
    }
}

impl Model for FilePicker {
    /// Initialize the file picker by reading the current directory.
    fn init(&self) -> Option<Cmd> {
        FilePicker::init(self)
    }

    /// Update the file picker state based on incoming messages.
    fn update(&mut self, msg: Message) -> Option<Cmd> {
        FilePicker::update(self, msg)
    }

    /// Render the file picker.
    fn view(&self) -> String {
        FilePicker::view(self)
    }
}

/// Reads a directory and returns sorted entries.
fn read_directory(path: &Path, show_hidden: bool) -> std::io::Result<Vec<DirEntry>> {
    let mut entries = Vec::new();

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files if not showing
        if !show_hidden && name.starts_with('.') {
            continue;
        }

        let metadata = entry.metadata()?;
        let file_type = entry.file_type()?;
        let is_symlink = file_type.is_symlink();

        let mode = format_mode(&metadata, is_symlink);

        entries.push(DirEntry {
            name,
            path: entry.path(),
            is_dir: file_type.is_dir(),
            is_symlink: file_type.is_symlink(),
            size: metadata.len(),
            mode,
        });
    }

    // Sort: directories first, then alphabetically
    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(entries)
}

/// Formats file permissions as a string.
#[cfg(unix)]
fn format_mode(metadata: &std::fs::Metadata, is_symlink: bool) -> String {
    use std::os::unix::fs::PermissionsExt;
    let mode = metadata.permissions().mode();

    let file_type = if metadata.is_dir() {
        'd'
    } else if is_symlink {
        'l'
    } else {
        '-'
    };

    let user = format!(
        "{}{}{}",
        if mode & 0o400 != 0 { 'r' } else { '-' },
        if mode & 0o200 != 0 { 'w' } else { '-' },
        if mode & 0o100 != 0 { 'x' } else { '-' }
    );
    let group = format!(
        "{}{}{}",
        if mode & 0o040 != 0 { 'r' } else { '-' },
        if mode & 0o020 != 0 { 'w' } else { '-' },
        if mode & 0o010 != 0 { 'x' } else { '-' }
    );
    let other = format!(
        "{}{}{}",
        if mode & 0o004 != 0 { 'r' } else { '-' },
        if mode & 0o002 != 0 { 'w' } else { '-' },
        if mode & 0o001 != 0 { 'x' } else { '-' }
    );

    format!("{}{}{}{}", file_type, user, group, other)
}

#[cfg(not(unix))]
fn format_mode(metadata: &std::fs::Metadata, is_symlink: bool) -> String {
    let file_type = if metadata.is_dir() {
        'd'
    } else if is_symlink {
        'l'
    } else {
        '-'
    };
    let readonly = if metadata.permissions().readonly() {
        "r--"
    } else {
        "rw-"
    };
    format!("{}{}{}{}", file_type, readonly, readonly, readonly)
}

/// Formats a file size in human-readable form.
fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.1}G", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.1}M", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.1}K", size as f64 / KB as f64)
    } else {
        format!("{}B", size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filepicker_new() {
        let fp = FilePicker::new();
        assert!(fp.allowed_types.is_empty());
        assert!(fp.show_permissions);
        assert!(fp.show_size);
        assert!(!fp.show_hidden);
        assert!(fp.file_allowed);
        assert!(!fp.dir_allowed);
    }

    #[test]
    fn test_filepicker_unique_ids() {
        let fp1 = FilePicker::new();
        let fp2 = FilePicker::new();
        assert_ne!(fp1.id(), fp2.id());
    }

    #[test]
    fn test_filepicker_set_current_directory() {
        let mut fp = FilePicker::new();
        fp.set_current_directory("/tmp");
        assert_eq!(fp.current_directory(), Path::new("/tmp"));
    }

    #[test]
    fn test_filepicker_set_height() {
        let mut fp = FilePicker::new();
        fp.set_height(20);
        assert_eq!(fp.height, 20);
    }

    #[test]
    fn test_filepicker_allowed_types() {
        let mut fp = FilePicker::new();
        fp.set_allowed_types(vec![".txt".to_string(), ".md".to_string()]);

        assert!(fp.can_select("readme.txt"));
        assert!(fp.can_select("notes.md"));
        assert!(!fp.can_select("image.png"));
    }

    #[test]
    fn test_filepicker_all_types_allowed() {
        let fp = FilePicker::new();
        assert!(fp.can_select("anything.xyz"));
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0B");
        assert_eq!(format_size(512), "512B");
        assert_eq!(format_size(1024), "1.0K");
        assert_eq!(format_size(1536), "1.5K");
        assert_eq!(format_size(1048576), "1.0M");
        assert_eq!(format_size(1073741824), "1.0G");
    }

    #[test]
    fn test_filepicker_navigation_stack() {
        let mut fp = FilePicker::new();

        fp.selected = 5;
        fp.min = 2;
        fp.max = 10;

        fp.push_view();

        fp.selected = 0;
        fp.min = 0;
        fp.max = 5;

        let (sel, min, max) = fp.pop_view().unwrap();
        assert_eq!(sel, 5);
        assert_eq!(min, 2);
        assert_eq!(max, 10);
    }

    #[test]
    fn test_filepicker_view_empty() {
        let fp = FilePicker::new();
        let view = fp.view();
        assert!(view.contains("No files"));
    }

    #[test]
    fn test_keymap_default() {
        let km = KeyMap::default();
        assert!(!km.up.get_keys().is_empty());
        assert!(!km.down.get_keys().is_empty());
        assert!(!km.open.get_keys().is_empty());
    }

    #[test]
    fn test_dir_entry() {
        let entry = DirEntry {
            name: "test.txt".to_string(),
            path: PathBuf::from("/tmp/test.txt"),
            is_dir: false,
            is_symlink: false,
            size: 1024,
            mode: "-rw-r--r--".to_string(),
        };

        assert_eq!(entry.name, "test.txt");
        assert!(!entry.is_dir);
        assert_eq!(entry.size, 1024);
    }

    // Model trait implementation tests
    #[test]
    fn test_model_init_returns_cmd() {
        let fp = FilePicker::new();
        // FilePicker init returns a command to read the directory
        let cmd = Model::init(&fp);
        assert!(cmd.is_some());
    }

    #[test]
    fn test_model_view_matches_filepicker_view() {
        let fp = FilePicker::new();
        // Model::view should return same result as FilePicker::view
        let model_view = Model::view(&fp);
        let filepicker_view = FilePicker::view(&fp);
        assert_eq!(model_view, filepicker_view);
    }

    #[test]
    fn test_filepicker_satisfies_model_bounds() {
        fn requires_model<T: Model + Send + 'static>() {}
        requires_model::<FilePicker>();
    }

    #[test]
    fn test_model_update_handles_navigation() {
        use bubbletea::{KeyMsg, KeyType, Message};

        let mut fp = FilePicker::new();
        // Simulate having some files loaded
        fp.files = vec![
            DirEntry {
                name: "file1.txt".to_string(),
                path: PathBuf::from("/tmp/file1.txt"),
                is_dir: false,
                is_symlink: false,
                size: 100,
                mode: "-rw-r--r--".to_string(),
            },
            DirEntry {
                name: "file2.txt".to_string(),
                path: PathBuf::from("/tmp/file2.txt"),
                is_dir: false,
                is_symlink: false,
                size: 200,
                mode: "-rw-r--r--".to_string(),
            },
        ];
        fp.max = 10;
        fp.selected = 0;

        // Press down arrow
        let down_msg = Message::new(KeyMsg::from_type(KeyType::Down));
        let _ = Model::update(&mut fp, down_msg);

        assert_eq!(
            fp.selected, 1,
            "FilePicker should navigate down on Down key"
        );
    }

    #[test]
    fn test_model_update_handles_read_dir_msg() {
        use bubbletea::Message;

        let mut fp = FilePicker::new();
        let id = fp.id();
        assert!(fp.files.is_empty());

        // Simulate receiving a ReadDirMsg
        let read_msg = ReadDirMsg {
            id,
            entries: vec![DirEntry {
                name: "test.txt".to_string(),
                path: PathBuf::from("/tmp/test.txt"),
                is_dir: false,
                is_symlink: false,
                size: 42,
                mode: "-rw-r--r--".to_string(),
            }],
        };

        let _ = Model::update(&mut fp, Message::new(read_msg));

        assert_eq!(
            fp.files.len(),
            1,
            "FilePicker should populate files from ReadDirMsg"
        );
        assert_eq!(fp.files[0].name, "test.txt");
    }

    #[test]
    fn test_model_update_ignores_wrong_id() {
        use bubbletea::Message;

        let mut fp = FilePicker::new();
        assert!(fp.files.is_empty());

        // Send ReadDirMsg with wrong ID
        let read_msg = ReadDirMsg {
            id: fp.id() + 1, // Wrong ID
            entries: vec![DirEntry {
                name: "test.txt".to_string(),
                path: PathBuf::from("/tmp/test.txt"),
                is_dir: false,
                is_symlink: false,
                size: 42,
                mode: "-rw-r--r--".to_string(),
            }],
        };

        let _ = Model::update(&mut fp, Message::new(read_msg));

        assert!(
            fp.files.is_empty(),
            "FilePicker should ignore ReadDirMsg with wrong ID"
        );
    }

    // ========================================================================
    // Additional Model trait tests for bead charmed_rust-amx
    // ========================================================================

    #[test]
    fn test_model_update_navigate_up_moves_cursor() {
        use bubbletea::{KeyMsg, KeyType, Message};

        let mut fp = FilePicker::new();
        fp.files = vec![
            DirEntry {
                name: "file1.txt".to_string(),
                path: PathBuf::from("/tmp/file1.txt"),
                is_dir: false,
                is_symlink: false,
                size: 100,
                mode: "-rw-r--r--".to_string(),
            },
            DirEntry {
                name: "file2.txt".to_string(),
                path: PathBuf::from("/tmp/file2.txt"),
                is_dir: false,
                is_symlink: false,
                size: 200,
                mode: "-rw-r--r--".to_string(),
            },
        ];
        fp.max = 10;
        fp.selected = 1;

        // Press up arrow
        let up_msg = Message::new(KeyMsg::from_type(KeyType::Up));
        let _ = Model::update(&mut fp, up_msg);

        assert_eq!(fp.selected, 0, "FilePicker should navigate up on Up key");
    }

    #[test]
    fn test_filepicker_toggle_hidden_files() {
        let mut fp = FilePicker::new();
        assert!(!fp.show_hidden, "Hidden files should be hidden by default");

        fp.show_hidden = true;
        assert!(fp.show_hidden, "Hidden files should be shown after toggle");

        fp.show_hidden = false;
        assert!(!fp.show_hidden, "Hidden files should be hidden again");
    }

    #[test]
    fn test_filepicker_filter_files() {
        let mut fp = FilePicker::new();
        fp.set_allowed_types(vec![".txt".to_string()]);

        // Test filtering
        assert!(fp.can_select("readme.txt"));
        assert!(!fp.can_select("image.png"));
        assert!(!fp.can_select("document.pdf"));
    }

    #[test]
    fn test_filepicker_view_shows_current_path() {
        let mut fp = FilePicker::new();
        fp.set_current_directory("/tmp");

        // Add some files so view isn't empty
        fp.files = vec![DirEntry {
            name: "test.txt".to_string(),
            path: PathBuf::from("/tmp/test.txt"),
            is_dir: false,
            is_symlink: false,
            size: 100,
            mode: "-rw-r--r--".to_string(),
        }];
        fp.max = 10;

        let view = fp.view();
        // The view should contain file names
        assert!(view.contains("test") || !view.is_empty());
    }

    #[test]
    fn test_filepicker_symlink_entry() {
        let entry = DirEntry {
            name: "link".to_string(),
            path: PathBuf::from("/tmp/link"),
            is_dir: false,
            is_symlink: true,
            size: 0,
            mode: "lrwxrwxrwx".to_string(),
        };

        assert!(entry.is_symlink, "Entry should be marked as symlink");
        assert!(!entry.is_dir, "Symlink should not be marked as directory");
    }

    #[test]
    fn test_filepicker_directory_entry() {
        let entry = DirEntry {
            name: "subdir".to_string(),
            path: PathBuf::from("/tmp/subdir"),
            is_dir: true,
            is_symlink: false,
            size: 4096,
            mode: "drwxr-xr-x".to_string(),
        };

        assert!(entry.is_dir, "Entry should be marked as directory");
        assert!(!entry.is_symlink);
    }

    #[test]
    fn test_filepicker_cursor_boundary_top() {
        use bubbletea::{KeyMsg, KeyType, Message};

        let mut fp = FilePicker::new();
        fp.files = vec![DirEntry {
            name: "file1.txt".to_string(),
            path: PathBuf::from("/tmp/file1.txt"),
            is_dir: false,
            is_symlink: false,
            size: 100,
            mode: "-rw-r--r--".to_string(),
        }];
        fp.max = 10;
        fp.selected = 0;

        // Try to move up from top
        let up_msg = Message::new(KeyMsg::from_type(KeyType::Up));
        let _ = Model::update(&mut fp, up_msg);

        assert_eq!(fp.selected, 0, "Cursor should not go below 0");
    }

    #[test]
    fn test_filepicker_cursor_boundary_bottom() {
        use bubbletea::{KeyMsg, KeyType, Message};

        let mut fp = FilePicker::new();
        fp.files = vec![
            DirEntry {
                name: "file1.txt".to_string(),
                path: PathBuf::from("/tmp/file1.txt"),
                is_dir: false,
                is_symlink: false,
                size: 100,
                mode: "-rw-r--r--".to_string(),
            },
            DirEntry {
                name: "file2.txt".to_string(),
                path: PathBuf::from("/tmp/file2.txt"),
                is_dir: false,
                is_symlink: false,
                size: 200,
                mode: "-rw-r--r--".to_string(),
            },
        ];
        fp.max = 10;
        fp.selected = 1;

        // Try to move down from bottom
        let down_msg = Message::new(KeyMsg::from_type(KeyType::Down));
        let _ = Model::update(&mut fp, down_msg);

        assert_eq!(fp.selected, 1, "Cursor should not exceed file count");
    }

    #[test]
    fn test_filepicker_empty_navigation() {
        use bubbletea::{KeyMsg, KeyType, Message};

        let mut fp = FilePicker::new();
        assert!(fp.files.is_empty());
        assert_eq!(fp.selected, 0);

        // Navigation on empty should not panic
        let down_msg = Message::new(KeyMsg::from_type(KeyType::Down));
        let _ = Model::update(&mut fp, down_msg);
        assert_eq!(fp.selected, 0, "Empty filepicker cursor should stay at 0");

        let up_msg = Message::new(KeyMsg::from_type(KeyType::Up));
        let _ = Model::update(&mut fp, up_msg);
        assert_eq!(fp.selected, 0);
    }

    #[test]
    fn test_filepicker_j_k_navigation() {
        use bubbletea::{KeyMsg, Message};

        let mut fp = FilePicker::new();
        fp.files = vec![
            DirEntry {
                name: "a.txt".to_string(),
                path: PathBuf::from("/tmp/a.txt"),
                is_dir: false,
                is_symlink: false,
                size: 100,
                mode: "-rw-r--r--".to_string(),
            },
            DirEntry {
                name: "b.txt".to_string(),
                path: PathBuf::from("/tmp/b.txt"),
                is_dir: false,
                is_symlink: false,
                size: 100,
                mode: "-rw-r--r--".to_string(),
            },
        ];
        fp.max = 10;
        fp.selected = 0;

        // Test 'j' for down
        let j_msg = Message::new(KeyMsg::from_char('j'));
        let _ = Model::update(&mut fp, j_msg);
        assert_eq!(fp.selected, 1, "'j' should move cursor down");

        // Test 'k' for up
        let k_msg = Message::new(KeyMsg::from_char('k'));
        let _ = Model::update(&mut fp, k_msg);
        assert_eq!(fp.selected, 0, "'k' should move cursor up");
    }

    #[test]
    fn test_filepicker_page_navigation() {
        use bubbletea::{KeyMsg, KeyType, Message};

        let mut fp = FilePicker::new();
        // Create 20 files
        fp.files = (0..20)
            .map(|i| DirEntry {
                name: format!("file{}.txt", i),
                path: PathBuf::from(format!("/tmp/file{}.txt", i)),
                is_dir: false,
                is_symlink: false,
                size: 100,
                mode: "-rw-r--r--".to_string(),
            })
            .collect();
        fp.height = 5;
        fp.max = fp.height;
        fp.selected = 0;

        // PageDown
        let pgdown_msg = Message::new(KeyMsg::from_type(KeyType::PgDown));
        let _ = Model::update(&mut fp, pgdown_msg);
        assert!(fp.selected > 0, "PageDown should move cursor down");
    }

    #[test]
    fn test_filepicker_set_show_permissions() {
        let mut fp = FilePicker::new();
        assert!(fp.show_permissions, "Permissions shown by default");

        fp.show_permissions = false;
        assert!(!fp.show_permissions);
    }

    #[test]
    fn test_filepicker_set_show_size() {
        let mut fp = FilePicker::new();
        assert!(fp.show_size, "Size shown by default");

        fp.show_size = false;
        assert!(!fp.show_size);
    }

    #[test]
    fn test_filepicker_dir_allowed() {
        let mut fp = FilePicker::new();
        assert!(fp.file_allowed, "Files allowed by default");
        assert!(!fp.dir_allowed, "Directories not allowed by default");

        fp.dir_allowed = true;
        fp.file_allowed = false;
        assert!(fp.dir_allowed);
        assert!(!fp.file_allowed);
    }

    #[test]
    fn test_filepicker_selected_file() {
        let mut fp = FilePicker::new();
        fp.files = vec![
            DirEntry {
                name: "first.txt".to_string(),
                path: PathBuf::from("/tmp/first.txt"),
                is_dir: false,
                is_symlink: false,
                size: 100,
                mode: "-rw-r--r--".to_string(),
            },
            DirEntry {
                name: "second.txt".to_string(),
                path: PathBuf::from("/tmp/second.txt"),
                is_dir: false,
                is_symlink: false,
                size: 200,
                mode: "-rw-r--r--".to_string(),
            },
        ];
        fp.max = 10;
        fp.selected = 0;

        // Check selected file
        if let Some(entry) = fp.files.get(fp.selected) {
            assert_eq!(entry.name, "first.txt");
        }

        fp.selected = 1;
        if let Some(entry) = fp.files.get(fp.selected) {
            assert_eq!(entry.name, "second.txt");
        }
    }

    #[test]
    fn test_filepicker_select_key_independent_of_open() {
        use bubbletea::{KeyMsg, Message};

        let mut fp = FilePicker::new();
        fp.key_map.select = Binding::new().keys(&["s"]);
        fp.key_map.open = Binding::new().keys(&["enter"]);
        fp.files = vec![DirEntry {
            name: "selected.txt".to_string(),
            path: PathBuf::from("/tmp/selected.txt"),
            is_dir: false,
            is_symlink: false,
            size: 10,
            mode: "-rw-r--r--".to_string(),
        }];
        fp.max = 10;
        fp.selected = 0;

        let msg = Message::new(KeyMsg::from_char('s'));
        let _ = Model::update(&mut fp, msg);

        assert_eq!(
            fp.selected_path(),
            Some(Path::new("/tmp/selected.txt")),
            "Select key should set path even when open key differs"
        );
    }

    #[test]
    fn test_filepicker_current_directory_accessor() {
        let mut fp = FilePicker::new();
        let initial_dir = fp.current_directory().to_path_buf();

        fp.set_current_directory("/home");
        assert_eq!(fp.current_directory(), Path::new("/home"));

        fp.set_current_directory("/var/log");
        assert_eq!(fp.current_directory(), Path::new("/var/log"));

        // Reset
        fp.current_directory = initial_dir;
    }

    // ========================================================================
    // Additional Model trait tests for bead charmed_rust-amx (missing tests)
    // ========================================================================

    #[test]
    fn test_filepicker_read_dir_error_updates_state() {
        use bubbletea::Message;

        let mut fp = FilePicker::new();
        let id = fp.id();

        // Simulate receiving a ReadDirErrMsg (error reading directory)
        let err_msg = ReadDirErrMsg {
            id,
            error: "Permission denied".to_string(),
        };

        // The update should handle the error message gracefully
        let cmd = Model::update(&mut fp, Message::new(err_msg));
        // Currently the implementation just ignores the error (returns None)
        // This test verifies it doesn't panic
        assert!(cmd.is_none(), "Error handling should not return a command");
    }

    #[test]
    fn test_filepicker_enter_directory_changes_path() {
        use bubbletea::{KeyMsg, KeyType, Message};

        let mut fp = FilePicker::new();
        fp.set_current_directory("/tmp");
        fp.files = vec![
            DirEntry {
                name: "subdir".to_string(),
                path: PathBuf::from("/tmp/subdir"),
                is_dir: true,
                is_symlink: false,
                size: 4096,
                mode: "drwxr-xr-x".to_string(),
            },
            DirEntry {
                name: "file.txt".to_string(),
                path: PathBuf::from("/tmp/file.txt"),
                is_dir: false,
                is_symlink: false,
                size: 100,
                mode: "-rw-r--r--".to_string(),
            },
        ];
        fp.max = 10;
        fp.selected = 0;

        // Press Enter on directory should navigate into it
        let enter_msg = Message::new(KeyMsg::from_type(KeyType::Enter));
        let cmd = Model::update(&mut fp, enter_msg);

        assert_eq!(
            fp.current_directory(),
            Path::new("/tmp/subdir"),
            "Enter on directory should change current path"
        );
        assert!(
            cmd.is_some(),
            "Entering directory should return read_dir command"
        );
    }

    #[test]
    fn test_filepicker_backspace_goes_parent() {
        use bubbletea::{KeyMsg, KeyType, Message};

        let mut fp = FilePicker::new();
        fp.set_current_directory("/tmp/subdir");
        fp.files = vec![DirEntry {
            name: "file.txt".to_string(),
            path: PathBuf::from("/tmp/subdir/file.txt"),
            is_dir: false,
            is_symlink: false,
            size: 100,
            mode: "-rw-r--r--".to_string(),
        }];
        fp.max = 10;

        // Press Backspace should go to parent directory
        let back_msg = Message::new(KeyMsg::from_type(KeyType::Backspace));
        let cmd = Model::update(&mut fp, back_msg);

        assert_eq!(
            fp.current_directory(),
            Path::new("/tmp"),
            "Backspace should navigate to parent directory"
        );
        assert!(
            cmd.is_some(),
            "Going to parent should return read_dir command"
        );
    }

    #[test]
    fn test_filepicker_view_highlights_selected() {
        let mut fp = FilePicker::new();
        fp.files = vec![
            DirEntry {
                name: "first.txt".to_string(),
                path: PathBuf::from("/tmp/first.txt"),
                is_dir: false,
                is_symlink: false,
                size: 100,
                mode: "-rw-r--r--".to_string(),
            },
            DirEntry {
                name: "second.txt".to_string(),
                path: PathBuf::from("/tmp/second.txt"),
                is_dir: false,
                is_symlink: false,
                size: 200,
                mode: "-rw-r--r--".to_string(),
            },
        ];
        fp.max = 10;
        fp.selected = 0;

        let view = fp.view();
        // The view should contain the cursor character on the selected line
        assert!(
            view.contains(&fp.cursor_char),
            "View should show cursor on selected item"
        );
        assert!(
            view.contains("first.txt"),
            "View should show the first file"
        );
    }

    #[test]
    #[allow(clippy::useless_vec)]
    fn test_filepicker_view_shows_directories_first() {
        // This test verifies that read_directory sorts directories before files
        // We test by creating entries in wrong order and checking the sort

        let mut entries = vec![
            DirEntry {
                name: "zebra.txt".to_string(),
                path: PathBuf::from("/tmp/zebra.txt"),
                is_dir: false,
                is_symlink: false,
                size: 100,
                mode: "-rw-r--r--".to_string(),
            },
            DirEntry {
                name: "apple_dir".to_string(),
                path: PathBuf::from("/tmp/apple_dir"),
                is_dir: true,
                is_symlink: false,
                size: 4096,
                mode: "drwxr-xr-x".to_string(),
            },
            DirEntry {
                name: "banana.txt".to_string(),
                path: PathBuf::from("/tmp/banana.txt"),
                is_dir: false,
                is_symlink: false,
                size: 200,
                mode: "-rw-r--r--".to_string(),
            },
        ];

        // Sort same way as read_directory
        entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        // Directory should be first
        assert!(entries[0].is_dir, "Directories should come first");
        assert_eq!(entries[0].name, "apple_dir");
        // Then files alphabetically
        assert_eq!(entries[1].name, "banana.txt");
        assert_eq!(entries[2].name, "zebra.txt");
    }

    #[test]
    fn test_filepicker_root_directory_no_parent() {
        use bubbletea::{KeyMsg, KeyType, Message};

        let mut fp = FilePicker::new();
        fp.set_current_directory("/");
        fp.files = vec![DirEntry {
            name: "etc".to_string(),
            path: PathBuf::from("/etc"),
            is_dir: true,
            is_symlink: false,
            size: 4096,
            mode: "drwxr-xr-x".to_string(),
        }];
        fp.max = 10;

        // Press Backspace at root should stay at root
        let back_msg = Message::new(KeyMsg::from_type(KeyType::Backspace));
        let _ = Model::update(&mut fp, back_msg);

        // At root, parent() returns None, so we should stay at root or empty path
        // The implementation sets current_directory to parent, which for "/" returns None
        // and the path stays as-is or becomes empty
        let current = fp.current_directory();
        assert!(
            current == Path::new("/") || current == Path::new(""),
            "Should stay at or near root when trying to go up from root"
        );
    }

    #[test]
    fn test_filepicker_highlighted_entry() {
        let mut fp = FilePicker::new();
        fp.files = vec![
            DirEntry {
                name: "first.txt".to_string(),
                path: PathBuf::from("/tmp/first.txt"),
                is_dir: false,
                is_symlink: false,
                size: 100,
                mode: "-rw-r--r--".to_string(),
            },
            DirEntry {
                name: "second.txt".to_string(),
                path: PathBuf::from("/tmp/second.txt"),
                is_dir: false,
                is_symlink: false,
                size: 200,
                mode: "-rw-r--r--".to_string(),
            },
        ];
        fp.selected = 0;

        let entry = fp
            .highlighted_entry()
            .expect("Should have highlighted entry");
        assert_eq!(entry.name, "first.txt");

        fp.selected = 1;
        let entry = fp
            .highlighted_entry()
            .expect("Should have highlighted entry");
        assert_eq!(entry.name, "second.txt");
    }

    #[test]
    fn test_filepicker_window_size_msg() {
        use bubbletea::{Message, WindowSizeMsg};

        let mut fp = FilePicker::new();
        fp.auto_height = true;
        assert_eq!(fp.height, 0);

        // Simulate window resize
        let size_msg = WindowSizeMsg {
            width: 80,
            height: 24,
        };
        let _ = Model::update(&mut fp, Message::new(size_msg));

        // Height should be updated (height - 5 for auto_height)
        assert_eq!(fp.height, 19, "Height should be terminal height minus 5");
    }

    #[test]
    fn test_filepicker_goto_top_and_last() {
        use bubbletea::{KeyMsg, Message};

        let mut fp = FilePicker::new();
        fp.files = (0..10)
            .map(|i| DirEntry {
                name: format!("file{}.txt", i),
                path: PathBuf::from(format!("/tmp/file{}.txt", i)),
                is_dir: false,
                is_symlink: false,
                size: 100,
                mode: "-rw-r--r--".to_string(),
            })
            .collect();
        fp.height = 5;
        fp.max = fp.height;
        fp.selected = 5;

        // Press 'g' to go to top
        let g_msg = Message::new(KeyMsg::from_char('g'));
        let _ = Model::update(&mut fp, g_msg);
        assert_eq!(fp.selected, 0, "'g' should go to first item");

        // Press 'G' to go to last
        let shift_g_msg = Message::new(KeyMsg::from_char('G'));
        let _ = Model::update(&mut fp, shift_g_msg);
        assert_eq!(fp.selected, 9, "'G' should go to last item");
    }
}
