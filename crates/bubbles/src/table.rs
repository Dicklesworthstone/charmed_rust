//! Table component for displaying tabular data.
//!
//! This module provides a table widget with keyboard navigation for TUI
//! applications.
//!
//! # Example
//!
//! ```rust
//! use bubbles::table::{Table, Column};
//!
//! let columns = vec![
//!     Column::new("ID", 10),
//!     Column::new("Name", 20),
//!     Column::new("Status", 15),
//! ];
//!
//! let rows = vec![
//!     vec!["1".into(), "Alice".into(), "Active".into()],
//!     vec!["2".into(), "Bob".into(), "Inactive".into()],
//! ];
//!
//! let table = Table::new()
//!     .columns(columns)
//!     .rows(rows);
//! ```

use crate::key::{matches, Binding};
use crate::viewport::Viewport;
use bubbletea::{Cmd, KeyMsg, Message};
use lipgloss::{Color, Style};

/// A single column definition for the table.
#[derive(Debug, Clone)]
pub struct Column {
    /// Column title displayed in the header.
    pub title: String,
    /// Width of the column in characters.
    pub width: usize,
}

impl Column {
    /// Creates a new column with the given title and width.
    #[must_use]
    pub fn new(title: impl Into<String>, width: usize) -> Self {
        Self {
            title: title.into(),
            width,
        }
    }
}

/// A row in the table (vector of cell values).
pub type Row = Vec<String>;

/// Key bindings for table navigation.
#[derive(Debug, Clone)]
pub struct KeyMap {
    /// Move up one line.
    pub line_up: Binding,
    /// Move down one line.
    pub line_down: Binding,
    /// Page up.
    pub page_up: Binding,
    /// Page down.
    pub page_down: Binding,
    /// Half page up.
    pub half_page_up: Binding,
    /// Half page down.
    pub half_page_down: Binding,
    /// Go to top.
    pub goto_top: Binding,
    /// Go to bottom.
    pub goto_bottom: Binding,
}

impl Default for KeyMap {
    fn default() -> Self {
        Self {
            line_up: Binding::new()
                .keys(&["up", "k"])
                .help("↑/k", "up"),
            line_down: Binding::new()
                .keys(&["down", "j"])
                .help("↓/j", "down"),
            page_up: Binding::new()
                .keys(&["b", "pgup"])
                .help("b/pgup", "page up"),
            page_down: Binding::new()
                .keys(&["f", "pgdown", " "])
                .help("f/pgdn", "page down"),
            half_page_up: Binding::new()
                .keys(&["u", "ctrl+u"])
                .help("u", "½ page up"),
            half_page_down: Binding::new()
                .keys(&["d", "ctrl+d"])
                .help("d", "½ page down"),
            goto_top: Binding::new()
                .keys(&["home", "g"])
                .help("g/home", "go to start"),
            goto_bottom: Binding::new()
                .keys(&["end", "G"])
                .help("G/end", "go to end"),
        }
    }
}

/// Styles for the table.
#[derive(Debug, Clone)]
pub struct Styles {
    /// Style for the header row.
    pub header: Style,
    /// Style for normal cells.
    pub cell: Style,
    /// Style for the selected row.
    pub selected: Style,
}

impl Default for Styles {
    fn default() -> Self {
        Self {
            header: Style::new().bold().padding_left(1).padding_right(1),
            cell: Style::new().padding_left(1).padding_right(1),
            selected: Style::new().bold().foreground_color(Color::from("212")),
        }
    }
}

/// Table model for displaying tabular data with keyboard navigation.
#[derive(Debug, Clone)]
pub struct Table {
    /// Key bindings for navigation.
    pub key_map: KeyMap,
    /// Styles for rendering.
    pub styles: Styles,
    /// Column definitions.
    columns: Vec<Column>,
    /// Table rows (data).
    rows: Vec<Row>,
    /// Currently selected row index.
    cursor: usize,
    /// Whether the table is focused.
    focus: bool,
    /// Internal viewport for scrolling.
    viewport: Viewport,
    /// Start index for rendered rows.
    start: usize,
    /// End index for rendered rows.
    end: usize,
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl Table {
    /// Creates a new empty table.
    #[must_use]
    pub fn new() -> Self {
        Self {
            key_map: KeyMap::default(),
            styles: Styles::default(),
            columns: Vec::new(),
            rows: Vec::new(),
            cursor: 0,
            focus: false,
            viewport: Viewport::new(0, 20),
            start: 0,
            end: 0,
        }
    }

    /// Sets the columns (builder pattern).
    #[must_use]
    pub fn columns(mut self, columns: Vec<Column>) -> Self {
        self.columns = columns;
        self.update_viewport();
        self
    }

    /// Sets the rows (builder pattern).
    #[must_use]
    pub fn rows(mut self, rows: Vec<Row>) -> Self {
        self.rows = rows;
        self.update_viewport();
        self
    }

    /// Sets the height (builder pattern).
    #[must_use]
    pub fn height(mut self, h: usize) -> Self {
        let header_height = 1; // Single header row
        self.viewport.height = h.saturating_sub(header_height);
        self.update_viewport();
        self
    }

    /// Sets the width (builder pattern).
    #[must_use]
    pub fn width(mut self, w: usize) -> Self {
        self.viewport.width = w;
        self.update_viewport();
        self
    }

    /// Sets the focused state (builder pattern).
    #[must_use]
    pub fn focused(mut self, f: bool) -> Self {
        self.focus = f;
        self.update_viewport();
        self
    }

    /// Sets the styles (builder pattern).
    #[must_use]
    pub fn with_styles(mut self, styles: Styles) -> Self {
        self.styles = styles;
        self.update_viewport();
        self
    }

    /// Sets the key map (builder pattern).
    #[must_use]
    pub fn with_key_map(mut self, key_map: KeyMap) -> Self {
        self.key_map = key_map;
        self
    }

    /// Returns whether the table is focused.
    #[must_use]
    pub fn is_focused(&self) -> bool {
        self.focus
    }

    /// Focuses the table.
    pub fn focus(&mut self) {
        self.focus = true;
        self.update_viewport();
    }

    /// Blurs (unfocuses) the table.
    pub fn blur(&mut self) {
        self.focus = false;
        self.update_viewport();
    }

    /// Returns the columns.
    #[must_use]
    pub fn get_columns(&self) -> &[Column] {
        &self.columns
    }

    /// Returns the rows.
    #[must_use]
    pub fn get_rows(&self) -> &[Row] {
        &self.rows
    }

    /// Sets the columns.
    pub fn set_columns(&mut self, columns: Vec<Column>) {
        self.columns = columns;
        self.update_viewport();
    }

    /// Sets the rows.
    pub fn set_rows(&mut self, rows: Vec<Row>) {
        self.rows = rows;
        if self.cursor > self.rows.len().saturating_sub(1) {
            self.cursor = self.rows.len().saturating_sub(1);
        }
        self.update_viewport();
    }

    /// Sets the width.
    pub fn set_width(&mut self, w: usize) {
        self.viewport.width = w;
        self.update_viewport();
    }

    /// Sets the height.
    pub fn set_height(&mut self, h: usize) {
        let header_height = 1;
        self.viewport.height = h.saturating_sub(header_height);
        self.update_viewport();
    }

    /// Returns the viewport height.
    #[must_use]
    pub fn get_height(&self) -> usize {
        self.viewport.height
    }

    /// Returns the viewport width.
    #[must_use]
    pub fn get_width(&self) -> usize {
        self.viewport.width
    }

    /// Returns the currently selected row, if any.
    #[must_use]
    pub fn selected_row(&self) -> Option<&Row> {
        self.rows.get(self.cursor)
    }

    /// Returns the cursor position (selected row index).
    #[must_use]
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Sets the cursor position.
    pub fn set_cursor(&mut self, n: usize) {
        self.cursor = n.min(self.rows.len().saturating_sub(1));
        self.update_viewport();
    }

    /// Moves the selection up by n rows.
    pub fn move_up(&mut self, n: usize) {
        if self.rows.is_empty() {
            return;
        }
        self.cursor = self.cursor.saturating_sub(n);
        self.update_viewport();
    }

    /// Moves the selection down by n rows.
    pub fn move_down(&mut self, n: usize) {
        if self.rows.is_empty() {
            return;
        }
        self.cursor = (self.cursor + n).min(self.rows.len().saturating_sub(1));
        self.update_viewport();
    }

    /// Moves to the first row.
    pub fn goto_top(&mut self) {
        self.cursor = 0;
        self.update_viewport();
    }

    /// Moves to the last row.
    pub fn goto_bottom(&mut self) {
        if !self.rows.is_empty() {
            self.cursor = self.rows.len() - 1;
        }
        self.update_viewport();
    }

    /// Parses rows from a string value with the given separator.
    pub fn from_values(&mut self, value: &str, separator: &str) {
        let rows: Vec<Row> = value
            .lines()
            .map(|line| line.split(separator).map(String::from).collect())
            .collect();
        self.set_rows(rows);
    }

    /// Updates the viewport to reflect current state.
    fn update_viewport(&mut self) {
        if self.rows.is_empty() {
            self.start = 0;
            self.end = 0;
            self.viewport.set_content("");
            return;
        }

        // Calculate visible range
        let height = self.viewport.height;
        self.start = if self.cursor >= height {
            self.cursor.saturating_sub(height)
        } else {
            0
        };
        self.end = (self.cursor + height).min(self.rows.len());

        // Render rows
        let rendered: Vec<String> = (self.start..self.end)
            .map(|i| self.render_row(i))
            .collect();

        self.viewport.set_content(&rendered.join("\n"));
    }

    /// Renders the header row.
    fn headers_view(&self) -> String {
        let cells: Vec<String> = self
            .columns
            .iter()
            .filter(|col| col.width > 0)
            .map(|col| {
                let truncated = truncate_string(&col.title, col.width);
                let padded = format!("{:width$}", truncated, width = col.width);
                self.styles.header.render(&padded)
            })
            .collect();

        cells.join("")
    }

    /// Renders a single row.
    fn render_row(&self, row_idx: usize) -> String {
        let row = &self.rows[row_idx];

        let cells: Vec<String> = self
            .columns
            .iter()
            .enumerate()
            .filter(|(_, col)| col.width > 0)
            .map(|(i, col)| {
                let value = row.get(i).map(String::as_str).unwrap_or("");
                let truncated = truncate_string(value, col.width);
                let padded = format!("{:width$}", truncated, width = col.width);
                self.styles.cell.render(&padded)
            })
            .collect();

        let row_str = cells.join("");

        if row_idx == self.cursor {
            self.styles.selected.render(&row_str)
        } else {
            row_str
        }
    }

    /// Updates the table based on messages.
    pub fn update(&mut self, msg: Message) -> Option<Cmd> {
        if !self.focus {
            return None;
        }

        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            let key_str = key.to_string();

            if matches(&key_str, &[&self.key_map.line_up]) {
                self.move_up(1);
            } else if matches(&key_str, &[&self.key_map.line_down]) {
                self.move_down(1);
            } else if matches(&key_str, &[&self.key_map.page_up]) {
                self.move_up(self.viewport.height);
            } else if matches(&key_str, &[&self.key_map.page_down]) {
                self.move_down(self.viewport.height);
            } else if matches(&key_str, &[&self.key_map.half_page_up]) {
                self.move_up(self.viewport.height / 2);
            } else if matches(&key_str, &[&self.key_map.half_page_down]) {
                self.move_down(self.viewport.height / 2);
            } else if matches(&key_str, &[&self.key_map.goto_top]) {
                self.goto_top();
            } else if matches(&key_str, &[&self.key_map.goto_bottom]) {
                self.goto_bottom();
            }
        }

        None
    }

    /// Renders the table.
    #[must_use]
    pub fn view(&self) -> String {
        format!("{}\n{}", self.headers_view(), self.viewport.view())
    }
}

/// Truncates a string to the given width, adding ellipsis if needed.
fn truncate_string(s: &str, width: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= width {
        s.to_string()
    } else if width > 0 {
        let truncated: String = chars[..width.saturating_sub(1)].iter().collect();
        format!("{}…", truncated)
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_new() {
        let col = Column::new("Name", 20);
        assert_eq!(col.title, "Name");
        assert_eq!(col.width, 20);
    }

    #[test]
    fn test_table_new() {
        let table = Table::new();
        assert!(table.get_columns().is_empty());
        assert!(table.get_rows().is_empty());
        assert!(!table.is_focused());
    }

    #[test]
    fn test_table_builder() {
        let columns = vec![Column::new("ID", 10), Column::new("Name", 20)];
        let rows = vec![
            vec!["1".into(), "Alice".into()],
            vec!["2".into(), "Bob".into()],
        ];

        let table = Table::new()
            .columns(columns)
            .rows(rows)
            .height(10)
            .focused(true);

        assert_eq!(table.get_columns().len(), 2);
        assert_eq!(table.get_rows().len(), 2);
        assert!(table.is_focused());
    }

    #[test]
    fn test_table_navigation() {
        let rows = vec![
            vec!["1".into()],
            vec!["2".into()],
            vec!["3".into()],
            vec!["4".into()],
            vec!["5".into()],
        ];

        let mut table = Table::new().rows(rows).height(10);

        assert_eq!(table.cursor(), 0);

        table.move_down(1);
        assert_eq!(table.cursor(), 1);

        table.move_down(2);
        assert_eq!(table.cursor(), 3);

        table.move_up(1);
        assert_eq!(table.cursor(), 2);

        table.goto_bottom();
        assert_eq!(table.cursor(), 4);

        table.goto_top();
        assert_eq!(table.cursor(), 0);
    }

    #[test]
    fn test_table_selected_row() {
        let rows = vec![
            vec!["1".into(), "Alice".into()],
            vec!["2".into(), "Bob".into()],
        ];

        let mut table = Table::new().rows(rows);

        assert_eq!(table.selected_row(), Some(&vec!["1".into(), "Alice".into()]));

        table.move_down(1);
        assert_eq!(table.selected_row(), Some(&vec!["2".into(), "Bob".into()]));
    }

    #[test]
    fn test_table_focus_blur() {
        let mut table = Table::new();
        assert!(!table.is_focused());

        table.focus();
        assert!(table.is_focused());

        table.blur();
        assert!(!table.is_focused());
    }

    #[test]
    fn test_table_set_cursor() {
        let rows = vec![vec!["1".into()], vec!["2".into()], vec!["3".into()]];

        let mut table = Table::new().rows(rows);

        table.set_cursor(2);
        assert_eq!(table.cursor(), 2);

        // Should clamp to last row
        table.set_cursor(100);
        assert_eq!(table.cursor(), 2);
    }

    #[test]
    fn test_table_from_values() {
        let mut table = Table::new();
        table.from_values("a,b,c\n1,2,3\nx,y,z", ",");

        assert_eq!(table.get_rows().len(), 3);
        assert_eq!(table.get_rows()[0], vec!["a", "b", "c"]);
        assert_eq!(table.get_rows()[1], vec!["1", "2", "3"]);
    }

    #[test]
    fn test_table_view() {
        let columns = vec![Column::new("ID", 5), Column::new("Name", 10)];
        let rows = vec![
            vec!["1".into(), "Alice".into()],
            vec!["2".into(), "Bob".into()],
        ];

        let table = Table::new().columns(columns).rows(rows).height(5);
        let view = table.view();

        assert!(view.contains("ID"));
        assert!(view.contains("Name"));
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("Hello", 10), "Hello");
        assert_eq!(truncate_string("Hello World", 5), "Hell…");
        assert_eq!(truncate_string("Hi", 2), "Hi");
        assert_eq!(truncate_string("", 5), "");
    }

    #[test]
    fn test_table_empty() {
        let table = Table::new();
        assert!(table.selected_row().is_none());
        assert_eq!(table.cursor(), 0);
    }

    #[test]
    fn test_keymap_default() {
        let km = KeyMap::default();
        assert!(!km.line_up.get_keys().is_empty());
        assert!(!km.goto_bottom.get_keys().is_empty());
    }
}
