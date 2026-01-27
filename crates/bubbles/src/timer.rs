//! Countdown timer component.
//!
//! This module provides a countdown timer that ticks down from a specified
//! duration and sends timeout messages when complete.
//!
//! # Example
//!
//! ```rust
//! use bubbles::timer::Timer;
//! use std::time::Duration;
//!
//! let timer = Timer::new(Duration::from_secs(60));
//! assert_eq!(timer.remaining(), Duration::from_secs(60));
//! assert!(!timer.timed_out());
//! ```

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use bubbletea::{Cmd, Message, Model};

/// Global ID counter for timer instances.
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

/// Message to start or stop the timer.
#[derive(Debug, Clone, Copy)]
pub struct StartStopMsg {
    /// The timer ID.
    pub id: u64,
    /// Whether to start (true) or stop (false).
    pub running: bool,
}

/// Message sent on every timer tick.
#[derive(Debug, Clone, Copy)]
pub struct TickMsg {
    /// The timer ID.
    pub id: u64,
    /// Whether this tick indicates a timeout.
    pub timeout: bool,
    /// Tag for message ordering.
    tag: u64,
}

impl TickMsg {
    /// Creates a new tick message.
    #[must_use]
    pub fn new(id: u64, timeout: bool, tag: u64) -> Self {
        Self { id, timeout, tag }
    }
}

/// Message sent once when the timer times out.
#[derive(Debug, Clone, Copy)]
pub struct TimeoutMsg {
    /// The timer ID.
    pub id: u64,
}

/// Countdown timer model.
#[derive(Debug, Clone)]
pub struct Timer {
    /// Remaining time.
    timeout: Duration,
    /// Tick interval.
    interval: Duration,
    /// Unique ID.
    id: u64,
    /// Message tag for ordering.
    tag: u64,
    /// Whether the timer is running.
    running: bool,
}

impl Timer {
    /// Creates a new timer with the given timeout and default 1-second interval.
    #[must_use]
    pub fn new(timeout: Duration) -> Self {
        Self::with_interval(timeout, Duration::from_secs(1))
    }

    /// Creates a new timer with the given timeout and tick interval.
    #[must_use]
    pub fn with_interval(timeout: Duration, interval: Duration) -> Self {
        Self {
            timeout,
            interval,
            id: next_id(),
            tag: 0,
            running: true,
        }
    }

    /// Returns the timer's unique ID.
    #[must_use]
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns whether the timer is currently running.
    #[must_use]
    pub fn running(&self) -> bool {
        if self.timed_out() {
            return false;
        }
        self.running
    }

    /// Returns whether the timer has timed out.
    #[must_use]
    pub fn timed_out(&self) -> bool {
        self.timeout.is_zero()
    }

    /// Returns the remaining time.
    #[must_use]
    pub fn remaining(&self) -> Duration {
        self.timeout
    }

    /// Returns the tick interval.
    #[must_use]
    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// Returns a command to initialize the timer (start ticking).
    #[must_use]
    pub fn init(&self) -> Option<Cmd> {
        Some(self.tick_cmd())
    }

    /// Starts the timer.
    pub fn start(&mut self) -> Option<Cmd> {
        let id = self.id;
        Some(Cmd::new(move || {
            Message::new(StartStopMsg { id, running: true })
        }))
    }

    /// Stops the timer.
    pub fn stop(&mut self) -> Option<Cmd> {
        let id = self.id;
        Some(Cmd::new(move || {
            Message::new(StartStopMsg { id, running: false })
        }))
    }

    /// Toggles the timer between running and stopped.
    pub fn toggle(&mut self) -> Option<Cmd> {
        if self.running() {
            self.stop()
        } else {
            self.start()
        }
    }

    /// Creates a tick command.
    fn tick_cmd(&self) -> Cmd {
        let id = self.id;
        let tag = self.tag;
        let interval = self.interval;
        let timed_out = self.timed_out();

        Cmd::new(move || {
            std::thread::sleep(interval);
            Message::new(TickMsg {
                id,
                tag,
                timeout: timed_out,
            })
        })
    }

    /// Updates the timer state.
    pub fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle start/stop
        if let Some(ss) = msg.downcast_ref::<StartStopMsg>() {
            if ss.id != 0 && ss.id != self.id {
                return None;
            }
            self.running = ss.running;
            return Some(self.tick_cmd());
        }

        // Handle tick
        if let Some(tick) = msg.downcast_ref::<TickMsg>() {
            if !self.running() || (tick.id != 0 && tick.id != self.id) {
                return None;
            }

            // Reject old tags
            if tick.tag > 0 && tick.tag != self.tag {
                return None;
            }

            // Decrease timeout
            self.timeout = self.timeout.saturating_sub(self.interval);
            self.tag = self.tag.wrapping_add(1);

            // Return tick command and optionally timeout message
            if self.timed_out() {
                let id = self.id;
                let tick_cmd = self.tick_cmd();
                return bubbletea::batch(vec![
                    Some(tick_cmd),
                    Some(Cmd::new(move || Message::new(TimeoutMsg { id }))),
                ]);
            }

            return Some(self.tick_cmd());
        }

        None
    }

    /// Renders the timer display.
    #[must_use]
    pub fn view(&self) -> String {
        format_duration(self.timeout)
    }
}

/// Implement the Model trait for standalone bubbletea usage.
impl Model for Timer {
    fn init(&self) -> Option<Cmd> {
        Some(self.tick_cmd())
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle start/stop
        if let Some(ss) = msg.downcast_ref::<StartStopMsg>() {
            if ss.id != 0 && ss.id != self.id {
                return None;
            }
            self.running = ss.running;
            return Some(self.tick_cmd());
        }

        // Handle tick
        if let Some(tick) = msg.downcast_ref::<TickMsg>() {
            if !self.running() || (tick.id != 0 && tick.id != self.id) {
                return None;
            }

            // Reject old tags
            if tick.tag > 0 && tick.tag != self.tag {
                return None;
            }

            // Decrease timeout
            self.timeout = self.timeout.saturating_sub(self.interval);
            self.tag = self.tag.wrapping_add(1);

            // Return tick command and optionally timeout message
            if self.timed_out() {
                let id = self.id;
                let tick_cmd = self.tick_cmd();
                return bubbletea::batch(vec![
                    Some(tick_cmd),
                    Some(Cmd::new(move || Message::new(TimeoutMsg { id }))),
                ]);
            }

            return Some(self.tick_cmd());
        }

        None
    }

    fn view(&self) -> String {
        format_duration(self.timeout)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_new() {
        let timer = Timer::new(Duration::from_secs(60));
        assert_eq!(timer.remaining(), Duration::from_secs(60));
        assert!(timer.running());
        assert!(!timer.timed_out());
    }

    #[test]
    fn test_timer_unique_ids() {
        let t1 = Timer::new(Duration::from_secs(10));
        let t2 = Timer::new(Duration::from_secs(10));
        assert_ne!(t1.id(), t2.id());
    }

    #[test]
    fn test_timer_with_interval() {
        let timer = Timer::with_interval(Duration::from_secs(60), Duration::from_millis(100));
        assert_eq!(timer.interval(), Duration::from_millis(100));
    }

    #[test]
    fn test_timer_tick() {
        let mut timer = Timer::new(Duration::from_secs(10));
        let tick = Message::new(TickMsg {
            id: timer.id(),
            tag: 0,
            timeout: false,
        });

        timer.update(tick);
        assert_eq!(timer.remaining(), Duration::from_secs(9));
    }

    #[test]
    fn test_timer_timeout() {
        let mut timer = Timer::new(Duration::from_secs(1));

        // Tick once
        let tick = Message::new(TickMsg {
            id: timer.id(),
            tag: 0,
            timeout: false,
        });
        timer.update(tick);

        assert!(timer.timed_out());
        assert!(!timer.running());
    }

    #[test]
    fn test_timer_ignores_other_ids() {
        let mut timer = Timer::new(Duration::from_secs(10));
        let original = timer.remaining();

        let tick = Message::new(TickMsg {
            id: 9999,
            tag: 0,
            timeout: false,
        });
        timer.update(tick);

        assert_eq!(timer.remaining(), original);
    }

    #[test]
    fn test_timer_view() {
        let timer = Timer::new(Duration::from_secs(125));
        assert_eq!(timer.view(), "2m5s");

        let timer = Timer::new(Duration::from_secs(3665));
        assert_eq!(timer.view(), "1h1m5s");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(0)), "0s");
        assert_eq!(format_duration(Duration::from_secs(45)), "45s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m30s");
        assert_eq!(format_duration(Duration::from_secs(3600)), "1h0m0s");
        assert_eq!(format_duration(Duration::from_millis(5500)), "5.5s");
    }

    // Model trait implementation tests

    #[test]
    fn test_model_trait_init_returns_cmd() {
        let timer = Timer::new(Duration::from_secs(30));
        // Use the Model trait method explicitly
        let cmd = Model::init(&timer);
        assert!(cmd.is_some(), "Model::init should return a command");
    }

    #[test]
    fn test_model_trait_view_formats_time() {
        let timer = Timer::new(Duration::from_secs(125));
        // Use the Model trait method explicitly
        let view = Model::view(&timer);
        assert_eq!(view, "2m5s");
    }

    #[test]
    fn test_model_trait_update_handles_tick() {
        let mut timer = Timer::new(Duration::from_secs(10));
        let id = timer.id();

        // Use the Model trait method explicitly
        let tick_msg = Message::new(TickMsg {
            id,
            tag: 0,
            timeout: false,
        });
        let cmd = Model::update(&mut timer, tick_msg);

        // Should return a command for the next tick
        assert!(
            cmd.is_some(),
            "Model::update should return next tick command"
        );
        assert_eq!(timer.remaining(), Duration::from_secs(9));
    }

    #[test]
    fn test_model_trait_update_handles_start_stop() {
        let mut timer = Timer::new(Duration::from_secs(10));
        let id = timer.id();

        // Stop the timer
        let stop_msg = Message::new(StartStopMsg { id, running: false });
        let _ = Model::update(&mut timer, stop_msg);
        assert!(!timer.running(), "Timer should be stopped");

        // Start the timer
        let start_msg = Message::new(StartStopMsg { id, running: true });
        let _ = Model::update(&mut timer, start_msg);
        assert!(timer.running(), "Timer should be running again");
    }

    #[test]
    fn test_timer_satisfies_model_bounds() {
        // This test verifies Timer can be used where Model + Send + 'static is required
        fn accepts_model<M: Model + Send + 'static>(_model: M) {}
        let timer = Timer::new(Duration::from_secs(10));
        accepts_model(timer);
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
        assert_eq!(
            format_duration(Duration::from_millis(3_600_500)),
            "1h0m0.5s"
        );
    }

    #[test]
    fn test_timer_countdown_progression() {
        // Test that timer counts down correctly over multiple ticks
        let mut timer = Timer::with_interval(Duration::from_secs(5), Duration::from_secs(1));

        // Tick 5 times
        for i in 0..5 {
            assert_eq!(timer.remaining(), Duration::from_secs(5 - i));
            if i < 5 {
                let tick = Message::new(TickMsg {
                    id: timer.id(),
                    tag: timer.tag,
                    timeout: false,
                });
                timer.update(tick);
            }
        }

        assert!(timer.timed_out());
        assert!(!timer.running());
    }

    #[test]
    fn test_timer_zero_duration() {
        // Timer created with zero duration should be timed out immediately
        let timer = Timer::new(Duration::ZERO);
        assert!(timer.timed_out());
        assert!(!timer.running());
        assert_eq!(timer.view(), "0s");
    }
}
