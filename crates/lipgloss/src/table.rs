//! Table rendering.
//!
//! Port of Go lipgloss's `table` subpackage: a declarative table with optional
//! headers, per-cell styling through a style function, configurable borders,
//! fixed total width (columns shrink or grow to fit), height limiting with an
//! overflow row, and row offsets for scrolling.
//!
//! # Example
//!
//! ```rust
//! use lipgloss::Border;
//! use lipgloss::table::Table;
//!
//! let t = Table::new()
//!     .border(Border::normal())
//!     .headers(["Name", "Lang"])
//!     .row(["glow", "Go"])
//!     .row(["charmed", "Rust"]);
//!
//! assert_eq!(
//!     t.to_string(),
//!     "┌───────┬────┐\n\
//!      │Name   │Lang│\n\
//!      ├───────┼────┤\n\
//!      │glow   │Go  │\n\
//!      │charmed│Rust│\n\
//!      └───────┴────┘"
//! );
//! ```

use std::fmt;
use std::sync::Arc;

use crate::border::Border;
use crate::style::{Style, truncate_line_ansi};
use crate::{Position, height, join_horizontal, visible_width};

/// Row index passed to the style function for header cells.
pub const HEADER_ROW: isize = -1;

/// Style function: `(row, column) -> Style`. Header cells use [`HEADER_ROW`].
pub type StyleFunc = Arc<dyn Fn(isize, usize) -> Style + Send + Sync>;

/// A source of table cells.
pub trait Data: Send + Sync {
    /// Returns the cell at `row`, `column` (empty if out of range).
    fn at(&self, row: usize, column: usize) -> String;
    /// Returns the number of rows.
    fn rows(&self) -> usize;
    /// Returns the number of columns.
    fn columns(&self) -> usize;
}

/// In-memory string table data.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StringData {
    rows: Vec<Vec<String>>,
    columns: usize,
}

impl StringData {
    /// Creates data from rows of cells.
    pub fn new<R, C, S>(rows: R) -> Self
    where
        R: IntoIterator<Item = C>,
        C: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut data = Self::default();
        for row in rows {
            data.append(row);
        }
        data
    }

    /// Appends a row.
    pub fn append<C, S>(&mut self, row: C)
    where
        C: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let row: Vec<String> = row.into_iter().map(Into::into).collect();
        self.columns = self.columns.max(row.len());
        self.rows.push(row);
    }
}

impl Data for StringData {
    fn at(&self, row: usize, column: usize) -> String {
        self.rows
            .get(row)
            .and_then(|r| r.get(column))
            .cloned()
            .unwrap_or_default()
    }

    fn rows(&self) -> usize {
        self.rows.len()
    }

    fn columns(&self) -> usize {
        self.columns
    }
}

/// Data wrapper that only exposes rows matching a predicate.
pub struct Filter<D: Data> {
    data: D,
    keep: Vec<usize>,
}

impl<D: Data> Filter<D> {
    /// Filters `data`, keeping rows for which `predicate(row)` is true.
    pub fn new(data: D, predicate: impl Fn(usize) -> bool) -> Self {
        let keep = (0..data.rows()).filter(|&r| predicate(r)).collect();
        Self { data, keep }
    }
}

impl<D: Data> Data for Filter<D> {
    fn at(&self, row: usize, column: usize) -> String {
        self.keep
            .get(row)
            .map(|&r| self.data.at(r, column))
            .unwrap_or_default()
    }

    fn rows(&self) -> usize {
        self.keep.len()
    }

    fn columns(&self) -> usize {
        self.data.columns()
    }
}

/// Default style function: unstyled cells.
pub fn default_styles(_row: isize, _col: usize) -> Style {
    Style::new()
}

/// A renderable table.
#[derive(Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct Table {
    headers: Vec<String>,
    data: Arc<dyn Data>,
    border: Border,
    border_style: Style,
    border_top: bool,
    border_bottom: bool,
    border_left: bool,
    border_right: bool,
    border_header: bool,
    border_column: bool,
    border_row: bool,
    style: StyleFunc,
    width: usize,
    height: usize,
    offset: usize,
    wrap: bool,
}

impl fmt::Debug for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Table")
            .field("headers", &self.headers)
            .field("rows", &self.data.rows())
            .field("width", &self.width)
            .field("height", &self.height)
            .field("offset", &self.offset)
            .finish_non_exhaustive()
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl Table {
    /// Creates an empty table with rounded borders.
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            data: Arc::new(StringData::default()),
            border: Border::rounded(),
            border_style: Style::new(),
            border_top: true,
            border_bottom: true,
            border_left: true,
            border_right: true,
            border_header: true,
            border_column: true,
            border_row: false,
            style: Arc::new(default_styles),
            width: 0,
            height: 0,
            offset: 0,
            wrap: true,
        }
    }

    /// Sets the header row.
    #[must_use]
    pub fn headers<I, S>(mut self, headers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.headers = headers.into_iter().map(Into::into).collect();
        self
    }

    /// Appends a data row. Replaces custom [`Data`] with string data that
    /// keeps the existing rows.
    #[must_use]
    pub fn row<I, S>(mut self, row: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut data = self.string_data();
        data.append(row);
        self.data = Arc::new(data);
        self
    }

    /// Appends several data rows.
    #[must_use]
    pub fn rows<R, C, S>(mut self, rows: R) -> Self
    where
        R: IntoIterator<Item = C>,
        C: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut data = self.string_data();
        for row in rows {
            data.append(row);
        }
        self.data = Arc::new(data);
        self
    }

    fn string_data(&self) -> StringData {
        let d = &self.data;
        StringData::new((0..d.rows()).map(|r| (0..d.columns()).map(move |c| d.at(r, c))))
    }

    /// Removes all data rows (headers are kept).
    #[must_use]
    pub fn clear_rows(mut self) -> Self {
        self.data = Arc::new(StringData::default());
        self
    }

    /// Uses a custom data source.
    #[must_use]
    pub fn data(mut self, data: impl Data + 'static) -> Self {
        self.data = Arc::new(data);
        self
    }

    /// Sets the border characters.
    #[must_use]
    pub fn border(mut self, border: Border) -> Self {
        self.border = border;
        self
    }

    /// Sets the style used to render borders.
    #[must_use]
    pub fn border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }

    /// Toggles the top border.
    #[must_use]
    pub fn border_top(mut self, v: bool) -> Self {
        self.border_top = v;
        self
    }

    /// Toggles the bottom border.
    #[must_use]
    pub fn border_bottom(mut self, v: bool) -> Self {
        self.border_bottom = v;
        self
    }

    /// Toggles the left border.
    #[must_use]
    pub fn border_left(mut self, v: bool) -> Self {
        self.border_left = v;
        self
    }

    /// Toggles the right border.
    #[must_use]
    pub fn border_right(mut self, v: bool) -> Self {
        self.border_right = v;
        self
    }

    /// Toggles the separator below the header row.
    #[must_use]
    pub fn border_header(mut self, v: bool) -> Self {
        self.border_header = v;
        self
    }

    /// Toggles separators between columns.
    #[must_use]
    pub fn border_column(mut self, v: bool) -> Self {
        self.border_column = v;
        self
    }

    /// Toggles separators between data rows.
    #[must_use]
    pub fn border_row(mut self, v: bool) -> Self {
        self.border_row = v;
        self
    }

    /// Sets the per-cell style function. Header cells receive [`HEADER_ROW`].
    #[must_use]
    pub fn style_func<F>(mut self, f: F) -> Self
    where
        F: Fn(isize, usize) -> Style + Send + Sync + 'static,
    {
        self.style = Arc::new(f);
        self
    }

    /// Sets the total table width (0 = natural width). Columns shrink or grow
    /// to fit.
    #[must_use]
    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Sets the total table height (0 = unlimited). Rows that do not fit are
    /// replaced by an overflow row of `…`.
    #[must_use]
    pub fn height(mut self, height: usize) -> Self {
        self.height = height;
        self
    }

    /// Skips the first `offset` data rows.
    #[must_use]
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    /// Enables (default) or disables wrapping; without wrapping, overlong
    /// cells are truncated with `…`.
    #[must_use]
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    fn cell_style(&self, row: isize, col: usize) -> Style {
        (self.style)(row, col)
    }

    fn column_count(&self) -> usize {
        self.headers.len().max(self.data.columns())
    }

    fn header_cell(&self, col: usize) -> String {
        self.headers.get(col).cloned().unwrap_or_default()
    }

    /// Computes natural column widths and adjusts them to the target width.
    fn column_widths(&self, ncols: usize) -> Vec<usize> {
        let mut widths = vec![0usize; ncols];
        if !self.headers.is_empty() {
            for (c, w) in widths.iter_mut().enumerate() {
                let rendered = self.cell_style(HEADER_ROW, c).render(&self.header_cell(c));
                *w = (*w).max(visible_width_block(&rendered));
            }
        }
        for r in 0..self.data.rows() {
            for (c, w) in widths.iter_mut().enumerate() {
                let rendered = self.cell_style(as_row(r), c).render(&self.data.at(r, c));
                *w = (*w).max(visible_width_block(&rendered));
            }
        }

        if self.width == 0 {
            return widths;
        }
        let frame = usize::from(self.border_left)
            + usize::from(self.border_right)
            + if self.border_column {
                ncols.saturating_sub(1)
            } else {
                0
            };
        let target = self.width.saturating_sub(frame);
        let mut total: usize = widths.iter().sum();
        while total < target {
            let (i, _) = widths
                .iter()
                .enumerate()
                .min_by_key(|&(_, w)| *w)
                .expect("ncols > 0");
            widths[i] += 1;
            total += 1;
        }
        while total > target {
            let (i, &w) = widths
                .iter()
                .enumerate()
                .rev()
                .max_by_key(|&(_, w)| *w)
                .expect("ncols > 0");
            if w <= 1 {
                break;
            }
            widths[i] -= 1;
            total -= 1;
        }
        widths
    }

    fn render_cell(&self, row: isize, col: usize, text: &str, width: usize, h: usize) -> String {
        let style = self.cell_style(row, col);
        let text = if self.wrap {
            text.to_string()
        } else {
            let inner = width.saturating_sub(style.get_horizontal_padding());
            text.lines()
                .map(|l| truncate_with_tail(l, inner))
                .collect::<Vec<_>>()
                .join("\n")
        };
        let w = clamp_u16(width.saturating_sub(style.get_horizontal_margin()));
        let hh = clamp_u16(h.saturating_sub(style.get_vertical_margin()));
        let rendered = style
            .width(w)
            .max_width(clamp_u16(width))
            .height(hh)
            .max_height(clamp_u16(h))
            .render(&text);
        // Guarantee a fixed-width block even if the style wraps oddly.
        rendered
            .lines()
            .map(|l| {
                let lw = visible_width(l);
                if lw > width {
                    truncate_line_ansi(l, width)
                } else {
                    format!("{l}{}", " ".repeat(width - lw))
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn row_height(&self, row: isize, cells: &[String], widths: &[usize]) -> usize {
        cells
            .iter()
            .zip(widths)
            .enumerate()
            .map(|(c, (text, &w))| {
                let style = self.cell_style(row, c).width(clamp_u16(w));
                if self.wrap {
                    height(&style.render(text))
                } else {
                    height(&style.render(text)).min(text.lines().count().max(1))
                }
            })
            .max()
            .unwrap_or(1)
            .max(1)
    }

    fn construct_row(&self, row: isize, cells: &[String], widths: &[usize], h: usize) -> String {
        let sep_line = |s: &str| {
            let one = self.border_style.render(s);
            vec![one; h].join("\n")
        };
        let mut parts: Vec<String> = Vec::with_capacity(widths.len() * 2 + 1);
        if self.border_left {
            parts.push(sep_line(&self.border.left));
        }
        for (c, &w) in widths.iter().enumerate() {
            if c > 0 && self.border_column {
                parts.push(sep_line(&self.border.left));
            }
            parts.push(self.render_cell(row, c, &cells[c], w, h));
        }
        if self.border_right {
            parts.push(sep_line(&self.border.right));
        }
        let refs: Vec<&str> = parts.iter().map(String::as_str).collect();
        join_horizontal(Position::Top, &refs)
    }

    fn border_line(
        &self,
        widths: &[usize],
        left: &str,
        fill: &str,
        mid: &str,
        right: &str,
    ) -> String {
        let mut s = String::new();
        if self.border_left {
            s.push_str(left);
        }
        for (c, &w) in widths.iter().enumerate() {
            if c > 0 && self.border_column {
                s.push_str(mid);
            }
            s.push_str(&fill.repeat(w));
        }
        if self.border_right {
            s.push_str(right);
        }
        self.border_style.render(&s)
    }

    /// Renders the table.
    pub fn render(&self) -> String {
        let ncols = self.column_count();
        if ncols == 0 {
            return String::new();
        }
        let widths = self.column_widths(ncols);
        let b = &self.border;

        let top = self.border_line(&widths, &b.top_left, &b.top, &b.middle_top, &b.top_right);
        let separator =
            self.border_line(&widths, &b.middle_left, &b.top, &b.middle, &b.middle_right);
        let bottom = self.border_line(
            &widths,
            &b.bottom_left,
            &b.bottom,
            &b.middle_bottom,
            &b.bottom_right,
        );

        let mut out: Vec<String> = Vec::new();
        if self.border_top && visible_width(&top) > 0 {
            out.push(top);
        }
        if !self.headers.is_empty() {
            let cells: Vec<String> = (0..ncols).map(|c| self.header_cell(c)).collect();
            let h = self.row_height(HEADER_ROW, &cells, &widths);
            out.push(self.construct_row(HEADER_ROW, &cells, &widths, h));
            if self.border_header && visible_width(&separator) > 0 {
                out.push(separator.clone());
            }
        }

        let bottom_lines = usize::from(self.border_bottom && visible_width(&bottom) > 0);
        let used = |out: &Vec<String>| out.iter().map(|s| height(s)).sum::<usize>();

        let total_rows = self.data.rows();
        let mut r = self.offset.min(total_rows);
        while r < total_rows {
            let cells: Vec<String> = (0..ncols).map(|c| self.data.at(r, c)).collect();
            let h = self.row_height(as_row(r), &cells, &widths);
            let row_sep = r > self.offset && self.border_row && visible_width(&separator) > 0;
            let needed = h + usize::from(row_sep);

            if self.height > 0 {
                let remaining_after = total_rows - r - 1;
                // Reserve one line for the overflow row unless this is the last row.
                let reserve = usize::from(remaining_after > 0);
                if used(&out) + needed + reserve + bottom_lines > self.height {
                    if used(&out) + usize::from(row_sep) + 1 + bottom_lines <= self.height {
                        if row_sep {
                            out.push(separator);
                        }
                        let overflow: Vec<String> = vec!["…".to_string(); ncols];
                        out.push(self.construct_row(as_row(r), &overflow, &widths, 1));
                    }
                    break;
                }
            }

            if row_sep {
                out.push(separator.clone());
            }
            out.push(self.construct_row(as_row(r), &cells, &widths, h));
            r += 1;
        }

        if bottom_lines > 0 {
            out.push(bottom);
        }
        out.join("\n")
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render())
    }
}

fn as_row(r: usize) -> isize {
    isize::try_from(r).unwrap_or(isize::MAX)
}

fn clamp_u16(n: usize) -> u16 {
    u16::try_from(n).unwrap_or(u16::MAX)
}

fn visible_width_block(s: &str) -> usize {
    s.lines().map(visible_width).max().unwrap_or(0)
}

/// Truncates `s` to `width` cells, ending with `…` when shortened.
fn truncate_with_tail(s: &str, width: usize) -> String {
    if visible_width(s) <= width {
        return s.to_string();
    }
    if width == 0 {
        return String::new();
    }
    format!("{}…", truncate_line_ansi(s, width - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn langs() -> Table {
        Table::new()
            .border(Border::normal())
            .headers(["LANGUAGE", "FORMAL", "INFORMAL"])
            .row(["Chinese", "Nǐn hǎo", "Nǐ hǎo"])
            .row(["French", "Bonjour", "Salut"])
            .row(["Japanese", "こんにちは", "やあ"])
    }

    #[test]
    fn basic_table() {
        let want = "\
┌────────┬──────────┬────────┐
│LANGUAGE│FORMAL    │INFORMAL│
├────────┼──────────┼────────┤
│Chinese │Nǐn hǎo   │Nǐ hǎo  │
│French  │Bonjour   │Salut   │
│Japanese│こんにちは│やあ    │
└────────┴──────────┴────────┘";
        assert_eq!(langs().to_string(), want);
    }

    #[test]
    fn padded_cells_via_style_func() {
        let t = Table::new()
            .border(Border::normal())
            .style_func(|_, _| Style::new().padding((0, 1)))
            .headers(["A", "B"])
            .row(["1", "22"]);
        let want = "\
┌───┬────┐
│ A │ B  │
├───┼────┤
│ 1 │ 22 │
└───┴────┘";
        assert_eq!(t.to_string(), want);
    }

    #[test]
    fn default_border_is_rounded() {
        let t = Table::new().row(["x"]);
        assert_eq!(t.to_string(), "╭─╮\n│x│\n╰─╯");
    }

    #[test]
    fn empty_table() {
        assert_eq!(Table::new().to_string(), "");
    }

    #[test]
    fn headers_only() {
        let t = Table::new().border(Border::normal()).headers(["a", "b"]);
        assert_eq!(t.to_string(), "┌─┬─┐\n│a│b│\n├─┼─┤\n└─┴─┘");
    }

    #[test]
    fn row_separators() {
        let t = Table::new()
            .border(Border::normal())
            .border_row(true)
            .rows([["a"], ["b"]]);
        assert_eq!(t.to_string(), "┌─┐\n│a│\n├─┤\n│b│\n└─┘");
    }

    #[test]
    fn no_outer_borders() {
        let t = Table::new()
            .border(Border::normal())
            .border_top(false)
            .border_bottom(false)
            .border_left(false)
            .border_right(false)
            .headers(["a", "b"])
            .row(["1", "2"]);
        assert_eq!(t.to_string(), "a│b\n─┼─\n1│2");
    }

    #[test]
    fn no_column_borders() {
        let t = Table::new()
            .border(Border::normal())
            .border_column(false)
            .row(["a", "b"]);
        assert_eq!(t.to_string(), "┌──┐\n│ab│\n└──┘");
    }

    #[test]
    fn ragged_rows_are_padded() {
        let t = Table::new()
            .border(Border::normal())
            .row(["a", "b"])
            .row(["c"]);
        assert_eq!(t.to_string(), "┌─┬─┐\n│a│b│\n│c│ │\n└─┴─┘");
    }

    #[test]
    fn width_expands_columns() {
        let t = Table::new()
            .border(Border::normal())
            .row(["a", "b"])
            .width(9);
        let out = t.to_string();
        for line in out.lines() {
            assert_eq!(visible_width(line), 9, "line {line:?}");
        }
    }

    #[test]
    fn width_shrinks_and_wraps() {
        let t = Table::new()
            .border(Border::normal())
            .row(["hello world", "x"])
            .width(10);
        let out = t.to_string();
        for line in out.lines() {
            assert_eq!(visible_width(line), 10, "line {line:?}");
        }
        assert!(out.lines().count() > 3, "long cell should wrap: {out}");
    }

    #[test]
    fn no_wrap_truncates_with_ellipsis() {
        let t = Table::new()
            .border(Border::normal())
            .wrap(false)
            .row(["hello world", "x"])
            .width(10);
        let out = t.to_string();
        assert_eq!(out.lines().count(), 3, "{out}");
        assert!(out.contains('…'), "{out}");
        for line in out.lines() {
            assert_eq!(visible_width(line), 10, "line {line:?}");
        }
    }

    #[test]
    fn offset_skips_rows() {
        let t = Table::new()
            .border(Border::normal())
            .rows([["a"], ["b"], ["c"]])
            .offset(1);
        assert_eq!(t.to_string(), "┌─┐\n│b│\n│c│\n└─┘");
    }

    #[test]
    fn height_adds_overflow_row() {
        let t = Table::new()
            .border(Border::normal())
            .rows([["a"], ["b"], ["c"], ["d"]])
            .height(5);
        assert_eq!(t.to_string(), "┌─┐\n│a│\n│b│\n│…│\n└─┘");
    }

    #[test]
    fn height_exact_fit_has_no_overflow() {
        let t = Table::new()
            .border(Border::normal())
            .rows([["a"], ["b"]])
            .height(4);
        assert_eq!(t.to_string(), "┌─┐\n│a│\n│b│\n└─┘");
    }

    #[test]
    fn style_func_receives_header_row() {
        let t = Table::new()
            .border(Border::normal())
            .style_func(|row, _| {
                if row == HEADER_ROW {
                    Style::new().padding_left(1)
                } else {
                    Style::new()
                }
            })
            .headers(["h"])
            .row(["v"]);
        assert_eq!(t.to_string(), "┌──┐\n│ h│\n├──┤\n│v │\n└──┘");
    }

    #[test]
    fn multiline_cells() {
        let t = Table::new().border(Border::normal()).row(["a\nb", "c"]);
        assert_eq!(t.to_string(), "┌─┬─┐\n│a│c│\n│b│ │\n└─┴─┘");
    }

    #[test]
    fn filter_data() {
        let data = StringData::new([["keep"], ["drop"], ["keep2"]]);
        let t = Table::new()
            .border(Border::normal())
            .data(Filter::new(data, |r| r != 1));
        assert_eq!(t.to_string(), "┌─────┐\n│keep │\n│keep2│\n└─────┘");
    }

    #[test]
    fn row_after_custom_data_keeps_rows() {
        let t = Table::new()
            .border(Border::normal())
            .data(StringData::new([["a"]]))
            .row(["b"]);
        assert_eq!(t.to_string(), "┌─┐\n│a│\n│b│\n└─┘");
    }

    #[test]
    fn clear_rows_keeps_headers() {
        let t = Table::new()
            .border(Border::normal())
            .headers(["h"])
            .row(["x"])
            .clear_rows();
        assert_eq!(t.to_string(), "┌─┐\n│h│\n├─┤\n└─┘");
    }

    #[test]
    fn truncate_tail_helper() {
        assert_eq!(truncate_with_tail("hello", 10), "hello");
        assert_eq!(truncate_with_tail("hello", 3), "he…");
        assert_eq!(truncate_with_tail("hello", 0), "");
    }
}
