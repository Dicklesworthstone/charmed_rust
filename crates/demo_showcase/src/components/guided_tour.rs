//! Guided tour component for demo_showcase.
//!
//! A step-by-step walkthrough that showcases the capabilities of the
//! Charm stack (bubbletea, lipgloss, bubbles, glamour, huh).
//!
//! # Features
//! - Sequential tour steps with navigation (next/prev/skip)
//! - Auto-navigation to relevant pages for each step
//! - Highlights features and explains what each page demonstrates
//!
//! # Example
//!
//! ```ignore
//! let mut tour = GuidedTour::new();
//! tour.start();
//!
//! // In update:
//! if let Some(cmd) = tour.update(msg) {
//!     return Some(cmd);
//! }
//!
//! // In view:
//! let tour_overlay = tour.view(&theme, screen_width, screen_height);
//! ```

use bubbletea::{Cmd, KeyMsg, KeyType, Message, batch};
use lipgloss::Style;

use crate::messages::{AppMsg, Page};
use crate::theme::Theme;

/// A tour step with information about what to show.
#[derive(Debug, Clone)]
pub struct TourStep {
    /// Step title.
    pub title: &'static str,
    /// Description of what this step demonstrates.
    pub description: &'static str,
    /// The page this step is on.
    pub page: Page,
    /// The Charm library/feature this highlights.
    pub feature: &'static str,
    /// Additional tips or details.
    pub tips: &'static [&'static str],
}

/// Predefined tour steps that walk through the demo.
const TOUR_STEPS: &[TourStep] = &[
    TourStep {
        title: "Welcome to the Charm Stack Demo",
        description: "This guided tour will walk you through the powerful Rust TUI libraries ported from Go's Charm ecosystem. Let's explore!",
        page: Page::Dashboard,
        feature: "Overview",
        tips: &[
            "Press 'n' or Right/Enter to move to the next step",
            "Press 'p' or Left to go back",
            "Press 'q' or Esc to exit the tour",
        ],
    },
    TourStep {
        title: "Dashboard - Real-time Metrics",
        description: "The Dashboard demonstrates bubbletea's reactive update loop. Metrics update in real-time as the simulation runs, showing CPU, memory, and service health.",
        page: Page::Dashboard,
        feature: "bubbletea",
        tips: &[
            "Watch the sparklines animate as data flows",
            "Status indicators update based on thresholds",
            "All rendering is done with lipgloss styles",
        ],
    },
    TourStep {
        title: "Dashboard - Styled Cards",
        description: "Each metric card uses lipgloss for consistent styling. Notice the borders, padding, colors, and layout - all declarative and composable.",
        page: Page::Dashboard,
        feature: "lipgloss",
        tips: &[
            "Cards adapt to terminal width",
            "Colors respect the current theme",
            "Borders use Unicode box-drawing characters",
        ],
    },
    TourStep {
        title: "Jobs - Async Task Management",
        description: "The Jobs page shows bubbletea's async command system. Background tasks run concurrently, with progress updates flowing through the message loop.",
        page: Page::Jobs,
        feature: "bubbletea (async)",
        tips: &[
            "Press Enter to start/cancel jobs",
            "Progress bars update in real-time",
            "Multiple jobs can run concurrently",
        ],
    },
    TourStep {
        title: "Jobs - Table Component",
        description: "The job list uses the bubbles Table component for column-aligned, scrollable data. It handles keyboard navigation and selection out of the box.",
        page: Page::Jobs,
        feature: "bubbles (Table)",
        tips: &[
            "Use j/k or arrows to navigate",
            "The table auto-sizes columns",
            "Selection state is tracked automatically",
        ],
    },
    TourStep {
        title: "Logs - Live Streaming",
        description: "The Logs page demonstrates real-time streaming with the bubbles Viewport. New lines appear at the bottom while maintaining scroll position.",
        page: Page::Logs,
        feature: "bubbles (Viewport)",
        tips: &[
            "Press 'f' to toggle follow mode",
            "Use Page Up/Down for fast scrolling",
            "Logs are color-coded by severity",
        ],
    },
    TourStep {
        title: "Logs - Search & Filter",
        description: "Filtering uses the bubbles TextInput component. Type to filter logs in real-time - the viewport updates instantly as you type.",
        page: Page::Logs,
        feature: "bubbles (TextInput)",
        tips: &[
            "Press '/' to focus the filter",
            "Matching text is highlighted",
            "Press Esc to clear the filter",
        ],
    },
    TourStep {
        title: "Docs - Markdown Rendering",
        description: "The Docs page uses glamour to render styled Markdown. Headers, code blocks, links, and lists are all beautifully formatted.",
        page: Page::Docs,
        feature: "glamour",
        tips: &[
            "Code blocks have syntax highlighting",
            "Links are underlined and colored",
            "The content reflows on resize",
        ],
    },
    TourStep {
        title: "Files - File Browser",
        description: "The Files page integrates the bubbles FilePicker component. Navigate directories, preview files, and see how Bubble Tea handles file system interaction.",
        page: Page::Files,
        feature: "bubbles (FilePicker)",
        tips: &[
            "Use arrows to navigate the tree",
            "Enter opens files or directories",
            "Preview pane shows file contents",
        ],
    },
    TourStep {
        title: "Wizard - Form Builder",
        description: "The Wizard uses huh for building interactive forms. Text inputs, selects, confirms, and multi-step flows - all with validation.",
        page: Page::Wizard,
        feature: "huh",
        tips: &[
            "Tab between form fields",
            "Some fields have validation",
            "Watch the progress indicator",
        ],
    },
    TourStep {
        title: "Settings - Theme Switching",
        description: "Settings demonstrates dynamic theming. Switch between dark and light themes instantly - all components respect the theme tokens.",
        page: Page::Settings,
        feature: "lipgloss (theming)",
        tips: &[
            "Toggle between dark and light modes",
            "All pages update immediately",
            "Theme colors are semantic tokens",
        ],
    },
    TourStep {
        title: "Global Features",
        description: "Try these features anywhere: '?' for help overlay, '/' for command palette, '`' (backtick) to restart this tour. The sidebar (Tab) provides navigation.",
        page: Page::Dashboard,
        feature: "bubbletea (routing)",
        tips: &[
            "Number keys 1-8 jump to pages",
            "Mouse clicks work on navigation",
            "Press Ctrl+C to quit",
        ],
    },
    TourStep {
        title: "Tour Complete!",
        description: "You've seen the full Charm stack in action. This demo showcases what's possible when Go's beloved TUI libraries are reimplemented in Rust.",
        page: Page::Dashboard,
        feature: "The Full Stack",
        tips: &[
            "Explore any page at your own pace",
            "Press '`' (backtick) to restart the tour",
            "Check out the source code!",
        ],
    },
];

/// Messages emitted by the GuidedTour.
#[derive(Debug, Clone)]
pub enum GuidedTourMsg {
    /// Tour was started.
    Started,
    /// Tour advanced to a new step.
    StepChanged { step: usize, page: Page },
    /// Tour was completed.
    Completed,
    /// Tour was cancelled.
    Cancelled,
}

impl GuidedTourMsg {
    /// Convert to a bubbletea Message.
    #[must_use]
    pub fn into_message(self) -> Message {
        Message::new(self)
    }
}

/// Guided tour state.
#[derive(Debug, Clone)]
pub struct GuidedTour {
    /// Current step index.
    current_step: usize,
    /// Whether the tour is currently active.
    active: bool,
}

impl Default for GuidedTour {
    fn default() -> Self {
        Self::new()
    }
}

impl GuidedTour {
    /// Create a new guided tour.
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_step: 0,
            active: false,
        }
    }

    /// Start the tour from the beginning.
    pub fn start(&mut self) -> Option<Cmd> {
        self.current_step = 0;
        self.active = true;
        self.emit_step_changed()
    }

    /// Stop/cancel the tour.
    pub fn stop(&mut self) {
        self.active = false;
    }

    /// Check if the tour is currently active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active
    }

    /// Get the current step.
    #[must_use]
    pub fn current(&self) -> Option<&TourStep> {
        if self.active {
            TOUR_STEPS.get(self.current_step)
        } else {
            None
        }
    }

    /// Get the current step index (0-based).
    #[must_use]
    pub const fn step_index(&self) -> usize {
        self.current_step
    }

    /// Get the total number of steps.
    #[must_use]
    pub const fn total_steps(&self) -> usize {
        TOUR_STEPS.len()
    }

    /// Move to the next step.
    fn next_step(&mut self) -> Option<Cmd> {
        if self.current_step + 1 < TOUR_STEPS.len() {
            self.current_step += 1;
            self.emit_step_changed()
        } else {
            // Tour complete
            self.active = false;
            Some(Cmd::new(|| GuidedTourMsg::Completed.into_message()))
        }
    }

    /// Move to the previous step.
    fn prev_step(&mut self) -> Option<Cmd> {
        if self.current_step > 0 {
            self.current_step -= 1;
            self.emit_step_changed()
        } else {
            None
        }
    }

    /// Emit a step changed message with navigation command.
    fn emit_step_changed(&self) -> Option<Cmd> {
        if let Some(step) = TOUR_STEPS.get(self.current_step) {
            let page = step.page;
            let step_idx = self.current_step;
            batch(vec![
                Some(Cmd::new(move || AppMsg::Navigate(page).into_message())),
                Some(Cmd::new(move || {
                    GuidedTourMsg::StepChanged {
                        step: step_idx,
                        page,
                    }
                    .into_message()
                })),
            ])
        } else {
            None
        }
    }

    /// Handle input when tour is active.
    ///
    /// Returns a command if navigation occurred.
    pub fn update(&mut self, msg: &Message) -> Option<Cmd> {
        if !self.active {
            return None;
        }

        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                // Next step
                KeyType::Enter | KeyType::Right => {
                    return self.next_step();
                }
                // Previous step
                KeyType::Left => {
                    return self.prev_step();
                }
                // Cancel tour
                KeyType::Esc => {
                    self.stop();
                    return Some(Cmd::new(|| GuidedTourMsg::Cancelled.into_message()));
                }
                KeyType::Runes => {
                    let runes = &key.runes;
                    match runes.as_slice() {
                        // Next step
                        ['n'] | [' '] => {
                            return self.next_step();
                        }
                        // Previous step
                        ['p'] => {
                            return self.prev_step();
                        }
                        // Cancel tour
                        ['q'] => {
                            self.stop();
                            return Some(Cmd::new(|| GuidedTourMsg::Cancelled.into_message()));
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        None
    }

    /// Render the tour overlay.
    #[must_use]
    pub fn view(&self, theme: &Theme, screen_width: usize, screen_height: usize) -> String {
        if !self.active {
            return String::new();
        }

        let step = match self.current() {
            Some(s) => s,
            None => return String::new(),
        };

        // Modal dimensions
        let modal_width = 60.min(screen_width.saturating_sub(4));
        let modal_height = 16.min(screen_height.saturating_sub(4));

        // Calculate centering
        let left_pad = screen_width.saturating_sub(modal_width) / 2;
        let top_pad = screen_height.saturating_sub(modal_height) / 2;

        let mut lines: Vec<String> = Vec::with_capacity(screen_height);

        // Top padding
        for _ in 0..top_pad {
            lines.push(String::new());
        }

        let indent = " ".repeat(left_pad);
        let content_width = modal_width.saturating_sub(4);

        // Styles
        let border_style = theme.info_style();
        let title_style = theme.heading_style().bold();
        let feature_style = theme.badge_primary_style();
        let text_style = Style::new().foreground(theme.text);
        let tip_style = theme.muted_style();
        let progress_style = theme.success_style();

        // Top border with title
        let title = format!(" {} ", step.title);
        let title_display = if title.len() > modal_width - 4 {
            format!("{}...", &title[..modal_width - 7])
        } else {
            title
        };
        let title_pad_left = (modal_width - 2 - title_display.len()) / 2;
        let title_pad_right = modal_width - 2 - title_pad_left - title_display.len();
        lines.push(format!(
            "{}{}",
            indent,
            border_style.render(&format!(
                "{}{}{}{}{}",
                "",
                "".repeat(title_pad_left),
                title_style.render(&title_display),
                "".repeat(title_pad_right),
                ""
            ))
        ));

        // Feature badge
        let feature_line = format!("[{}]", step.feature);
        let feature_padded = format!("{:^width$}", feature_line, width = content_width);
        lines.push(format!(
            "{}{}{}{}",
            indent,
            border_style.render(""),
            feature_style.render(&feature_padded),
            border_style.render("")
        ));

        // Empty line
        let empty = " ".repeat(content_width);
        lines.push(format!(
            "{}{}{}{}",
            indent,
            border_style.render(""),
            text_style.render(&empty),
            border_style.render("")
        ));

        // Description - word wrap
        let desc_lines = word_wrap(step.description, content_width);
        for desc_line in desc_lines.iter().take(4) {
            let padded = format!("{:width$}", desc_line, width = content_width);
            lines.push(format!(
                "{}{}{}{}",
                indent,
                border_style.render(""),
                text_style.render(&padded),
                border_style.render("")
            ));
        }

        // Pad to fill space
        let used = 3 + desc_lines.len().min(4);
        let tips_space = 3; // Reserve for tips
        let footer_space = 2; // Progress + nav hints
        let remaining = modal_height.saturating_sub(used + tips_space + footer_space);
        for _ in 0..remaining {
            lines.push(format!(
                "{}{}{}{}",
                indent,
                border_style.render(""),
                text_style.render(&empty),
                border_style.render("")
            ));
        }

        // Tips section
        lines.push(format!(
            "{}{}{}{}",
            indent,
            border_style.render(""),
            tip_style.render(&format!("{:width$}", "Tips:", width = content_width)),
            border_style.render("")
        ));
        for tip in step.tips.iter().take(2) {
            let tip_text = format!("  {}", tip);
            let tip_padded = if tip_text.len() > content_width {
                format!("{}...", &tip_text[..content_width - 3])
            } else {
                format!("{:width$}", tip_text, width = content_width)
            };
            lines.push(format!(
                "{}{}{}{}",
                indent,
                border_style.render(""),
                tip_style.render(&tip_padded),
                border_style.render("")
            ));
        }

        // Progress bar
        let progress = format!(
            "Step {} of {}",
            self.current_step + 1,
            TOUR_STEPS.len()
        );
        let bar_width = modal_width.saturating_sub(progress.len() + 8);
        let filled = (self.current_step * bar_width) / TOUR_STEPS.len().max(1);
        let empty_bar = bar_width.saturating_sub(filled);
        let progress_bar = format!(
            "[{}{}] {}",
            "".repeat(filled),
            "".repeat(empty_bar),
            progress
        );
        let progress_padded = format!("{:^width$}", progress_bar, width = content_width);
        lines.push(format!(
            "{}{}{}{}",
            indent,
            border_style.render(""),
            progress_style.render(&progress_padded),
            border_style.render("")
        ));

        // Navigation hints
        let nav_hints = "n/Enter next    p/Left back    q/Esc exit";
        let nav_padded = format!("{:^width$}", nav_hints, width = content_width);
        lines.push(format!(
            "{}{}{}{}",
            indent,
            border_style.render(""),
            tip_style.render(&nav_padded),
            border_style.render("")
        ));

        // Bottom border
        lines.push(format!(
            "{}{}",
            indent,
            border_style.render(&format!("{}{}{}", "", "".repeat(modal_width - 2), ""))
        ));

        // Bottom padding
        while lines.len() < screen_height {
            lines.push(String::new());
        }

        lines.join("\n")
    }
}

/// Simple word wrap implementation.
fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tour_starts_inactive() {
        let tour = GuidedTour::new();
        assert!(!tour.is_active());
        assert!(tour.current().is_none());
    }

    #[test]
    fn tour_start_activates() {
        let mut tour = GuidedTour::new();
        let cmd = tour.start();
        assert!(tour.is_active());
        assert!(tour.current().is_some());
        assert!(cmd.is_some());
    }

    #[test]
    fn tour_stop_deactivates() {
        let mut tour = GuidedTour::new();
        tour.start();
        assert!(tour.is_active());
        tour.stop();
        assert!(!tour.is_active());
    }

    #[test]
    fn tour_starts_at_first_step() {
        let mut tour = GuidedTour::new();
        tour.start();
        assert_eq!(tour.step_index(), 0);
    }

    #[test]
    fn tour_has_multiple_steps() {
        let tour = GuidedTour::new();
        assert!(tour.total_steps() > 5);
    }

    #[test]
    fn tour_step_has_required_fields() {
        for step in TOUR_STEPS {
            assert!(!step.title.is_empty());
            assert!(!step.description.is_empty());
            assert!(!step.feature.is_empty());
        }
    }

    #[test]
    fn tour_next_step_advances() {
        let mut tour = GuidedTour::new();
        tour.start();
        assert_eq!(tour.step_index(), 0);

        let key = KeyMsg::from_char('n');
        let cmd = tour.update(&Message::new(key));
        assert!(cmd.is_some());
        assert_eq!(tour.step_index(), 1);
    }

    #[test]
    fn tour_prev_step_goes_back() {
        let mut tour = GuidedTour::new();
        tour.start();

        // Move forward first
        let key = KeyMsg::from_char('n');
        tour.update(&Message::new(key));
        assert_eq!(tour.step_index(), 1);

        // Now go back
        let key = KeyMsg::from_char('p');
        let cmd = tour.update(&Message::new(key));
        assert!(cmd.is_some());
        assert_eq!(tour.step_index(), 0);
    }

    #[test]
    fn tour_prev_at_start_does_nothing() {
        let mut tour = GuidedTour::new();
        tour.start();
        assert_eq!(tour.step_index(), 0);

        let key = KeyMsg::from_char('p');
        let cmd = tour.update(&Message::new(key));
        assert!(cmd.is_none());
        assert_eq!(tour.step_index(), 0);
    }

    #[test]
    fn tour_q_cancels() {
        let mut tour = GuidedTour::new();
        tour.start();
        assert!(tour.is_active());

        let key = KeyMsg::from_char('q');
        let cmd = tour.update(&Message::new(key));
        assert!(cmd.is_some());
        assert!(!tour.is_active());
    }

    #[test]
    fn tour_esc_cancels() {
        let mut tour = GuidedTour::new();
        tour.start();
        assert!(tour.is_active());

        let key = KeyMsg::from_type(KeyType::Esc);
        let cmd = tour.update(&Message::new(key));
        assert!(cmd.is_some());
        assert!(!tour.is_active());
    }

    #[test]
    fn tour_enter_advances() {
        let mut tour = GuidedTour::new();
        tour.start();
        assert_eq!(tour.step_index(), 0);

        let key = KeyMsg::from_type(KeyType::Enter);
        let cmd = tour.update(&Message::new(key));
        assert!(cmd.is_some());
        assert_eq!(tour.step_index(), 1);
    }

    #[test]
    fn tour_arrow_keys_work() {
        let mut tour = GuidedTour::new();
        tour.start();

        // Right arrow advances
        let key = KeyMsg::from_type(KeyType::Right);
        tour.update(&Message::new(key));
        assert_eq!(tour.step_index(), 1);

        // Left arrow goes back
        let key = KeyMsg::from_type(KeyType::Left);
        tour.update(&Message::new(key));
        assert_eq!(tour.step_index(), 0);
    }

    #[test]
    fn tour_view_empty_when_inactive() {
        let tour = GuidedTour::new();
        let theme = Theme::dark();
        let view = tour.view(&theme, 80, 24);
        assert!(view.is_empty());
    }

    #[test]
    fn tour_view_non_empty_when_active() {
        let mut tour = GuidedTour::new();
        tour.start();
        let theme = Theme::dark();
        let view = tour.view(&theme, 80, 24);
        assert!(!view.is_empty());
    }

    #[test]
    fn tour_view_contains_step_info() {
        let mut tour = GuidedTour::new();
        tour.start();
        let theme = Theme::dark();
        let view = tour.view(&theme, 80, 24);

        // Should contain some text from the first step
        let step = tour.current().unwrap();
        // Check for feature badge
        assert!(view.contains(step.feature) || view.contains("Step 1"));
    }

    #[test]
    fn tour_completes_at_end() {
        let mut tour = GuidedTour::new();
        tour.start();

        // Advance to the end
        for _ in 0..TOUR_STEPS.len() - 1 {
            let key = KeyMsg::from_char('n');
            tour.update(&Message::new(key));
        }

        // At last step
        assert_eq!(tour.step_index(), TOUR_STEPS.len() - 1);
        assert!(tour.is_active());

        // One more advance should complete
        let key = KeyMsg::from_char('n');
        let cmd = tour.update(&Message::new(key));
        assert!(cmd.is_some());
        assert!(!tour.is_active());
    }

    #[test]
    fn word_wrap_works() {
        let text = "This is a test of the word wrap function.";
        let lines = word_wrap(text, 20);
        assert!(lines.len() > 1);
        for line in &lines {
            assert!(line.len() <= 20);
        }
    }

    #[test]
    fn word_wrap_handles_short_text() {
        let text = "Short";
        let lines = word_wrap(text, 20);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "Short");
    }

    #[test]
    fn word_wrap_handles_empty() {
        let lines = word_wrap("", 20);
        assert!(lines.is_empty());
    }

    #[test]
    fn all_steps_have_valid_pages() {
        for step in TOUR_STEPS {
            // Just verify the page is valid (compile-time check)
            let _ = step.page.name();
        }
    }
}
