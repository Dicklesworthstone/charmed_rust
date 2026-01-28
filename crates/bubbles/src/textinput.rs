//! Single-line text input component.
//!
//! This module provides a text input field for TUI applications with features
//! like password masking, suggestions, and validation.
//!
//! # Example
//!
//! ```rust
//! use bubbles::textinput::TextInput;
//!
//! let mut input = TextInput::new();
//! input.set_placeholder("Enter your name");
//! input.set_value("Hello");
//!
//! // Render the input
//! let view = input.view();
//! ```

use crate::cursor::{Cursor, blink_cmd};
use crate::key::{Binding, matches};
use crate::runeutil::Sanitizer;
use bubbletea::{Cmd, KeyMsg, Message, Model};
use lipgloss::{Color, Style};
use unicode_width::UnicodeWidthChar;

/// Echo mode for the text input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EchoMode {
    /// Display text as-is (default).
    #[default]
    Normal,
    /// Display echo character instead of actual text (for passwords).
    Password,
    /// Display nothing (hidden input).
    None,
}

/// Validation function type.
pub type ValidateFn = Box<dyn Fn(&str) -> Option<String> + Send + Sync>;

/// Key bindings for text input navigation.
#[derive(Debug, Clone)]
pub struct KeyMap {
    /// Move cursor forward one character.
    pub character_forward: Binding,
    /// Move cursor backward one character.
    pub character_backward: Binding,
    /// Move cursor forward one word.
    pub word_forward: Binding,
    /// Move cursor backward one word.
    pub word_backward: Binding,
    /// Delete word backward.
    pub delete_word_backward: Binding,
    /// Delete word forward.
    pub delete_word_forward: Binding,
    /// Delete text after cursor.
    pub delete_after_cursor: Binding,
    /// Delete text before cursor.
    pub delete_before_cursor: Binding,
    /// Delete character backward.
    pub delete_character_backward: Binding,
    /// Delete character forward.
    pub delete_character_forward: Binding,
    /// Move to start of line.
    pub line_start: Binding,
    /// Move to end of line.
    pub line_end: Binding,
    /// Paste from clipboard.
    pub paste: Binding,
    /// Accept current suggestion.
    pub accept_suggestion: Binding,
    /// Next suggestion.
    pub next_suggestion: Binding,
    /// Previous suggestion.
    pub prev_suggestion: Binding,
}

impl Default for KeyMap {
    fn default() -> Self {
        Self {
            character_forward: Binding::new().keys(&["right", "ctrl+f"]),
            character_backward: Binding::new().keys(&["left", "ctrl+b"]),
            word_forward: Binding::new().keys(&["alt+right", "ctrl+right", "alt+f"]),
            word_backward: Binding::new().keys(&["alt+left", "ctrl+left", "alt+b"]),
            delete_word_backward: Binding::new().keys(&["alt+backspace", "ctrl+w"]),
            delete_word_forward: Binding::new().keys(&["alt+delete", "alt+d"]),
            delete_after_cursor: Binding::new().keys(&["ctrl+k"]),
            delete_before_cursor: Binding::new().keys(&["ctrl+u"]),
            delete_character_backward: Binding::new().keys(&["backspace", "ctrl+h"]),
            delete_character_forward: Binding::new().keys(&["delete", "ctrl+d"]),
            line_start: Binding::new().keys(&["home", "ctrl+a"]),
            line_end: Binding::new().keys(&["end", "ctrl+e"]),
            paste: Binding::new().keys(&["ctrl+v"]),
            accept_suggestion: Binding::new().keys(&["tab"]),
            next_suggestion: Binding::new().keys(&["down", "ctrl+n"]),
            prev_suggestion: Binding::new().keys(&["up", "ctrl+p"]),
        }
    }
}

/// Message for paste operations.
#[derive(Debug, Clone)]
pub struct PasteMsg(pub String);

/// Message for paste errors.
#[derive(Debug, Clone)]
pub struct PasteErrMsg(pub String);

/// Single-line text input model.
pub struct TextInput {
    /// Current error from validation.
    pub err: Option<String>,
    /// Prompt displayed before input.
    pub prompt: String,
    /// Placeholder text when empty.
    pub placeholder: String,
    /// Echo mode (normal, password, none).
    pub echo_mode: EchoMode,
    /// Character to display in password mode.
    pub echo_character: char,
    /// Cursor model.
    pub cursor: Cursor,
    /// Style for the prompt.
    pub prompt_style: Style,
    /// Style for the text.
    pub text_style: Style,
    /// Style for the placeholder.
    pub placeholder_style: Style,
    /// Style for completions.
    pub completion_style: Style,
    /// Maximum characters allowed (0 = no limit).
    pub char_limit: usize,
    /// Maximum display width (0 = no limit).
    pub width: usize,
    /// Key bindings.
    pub key_map: KeyMap,
    /// Whether to show suggestions.
    pub show_suggestions: bool,
    /// Underlying text value.
    value: Vec<char>,
    /// Focus state.
    focus: bool,
    /// Cursor position.
    pos: usize,
    /// Viewport offset (left).
    offset: usize,
    /// Viewport offset (right).
    offset_right: usize,
    /// Validation function.
    validate: Option<ValidateFn>,
    /// Rune sanitizer.
    sanitizer: Sanitizer,
    /// Available suggestions.
    suggestions: Vec<Vec<char>>,
    /// Matched suggestions.
    matched_suggestions: Vec<Vec<char>>,
    /// Current suggestion index.
    current_suggestion_index: usize,
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for TextInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextInput")
            .field("err", &self.err)
            .field("prompt", &self.prompt)
            .field("placeholder", &self.placeholder)
            .field("echo_mode", &self.echo_mode)
            .field("cursor", &self.cursor)
            .field("char_limit", &self.char_limit)
            .field("width", &self.width)
            .field("focus", &self.focus)
            .field("pos", &self.pos)
            .field("value_len", &self.value.len())
            .field("validate", &self.validate.as_ref().map(|_| "<fn>"))
            .finish()
    }
}

impl Clone for TextInput {
    fn clone(&self) -> Self {
        Self {
            err: self.err.clone(),
            prompt: self.prompt.clone(),
            placeholder: self.placeholder.clone(),
            echo_mode: self.echo_mode,
            echo_character: self.echo_character,
            cursor: self.cursor.clone(),
            prompt_style: self.prompt_style.clone(),
            text_style: self.text_style.clone(),
            placeholder_style: self.placeholder_style.clone(),
            completion_style: self.completion_style.clone(),
            char_limit: self.char_limit,
            width: self.width,
            key_map: self.key_map.clone(),
            show_suggestions: self.show_suggestions,
            value: self.value.clone(),
            focus: self.focus,
            pos: self.pos,
            offset: self.offset,
            offset_right: self.offset_right,
            validate: None, // Can't clone Box<dyn Fn>
            sanitizer: self.sanitizer.clone(),
            suggestions: self.suggestions.clone(),
            matched_suggestions: self.matched_suggestions.clone(),
            current_suggestion_index: self.current_suggestion_index,
        }
    }
}

impl TextInput {
    /// Creates a new text input with default settings.
    #[must_use]
    pub fn new() -> Self {
        let sanitizer = Sanitizer::new()
            .with_tab_replacement(" ")
            .with_newline_replacement(" ");

        Self {
            err: None,
            prompt: "> ".to_string(),
            placeholder: String::new(),
            echo_mode: EchoMode::Normal,
            echo_character: '*',
            cursor: Cursor::new(),
            prompt_style: Style::new(),
            text_style: Style::new(),
            placeholder_style: Style::new().foreground_color(Color::from("240")),
            completion_style: Style::new().foreground_color(Color::from("240")),
            char_limit: 0,
            width: 0,
            key_map: KeyMap::default(),
            show_suggestions: false,
            value: Vec::new(),
            focus: false,
            pos: 0,
            offset: 0,
            offset_right: 0,
            validate: None,
            sanitizer,
            suggestions: Vec::new(),
            matched_suggestions: Vec::new(),
            current_suggestion_index: 0,
        }
    }

    /// Sets the prompt string.
    pub fn set_prompt(&mut self, prompt: impl Into<String>) {
        self.prompt = prompt.into();
    }

    /// Sets the placeholder text.
    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.placeholder = placeholder.into();
    }

    /// Sets the echo mode.
    pub fn set_echo_mode(&mut self, mode: EchoMode) {
        self.echo_mode = mode;
    }

    /// Sets the value of the text input.
    pub fn set_value(&mut self, s: &str) {
        let runes = self.sanitizer.sanitize(&s.chars().collect::<Vec<_>>());
        let err = self.do_validate(&runes);
        self.set_value_internal(runes, err);
    }

    fn set_value_internal(&mut self, runes: Vec<char>, err: Option<String>) {
        self.err = err;
        let empty = self.value.is_empty();

        if self.char_limit > 0 && runes.len() > self.char_limit {
            self.value = runes[..self.char_limit].to_vec();
        } else {
            self.value = runes;
        }

        if (self.pos == 0 && empty) || self.pos > self.value.len() {
            self.set_cursor(self.value.len());
        }
        self.handle_overflow();
        self.update_suggestions();
    }

    /// Returns the current value as a string.
    #[must_use]
    pub fn value(&self) -> String {
        self.value.iter().collect()
    }

    /// Returns the cursor position.
    #[must_use]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Sets the cursor position.
    pub fn set_cursor(&mut self, pos: usize) {
        self.pos = pos.min(self.value.len());
        self.handle_overflow();
    }

    /// Moves cursor to start of input.
    pub fn cursor_start(&mut self) {
        self.set_cursor(0);
    }

    /// Moves cursor to end of input.
    pub fn cursor_end(&mut self) {
        self.set_cursor(self.value.len());
    }

    /// Returns whether the input is focused.
    #[must_use]
    pub fn focused(&self) -> bool {
        self.focus
    }

    /// Focuses the input and returns the cursor blink command.
    pub fn focus(&mut self) -> Option<Cmd> {
        self.focus = true;
        self.cursor.focus()
    }

    /// Blurs the input.
    pub fn blur(&mut self) {
        self.focus = false;
        self.cursor.blur();
    }

    /// Resets the input to empty.
    pub fn reset(&mut self) {
        self.value.clear();
        self.err = self.do_validate(&self.value);
        self.set_cursor(0);
        self.update_suggestions();
    }

    /// Sets the suggestions list.
    pub fn set_suggestions(&mut self, suggestions: &[&str]) {
        self.suggestions = suggestions.iter().map(|s| s.chars().collect()).collect();
        self.update_suggestions();
    }

    /// Sets the validation function.
    pub fn set_validate<F>(&mut self, f: F)
    where
        F: Fn(&str) -> Option<String> + Send + Sync + 'static,
    {
        self.validate = Some(Box::new(f));
    }

    /// Returns available suggestions as strings.
    #[must_use]
    pub fn available_suggestions(&self) -> Vec<String> {
        self.suggestions
            .iter()
            .map(|s| s.iter().collect())
            .collect()
    }

    /// Returns matched suggestions as strings.
    #[must_use]
    pub fn matched_suggestions(&self) -> Vec<String> {
        self.matched_suggestions
            .iter()
            .map(|s| s.iter().collect())
            .collect()
    }

    /// Returns the current suggestion index.
    #[must_use]
    pub fn current_suggestion_index(&self) -> usize {
        self.current_suggestion_index
    }

    /// Returns the current suggestion.
    #[must_use]
    pub fn current_suggestion(&self) -> String {
        self.matched_suggestions
            .get(self.current_suggestion_index)
            .map(|s| s.iter().collect())
            .unwrap_or_default()
    }

    fn do_validate(&self, v: &[char]) -> Option<String> {
        self.validate
            .as_ref()
            .and_then(|f| f(&v.iter().collect::<String>()))
    }

    fn insert_runes_from_user_input(&mut self, v: &[char]) {
        let paste = self.sanitizer.sanitize(v);

        let mut available = if self.char_limit > 0 {
            let avail = self.char_limit.saturating_sub(self.value.len());
            if avail == 0 {
                return;
            }
            avail
        } else {
            usize::MAX
        };

        let paste = if paste.len() > available {
            &paste[..available]
        } else {
            &paste
        };

        // Split at cursor
        let head = &self.value[..self.pos];
        let tail = &self.value[self.pos..];

        let mut new_value = head.to_vec();
        for &c in paste {
            if available == 0 {
                break;
            }
            new_value.push(c);
            self.pos += 1;
            available = available.saturating_sub(1);
        }
        new_value.extend_from_slice(tail);

        let err = self.do_validate(&new_value);
        self.set_value_internal(new_value, err);
    }

    fn handle_overflow(&mut self) {
        let total_width: usize = self.value.iter().map(|c| c.width().unwrap_or(0)).sum();
        if self.width == 0 || total_width <= self.width {
            self.offset = 0;
            self.offset_right = self.value.len();
            return;
        }

        self.offset_right = self.offset_right.min(self.value.len());

        if self.pos < self.offset {
            self.offset = self.pos;
            let mut w = 0;
            let mut i = 0;
            let runes = &self.value[self.offset..];

            while i < runes.len() {
                let cw = runes[i].width().unwrap_or(0);
                if w + cw > self.width {
                    break;
                }
                w += cw;
                i += 1;
            }
            self.offset_right = self.offset + i;
        } else if self.pos >= self.offset_right {
            self.offset_right = self.pos;
            let mut w = 0;
            let runes = &self.value[..self.offset_right];
            let mut start_index = self.offset_right;

            // Scan backwards from offset_right
            while start_index > 0 {
                let prev = start_index - 1;
                let cw = runes[prev].width().unwrap_or(0);
                if w + cw > self.width {
                    break;
                }
                w += cw;
                start_index = prev;
            }
            self.offset = start_index;
        }
    }

    fn delete_before_cursor(&mut self) {
        self.value = self.value[self.pos..].to_vec();
        self.err = self.do_validate(&self.value);
        self.offset = 0;
        self.set_cursor(0);
    }

    fn delete_after_cursor(&mut self) {
        self.value = self.value[..self.pos].to_vec();
        self.err = self.do_validate(&self.value);
        self.set_cursor(self.value.len());
    }

    fn delete_word_backward(&mut self) {
        if self.pos == 0 || self.value.is_empty() {
            return;
        }

        if self.echo_mode != EchoMode::Normal {
            self.delete_before_cursor();
            return;
        }

        let old_pos = self.pos;
        self.set_cursor(self.pos.saturating_sub(1));

        // Skip whitespace backward
        while self.pos > 0 {
            let prev = self.pos - 1;
            if let Some(c) = self.value.get(prev) {
                if c.is_whitespace() {
                    self.set_cursor(prev);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Skip non-whitespace backward
        while self.pos > 0 {
            let prev = self.pos - 1;
            if let Some(c) = self.value.get(prev) {
                if !c.is_whitespace() {
                    self.set_cursor(prev);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if old_pos > self.value.len() {
            self.value = self.value[..self.pos].to_vec();
        } else {
            let mut new_value = self.value[..self.pos].to_vec();
            new_value.extend_from_slice(&self.value[old_pos..]);
            self.value = new_value;
        }
        self.err = self.do_validate(&self.value);
        self.handle_overflow();
    }

    fn delete_word_forward(&mut self) {
        if self.pos >= self.value.len() || self.value.is_empty() {
            return;
        }

        if self.echo_mode != EchoMode::Normal {
            self.delete_after_cursor();
            return;
        }

        let old_pos = self.pos;
        self.set_cursor(self.pos + 1);

        // Skip whitespace
        while self.pos < self.value.len()
            && self.value.get(self.pos).is_some_and(|c| c.is_whitespace())
        {
            self.set_cursor(self.pos + 1);
        }

        // Skip non-whitespace
        while self.pos < self.value.len() {
            if !self.value.get(self.pos).is_some_and(|c| c.is_whitespace()) {
                self.set_cursor(self.pos + 1);
            } else {
                break;
            }
        }

        if self.pos > self.value.len() {
            self.value = self.value[..old_pos].to_vec();
        } else {
            let mut new_value = self.value[..old_pos].to_vec();
            new_value.extend_from_slice(&self.value[self.pos..]);
            self.value = new_value;
        }
        self.err = self.do_validate(&self.value);
        self.set_cursor(old_pos);
    }

    fn word_backward(&mut self) {
        if self.pos == 0 || self.value.is_empty() {
            return;
        }

        if self.echo_mode != EchoMode::Normal {
            self.cursor_start();
            return;
        }

        // Skip whitespace backward
        while self.pos > 0 {
            let prev = self.pos - 1;
            if let Some(c) = self.value.get(prev) {
                if c.is_whitespace() {
                    self.set_cursor(prev);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Skip non-whitespace backward
        while self.pos > 0 {
            let prev = self.pos - 1;
            if let Some(c) = self.value.get(prev) {
                if !c.is_whitespace() {
                    self.set_cursor(prev);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn word_forward(&mut self) {
        if self.pos >= self.value.len() || self.value.is_empty() {
            return;
        }

        if self.echo_mode != EchoMode::Normal {
            self.cursor_end();
            return;
        }

        let mut i = self.pos;

        // Skip whitespace
        while i < self.value.len() && self.value.get(i).is_some_and(|c| c.is_whitespace()) {
            self.set_cursor(self.pos + 1);
            i += 1;
        }

        // Skip non-whitespace
        while i < self.value.len() {
            if !self.value.get(i).is_some_and(|c| c.is_whitespace()) {
                self.set_cursor(self.pos + 1);
                i += 1;
            } else {
                break;
            }
        }
    }

    fn echo_transform(&self, v: &str) -> String {
        match self.echo_mode {
            EchoMode::Normal => v.to_string(),
            EchoMode::Password => self.echo_character.to_string().repeat(v.chars().count()),
            EchoMode::None => String::new(),
        }
    }

    fn can_accept_suggestion(&self) -> bool {
        !self.matched_suggestions.is_empty()
    }

    fn update_suggestions(&mut self) {
        if !self.show_suggestions {
            return;
        }

        if self.value.is_empty() || self.suggestions.is_empty() {
            self.matched_suggestions.clear();
            return;
        }

        let value_str: String = self.value.iter().collect();
        let value_lower = value_str.to_lowercase();

        let matches: Vec<Vec<char>> = self
            .suggestions
            .iter()
            .filter(|s| {
                let suggestion: String = s.iter().collect();
                suggestion.to_lowercase().starts_with(&value_lower)
            })
            .cloned()
            .collect();

        if matches != self.matched_suggestions {
            self.current_suggestion_index = 0;
        }

        self.matched_suggestions = matches;
    }

    fn next_suggestion(&mut self) {
        if self.matched_suggestions.is_empty() {
            return;
        }
        self.current_suggestion_index =
            (self.current_suggestion_index + 1) % self.matched_suggestions.len();
    }

    fn previous_suggestion(&mut self) {
        if self.matched_suggestions.is_empty() {
            return;
        }
        if self.current_suggestion_index == 0 {
            self.current_suggestion_index = self.matched_suggestions.len().saturating_sub(1);
        } else {
            self.current_suggestion_index -= 1;
        }
    }

    /// Updates the text input based on messages.
    pub fn update(&mut self, msg: Message) -> Option<Cmd> {
        if !self.focus {
            return None;
        }

        // Handle paste message
        if let Some(paste) = msg.downcast_ref::<PasteMsg>() {
            self.insert_runes_from_user_input(&paste.0.chars().collect::<Vec<_>>());
            return None;
        }

        if let Some(paste_err) = msg.downcast_ref::<PasteErrMsg>() {
            self.err = Some(paste_err.0.clone());
            return None;
        }

        let old_pos = self.pos;

        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            let key_str = key.to_string();

            // Check for suggestion acceptance first
            if matches(&key_str, &[&self.key_map.accept_suggestion])
                && self.can_accept_suggestion()
                && let Some(suggestion) =
                    self.matched_suggestions.get(self.current_suggestion_index)
                && self.value.len() < suggestion.len()
            {
                self.value
                    .extend_from_slice(&suggestion[self.value.len()..]);
                self.cursor_end();
            }

            if matches(&key_str, &[&self.key_map.delete_word_backward]) {
                self.delete_word_backward();
            } else if matches(&key_str, &[&self.key_map.delete_character_backward]) {
                self.err = None;
                if !self.value.is_empty() && self.pos > 0 {
                    self.value.remove(self.pos - 1);
                    self.err = self.do_validate(&self.value);
                    self.set_cursor(self.pos.saturating_sub(1));
                }
            } else if matches(&key_str, &[&self.key_map.word_backward]) {
                self.word_backward();
            } else if matches(&key_str, &[&self.key_map.character_backward]) {
                if self.pos > 0 {
                    self.set_cursor(self.pos - 1);
                }
            } else if matches(&key_str, &[&self.key_map.word_forward]) {
                self.word_forward();
            } else if matches(&key_str, &[&self.key_map.character_forward]) {
                if self.pos < self.value.len() {
                    self.set_cursor(self.pos + 1);
                }
            } else if matches(&key_str, &[&self.key_map.line_start]) {
                self.cursor_start();
            } else if matches(&key_str, &[&self.key_map.delete_character_forward]) {
                if !self.value.is_empty() && self.pos < self.value.len() {
                    self.value.remove(self.pos);
                    self.err = self.do_validate(&self.value);
                }
            } else if matches(&key_str, &[&self.key_map.line_end]) {
                self.cursor_end();
            } else if matches(&key_str, &[&self.key_map.delete_after_cursor]) {
                self.delete_after_cursor();
            } else if matches(&key_str, &[&self.key_map.delete_before_cursor]) {
                self.delete_before_cursor();
            } else if matches(&key_str, &[&self.key_map.delete_word_forward]) {
                self.delete_word_forward();
            } else if matches(&key_str, &[&self.key_map.next_suggestion]) {
                self.next_suggestion();
            } else if matches(&key_str, &[&self.key_map.prev_suggestion]) {
                self.previous_suggestion();
            } else if !matches(
                &key_str,
                &[&self.key_map.paste, &self.key_map.accept_suggestion],
            ) {
                // Input regular characters
                let runes: Vec<char> = key.runes.clone();
                if !runes.is_empty() {
                    self.insert_runes_from_user_input(&runes);
                }
            }

            self.update_suggestions();
        }

        let mut cmds: Vec<Option<Cmd>> = Vec::new();

        if let Some(cmd) = self.cursor.update(msg) {
            cmds.push(Some(cmd));
        }

        if old_pos != self.pos && self.cursor.mode() == crate::cursor::Mode::Blink {
            // Reset blink state when cursor moves - trigger blink cycle
            cmds.push(Some(blink_cmd()));
        }

        self.handle_overflow();

        bubbletea::batch(cmds)
    }

    /// Renders the text input.
    #[must_use]
    pub fn view(&self) -> String {
        if self.value.is_empty() && !self.placeholder.is_empty() {
            return self.placeholder_view();
        }

        let value: Vec<char> = self.value[self.offset..self.offset_right].to_vec();
        let pos = self.pos.saturating_sub(self.offset);

        let before: String = value[..pos.min(value.len())].iter().collect();
        let mut v = self
            .text_style
            .clone()
            .inline()
            .render(&self.echo_transform(&before));

        if pos < value.len() {
            let char_at_cursor: String = value[pos..pos + 1].iter().collect();
            let char_display = self.echo_transform(&char_at_cursor);

            let mut cursor = self.cursor.clone();
            cursor.set_char(&char_display);
            v.push_str(&cursor.view());

            let after: String = value[pos + 1..].iter().collect();
            v.push_str(
                &self
                    .text_style
                    .clone()
                    .inline()
                    .render(&self.echo_transform(&after)),
            );
            v.push_str(&self.completion_view(0));
        } else if self.focus && self.can_accept_suggestion() {
            if let Some(suggestion) = self.matched_suggestions.get(self.current_suggestion_index) {
                if self.value.len() < suggestion.len() && self.pos < suggestion.len() {
                    let mut cursor = self.cursor.clone();
                    cursor.text_style = self.completion_style.clone();
                    let char_display: String = suggestion[self.pos..self.pos + 1].iter().collect();
                    cursor.set_char(&self.echo_transform(&char_display));
                    v.push_str(&cursor.view());
                    v.push_str(&self.completion_view(1));
                } else {
                    let mut cursor = self.cursor.clone();
                    cursor.set_char(" ");
                    v.push_str(&cursor.view());
                }
            }
        } else {
            let mut cursor = self.cursor.clone();
            cursor.set_char(" ");
            v.push_str(&cursor.view());
        }

        // Padding for width
        if self.width > 0 {
            let val_width: usize = value.iter().map(|c| c.width().unwrap_or(0)).sum();
            if val_width <= self.width {
                let padding = self.width.saturating_sub(val_width);
                v.push_str(
                    &self
                        .text_style
                        .clone()
                        .inline()
                        .render(&" ".repeat(padding)),
                );
            }
        }

        format!("{}{}", self.prompt_style.render(&self.prompt), v)
    }

    fn placeholder_view(&self) -> String {
        let prompt = self.prompt_style.render(&self.prompt);

        let mut cursor = self.cursor.clone();
        cursor.text_style = self.placeholder_style.clone();

        let first_char: String = self.placeholder.chars().take(1).collect();
        let rest: String = self.placeholder.chars().skip(1).collect();

        cursor.set_char(&first_char);
        let v = cursor.view();

        let styled_rest = self.placeholder_style.clone().inline().render(&rest);

        format!("{}{}{}", prompt, v, styled_rest)
    }

    fn completion_view(&self, offset: usize) -> String {
        if self.can_accept_suggestion()
            && let Some(suggestion) = self.matched_suggestions.get(self.current_suggestion_index)
            && self.value.len() + offset <= suggestion.len()
        {
            let completion: String = suggestion[self.value.len() + offset..].iter().collect();
            return self.placeholder_style.clone().inline().render(&completion);
        }
        String::new()
    }
}

impl Model for TextInput {
    /// Initialize the text input.
    ///
    /// If focused and cursor is in blink mode, returns a blink command.
    fn init(&self) -> Option<Cmd> {
        if self.focus { Some(blink_cmd()) } else { None }
    }

    /// Update the text input state based on incoming messages.
    ///
    /// Handles:
    /// - `KeyMsg` - Keyboard input for text entry and navigation
    /// - `PasteMsg` - Paste operations
    /// - Cursor blink messages
    fn update(&mut self, msg: Message) -> Option<Cmd> {
        TextInput::update(self, msg)
    }

    /// Render the text input.
    fn view(&self) -> String {
        TextInput::view(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_textinput_new() {
        let input = TextInput::new();
        assert_eq!(input.prompt, "> ");
        assert_eq!(input.echo_character, '*');
        assert!(!input.focused());
    }

    #[test]
    fn test_textinput_set_value() {
        let mut input = TextInput::new();
        input.set_value("hello");
        assert_eq!(input.value(), "hello");
    }

    #[test]
    fn test_textinput_cursor_position() {
        let mut input = TextInput::new();
        input.set_value("hello");
        assert_eq!(input.position(), 5);

        input.set_cursor(2);
        assert_eq!(input.position(), 2);

        input.cursor_start();
        assert_eq!(input.position(), 0);

        input.cursor_end();
        assert_eq!(input.position(), 5);
    }

    #[test]
    fn test_textinput_focus_blur() {
        let mut input = TextInput::new();
        assert!(!input.focused());

        input.focus();
        assert!(input.focused());

        input.blur();
        assert!(!input.focused());
    }

    #[test]
    fn test_textinput_reset() {
        let mut input = TextInput::new();
        input.set_value("hello");
        assert!(!input.value.is_empty());

        input.reset();
        assert!(input.value.is_empty());
    }

    #[test]
    fn test_textinput_reset_clears_error_and_suggestions() {
        let mut input = TextInput::new();
        input.show_suggestions = true;
        input.set_suggestions(&["apple", "apricot"]);
        input.set_validate(|v| (!v.is_empty()).then(|| "err".to_string()));

        input.set_value("ap");
        assert!(input.err.is_some());
        assert!(!input.matched_suggestions().is_empty());

        input.reset();
        assert!(input.err.is_none());
        assert!(input.matched_suggestions().is_empty());
    }

    #[test]
    fn test_textinput_char_limit() {
        let mut input = TextInput::new();
        input.char_limit = 5;
        input.set_value("hello world");
        assert_eq!(input.value(), "hello");
    }

    #[test]
    fn test_textinput_echo_mode() {
        let mut input = TextInput::new();
        input.set_value("secret");

        assert_eq!(input.echo_transform("secret"), "secret");

        input.echo_mode = EchoMode::Password;
        assert_eq!(input.echo_transform("secret"), "******");

        input.echo_mode = EchoMode::None;
        assert_eq!(input.echo_transform("secret"), "");
    }

    #[test]
    fn test_textinput_placeholder() {
        let mut input = TextInput::new();
        input.set_placeholder("Enter text...");
        assert_eq!(input.placeholder, "Enter text...");
    }

    #[test]
    fn test_textinput_suggestions() {
        let mut input = TextInput::new();
        input.show_suggestions = true;
        input.set_suggestions(&["apple", "apricot", "banana"]);
        input.set_value("ap");
        input.update_suggestions();

        assert_eq!(input.matched_suggestions().len(), 2);
        assert!(input.matched_suggestions().contains(&"apple".to_string()));
        assert!(input.matched_suggestions().contains(&"apricot".to_string()));
    }

    #[test]
    fn test_textinput_set_value_updates_suggestions() {
        let mut input = TextInput::new();
        input.show_suggestions = true;
        input.set_suggestions(&["apple", "banana"]);

        input.set_value("ap");

        assert_eq!(input.matched_suggestions().len(), 1);
        assert!(input.matched_suggestions().contains(&"apple".to_string()));
    }

    #[test]
    fn test_textinput_suggestion_overflow_uses_global_position() {
        let mut input = TextInput::new();
        input.width = 5;
        input.show_suggestions = true;
        input.set_value("abcdefghij");
        input.set_suggestions(&["abcdefghijZ"]);
        input.focus();
        input.cursor_end();

        let view = input.view();
        assert!(
            view.contains("Z"),
            "Expected suggestion character to render at cursor when scrolled"
        );
    }

    #[test]
    fn test_textinput_validation() {
        let mut input = TextInput::new();
        input.set_validate(|s| {
            if s.contains("bad") {
                Some("Contains bad word".to_string())
            } else {
                None
            }
        });

        input.set_value("good");
        assert!(input.err.is_none());

        input.set_value("bad");
        assert!(input.err.is_some());
    }

    #[test]
    fn test_textinput_view() {
        let mut input = TextInput::new();
        input.set_value("hello");
        let view = input.view();
        assert!(view.contains("> "));
        assert!(view.contains("hello"));
    }

    #[test]
    fn test_textinput_placeholder_view() {
        let mut input = TextInput::new();
        input.set_placeholder("Type here...");
        let view = input.view();
        assert!(view.contains("> "));
    }

    #[test]
    fn test_keymap_default() {
        let km = KeyMap::default();
        assert!(!km.character_forward.get_keys().is_empty());
        assert!(!km.delete_character_backward.get_keys().is_empty());
    }

    // Model trait implementation tests
    #[test]
    fn test_model_init_unfocused() {
        let input = TextInput::new();
        // Unfocused input should not return init command
        let cmd = Model::init(&input);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_model_init_focused() {
        let mut input = TextInput::new();
        input.focus();
        // Focused input should return init command for cursor blink
        let cmd = Model::init(&input);
        assert!(cmd.is_some());
    }

    #[test]
    fn test_model_view() {
        let mut input = TextInput::new();
        input.set_value("test");
        // Model::view should return same result as TextInput::view
        let model_view = Model::view(&input);
        let textinput_view = TextInput::view(&input);
        assert_eq!(model_view, textinput_view);
    }

    #[test]
    fn test_model_update_handles_paste_msg() {
        let mut input = TextInput::new();
        input.focus();
        assert_eq!(input.value(), "");

        // Use Model::update to handle a paste message
        let paste_msg = Message::new(PasteMsg("hello world".to_string()));
        let _ = Model::update(&mut input, paste_msg);

        assert_eq!(input.value(), "hello world");
    }

    #[test]
    fn test_model_update_unfocused_ignores_input() {
        let mut input = TextInput::new();
        assert!(!input.focused());
        assert_eq!(input.value(), "");

        // Unfocused input should ignore paste
        let paste_msg = Message::new(PasteMsg("ignored".to_string()));
        let _ = Model::update(&mut input, paste_msg);

        assert_eq!(input.value(), "", "Unfocused input should ignore messages");
    }

    #[test]
    fn test_model_update_handles_key_input() {
        let mut input = TextInput::new();
        input.focus();
        input.set_value("hello");
        assert_eq!(input.position(), 5);

        // Create a left arrow key message to move cursor
        let key_msg = Message::new(KeyMsg {
            key_type: bubbletea::KeyType::Left,
            runes: vec![],
            alt: false,
            paste: false,
        });
        let _ = Model::update(&mut input, key_msg);

        assert_eq!(input.position(), 4, "Cursor should have moved left");
    }

    #[test]
    fn test_textinput_satisfies_model_bounds() {
        // Verify TextInput can be used where Model + Send + 'static is required
        fn accepts_model<M: Model + Send + 'static>(_model: M) {}
        let input = TextInput::new();
        accepts_model(input);
    }

    #[test]
    fn test_word_backward_boundary() {
        let mut input = TextInput::new();
        input.set_value("abc");
        input.set_cursor(1); // "a|bc" (cursor is at 1, so 'b' is to the right, 'a' to the left)
        input.word_backward();
        assert_eq!(input.position(), 0); // Should move to start
    }

    #[test]
    fn test_delete_word_backward_boundary() {
        let mut input = TextInput::new();
        input.set_value("abc");
        input.set_cursor(1); // "a|bc"
        input.delete_word_backward();
        assert_eq!(input.value(), "bc");
        assert_eq!(input.position(), 0);
    }

    #[test]
    fn test_handle_overflow_wide_chars() {
        let mut input = TextInput::new();
        input.width = 3;
        input.set_value("a😀b"); // 'a' (1), '😀' (2), 'b' (1). Total 4.
        // pos=0. offset=0. offset_right=?
        // w=0. 'a'(1) -> w=1. '😀'(2) -> w=3. 'b'(1) -> w=4 > 3. Break.
        // offset_right should be 2 ("a😀").

        // Force overflow update
        input.set_cursor(0);
        // internal update triggers handle_overflow

        // Can't check internal state easily without exposing or deducing from view
        // But let's check view length
        let view = input.view();
        // view should contain "a😀" (char count 2) or something fitting width 3.
        // But view strips ANSI from result to check?
        // Let's rely on logic correctness.
        // width=3. "a😀" is width 3. "a😀b" is 4.
        // So it should show "a😀".
        assert!(view.contains("a😀"));
        assert!(!view.contains("b")); // 'b' is clipped
    }

    #[test]
    fn test_delete_word_backward_on_whitespace() {
        let mut input = TextInput::new();
        input.set_value("abc   ");
        input.set_cursor(6); // At end

        input.delete_word_backward();

        // Current implementation: deletes whitespace AND word -> ""
        // "Standard" (bash) behavior: deletes whitespace -> "abc"
        // Let's see what it does.
        assert_eq!(
            input.value(),
            "",
            "Aggressive deletion: deleted both whitespace and word"
        );
    }

    // === Bracketed Paste Tests ===
    // These tests verify paste behavior when receiving KeyMsg with paste=true,
    // which is how terminals deliver bracketed paste sequences.

    #[test]
    fn test_bracketed_paste_basic() {
        let mut input = TextInput::new();
        input.focus();

        // Simulate bracketed paste: KeyMsg with paste=true and runes
        let key_msg = Message::new(KeyMsg {
            key_type: bubbletea::KeyType::Runes,
            runes: vec!['h', 'e', 'l', 'l', 'o'],
            alt: false,
            paste: true,
        });
        let _ = Model::update(&mut input, key_msg);

        assert_eq!(input.value(), "hello");
    }

    #[test]
    fn test_bracketed_paste_multiline_converts_newlines() {
        let mut input = TextInput::new();
        input.focus();

        // Paste with newlines - should be converted to spaces for single-line input
        let key_msg = Message::new(KeyMsg {
            key_type: bubbletea::KeyType::Runes,
            runes: "line1\nline2\nline3".chars().collect(),
            alt: false,
            paste: true,
        });
        let _ = Model::update(&mut input, key_msg);

        assert_eq!(
            input.value(),
            "line1 line2 line3",
            "Newlines should be converted to spaces in single-line input"
        );
    }

    #[test]
    fn test_bracketed_paste_crlf_converts_to_space() {
        let mut input = TextInput::new();
        input.focus();

        // Windows-style CRLF should also be converted
        let key_msg = Message::new(KeyMsg {
            key_type: bubbletea::KeyType::Runes,
            runes: "line1\r\nline2".chars().collect(),
            alt: false,
            paste: true,
        });
        let _ = Model::update(&mut input, key_msg);

        assert_eq!(
            input.value(),
            "line1 line2",
            "CRLF should be converted to single space"
        );
    }

    #[test]
    fn test_bracketed_paste_respects_char_limit() {
        let mut input = TextInput::new();
        input.focus();
        input.char_limit = 10;

        // Try to paste more than the limit
        let key_msg = Message::new(KeyMsg {
            key_type: bubbletea::KeyType::Runes,
            runes: "this is a very long paste that exceeds the limit".chars().collect(),
            alt: false,
            paste: true,
        });
        let _ = Model::update(&mut input, key_msg);

        assert_eq!(input.value().len(), 10, "Paste should be truncated at char_limit");
        assert_eq!(input.value(), "this is a ");
    }

    #[test]
    fn test_bracketed_paste_respects_remaining_capacity() {
        let mut input = TextInput::new();
        input.focus();
        input.char_limit = 15;
        input.set_value("hello ");

        // Paste should only insert up to the remaining capacity
        let key_msg = Message::new(KeyMsg {
            key_type: bubbletea::KeyType::Runes,
            runes: "world and more text".chars().collect(),
            alt: false,
            paste: true,
        });
        let _ = Model::update(&mut input, key_msg);

        assert_eq!(input.value().len(), 15);
        assert_eq!(input.value(), "hello world and");
    }

    #[test]
    fn test_bracketed_paste_at_full_capacity_ignored() {
        let mut input = TextInput::new();
        input.focus();
        input.char_limit = 5;
        input.set_value("hello");

        // Input is at capacity, paste should be ignored
        let key_msg = Message::new(KeyMsg {
            key_type: bubbletea::KeyType::Runes,
            runes: "world".chars().collect(),
            alt: false,
            paste: true,
        });
        let _ = Model::update(&mut input, key_msg);

        assert_eq!(input.value(), "hello", "Paste at full capacity should be ignored");
    }

    #[test]
    fn test_bracketed_paste_unfocused_ignored() {
        let mut input = TextInput::new();
        // Not focused!
        assert_eq!(input.value(), "");

        let key_msg = Message::new(KeyMsg {
            key_type: bubbletea::KeyType::Runes,
            runes: "ignored".chars().collect(),
            alt: false,
            paste: true,
        });
        let _ = Model::update(&mut input, key_msg);

        assert_eq!(input.value(), "", "Unfocused input should ignore paste");
    }

    #[test]
    fn test_bracketed_paste_inserts_at_cursor() {
        let mut input = TextInput::new();
        input.focus();
        input.set_value("helloworld");
        input.set_cursor(5); // Position after "hello"

        let key_msg = Message::new(KeyMsg {
            key_type: bubbletea::KeyType::Runes,
            runes: " ".chars().collect(),
            alt: false,
            paste: true,
        });
        let _ = Model::update(&mut input, key_msg);

        assert_eq!(input.value(), "hello world");
        assert_eq!(input.position(), 6, "Cursor should be after pasted content");
    }

    #[test]
    fn test_bracketed_paste_strips_control_chars() {
        let mut input = TextInput::new();
        input.focus();

        // Paste with control characters that should be stripped
        let key_msg = Message::new(KeyMsg {
            key_type: bubbletea::KeyType::Runes,
            runes: "hello\x01\x02world".chars().collect(),
            alt: false,
            paste: true,
        });
        let _ = Model::update(&mut input, key_msg);

        assert_eq!(
            input.value(),
            "helloworld",
            "Control characters should be stripped"
        );
    }

    #[test]
    fn test_bracketed_paste_preserves_unicode() {
        let mut input = TextInput::new();
        input.focus();

        let key_msg = Message::new(KeyMsg {
            key_type: bubbletea::KeyType::Runes,
            runes: "hello 世界 🌍".chars().collect(),
            alt: false,
            paste: true,
        });
        let _ = Model::update(&mut input, key_msg);

        assert_eq!(input.value(), "hello 世界 🌍");
    }

    #[test]
    fn test_bracketed_paste_tabs_to_spaces() {
        let mut input = TextInput::new();
        input.focus();

        // Tabs should be converted to spaces
        let key_msg = Message::new(KeyMsg {
            key_type: bubbletea::KeyType::Runes,
            runes: "col1\tcol2".chars().collect(),
            alt: false,
            paste: true,
        });
        let _ = Model::update(&mut input, key_msg);

        assert_eq!(
            input.value(),
            "col1 col2",
            "Tabs should be converted to single space"
        );
    }
}
