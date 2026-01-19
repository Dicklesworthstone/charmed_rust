//! Multi-line text area component.
//!
//! This module provides a multi-line text editor for TUI applications with
//! features like line numbers, word wrapping, and viewport scrolling.
//!
//! # Example
//!
//! ```rust
//! use bubbles::textarea::TextArea;
//!
//! let mut textarea = TextArea::new();
//! textarea.set_value("Line 1\nLine 2\nLine 3");
//!
//! // Render the textarea
//! let view = textarea.view();
//! ```

use crate::cursor::{Cursor, Mode as CursorMode, blink_cmd};
use crate::key::{Binding, matches};
use crate::runeutil::Sanitizer;
use crate::viewport::Viewport;
use bubbletea::{Cmd, KeyMsg, Message, Model};
use lipgloss::{Color, Style};

const MIN_HEIGHT: usize = 1;
const DEFAULT_HEIGHT: usize = 6;
const DEFAULT_WIDTH: usize = 40;
const DEFAULT_MAX_HEIGHT: usize = 99;
const DEFAULT_MAX_WIDTH: usize = 500;
const MAX_LINES: usize = 10000;

/// Key bindings for textarea navigation.
#[derive(Debug, Clone)]
pub struct KeyMap {
    /// Move character forward.
    pub character_forward: Binding,
    /// Move character backward.
    pub character_backward: Binding,
    /// Delete text after cursor.
    pub delete_after_cursor: Binding,
    /// Delete text before cursor.
    pub delete_before_cursor: Binding,
    /// Delete character backward.
    pub delete_character_backward: Binding,
    /// Delete character forward.
    pub delete_character_forward: Binding,
    /// Delete word backward.
    pub delete_word_backward: Binding,
    /// Delete word forward.
    pub delete_word_forward: Binding,
    /// Insert newline.
    pub insert_newline: Binding,
    /// Move to line end.
    pub line_end: Binding,
    /// Move to next line.
    pub line_next: Binding,
    /// Move to previous line.
    pub line_previous: Binding,
    /// Move to line start.
    pub line_start: Binding,
    /// Paste from clipboard.
    pub paste: Binding,
    /// Move word backward.
    pub word_backward: Binding,
    /// Move word forward.
    pub word_forward: Binding,
    /// Move to input begin.
    pub input_begin: Binding,
    /// Move to input end.
    pub input_end: Binding,
    /// Uppercase word forward.
    pub uppercase_word_forward: Binding,
    /// Lowercase word forward.
    pub lowercase_word_forward: Binding,
    /// Capitalize word forward.
    pub capitalize_word_forward: Binding,
    /// Transpose character backward.
    pub transpose_character_backward: Binding,
}

impl Default for KeyMap {
    fn default() -> Self {
        Self {
            character_forward: Binding::new()
                .keys(&["right", "ctrl+f"])
                .help("right", "character forward"),
            character_backward: Binding::new()
                .keys(&["left", "ctrl+b"])
                .help("left", "character backward"),
            word_forward: Binding::new()
                .keys(&["alt+right", "alt+f"])
                .help("alt+right", "word forward"),
            word_backward: Binding::new()
                .keys(&["alt+left", "alt+b"])
                .help("alt+left", "word backward"),
            line_next: Binding::new()
                .keys(&["down", "ctrl+n"])
                .help("down", "next line"),
            line_previous: Binding::new()
                .keys(&["up", "ctrl+p"])
                .help("up", "previous line"),
            delete_word_backward: Binding::new()
                .keys(&["alt+backspace", "ctrl+w"])
                .help("alt+backspace", "delete word backward"),
            delete_word_forward: Binding::new()
                .keys(&["alt+delete", "alt+d"])
                .help("alt+delete", "delete word forward"),
            delete_after_cursor: Binding::new()
                .keys(&["ctrl+k"])
                .help("ctrl+k", "delete after cursor"),
            delete_before_cursor: Binding::new()
                .keys(&["ctrl+u"])
                .help("ctrl+u", "delete before cursor"),
            insert_newline: Binding::new()
                .keys(&["enter", "ctrl+m"])
                .help("enter", "insert newline"),
            delete_character_backward: Binding::new()
                .keys(&["backspace", "ctrl+h"])
                .help("backspace", "delete character backward"),
            delete_character_forward: Binding::new()
                .keys(&["delete", "ctrl+d"])
                .help("delete", "delete character forward"),
            line_start: Binding::new()
                .keys(&["home", "ctrl+a"])
                .help("home", "line start"),
            line_end: Binding::new()
                .keys(&["end", "ctrl+e"])
                .help("end", "line end"),
            paste: Binding::new().keys(&["ctrl+v"]).help("ctrl+v", "paste"),
            input_begin: Binding::new()
                .keys(&["alt+<", "ctrl+home"])
                .help("alt+<", "input begin"),
            input_end: Binding::new()
                .keys(&["alt+>", "ctrl+end"])
                .help("alt+>", "input end"),
            capitalize_word_forward: Binding::new()
                .keys(&["alt+c"])
                .help("alt+c", "capitalize word forward"),
            lowercase_word_forward: Binding::new()
                .keys(&["alt+l"])
                .help("alt+l", "lowercase word forward"),
            uppercase_word_forward: Binding::new()
                .keys(&["alt+u"])
                .help("alt+u", "uppercase word forward"),
            transpose_character_backward: Binding::new()
                .keys(&["ctrl+t"])
                .help("ctrl+t", "transpose character backward"),
        }
    }
}

/// Styles for the textarea in different states.
#[derive(Debug, Clone)]
pub struct Styles {
    /// Base style.
    pub base: Style,
    /// Cursor line style.
    pub cursor_line: Style,
    /// Cursor line number style.
    pub cursor_line_number: Style,
    /// End of buffer style.
    pub end_of_buffer: Style,
    /// Line number style.
    pub line_number: Style,
    /// Placeholder style.
    pub placeholder: Style,
    /// Prompt style.
    pub prompt: Style,
    /// Text style.
    pub text: Style,
}

impl Default for Styles {
    fn default() -> Self {
        Self {
            base: Style::new(),
            cursor_line: Style::new(),
            cursor_line_number: Style::new().foreground_color(Color::from("240")),
            end_of_buffer: Style::new().foreground_color(Color::from("240")),
            line_number: Style::new().foreground_color(Color::from("240")),
            placeholder: Style::new().foreground_color(Color::from("240")),
            prompt: Style::new().foreground_color(Color::from("7")),
            text: Style::new(),
        }
    }
}

/// Message for paste operations.
#[derive(Debug, Clone)]
pub struct PasteMsg(pub String);

/// Message for paste errors.
#[derive(Debug, Clone)]
pub struct PasteErrMsg(pub String);

/// Multi-line text area model.
#[derive(Debug, Clone)]
pub struct TextArea {
    /// Current error.
    pub err: Option<String>,
    /// Prompt string (displayed at start of each line).
    pub prompt: String,
    /// Placeholder text.
    pub placeholder: String,
    /// Whether to show line numbers.
    pub show_line_numbers: bool,
    /// End of buffer character.
    pub end_of_buffer_character: char,
    /// Key bindings.
    pub key_map: KeyMap,
    /// Style for focused state.
    pub focused_style: Styles,
    /// Style for blurred state.
    pub blurred_style: Styles,
    /// Cursor model.
    pub cursor: Cursor,
    /// Character limit (0 = no limit).
    pub char_limit: usize,
    /// Maximum height in rows.
    pub max_height: usize,
    /// Maximum width in columns.
    pub max_width: usize,
    /// Current style (points to focused or blurred).
    use_focused_style: bool,
    /// Prompt width.
    prompt_width: usize,
    /// Display width.
    width: usize,
    /// Display height.
    height: usize,
    /// Text value (lines of characters).
    value: Vec<Vec<char>>,
    /// Focus state.
    focus: bool,
    /// Cursor column.
    col: usize,
    /// Cursor row.
    row: usize,
    /// Last character offset for vertical navigation.
    last_char_offset: usize,
    /// Viewport for scrolling.
    viewport: Viewport,
    /// Rune sanitizer.
    sanitizer: Sanitizer,
}

impl Default for TextArea {
    fn default() -> Self {
        Self::new()
    }
}

impl TextArea {
    /// Creates a new textarea with default settings.
    #[must_use]
    pub fn new() -> Self {
        let viewport = Viewport::new(0, 0);

        let mut ta = Self {
            err: None,
            prompt: "┃ ".to_string(),
            placeholder: String::new(),
            show_line_numbers: true,
            end_of_buffer_character: ' ',
            key_map: KeyMap::default(),
            focused_style: Styles::default(),
            blurred_style: Styles::default(),
            use_focused_style: false,
            cursor: Cursor::new(),
            char_limit: 0,
            max_height: DEFAULT_MAX_HEIGHT,
            max_width: DEFAULT_MAX_WIDTH,
            prompt_width: 2, // "┃ " is 2 chars
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            value: vec![Vec::new()],
            focus: false,
            col: 0,
            row: 0,
            last_char_offset: 0,
            viewport,
            sanitizer: Sanitizer::new(),
        };

        ta.set_height(DEFAULT_HEIGHT);
        ta.set_width(DEFAULT_WIDTH);
        ta
    }

    /// Sets the value of the textarea.
    pub fn set_value(&mut self, s: &str) {
        self.reset();
        self.insert_string(s);
    }

    /// Inserts a string at the cursor position.
    pub fn insert_string(&mut self, s: &str) {
        self.insert_runes_from_user_input(&s.chars().collect::<Vec<_>>());
    }

    /// Inserts a single character at the cursor position.
    pub fn insert_rune(&mut self, r: char) {
        self.insert_runes_from_user_input(&[r]);
    }

    fn insert_runes_from_user_input(&mut self, runes: &[char]) {
        let runes = self.sanitizer.sanitize(runes);

        let runes = if self.char_limit > 0 {
            let current_len = self.length();
            let avail = self.char_limit.saturating_sub(current_len);
            if avail == 0 {
                return;
            }
            if runes.len() > avail {
                runes[..avail].to_vec()
            } else {
                runes
            }
        } else {
            runes
        };

        // Split input into lines
        let mut lines: Vec<Vec<char>> = Vec::new();
        let mut current_line = Vec::new();

        for c in &runes {
            if *c == '\n' {
                lines.push(current_line);
                current_line = Vec::new();
            } else {
                current_line.push(*c);
            }
        }
        lines.push(current_line);

        // Obey max lines
        if MAX_LINES > 0 && self.value.len() + lines.len() - 1 > MAX_LINES {
            let allowed = MAX_LINES.saturating_sub(self.value.len()) + 1;
            lines.truncate(allowed);
        }

        if lines.is_empty() {
            return;
        }

        // Save tail of current line
        let tail: Vec<char> = self.value[self.row][self.col..].to_vec();

        // Paste first line at cursor
        self.value[self.row].truncate(self.col);
        self.value[self.row].extend_from_slice(&lines[0]);
        self.col += lines[0].len();

        // Handle additional lines
        if lines.len() > 1 {
            // Insert new lines
            for line in lines.into_iter().skip(1) {
                self.row += 1;
                self.value.insert(self.row, line.clone());
                self.col = line.len();
            }
        }

        // Add tail at end
        self.value[self.row].extend_from_slice(&tail);
        self.set_cursor_col(self.col);
    }

    /// Returns the current value as a string.
    #[must_use]
    pub fn value(&self) -> String {
        self.value
            .iter()
            .map(|line| line.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Returns the total length in characters.
    #[must_use]
    pub fn length(&self) -> usize {
        let char_count: usize = self.value.iter().map(|line| line.len()).sum();
        // Add newlines between lines
        char_count + self.value.len().saturating_sub(1)
    }

    /// Returns the number of lines.
    #[must_use]
    pub fn line_count(&self) -> usize {
        self.value.len()
    }

    /// Returns the current line number (0-indexed).
    #[must_use]
    pub fn line(&self) -> usize {
        self.row
    }

    /// Moves cursor down one line.
    pub fn cursor_down(&mut self) {
        if self.row < self.value.len() - 1 {
            self.row += 1;
            self.col = self.col.min(self.value[self.row].len());
        }
    }

    /// Moves cursor up one line.
    pub fn cursor_up(&mut self) {
        if self.row > 0 {
            self.row -= 1;
            self.col = self.col.min(self.value[self.row].len());
        }
    }

    /// Sets cursor column position.
    pub fn set_cursor_col(&mut self, col: usize) {
        self.col = col.min(self.value[self.row].len());
        self.last_char_offset = 0;
    }

    /// Moves cursor to start of line.
    pub fn cursor_start(&mut self) {
        self.set_cursor_col(0);
    }

    /// Moves cursor to end of line.
    pub fn cursor_end(&mut self) {
        self.set_cursor_col(self.value[self.row].len());
    }

    /// Returns whether the textarea is focused.
    #[must_use]
    pub fn focused(&self) -> bool {
        self.focus
    }

    /// Focuses the textarea.
    pub fn focus(&mut self) -> Option<Cmd> {
        self.focus = true;
        self.use_focused_style = true;
        self.cursor.focus()
    }

    /// Blurs the textarea.
    pub fn blur(&mut self) {
        self.focus = false;
        self.use_focused_style = false;
        self.cursor.blur();
    }

    /// Resets the textarea to empty.
    pub fn reset(&mut self) {
        self.value = vec![Vec::new()];
        self.col = 0;
        self.row = 0;
        self.viewport.goto_top();
        self.set_cursor_col(0);
    }

    fn current_style(&self) -> &Styles {
        if self.use_focused_style {
            &self.focused_style
        } else {
            &self.blurred_style
        }
    }

    fn delete_before_cursor(&mut self) {
        self.value[self.row] = self.value[self.row][self.col..].to_vec();
        self.set_cursor_col(0);
    }

    fn delete_after_cursor(&mut self) {
        self.value[self.row].truncate(self.col);
        self.set_cursor_col(self.value[self.row].len());
    }

    fn transpose_left(&mut self) {
        if self.col == 0 || self.value[self.row].len() < 2 {
            return;
        }
        if self.col >= self.value[self.row].len() {
            self.set_cursor_col(self.col.saturating_sub(1));
        }
        self.value[self.row].swap(self.col - 1, self.col);
        if self.col < self.value[self.row].len() {
            self.set_cursor_col(self.col + 1);
        }
    }

    fn delete_word_left(&mut self) {
        if self.col == 0 || self.value[self.row].is_empty() {
            return;
        }

        let old_col = self.col;
        self.set_cursor_col(self.col.saturating_sub(1));

        // Skip whitespace
        while self.col > 0
            && self.value[self.row]
                .get(self.col)
                .is_some_and(|c| c.is_whitespace())
        {
            self.set_cursor_col(self.col.saturating_sub(1));
        }

        // Skip non-whitespace
        while self.col > 0 {
            if !self.value[self.row]
                .get(self.col)
                .is_some_and(|c| c.is_whitespace())
            {
                self.set_cursor_col(self.col.saturating_sub(1));
            } else {
                if self.col > 0 {
                    self.set_cursor_col(self.col + 1);
                }
                break;
            }
        }

        let mut new_line = self.value[self.row][..self.col].to_vec();
        if old_col <= self.value[self.row].len() {
            new_line.extend_from_slice(&self.value[self.row][old_col..]);
        }
        self.value[self.row] = new_line;
    }

    fn delete_word_right(&mut self) {
        if self.col >= self.value[self.row].len() || self.value[self.row].is_empty() {
            return;
        }

        let old_col = self.col;

        // Skip whitespace
        while self.col < self.value[self.row].len()
            && self.value[self.row]
                .get(self.col)
                .is_some_and(|c| c.is_whitespace())
        {
            self.set_cursor_col(self.col + 1);
        }

        // Skip non-whitespace
        while self.col < self.value[self.row].len() {
            if !self.value[self.row]
                .get(self.col)
                .is_some_and(|c| c.is_whitespace())
            {
                self.set_cursor_col(self.col + 1);
            } else {
                break;
            }
        }

        let mut new_line = self.value[self.row][..old_col].to_vec();
        if self.col <= self.value[self.row].len() {
            new_line.extend_from_slice(&self.value[self.row][self.col..]);
        }
        self.value[self.row] = new_line;
        self.set_cursor_col(old_col);
    }

    fn character_right(&mut self) {
        if self.col < self.value[self.row].len() {
            self.set_cursor_col(self.col + 1);
        } else if self.row < self.value.len() - 1 {
            self.row += 1;
            self.cursor_start();
        }
    }

    fn character_left(&mut self, inside_line: bool) {
        if self.col == 0 && self.row > 0 {
            self.row -= 1;
            self.cursor_end();
            if !inside_line {
                return;
            }
        }
        if self.col > 0 {
            self.set_cursor_col(self.col - 1);
        }
    }

    fn word_left(&mut self) {
        loop {
            self.character_left(true);
            if self.col < self.value[self.row].len()
                && !self.value[self.row]
                    .get(self.col)
                    .is_some_and(|c| c.is_whitespace())
            {
                break;
            }
            if self.col == 0 && self.row == 0 {
                break;
            }
        }

        while self.col > 0 {
            if self.value[self.row]
                .get(self.col - 1)
                .is_some_and(|c| c.is_whitespace())
            {
                break;
            }
            self.set_cursor_col(self.col - 1);
        }
    }

    fn word_right(&mut self) {
        // Skip whitespace
        while self.col >= self.value[self.row].len()
            || self.value[self.row]
                .get(self.col)
                .is_some_and(|c| c.is_whitespace())
        {
            if self.row == self.value.len() - 1 && self.col == self.value[self.row].len() {
                break;
            }
            self.character_right();
        }

        // Skip non-whitespace
        while self.col < self.value[self.row].len() {
            if self.value[self.row]
                .get(self.col)
                .is_some_and(|c| c.is_whitespace())
            {
                break;
            }
            self.set_cursor_col(self.col + 1);
        }
    }

    fn uppercase_right(&mut self) {
        self.do_word_right(|line, i| {
            line[i] = line[i].to_uppercase().next().unwrap_or(line[i]);
        });
    }

    fn lowercase_right(&mut self) {
        self.do_word_right(|line, i| {
            line[i] = line[i].to_lowercase().next().unwrap_or(line[i]);
        });
    }

    fn capitalize_right(&mut self) {
        let mut char_idx = 0;
        self.do_word_right(|line, i| {
            if char_idx == 0 {
                line[i] = line[i].to_uppercase().next().unwrap_or(line[i]);
            }
            char_idx += 1;
        });
    }

    fn do_word_right<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Vec<char>, usize),
    {
        // Skip whitespace
        while self.col >= self.value[self.row].len()
            || self.value[self.row]
                .get(self.col)
                .is_some_and(|c| c.is_whitespace())
        {
            if self.row == self.value.len() - 1 && self.col == self.value[self.row].len() {
                break;
            }
            self.character_right();
        }

        while self.col < self.value[self.row].len() {
            if self.value[self.row]
                .get(self.col)
                .is_some_and(|c| c.is_whitespace())
            {
                break;
            }
            f(&mut self.value[self.row], self.col);
            self.set_cursor_col(self.col + 1);
        }
    }

    fn move_to_begin(&mut self) {
        self.row = 0;
        self.set_cursor_col(0);
    }

    fn move_to_end(&mut self) {
        self.row = self.value.len().saturating_sub(1);
        self.set_cursor_col(self.value[self.row].len());
    }

    /// Sets the width of the textarea.
    pub fn set_width(&mut self, w: usize) {
        self.prompt_width = self.prompt.chars().count();

        let reserved_outer = 0; // No frame in base style
        let mut reserved_inner = self.prompt_width;

        if self.show_line_numbers {
            reserved_inner += 4; // Line numbers
        }

        let min_width = reserved_inner + reserved_outer + 1;
        let mut input_width = w.max(min_width);

        if self.max_width > 0 {
            input_width = input_width.min(self.max_width);
        }

        self.viewport.width = input_width.saturating_sub(reserved_outer);
        self.width = input_width
            .saturating_sub(reserved_outer)
            .saturating_sub(reserved_inner);
    }

    /// Returns the width.
    #[must_use]
    pub fn width(&self) -> usize {
        self.width
    }

    /// Sets the height of the textarea.
    pub fn set_height(&mut self, h: usize) {
        if self.max_height > 0 {
            self.height = h.clamp(MIN_HEIGHT, self.max_height);
            self.viewport.height = h.clamp(MIN_HEIGHT, self.max_height);
        } else {
            self.height = h.max(MIN_HEIGHT);
            self.viewport.height = h.max(MIN_HEIGHT);
        }
    }

    /// Returns the height.
    #[must_use]
    pub fn height(&self) -> usize {
        self.height
    }

    fn merge_line_below(&mut self, row: usize) {
        if row >= self.value.len() - 1 {
            return;
        }

        let below = self.value.remove(row + 1);
        self.value[row].extend(below);
    }

    fn merge_line_above(&mut self, row: usize) {
        if row == 0 {
            return;
        }

        self.col = self.value[row - 1].len();
        let current = self.value.remove(row);
        self.value[row - 1].extend(current);
        self.row -= 1;
    }

    fn split_line(&mut self, row: usize, col: usize) {
        let tail = self.value[row][col..].to_vec();
        self.value[row].truncate(col);
        self.value.insert(row + 1, tail);
        self.col = 0;
        self.row += 1;
    }

    fn reposition_view(&mut self) {
        let minimum = self.viewport.y_offset();
        let maximum = minimum + self.viewport.height.saturating_sub(1);

        if self.row < minimum {
            self.viewport.scroll_up(minimum - self.row);
        } else if self.row > maximum {
            self.viewport.scroll_down(self.row - maximum);
        }
    }

    /// Updates the textarea based on messages.
    pub fn update(&mut self, msg: Message) -> Option<Cmd> {
        if !self.focus {
            self.cursor.blur();
            return None;
        }

        let old_row = self.row;
        let old_col = self.col;

        // Handle paste message
        if let Some(paste) = msg.downcast_ref::<PasteMsg>() {
            self.insert_runes_from_user_input(&paste.0.chars().collect::<Vec<_>>());
        }

        if let Some(paste_err) = msg.downcast_ref::<PasteErrMsg>() {
            self.err = Some(paste_err.0.clone());
        }

        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            let key_str = key.to_string();

            if matches(&key_str, &[&self.key_map.delete_after_cursor]) {
                self.col = self.col.min(self.value[self.row].len());
                if self.col >= self.value[self.row].len() {
                    self.merge_line_below(self.row);
                } else {
                    self.delete_after_cursor();
                }
            } else if matches(&key_str, &[&self.key_map.delete_before_cursor]) {
                self.col = self.col.min(self.value[self.row].len());
                if self.col == 0 {
                    self.merge_line_above(self.row);
                } else {
                    self.delete_before_cursor();
                }
            } else if matches(&key_str, &[&self.key_map.delete_character_backward]) {
                self.col = self.col.min(self.value[self.row].len());
                if self.col == 0 {
                    self.merge_line_above(self.row);
                } else if !self.value[self.row].is_empty() {
                    self.value[self.row].remove(self.col - 1);
                    self.set_cursor_col(self.col.saturating_sub(1));
                }
            } else if matches(&key_str, &[&self.key_map.delete_character_forward]) {
                if !self.value[self.row].is_empty() && self.col < self.value[self.row].len() {
                    self.value[self.row].remove(self.col);
                }
                if self.col >= self.value[self.row].len() {
                    self.merge_line_below(self.row);
                }
            } else if matches(&key_str, &[&self.key_map.delete_word_backward]) {
                if self.col == 0 {
                    self.merge_line_above(self.row);
                } else {
                    self.delete_word_left();
                }
            } else if matches(&key_str, &[&self.key_map.delete_word_forward]) {
                self.col = self.col.min(self.value[self.row].len());
                if self.col >= self.value[self.row].len() {
                    self.merge_line_below(self.row);
                } else {
                    self.delete_word_right();
                }
            } else if matches(&key_str, &[&self.key_map.insert_newline]) {
                if self.max_height == 0 || self.value.len() < self.max_height {
                    self.col = self.col.min(self.value[self.row].len());
                    self.split_line(self.row, self.col);
                }
            } else if matches(&key_str, &[&self.key_map.line_end]) {
                self.cursor_end();
            } else if matches(&key_str, &[&self.key_map.line_start]) {
                self.cursor_start();
            } else if matches(&key_str, &[&self.key_map.character_forward]) {
                self.character_right();
            } else if matches(&key_str, &[&self.key_map.line_next]) {
                self.cursor_down();
            } else if matches(&key_str, &[&self.key_map.word_forward]) {
                self.word_right();
            } else if matches(&key_str, &[&self.key_map.character_backward]) {
                self.character_left(false);
            } else if matches(&key_str, &[&self.key_map.line_previous]) {
                self.cursor_up();
            } else if matches(&key_str, &[&self.key_map.word_backward]) {
                self.word_left();
            } else if matches(&key_str, &[&self.key_map.input_begin]) {
                self.move_to_begin();
            } else if matches(&key_str, &[&self.key_map.input_end]) {
                self.move_to_end();
            } else if matches(&key_str, &[&self.key_map.lowercase_word_forward]) {
                self.lowercase_right();
            } else if matches(&key_str, &[&self.key_map.uppercase_word_forward]) {
                self.uppercase_right();
            } else if matches(&key_str, &[&self.key_map.capitalize_word_forward]) {
                self.capitalize_right();
            } else if matches(&key_str, &[&self.key_map.transpose_character_backward]) {
                self.transpose_left();
            } else if !matches(&key_str, &[&self.key_map.paste]) {
                // Insert regular characters
                let runes: Vec<char> = key.runes.clone();
                if !runes.is_empty() {
                    self.insert_runes_from_user_input(&runes);
                }
            }
        }

        self.viewport.update(&msg);

        let mut cmds: Vec<Option<Cmd>> = Vec::new();

        if let Some(cmd) = self.cursor.update(msg) {
            cmds.push(Some(cmd));
        }

        if (self.row != old_row || self.col != old_col) && self.cursor.mode() == CursorMode::Blink {
            // Reset blink state when cursor moves - trigger blink cycle
            cmds.push(Some(blink_cmd()));
        }

        self.reposition_view();

        bubbletea::batch(cmds)
    }

    /// Renders the textarea.
    #[must_use]
    pub fn view(&self) -> String {
        if self.value() == "" && self.row == 0 && self.col == 0 && !self.placeholder.is_empty() {
            return self.placeholder_view();
        }

        let style = self.current_style();
        let mut lines = Vec::new();

        for (l, line) in self.value.iter().enumerate() {
            let is_cursor_line = self.row == l;
            let line_style = if is_cursor_line {
                &style.cursor_line
            } else {
                &style.text
            };

            let mut s = String::new();

            // Prompt
            s.push_str(&style.prompt.render(&self.prompt));

            // Line numbers
            if self.show_line_numbers {
                let ln_style = if is_cursor_line {
                    &style.cursor_line_number
                } else {
                    &style.line_number
                };
                s.push_str(&ln_style.render(&format!("{:>3} ", l + 1)));
            }

            // Line content
            let line_str: String = line.iter().collect();
            if is_cursor_line {
                let before: String = line[..self.col.min(line.len())].iter().collect();
                s.push_str(&line_style.render(&before));

                if self.col < line.len() {
                    let cursor_char: String = line[self.col..self.col + 1].iter().collect();
                    let mut cursor = self.cursor.clone();
                    cursor.set_char(&cursor_char);
                    s.push_str(&cursor.view());

                    let after: String = line[self.col + 1..].iter().collect();
                    s.push_str(&line_style.render(&after));
                } else {
                    let mut cursor = self.cursor.clone();
                    cursor.set_char(" ");
                    s.push_str(&cursor.view());
                }
            } else {
                s.push_str(&line_style.render(&line_str));
            }

            // Padding
            let padding = self.width.saturating_sub(line.len());
            if padding > 0 {
                s.push_str(&line_style.render(&" ".repeat(padding)));
            }

            lines.push(s);
        }

        // Pad to height with empty lines
        while lines.len() < self.height {
            let mut s = String::new();
            s.push_str(&style.prompt.render(&self.prompt));
            if self.show_line_numbers {
                s.push_str(&style.line_number.render("    "));
            }
            s.push_str(
                &style
                    .end_of_buffer
                    .render(&format!("{}", self.end_of_buffer_character)),
            );
            let padding = self.width.saturating_sub(1);
            s.push_str(&" ".repeat(padding));
            lines.push(s);
        }

        // Apply viewport
        let start = self.viewport.y_offset();
        let end = (start + self.height).min(lines.len());
        let visible: String = lines[start..end].join("\n");

        style.base.render(&visible)
    }

    fn placeholder_view(&self) -> String {
        let style = self.current_style();
        let mut lines = Vec::new();

        let placeholder_lines: Vec<&str> = self.placeholder.lines().collect();

        for i in 0..self.height {
            let mut s = String::new();

            // Prompt
            s.push_str(&style.prompt.render(&self.prompt));

            // Line numbers
            if self.show_line_numbers {
                let ln_style = if i == 0 {
                    &style.cursor_line_number
                } else {
                    &style.line_number
                };
                if i == 0 {
                    s.push_str(&ln_style.render(&format!("{:>3} ", 1)));
                } else {
                    s.push_str(&ln_style.render("    "));
                }
            }

            if i < placeholder_lines.len() {
                let line = placeholder_lines[i];
                if i == 0 && !line.is_empty() {
                    // First char as cursor
                    let first: String = line.chars().take(1).collect();
                    let rest: String = line.chars().skip(1).collect();

                    let mut cursor = self.cursor.clone();
                    cursor.text_style = style.placeholder.clone();
                    cursor.set_char(&first);
                    s.push_str(&cursor.view());
                    s.push_str(&style.placeholder.render(&rest));
                } else {
                    s.push_str(&style.placeholder.render(line));
                }
            } else {
                s.push_str(
                    &style
                        .end_of_buffer
                        .render(&format!("{}", self.end_of_buffer_character)),
                );
            }

            lines.push(s);
        }

        style.base.render(&lines.join("\n"))
    }
}

impl Model for TextArea {
    /// Initialize the textarea and return a blink command if focused.
    fn init(&self) -> Option<Cmd> {
        if self.focus {
            Some(blink_cmd())
        } else {
            None
        }
    }

    /// Update the textarea state based on incoming messages.
    fn update(&mut self, msg: Message) -> Option<Cmd> {
        TextArea::update(self, msg)
    }

    /// Render the textarea.
    fn view(&self) -> String {
        TextArea::view(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_textarea_new() {
        let ta = TextArea::new();
        assert_eq!(ta.height, DEFAULT_HEIGHT);
        assert!(ta.show_line_numbers);
        assert!(!ta.focused());
    }

    #[test]
    fn test_textarea_set_value() {
        let mut ta = TextArea::new();
        ta.set_value("Hello\nWorld");
        assert_eq!(ta.value(), "Hello\nWorld");
        assert_eq!(ta.line_count(), 2);
    }

    #[test]
    fn test_textarea_cursor_navigation() {
        let mut ta = TextArea::new();
        ta.set_value("Line 1\nLine 2\nLine 3");

        assert_eq!(ta.row, 2); // After set_value, cursor at end
        ta.move_to_begin();
        assert_eq!(ta.row, 0);
        assert_eq!(ta.col, 0);

        ta.cursor_end();
        assert_eq!(ta.col, 6);

        ta.cursor_down();
        assert_eq!(ta.row, 1);
    }

    #[test]
    fn test_textarea_focus_blur() {
        let mut ta = TextArea::new();
        assert!(!ta.focused());

        ta.focus();
        assert!(ta.focused());

        ta.blur();
        assert!(!ta.focused());
    }

    #[test]
    fn test_textarea_reset() {
        let mut ta = TextArea::new();
        ta.set_value("Hello\nWorld");
        ta.reset();
        assert_eq!(ta.value(), "");
        assert_eq!(ta.line_count(), 1);
    }

    #[test]
    fn test_textarea_insert_newline() {
        let mut ta = TextArea::new();
        ta.set_value("Hello");
        ta.move_to_begin();
        ta.set_cursor_col(2); // After "He"
        ta.split_line(0, 2);

        assert_eq!(ta.line_count(), 2);
        assert_eq!(ta.value(), "He\nllo");
    }

    #[test]
    fn test_textarea_delete_line() {
        let mut ta = TextArea::new();
        ta.set_value("Line 1\nLine 2\nLine 3");
        ta.move_to_begin();
        ta.row = 1;
        ta.col = 0;
        ta.merge_line_above(1);

        assert_eq!(ta.line_count(), 2);
        assert_eq!(ta.value(), "Line 1Line 2\nLine 3");
    }

    #[test]
    fn test_textarea_char_limit() {
        let mut ta = TextArea::new();
        ta.char_limit = 10;
        ta.set_value("This is a very long string");
        assert!(ta.length() <= 10);
    }

    #[test]
    fn test_textarea_dimensions() {
        let mut ta = TextArea::new();
        ta.set_width(80);
        ta.set_height(24);

        assert_eq!(ta.height(), 24);
    }

    #[test]
    fn test_textarea_view() {
        let mut ta = TextArea::new();
        ta.set_value("Hello\nWorld");
        let view = ta.view();

        assert!(view.contains("Hello"));
        assert!(view.contains("World"));
    }

    #[test]
    fn test_textarea_placeholder() {
        let mut ta = TextArea::new();
        ta.placeholder = "Enter text...".to_string();
        let view = ta.view();
        // The placeholder is split by cursor rendering: "E" (cursor) + "nter text..." (styled)
        // So check for both parts - the cursor char and the rest
        assert!(view.contains("E"), "View should contain cursor char 'E'");
        assert!(
            view.contains("nter text..."),
            "View should contain rest of placeholder"
        );
    }

    #[test]
    fn test_keymap_default() {
        let km = KeyMap::default();
        assert!(!km.character_forward.get_keys().is_empty());
        assert!(!km.insert_newline.get_keys().is_empty());
    }

    // Model trait implementation tests
    #[test]
    fn test_model_init_unfocused() {
        let ta = TextArea::new();
        // Unfocused textarea should not return init command
        let cmd = Model::init(&ta);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_model_init_focused() {
        let mut ta = TextArea::new();
        ta.focus();
        // Focused textarea should return blink command
        let cmd = Model::init(&ta);
        assert!(cmd.is_some());
    }

    #[test]
    fn test_model_view() {
        let mut ta = TextArea::new();
        ta.set_value("Test content");
        // Model::view should return same result as TextArea::view
        let model_view = Model::view(&ta);
        let textarea_view = TextArea::view(&ta);
        assert_eq!(model_view, textarea_view);
    }
}
