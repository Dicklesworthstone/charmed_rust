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
use unicode_width::UnicodeWidthChar;

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
            .map(|l| visible_width(l))
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
        self.lines.len().saturating_sub(self.content_height())
    }

    /// Returns the currently visible lines.
    fn visible_lines(&self) -> Vec<String> {
        if self.lines.is_empty() {
            return Vec::new();
        }

        let content_height = self.content_height();
        if content_height == 0 {
            return Vec::new();
        }

        let top = self.y_offset.min(self.lines.len());
        let bottom = top.saturating_add(content_height).min(self.lines.len());

        let visible = &self.lines[top..bottom];
        let content_width = self.content_width();
        if (self.x_offset == 0 && self.longest_line_width <= content_width) || content_width == 0 {
            return visible.to_vec();
        }

        visible
            .iter()
            .map(|line| cut_line(line, self.x_offset, content_width))
            .collect()
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
            if !self.mouse_wheel_enabled || mouse.action != bubbletea::MouseAction::Press {
                return;
            }
            match mouse.button {
                bubbletea::MouseButton::WheelUp => {
                    if mouse.shift {
                        self.scroll_left(self.horizontal_step);
                    } else {
                        self.scroll_up(self.mouse_wheel_delta);
                    }
                }
                bubbletea::MouseButton::WheelDown => {
                    if mouse.shift {
                        self.scroll_right(self.horizontal_step);
                    } else {
                        self.scroll_down(self.mouse_wheel_delta);
                    }
                }
                bubbletea::MouseButton::WheelLeft => self.scroll_left(self.horizontal_step),
                bubbletea::MouseButton::WheelRight => self.scroll_right(self.horizontal_step),
                _ => {}
            }
        }
    }

    /// Renders the viewport content.
    #[must_use]
    pub fn view(&self) -> String {
        let mut width = self.width;
        if let Some(style_width) = self.style.get_width()
            && style_width > 0
        {
            width = width.min(style_width as usize);
        }

        let mut height = self.height;
        if let Some(style_height) = self.style.get_height()
            && style_height > 0
        {
            height = height.min(style_height as usize);
        }

        let frame_width = self.style.get_horizontal_frame_size();
        let frame_height = self.style.get_vertical_frame_size();
        let content_width = width.saturating_sub(frame_width);
        let content_height = height.saturating_sub(frame_height);
        let lines = self.visible_lines();
        let contents = if content_width == 0 || content_height == 0 {
            String::new()
        } else {
            let content_style = Style::new()
                .width(as_u16(content_width))
                .height(as_u16(content_height))
                .max_width(as_u16(content_width))
                .max_height(as_u16(content_height));
            content_style.render(&lines.join("\n"))
        };

        self.style.render(&contents)
    }

    fn content_width(&self) -> usize {
        self.width
            .saturating_sub(self.style.get_horizontal_frame_size())
    }

    fn content_height(&self) -> usize {
        self.height
            .saturating_sub(self.style.get_vertical_frame_size())
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

fn as_u16(value: usize) -> u16 {
    value.min(u16::MAX as usize) as u16
}

fn visible_width(s: &str) -> usize {
    let mut width = 0;
    let mut in_escape = false;

    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
            continue;
        }
        if in_escape {
            if c == 'm' {
                in_escape = false;
            }
            continue;
        }
        width += UnicodeWidthChar::width(c).unwrap_or(0);
    }

    width
}

fn cut_line(line: &str, start: usize, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let end = start.saturating_add(width);
    let mut result = String::new();
    let mut in_escape = false;
    let mut visible = 0;

    for c in line.chars() {
        if c == '\x1b' {
            in_escape = true;
            result.push(c);
            continue;
        }
        if in_escape {
            result.push(c);
            if c == 'm' {
                in_escape = false;
            }
            continue;
        }

        let cw = UnicodeWidthChar::width(c).unwrap_or(0);
        if visible + cw <= start {
            visible += cw;
            continue;
        }
        if visible >= end {
            break;
        }
        result.push(c);
        visible += cw;
    }

    result
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
    fn test_viewport_view_pads_to_dimensions() {
        let mut v = Viewport::new(4, 2);
        v.set_content("a");
        assert_eq!(v.view(), "a   \n    ");
    }

    #[test]
    fn test_viewport_frame_affects_visible_height() {
        let mut v = Viewport::new(10, 5);
        v.style = Style::new().padding(1);
        v.set_content("1\n2\n3\n4\n5\n6");
        assert_eq!(v.visible_line_count(), 3);

        v.goto_bottom();
        assert_eq!(v.y_offset(), 3);
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
    fn test_viewport_horizontal_scroll_uses_display_width() {
        let mut v = Viewport::new(4, 1);
        v.set_content("日本語abc");
        v.set_x_offset(2);
        assert_eq!(v.view(), "本語");
    }

    #[test]
    fn test_viewport_mouse_wheel_shift_scrolls_horizontal() {
        let mut v = Viewport::new(10, 2);
        v.set_content("This is a very long line that exceeds the width");
        v.set_horizontal_step(2);

        let down_shift = MouseMsg {
            button: bubbletea::MouseButton::WheelDown,
            shift: true,
            ..MouseMsg::default()
        };
        v.update(&Message::new(down_shift));
        assert_eq!(v.x_offset(), 2);

        let up_shift = MouseMsg {
            button: bubbletea::MouseButton::WheelUp,
            shift: true,
            ..MouseMsg::default()
        };
        v.update(&Message::new(up_shift));
        assert_eq!(v.x_offset(), 0);
    }

    #[test]
    fn test_viewport_mouse_wheel_ignores_release() {
        let mut v = Viewport::new(10, 2);
        v.set_content("1\n2\n3\n4");

        let release = MouseMsg {
            button: bubbletea::MouseButton::WheelDown,
            action: bubbletea::MouseAction::Release,
            ..MouseMsg::default()
        };
        v.update(&Message::new(release));
        assert_eq!(v.y_offset(), 0);
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
