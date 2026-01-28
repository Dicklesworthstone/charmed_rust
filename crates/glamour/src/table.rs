#![doc = include_str!("../docs/tables/README.md")]

use pulldown_cmark::{Alignment, Event, Tag, TagEnd};
use tracing::debug;
use unicode_width::UnicodeWidthChar;

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
/// Convert pulldown-cmark alignment into a lipgloss position string.
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
            let cell_width = measure_width(&cell.content);
            widths[i] = widths[i].max(cell_width);
        }
    }

    // Measure body rows
    for row in &table.rows {
        for (i, cell) in row.iter().enumerate() {
            if i < column_count {
                let cell_width = measure_width(&cell.content);
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

        if config.max_table_width < fixed_overhead {
            debug!(
                target: "glamour::table",
                max_table_width = config.max_table_width,
                fixed_overhead,
                column_count,
                "table structural overhead ({fixed_overhead}) exceeds max_table_width ({max}); \
                 all columns will have zero content width",
                max = config.max_table_width,
            );
        } else if available_content < min_required {
            debug!(
                target: "glamour::table",
                available_content,
                min_required,
                column_count,
                min_width = config.min_width,
                "table content area ({available_content}) is less than minimum required \
                 ({min_required}); columns will be narrower than min_width",
            );
        }

        if available_content >= min_required {
            // Proportionally shrink columns while respecting min_width
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
            // Not enough space for all columns at min_width.
            // Distribute available_content evenly so the total stays
            // within max_table_width as closely as possible.
            let per_column = available_content / column_count.max(1);
            let extra = available_content % column_count.max(1);
            for (i, w) in widths.iter_mut().enumerate() {
                *w = per_column + usize::from(i < extra);
            }
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
    crate::visible_width(s)
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
    let content_width = measure_width(content);

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

/// Fit content to a target width: truncate if too wide, pad if too narrow.
///
/// Unlike [`pad_content`], which returns content unchanged when wider than the
/// target, this function uses [`truncate_content`] to ensure the result never
/// exceeds `width` display units. This is used by the table rendering functions
/// to maintain column alignment even when content is wider than the available
/// column width.
///
/// # Example
///
/// ```rust
/// use glamour::table::fit_content;
/// use pulldown_cmark::Alignment;
///
/// // Content fits — padded normally
/// assert_eq!(fit_content("Hi", 6, Alignment::Left), "Hi    ");
///
/// // Content too wide — truncated with ellipsis
/// assert_eq!(fit_content("Hello, World!", 5, Alignment::Left), "Hell…");
/// ```
#[must_use]
pub fn fit_content(content: &str, width: usize, alignment: Alignment) -> String {
    let content_width = measure_width(content);
    if content_width > width {
        truncate_content(content, width)
    } else {
        pad_content(content, width, alignment)
    }
}

/// Render a cell with proper alignment and optional cell margins.
///
/// This function fits the cell content to the specified column width
/// (truncating with an ellipsis if needed) and adds cell margins on each side.
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
    let fitted = fit_content(&cell.content, col_width, cell.alignment);
    let margin = " ".repeat(cell_margin);
    format!("{}{}{}", margin, fitted, margin)
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
    let fitted = fit_content(content, col_width, alignment);
    let margin = " ".repeat(cell_margin);
    format!("{}{}{}", margin, fitted, margin)
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

    let content_width = measure_width(content);
    if content_width <= max_width {
        return content.to_string();
    }

    // Need to truncate - ellipsis takes 1 unit
    let target_width = max_width.saturating_sub(1);
    let mut result = String::new();
    let mut current_width = 0;
    let mut in_escape = false;
    let mut has_style = false;

    // ANSI-aware truncation: skip escape sequences when counting width
    for c in content.chars() {
        if in_escape {
            result.push(c);
            // CSI sequences end with a letter in 0x40-0x7E range
            if ('@'..='~').contains(&c) {
                in_escape = false;
            }
        } else if c == '\x1b' {
            in_escape = true;
            has_style = true;
            result.push(c);
        } else {
            let char_width = c.width().unwrap_or(0);
            if current_width + char_width > target_width {
                break;
            }
            result.push(c);
            current_width += char_width;
        }
    }

    // Reset styles before ellipsis if we had any styling
    if has_style {
        result.push_str("\x1b[0m");
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

/// Minimal border - only internal separators (matches Go glamour's default).
///
/// This style renders tables without outer edges, showing only:
/// - Vertical column separators between cells
/// - Horizontal separator between header and body
/// - Cross junction at separator intersections
pub const MINIMAL_BORDER: TableBorder = TableBorder {
    top_left: "",
    top_right: "",
    bottom_left: "",
    bottom_right: "",
    horizontal: "─",
    vertical: "│",
    cross: "┼",
    top_t: "",
    bottom_t: "",
    left_t: "",
    right_t: "",
};

/// Minimal ASCII border - only internal separators using ASCII-style characters.
/// Uses Unicode horizontal (─) but ASCII vertical (|) to match Go glamour ASCII style.
pub const MINIMAL_ASCII_BORDER: TableBorder = TableBorder {
    top_left: "",
    top_right: "",
    bottom_left: "",
    bottom_right: "",
    horizontal: "─", // Unicode horizontal to match Go
    vertical: "|",
    cross: "|", // Go uses "|" as cross in ASCII mode
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
        let fitted = fit_content(&cell.content, width, cell.alignment);
        result.push_str(&padding);
        result.push_str(&fitted);
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

/// Render a data row without outer borders (minimal style, matches Go glamour).
///
/// This renders only internal column separators, not outer left/right borders.
///
/// # Arguments
///
/// * `cells` - The cells to render
/// * `widths` - Column content widths (not including padding)
/// * `border` - Border character set (uses vertical for internal separators)
/// * `cell_padding` - Padding on each side of cell content
///
/// # Example
///
/// ```rust
/// use glamour::table::{render_minimal_row, TableCell, MINIMAL_BORDER};
/// use pulldown_cmark::Alignment;
///
/// let cells = vec![
///     TableCell::new("Alice", Alignment::Left),
///     TableCell::new("30", Alignment::Right),
/// ];
/// let widths = vec![5, 3];
/// let result = render_minimal_row(&cells, &widths, &MINIMAL_BORDER, 1);
/// assert_eq!(result, " Alice │  30 ");
/// ```
#[must_use]
pub fn render_minimal_row(
    cells: &[TableCell],
    widths: &[usize],
    border: &TableBorder,
    cell_padding: usize,
) -> String {
    if cells.is_empty() {
        return String::new();
    }

    let padding = " ".repeat(cell_padding);
    let mut parts = Vec::new();

    for (i, cell) in cells.iter().enumerate() {
        let width = widths.get(i).copied().unwrap_or(0);
        let fitted = fit_content(&cell.content, width, cell.alignment);
        parts.push(format!("{}{}{}", padding, fitted, padding));
    }

    // Handle missing cells (if row has fewer cells than widths)
    for width in widths.iter().skip(cells.len()) {
        parts.push(format!("{}{}{}", padding, " ".repeat(*width), padding));
    }

    parts.join(border.vertical)
}

/// Render a horizontal separator without outer edges (minimal style).
///
/// This renders only the internal separator line without left/right corners.
///
/// # Arguments
///
/// * `widths` - Column content widths
/// * `border` - Border character set
/// * `cell_padding` - Padding on each side of cell content
///
/// # Example
///
/// ```rust
/// use glamour::table::{render_minimal_separator, MINIMAL_BORDER};
///
/// let widths = vec![5, 3];
/// let result = render_minimal_separator(&widths, &MINIMAL_BORDER, 1);
/// assert_eq!(result, "───────┼─────");
/// ```
#[must_use]
pub fn render_minimal_separator(
    widths: &[usize],
    border: &TableBorder,
    cell_padding: usize,
) -> String {
    if widths.is_empty() || border.horizontal.is_empty() {
        return String::new();
    }

    let padding_width = cell_padding * 2;
    let mut parts = Vec::new();

    for width in widths {
        parts.push(border.horizontal.repeat(width + padding_width));
    }

    parts.join(border.cross)
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

        let fitted = fit_content(&content, width, cell.alignment);
        let cell_content = format!("{}{}{}", padding, fitted, padding);

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
    fn test_unicode_edge_cases() {
        // Combining characters: e + combining acute accent
        let combining = "e\u{0301}"; // é as two code points
        assert_eq!(measure_width(combining), 1); // Should be 1 display width

        // Precomposed form
        let precomposed = "é"; // Single code point
        assert_eq!(measure_width(precomposed), 1);

        // Zero-width joiner (often in emoji sequences)
        let zwj = "\u{200D}";
        assert_eq!(measure_width(zwj), 0);

        // Regional indicator symbols (flags)
        // Note: Flag emoji width varies by terminal, unicode-width treats as 1 each
        let flag = "🇺🇸"; // U+1F1FA U+1F1F8
        let flag_width = measure_width(flag);
        assert!(flag_width >= 1); // Terminal-dependent, at least 1

        // Variation selectors (shouldn't add width)
        let with_vs = "☀\u{FE0F}"; // sun with variation selector
        let without_vs = "☀";
        // Both should have similar width
        assert!(measure_width(with_vs) <= measure_width(without_vs) + 1);
    }

    #[test]
    fn test_unicode_width_cjk_variants() {
        // Full-width Latin letters
        assert_eq!(measure_width("Ａ"), 2); // Full-width A
        assert_eq!(measure_width("ａ"), 2); // Full-width a

        // Half-width katakana
        assert_eq!(measure_width("ｱ"), 1); // Half-width katakana A

        // Regular katakana (full-width)
        assert_eq!(measure_width("ア"), 2); // Full-width katakana A

        // Korean (Hangul)
        assert_eq!(measure_width("한"), 2);
        assert_eq!(measure_width("한글"), 4);
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

    // ========================================================================
    // Edge cases: extreme table width (bd-15ip)
    // ========================================================================

    fn make_table(cols: usize, rows: usize) -> ParsedTable {
        let header: Vec<TableCell> = (0..cols)
            .map(|i| TableCell::new(format!("Col{i}"), Alignment::Left))
            .collect();
        let body: Vec<Vec<TableCell>> = (0..rows)
            .map(|r| {
                (0..cols)
                    .map(|c| TableCell::new(format!("r{r}c{c}"), Alignment::Left))
                    .collect()
            })
            .collect();
        ParsedTable {
            alignments: vec![Alignment::Left; cols],
            header,
            rows: body,
        }
    }

    #[test]
    fn width_zero_means_no_limit() {
        let table = make_table(3, 2);
        let config = ColumnWidthConfig::default().max_table_width(0);
        let widths = calculate_column_widths(&table, &config);
        // max_table_width=0 means no limit; columns get their natural widths
        assert!(widths.total_width > 0);
        for &w in &widths.widths {
            assert!(w >= config.min_width);
        }
    }

    #[test]
    fn width_smaller_than_overhead_no_panic() {
        // 3 columns, padding=1, border=1
        // overhead = 3*1*2 + (3+1)*1 = 6 + 4 = 10
        let table = make_table(3, 1);
        let config = ColumnWidthConfig::default()
            .max_table_width(5) // less than 10 overhead
            .cell_padding(1)
            .border_width(1);
        let widths = calculate_column_widths(&table, &config);
        // Should not panic; columns may be 0 but that's acceptable degradation
        assert_eq!(widths.widths.len(), 3);
    }

    #[test]
    fn width_exactly_overhead_gives_zero_columns() {
        let table = make_table(3, 1);
        // overhead = 3*1*2 + 4*1 = 10
        let config = ColumnWidthConfig::default()
            .max_table_width(10)
            .cell_padding(1)
            .border_width(1);
        let widths = calculate_column_widths(&table, &config);
        assert_eq!(widths.widths.len(), 3);
        // available_content = 10 - 10 = 0; each column gets 0
        for &w in &widths.widths {
            assert_eq!(w, 0);
        }
    }

    #[test]
    fn width_one_above_overhead_distributes_one_char() {
        let table = make_table(3, 1);
        // overhead = 10, max = 11 → available_content = 1
        let config = ColumnWidthConfig::default()
            .max_table_width(11)
            .cell_padding(1)
            .border_width(1);
        let widths = calculate_column_widths(&table, &config);
        let total_content: usize = widths.widths.iter().sum();
        assert_eq!(total_content, 1);
    }

    #[test]
    fn width_tight_distributes_evenly() {
        let table = make_table(4, 1);
        // overhead = 4*2 + 5 = 13, min_required = 4*3 = 12
        // max = 20 → available = 7, which is < 12
        // Should distribute 7 across 4 cols: 2,2,2,1 (or 1,1,1,1+3 extra)
        let config = ColumnWidthConfig::default()
            .max_table_width(20)
            .cell_padding(1)
            .border_width(1);
        let widths = calculate_column_widths(&table, &config);
        let total_content: usize = widths.widths.iter().sum();
        assert_eq!(total_content, 7);
        // Each column should be at least 1 (7 / 4 = 1 remainder 3)
        for &w in &widths.widths {
            assert!(w >= 1);
        }
    }

    #[test]
    fn width_max_one_no_panic() {
        let table = make_table(2, 1);
        let config = ColumnWidthConfig::default().max_table_width(1);
        let widths = calculate_column_widths(&table, &config);
        assert_eq!(widths.widths.len(), 2);
    }

    #[test]
    fn single_column_narrow_width() {
        let table = make_table(1, 1);
        // overhead = 1*2 + 2 = 4, max = 6 → available = 2
        let config = ColumnWidthConfig::default()
            .max_table_width(6)
            .cell_padding(1)
            .border_width(1);
        let widths = calculate_column_widths(&table, &config);
        assert_eq!(widths.widths.len(), 1);
        assert_eq!(widths.widths[0], 2);
        assert_eq!(widths.total_width, 6);
    }

    #[test]
    fn many_columns_narrow_width() {
        let table = make_table(10, 1);
        // overhead = 10*2 + 11 = 31, max = 15 → available = 0
        let config = ColumnWidthConfig::default()
            .max_table_width(15)
            .cell_padding(1)
            .border_width(1);
        let widths = calculate_column_widths(&table, &config);
        assert_eq!(widths.widths.len(), 10);
        let total_content: usize = widths.widths.iter().sum();
        assert_eq!(total_content, 0);
    }

    #[test]
    fn render_table_with_narrow_width_no_panic() {
        let table = make_table(3, 2);
        let config = ColumnWidthConfig::default()
            .max_table_width(5)
            .cell_padding(1)
            .border_width(1);
        let _col_widths = calculate_column_widths(&table, &config);
        // Render the full table — should not panic
        let render_config = TableRenderConfig::default();
        let rendered = render_table(&table, &render_config);
        assert!(!rendered.is_empty());
    }

    #[test]
    fn render_data_row_zero_width_columns() {
        let cells = vec![
            TableCell::new("Hello", Alignment::Left),
            TableCell::new("World", Alignment::Left),
        ];
        let widths = vec![0, 0];
        // Should not panic; cells will just show full content (pad_content returns as-is)
        let row = render_data_row(&cells, &widths, &ROUNDED_BORDER, 1);
        assert!(!row.is_empty());
    }

    #[test]
    fn truncate_content_width_zero() {
        assert_eq!(truncate_content("Hello", 0), "");
    }

    #[test]
    fn truncate_content_width_one() {
        // Width 1 means ellipsis only (since ellipsis is 1 unit)
        let result = truncate_content("Hello", 1);
        assert_eq!(result, "…");
    }

    #[test]
    fn pad_content_width_zero() {
        // Width 0: content_width >= width, returns content as-is
        let result = pad_content("Hi", 0, Alignment::Left);
        assert_eq!(result, "Hi");
    }

    #[test]
    fn horizontal_border_zero_width_columns() {
        let widths = vec![0, 0, 0];
        let result = render_horizontal_border(&widths, &ROUNDED_BORDER, BorderPosition::Top, 1);
        // Should not panic; renders border structure with just padding
        assert!(!result.is_empty());
    }

    #[test]
    fn config_min_width_zero() {
        let table = make_table(3, 1);
        let config = ColumnWidthConfig::default()
            .min_width(0)
            .max_table_width(50);
        let widths = calculate_column_widths(&table, &config);
        assert_eq!(widths.widths.len(), 3);
        // All should work without panic
        assert!(widths.total_width > 0);
    }

    #[test]
    fn config_all_zeros() {
        let table = make_table(2, 1);
        let config = ColumnWidthConfig::default()
            .min_width(0)
            .max_table_width(0) // no limit
            .cell_padding(0)
            .border_width(0);
        let widths = calculate_column_widths(&table, &config);
        assert_eq!(widths.widths.len(), 2);
        // No padding, no borders, just content
        let total_content: usize = widths.widths.iter().sum();
        assert_eq!(widths.total_width, total_content);
    }

    #[test]
    fn tight_width_total_stays_within_max() {
        // Verify the fix: when available_content < min_required, total should
        // stay within max_table_width (minus any unavoidable structural overshoot)
        let table = make_table(3, 1);
        let max = 20;
        let config = ColumnWidthConfig::default()
            .max_table_width(max)
            .cell_padding(1)
            .border_width(1);
        let widths = calculate_column_widths(&table, &config);
        // total_width should be at most max_table_width
        assert!(
            widths.total_width <= max,
            "total_width {} exceeds max {}",
            widths.total_width,
            max
        );
    }

    #[test]
    fn extreme_overhead_total_stays_within_max() {
        // max_table_width < overhead; total_width should still be based on
        // actual column widths (which will be 0), so it equals just overhead.
        let table = make_table(3, 1);
        let config = ColumnWidthConfig::default()
            .max_table_width(5)
            .cell_padding(1)
            .border_width(1);
        let widths = calculate_column_widths(&table, &config);
        // overhead = 10, columns = 0 each, so total = 10.
        // This still exceeds 5, but there's nothing we can do about structural
        // overhead. The key point is columns don't inflate the total further.
        let total_content: usize = widths.widths.iter().sum();
        assert_eq!(total_content, 0);
        // total_width = overhead + 0 content
        let expected_overhead = 3 * 2 + 4; // 10
        assert_eq!(widths.total_width, expected_overhead);
    }

    // === Edge Case Tests for bd-15ip ===

    #[test]
    fn fit_content_truncates_wide_content() {
        let result = fit_content("Hello, World!", 5, Alignment::Left);
        assert_eq!(result, "Hell…");
        assert!(measure_width(&result) <= 5);
    }

    #[test]
    fn fit_content_pads_narrow_content() {
        let result = fit_content("Hi", 6, Alignment::Left);
        assert_eq!(result, "Hi    ");
        assert_eq!(measure_width(&result), 6);
    }

    #[test]
    fn fit_content_exact_width() {
        let result = fit_content("Hello", 5, Alignment::Left);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn fit_content_zero_width() {
        let result = fit_content("Hello", 0, Alignment::Left);
        assert_eq!(result, "");
    }

    #[test]
    fn fit_content_one_width_truncates_to_ellipsis() {
        let result = fit_content("Hello", 1, Alignment::Left);
        assert_eq!(result, "…");
    }

    #[test]
    fn fit_content_unicode_cjk() {
        // CJK chars are 2 units wide; "日本語" = 6 units
        let result = fit_content("日本語", 4, Alignment::Left);
        assert_eq!(result, "日…");
        assert!(measure_width(&result) <= 4);
    }

    #[test]
    fn render_cell_truncates_overflow() {
        // Column width 3, content "Hello" (5 wide)
        let cell = TableCell::new("Hello", Alignment::Left);
        let rendered = render_cell(&cell, 3, 1);
        // Should be: margin + "He…" + margin = " He… "
        assert_eq!(rendered, " He… ");
    }

    #[test]
    fn render_data_row_truncates_in_narrow_columns() {
        let cells = vec![
            TableCell::new("LongContent", Alignment::Left),
            TableCell::new("Also Long", Alignment::Left),
        ];
        let widths = vec![4, 4];
        let row = render_data_row(&cells, &widths, &ROUNDED_BORDER, 1);
        // Content should be truncated to fit 4-width columns
        assert!(row.contains("Lon…"));
        assert!(row.contains("Als…"));
    }

    #[test]
    fn render_table_extreme_narrow_no_panic() {
        let table = make_table(5, 3);
        // max_table_width=1 is smaller than any possible overhead
        let config = ColumnWidthConfig::default()
            .max_table_width(1)
            .cell_padding(1)
            .border_width(1);
        let widths = calculate_column_widths(&table, &config);
        // All columns should be 0 (overhead alone > 1)
        assert!(widths.widths.iter().all(|&w| w == 0));

        // Full render should not panic
        let render_config = TableRenderConfig::default();
        let rendered = render_table(&table, &render_config);
        assert!(!rendered.is_empty());
    }

    #[test]
    fn render_table_max_width_zero_no_limit() {
        let table = make_table(2, 1);
        let config = ColumnWidthConfig::default().max_table_width(0); // 0 = no limit
        let widths = calculate_column_widths(&table, &config);
        // Should use natural content widths
        assert!(widths.widths.iter().all(|&w| w >= 3));
    }

    #[test]
    fn render_single_column_narrow_truncates() {
        let header = vec![TableCell::new("Name", Alignment::Left)];
        let row = vec![TableCell::new(
            "VeryLongNameThatShouldBeTruncated",
            Alignment::Left,
        )];

        // Use constrained widths: column width 6
        let widths = vec![6];
        let header_rendered = render_data_row(&header, &widths, &ROUNDED_BORDER, 1);
        let row_rendered = render_data_row(&row, &widths, &ROUNDED_BORDER, 1);

        // "Name" (4 wide) fits in 6 — should be padded
        assert!(header_rendered.contains("Name"));
        // "VeryLongNameThatShouldBeTruncated" should be truncated to 6 chars
        assert!(!row_rendered.contains("VeryLongNameThatShouldBeTruncated"));
        assert!(row_rendered.contains("…"));
    }

    #[test]
    fn many_columns_extreme_narrow_no_panic() {
        let table = make_table(20, 2);
        let config = ColumnWidthConfig::default()
            .max_table_width(10)
            .cell_padding(0)
            .border_width(0);
        let widths = calculate_column_widths(&table, &config);
        assert_eq!(widths.widths.len(), 20);
        // Available = 10, per_column = 10/20 = 0, extra = 10
        // First 10 columns get 1, rest get 0
        let nonzero: usize = widths.widths.iter().filter(|&&w| w > 0).count();
        assert_eq!(nonzero, 10);
        let zero: usize = widths.widths.iter().filter(|&&w| w == 0).count();
        assert_eq!(zero, 10);
    }

    #[test]
    fn render_minimal_row_truncates_overflow() {
        let cells = vec![
            TableCell::new("Hello", Alignment::Left),
            TableCell::new("World", Alignment::Left),
        ];
        let widths = vec![3, 3];
        let row = render_minimal_row(&cells, &widths, &MINIMAL_BORDER, 1);
        assert!(row.contains("He…"));
        assert!(row.contains("Wo…"));
    }

    #[test]
    fn render_header_row_truncates_overflow() {
        let cells = vec![TableCell::new("LongHeader", Alignment::Left)];
        let widths = vec![4];
        let row = render_header_row(&cells, &widths, &ROUNDED_BORDER, 1, None);
        // "LongHeader" (10 wide) in 4-wide column should truncate to "Lon…"
        assert!(row.contains("Lon…"));
    }

    #[test]
    fn max_table_width_equals_overhead_exactly() {
        let table = make_table(2, 1);
        // overhead = 2*2*1 + 3*1 = 7 for 2 columns, padding=1, border=1
        let config = ColumnWidthConfig::default()
            .max_table_width(7)
            .cell_padding(1)
            .border_width(1);
        let widths = calculate_column_widths(&table, &config);
        // available_content = 7 - 7 = 0, all columns 0
        assert!(widths.widths.iter().all(|&w| w == 0));
        assert_eq!(widths.total_width, 7);
    }

    #[test]
    fn max_table_width_one_more_than_overhead() {
        let table = make_table(2, 1);
        // overhead = 7 for 2 columns, padding=1, border=1
        let config = ColumnWidthConfig::default()
            .max_table_width(8)
            .cell_padding(1)
            .border_width(1);
        let widths = calculate_column_widths(&table, &config);
        // available_content = 1, per_column = 0, extra = 1
        // First column gets 1, second gets 0
        assert_eq!(widths.widths[0], 1);
        assert_eq!(widths.widths[1], 0);
    }
}
