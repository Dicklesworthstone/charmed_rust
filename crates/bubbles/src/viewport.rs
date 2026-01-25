//! Scrollable viewport component.
//!
//! This module provides a viewport for rendering scrollable content in TUI
//! applications.
//!
//! # Example
//!
//! ```rust
//! use bubbles::viewport::Viewport;
//!
//! let mut viewport = Viewport::new(80, 24);
//! viewport.set_content("Line 1\nLine 2\nLine 3");
//!
//! // Scroll down
//! viewport.scroll_down(1);
//! ```

use crate::key::{Binding, matches};
use bubbletea::{Cmd, KeyMsg, Message, Model, MouseMsg};
use lipgloss::Style;

/// Key bindings for viewport navigation.
#[derive(Debug, Clone)]
pub struct KeyMap {
    /// Page down binding.
    pub page_down: Binding,
    /// Page up binding.
    pub page_up: Binding,
    /// Half page up binding.
    pub half_page_up: Binding,
    /// Half page down binding.
    pub half_page_down: Binding,
    /// Down one line binding.
    pub down: Binding,
    /// Up one line binding.
    pub up: Binding,
    /// Scroll left binding.
    pub left: Binding,
    /// Scroll right binding.
    pub right: Binding,
}

impl Default for KeyMap {
    fn default() -> Self {
        Self {
            page_down: Binding::new()
                .keys(&["pgdown", " ", "f"])
                .help("f/pgdn", "page down"),
            page_up: Binding::new()
                .keys(&["pgup", "b"])
                .help("b/pgup", "page up"),
            half_page_up: Binding::new().keys(&["u", "ctrl+u"]).help("u", "½ page up"),
            half_page_down: Binding::new()
                .keys(&["d", "ctrl+d"])
                .help("d", "½ page down"),
            up: Binding::new().keys(&["up", "k"]).help("↑/k", "up"),
            down: Binding::new().keys(&["down", "j"]).help("↓/j", "down"),
            left: Binding::new().keys(&["left", "h"]).help("←/h", "move left"),
            right: Binding::new()
                .keys(&["right", "l"])
                .help("→/l", "move right"),
        }
    }
}

/// Viewport model for scrollable content.
#[derive(Debug, Clone)]
pub struct Viewport {
    /// Width of the viewport.
    pub width: usize,
    /// Height of the viewport.
    pub height: usize,
    /// Key bindings for navigation.
    pub key_map: KeyMap,
    /// Whether mouse wheel scrolling is enabled.
    pub mouse_wheel_enabled: bool,
    /// Number of lines to scroll per mouse wheel tick.
    pub mouse_wheel_delta: usize,
    /// Vertical scroll offset.
    y_offset: usize,
    /// Horizontal scroll offset.
    x_offset: usize,
    /// Horizontal scroll step size.
    horizontal_step: usize,
    /// Style for rendering the viewport.
    pub style: Style,
    /// Content lines.
    lines: Vec<String>,
    /// Width of the longest line.
    longest_line_width: usize,
}

impl Viewport {
    /// Creates a new viewport with the given dimensions.
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            key_map: KeyMap::default(),
            mouse_wheel_enabled: true,
            mouse_wheel_delta: 3,
            y_offset: 0,
            x_offset: 0,
            horizontal_step: 0,
            style: Style::new(),
            lines: Vec::new(),
            longest_line_width: 0,
        }
    }

    /// Sets the content of the viewport.
    pub fn set_content(&mut self, content: &str) {
        let normalized = content.replace("\r\n", "\n");
        self.lines = normalized.split('\n').map(String::from).collect();
        self.longest_line_width = self
            .lines
            .iter()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(0);

        if self.y_offset > self.lines.len().saturating_sub(1) {
            self.goto_bottom();
        }
    }

    /// Returns the vertical scroll offset.
    #[must_use]
    pub fn y_offset(&self) -> usize {
        self.y_offset
    }

    /// Sets the vertical scroll offset.
    pub fn set_y_offset(&mut self, n: usize) {
        self.y_offset = n.min(self.max_y_offset());
    }

    /// Returns the horizontal scroll offset.
    #[must_use]
    pub fn x_offset(&self) -> usize {
        self.x_offset
    }

    /// Sets the horizontal scroll offset.
    pub fn set_x_offset(&mut self, n: usize) {
        self.x_offset = n.min(self.longest_line_width.saturating_sub(self.width));
    }

    /// Sets the horizontal scroll step size.
    pub fn set_horizontal_step(&mut self, n: usize) {
        self.horizontal_step = n;
    }

    /// Returns whether the viewport is at the top.
    #[must_use]
    pub fn at_top(&self) -> bool {
        self.y_offset == 0
    }

    /// Returns whether the viewport is at the bottom.
    #[must_use]
    pub fn at_bottom(&self) -> bool {
        self.y_offset >= self.max_y_offset()
    }

    /// Returns whether the viewport is past the bottom.
    #[must_use]
    pub fn past_bottom(&self) -> bool {
        self.y_offset > self.max_y_offset()
    }

    /// Returns the scroll percentage (0.0 to 1.0).
    #[must_use]
    pub fn scroll_percent(&self) -> f64 {
        if self.height >= self.lines.len() {
            return 1.0;
        }
        let y = self.y_offset as f64;
        let h = self.height as f64;
        let t = self.lines.len() as f64;
        let v = y / (t - h);
        v.clamp(0.0, 1.0)
    }

    /// Returns the horizontal scroll percentage (0.0 to 1.0).
    #[must_use]
    pub fn horizontal_scroll_percent(&self) -> f64 {
        if self.x_offset >= self.longest_line_width.saturating_sub(self.width) {
            return 1.0;
        }
        let x = self.x_offset as f64;
        let w = self.width as f64;
        let t = self.longest_line_width as f64;
        let v = x / (t - w);
        v.clamp(0.0, 1.0)
    }

    /// Returns the total number of lines.
    #[must_use]
    pub fn total_line_count(&self) -> usize {
        self.lines.len()
    }

    /// Returns the number of visible lines.
    #[must_use]
    pub fn visible_line_count(&self) -> usize {
        self.visible_lines().len()
    }

    /// Returns the maximum Y offset.
    fn max_y_offset(&self) -> usize {
        self.lines.len().saturating_sub(self.height)
    }

    /// Returns the currently visible lines.
    fn visible_lines(&self) -> &[String] {
        if self.lines.is_empty() {
            return &[];
        }

        let top = self.y_offset.min(self.lines.len());
        let bottom = (self.y_offset + self.height).min(self.lines.len());

        &self.lines[top..bottom]
    }

    /// Scrolls down by the given number of lines.
    pub fn scroll_down(&mut self, n: usize) {
        if self.at_bottom() || n == 0 || self.lines.is_empty() {
            return;
        }
        self.set_y_offset(self.y_offset + n);
    }

    /// Scrolls up by the given number of lines.
    pub fn scroll_up(&mut self, n: usize) {
        if self.at_top() || n == 0 || self.lines.is_empty() {
            return;
        }
        self.set_y_offset(self.y_offset.saturating_sub(n));
    }

    /// Scrolls left by the given number of columns.
    pub fn scroll_left(&mut self, n: usize) {
        self.set_x_offset(self.x_offset.saturating_sub(n));
    }

    /// Scrolls right by the given number of columns.
    pub fn scroll_right(&mut self, n: usize) {
        self.set_x_offset(self.x_offset + n);
    }

    /// Moves down one page.
    pub fn page_down(&mut self) {
        if !self.at_bottom() {
            self.scroll_down(self.height);
        }
    }

    /// Moves up one page.
    pub fn page_up(&mut self) {
        if !self.at_top() {
            self.scroll_up(self.height);
        }
    }

    /// Moves down half a page.
    pub fn half_page_down(&mut self) {
        if !self.at_bottom() {
            self.scroll_down(self.height / 2);
        }
    }

    /// Moves up half a page.
    pub fn half_page_up(&mut self) {
        if !self.at_top() {
            self.scroll_up(self.height / 2);
        }
    }

    /// Goes to the top.
    pub fn goto_top(&mut self) {
        self.set_y_offset(0);
    }

    /// Goes to the bottom.
    pub fn goto_bottom(&mut self) {
        self.set_y_offset(self.max_y_offset());
    }

    /// Updates the viewport based on key/mouse input.
    pub fn update(&mut self, msg: &Message) {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            let key_str = key.to_string();

            if matches(&key_str, &[&self.key_map.page_down]) {
                self.page_down();
            } else if matches(&key_str, &[&self.key_map.page_up]) {
                self.page_up();
            } else if matches(&key_str, &[&self.key_map.half_page_down]) {
                self.half_page_down();
            } else if matches(&key_str, &[&self.key_map.half_page_up]) {
                self.half_page_up();
            } else if matches(&key_str, &[&self.key_map.down]) {
                self.scroll_down(1);
            } else if matches(&key_str, &[&self.key_map.up]) {
                self.scroll_up(1);
            } else if matches(&key_str, &[&self.key_map.left]) {
                self.scroll_left(self.horizontal_step);
            } else if matches(&key_str, &[&self.key_map.right]) {
                self.scroll_right(self.horizontal_step);
            }
            return;
        }

        if let Some(mouse) = msg.downcast_ref::<MouseMsg>() {
            if !self.mouse_wheel_enabled {
                return;
            }
            match mouse.button {
                bubbletea::MouseButton::WheelUp => self.scroll_up(self.mouse_wheel_delta),
                bubbletea::MouseButton::WheelDown => self.scroll_down(self.mouse_wheel_delta),
                bubbletea::MouseButton::WheelLeft => self.scroll_left(self.horizontal_step),
                bubbletea::MouseButton::WheelRight => self.scroll_right(self.horizontal_step),
                _ => {}
            }
        }
    }

    /// Renders the viewport content.
    #[must_use]
    pub fn view(&self) -> String {
        let content_height = self.height;
        let mut lines: Vec<String> = Vec::with_capacity(content_height);

        // Get visible content
        for line in self.visible_lines() {
            // Apply horizontal offset and width limit
            let chars: Vec<char> = line.chars().collect();
            let start = self.x_offset.min(chars.len());
            let end = (self.x_offset + self.width).min(chars.len());
            let visible: String = chars[start..end].iter().collect();
            lines.push(visible);
        }

        // Pad with empty lines if needed
        while lines.len() < content_height {
            lines.push(String::new());
        }

        self.style.render(&lines.join("\n"))
    }
}

/// Implement the Model trait for standalone bubbletea usage.
impl Model for Viewport {
    fn init(&self) -> Option<Cmd> {
        // Viewport doesn't need initialization
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Call the existing update method
        Viewport::update(self, &msg);
        None
    }

    fn view(&self) -> String {
        Viewport::view(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_new() {
        let v = Viewport::new(80, 24);
        assert_eq!(v.width, 80);
        assert_eq!(v.height, 24);
        assert!(v.mouse_wheel_enabled);
    }

    #[test]
    fn test_viewport_set_content() {
        let mut v = Viewport::new(80, 5);
        v.set_content("Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7");
        assert_eq!(v.total_line_count(), 7);
    }

    #[test]
    fn test_viewport_at_top_bottom() {
        let mut v = Viewport::new(80, 3);
        v.set_content("1\n2\n3\n4\n5");

        assert!(v.at_top());
        assert!(!v.at_bottom());

        v.goto_bottom();
        assert!(!v.at_top());
        assert!(v.at_bottom());
    }

    #[test]
    fn test_viewport_scroll() {
        let mut v = Viewport::new(80, 3);
        v.set_content("1\n2\n3\n4\n5\n6\n7\n8\n9\n10");

        assert_eq!(v.y_offset(), 0);

        v.scroll_down(2);
        assert_eq!(v.y_offset(), 2);

        v.scroll_up(1);
        assert_eq!(v.y_offset(), 1);
    }

    #[test]
    fn test_viewport_page_navigation() {
        let mut v = Viewport::new(80, 3);
        v.set_content("1\n2\n3\n4\n5\n6\n7\n8\n9\n10");

        v.page_down();
        assert_eq!(v.y_offset(), 3);

        v.page_up();
        assert_eq!(v.y_offset(), 0);
    }

    #[test]
    fn test_viewport_scroll_percent() {
        let mut v = Viewport::new(80, 5);
        v.set_content("1\n2\n3\n4\n5\n6\n7\n8\n9\n10");

        assert!((v.scroll_percent() - 0.0).abs() < 0.01);

        v.goto_bottom();
        assert!((v.scroll_percent() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_viewport_view() {
        let mut v = Viewport::new(80, 3);
        v.set_content("Line 1\nLine 2\nLine 3\nLine 4");

        let view = v.view();
        assert!(view.contains("Line 1"));
        assert!(view.contains("Line 2"));
        assert!(view.contains("Line 3"));
        assert!(!view.contains("Line 4"));
    }

    #[test]
    fn test_viewport_horizontal_scroll() {
        let mut v = Viewport::new(10, 5);
        v.set_horizontal_step(5);
        v.set_content("This is a very long line that exceeds the width");

        assert_eq!(v.x_offset(), 0);

        v.scroll_right(5);
        assert_eq!(v.x_offset(), 5);

        v.scroll_left(3);
        assert_eq!(v.x_offset(), 2);
    }

    #[test]
    fn test_viewport_empty_content() {
        let v = Viewport::new(80, 24);
        assert_eq!(v.total_line_count(), 0);
        assert!(v.at_top());
        assert!(v.at_bottom());
    }

    #[test]
    fn test_viewport_model_init_returns_none() {
        let v = Viewport::new(80, 24);
        assert!(Model::init(&v).is_none());
    }

    #[test]
    fn test_viewport_model_update_scrolls() {
        let mut v = Viewport::new(10, 2);
        v.set_content("1\n2\n3\n4");
        assert_eq!(v.y_offset(), 0);

        let down_msg = Message::new(KeyMsg::from_char('j'));
        let result = Model::update(&mut v, down_msg);
        assert!(result.is_none());
        assert_eq!(v.y_offset(), 1);
    }

    #[test]
    fn test_viewport_model_view_matches_view() {
        let mut v = Viewport::new(10, 2);
        v.set_content("Line 1\nLine 2\nLine 3");
        assert_eq!(Model::view(&v), v.view());
    }
}
