//! Loading state components for demo_showcase.
//!
//! This module provides loading indicators, skeleton placeholders, and
//! loading overlays that integrate with the animation subsystem and
//! respect reduce-motion preferences.
//!
//! # Components
//!
//! - [`LoadingSpinner`] - Animated spinner with optional label
//! - [`SkeletonLine`] - Placeholder for text content
//! - [`SkeletonBlock`] - Placeholder for block content
//! - [`LoadingOverlay`] - Centered loading message
//!
//! # Accessibility
//!
//! All components respect the `--no-animations` flag and `REDUCE_MOTION`
//! environment variable. When animations are disabled:
//! - Spinners show a static "Loading..." indicator
//! - Skeleton placeholders show a static pattern
//! - No CPU usage from animation ticks
//!
//! # Example
//!
//! ```ignore
//! use demo_showcase::components::loading::{LoadingSpinner, SkeletonLine};
//!
//! // Create a spinner with label
//! let spinner = LoadingSpinner::new("Loading docs...");
//!
//! // In update:
//! if let Some(cmd) = spinner.update(msg, animations_enabled) {
//!     return Some(cmd);
//! }
//!
//! // In view:
//! let loading_view = spinner.view(&theme);
//! ```

use bubbles::spinner::{spinners, SpinnerModel, TickMsg};
use bubbletea::{Cmd, Message};

use crate::theme::Theme;

/// Spinner style variants for different contexts.
#[derive(Debug, Clone, Copy, Default)]
pub enum SpinnerStyle {
    /// Default dot spinner - good for general loading.
    #[default]
    Dot,
    /// Line spinner - ASCII-safe fallback.
    Line,
    /// Pulse spinner - for subtle loading indication.
    Pulse,
    /// Points spinner - for progress-like loading.
    Points,
    /// Mini dot spinner - compact inline loading.
    MiniDot,
}

impl SpinnerStyle {
    /// Get the bubbles spinner definition for this style.
    fn to_spinner(self) -> bubbles::spinner::Spinner {
        match self {
            Self::Dot => spinners::dot(),
            Self::Line => spinners::line(),
            Self::Pulse => spinners::pulse(),
            Self::Points => spinners::points(),
            Self::MiniDot => spinners::mini_dot(),
        }
    }

    /// Get a static character to show when animations are disabled.
    fn static_char(self) -> &'static str {
        match self {
            Self::Dot | Self::MiniDot => "...",
            Self::Line => "-",
            Self::Pulse => "*",
            Self::Points => "...",
        }
    }
}

/// A themed loading spinner with optional label.
#[derive(Debug, Clone)]
pub struct LoadingSpinner {
    /// The underlying spinner model.
    spinner: SpinnerModel,
    /// Optional label to show next to the spinner.
    label: Option<String>,
    /// Whether the spinner has been started.
    started: bool,
    /// Spinner style variant.
    style: SpinnerStyle,
}

impl Default for LoadingSpinner {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadingSpinner {
    /// Create a new loading spinner with default style.
    #[must_use]
    pub fn new() -> Self {
        Self {
            spinner: SpinnerModel::with_spinner(spinners::dot()),
            label: None,
            started: false,
            style: SpinnerStyle::Dot,
        }
    }

    /// Create a new loading spinner with a label.
    #[must_use]
    pub fn with_label(label: impl Into<String>) -> Self {
        Self {
            spinner: SpinnerModel::with_spinner(spinners::dot()),
            label: Some(label.into()),
            started: false,
            style: SpinnerStyle::Dot,
        }
    }

    /// Set the spinner style.
    #[must_use]
    pub fn style(mut self, style: SpinnerStyle) -> Self {
        self.style = style;
        self.spinner = SpinnerModel::with_spinner(style.to_spinner());
        self
    }

    /// Set the label.
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = Some(label.into());
    }

    /// Clear the label.
    pub fn clear_label(&mut self) {
        self.label = None;
    }

    /// Get the spinner ID for message routing.
    #[must_use]
    pub fn id(&self) -> u64 {
        self.spinner.id()
    }

    /// Start the spinner animation.
    ///
    /// Returns a tick command if animations are enabled.
    pub fn start(&mut self, animations_enabled: bool) -> Option<Cmd> {
        self.started = true;
        if animations_enabled {
            // Return the initial tick message wrapped in a command
            let tick = self.spinner.tick();
            Some(Cmd::new(move || tick))
        } else {
            None
        }
    }

    /// Stop the spinner.
    pub fn stop(&mut self) {
        self.started = false;
    }

    /// Check if the spinner is running.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.started
    }

    /// Update the spinner state.
    ///
    /// Returns a command to schedule the next tick if active.
    pub fn update(&mut self, msg: Message, animations_enabled: bool) -> Option<Cmd> {
        if !self.started || !animations_enabled {
            return None;
        }

        // Only handle tick messages for this spinner
        if let Some(tick) = msg.downcast_ref::<TickMsg>() {
            if tick.id != self.spinner.id() {
                return None;
            }
        }

        self.spinner.update(msg)
    }

    /// Render the spinner.
    #[must_use]
    pub fn view(&self, theme: &Theme, animations_enabled: bool) -> String {
        let spinner_text = if animations_enabled && self.started {
            self.spinner.view()
        } else {
            self.style.static_char().to_string()
        };

        let spinner_styled = theme.info_style().render(&spinner_text);

        if let Some(ref label) = self.label {
            let label_styled = theme.muted_style().render(label);
            format!("{spinner_styled} {label_styled}")
        } else {
            spinner_styled
        }
    }
}

/// A skeleton placeholder line for loading content.
#[derive(Debug, Clone)]
pub struct SkeletonLine {
    /// Width of the skeleton line in characters.
    width: usize,
    /// Whether to use a pulsing animation.
    animate: bool,
    /// Animation frame (0-3 for pulse effect).
    frame: usize,
}

impl Default for SkeletonLine {
    fn default() -> Self {
        Self::new(20)
    }
}

impl SkeletonLine {
    /// Create a new skeleton line with the given width.
    #[must_use]
    pub fn new(width: usize) -> Self {
        Self {
            width,
            animate: true,
            frame: 0,
        }
    }

    /// Set whether to animate the skeleton.
    #[must_use]
    pub fn animated(mut self, animate: bool) -> Self {
        self.animate = animate;
        self
    }

    /// Advance the animation frame.
    pub fn tick(&mut self) {
        if self.animate {
            self.frame = (self.frame + 1) % 4;
        }
    }

    /// Render the skeleton line.
    #[must_use]
    pub fn view(&self, theme: &Theme, animations_enabled: bool) -> String {
        let char = if animations_enabled && self.animate {
            match self.frame {
                0 => "░",
                1 => "▒",
                2 => "▓",
                _ => "▒",
            }
        } else {
            "░"
        };

        let line = char.repeat(self.width);
        theme.muted_style().render(&line)
    }
}

/// A skeleton placeholder block for loading larger content.
#[derive(Debug, Clone)]
pub struct SkeletonBlock {
    /// Width of each line.
    width: usize,
    /// Number of lines.
    lines: usize,
    /// Line widths (randomized for natural look).
    line_widths: Vec<usize>,
}

impl Default for SkeletonBlock {
    fn default() -> Self {
        Self::new(40, 5)
    }
}

impl SkeletonBlock {
    /// Create a new skeleton block.
    #[must_use]
    pub fn new(width: usize, lines: usize) -> Self {
        // Generate varied line widths for natural appearance
        let line_widths: Vec<usize> = (0..lines)
            .map(|i| {
                // Vary width by line index to create visual interest
                let variation = (i * 7) % 30; // Deterministic "randomness"
                width.saturating_sub(variation).max(width / 2)
            })
            .collect();

        Self {
            width,
            lines,
            line_widths,
        }
    }

    /// Create a skeleton block with a seed for deterministic variation.
    #[must_use]
    pub fn with_seed(width: usize, lines: usize, seed: u64) -> Self {
        let line_widths: Vec<usize> = (0..lines)
            .map(|i| {
                let variation = ((i as u64).wrapping_mul(seed) % 30) as usize;
                width.saturating_sub(variation).max(width / 2)
            })
            .collect();

        Self {
            width,
            lines,
            line_widths,
        }
    }

    /// Render the skeleton block.
    #[must_use]
    pub fn view(&self, theme: &Theme) -> String {
        self.line_widths
            .iter()
            .map(|&w| {
                let line = "░".repeat(w);
                theme.muted_style().render(&line)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// A centered loading overlay with spinner and message.
#[derive(Debug, Clone)]
pub struct LoadingOverlay {
    /// The spinner.
    spinner: LoadingSpinner,
    /// Main message.
    message: String,
    /// Optional sub-message.
    sub_message: Option<String>,
}

impl LoadingOverlay {
    /// Create a new loading overlay.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            spinner: LoadingSpinner::new().style(SpinnerStyle::Dot),
            message: message.into(),
            sub_message: None,
        }
    }

    /// Set a sub-message.
    #[must_use]
    pub fn with_sub_message(mut self, sub_message: impl Into<String>) -> Self {
        self.sub_message = Some(sub_message.into());
        self
    }

    /// Start the overlay animation.
    pub fn start(&mut self, animations_enabled: bool) -> Option<Cmd> {
        self.spinner.start(animations_enabled)
    }

    /// Stop the overlay animation.
    pub fn stop(&mut self) {
        self.spinner.stop();
    }

    /// Update the overlay state.
    pub fn update(&mut self, msg: Message, animations_enabled: bool) -> Option<Cmd> {
        self.spinner.update(msg, animations_enabled)
    }

    /// Render the overlay centered on screen.
    #[must_use]
    pub fn view(&self, theme: &Theme, animations_enabled: bool, width: usize, height: usize) -> String {
        let spinner_view = self.spinner.view(theme, animations_enabled);
        let message_styled = theme.heading_style().render(&self.message);

        let content = if let Some(ref sub) = self.sub_message {
            let sub_styled = theme.muted_style().render(sub);
            format!("{spinner_view}\n\n{message_styled}\n{sub_styled}")
        } else {
            format!("{spinner_view}\n\n{message_styled}")
        };

        // Center the content
        let content_lines: Vec<&str> = content.lines().collect();
        let content_height = content_lines.len();
        let max_line_width = content_lines
            .iter()
            .map(|l| lipgloss::visible_width(l))
            .max()
            .unwrap_or(0);

        let top_padding = height.saturating_sub(content_height) / 2;
        let left_padding = width.saturating_sub(max_line_width) / 2;

        let mut lines = Vec::with_capacity(height);

        // Top padding
        for _ in 0..top_padding {
            lines.push(String::new());
        }

        // Content with left padding
        let left_pad = " ".repeat(left_padding);
        for line in content_lines {
            lines.push(format!("{left_pad}{line}"));
        }

        // Bottom padding
        while lines.len() < height {
            lines.push(String::new());
        }

        lines.join("\n")
    }
}

/// Pulsing indicator for warning/attention states.
#[derive(Debug, Clone)]
pub struct PulsingIndicator {
    /// The text to pulse.
    text: String,
    /// Current pulse frame (0-3).
    frame: usize,
    /// Whether pulsing is active.
    active: bool,
}

impl PulsingIndicator {
    /// Create a new pulsing indicator.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            frame: 0,
            active: true,
        }
    }

    /// Start pulsing.
    pub fn start(&mut self) {
        self.active = true;
    }

    /// Stop pulsing.
    pub fn stop(&mut self) {
        self.active = false;
        self.frame = 0;
    }

    /// Advance the pulse frame.
    pub fn tick(&mut self) {
        if self.active {
            self.frame = (self.frame + 1) % 4;
        }
    }

    /// Render the pulsing indicator.
    #[must_use]
    pub fn view(&self, theme: &Theme, animations_enabled: bool) -> String {
        if !animations_enabled || !self.active {
            // Static warning style when not animating
            return theme.warning_style().render(&self.text);
        }

        // Pulse between warning and dimmed warning
        let style = match self.frame {
            0 => theme.warning_style(),
            1 => theme.warning_style().bold(),
            2 => theme.warning_style(),
            _ => theme.muted_style(),
        };

        style.render(&self.text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_theme() -> Theme {
        Theme::dark()
    }

    #[test]
    fn loading_spinner_creates() {
        let spinner = LoadingSpinner::new();
        assert!(!spinner.is_running());
        assert!(spinner.id() > 0);
    }

    #[test]
    fn loading_spinner_with_label() {
        let spinner = LoadingSpinner::with_label("Loading...");
        assert!(spinner.label.is_some());
    }

    #[test]
    fn loading_spinner_start_stop() {
        let mut spinner = LoadingSpinner::new();
        assert!(!spinner.is_running());

        spinner.start(false);
        assert!(spinner.is_running());

        spinner.stop();
        assert!(!spinner.is_running());
    }

    #[test]
    fn loading_spinner_view_with_animations() {
        let mut spinner = LoadingSpinner::new();
        spinner.start(true);
        let theme = test_theme();
        let view = spinner.view(&theme, true);
        assert!(!view.is_empty());
    }

    #[test]
    fn loading_spinner_view_without_animations() {
        let mut spinner = LoadingSpinner::new();
        spinner.start(false);
        let theme = test_theme();
        let view = spinner.view(&theme, false);
        assert!(view.contains("...")); // Static fallback
    }

    #[test]
    fn loading_spinner_styles() {
        let theme = test_theme();

        for style in [
            SpinnerStyle::Dot,
            SpinnerStyle::Line,
            SpinnerStyle::Pulse,
            SpinnerStyle::Points,
            SpinnerStyle::MiniDot,
        ] {
            let spinner = LoadingSpinner::new().style(style);
            let view = spinner.view(&theme, false);
            assert!(!view.is_empty());
        }
    }

    #[test]
    fn skeleton_line_creates() {
        let line = SkeletonLine::new(20);
        assert_eq!(line.width, 20);
    }

    #[test]
    fn skeleton_line_view() {
        let line = SkeletonLine::new(10);
        let theme = test_theme();
        let view = line.view(&theme, false);
        assert!(!view.is_empty());
    }

    #[test]
    fn skeleton_line_tick_advances() {
        let mut line = SkeletonLine::new(10);
        assert_eq!(line.frame, 0);

        line.tick();
        assert_eq!(line.frame, 1);

        line.tick();
        line.tick();
        line.tick();
        assert_eq!(line.frame, 0); // Should wrap
    }

    #[test]
    fn skeleton_block_creates() {
        let block = SkeletonBlock::new(40, 5);
        assert_eq!(block.lines, 5);
        assert_eq!(block.line_widths.len(), 5);
    }

    #[test]
    fn skeleton_block_view() {
        let block = SkeletonBlock::new(30, 3);
        let theme = test_theme();
        let view = block.view(&theme);
        let line_count = view.lines().count();
        assert_eq!(line_count, 3);
    }

    #[test]
    fn skeleton_block_with_seed_deterministic() {
        let block1 = SkeletonBlock::with_seed(40, 5, 42);
        let block2 = SkeletonBlock::with_seed(40, 5, 42);
        assert_eq!(block1.line_widths, block2.line_widths);

        let block3 = SkeletonBlock::with_seed(40, 5, 99);
        assert_ne!(block1.line_widths, block3.line_widths);
    }

    #[test]
    fn loading_overlay_creates() {
        let overlay = LoadingOverlay::new("Loading data...");
        assert_eq!(overlay.message, "Loading data...");
    }

    #[test]
    fn loading_overlay_with_sub_message() {
        let overlay = LoadingOverlay::new("Loading")
            .with_sub_message("This may take a moment");
        assert!(overlay.sub_message.is_some());
    }

    #[test]
    fn loading_overlay_view() {
        let overlay = LoadingOverlay::new("Loading...");
        let theme = test_theme();
        let view = overlay.view(&theme, false, 80, 24);
        assert!(view.contains("Loading"));
    }

    #[test]
    fn pulsing_indicator_creates() {
        let indicator = PulsingIndicator::new("WARNING");
        assert!(indicator.active);
    }

    #[test]
    fn pulsing_indicator_tick() {
        let mut indicator = PulsingIndicator::new("!");
        assert_eq!(indicator.frame, 0);

        indicator.tick();
        assert_eq!(indicator.frame, 1);

        indicator.tick();
        indicator.tick();
        indicator.tick();
        assert_eq!(indicator.frame, 0); // Wraps
    }

    #[test]
    fn pulsing_indicator_start_stop() {
        let mut indicator = PulsingIndicator::new("!");
        indicator.stop();
        assert!(!indicator.active);
        assert_eq!(indicator.frame, 0);

        indicator.tick(); // Should not advance when stopped
        assert_eq!(indicator.frame, 0);

        indicator.start();
        indicator.tick();
        assert_eq!(indicator.frame, 1);
    }

    #[test]
    fn pulsing_indicator_view() {
        let indicator = PulsingIndicator::new("ALERT");
        let theme = test_theme();

        let view_animated = indicator.view(&theme, true);
        let view_static = indicator.view(&theme, false);

        assert!(view_animated.contains("ALERT"));
        assert!(view_static.contains("ALERT"));
    }
}
