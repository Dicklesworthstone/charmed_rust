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

use bubbletea::{Cmd, Message, Model};

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

/// Formats a duration for display, matching Go's time.Duration.String() behavior.
///
/// Format rules (matching Go):
/// - Less than 1 second: show as milliseconds (e.g., "100ms", "1ms")
/// - 1 second or more: show with decimal precision (e.g., "5.001s", "10.5s")
/// - 1 minute or more: show minutes and seconds (e.g., "1m30s", "2m5.5s")
/// - 1 hour or more: show hours, minutes, seconds (e.g., "1h0m0s", "1h30m15.5s")
fn format_duration(d: Duration) -> String {
    let total_nanos = d.as_nanos();

    // Zero case
    if total_nanos == 0 {
        return "0s".to_string();
    }

    let total_secs = d.as_secs();
    let subsec_nanos = d.subsec_nanos();

    // Less than 1 second - show as ms, µs, or ns
    if total_secs == 0 {
        let micros = d.as_micros();
        if micros >= 1000 {
            // Milliseconds
            let millis = d.as_millis();
            let remainder_micros = micros % 1000;
            if remainder_micros == 0 {
                return format!("{}ms", millis);
            }
            // Show with decimal precision
            let decimal = format!("{:06}", d.as_nanos() % 1_000_000);
            let trimmed = decimal.trim_end_matches('0');
            if trimmed.is_empty() {
                return format!("{}ms", millis);
            }
            return format!("{}.{}ms", millis, trimmed);
        } else if micros >= 1 {
            // Microseconds
            let nanos = d.as_nanos() % 1000;
            if nanos == 0 {
                return format!("{}µs", micros);
            }
            let decimal = format!("{:03}", nanos);
            let trimmed = decimal.trim_end_matches('0');
            return format!("{}.{}µs", micros, trimmed);
        } else {
            // Nanoseconds
            return format!("{}ns", d.as_nanos());
        }
    }

    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    // Format sub-second part
    let subsec_str = if subsec_nanos > 0 {
        // Convert to string with 9 decimal places, then trim trailing zeros
        let decimal = format!("{:09}", subsec_nanos);
        let trimmed = decimal.trim_end_matches('0');
        if trimmed.is_empty() {
            String::new()
        } else {
            format!(".{}", trimmed)
        }
    } else {
        String::new()
    };

    if hours > 0 {
        if subsec_str.is_empty() {
            format!("{}h{}m{}s", hours, minutes, seconds)
        } else {
            format!("{}h{}m{}{}s", hours, minutes, seconds, subsec_str)
        }
    } else if minutes > 0 {
        if subsec_str.is_empty() {
            format!("{}m{}s", minutes, seconds)
        } else {
            format!("{}m{}{}s", minutes, seconds, subsec_str)
        }
    } else {
        format!("{}{}s", seconds, subsec_str)
    }
}

/// Implement the Model trait for standalone bubbletea usage.
impl Model for Stopwatch {
    fn init(&self) -> Option<Cmd> {
        Stopwatch::init(self)
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        Stopwatch::update(self, msg)
    }

    fn view(&self) -> String {
        Stopwatch::view(self)
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

    // Model trait tests

    #[test]
    fn test_stopwatch_model_init_returns_cmd() {
        let sw = Stopwatch::new();
        // init() returns a command to start the stopwatch
        assert!(sw.init().is_some());
    }

    #[test]
    fn test_stopwatch_model_update_start_stop() {
        let mut sw = Stopwatch::new();
        assert!(!sw.running());

        // Start via update
        let msg = Message::new(StartStopMsg {
            id: sw.id(),
            running: true,
        });
        let result = sw.update(msg);
        assert!(sw.running());
        assert!(result.is_none()); // start/stop doesn't return a command

        // Stop via update
        let msg = Message::new(StartStopMsg {
            id: sw.id(),
            running: false,
        });
        let result = sw.update(msg);
        assert!(!sw.running());
        assert!(result.is_none());
    }

    #[test]
    fn test_stopwatch_model_update_tick_returns_cmd() {
        let mut sw = Stopwatch::new();
        sw.running = true;

        let tick = Message::new(TickMsg {
            id: sw.id(),
            tag: 0,
        });
        let result = sw.update(tick);

        // Tick returns a command to schedule the next tick
        assert!(result.is_some());
        assert_eq!(sw.elapsed(), Duration::from_secs(1));
    }

    #[test]
    fn test_stopwatch_model_update_tick_when_stopped_returns_none() {
        let mut sw = Stopwatch::new();
        assert!(!sw.running());

        let tick = Message::new(TickMsg {
            id: sw.id(),
            tag: 0,
        });
        let result = sw.update(tick);

        // Tick when stopped returns None and doesn't update elapsed
        assert!(result.is_none());
        assert_eq!(sw.elapsed(), Duration::ZERO);
    }

    #[test]
    fn test_stopwatch_model_update_reset() {
        let mut sw = Stopwatch::new();
        sw.elapsed = Duration::from_secs(100);
        sw.running = true;

        let msg = Message::new(ResetMsg { id: sw.id() });
        let result = sw.update(msg);

        assert_eq!(sw.elapsed(), Duration::ZERO);
        assert!(result.is_none());
    }

    #[test]
    fn test_stopwatch_model_view_zero_time() {
        let sw = Stopwatch::new();
        assert_eq!(sw.view(), "0s");
    }

    #[test]
    fn test_stopwatch_model_view_seconds_only() {
        let mut sw = Stopwatch::new();
        sw.elapsed = Duration::from_secs(45);
        assert_eq!(sw.view(), "45s");
    }

    #[test]
    fn test_stopwatch_model_view_minutes_seconds() {
        let mut sw = Stopwatch::new();
        sw.elapsed = Duration::from_secs(125);
        assert_eq!(sw.view(), "2m5s");
    }

    #[test]
    fn test_stopwatch_model_view_hours_minutes_seconds() {
        let mut sw = Stopwatch::new();
        sw.elapsed = Duration::from_secs(3665);
        assert_eq!(sw.view(), "1h1m5s");
    }

    #[test]
    fn test_stopwatch_model_view_with_milliseconds() {
        let mut sw = Stopwatch::new();
        // For times under 10 seconds with milliseconds, format shows decimal
        sw.elapsed = Duration::from_millis(5500);
        assert_eq!(sw.view(), "5.5s");
    }

    #[test]
    fn test_stopwatch_model_very_long_duration() {
        let mut sw = Stopwatch::new();
        // Test 100 hours
        sw.elapsed = Duration::from_secs(100 * 3600 + 30 * 60 + 15);
        assert_eq!(sw.view(), "100h30m15s");
    }

    #[test]
    fn test_stopwatch_model_tick_increments_tag() {
        let mut sw = Stopwatch::new();
        sw.running = true;
        let initial_tag = sw.tag;

        let tick = Message::new(TickMsg {
            id: sw.id(),
            tag: initial_tag,
        });
        sw.update(tick);

        // Tag should increment after each tick
        assert_eq!(sw.tag, initial_tag.wrapping_add(1));
    }

    #[test]
    fn test_stopwatch_model_old_tag_rejected() {
        let mut sw = Stopwatch::new();
        sw.running = true;
        sw.tag = 5; // Set current tag to 5

        // Old tag (1) should be rejected when current tag is 5
        let tick = Message::new(TickMsg {
            id: sw.id(),
            tag: 1,
        });
        let result = sw.update(tick);

        assert!(result.is_none());
        assert_eq!(sw.elapsed(), Duration::ZERO);
    }

    // Go parity tests - format_duration should match Go's time.Duration.String()

    #[test]
    fn test_format_duration_go_parity_sub_second() {
        // Sub-second durations use ms, µs, or ns units
        assert_eq!(format_duration(Duration::from_millis(100)), "100ms");
        assert_eq!(format_duration(Duration::from_millis(1)), "1ms");
        assert_eq!(format_duration(Duration::from_millis(999)), "999ms");
        assert_eq!(format_duration(Duration::from_micros(500)), "500µs");
        assert_eq!(format_duration(Duration::from_nanos(123)), "123ns");
    }

    #[test]
    fn test_format_duration_go_parity_seconds_with_decimals() {
        // Seconds with sub-second precision
        assert_eq!(format_duration(Duration::from_millis(5050)), "5.05s");
        assert_eq!(format_duration(Duration::from_millis(5100)), "5.1s");
        assert_eq!(format_duration(Duration::from_millis(5001)), "5.001s");
        assert_eq!(format_duration(Duration::from_millis(9999)), "9.999s");
        assert_eq!(format_duration(Duration::from_millis(10000)), "10s");
        assert_eq!(format_duration(Duration::from_millis(10001)), "10.001s");
    }

    #[test]
    fn test_format_duration_go_parity_minutes() {
        // Minutes and seconds
        assert_eq!(format_duration(Duration::from_secs(60)), "1m0s");
        assert_eq!(format_duration(Duration::from_secs(61)), "1m1s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m30s");
        assert_eq!(format_duration(Duration::from_secs(125)), "2m5s");
        // Minutes with sub-second precision
        assert_eq!(format_duration(Duration::from_millis(90500)), "1m30.5s");
    }

    #[test]
    fn test_format_duration_go_parity_hours() {
        // Hours, minutes, and seconds
        assert_eq!(format_duration(Duration::from_secs(3600)), "1h0m0s");
        assert_eq!(format_duration(Duration::from_secs(3665)), "1h1m5s");
        assert_eq!(
            format_duration(Duration::from_secs(100 * 3600 + 30 * 60 + 15)),
            "100h30m15s"
        );
        // Hours with sub-second precision
        assert_eq!(format_duration(Duration::from_millis(3600_500)), "1h0m0.5s");
    }
}
