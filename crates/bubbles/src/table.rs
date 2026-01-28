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

use crate::key::{Binding, matches};
use crate::viewport::Viewport;
use bubbletea::{Cmd, KeyMsg, Message, Model, MouseAction, MouseButton, MouseMsg};
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
            line_up: Binding::new().keys(&["up", "k"]).help("↑/k", "up"),
            line_down: Binding::new().keys(&["down", "j"]).help("↓/j", "down"),
            page_up: Binding::new()
                .keys(&["b", "pgup"])
                .help("b/pgup", "page up"),
            page_down: Binding::new()
                .keys(&["f", "pgdown", " "])
                .help("f/pgdn", "page down"),
            half_page_up: Binding::new().keys(&["u", "ctrl+u"]).help("u", "½ page up"),
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
    /// Whether mouse wheel scrolling is enabled.
    pub mouse_wheel_enabled: bool,
    /// Number of rows to scroll per mouse wheel tick.
    pub mouse_wheel_delta: usize,
    /// Whether mouse click selection is enabled.
    pub mouse_click_enabled: bool,
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
            mouse_wheel_enabled: true,
            mouse_wheel_delta: 3,
            mouse_click_enabled: true,
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

    /// Enables or disables mouse wheel scrolling (builder pattern).
    #[must_use]
    pub fn mouse_wheel(mut self, enabled: bool) -> Self {
        self.mouse_wheel_enabled = enabled;
        self
    }

    /// Sets the number of rows to scroll per mouse wheel tick (builder pattern).
    #[must_use]
    pub fn mouse_wheel_delta(mut self, delta: usize) -> Self {
        self.mouse_wheel_delta = delta;
        self
    }

    /// Enables or disables mouse click row selection (builder pattern).
    #[must_use]
    pub fn mouse_click(mut self, enabled: bool) -> Self {
        self.mouse_click_enabled = enabled;
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

        let height = self.viewport.height;
        if height == 0 {
            self.start = 0;
            self.end = 0;
            self.viewport.set_content("");
            return;
        }

        // Keep cursor visible - adjust start window if cursor moves out of view
        if self.cursor < self.start {
            // Cursor moved above visible window
            self.start = self.cursor;
        } else if self.cursor >= self.start + height {
            // Cursor moved below visible window
            self.start = self.cursor - height + 1;
        }

        // Calculate end to show exactly height rows (or fewer if not enough data)
        self.end = (self.start + height).min(self.rows.len());

        // If we're near the end and have room, fill the viewport
        if self.end - self.start < height && self.start > 0 {
            self.start = self.end.saturating_sub(height);
        }

        // Render only the visible rows
        let rendered: Vec<String> = (self.start..self.end).map(|i| self.render_row(i)).collect();

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
                let padded = pad_string(&truncated, col.width);
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
                let padded = pad_string(&truncated, col.width);
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

    /// Updates the table based on key/mouse input.
    pub fn update(&mut self, msg: &Message) {
        if !self.focus {
            return;
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

        // Handle mouse events
        if let Some(mouse) = msg.downcast_ref::<MouseMsg>() {
            // Only respond to press events
            if mouse.action != MouseAction::Press {
                return;
            }

            match mouse.button {
                // Wheel scrolling
                MouseButton::WheelUp if self.mouse_wheel_enabled => {
                    self.move_up(self.mouse_wheel_delta);
                }
                MouseButton::WheelDown if self.mouse_wheel_enabled => {
                    self.move_down(self.mouse_wheel_delta);
                }
                // Click to select row
                MouseButton::Left if self.mouse_click_enabled => {
                    // y=0 is the header row, data rows start at y=1
                    // Convert click y to row index, accounting for viewport scroll
                    let header_height = 1usize;
                    let click_y = mouse.y as usize;

                    if click_y >= header_height {
                        // Calculate which visible row was clicked
                        let visible_row = click_y - header_height;
                        // Convert to actual row index using viewport offset
                        let row_index = self.start + visible_row;

                        // Only select if within bounds
                        if row_index < self.rows.len() {
                            self.cursor = row_index;
                            self.update_viewport();
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Renders the table.
    #[must_use]
    pub fn view(&self) -> String {
        format!("{}\n{}", self.headers_view(), self.viewport.view())
    }
}

/// Pads a string to the given width with spaces.
fn pad_string(s: &str, width: usize) -> String {
    use unicode_width::UnicodeWidthStr;
    let current_width = UnicodeWidthStr::width(s);
    if current_width >= width {
        s.to_string()
    } else {
        let padding = width - current_width;
        format!("{}{}", s, " ".repeat(padding))
    }
}

/// Truncates a string to the given width, adding ellipsis if needed.
fn truncate_string(s: &str, width: usize) -> String {
    use unicode_width::UnicodeWidthStr;

    if UnicodeWidthStr::width(s) <= width {
        return s.to_string();
    }

    if width == 0 {
        return String::new();
    }

    // We need to truncate to width - 1 (for ellipsis)
    let target_width = width.saturating_sub(1);
    let mut current_width = 0;
    let mut result = String::new();

    for c in s.chars() {
        let w = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        if current_width + w > target_width {
            break;
        }
        result.push(c);
        current_width += w;
    }

    format!("{}…", result)
}

impl Model for Table {
    /// Initialize the table.
    ///
    /// Tables don't require initialization commands.
    fn init(&self) -> Option<Cmd> {
        None
    }

    /// Update the table state based on incoming messages.
    fn update(&mut self, msg: Message) -> Option<Cmd> {
        self.update(&msg);
        None
    }

    /// Render the table.
    fn view(&self) -> String {
        Table::view(self)
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

        assert_eq!(
            table.selected_row(),
            Some(&vec!["1".into(), "Alice".into()])
        );

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

    // Model trait implementation tests
    #[test]
    fn test_model_init() {
        let table = Table::new();
        // Tables don't require init commands
        let cmd = Model::init(&table);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_model_view() {
        let columns = vec![Column::new("ID", 5)];
        let rows = vec![vec!["1".into()]];
        let table = Table::new().columns(columns).rows(rows);
        // Model::view should return same result as Table::view
        let model_view = Model::view(&table);
        let table_view = Table::view(&table);
        assert_eq!(model_view, table_view);
    }

    #[test]
    fn test_model_update_handles_navigation() {
        use bubbletea::{KeyMsg, KeyType, Message};

        let columns = vec![Column::new("ID", 5), Column::new("Name", 20)];
        let rows = vec![
            vec!["1".into(), "First".into()],
            vec!["2".into(), "Second".into()],
            vec!["3".into(), "Third".into()],
        ];
        let mut table = Table::new().columns(columns).rows(rows).focused(true);
        assert_eq!(table.cursor(), 0);

        // Press down arrow
        let down_msg = Message::new(KeyMsg::from_type(KeyType::Down));
        let _ = Model::update(&mut table, down_msg);

        assert_eq!(table.cursor(), 1, "Table should navigate down on Down key");
    }

    #[test]
    fn test_model_update_unfocused_ignores_input() {
        use bubbletea::{KeyMsg, KeyType, Message};

        let columns = vec![Column::new("ID", 5)];
        let rows = vec![vec!["1".into()], vec!["2".into()]];
        let mut table = Table::new().columns(columns).rows(rows);
        // Table is not focused by default
        assert!(!table.focus);
        assert_eq!(table.cursor(), 0);

        // Press down arrow
        let down_msg = Message::new(KeyMsg::from_type(KeyType::Down));
        let _ = Model::update(&mut table, down_msg);

        assert_eq!(table.cursor(), 0, "Unfocused table should ignore key input");
    }

    #[test]
    fn test_model_update_goto_bottom() {
        use bubbletea::{KeyMsg, KeyType, Message};

        let columns = vec![Column::new("ID", 5)];
        let rows = vec![vec!["1".into()], vec!["2".into()], vec!["3".into()]];
        let mut table = Table::new().columns(columns).rows(rows).focused(true);
        assert_eq!(table.cursor(), 0);

        // Press End to go to bottom
        let end_msg = Message::new(KeyMsg::from_type(KeyType::End));
        let _ = Model::update(&mut table, end_msg);

        assert_eq!(table.cursor(), 2, "Table should go to bottom on End key");
    }

    #[test]
    fn test_table_satisfies_model_bounds() {
        fn requires_model<T: Model + Send + 'static>() {}
        requires_model::<Table>();
    }

    #[test]
    fn test_model_update_page_down() {
        use bubbletea::{KeyMsg, KeyType, Message};

        let columns = vec![Column::new("ID", 5)];
        // Create 20 rows to test page navigation
        let rows: Vec<Row> = (1..=20).map(|i| vec![i.to_string()]).collect();
        let mut table = Table::new()
            .columns(columns)
            .rows(rows)
            .focused(true)
            .height(5); // 5 visible rows

        assert_eq!(table.cursor(), 0);

        // Press PageDown
        let msg = Message::new(KeyMsg::from_type(KeyType::PgDown));
        let _ = Model::update(&mut table, msg);

        // Should move down by height (5 rows)
        assert!(
            table.cursor() > 0,
            "Table should navigate down on PageDown key"
        );
    }

    #[test]
    fn test_model_update_page_up() {
        use bubbletea::{KeyMsg, KeyType, Message};

        let columns = vec![Column::new("ID", 5)];
        let rows: Vec<Row> = (1..=20).map(|i| vec![i.to_string()]).collect();
        let mut table = Table::new()
            .columns(columns)
            .rows(rows)
            .focused(true)
            .height(5);

        // Start at row 10
        table.set_cursor(10);
        assert_eq!(table.cursor(), 10);

        // Press PageUp
        let msg = Message::new(KeyMsg::from_type(KeyType::PgUp));
        let _ = Model::update(&mut table, msg);

        // Should move up
        assert!(
            table.cursor() < 10,
            "Table should navigate up on PageUp key"
        );
    }

    #[test]
    fn test_model_update_goto_top() {
        use bubbletea::{KeyMsg, KeyType, Message};

        let columns = vec![Column::new("ID", 5)];
        let rows = vec![vec!["1".into()], vec!["2".into()], vec!["3".into()]];
        let mut table = Table::new().columns(columns).rows(rows).focused(true);

        // Start at last row
        table.set_cursor(2);
        assert_eq!(table.cursor(), 2);

        // Press Home to go to top
        let msg = Message::new(KeyMsg::from_type(KeyType::Home));
        let _ = Model::update(&mut table, msg);

        assert_eq!(table.cursor(), 0, "Table should go to top on Home key");
    }

    #[test]
    fn test_table_set_rows_replaces_data() {
        let columns = vec![Column::new("Name", 10)];
        let initial_rows = vec![vec!["Alice".into()], vec!["Bob".into()]];
        let mut table = Table::new().columns(columns).rows(initial_rows);

        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[0][0], "Alice");

        // Replace rows
        let new_rows = vec![
            vec!["Charlie".into()],
            vec!["Diana".into()],
            vec!["Eve".into()],
        ];
        table.set_rows(new_rows);

        assert_eq!(table.rows.len(), 3);
        assert_eq!(table.rows[0][0], "Charlie");
        assert_eq!(table.rows[1][0], "Diana");
        assert_eq!(table.rows[2][0], "Eve");
    }

    #[test]
    fn test_table_set_columns_updates_headers() {
        let initial_cols = vec![Column::new("A", 5), Column::new("B", 5)];
        let rows = vec![vec!["1".into(), "2".into()]];
        let mut table = Table::new().columns(initial_cols).rows(rows);

        assert_eq!(table.columns.len(), 2);
        assert_eq!(table.columns[0].title, "A");

        // Update columns
        let new_cols = vec![
            Column::new("X", 10),
            Column::new("Y", 10),
            Column::new("Z", 10),
        ];
        table.set_columns(new_cols);

        assert_eq!(table.columns.len(), 3);
        assert_eq!(table.columns[0].title, "X");
        assert_eq!(table.columns[1].title, "Y");
        assert_eq!(table.columns[2].title, "Z");
    }

    #[test]
    fn test_table_single_row() {
        use bubbletea::{KeyMsg, KeyType, Message};

        let columns = vec![Column::new("Item", 10)];
        let rows = vec![vec!["Only One".into()]];
        let mut table = Table::new().columns(columns).rows(rows).focused(true);

        assert_eq!(table.cursor(), 0);
        assert_eq!(table.rows.len(), 1);

        // Try to navigate down - should stay at 0
        let down_msg = Message::new(KeyMsg::from_type(KeyType::Down));
        let _ = Model::update(&mut table, down_msg);
        assert_eq!(table.cursor(), 0, "Single row table should not move down");

        // Try to navigate up - should stay at 0
        let up_msg = Message::new(KeyMsg::from_type(KeyType::Up));
        let _ = Model::update(&mut table, up_msg);
        assert_eq!(table.cursor(), 0, "Single row table should not move up");

        // Selected row should work
        assert!(table.selected_row().is_some());
        assert_eq!(table.selected_row().unwrap()[0], "Only One");
    }

    #[test]
    fn test_table_single_column() {
        let columns = vec![Column::new("Solo", 15)];
        let rows = vec![
            vec!["Row 1".into()],
            vec!["Row 2".into()],
            vec!["Row 3".into()],
        ];
        let table = Table::new().columns(columns).rows(rows);

        assert_eq!(table.columns.len(), 1);
        assert_eq!(table.columns[0].title, "Solo");
        assert_eq!(table.columns[0].width, 15);

        // View should still render correctly
        let view = table.view();
        assert!(!view.is_empty());
        assert!(
            view.contains("Solo") || view.contains("Row"),
            "Single column table should render"
        );
    }

    // ========================================================================
    // Additional Model trait tests for bead charmed_rust-zg4
    // ========================================================================

    #[test]
    fn test_table_empty_navigation() {
        use bubbletea::{KeyMsg, KeyType, Message};

        let mut table = Table::new().focused(true);
        assert!(table.rows.is_empty());
        assert_eq!(table.cursor(), 0);

        // Navigation on empty table should not panic
        let down_msg = Message::new(KeyMsg::from_type(KeyType::Down));
        let _ = Model::update(&mut table, down_msg);
        assert_eq!(table.cursor(), 0, "Empty table cursor should stay at 0");

        let up_msg = Message::new(KeyMsg::from_type(KeyType::Up));
        let _ = Model::update(&mut table, up_msg);
        assert_eq!(table.cursor(), 0, "Empty table cursor should stay at 0");

        let end_msg = Message::new(KeyMsg::from_type(KeyType::End));
        let _ = Model::update(&mut table, end_msg);
        assert_eq!(
            table.cursor(),
            0,
            "Empty table goto_bottom should stay at 0"
        );

        let home_msg = Message::new(KeyMsg::from_type(KeyType::Home));
        let _ = Model::update(&mut table, home_msg);
        assert_eq!(table.cursor(), 0, "Empty table goto_top should stay at 0");
    }

    #[test]
    fn test_table_view_empty() {
        let table = Table::new();
        let view = table.view();
        // Empty table should still produce a view (may be empty or minimal)
        // Just verify it doesn't panic
        let _ = view;
    }

    #[test]
    fn test_table_view_renders_column_widths() {
        let columns = vec![Column::new("Short", 5), Column::new("LongerColumn", 15)];
        let rows = vec![vec!["A".into(), "B".into()]];
        let table = Table::new().columns(columns).rows(rows);
        let view = table.view();

        // View should be non-empty and contain column headers
        assert!(!view.is_empty());
        // The view includes headers that should be visible
        assert!(view.contains("Short") || view.contains("Longer"));
    }

    #[test]
    fn test_model_update_navigate_up() {
        use bubbletea::{KeyMsg, KeyType, Message};

        let columns = vec![Column::new("ID", 5)];
        let rows = vec![vec!["1".into()], vec!["2".into()], vec!["3".into()]];
        let mut table = Table::new().columns(columns).rows(rows).focused(true);
        table.set_cursor(2);
        assert_eq!(table.cursor(), 2);

        // Press Up arrow
        let up_msg = Message::new(KeyMsg::from_type(KeyType::Up));
        let _ = Model::update(&mut table, up_msg);

        assert_eq!(table.cursor(), 1, "Table should navigate up on Up key");
    }

    #[test]
    fn test_table_view_with_long_content() {
        let columns = vec![Column::new("Name", 5)];
        let rows = vec![vec!["VeryLongNameThatExceedsColumnWidth".into()]];
        let table = Table::new().columns(columns).rows(rows);
        let view = table.view();

        // Content should be truncated in view (not crash)
        assert!(!view.is_empty());
    }

    #[test]
    fn test_table_cursor_boundary_top() {
        use bubbletea::{KeyMsg, KeyType, Message};

        let columns = vec![Column::new("ID", 5)];
        let rows = vec![vec!["1".into()], vec!["2".into()]];
        let mut table = Table::new().columns(columns).rows(rows).focused(true);
        assert_eq!(table.cursor(), 0);

        // Try to move up from top - should stay at 0
        let up_msg = Message::new(KeyMsg::from_type(KeyType::Up));
        let _ = Model::update(&mut table, up_msg);
        assert_eq!(table.cursor(), 0, "Cursor should not go below 0");
    }

    #[test]
    fn test_table_cursor_boundary_bottom() {
        use bubbletea::{KeyMsg, KeyType, Message};

        let columns = vec![Column::new("ID", 5)];
        let rows = vec![vec!["1".into()], vec!["2".into()]];
        let mut table = Table::new().columns(columns).rows(rows).focused(true);
        table.set_cursor(1);
        assert_eq!(table.cursor(), 1);

        // Try to move down from bottom - should stay at 1
        let down_msg = Message::new(KeyMsg::from_type(KeyType::Down));
        let _ = Model::update(&mut table, down_msg);
        assert_eq!(table.cursor(), 1, "Cursor should not exceed row count");
    }

    #[test]
    fn test_table_update_with_j_k_keys() {
        use bubbletea::{KeyMsg, Message};

        let columns = vec![Column::new("ID", 5)];
        let rows = vec![vec!["1".into()], vec!["2".into()], vec!["3".into()]];
        let mut table = Table::new().columns(columns).rows(rows).focused(true);
        assert_eq!(table.cursor(), 0);

        // Test 'j' key for down
        let j_msg = Message::new(KeyMsg::from_char('j'));
        let _ = Model::update(&mut table, j_msg);
        assert_eq!(table.cursor(), 1, "'j' should move cursor down");

        // Test 'k' key for up
        let k_msg = Message::new(KeyMsg::from_char('k'));
        let _ = Model::update(&mut table, k_msg);
        assert_eq!(table.cursor(), 0, "'k' should move cursor up");
    }

    #[test]
    fn test_table_update_with_g_and_shift_g_keys() {
        use bubbletea::{KeyMsg, Message};

        let columns = vec![Column::new("ID", 5)];
        let rows = vec![vec!["1".into()], vec!["2".into()], vec!["3".into()]];
        let mut table = Table::new().columns(columns).rows(rows).focused(true);
        assert_eq!(table.cursor(), 0);

        // Test 'G' key for goto bottom
        let g_upper_msg = Message::new(KeyMsg::from_char('G'));
        let _ = Model::update(&mut table, g_upper_msg);
        assert_eq!(table.cursor(), 2, "'G' should go to bottom");

        // Test 'g' key for goto top
        let g_msg = Message::new(KeyMsg::from_char('g'));
        let _ = Model::update(&mut table, g_msg);
        assert_eq!(table.cursor(), 0, "'g' should go to top");
    }

    #[test]
    fn test_table_height_affects_pagination() {
        use bubbletea::{KeyMsg, KeyType, Message};

        let columns = vec![Column::new("ID", 5)];
        // 20 rows
        let rows: Vec<Row> = (1..=20).map(|i| vec![i.to_string()]).collect();
        let mut table = Table::new()
            .columns(columns)
            .rows(rows)
            .focused(true)
            .height(3); // Small viewport

        assert_eq!(table.cursor(), 0);

        // PageDown should move by height
        let pgdown_msg = Message::new(KeyMsg::from_type(KeyType::PgDown));
        let _ = Model::update(&mut table, pgdown_msg);

        // Cursor should move down (exact amount depends on implementation)
        assert!(table.cursor() > 0, "PageDown should move cursor down");
    }

    #[test]
    fn test_table_selected_row_after_navigation() {
        use bubbletea::{KeyMsg, KeyType, Message};

        let columns = vec![Column::new("Name", 10)];
        let rows = vec![
            vec!["Alice".into()],
            vec!["Bob".into()],
            vec!["Carol".into()],
        ];
        let mut table = Table::new().columns(columns).rows(rows).focused(true);

        assert_eq!(table.selected_row().unwrap()[0], "Alice");

        let down_msg = Message::new(KeyMsg::from_type(KeyType::Down));
        let _ = Model::update(&mut table, down_msg);
        assert_eq!(table.selected_row().unwrap()[0], "Bob");

        let _ = Model::update(&mut table, Message::new(KeyMsg::from_type(KeyType::Down)));
        assert_eq!(table.selected_row().unwrap()[0], "Carol");
    }

    // -------------------------------------------------------------------------
    // Mouse support tests (bd-3ps4)
    // -------------------------------------------------------------------------

    mod mouse_tests {
        use super::*;
        use bubbletea::Message;

        fn wheel_up_msg() -> Message {
            Message::new(MouseMsg {
                x: 0,
                y: 0,
                shift: false,
                alt: false,
                ctrl: false,
                action: MouseAction::Press,
                button: MouseButton::WheelUp,
            })
        }

        fn wheel_down_msg() -> Message {
            Message::new(MouseMsg {
                x: 0,
                y: 0,
                shift: false,
                alt: false,
                ctrl: false,
                action: MouseAction::Press,
                button: MouseButton::WheelDown,
            })
        }

        fn click_msg(x: u16, y: u16) -> Message {
            Message::new(MouseMsg {
                x,
                y,
                shift: false,
                alt: false,
                ctrl: false,
                action: MouseAction::Press,
                button: MouseButton::Left,
            })
        }

        #[test]
        fn test_table_mouse_wheel_scroll_down() {
            let rows = vec![
                vec!["1".into()],
                vec!["2".into()],
                vec!["3".into()],
                vec!["4".into()],
                vec!["5".into()],
            ];
            let mut table = Table::new().rows(rows).focused(true);
            assert_eq!(table.cursor(), 0);

            table.update(&wheel_down_msg());
            // Default delta is 3, so cursor should be at 3
            assert_eq!(table.cursor(), 3);
        }

        #[test]
        fn test_table_mouse_wheel_scroll_up() {
            let rows = vec![
                vec!["1".into()],
                vec!["2".into()],
                vec!["3".into()],
                vec!["4".into()],
                vec!["5".into()],
            ];
            let mut table = Table::new().rows(rows).focused(true);

            // Move to bottom first
            table.goto_bottom();
            assert_eq!(table.cursor(), 4);

            table.update(&wheel_up_msg());
            // Default delta is 3, so cursor should be at 1
            assert_eq!(table.cursor(), 1);
        }

        #[test]
        fn test_table_mouse_click_select_row() {
            let rows = vec![vec!["1".into()], vec!["2".into()], vec!["3".into()]];
            let mut table = Table::new().rows(rows).focused(true);
            assert_eq!(table.cursor(), 0);

            // Click on row 2 (y=0 is header, y=1 is row 0, y=2 is row 1)
            table.update(&click_msg(5, 2));
            assert_eq!(table.cursor(), 1);

            // Click on row 3
            table.update(&click_msg(5, 3));
            assert_eq!(table.cursor(), 2);
        }

        #[test]
        fn test_table_mouse_click_header_does_nothing() {
            let rows = vec![vec!["1".into()], vec!["2".into()]];
            let mut table = Table::new().rows(rows).focused(true);
            assert_eq!(table.cursor(), 0);

            // Click on header (y=0)
            table.update(&click_msg(5, 0));
            // Cursor should not change
            assert_eq!(table.cursor(), 0);
        }

        #[test]
        fn test_table_mouse_click_out_of_bounds() {
            let rows = vec![vec!["1".into()], vec!["2".into()]];
            let mut table = Table::new().rows(rows).focused(true);
            assert_eq!(table.cursor(), 0);

            // Click way below the table
            table.update(&click_msg(5, 100));
            // Cursor should not change (out of bounds)
            assert_eq!(table.cursor(), 0);
        }

        #[test]
        fn test_table_mouse_disabled() {
            let rows = vec![vec!["1".into()], vec!["2".into()], vec!["3".into()]];
            let mut table = Table::new()
                .rows(rows)
                .focused(true)
                .mouse_wheel(false)
                .mouse_click(false);

            // Wheel should be ignored
            table.update(&wheel_down_msg());
            assert_eq!(table.cursor(), 0);

            // Click should be ignored
            table.update(&click_msg(5, 2));
            assert_eq!(table.cursor(), 0);
        }

        #[test]
        fn test_table_mouse_not_focused() {
            let rows = vec![vec!["1".into()], vec!["2".into()]];
            let mut table = Table::new().rows(rows).focused(false);

            // Mouse should be ignored when not focused
            table.update(&wheel_down_msg());
            assert_eq!(table.cursor(), 0);

            table.update(&click_msg(5, 2));
            assert_eq!(table.cursor(), 0);
        }

        #[test]
        fn test_table_mouse_wheel_delta_builder() {
            let rows = vec![
                vec!["1".into()],
                vec!["2".into()],
                vec!["3".into()],
                vec!["4".into()],
                vec!["5".into()],
            ];
            let mut table = Table::new().rows(rows).focused(true).mouse_wheel_delta(1); // Single step

            table.update(&wheel_down_msg());
            assert_eq!(table.cursor(), 1); // Only moved by 1
        }

        #[test]
        fn test_table_mouse_release_ignored() {
            let rows = vec![vec!["1".into()], vec!["2".into()]];
            let mut table = Table::new().rows(rows).focused(true);

            // Release event should be ignored
            let release_msg = Message::new(MouseMsg {
                x: 5,
                y: 2,
                shift: false,
                alt: false,
                ctrl: false,
                action: MouseAction::Release,
                button: MouseButton::Left,
            });
            table.update(&release_msg);
            assert_eq!(table.cursor(), 0);
        }
    }
}
