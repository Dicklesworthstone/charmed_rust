//! Spinner component for loading indicators.
//!
//! This module provides animated spinners with multiple preset styles.
//!
//! # Example
//!
//! ```rust
//! use bubbles::spinner::{Spinner, SpinnerModel, spinners};
//!
//! // Create a spinner with the default style
//! let spinner = SpinnerModel::new();
//!
//! // Or with a specific style
//! let spinner = SpinnerModel::with_spinner(spinners::dot());
//!
//! // Get the tick command to start animation
//! let tick_msg = spinner.tick();
//! ```

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use bubbletea::{Cmd, Message, Model};
use lipgloss::Style;

/// Global ID counter for spinner instances.
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

/// A spinner animation definition.
#[derive(Debug, Clone)]
pub struct Spinner {
    /// The frames of the animation.
    pub frames: Vec<String>,
    /// Frames per second for the animation.
    pub fps: u32,
}

impl Spinner {
    /// Creates a new spinner with the given frames and FPS.
    #[must_use]
    pub fn new(frames: Vec<&str>, fps: u32) -> Self {
        Self {
            frames: frames.into_iter().map(String::from).collect(),
            fps,
        }
    }

    /// Returns the duration between frames.
    #[must_use]
    pub fn frame_duration(&self) -> Duration {
        if self.fps == 0 {
            Duration::from_secs(1)
        } else {
            Duration::from_secs_f64(1.0 / f64::from(self.fps))
        }
    }
}

/// Predefined spinner styles.
pub mod spinners {
    use super::Spinner;

    /// Line spinner: `| / - \`
    #[must_use]
    pub fn line() -> Spinner {
        Spinner::new(vec!["|", "/", "-", "\\"], 10)
    }

    /// Braille dot spinner.
    #[must_use]
    pub fn dot() -> Spinner {
        Spinner::new(vec!["⣾ ", "⣽ ", "⣻ ", "⢿ ", "⡿ ", "⣟ ", "⣯ ", "⣷ "], 10)
    }

    /// Mini braille dot spinner.
    #[must_use]
    pub fn mini_dot() -> Spinner {
        Spinner::new(vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"], 12)
    }

    /// Jump spinner.
    #[must_use]
    pub fn jump() -> Spinner {
        Spinner::new(vec!["⢄", "⢂", "⢁", "⡁", "⡈", "⡐", "⡠"], 10)
    }

    /// Pulse spinner.
    #[must_use]
    pub fn pulse() -> Spinner {
        Spinner::new(vec!["█", "▓", "▒", "░"], 8)
    }

    /// Points spinner.
    #[must_use]
    pub fn points() -> Spinner {
        Spinner::new(vec!["∙∙∙", "●∙∙", "∙●∙", "∙∙●"], 7)
    }

    /// Globe spinner.
    #[must_use]
    pub fn globe() -> Spinner {
        Spinner::new(vec!["🌍", "🌎", "🌏"], 4)
    }

    /// Moon phases spinner.
    #[must_use]
    pub fn moon() -> Spinner {
        Spinner::new(vec!["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"], 8)
    }

    /// Monkey spinner.
    #[must_use]
    pub fn monkey() -> Spinner {
        Spinner::new(vec!["🙈", "🙉", "🙊"], 3)
    }

    /// Meter spinner.
    #[must_use]
    pub fn meter() -> Spinner {
        Spinner::new(vec!["▱▱▱", "▰▱▱", "▰▰▱", "▰▰▰", "▰▰▱", "▰▱▱", "▱▱▱"], 7)
    }

    /// Hamburger spinner.
    #[must_use]
    pub fn hamburger() -> Spinner {
        Spinner::new(vec!["☱", "☲", "☴", "☲"], 3)
    }

    /// Ellipsis spinner.
    #[must_use]
    pub fn ellipsis() -> Spinner {
        Spinner::new(vec!["", ".", "..", "..."], 3)
    }
}

/// Message indicating that the spinner should advance to the next frame.
#[derive(Debug, Clone)]
pub struct TickMsg {
    /// The spinner ID this tick is for.
    pub id: u64,
    /// Tag for message ordering.
    tag: u64,
}

/// The spinner model.
#[derive(Debug, Clone)]
pub struct SpinnerModel {
    /// The spinner animation to use.
    pub spinner: Spinner,
    /// Style for rendering the spinner.
    pub style: Style,

    frame: usize,
    id: u64,
    tag: u64,
}

impl Default for SpinnerModel {
    fn default() -> Self {
        Self::new()
    }
}

impl SpinnerModel {
    /// Creates a new spinner with the default line style.
    #[must_use]
    pub fn new() -> Self {
        Self {
            spinner: spinners::line(),
            style: Style::new(),
            frame: 0,
            id: next_id(),
            tag: 0,
        }
    }

    /// Creates a new spinner with the given spinner style.
    #[must_use]
    pub fn with_spinner(spinner: Spinner) -> Self {
        Self {
            spinner,
            style: Style::new(),
            frame: 0,
            id: next_id(),
            tag: 0,
        }
    }

    /// Sets the spinner animation style.
    #[must_use]
    pub fn spinner(mut self, spinner: Spinner) -> Self {
        self.spinner = spinner;
        self
    }

    /// Sets the lipgloss style.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Returns the spinner's unique ID.
    #[must_use]
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Creates a tick message to start or continue the spinner animation.
    ///
    /// Use this to get the initial tick message, then the spinner will
    /// continue ticking via the command returned from `update`.
    #[must_use]
    pub fn tick(&self) -> Message {
        Message::new(TickMsg {
            id: self.id,
            tag: self.tag,
        })
    }

    /// Creates a command to tick the spinner after the appropriate delay.
    fn tick_cmd(&self) -> Cmd {
        let id = self.id;
        let tag = self.tag;
        let duration = self.spinner.frame_duration();

        Cmd::new(move || {
            std::thread::sleep(duration);
            Message::new(TickMsg { id, tag })
        })
    }

    /// Updates the spinner state.
    ///
    /// Returns a command to schedule the next tick.
    pub fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(tick) = msg.downcast_ref::<TickMsg>() {
            // Reject messages for other spinners
            if tick.id > 0 && tick.id != self.id {
                return None;
            }

            // Reject outdated tags
            if tick.tag > 0 && tick.tag != self.tag {
                return None;
            }

            // Advance frame
            self.frame += 1;
            if self.frame >= self.spinner.frames.len() {
                self.frame = 0;
            }

            // Increment tag and schedule next tick
            self.tag = self.tag.wrapping_add(1);
            return Some(self.tick_cmd());
        }

        None
    }

    /// Renders the current spinner frame.
    #[must_use]
    pub fn view(&self) -> String {
        if self.frame >= self.spinner.frames.len() {
            return "(error)".to_string();
        }

        self.style.render(&self.spinner.frames[self.frame])
    }
}

/// Implement the Model trait for standalone bubbletea usage.
impl Model for SpinnerModel {
    fn init(&self) -> Option<Cmd> {
        // Return a command to start the spinner's tick cycle
        let id = self.id;
        let tag = self.tag;
        let duration = self.spinner.frame_duration();

        Some(Cmd::new(move || {
            std::thread::sleep(duration);
            Message::new(TickMsg { id, tag })
        }))
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        SpinnerModel::update(self, msg)
    }

    fn view(&self) -> String {
        SpinnerModel::view(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_new() {
        let spinner = SpinnerModel::new();
        assert!(!spinner.spinner.frames.is_empty());
        assert!(spinner.id() > 0);
    }

    #[test]
    fn test_spinner_unique_ids() {
        let s1 = SpinnerModel::new();
        let s2 = SpinnerModel::new();
        assert_ne!(s1.id(), s2.id());
    }

    #[test]
    fn test_spinner_with_style() {
        let spinner = SpinnerModel::with_spinner(spinners::dot());
        assert_eq!(spinner.spinner.frames.len(), 8);
    }

    #[test]
    fn test_spinner_view() {
        let spinner = SpinnerModel::new();
        let view = spinner.view();
        assert!(!view.is_empty());
    }

    #[test]
    fn test_spinner_frame_advance() {
        let mut spinner = SpinnerModel::new();
        let initial_frame = spinner.frame;

        // Simulate a tick
        let tick = Message::new(TickMsg {
            id: spinner.id(),
            tag: spinner.tag,
        });
        spinner.update(tick);

        assert_eq!(spinner.frame, initial_frame + 1);
    }

    #[test]
    fn test_spinner_frame_wrap() {
        let mut spinner = SpinnerModel::with_spinner(Spinner::new(vec!["a", "b"], 10));
        spinner.frame = 1;
        spinner.tag = 0;

        let tick = Message::new(TickMsg {
            id: spinner.id(),
            tag: 0,
        });
        spinner.update(tick);

        assert_eq!(spinner.frame, 0); // Should wrap around
    }

    #[test]
    fn test_spinner_ignores_other_ids() {
        let mut spinner = SpinnerModel::new();
        let initial_frame = spinner.frame;

        // Tick with wrong ID
        let tick = Message::new(TickMsg { id: 9999, tag: 0 });
        spinner.update(tick);

        assert_eq!(spinner.frame, initial_frame); // Should not advance
    }

    #[test]
    fn test_spinner_ignores_old_tags() {
        let mut spinner = SpinnerModel::new();
        spinner.tag = 5;
        let initial_frame = spinner.frame;

        // Tick with old tag
        let tick = Message::new(TickMsg {
            id: spinner.id(),
            tag: 3,
        });
        spinner.update(tick);

        assert_eq!(spinner.frame, initial_frame); // Should not advance
    }

    #[test]
    fn test_predefined_spinners() {
        // Just verify they can be created
        let _ = spinners::line();
        let _ = spinners::dot();
        let _ = spinners::mini_dot();
        let _ = spinners::jump();
        let _ = spinners::pulse();
        let _ = spinners::points();
        let _ = spinners::globe();
        let _ = spinners::moon();
        let _ = spinners::monkey();
        let _ = spinners::meter();
        let _ = spinners::hamburger();
        let _ = spinners::ellipsis();
    }

    #[test]
    fn test_spinner_frame_duration() {
        let spinner = Spinner::new(vec!["a"], 10);
        assert_eq!(spinner.frame_duration(), Duration::from_millis(100));

        let spinner = Spinner::new(vec!["a"], 0);
        assert_eq!(spinner.frame_duration(), Duration::from_secs(1));
    }

    #[test]
    fn test_model_init_returns_tick_cmd() {
        let spinner = SpinnerModel::new();
        let cmd = Model::init(&spinner);
        assert!(cmd.is_some());
    }

    #[test]
    fn test_model_update_advances_frame() {
        let mut spinner = SpinnerModel::new();
        let initial_frame = spinner.frame;
        let tick = Message::new(TickMsg {
            id: spinner.id(),
            tag: spinner.tag,
        });

        let cmd = Model::update(&mut spinner, tick);

        assert!(cmd.is_some());
        assert_eq!(spinner.frame, initial_frame + 1);
    }

    #[test]
    fn test_model_view_matches_view() {
        let spinner = SpinnerModel::new();
        assert_eq!(Model::view(&spinner), spinner.view());
    }
}
