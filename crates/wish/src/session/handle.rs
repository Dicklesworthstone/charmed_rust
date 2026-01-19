//! Session handle and info types.

use crate::auth::SessionId;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::oneshot;

/// Handle to an active session with metadata and shutdown signaling.
pub struct SessionHandle {
    /// Unique session identifier.
    pub id: SessionId,
    /// Authenticated username.
    pub user: String,
    /// Client's remote address.
    pub remote_addr: SocketAddr,
    /// When the session was established.
    pub started_at: Instant,
    /// Last activity timestamp (updated on data/commands).
    last_activity: Arc<AtomicU64>,
    /// Number of active channels in this session.
    pub channel_count: AtomicU64,
    /// Shutdown signal sender - dropping this signals the session to terminate.
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl SessionHandle {
    /// Creates a new session handle.
    pub fn new(
        id: SessionId,
        user: String,
        remote_addr: SocketAddr,
        shutdown_tx: oneshot::Sender<()>,
    ) -> Self {
        let now = Instant::now();
        Self {
            id,
            user,
            remote_addr,
            started_at: now,
            last_activity: Arc::new(AtomicU64::new(0)), // 0 means "same as started_at"
            channel_count: AtomicU64::new(0),
            shutdown_tx: Some(shutdown_tx),
        }
    }

    /// Updates the last activity timestamp.
    pub fn touch(&self) {
        let elapsed = self.started_at.elapsed().as_millis() as u64;
        self.last_activity.store(elapsed, Ordering::Relaxed);
    }

    /// Returns the duration since last activity.
    pub fn idle_time(&self) -> Duration {
        let activity_offset_ms = self.last_activity.load(Ordering::Relaxed);
        if activity_offset_ms == 0 {
            // No activity recorded, use session start time
            self.started_at.elapsed()
        } else {
            let activity_instant =
                self.started_at + Duration::from_millis(activity_offset_ms);
            activity_instant.elapsed()
        }
    }

    /// Returns the total session duration.
    pub fn duration(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Increments the channel count.
    pub fn add_channel(&self) {
        self.channel_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrements the channel count.
    pub fn remove_channel(&self) {
        self.channel_count.fetch_sub(1, Ordering::Relaxed);
    }

    /// Returns the current channel count.
    pub fn channels(&self) -> u64 {
        self.channel_count.load(Ordering::Relaxed)
    }

    /// Signals the session to shut down by consuming the shutdown sender.
    pub fn signal_shutdown(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            // Dropping the sender signals shutdown; explicit send is redundant
            // but we do it for clarity
            let _ = tx.send(());
        }
    }

    /// Creates a SessionInfo snapshot for external consumption.
    pub fn info(&self) -> SessionInfo {
        SessionInfo {
            id: self.id.0,
            user: self.user.clone(),
            remote_addr: self.remote_addr,
            started_at: SystemTime::now() - self.started_at.elapsed(),
            duration: self.duration(),
            idle_time: self.idle_time(),
            channel_count: self.channels(),
        }
    }
}

/// Information about an active session (snapshot for inspection/logging).
#[derive(Clone, Debug)]
pub struct SessionInfo {
    /// Unique session identifier.
    pub id: u64,
    /// Authenticated username.
    pub user: String,
    /// Client's remote address.
    pub remote_addr: SocketAddr,
    /// When the session was established.
    pub started_at: SystemTime,
    /// Total session duration.
    pub duration: Duration,
    /// Time since last activity.
    pub idle_time: Duration,
    /// Number of active channels.
    pub channel_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn test_addr() -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 22222)
    }

    #[test]
    fn test_session_handle_new() {
        let (tx, _rx) = oneshot::channel();
        let handle = SessionHandle::new(SessionId(1), "test_user".to_string(), test_addr(), tx);

        assert_eq!(handle.id.0, 1);
        assert_eq!(handle.user, "test_user");
        assert_eq!(handle.channels(), 0);
    }

    #[test]
    fn test_session_handle_touch() {
        let (tx, _rx) = oneshot::channel();
        let handle = SessionHandle::new(SessionId(1), "test_user".to_string(), test_addr(), tx);

        // Initially idle time should be very small
        let initial_idle = handle.idle_time();
        assert!(initial_idle < Duration::from_millis(100));

        // Wait a bit
        std::thread::sleep(Duration::from_millis(50));

        // Touch should reset idle time
        handle.touch();
        let new_idle = handle.idle_time();
        assert!(new_idle < Duration::from_millis(10));
    }

    #[test]
    fn test_session_handle_channels() {
        let (tx, _rx) = oneshot::channel();
        let handle = SessionHandle::new(SessionId(1), "test_user".to_string(), test_addr(), tx);

        assert_eq!(handle.channels(), 0);
        handle.add_channel();
        assert_eq!(handle.channels(), 1);
        handle.add_channel();
        assert_eq!(handle.channels(), 2);
        handle.remove_channel();
        assert_eq!(handle.channels(), 1);
    }

    #[test]
    fn test_session_info() {
        let (tx, _rx) = oneshot::channel();
        let handle = SessionHandle::new(SessionId(42), "alice".to_string(), test_addr(), tx);

        let info = handle.info();
        assert_eq!(info.id, 42);
        assert_eq!(info.user, "alice");
        assert_eq!(info.channel_count, 0);
    }
}
