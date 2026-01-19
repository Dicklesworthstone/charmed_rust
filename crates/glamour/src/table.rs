//! Table structure parsing from pulldown-cmark events.
//!
//! This module provides data structures and a state machine for parsing markdown tables
//! into a structured format suitable for rendering with lipgloss styling.
//!
//! # Example
//!
//! ```rust
//! use glamour::table::{ParsedTable, TableParser};
//! use pulldown_cmark::{Parser, Options};
//!
//! let markdown = r#"
//! | Name | Age |
//! |------|-----|
//! | Alice | 30 |
//! | Bob | 25 |
//! "#;
//!
//! let mut opts = Options::empty();
//! opts.insert(Options::ENABLE_TABLES);
//! let parser = Parser::new_ext(markdown, opts);
//!
//! let tables = TableParser::parse_all(parser);
//! assert_eq!(tables.len(), 1);
//! assert_eq!(tables[0].header.len(), 2);
//! ```

use pulldown_cmark::{Alignment, Event, Tag, TagEnd};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Represents a parsed table ready for rendering.
#[derive(Debug, Clone, Default)]
pub struct ParsedTable {
    /// Column alignments from the markdown table definition.
    pub alignments: Vec<Alignment>,
    /// Header cells (the first row).
    pub header: Vec<TableCell>,
    /// Body rows (all rows after the header).
    pub rows: Vec<Vec<TableCell>>,
}

impl ParsedTable {
    /// Creates a new empty parsed table.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of columns in this table.
    #[must_use]
    pub fn column_count(&self) -> usize {
        self.alignments.len()
    }

    /// Returns the total number of rows (header + body).
    #[must_use]
    pub fn row_count(&self) -> usize {
        let header_rows = if self.header.is_empty() { 0 } else { 1 };
        header_rows + self.rows.len()
    }

    /// Returns true if the table has no content.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.header.is_empty() && self.rows.is_empty()
    }
}

/// A single cell in a table.
#[derive(Debug, Clone)]
pub struct TableCell {
    /// The text content of the cell.
    pub content: String,
    /// The alignment for this cell (inherited from column).
    pub alignment: Alignment,
}

impl Default for TableCell {
    fn default() -> Self {
        Self {
            content: String::new(),
            alignment: Alignment::None,
        }
    }
}

impl TableCell {
    /// Creates a new table cell with content and alignment.
    #[must_use]
    pub fn new(content: impl Into<String>, alignment: Alignment) -> Self {
        Self {
            content: content.into(),
            alignment,
        }
    }

    /// Creates a new table cell with default (left) alignment.
    #[must_use]
    pub fn with_content(content: impl Into<String>) -> Self {
        Self::new(content, Alignment::None)
    }
}

/// State machine for parsing table events from pulldown-cmark.
#[derive(Debug, Clone, Default)]
pub enum TableState {
    /// Not inside a table.
    #[default]
    None,
    /// Inside a table, have column alignments.
    InTable { alignments: Vec<Alignment> },
    /// Inside the table header row.
    InHead {
        alignments: Vec<Alignment>,
        cells: Vec<TableCell>,
        current_cell: String,
    },
    /// Inside a table body row.
    InRow {
        alignments: Vec<Alignment>,
        header: Vec<TableCell>,
        rows: Vec<Vec<TableCell>>,
        current_row: Vec<TableCell>,
        current_cell: String,
    },
}

impl TableState {
    /// Creates a new table state machine in the initial state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if currently inside a table.
    #[must_use]
    pub fn in_table(&self) -> bool {
        !matches!(self, TableState::None)
    }

    /// Handle a pulldown-cmark event and return a completed table if one is finished.
    ///
    /// Returns `Some(ParsedTable)` when a table is complete, `None` otherwise.
    pub fn handle_event(&mut self, event: Event<'_>) -> Option<ParsedTable> {
        match event {
            Event::Start(Tag::Table(alignments)) => {
                *self = TableState::InTable { alignments };
                None
            }

            Event::Start(Tag::TableHead) => {
                if let TableState::InTable { alignments } =
                    std::mem::replace(self, TableState::None)
                {
                    *self = TableState::InHead {
                        alignments,
                        cells: Vec::new(),
                        current_cell: String::new(),
                    };
                }
                None
            }

            Event::End(TagEnd::TableHead) => {
                if let TableState::InHead {
                    alignments,
                    cells,
                    current_cell: _,
                } = std::mem::replace(self, TableState::None)
                {
                    *self = TableState::InRow {
                        alignments,
                        header: cells,
                        rows: Vec::new(),
                        current_row: Vec::new(),
                        current_cell: String::new(),
                    };
                }
                None
            }

            Event::Start(Tag::TableRow) => {
                // Clear current row for a new body row
                if let TableState::InRow { current_row, .. } = self {
                    current_row.clear();
                }
                None
            }

            Event::End(TagEnd::TableRow) => {
                if let TableState::InRow {
                    alignments,
                    rows,
                    current_row,
                    ..
                } = self
                {
                    // Store the completed row
                    let row = std::mem::take(current_row);
                    rows.push(row);

                    // Reset alignment index for cells we'll read
                    let _ = alignments;
                }
                None
            }

            Event::Start(Tag::TableCell) => {
                // Clear current cell
                match self {
                    TableState::InHead { current_cell, .. } => {
                        current_cell.clear();
                    }
                    TableState::InRow { current_cell, .. } => {
                        current_cell.clear();
                    }
                    _ => {}
                }
                None
            }

            Event::End(TagEnd::TableCell) => {
                match self {
                    TableState::InHead {
                        alignments,
                        cells,
                        current_cell,
                    } => {
                        let alignment = alignments
                            .get(cells.len())
                            .copied()
                            .unwrap_or(Alignment::None);
                        let content = current_cell.trim().to_string();
                        cells.push(TableCell::new(content, alignment));
                    }
                    TableState::InRow {
                        alignments,
                        current_row,
                        current_cell,
                        ..
                    } => {
                        let alignment = alignments
                            .get(current_row.len())
                            .copied()
                            .unwrap_or(Alignment::None);
                        let content = current_cell.trim().to_string();
                        current_row.push(TableCell::new(content, alignment));
                    }
                    _ => {}
                }
                None
            }

            Event::End(TagEnd::Table) => {
                // Finalize and return the completed table
                self.finalize()
            }

            // Handle inline content within cells
            Event::Text(text) => {
                self.push_text(&text);
                None
            }

            Event::Code(code) => {
                self.push_text("`");
                self.push_text(&code);
                self.push_text("`");
                None
            }

            Event::SoftBreak | Event::HardBreak => {
                self.push_text(" ");
                None
            }

            // Handle inline formatting markers
            Event::Start(Tag::Emphasis) | Event::End(TagEnd::Emphasis) => {
                self.push_text("_");
                None
            }

            Event::Start(Tag::Strong) | Event::End(TagEnd::Strong) => {
                self.push_text("**");
                None
            }

            Event::Start(Tag::Strikethrough) | Event::End(TagEnd::Strikethrough) => {
                self.push_text("~~");
                None
            }

            _ => None,
        }
    }

    /// Push text to the current cell buffer.
    fn push_text(&mut self, text: &str) {
        match self {
            TableState::InHead { current_cell, .. } => {
                current_cell.push_str(text);
            }
            TableState::InRow { current_cell, .. } => {
                current_cell.push_str(text);
            }
            _ => {}
        }
    }

    /// Finalize parsing and return the completed table.
    fn finalize(&mut self) -> Option<ParsedTable> {
        match std::mem::replace(self, TableState::None) {
            TableState::InRow {
                alignments,
                header,
                rows,
                ..
            } => Some(ParsedTable {
                alignments,
                header,
                rows,
            }),
            _ => None,
        }
    }
}

/// High-level table parser that extracts all tables from markdown events.
pub struct TableParser;

impl TableParser {
    /// Parse all tables from a pulldown-cmark event iterator.
    ///
    /// Returns a vector of all tables found in the markdown content.
    pub fn parse_all<'a>(events: impl Iterator<Item = Event<'a>>) -> Vec<ParsedTable> {
        let mut tables = Vec::new();
        let mut state = TableState::new();

        for event in events {
            if let Some(table) = state.handle_event(event) {
                tables.push(table);
            }
        }

        tables
    }

    /// Parse the first table from a pulldown-cmark event iterator.
    ///
    /// Returns `Some(ParsedTable)` if a table is found, `None` otherwise.
    pub fn parse_first<'a>(events: impl Iterator<Item = Event<'a>>) -> Option<ParsedTable> {
        let mut state = TableState::new();

        for event in events {
            if let Some(table) = state.handle_event(event) {
                return Some(table);
            }
        }

        None
    }
}

/// Convert pulldown-cmark alignment to a position string for lipgloss.
#[must_use]
pub fn alignment_to_position(alignment: Alignment) -> &'static str {
    match alignment {
        Alignment::None | Alignment::Left => "left",
        Alignment::Center => "center",
        Alignment::Right => "right",
    }
}

// ============================================================================
// Column Width Calculation
// ============================================================================

/// Configuration for column width calculation.
#[derive(Debug, Clone)]
pub struct ColumnWidthConfig {
    /// Minimum width for any column.
    pub min_width: usize,
    /// Maximum total table width (0 = no limit).
    pub max_table_width: usize,
    /// Padding to add to each column (cells on each side).
    pub cell_padding: usize,
    /// Width of vertical borders between columns.
    pub border_width: usize,
}

impl Default for ColumnWidthConfig {
    fn default() -> Self {
        Self {
            min_width: 3,
            max_table_width: 0,
            cell_padding: 1,
            border_width: 1,
        }
    }
}

impl ColumnWidthConfig {
    /// Creates a new column width configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the minimum width for any column.
    #[must_use]
    pub fn min_width(mut self, width: usize) -> Self {
        self.min_width = width;
        self
    }

    /// Sets the maximum total table width.
    #[must_use]
    pub fn max_table_width(mut self, width: usize) -> Self {
        self.max_table_width = width;
        self
    }

    /// Sets the cell padding (space on each side of cell content).
    #[must_use]
    pub fn cell_padding(mut self, padding: usize) -> Self {
        self.cell_padding = padding;
        self
    }

    /// Sets the border width between columns.
    #[must_use]
    pub fn border_width(mut self, width: usize) -> Self {
        self.border_width = width;
        self
    }
}

/// Calculated column widths for a table.
#[derive(Debug, Clone)]
pub struct ColumnWidths {
    /// Width for each column (content width, not including padding).
    pub widths: Vec<usize>,
    /// Total table width including borders and padding.
    pub total_width: usize,
}

impl ColumnWidths {
    /// Returns the number of columns.
    #[must_use]
    pub fn column_count(&self) -> usize {
        self.widths.len()
    }

    /// Returns the width for a specific column.
    #[must_use]
    pub fn width(&self, column: usize) -> Option<usize> {
        self.widths.get(column).copied()
    }
}

/// Calculate optimal column widths for a parsed table.
///
/// This algorithm:
/// 1. Measures the maximum content width for each column
/// 2. Applies minimum width constraints
/// 3. If max_table_width is set, shrinks columns proportionally to fit
///
/// # Example
///
/// ```rust
/// use glamour::table::{ParsedTable, TableCell, ColumnWidthConfig, calculate_column_widths};
/// use pulldown_cmark::Alignment;
///
/// let table = ParsedTable {
///     alignments: vec![Alignment::Left, Alignment::Right],
///     header: vec![
///         TableCell::new("Name", Alignment::Left),
///         TableCell::new("Age", Alignment::Right),
///     ],
///     rows: vec![
///         vec![
///             TableCell::new("Alice", Alignment::Left),
///             TableCell::new("30", Alignment::Right),
///         ],
///     ],
/// };
///
/// let config = ColumnWidthConfig::default();
/// let widths = calculate_column_widths(&table, &config);
/// assert_eq!(widths.widths.len(), 2);
/// ```
#[must_use]
pub fn calculate_column_widths(table: &ParsedTable, config: &ColumnWidthConfig) -> ColumnWidths {
    let column_count = table.column_count();
    if column_count == 0 {
        return ColumnWidths {
            widths: Vec::new(),
            total_width: 0,
        };
    }

    // Step 1: Calculate maximum content width for each column
    let mut widths: Vec<usize> = vec![0; column_count];

    // Measure header
    for (i, cell) in table.header.iter().enumerate() {
        if i < column_count {
            let cell_width = cell.content.width();
            widths[i] = widths[i].max(cell_width);
        }
    }

    // Measure body rows
    for row in &table.rows {
        for (i, cell) in row.iter().enumerate() {
            if i < column_count {
                let cell_width = cell.content.width();
                widths[i] = widths[i].max(cell_width);
            }
        }
    }

    // Step 2: Apply minimum width constraint
    for width in &mut widths {
        *width = (*width).max(config.min_width);
    }

    // Step 3: Calculate total width with padding and borders
    let total_content_width: usize = widths.iter().sum();
    let total_padding = column_count * config.cell_padding * 2;
    let total_borders = (column_count + 1) * config.border_width;
    let mut total_width = total_content_width + total_padding + total_borders;

    // Step 4: Shrink columns if max_table_width is set and exceeded
    if config.max_table_width > 0 && total_width > config.max_table_width {
        let fixed_overhead = total_padding + total_borders;
        let available_content = config.max_table_width.saturating_sub(fixed_overhead);
        let min_required = column_count * config.min_width;

        if available_content >= min_required {
            // Proportionally shrink columns
            let current_content: usize = widths.iter().sum();
            if current_content > 0 {
                let scale = available_content as f64 / current_content as f64;
                let mut remaining = available_content;

                // Scale all but the last column
                for width in widths.iter_mut().take(column_count - 1) {
                    let scaled = (*width as f64 * scale).floor() as usize;
                    let new_width = scaled.max(config.min_width);
                    *width = new_width;
                    remaining = remaining.saturating_sub(new_width);
                }

                // Give remaining space to last column
                if let Some(last) = widths.last_mut() {
                    *last = remaining.max(config.min_width);
                }
            }
        } else {
            // Can't fit even with minimum widths - use minimums
            widths.fill(config.min_width);
        }

        // Recalculate total width
        let total_content_width: usize = widths.iter().sum();
        total_width = total_content_width + total_padding + total_borders;
    }

    ColumnWidths {
        widths,
        total_width,
    }
}

/// Measure the display width of a string, handling unicode properly.
#[must_use]
pub fn measure_width(s: &str) -> usize {
    s.width()
}

// ============================================================================
// Cell Alignment and Padding
// ============================================================================

/// Pad content to a target width with the specified alignment.
///
/// If the content is already wider than the target width, it is returned unchanged.
///
/// # Example
///
/// ```rust
/// use glamour::table::pad_content;
/// use pulldown_cmark::Alignment;
///
/// assert_eq!(pad_content("Hi", 6, Alignment::Left), "Hi    ");
/// assert_eq!(pad_content("Hi", 6, Alignment::Right), "    Hi");
/// assert_eq!(pad_content("Hi", 6, Alignment::Center), "  Hi  ");
/// ```
#[must_use]
pub fn pad_content(content: &str, width: usize, alignment: Alignment) -> String {
    let content_width = content.width();

    if content_width >= width {
        return content.to_string();
    }

    let padding_needed = width - content_width;

    match alignment {
        Alignment::None | Alignment::Left => {
            format!("{}{}", content, " ".repeat(padding_needed))
        }
        Alignment::Right => {
            format!("{}{}", " ".repeat(padding_needed), content)
        }
        Alignment::Center => {
            let left_pad = padding_needed / 2;
            let right_pad = padding_needed - left_pad;
            format!(
                "{}{}{}",
                " ".repeat(left_pad),
                content,
                " ".repeat(right_pad)
            )
        }
    }
}

/// Render a cell with proper alignment and optional cell margins.
///
/// This function pads the cell content to the specified column width
/// and adds cell margins (spaces) on each side.
///
/// # Arguments
///
/// * `cell` - The table cell to render
/// * `col_width` - The column width (content area, not including margins)
/// * `cell_margin` - Number of space characters to add on each side
///
/// # Example
///
/// ```rust
/// use glamour::table::{render_cell, TableCell};
/// use pulldown_cmark::Alignment;
///
/// let cell = TableCell::new("Hi", Alignment::Center);
/// let rendered = render_cell(&cell, 6, 1);
/// assert_eq!(rendered, "   Hi   "); // 1 margin + "  Hi  " + 1 margin
/// ```
#[must_use]
pub fn render_cell(cell: &TableCell, col_width: usize, cell_margin: usize) -> String {
    let padded = pad_content(&cell.content, col_width, cell.alignment);
    let margin = " ".repeat(cell_margin);
    format!("{}{}{}", margin, padded, margin)
}

/// Render a cell content string with alignment and optional margins.
///
/// This is a convenience function when you have the content and alignment
/// separately (not in a `TableCell`).
///
/// # Example
///
/// ```rust
/// use glamour::table::render_cell_content;
/// use pulldown_cmark::Alignment;
///
/// let rendered = render_cell_content("Hello", 10, Alignment::Right, 1);
/// assert_eq!(rendered, "      Hello "); // margin + 5 spaces + Hello + margin
/// ```
#[must_use]
pub fn render_cell_content(
    content: &str,
    col_width: usize,
    alignment: Alignment,
    cell_margin: usize,
) -> String {
    let padded = pad_content(content, col_width, alignment);
    let margin = " ".repeat(cell_margin);
    format!("{}{}{}", margin, padded, margin)
}

/// Align multiple cells in a row to their respective column widths.
///
/// Returns a vector of aligned cell strings ready for joining with separators.
///
/// # Example
///
/// ```rust
/// use glamour::table::{align_row, TableCell};
/// use pulldown_cmark::Alignment;
///
/// let cells = vec![
///     TableCell::new("Alice", Alignment::Left),
///     TableCell::new("30", Alignment::Right),
/// ];
/// let widths = vec![10, 5];
/// let aligned = align_row(&cells, &widths, 1);
///
/// assert_eq!(aligned.len(), 2);
/// assert_eq!(aligned[0], " Alice      "); // left aligned in 10 chars + margins
/// assert_eq!(aligned[1], "    30 ");      // right aligned in 5 chars + margins
/// ```
#[must_use]
pub fn align_row(cells: &[TableCell], col_widths: &[usize], cell_margin: usize) -> Vec<String> {
    cells
        .iter()
        .zip(col_widths.iter())
        .map(|(cell, &width)| render_cell(cell, width, cell_margin))
        .collect()
}

/// Truncate content to fit within a maximum width, adding an ellipsis if needed.
///
/// This handles unicode-aware truncation by measuring display width.
/// The ellipsis ("…") takes 1 display unit.
///
/// # Example
///
/// ```rust
/// use glamour::table::truncate_content;
///
/// assert_eq!(truncate_content("Hello, World!", 5), "Hell…");
/// assert_eq!(truncate_content("Hi", 10), "Hi");
/// assert_eq!(truncate_content("日本語", 4), "日…"); // CJK chars are 2 wide
/// ```
#[must_use]
pub fn truncate_content(content: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    let content_width = content.width();
    if content_width <= max_width {
        return content.to_string();
    }

    // Need to truncate - ellipsis takes 1 unit
    let target_width = max_width.saturating_sub(1);
    let mut result = String::new();
    let mut current_width = 0;

    for c in content.chars() {
        let char_width = c.width().unwrap_or(0);
        if current_width + char_width > target_width {
            break;
        }
        result.push(c);
        current_width += char_width;
    }

    result.push('…');
    result
}

// ============================================================================
// Border Rendering
// ============================================================================

/// Border characters for table rendering.
#[derive(Debug, Clone, Copy)]
pub struct TableBorder {
    /// Top-left corner character.
    pub top_left: &'static str,
    /// Top-right corner character.
    pub top_right: &'static str,
    /// Bottom-left corner character.
    pub bottom_left: &'static str,
    /// Bottom-right corner character.
    pub bottom_right: &'static str,
    /// Horizontal line character.
    pub horizontal: &'static str,
    /// Vertical line character.
    pub vertical: &'static str,
    /// Cross intersection character.
    pub cross: &'static str,
    /// Top T-intersection character.
    pub top_t: &'static str,
    /// Bottom T-intersection character.
    pub bottom_t: &'static str,
    /// Left T-intersection character.
    pub left_t: &'static str,
    /// Right T-intersection character.
    pub right_t: &'static str,
}

/// Standard ASCII border using +, -, and | characters.
pub const ASCII_BORDER: TableBorder = TableBorder {
    top_left: "+",
    top_right: "+",
    bottom_left: "+",
    bottom_right: "+",
    horizontal: "-",
    vertical: "|",
    cross: "+",
    top_t: "+",
    bottom_t: "+",
    left_t: "+",
    right_t: "+",
};

/// Unicode rounded border (matches lipgloss RoundedBorder).
pub const ROUNDED_BORDER: TableBorder = TableBorder {
    top_left: "╭",
    top_right: "╮",
    bottom_left: "╰",
    bottom_right: "╯",
    horizontal: "─",
    vertical: "│",
    cross: "┼",
    top_t: "┬",
    bottom_t: "┴",
    left_t: "├",
    right_t: "┤",
};

/// Unicode normal/sharp border (matches lipgloss NormalBorder).
pub const NORMAL_BORDER: TableBorder = TableBorder {
    top_left: "┌",
    top_right: "┐",
    bottom_left: "└",
    bottom_right: "┘",
    horizontal: "─",
    vertical: "│",
    cross: "┼",
    top_t: "┬",
    bottom_t: "┴",
    left_t: "├",
    right_t: "┤",
};

/// Double-line Unicode border.
pub const DOUBLE_BORDER: TableBorder = TableBorder {
    top_left: "╔",
    top_right: "╗",
    bottom_left: "╚",
    bottom_right: "╝",
    horizontal: "═",
    vertical: "║",
    cross: "╬",
    top_t: "╦",
    bottom_t: "╩",
    left_t: "╠",
    right_t: "╣",
};

/// No visible border (empty strings).
pub const NO_BORDER: TableBorder = TableBorder {
    top_left: "",
    top_right: "",
    bottom_left: "",
    bottom_right: "",
    horizontal: "",
    vertical: "",
    cross: "",
    top_t: "",
    bottom_t: "",
    left_t: "",
    right_t: "",
};

/// Position of a horizontal border line within the table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderPosition {
    /// Top edge of the table.
    Top,
    /// Middle (between header and body, or between rows).
    Middle,
    /// Bottom edge of the table.
    Bottom,
}

/// Style configuration for table rendering.
#[derive(Debug, Clone)]
pub struct TableRenderConfig {
    /// Border character set to use.
    pub border: TableBorder,
    /// Whether to show a separator between header and body.
    pub header_separator: bool,
    /// Whether to show separators between body rows.
    pub row_separator: bool,
    /// Padding (spaces) on each side of cell content.
    pub cell_padding: usize,
}

impl Default for TableRenderConfig {
    fn default() -> Self {
        Self {
            border: ROUNDED_BORDER,
            header_separator: true,
            row_separator: false,
            cell_padding: 1,
        }
    }
}

impl TableRenderConfig {
    /// Creates a new table render configuration with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the border character set.
    #[must_use]
    pub fn border(mut self, border: TableBorder) -> Self {
        self.border = border;
        self
    }

    /// Sets whether to show the header separator.
    #[must_use]
    pub fn header_separator(mut self, show: bool) -> Self {
        self.header_separator = show;
        self
    }

    /// Sets whether to show row separators.
    #[must_use]
    pub fn row_separator(mut self, show: bool) -> Self {
        self.row_separator = show;
        self
    }

    /// Sets the cell padding.
    #[must_use]
    pub fn cell_padding(mut self, padding: usize) -> Self {
        self.cell_padding = padding;
        self
    }
}

/// Render a horizontal border line.
///
/// # Arguments
///
/// * `widths` - Column content widths (not including padding)
/// * `border` - Border character set
/// * `position` - Position of the border (top, middle, bottom)
/// * `cell_padding` - Padding on each side of cell content
///
/// # Example
///
/// ```rust
/// use glamour::table::{render_horizontal_border, ASCII_BORDER, BorderPosition};
///
/// let widths = vec![5, 3, 7];
/// let result = render_horizontal_border(&widths, &ASCII_BORDER, BorderPosition::Top, 1);
/// assert_eq!(result, "+-------+-----+---------+");
/// ```
#[must_use]
pub fn render_horizontal_border(
    widths: &[usize],
    border: &TableBorder,
    position: BorderPosition,
    cell_padding: usize,
) -> String {
    if widths.is_empty() || border.horizontal.is_empty() {
        return String::new();
    }

    let (left, mid, right) = match position {
        BorderPosition::Top => (border.top_left, border.top_t, border.top_right),
        BorderPosition::Middle => (border.left_t, border.cross, border.right_t),
        BorderPosition::Bottom => (border.bottom_left, border.bottom_t, border.bottom_right),
    };

    let mut result = String::from(left);
    let padding_width = cell_padding * 2;

    for (i, width) in widths.iter().enumerate() {
        // Content width + padding on each side
        result.push_str(&border.horizontal.repeat(width + padding_width));
        if i < widths.len() - 1 {
            result.push_str(mid);
        }
    }

    result.push_str(right);
    result
}

/// Render a data row with vertical borders.
///
/// # Arguments
///
/// * `cells` - The cells to render
/// * `widths` - Column content widths (not including padding)
/// * `border` - Border character set
/// * `cell_padding` - Padding on each side of cell content
///
/// # Example
///
/// ```rust
/// use glamour::table::{render_data_row, TableCell, ASCII_BORDER};
/// use pulldown_cmark::Alignment;
///
/// let cells = vec![
///     TableCell::new("Alice", Alignment::Left),
///     TableCell::new("30", Alignment::Right),
/// ];
/// let widths = vec![5, 3];
/// let result = render_data_row(&cells, &widths, &ASCII_BORDER, 1);
/// assert_eq!(result, "| Alice |  30 |");
/// ```
#[must_use]
pub fn render_data_row(
    cells: &[TableCell],
    widths: &[usize],
    border: &TableBorder,
    cell_padding: usize,
) -> String {
    let mut result = String::from(border.vertical);
    let padding = " ".repeat(cell_padding);

    for (i, cell) in cells.iter().enumerate() {
        let width = widths.get(i).copied().unwrap_or(0);
        let padded = pad_content(&cell.content, width, cell.alignment);
        result.push_str(&padding);
        result.push_str(&padded);
        result.push_str(&padding);
        result.push_str(border.vertical);
    }

    // Handle missing cells (if row has fewer cells than widths)
    for width in widths.iter().skip(cells.len()) {
        result.push_str(&padding);
        result.push_str(&" ".repeat(*width));
        result.push_str(&padding);
        result.push_str(border.vertical);
    }

    result
}

/// Render a complete table with borders.
///
/// # Arguments
///
/// * `table` - The parsed table to render
/// * `config` - Render configuration (border style, separators, etc.)
///
/// # Example
///
/// ```rust
/// use glamour::table::{render_table, ParsedTable, TableCell, TableRenderConfig, ASCII_BORDER};
/// use pulldown_cmark::Alignment;
///
/// let table = ParsedTable {
///     alignments: vec![Alignment::Left, Alignment::Right],
///     header: vec![
///         TableCell::new("Name", Alignment::Left),
///         TableCell::new("Age", Alignment::Right),
///     ],
///     rows: vec![
///         vec![
///             TableCell::new("Alice", Alignment::Left),
///             TableCell::new("30", Alignment::Right),
///         ],
///     ],
/// };
///
/// let config = TableRenderConfig::new().border(ASCII_BORDER);
/// let rendered = render_table(&table, &config);
/// assert!(rendered.contains("+"));
/// assert!(rendered.contains("Alice"));
/// ```
#[must_use]
pub fn render_table(table: &ParsedTable, config: &TableRenderConfig) -> String {
    if table.is_empty() {
        return String::new();
    }

    // Calculate column widths
    let width_config = ColumnWidthConfig::new()
        .cell_padding(config.cell_padding)
        .border_width(1);
    let column_widths = calculate_column_widths(table, &width_config);
    let widths = &column_widths.widths;

    let mut lines = Vec::new();

    // Top border
    let top = render_horizontal_border(
        widths,
        &config.border,
        BorderPosition::Top,
        config.cell_padding,
    );
    if !top.is_empty() {
        lines.push(top);
    }

    // Header row
    if !table.header.is_empty() {
        lines.push(render_data_row(
            &table.header,
            widths,
            &config.border,
            config.cell_padding,
        ));
    }

    // Header separator
    if config.header_separator && !table.header.is_empty() {
        let sep = render_horizontal_border(
            widths,
            &config.border,
            BorderPosition::Middle,
            config.cell_padding,
        );
        if !sep.is_empty() {
            lines.push(sep);
        }
    }

    // Body rows
    for (i, row) in table.rows.iter().enumerate() {
        lines.push(render_data_row(
            row,
            widths,
            &config.border,
            config.cell_padding,
        ));

        // Optional row separators (except after last row)
        if config.row_separator && i < table.rows.len() - 1 {
            let sep = render_horizontal_border(
                widths,
                &config.border,
                BorderPosition::Middle,
                config.cell_padding,
            );
            if !sep.is_empty() {
                lines.push(sep);
            }
        }
    }

    // Bottom border
    let bottom = render_horizontal_border(
        widths,
        &config.border,
        BorderPosition::Bottom,
        config.cell_padding,
    );
    if !bottom.is_empty() {
        lines.push(bottom);
    }

    lines.join("\n")
}

// ============================================================================
// Header Styling
// ============================================================================

/// Text transformation options for header content.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TextTransform {
    /// No transformation - text as-is.
    #[default]
    None,
    /// Convert to UPPERCASE.
    Uppercase,
    /// Convert to lowercase.
    Lowercase,
    /// Capitalize first letter of each word.
    Capitalize,
}

impl TextTransform {
    /// Apply the transformation to a string.
    #[must_use]
    pub fn apply(&self, text: &str) -> String {
        match self {
            TextTransform::None => text.to_string(),
            TextTransform::Uppercase => text.to_uppercase(),
            TextTransform::Lowercase => text.to_lowercase(),
            TextTransform::Capitalize => text
                .split_whitespace()
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(c) => c.to_uppercase().chain(chars).collect::<String>(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" "),
        }
    }
}

/// Configuration for header row styling.
#[derive(Debug, Clone, Default)]
pub struct HeaderStyle {
    /// Whether to render header text in bold.
    pub bold: bool,
    /// Whether to render header text in italic.
    pub italic: bool,
    /// Whether to underline header text.
    pub underline: bool,
    /// Text transformation to apply.
    pub transform: TextTransform,
    /// Optional foreground color (CSS hex, ANSI code, or color name).
    pub foreground: Option<String>,
    /// Optional background color (CSS hex, ANSI code, or color name).
    pub background: Option<String>,
}

impl HeaderStyle {
    /// Creates a new header style with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable bold text.
    #[must_use]
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Enable italic text.
    #[must_use]
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// Enable underlined text.
    #[must_use]
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    /// Set the text transformation.
    #[must_use]
    pub fn transform(mut self, transform: TextTransform) -> Self {
        self.transform = transform;
        self
    }

    /// Set the foreground color.
    #[must_use]
    pub fn foreground(mut self, color: impl Into<String>) -> Self {
        self.foreground = Some(color.into());
        self
    }

    /// Set the background color.
    #[must_use]
    pub fn background(mut self, color: impl Into<String>) -> Self {
        self.background = Some(color.into());
        self
    }

    /// Build a lipgloss Style from this configuration.
    #[must_use]
    pub fn build_style(&self) -> lipgloss::Style {
        let mut style = lipgloss::Style::new();

        if self.bold {
            style = style.bold();
        }
        if self.italic {
            style = style.italic();
        }
        if self.underline {
            style = style.underline();
        }
        if let Some(ref fg) = self.foreground {
            style = style.foreground(fg.clone());
        }
        if let Some(ref bg) = self.background {
            style = style.background(bg.clone());
        }

        style
    }

    /// Check if any styling is configured.
    #[must_use]
    pub fn has_styling(&self) -> bool {
        self.bold
            || self.italic
            || self.underline
            || self.foreground.is_some()
            || self.background.is_some()
    }
}

/// Render a header row with optional styling.
///
/// # Arguments
///
/// * `cells` - The header cells to render
/// * `widths` - Column content widths
/// * `border` - Border character set
/// * `cell_padding` - Padding on each side of cell content
/// * `style` - Optional header styling
///
/// # Example
///
/// ```rust
/// use glamour::table::{render_header_row, TableCell, ASCII_BORDER, HeaderStyle};
/// use pulldown_cmark::Alignment;
///
/// let cells = vec![
///     TableCell::new("Name", Alignment::Left),
///     TableCell::new("Age", Alignment::Right),
/// ];
/// let widths = vec![10, 5];
/// let style = HeaderStyle::new().bold();
/// let result = render_header_row(&cells, &widths, &ASCII_BORDER, 1, Some(&style));
/// assert!(result.contains("Name"));
/// ```
#[must_use]
pub fn render_header_row(
    cells: &[TableCell],
    widths: &[usize],
    border: &TableBorder,
    cell_padding: usize,
    style: Option<&HeaderStyle>,
) -> String {
    let mut result = String::from(border.vertical);
    let padding = " ".repeat(cell_padding);

    for (i, cell) in cells.iter().enumerate() {
        let width = widths.get(i).copied().unwrap_or(0);

        // Apply text transform if style is provided
        let content = if let Some(s) = style {
            s.transform.apply(&cell.content)
        } else {
            cell.content.clone()
        };

        let padded = pad_content(&content, width, cell.alignment);
        let cell_content = format!("{}{}{}", padding, padded, padding);

        // Apply styling if provided and has styling
        let styled_content = if let Some(s) = style {
            if s.has_styling() {
                s.build_style().render(&cell_content)
            } else {
                cell_content
            }
        } else {
            cell_content
        };

        result.push_str(&styled_content);
        result.push_str(border.vertical);
    }

    // Handle missing cells (if row has fewer cells than widths)
    for width in widths.iter().skip(cells.len()) {
        result.push_str(&padding);
        result.push_str(&" ".repeat(*width));
        result.push_str(&padding);
        result.push_str(border.vertical);
    }

    result
}

/// Render a complete table with borders and optional header styling.
///
/// This is an enhanced version of `render_table` that supports header styling.
///
/// # Example
///
/// ```rust
/// use glamour::table::{render_styled_table, ParsedTable, TableCell, TableRenderConfig, HeaderStyle, ASCII_BORDER};
/// use pulldown_cmark::Alignment;
///
/// let table = ParsedTable {
///     alignments: vec![Alignment::Left],
///     header: vec![TableCell::new("Name", Alignment::Left)],
///     rows: vec![vec![TableCell::new("Alice", Alignment::Left)]],
/// };
///
/// let config = TableRenderConfig::new().border(ASCII_BORDER);
/// let header_style = HeaderStyle::new().bold();
/// let rendered = render_styled_table(&table, &config, Some(&header_style));
/// assert!(rendered.contains("Name"));
/// ```
#[must_use]
pub fn render_styled_table(
    table: &ParsedTable,
    config: &TableRenderConfig,
    header_style: Option<&HeaderStyle>,
) -> String {
    if table.is_empty() {
        return String::new();
    }

    // Calculate column widths
    let width_config = ColumnWidthConfig::new()
        .cell_padding(config.cell_padding)
        .border_width(1);
    let column_widths = calculate_column_widths(table, &width_config);
    let widths = &column_widths.widths;

    let mut lines = Vec::new();

    // Top border
    let top = render_horizontal_border(
        widths,
        &config.border,
        BorderPosition::Top,
        config.cell_padding,
    );
    if !top.is_empty() {
        lines.push(top);
    }

    // Header row with optional styling
    if !table.header.is_empty() {
        lines.push(render_header_row(
            &table.header,
            widths,
            &config.border,
            config.cell_padding,
            header_style,
        ));
    }

    // Header separator
    if config.header_separator && !table.header.is_empty() {
        let sep = render_horizontal_border(
            widths,
            &config.border,
            BorderPosition::Middle,
            config.cell_padding,
        );
        if !sep.is_empty() {
            lines.push(sep);
        }
    }

    // Body rows
    for (i, row) in table.rows.iter().enumerate() {
        lines.push(render_data_row(
            row,
            widths,
            &config.border,
            config.cell_padding,
        ));

        // Optional row separators (except after last row)
        if config.row_separator && i < table.rows.len() - 1 {
            let sep = render_horizontal_border(
                widths,
                &config.border,
                BorderPosition::Middle,
                config.cell_padding,
            );
            if !sep.is_empty() {
                lines.push(sep);
            }
        }
    }

    // Bottom border
    let bottom = render_horizontal_border(
        widths,
        &config.border,
        BorderPosition::Bottom,
        config.cell_padding,
    );
    if !bottom.is_empty() {
        lines.push(bottom);
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::{Options, Parser};

    fn parse_markdown(markdown: &str) -> Vec<ParsedTable> {
        let mut opts = Options::empty();
        opts.insert(Options::ENABLE_TABLES);
        let parser = Parser::new_ext(markdown, opts);
        TableParser::parse_all(parser)
    }

    #[test]
    fn test_simple_table() {
        let markdown = r#"
| Name | Age |
|------|-----|
| Alice | 30 |
| Bob | 25 |
"#;
        let tables = parse_markdown(markdown);

        assert_eq!(tables.len(), 1);
        let table = &tables[0];

        assert_eq!(table.header.len(), 2);
        assert_eq!(table.header[0].content, "Name");
        assert_eq!(table.header[1].content, "Age");

        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[0][0].content, "Alice");
        assert_eq!(table.rows[0][1].content, "30");
        assert_eq!(table.rows[1][0].content, "Bob");
        assert_eq!(table.rows[1][1].content, "25");
    }

    #[test]
    fn test_aligned_columns() {
        let markdown = r#"
| Left | Center | Right |
|:-----|:------:|------:|
| L | C | R |
"#;
        let tables = parse_markdown(markdown);

        assert_eq!(tables.len(), 1);
        let table = &tables[0];

        assert_eq!(table.alignments.len(), 3);
        assert_eq!(table.alignments[0], Alignment::Left);
        assert_eq!(table.alignments[1], Alignment::Center);
        assert_eq!(table.alignments[2], Alignment::Right);

        // Check that cells inherit alignment
        assert_eq!(table.header[0].alignment, Alignment::Left);
        assert_eq!(table.header[1].alignment, Alignment::Center);
        assert_eq!(table.header[2].alignment, Alignment::Right);
    }

    #[test]
    fn test_empty_cells() {
        let markdown = r#"
| A | B | C |
|---|---|---|
| 1 |   | 3 |
|   | 2 |   |
"#;
        let tables = parse_markdown(markdown);

        assert_eq!(tables.len(), 1);
        let table = &tables[0];

        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[0][0].content, "1");
        assert_eq!(table.rows[0][1].content, "");
        assert_eq!(table.rows[0][2].content, "3");
        assert_eq!(table.rows[1][0].content, "");
        assert_eq!(table.rows[1][1].content, "2");
        assert_eq!(table.rows[1][2].content, "");
    }

    #[test]
    fn test_inline_code_in_cells() {
        let markdown = r#"
| Code | Description |
|------|-------------|
| `fn main()` | Entry point |
"#;
        let tables = parse_markdown(markdown);

        assert_eq!(tables.len(), 1);
        let table = &tables[0];

        assert_eq!(table.rows[0][0].content, "`fn main()`");
    }

    #[test]
    fn test_unicode_content() {
        let markdown = r#"
| Emoji | Name |
|-------|------|
| 🦀 | Rust |
| 🐍 | Python |
"#;
        let tables = parse_markdown(markdown);

        assert_eq!(tables.len(), 1);
        let table = &tables[0];

        assert_eq!(table.rows[0][0].content, "🦀");
        assert_eq!(table.rows[0][1].content, "Rust");
        assert_eq!(table.rows[1][0].content, "🐍");
        assert_eq!(table.rows[1][1].content, "Python");
    }

    #[test]
    fn test_multiple_tables() {
        let markdown = r#"
| A | B |
|---|---|
| 1 | 2 |

Some text between tables.

| X | Y | Z |
|---|---|---|
| a | b | c |
"#;
        let tables = parse_markdown(markdown);

        assert_eq!(tables.len(), 2);
        assert_eq!(tables[0].column_count(), 2);
        assert_eq!(tables[1].column_count(), 3);
    }

    #[test]
    fn test_table_with_emphasis() {
        let markdown = r#"
| Style | Example |
|-------|---------|
| Bold | **text** |
| Italic | _text_ |
"#;
        let tables = parse_markdown(markdown);

        assert_eq!(tables.len(), 1);
        let table = &tables[0];

        // Note: inline formatting is preserved as markers in the content
        assert_eq!(table.rows[0][1].content, "**text**");
        assert_eq!(table.rows[1][1].content, "_text_");
    }

    #[test]
    fn test_column_count() {
        let markdown = r#"
| A | B | C | D |
|---|---|---|---|
| 1 | 2 | 3 | 4 |
"#;
        let tables = parse_markdown(markdown);
        let table = &tables[0];

        assert_eq!(table.column_count(), 4);
    }

    #[test]
    fn test_row_count() {
        let markdown = r#"
| Header |
|--------|
| Row 1 |
| Row 2 |
| Row 3 |
"#;
        let tables = parse_markdown(markdown);
        let table = &tables[0];

        assert_eq!(table.row_count(), 4); // 1 header + 3 body rows
    }

    #[test]
    fn test_is_empty() {
        let table = ParsedTable::new();
        assert!(table.is_empty());

        let markdown = r#"
| A |
|---|
"#;
        let tables = parse_markdown(markdown);
        // Table with only header
        assert!(!tables[0].is_empty());
    }

    #[test]
    fn test_parse_first() {
        let markdown = r#"
| First |
|-------|
| 1 |

| Second |
|--------|
| 2 |
"#;
        let mut opts = Options::empty();
        opts.insert(Options::ENABLE_TABLES);
        let parser = Parser::new_ext(markdown, opts);

        let table = TableParser::parse_first(parser).unwrap();
        assert_eq!(table.header[0].content, "First");
    }

    #[test]
    fn test_alignment_to_position() {
        assert_eq!(alignment_to_position(Alignment::None), "left");
        assert_eq!(alignment_to_position(Alignment::Left), "left");
        assert_eq!(alignment_to_position(Alignment::Center), "center");
        assert_eq!(alignment_to_position(Alignment::Right), "right");
    }

    #[test]
    fn test_table_cell_constructors() {
        let cell1 = TableCell::new("hello", Alignment::Right);
        assert_eq!(cell1.content, "hello");
        assert_eq!(cell1.alignment, Alignment::Right);

        let cell2 = TableCell::with_content("world");
        assert_eq!(cell2.content, "world");
        assert_eq!(cell2.alignment, Alignment::None);
    }

    // ========================================================================
    // Column Width Calculation Tests
    // ========================================================================

    #[test]
    fn test_column_width_simple() {
        let table = ParsedTable {
            alignments: vec![Alignment::Left, Alignment::Right],
            header: vec![
                TableCell::new("Name", Alignment::Left),
                TableCell::new("Age", Alignment::Right),
            ],
            rows: vec![
                vec![
                    TableCell::new("Alice", Alignment::Left),
                    TableCell::new("30", Alignment::Right),
                ],
                vec![
                    TableCell::new("Bob", Alignment::Left),
                    TableCell::new("25", Alignment::Right),
                ],
            ],
        };

        let config = ColumnWidthConfig::default();
        let widths = calculate_column_widths(&table, &config);

        // "Alice" is 5 chars, "Name" is 4 chars - should use 5
        assert_eq!(widths.widths[0], 5);
        // "Age" is 3 chars, "30" and "25" are 2 chars - should use 3
        assert_eq!(widths.widths[1], 3);
    }

    #[test]
    fn test_column_width_min_width() {
        let table = ParsedTable {
            alignments: vec![Alignment::Left],
            header: vec![TableCell::new("A", Alignment::Left)],
            rows: vec![vec![TableCell::new("B", Alignment::Left)]],
        };

        let config = ColumnWidthConfig::default().min_width(5);
        let widths = calculate_column_widths(&table, &config);

        // Content is 1 char, but min_width is 5
        assert_eq!(widths.widths[0], 5);
    }

    #[test]
    fn test_column_width_empty_table() {
        let table = ParsedTable::default();
        let config = ColumnWidthConfig::default();
        let widths = calculate_column_widths(&table, &config);

        assert_eq!(widths.widths.len(), 0);
        assert_eq!(widths.total_width, 0);
    }

    #[test]
    fn test_column_width_unicode() {
        let table = ParsedTable {
            alignments: vec![Alignment::Left, Alignment::Left],
            header: vec![
                TableCell::new("Emoji", Alignment::Left),
                TableCell::new("Name", Alignment::Left),
            ],
            rows: vec![vec![
                TableCell::new("🦀", Alignment::Left),
                TableCell::new("Rust", Alignment::Left),
            ]],
        };

        let config = ColumnWidthConfig::default();
        let widths = calculate_column_widths(&table, &config);

        // "Emoji" is 5 chars, "🦀" is 2 display units
        assert_eq!(widths.widths[0], 5);
        // "Name" and "Rust" are both 4 chars
        assert_eq!(widths.widths[1], 4);
    }

    #[test]
    fn test_column_width_max_table_width() {
        let table = ParsedTable {
            alignments: vec![Alignment::Left, Alignment::Left],
            header: vec![
                TableCell::new("VeryLongHeaderName", Alignment::Left),
                TableCell::new("AnotherLongHeader", Alignment::Left),
            ],
            rows: vec![],
        };

        let config = ColumnWidthConfig::default()
            .max_table_width(30)
            .cell_padding(1)
            .border_width(1);
        let widths = calculate_column_widths(&table, &config);

        // Total width should be constrained
        assert!(widths.total_width <= 30);
    }

    #[test]
    fn test_column_width_config_builder() {
        let config = ColumnWidthConfig::new()
            .min_width(5)
            .max_table_width(100)
            .cell_padding(2)
            .border_width(1);

        assert_eq!(config.min_width, 5);
        assert_eq!(config.max_table_width, 100);
        assert_eq!(config.cell_padding, 2);
        assert_eq!(config.border_width, 1);
    }

    #[test]
    fn test_measure_width() {
        assert_eq!(measure_width("hello"), 5);
        assert_eq!(measure_width(""), 0);
        assert_eq!(measure_width("🦀"), 2); // Emoji is 2 display units wide
        assert_eq!(measure_width("café"), 4); // Accented char is 1 unit
    }

    #[test]
    fn test_column_widths_accessors() {
        let widths = ColumnWidths {
            widths: vec![10, 20, 30],
            total_width: 100,
        };

        assert_eq!(widths.column_count(), 3);
        assert_eq!(widths.width(0), Some(10));
        assert_eq!(widths.width(1), Some(20));
        assert_eq!(widths.width(2), Some(30));
        assert_eq!(widths.width(3), None);
    }

    // ========================================================================
    // Cell Alignment and Padding Tests
    // ========================================================================

    #[test]
    fn test_pad_content_left() {
        assert_eq!(pad_content("Hi", 6, Alignment::Left), "Hi    ");
        assert_eq!(pad_content("Hello", 5, Alignment::Left), "Hello");
        assert_eq!(pad_content("Hi", 10, Alignment::None), "Hi        "); // None = Left
    }

    #[test]
    fn test_pad_content_right() {
        assert_eq!(pad_content("Hi", 6, Alignment::Right), "    Hi");
        assert_eq!(pad_content("Hello", 5, Alignment::Right), "Hello");
        assert_eq!(pad_content("X", 5, Alignment::Right), "    X");
    }

    #[test]
    fn test_pad_content_center() {
        assert_eq!(pad_content("Hi", 6, Alignment::Center), "  Hi  ");
        assert_eq!(pad_content("Hi", 5, Alignment::Center), " Hi  "); // Favor right padding
        assert_eq!(pad_content("A", 5, Alignment::Center), "  A  ");
    }

    #[test]
    fn test_pad_content_already_wider() {
        // Content wider than target - return unchanged
        assert_eq!(
            pad_content("Hello, World!", 5, Alignment::Left),
            "Hello, World!"
        );
        assert_eq!(
            pad_content("Hello, World!", 5, Alignment::Center),
            "Hello, World!"
        );
    }

    #[test]
    fn test_pad_content_unicode() {
        // CJK characters are 2 display units wide
        assert_eq!(pad_content("日本", 8, Alignment::Center), "  日本  ");
        assert_eq!(pad_content("日本", 8, Alignment::Left), "日本    ");
        assert_eq!(pad_content("日本", 8, Alignment::Right), "    日本");

        // Emoji is typically 2 display units
        assert_eq!(pad_content("🦀", 6, Alignment::Center), "  🦀  ");
    }

    #[test]
    fn test_render_cell() {
        let cell = TableCell::new("Hello", Alignment::Left);
        assert_eq!(render_cell(&cell, 10, 1), " Hello      ");

        let cell = TableCell::new("Hi", Alignment::Center);
        assert_eq!(render_cell(&cell, 6, 1), "   Hi   ");

        let cell = TableCell::new("X", Alignment::Right);
        assert_eq!(render_cell(&cell, 5, 1), "     X ");
    }

    #[test]
    fn test_render_cell_content() {
        assert_eq!(
            render_cell_content("Hello", 10, Alignment::Right, 1),
            "      Hello "
        );
        assert_eq!(
            render_cell_content("Hi", 6, Alignment::Center, 2),
            "    Hi    "
        );
    }

    #[test]
    fn test_align_row() {
        let cells = vec![
            TableCell::new("Alice", Alignment::Left),
            TableCell::new("30", Alignment::Right),
        ];
        let widths = vec![10, 5];
        let aligned = align_row(&cells, &widths, 1);

        assert_eq!(aligned.len(), 2);
        assert_eq!(aligned[0], " Alice      ");
        assert_eq!(aligned[1], "    30 ");
    }

    #[test]
    fn test_align_row_empty() {
        let cells: Vec<TableCell> = vec![];
        let widths: Vec<usize> = vec![];
        let aligned = align_row(&cells, &widths, 1);
        assert!(aligned.is_empty());
    }

    #[test]
    fn test_truncate_content_simple() {
        assert_eq!(truncate_content("Hello, World!", 5), "Hell…");
        assert_eq!(truncate_content("Hello", 10), "Hello");
        assert_eq!(truncate_content("Hi", 2), "Hi");
    }

    #[test]
    fn test_truncate_content_edge_cases() {
        assert_eq!(truncate_content("Hello", 1), "…");
        assert_eq!(truncate_content("Hello", 0), "");
        assert_eq!(truncate_content("", 5), "");
    }

    #[test]
    fn test_truncate_content_unicode() {
        // CJK characters are 2 wide
        assert_eq!(truncate_content("日本語", 4), "日…"); // 2 for 日 + 1 for ellipsis
        assert_eq!(truncate_content("日本語", 5), "日本…"); // 4 for 日本 + 1 for ellipsis
        assert_eq!(truncate_content("日本語", 6), "日本語"); // Exactly fits

        // Mixed content
        assert_eq!(truncate_content("Hi日本", 5), "Hi日…"); // 2 + 2 + 1
    }

    #[test]
    fn test_alignment_integration() {
        // Integration test: calculate widths and align cells
        let table = ParsedTable {
            alignments: vec![Alignment::Left, Alignment::Center, Alignment::Right],
            header: vec![
                TableCell::new("Name", Alignment::Left),
                TableCell::new("Score", Alignment::Center),
                TableCell::new("Rank", Alignment::Right),
            ],
            rows: vec![
                vec![
                    TableCell::new("Alice", Alignment::Left),
                    TableCell::new("95", Alignment::Center),
                    TableCell::new("1", Alignment::Right),
                ],
                vec![
                    TableCell::new("Bob", Alignment::Left),
                    TableCell::new("87", Alignment::Center),
                    TableCell::new("2", Alignment::Right),
                ],
            ],
        };

        let config = ColumnWidthConfig::default();
        let widths = calculate_column_widths(&table, &config);

        // Align header
        let header_aligned = align_row(&table.header, &widths.widths, 1);
        assert_eq!(header_aligned.len(), 3);

        // Align body rows
        for row in &table.rows {
            let row_aligned = align_row(row, &widths.widths, 1);
            assert_eq!(row_aligned.len(), 3);
        }
    }

    // ========================================================================
    // Border Rendering Tests
    // ========================================================================

    #[test]
    fn test_ascii_border_top() {
        let widths = vec![5, 3, 7];
        let result = render_horizontal_border(&widths, &ASCII_BORDER, BorderPosition::Top, 1);
        assert_eq!(result, "+-------+-----+---------+");
    }

    #[test]
    fn test_ascii_border_middle() {
        let widths = vec![5, 3];
        let result = render_horizontal_border(&widths, &ASCII_BORDER, BorderPosition::Middle, 1);
        assert_eq!(result, "+-------+-----+");
    }

    #[test]
    fn test_ascii_border_bottom() {
        let widths = vec![4, 4];
        let result = render_horizontal_border(&widths, &ASCII_BORDER, BorderPosition::Bottom, 1);
        assert_eq!(result, "+------+------+");
    }

    #[test]
    fn test_rounded_border_top() {
        let widths = vec![4, 4];
        let result = render_horizontal_border(&widths, &ROUNDED_BORDER, BorderPosition::Top, 1);
        assert_eq!(result, "╭──────┬──────╮");
    }

    #[test]
    fn test_normal_border_top() {
        let widths = vec![3];
        let result = render_horizontal_border(&widths, &NORMAL_BORDER, BorderPosition::Top, 1);
        assert_eq!(result, "┌─────┐");
    }

    #[test]
    fn test_double_border_top() {
        let widths = vec![3, 3];
        let result = render_horizontal_border(&widths, &DOUBLE_BORDER, BorderPosition::Top, 1);
        assert_eq!(result, "╔═════╦═════╗");
    }

    #[test]
    fn test_no_border() {
        let widths = vec![5, 5];
        let result = render_horizontal_border(&widths, &NO_BORDER, BorderPosition::Top, 1);
        assert!(result.is_empty());
    }

    #[test]
    fn test_empty_widths() {
        let widths: Vec<usize> = vec![];
        let result = render_horizontal_border(&widths, &ASCII_BORDER, BorderPosition::Top, 1);
        assert!(result.is_empty());
    }

    #[test]
    fn test_render_data_row_ascii() {
        let cells = vec![
            TableCell::new("Alice", Alignment::Left),
            TableCell::new("30", Alignment::Right),
        ];
        let widths = vec![5, 3];
        let result = render_data_row(&cells, &widths, &ASCII_BORDER, 1);
        assert_eq!(result, "| Alice |  30 |");
    }

    #[test]
    fn test_render_data_row_rounded() {
        let cells = vec![TableCell::new("Hi", Alignment::Center)];
        let widths = vec![6];
        let result = render_data_row(&cells, &widths, &ROUNDED_BORDER, 1);
        assert_eq!(result, "│   Hi   │");
    }

    #[test]
    fn test_render_data_row_missing_cells() {
        let cells = vec![TableCell::new("A", Alignment::Left)];
        let widths = vec![3, 3, 3];
        let result = render_data_row(&cells, &widths, &ASCII_BORDER, 1);
        assert_eq!(result, "| A   |     |     |");
    }

    #[test]
    fn test_table_render_config_builder() {
        let config = TableRenderConfig::new()
            .border(ASCII_BORDER)
            .header_separator(false)
            .row_separator(true)
            .cell_padding(2);

        assert!(!config.header_separator);
        assert!(config.row_separator);
        assert_eq!(config.cell_padding, 2);
    }

    #[test]
    fn test_render_table_simple() {
        let table = ParsedTable {
            alignments: vec![Alignment::Left, Alignment::Right],
            header: vec![
                TableCell::new("Name", Alignment::Left),
                TableCell::new("Age", Alignment::Right),
            ],
            rows: vec![vec![
                TableCell::new("Alice", Alignment::Left),
                TableCell::new("30", Alignment::Right),
            ]],
        };

        let config = TableRenderConfig::new().border(ASCII_BORDER);
        let rendered = render_table(&table, &config);

        let lines: Vec<&str> = rendered.lines().collect();
        assert_eq!(lines.len(), 5); // top, header, sep, row, bottom
        assert!(lines[0].starts_with('+'));
        assert!(lines[0].ends_with('+'));
        assert!(lines[1].contains("Name"));
        assert!(lines[1].contains("Age"));
        assert!(lines[3].contains("Alice"));
        assert!(lines[3].contains("30"));
    }

    #[test]
    fn test_render_table_rounded() {
        let table = ParsedTable {
            alignments: vec![Alignment::Left],
            header: vec![TableCell::new("Hello", Alignment::Left)],
            rows: vec![vec![TableCell::new("World", Alignment::Left)]],
        };

        let config = TableRenderConfig::new().border(ROUNDED_BORDER);
        let rendered = render_table(&table, &config);

        assert!(rendered.contains('╭'));
        assert!(rendered.contains('╰'));
        assert!(rendered.contains('│'));
    }

    #[test]
    fn test_render_table_no_header_separator() {
        let table = ParsedTable {
            alignments: vec![Alignment::Left],
            header: vec![TableCell::new("A", Alignment::Left)],
            rows: vec![vec![TableCell::new("B", Alignment::Left)]],
        };

        let config = TableRenderConfig::new()
            .border(ASCII_BORDER)
            .header_separator(false);
        let rendered = render_table(&table, &config);

        let lines: Vec<&str> = rendered.lines().collect();
        assert_eq!(lines.len(), 4); // top, header, row, bottom (no separator)
    }

    #[test]
    fn test_render_table_with_row_separators() {
        let table = ParsedTable {
            alignments: vec![Alignment::Left],
            header: vec![TableCell::new("H", Alignment::Left)],
            rows: vec![
                vec![TableCell::new("R1", Alignment::Left)],
                vec![TableCell::new("R2", Alignment::Left)],
                vec![TableCell::new("R3", Alignment::Left)],
            ],
        };

        let config = TableRenderConfig::new()
            .border(ASCII_BORDER)
            .row_separator(true);
        let rendered = render_table(&table, &config);

        let lines: Vec<&str> = rendered.lines().collect();
        // top + header + header_sep + row1 + row_sep + row2 + row_sep + row3 + bottom
        assert_eq!(lines.len(), 9);
    }

    #[test]
    fn test_render_table_empty() {
        let table = ParsedTable::default();
        let config = TableRenderConfig::default();
        let rendered = render_table(&table, &config);
        assert!(rendered.is_empty());
    }

    #[test]
    fn test_render_table_alignment() {
        let table = ParsedTable {
            alignments: vec![Alignment::Left, Alignment::Center, Alignment::Right],
            header: vec![
                TableCell::new("L", Alignment::Left),
                TableCell::new("C", Alignment::Center),
                TableCell::new("R", Alignment::Right),
            ],
            rows: vec![vec![
                TableCell::new("1", Alignment::Left),
                TableCell::new("2", Alignment::Center),
                TableCell::new("3", Alignment::Right),
            ]],
        };

        let config = TableRenderConfig::new().border(ASCII_BORDER);
        let rendered = render_table(&table, &config);

        // Verify the table renders without panicking and contains expected content
        assert!(rendered.contains("L"));
        assert!(rendered.contains("C"));
        assert!(rendered.contains("R"));
    }

    #[test]
    fn test_border_position_equality() {
        assert_eq!(BorderPosition::Top, BorderPosition::Top);
        assert_ne!(BorderPosition::Top, BorderPosition::Middle);
        assert_ne!(BorderPosition::Middle, BorderPosition::Bottom);
    }

    // ========================================================================
    // Header Styling Tests
    // ========================================================================

    #[test]
    fn test_text_transform_none() {
        assert_eq!(TextTransform::None.apply("Hello World"), "Hello World");
    }

    #[test]
    fn test_text_transform_uppercase() {
        assert_eq!(TextTransform::Uppercase.apply("hello world"), "HELLO WORLD");
        assert_eq!(TextTransform::Uppercase.apply("Name"), "NAME");
    }

    #[test]
    fn test_text_transform_lowercase() {
        assert_eq!(TextTransform::Lowercase.apply("HELLO WORLD"), "hello world");
        assert_eq!(TextTransform::Lowercase.apply("Name"), "name");
    }

    #[test]
    fn test_text_transform_capitalize() {
        assert_eq!(
            TextTransform::Capitalize.apply("hello world"),
            "Hello World"
        );
        assert_eq!(TextTransform::Capitalize.apply("name"), "Name");
        assert_eq!(TextTransform::Capitalize.apply("HELLO"), "HELLO"); // Only capitalizes first letter
    }

    #[test]
    fn test_header_style_builder() {
        let style = HeaderStyle::new()
            .bold()
            .italic()
            .underline()
            .transform(TextTransform::Uppercase)
            .foreground("#ff0000")
            .background("#000000");

        assert!(style.bold);
        assert!(style.italic);
        assert!(style.underline);
        assert_eq!(style.transform, TextTransform::Uppercase);
        assert_eq!(style.foreground, Some("#ff0000".to_string()));
        assert_eq!(style.background, Some("#000000".to_string()));
    }

    #[test]
    fn test_header_style_has_styling() {
        let empty = HeaderStyle::new();
        assert!(!empty.has_styling());

        let bold = HeaderStyle::new().bold();
        assert!(bold.has_styling());

        let fg = HeaderStyle::new().foreground("#fff");
        assert!(fg.has_styling());
    }

    #[test]
    fn test_render_header_row_no_style() {
        let cells = vec![
            TableCell::new("Name", Alignment::Left),
            TableCell::new("Age", Alignment::Right),
        ];
        let widths = vec![10, 5];
        let result = render_header_row(&cells, &widths, &ASCII_BORDER, 1, None);

        assert!(result.contains("Name"));
        assert!(result.contains("Age"));
        assert!(result.starts_with('|'));
        assert!(result.ends_with('|'));
    }

    #[test]
    fn test_render_header_row_with_transform() {
        let cells = vec![TableCell::new("name", Alignment::Left)];
        let widths = vec![10];
        let style = HeaderStyle::new().transform(TextTransform::Uppercase);
        let result = render_header_row(&cells, &widths, &ASCII_BORDER, 1, Some(&style));

        assert!(result.contains("NAME")); // Uppercase transform applied
        assert!(!result.contains("name"));
    }

    #[test]
    fn test_render_header_row_with_bold() {
        let cells = vec![TableCell::new("Header", Alignment::Left)];
        let widths = vec![10];
        let style = HeaderStyle::new().bold();
        let result = render_header_row(&cells, &widths, &ASCII_BORDER, 1, Some(&style));

        // Should contain ANSI bold escape sequence
        assert!(result.contains("\x1b[1m")); // Bold start
        assert!(result.contains("Header"));
    }

    #[test]
    fn test_render_styled_table() {
        let table = ParsedTable {
            alignments: vec![Alignment::Left],
            header: vec![TableCell::new("name", Alignment::Left)],
            rows: vec![vec![TableCell::new("Alice", Alignment::Left)]],
        };

        let config = TableRenderConfig::new().border(ASCII_BORDER);
        let style = HeaderStyle::new()
            .bold()
            .transform(TextTransform::Uppercase);
        let rendered = render_styled_table(&table, &config, Some(&style));

        // Header should be uppercase and bold
        assert!(rendered.contains("NAME"));
        // Body should remain unchanged
        assert!(rendered.contains("Alice"));
    }

    #[test]
    fn test_render_styled_table_no_style() {
        let table = ParsedTable {
            alignments: vec![Alignment::Left],
            header: vec![TableCell::new("Name", Alignment::Left)],
            rows: vec![vec![TableCell::new("Alice", Alignment::Left)]],
        };

        let config = TableRenderConfig::new().border(ASCII_BORDER);
        let rendered = render_styled_table(&table, &config, None);

        // Should render normally
        assert!(rendered.contains("Name"));
        assert!(rendered.contains("Alice"));
    }

    #[test]
    fn test_render_styled_table_empty() {
        let table = ParsedTable::default();
        let config = TableRenderConfig::default();
        let style = HeaderStyle::new().bold();
        let rendered = render_styled_table(&table, &config, Some(&style));
        assert!(rendered.is_empty());
    }

    #[test]
    fn test_text_transform_default() {
        let transform = TextTransform::default();
        assert_eq!(transform, TextTransform::None);
    }

    #[test]
    fn test_header_style_default() {
        let style = HeaderStyle::default();
        assert!(!style.bold);
        assert!(!style.italic);
        assert!(!style.underline);
        assert_eq!(style.transform, TextTransform::None);
        assert!(style.foreground.is_none());
        assert!(style.background.is_none());
    }
}
