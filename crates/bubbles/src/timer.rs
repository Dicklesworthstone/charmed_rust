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

use bubbletea::{Cmd, Message};

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
                return Some(bubbletea::batch(vec![
                    Some(tick_cmd),
                    Some(Cmd::new(move || Message::new(TimeoutMsg { id }))),
                ])?);
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
}
