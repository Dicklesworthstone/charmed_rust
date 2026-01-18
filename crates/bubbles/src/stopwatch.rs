//! Stopwatch component for tracking elapsed time.
//!
//! This module provides a stopwatch that counts up from zero, useful for
//! measuring elapsed time in TUI applications.
//!
//! # Example
//!
//! ```rust
//! use bubbles::stopwatch::Stopwatch;
//! use std::time::Duration;
//!
//! let stopwatch = Stopwatch::new();
//! assert_eq!(stopwatch.elapsed(), Duration::ZERO);
//! assert!(!stopwatch.running());
//! ```

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use bubbletea::{Cmd, Message};

/// Global ID counter for stopwatch instances.
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

/// Message sent on every stopwatch tick.
#[derive(Debug, Clone, Copy)]
pub struct TickMsg {
    /// The stopwatch ID.
    pub id: u64,
    /// Tag for message ordering.
    tag: u64,
}

impl TickMsg {
    /// Creates a new tick message.
    #[must_use]
    pub fn new(id: u64, tag: u64) -> Self {
        Self { id, tag }
    }
}

/// Message to start or stop the stopwatch.
#[derive(Debug, Clone, Copy)]
pub struct StartStopMsg {
    /// The stopwatch ID.
    pub id: u64,
    /// Whether to start (true) or stop (false).
    pub running: bool,
}

/// Message to reset the stopwatch.
#[derive(Debug, Clone, Copy)]
pub struct ResetMsg {
    /// The stopwatch ID.
    pub id: u64,
}

/// Stopwatch model.
#[derive(Debug, Clone)]
pub struct Stopwatch {
    /// Elapsed time.
    elapsed: Duration,
    /// Tick interval.
    interval: Duration,
    /// Unique ID.
    id: u64,
    /// Message tag for ordering.
    tag: u64,
    /// Whether the stopwatch is running.
    running: bool,
}

impl Default for Stopwatch {
    fn default() -> Self {
        Self::new()
    }
}

impl Stopwatch {
    /// Creates a new stopwatch with the default 1-second interval.
    #[must_use]
    pub fn new() -> Self {
        Self::with_interval(Duration::from_secs(1))
    }

    /// Creates a new stopwatch with the given tick interval.
    #[must_use]
    pub fn with_interval(interval: Duration) -> Self {
        Self {
            elapsed: Duration::ZERO,
            interval,
            id: next_id(),
            tag: 0,
            running: false,
        }
    }

    /// Returns the stopwatch's unique ID.
    #[must_use]
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns whether the stopwatch is currently running.
    #[must_use]
    pub fn running(&self) -> bool {
        self.running
    }

    /// Returns the elapsed time.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// Returns the tick interval.
    #[must_use]
    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// Returns a command to initialize and start the stopwatch.
    #[must_use]
    pub fn init(&self) -> Option<Cmd> {
        self.start_cmd()
    }

    /// Starts the stopwatch.
    fn start_cmd(&self) -> Option<Cmd> {
        let id = self.id;
        let tag = self.tag;
        let interval = self.interval;

        bubbletea::sequence(vec![
            Some(Cmd::new(move || {
                Message::new(StartStopMsg { id, running: true })
            })),
            Some(Cmd::new(move || {
                std::thread::sleep(interval);
                Message::new(TickMsg { id, tag })
            })),
        ])
    }

    /// Creates a command to start the stopwatch.
    pub fn start(&self) -> Option<Cmd> {
        self.start_cmd()
    }

    /// Creates a command to stop the stopwatch.
    pub fn stop(&self) -> Option<Cmd> {
        let id = self.id;
        Some(Cmd::new(move || {
            Message::new(StartStopMsg { id, running: false })
        }))
    }

    /// Creates a command to toggle the stopwatch.
    pub fn toggle(&self) -> Option<Cmd> {
        if self.running() {
            self.stop()
        } else {
            self.start()
        }
    }

    /// Creates a command to reset the stopwatch.
    pub fn reset(&self) -> Option<Cmd> {
        let id = self.id;
        Some(Cmd::new(move || Message::new(ResetMsg { id })))
    }

    /// Creates a tick command.
    fn tick_cmd(&self) -> Cmd {
        let id = self.id;
        let tag = self.tag;
        let interval = self.interval;

        Cmd::new(move || {
            std::thread::sleep(interval);
            Message::new(TickMsg { id, tag })
        })
    }

    /// Updates the stopwatch state.
    pub fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle start/stop
        if let Some(ss) = msg.downcast_ref::<StartStopMsg>() {
            if ss.id != self.id {
                return None;
            }
            self.running = ss.running;
            return None;
        }

        // Handle reset
        if let Some(reset) = msg.downcast_ref::<ResetMsg>() {
            if reset.id != self.id {
                return None;
            }
            self.elapsed = Duration::ZERO;
            return None;
        }

        // Handle tick
        if let Some(tick) = msg.downcast_ref::<TickMsg>() {
            if !self.running || tick.id != self.id {
                return None;
            }

            // Reject old tags
            if tick.tag > 0 && tick.tag != self.tag {
                return None;
            }

            self.elapsed += self.interval;
            self.tag = self.tag.wrapping_add(1);
            return Some(self.tick_cmd());
        }

        None
    }

    /// Renders the stopwatch display.
    #[must_use]
    pub fn view(&self) -> String {
        format_duration(self.elapsed)
    }
}

/// Formats a duration for display.
fn format_duration(d: Duration) -> String {
    let total_secs = d.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    let millis = d.subsec_millis();

    if hours > 0 {
        format!("{}h{}m{}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m{}s", minutes, seconds)
    } else if millis > 0 && seconds < 10 {
        format!("{}.{}s", seconds, millis / 100)
    } else {
        format!("{}s", seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stopwatch_new() {
        let sw = Stopwatch::new();
        assert_eq!(sw.elapsed(), Duration::ZERO);
        assert!(!sw.running());
        assert_eq!(sw.interval(), Duration::from_secs(1));
    }

    #[test]
    fn test_stopwatch_unique_ids() {
        let sw1 = Stopwatch::new();
        let sw2 = Stopwatch::new();
        assert_ne!(sw1.id(), sw2.id());
    }

    #[test]
    fn test_stopwatch_with_interval() {
        let sw = Stopwatch::with_interval(Duration::from_millis(100));
        assert_eq!(sw.interval(), Duration::from_millis(100));
    }

    #[test]
    fn test_stopwatch_start_stop() {
        let mut sw = Stopwatch::new();
        assert!(!sw.running());

        // Simulate start message
        let msg = Message::new(StartStopMsg {
            id: sw.id(),
            running: true,
        });
        sw.update(msg);
        assert!(sw.running());

        // Simulate stop message
        let msg = Message::new(StartStopMsg {
            id: sw.id(),
            running: false,
        });
        sw.update(msg);
        assert!(!sw.running());
    }

    #[test]
    fn test_stopwatch_tick() {
        let mut sw = Stopwatch::new();
        sw.running = true;

        let tick = Message::new(TickMsg {
            id: sw.id(),
            tag: 0,
        });
        sw.update(tick);

        assert_eq!(sw.elapsed(), Duration::from_secs(1));
    }

    #[test]
    fn test_stopwatch_reset() {
        let mut sw = Stopwatch::new();
        sw.elapsed = Duration::from_secs(100);

        let msg = Message::new(ResetMsg { id: sw.id() });
        sw.update(msg);

        assert_eq!(sw.elapsed(), Duration::ZERO);
    }

    #[test]
    fn test_stopwatch_ignores_other_ids() {
        let mut sw = Stopwatch::new();
        sw.running = true;

        let tick = Message::new(TickMsg { id: 9999, tag: 0 });
        sw.update(tick);

        assert_eq!(sw.elapsed(), Duration::ZERO);
    }

    #[test]
    fn test_stopwatch_view() {
        let mut sw = Stopwatch::new();
        sw.elapsed = Duration::from_secs(125);
        assert_eq!(sw.view(), "2m5s");

        sw.elapsed = Duration::from_secs(3665);
        assert_eq!(sw.view(), "1h1m5s");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(0)), "0s");
        assert_eq!(format_duration(Duration::from_secs(45)), "45s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m30s");
        assert_eq!(format_duration(Duration::from_secs(3600)), "1h0m0s");
    }
}
