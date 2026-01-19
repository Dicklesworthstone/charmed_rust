//! Progress bar component.
//!
//! This module provides a progress bar with optional gradient fill and
//! spring-based animations.
//!
//! # Example
//!
//! ```rust
//! use bubbles::progress::Progress;
//!
//! // Create a progress bar
//! let progress = Progress::new();
//!
//! // Render at a specific percentage
//! let view = progress.view_as(0.5);
//! ```

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use bubbletea::{Cmd, Message, Model};
use harmonica::Spring;
use lipgloss::Style;

const FPS: u32 = 60;
const DEFAULT_WIDTH: usize = 40;
const DEFAULT_FREQUENCY: f64 = 18.0;
const DEFAULT_DAMPING: f64 = 1.0;

/// Global ID counter for progress instances.
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

/// Message indicating that an animation frame should occur.
#[derive(Debug, Clone, Copy)]
pub struct FrameMsg {
    /// The progress bar ID.
    pub id: u64,
    /// Tag for message ordering.
    tag: u64,
}

/// Progress bar gradient configuration.
#[derive(Debug, Clone)]
pub struct Gradient {
    /// Start color (hex).
    pub color_a: String,
    /// End color (hex).
    pub color_b: String,
    /// Whether to scale the gradient to the filled portion.
    pub scaled: bool,
}

impl Default for Gradient {
    fn default() -> Self {
        Self {
            color_a: "#5A56E0".to_string(),
            color_b: "#EE6FF8".to_string(),
            scaled: false,
        }
    }
}

/// Progress bar model.
#[derive(Debug, Clone)]
pub struct Progress {
    /// Unique identifier.
    id: u64,
    /// Tag for frame message ordering.
    tag: u64,
    /// Total width of the progress bar.
    pub width: usize,
    /// Character for filled sections.
    pub full_char: char,
    /// Color for filled sections (when not using gradient).
    pub full_color: String,
    /// Character for empty sections.
    pub empty_char: char,
    /// Color for empty sections.
    pub empty_color: String,
    /// Whether to show percentage text.
    pub show_percentage: bool,
    /// Format string for percentage.
    pub percent_format: String,
    /// Style for percentage text.
    pub percentage_style: Style,
    /// Spring for animations.
    spring: Spring,
    /// Currently displayed percentage (for animation).
    percent_shown: f64,
    /// Target percentage (for animation).
    target_percent: f64,
    /// Animation velocity.
    velocity: f64,
    /// Gradient configuration (if using gradient).
    gradient: Option<Gradient>,
}

impl Default for Progress {
    fn default() -> Self {
        Self::new()
    }
}

impl Progress {
    /// Creates a new progress bar with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: next_id(),
            tag: 0,
            width: DEFAULT_WIDTH,
            full_char: '█',
            full_color: "#7571F9".to_string(),
            empty_char: '░',
            empty_color: "#606060".to_string(),
            show_percentage: true,
            percent_format: " {:3.0}%".to_string(),
            percentage_style: Style::new(),
            spring: Spring::new(FPS as f64, DEFAULT_FREQUENCY, DEFAULT_DAMPING),
            percent_shown: 0.0,
            target_percent: 0.0,
            velocity: 0.0,
            gradient: None,
        }
    }

    /// Creates a progress bar with default gradient colors.
    #[must_use]
    pub fn with_gradient() -> Self {
        let mut p = Self::new();
        p.gradient = Some(Gradient::default());
        p
    }

    /// Creates a progress bar with custom gradient colors.
    #[must_use]
    pub fn with_gradient_colors(color_a: &str, color_b: &str) -> Self {
        let mut p = Self::new();
        p.gradient = Some(Gradient {
            color_a: color_a.to_string(),
            color_b: color_b.to_string(),
            scaled: false,
        });
        p
    }

    /// Creates a progress bar with a scaled gradient.
    #[must_use]
    pub fn with_scaled_gradient(color_a: &str, color_b: &str) -> Self {
        let mut p = Self::new();
        p.gradient = Some(Gradient {
            color_a: color_a.to_string(),
            color_b: color_b.to_string(),
            scaled: true,
        });
        p
    }

    /// Sets the width of the progress bar.
    #[must_use]
    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Sets the fill characters.
    #[must_use]
    pub fn fill_chars(mut self, full: char, empty: char) -> Self {
        self.full_char = full;
        self.empty_char = empty;
        self
    }

    /// Sets the solid fill color (disables gradient).
    #[must_use]
    pub fn solid_fill(mut self, color: &str) -> Self {
        self.full_color = color.to_string();
        self.gradient = None;
        self
    }

    /// Disables percentage display.
    #[must_use]
    pub fn without_percentage(mut self) -> Self {
        self.show_percentage = false;
        self
    }

    /// Sets the spring animation parameters.
    pub fn set_spring_options(&mut self, frequency: f64, damping: f64) {
        self.spring = Spring::new(FPS as f64, frequency, damping);
    }

    /// Returns the progress bar's unique ID.
    #[must_use]
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns the current target percentage.
    #[must_use]
    pub fn percent(&self) -> f64 {
        self.target_percent
    }

    /// Sets the percentage and returns a command to start animation.
    pub fn set_percent(&mut self, p: f64) -> Option<Cmd> {
        self.target_percent = p.clamp(0.0, 1.0);
        self.tag = self.tag.wrapping_add(1);
        self.next_frame()
    }

    /// Increments the percentage.
    pub fn incr_percent(&mut self, v: f64) -> Option<Cmd> {
        self.set_percent(self.percent() + v)
    }

    /// Decrements the percentage.
    pub fn decr_percent(&mut self, v: f64) -> Option<Cmd> {
        self.set_percent(self.percent() - v)
    }

    /// Returns whether the progress bar is still animating.
    #[must_use]
    pub fn is_animating(&self) -> bool {
        let dist = (self.percent_shown - self.target_percent).abs();
        !(dist < 0.001 && self.velocity < 0.01)
    }

    /// Creates a command for the next animation frame.
    fn next_frame(&self) -> Option<Cmd> {
        let id = self.id;
        let tag = self.tag;
        let delay = Duration::from_secs_f64(1.0 / f64::from(FPS));

        Some(Cmd::new(move || {
            std::thread::sleep(delay);
            Message::new(FrameMsg { id, tag })
        }))
    }

    /// Updates the progress bar state.
    pub fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(frame) = msg.downcast_ref::<FrameMsg>() {
            if frame.id != self.id || frame.tag != self.tag {
                return None;
            }

            if !self.is_animating() {
                return None;
            }

            let (new_pos, new_vel) =
                self.spring
                    .update(self.percent_shown, self.velocity, self.target_percent);
            self.percent_shown = new_pos;
            self.velocity = new_vel;

            return self.next_frame();
        }

        None
    }

    /// Renders the progress bar at the current animated position.
    #[must_use]
    pub fn view(&self) -> String {
        self.view_as(self.percent_shown)
    }

    /// Renders the progress bar at a specific percentage.
    #[must_use]
    pub fn view_as(&self, percent: f64) -> String {
        let mut result = String::new();
        let percent_view = self.percentage_view(percent);
        let percent_width = percent_view.chars().count();

        self.bar_view(&mut result, percent, percent_width);
        result.push_str(&percent_view);
        result
    }

    fn bar_view(&self, buf: &mut String, percent: f64, text_width: usize) {
        let tw = self.width.saturating_sub(text_width);
        let fw = ((tw as f64 * percent).round() as usize).min(tw);

        if let Some(ref gradient) = self.gradient {
            // Gradient fill
            for i in 0..fw {
                let p = if fw == 1 {
                    0.5
                } else if gradient.scaled {
                    i as f64 / (fw - 1) as f64
                } else {
                    i as f64 / (tw - 1).max(1) as f64
                };

                // Simple linear interpolation between colors
                let color = interpolate_color(&gradient.color_a, &gradient.color_b, p);
                buf.push_str(&format!("\x1b[38;2;{}m{}\x1b[0m", color, self.full_char));
            }
        } else {
            // Solid fill
            let colored_char = format_colored_char(self.full_char, &self.full_color).repeat(fw);
            buf.push_str(&colored_char);
        }

        // Empty fill
        let empty_colored = format_colored_char(self.empty_char, &self.empty_color);
        let empty_count = tw.saturating_sub(fw);
        for _ in 0..empty_count {
            buf.push_str(&empty_colored);
        }
    }

    fn percentage_view(&self, percent: f64) -> String {
        if !self.show_percentage {
            return String::new();
        }
        let percent = percent.clamp(0.0, 1.0) * 100.0;
        format!(" {:3.0}%", percent)
    }
}

/// Format a character with ANSI color.
fn format_colored_char(c: char, hex_color: &str) -> String {
    if let Some(rgb) = parse_hex_color(hex_color) {
        format!("\x1b[38;2;{};{};{}m{}\x1b[0m", rgb.0, rgb.1, rgb.2, c)
    } else {
        c.to_string()
    }
}

/// Parse a hex color string to RGB.
fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}

/// Interpolate between two hex colors.
fn interpolate_color(color_a: &str, color_b: &str, t: f64) -> String {
    let a = parse_hex_color(color_a).unwrap_or((0, 0, 0));
    let b = parse_hex_color(color_b).unwrap_or((0, 0, 0));

    let r = (a.0 as f64 + (b.0 as f64 - a.0 as f64) * t).round() as u8;
    let g = (a.1 as f64 + (b.1 as f64 - a.1 as f64) * t).round() as u8;
    let bl = (a.2 as f64 + (b.2 as f64 - a.2 as f64) * t).round() as u8;

    format!("{};{};{}", r, g, bl)
}

impl Model for Progress {
    /// Initialize the progress bar.
    ///
    /// Progress bars don't require initialization commands.
    fn init(&self) -> Option<Cmd> {
        None
    }

    /// Update the progress bar state based on incoming messages.
    fn update(&mut self, msg: Message) -> Option<Cmd> {
        Progress::update(self, msg)
    }

    /// Render the progress bar at the current animated position.
    fn view(&self) -> String {
        Progress::view(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_new() {
        let p = Progress::new();
        assert_eq!(p.width, DEFAULT_WIDTH);
        assert!(p.show_percentage);
        assert_eq!(p.percent(), 0.0);
    }

    #[test]
    fn test_progress_unique_ids() {
        let p1 = Progress::new();
        let p2 = Progress::new();
        assert_ne!(p1.id(), p2.id());
    }

    #[test]
    fn test_progress_set_percent() {
        let mut p = Progress::new();
        p.set_percent(0.5);
        assert!((p.percent() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_progress_percent_clamp() {
        let mut p = Progress::new();
        p.set_percent(1.5);
        assert!((p.percent() - 1.0).abs() < 0.001);

        p.set_percent(-0.5);
        assert!(p.percent().abs() < 0.001);
    }

    #[test]
    fn test_progress_view_as() {
        let p = Progress::new().width(20).without_percentage();
        let view = p.view_as(0.5);
        // Should have some filled and empty chars
        assert!(!view.is_empty());
    }

    #[test]
    fn test_progress_builder() {
        let p = Progress::new()
            .width(50)
            .fill_chars('#', '-')
            .without_percentage();

        assert_eq!(p.width, 50);
        assert_eq!(p.full_char, '#');
        assert_eq!(p.empty_char, '-');
        assert!(!p.show_percentage);
    }

    #[test]
    fn test_progress_with_gradient() {
        let p = Progress::with_gradient();
        assert!(p.gradient.is_some());
    }

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(parse_hex_color("#FF0000"), Some((255, 0, 0)));
        assert_eq!(parse_hex_color("#00FF00"), Some((0, 255, 0)));
        assert_eq!(parse_hex_color("#0000FF"), Some((0, 0, 255)));
        assert_eq!(parse_hex_color("FFFFFF"), Some((255, 255, 255)));
        assert_eq!(parse_hex_color("invalid"), None);
    }

    #[test]
    fn test_interpolate_color() {
        let mid = interpolate_color("#000000", "#FFFFFF", 0.5);
        // Should be approximately middle gray
        assert!(mid.contains("127") || mid.contains("128"));
    }

    #[test]
    fn test_progress_animation_state() {
        let mut p = Progress::new();
        p.percent_shown = 0.5;
        p.target_percent = 0.5;
        p.velocity = 0.0;
        assert!(!p.is_animating());

        p.target_percent = 0.8;
        assert!(p.is_animating());
    }

    // Model trait implementation tests
    #[test]
    fn test_model_init() {
        let p = Progress::new();
        // Progress bars don't require init commands
        let cmd = Model::init(&p);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_model_view() {
        let p = Progress::new();
        // Model::view should return same result as Progress::view
        let model_view = Model::view(&p);
        let progress_view = Progress::view(&p);
        assert_eq!(model_view, progress_view);
    }

    #[test]
    fn test_model_update_handles_frame_msg() {
        let mut p = Progress::new();
        p.target_percent = 1.0;
        p.percent_shown = 0.0;
        p.velocity = 0.0;
        let id = p.id();
        let tag = p.tag;

        // Use Model::update explicitly
        let frame_msg = Message::new(FrameMsg { id, tag });
        let cmd = Model::update(&mut p, frame_msg);

        // Should return a command for the next frame (animating)
        assert!(cmd.is_some(), "Model::update should return next frame command when animating");
        // Percent should have changed
        assert!(p.percent_shown > 0.0, "percent_shown should have advanced");
    }

    #[test]
    fn test_model_update_ignores_wrong_id() {
        let mut p = Progress::new();
        p.target_percent = 1.0;
        p.percent_shown = 0.0;
        let original_percent = p.percent_shown;

        // Send frame message with wrong ID
        let frame_msg = Message::new(FrameMsg { id: 99999, tag: 0 });
        let cmd = Model::update(&mut p, frame_msg);

        assert!(cmd.is_none(), "Should ignore messages for other progress bars");
        assert!((p.percent_shown - original_percent).abs() < 0.001, "percent_shown should not change");
    }

    #[test]
    fn test_model_update_ignores_wrong_tag() {
        let mut p = Progress::new();
        p.target_percent = 1.0;
        p.percent_shown = 0.0;
        p.tag = 5;
        let id = p.id();
        let original_percent = p.percent_shown;

        // Send frame message with wrong tag
        let frame_msg = Message::new(FrameMsg { id, tag: 3 });
        let cmd = Model::update(&mut p, frame_msg);

        assert!(cmd.is_none(), "Should ignore messages with old tag");
        assert!((p.percent_shown - original_percent).abs() < 0.001, "percent_shown should not change");
    }

    #[test]
    fn test_progress_satisfies_model_bounds() {
        // Verify Progress can be used where Model + Send + 'static is required
        fn accepts_model<M: Model + Send + 'static>(_model: M) {}
        let p = Progress::new();
        accepts_model(p);
    }
}
