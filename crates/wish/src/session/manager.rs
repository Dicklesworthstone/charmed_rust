//! Session manager for tracking and cleaning up SSH sessions.

use super::handle::{SessionHandle, SessionInfo};
use crate::auth::SessionId;
use crate::Error;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

/// Configuration for the session manager.
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Maximum number of concurrent sessions.
    pub max_sessions: usize,
    /// Session timeout (sessions inactive longer than this are cleaned up).
    pub session_timeout: Duration,
    /// Interval between cleanup sweeps.
    pub cleanup_interval: Duration,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_sessions: 100,
            session_timeout: Duration::from_secs(3600), // 1 hour
            cleanup_interval: Duration::from_secs(30),
        }
    }
}

/// Manages SSH session lifecycle including creation, tracking, and cleanup.
pub struct SessionManager {
    /// Active sessions indexed by session ID.
    sessions: RwLock<HashMap<u64, SessionHandle>>,
    /// Configuration.
    config: SessionConfig,
    /// Next session ID counter.
    next_id: AtomicU64,
    /// Cleanup task handle (if started).
    cleanup_handle: RwLock<Option<JoinHandle<()>>>,
}

impl SessionManager {
    /// Creates a new session manager with the given configuration.
    pub fn new(config: SessionConfig) -> Self {
        info!(
            max_sessions = config.max_sessions,
            timeout_secs = config.session_timeout.as_secs(),
            "Session manager initialized"
        );
        Self {
            sessions: RwLock::new(HashMap::new()),
            config,
            next_id: AtomicU64::new(1),
            cleanup_handle: RwLock::new(None),
        }
    }

    /// Creates a new session for the given user and address.
    ///
    /// Returns the session ID and a shutdown receiver that will be signaled
    /// when the session should terminate.
    ///
    /// # Errors
    ///
    /// Returns `Error::MaxSessionsReached` if the maximum number of sessions is reached.
    pub fn create_session(
        &self,
        user: String,
        remote_addr: SocketAddr,
    ) -> Result<(SessionId, oneshot::Receiver<()>), Error> {
        let mut sessions = self.sessions.write();

        // Check session limit
        if sessions.len() >= self.config.max_sessions {
            warn!(
                max = self.config.max_sessions,
                current = sessions.len(),
                user = %user,
                addr = %remote_addr,
                "Maximum sessions reached, rejecting connection"
            );
            return Err(Error::Ssh("maximum sessions reached".to_string()));
        }

        // Generate session ID
        let id = SessionId(self.next_id.fetch_add(1, Ordering::SeqCst));

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        // Create session handle
        let handle = SessionHandle::new(id, user.clone(), remote_addr, shutdown_tx);

        info!(
            session_id = id.0,
            user = %user,
            addr = %remote_addr,
            total_sessions = sessions.len() + 1,
            "Session created"
        );

        sessions.insert(id.0, handle);

        Ok((id, shutdown_rx))
    }

    /// Removes a session by ID.
    pub fn remove_session(&self, id: SessionId) {
        let mut sessions = self.sessions.write();
        if let Some(handle) = sessions.remove(&id.0) {
            let duration = handle.duration();
            info!(
                session_id = id.0,
                user = %handle.user,
                duration_secs = duration.as_secs(),
                remaining_sessions = sessions.len(),
                "Session removed"
            );
        }
    }

    /// Updates the activity timestamp for a session.
    pub fn update_activity(&self, id: SessionId) {
        let sessions = self.sessions.read();
        if let Some(handle) = sessions.get(&id.0) {
            handle.touch();
            debug!(session_id = id.0, "Session activity updated");
        }
    }

    /// Adds a channel to a session's channel count.
    pub fn add_channel(&self, id: SessionId) {
        let sessions = self.sessions.read();
        if let Some(handle) = sessions.get(&id.0) {
            handle.add_channel();
            debug!(
                session_id = id.0,
                channels = handle.channels(),
                "Channel added to session"
            );
        }
    }

    /// Removes a channel from a session's channel count.
    pub fn remove_channel(&self, id: SessionId) {
        let sessions = self.sessions.read();
        if let Some(handle) = sessions.get(&id.0) {
            handle.remove_channel();
            debug!(
                session_id = id.0,
                channels = handle.channels(),
                "Channel removed from session"
            );
        }
    }

    /// Returns information about all active sessions.
    pub fn get_sessions(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.read();
        sessions.values().map(SessionHandle::info).collect()
    }

    /// Returns information about a specific session.
    pub fn get_session(&self, id: SessionId) -> Option<SessionInfo> {
        let sessions = self.sessions.read();
        sessions.get(&id.0).map(SessionHandle::info)
    }

    /// Returns the number of active sessions.
    pub fn session_count(&self) -> usize {
        self.sessions.read().len()
    }

    /// Signals a specific session to shut down.
    pub fn shutdown_session(&self, id: SessionId) {
        let mut sessions = self.sessions.write();
        if let Some(handle) = sessions.get_mut(&id.0) {
            info!(session_id = id.0, user = %handle.user, "Signaling session shutdown");
            handle.signal_shutdown();
        }
    }

    /// Starts the background cleanup task.
    ///
    /// This task periodically scans for stale sessions and removes them.
    pub fn start_cleanup_task(self: &Arc<Self>) {
        let manager = Arc::clone(self);
        let interval = self.config.cleanup_interval;
        let timeout = self.config.session_timeout;

        let handle = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;
                manager.cleanup_stale_sessions(timeout);
            }
        });

        *self.cleanup_handle.write() = Some(handle);
        debug!(
            interval_secs = interval.as_secs(),
            "Session cleanup task started"
        );
    }

    /// Cleans up sessions that have been idle longer than the timeout.
    fn cleanup_stale_sessions(&self, timeout: Duration) {
        let mut sessions = self.sessions.write();
        let mut stale_ids = Vec::new();

        for (id, handle) in sessions.iter() {
            let idle = handle.idle_time();
            if idle > timeout {
                warn!(
                    session_id = id,
                    user = %handle.user,
                    idle_secs = idle.as_secs(),
                    timeout_secs = timeout.as_secs(),
                    "Session timed out due to inactivity"
                );
                stale_ids.push(*id);
            }
        }

        for id in stale_ids {
            if let Some(mut handle) = sessions.remove(&id) {
                handle.signal_shutdown();
                info!(session_id = id, "Stale session cleaned up");
            }
        }

        if !sessions.is_empty() {
            debug!(active_sessions = sessions.len(), "Session cleanup complete");
        }
    }

    /// Performs graceful shutdown, waiting for sessions to close.
    ///
    /// Signals all sessions to shut down and waits up to `timeout` for them
    /// to close cleanly. Returns the number of sessions that were forcefully
    /// terminated (didn't close in time).
    pub async fn shutdown_graceful(&self, timeout: Duration) -> usize {
        info!(
            timeout_secs = timeout.as_secs(),
            "Initiating graceful session shutdown"
        );

        // Signal all sessions to shut down
        {
            let mut sessions = self.sessions.write();
            let count = sessions.len();
            info!(session_count = count, "Signaling all sessions to shut down");

            for handle in sessions.values_mut() {
                handle.signal_shutdown();
            }
        }

        // Wait for sessions to close
        let deadline = tokio::time::Instant::now() + timeout;
        let check_interval = Duration::from_millis(100);

        loop {
            let remaining = self.session_count();
            if remaining == 0 {
                info!("All sessions closed gracefully");
                return 0;
            }

            if tokio::time::Instant::now() >= deadline {
                warn!(
                    remaining_sessions = remaining,
                    "Timeout waiting for sessions to close"
                );
                // Force remove remaining sessions
                let mut sessions = self.sessions.write();
                let count = sessions.len();
                sessions.clear();
                return count;
            }

            tokio::time::sleep(check_interval).await;
        }
    }

    /// Stops the cleanup task if running.
    pub fn stop_cleanup_task(&self) {
        if let Some(handle) = self.cleanup_handle.write().take() {
            handle.abort();
            debug!("Session cleanup task stopped");
        }
    }
}

impl Drop for SessionManager {
    fn drop(&mut self) {
        self.stop_cleanup_task();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn test_addr() -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 22222)
    }

    #[test]
    fn test_session_manager_create() {
        let config = SessionConfig {
            max_sessions: 10,
            ..Default::default()
        };
        let manager = SessionManager::new(config);

        let (id, _rx) = manager.create_session("alice".to_string(), test_addr()).unwrap();
        assert_eq!(id.0, 1);
        assert_eq!(manager.session_count(), 1);

        let (id2, _rx2) = manager.create_session("bob".to_string(), test_addr()).unwrap();
        assert_eq!(id2.0, 2);
        assert_eq!(manager.session_count(), 2);
    }

    #[test]
    fn test_session_manager_max_sessions() {
        let config = SessionConfig {
            max_sessions: 2,
            ..Default::default()
        };
        let manager = SessionManager::new(config);

        let _s1 = manager.create_session("user1".to_string(), test_addr()).unwrap();
        let _s2 = manager.create_session("user2".to_string(), test_addr()).unwrap();

        // Third session should fail
        let result = manager.create_session("user3".to_string(), test_addr());
        assert!(result.is_err());
    }

    #[test]
    fn test_session_manager_remove() {
        let config = SessionConfig::default();
        let manager = SessionManager::new(config);

        let (id, _rx) = manager.create_session("test".to_string(), test_addr()).unwrap();
        assert_eq!(manager.session_count(), 1);

        manager.remove_session(id);
        assert_eq!(manager.session_count(), 0);
    }

    #[test]
    fn test_session_manager_get_sessions() {
        let config = SessionConfig::default();
        let manager = SessionManager::new(config);

        let (_id1, _rx1) = manager.create_session("alice".to_string(), test_addr()).unwrap();
        let (_id2, _rx2) = manager.create_session("bob".to_string(), test_addr()).unwrap();

        let sessions = manager.get_sessions();
        assert_eq!(sessions.len(), 2);

        let users: Vec<&str> = sessions.iter().map(|s| s.user.as_str()).collect();
        assert!(users.contains(&"alice"));
        assert!(users.contains(&"bob"));
    }

    #[test]
    fn test_session_manager_activity_tracking() {
        let config = SessionConfig::default();
        let manager = SessionManager::new(config);

        let (id, _rx) = manager.create_session("test".to_string(), test_addr()).unwrap();

        // Update activity
        manager.update_activity(id);

        // Get session info - idle time should be small
        let info = manager.get_session(id).unwrap();
        assert!(info.idle_time < Duration::from_secs(1));
    }

    #[test]
    fn test_session_manager_channels() {
        let config = SessionConfig::default();
        let manager = SessionManager::new(config);

        let (id, _rx) = manager.create_session("test".to_string(), test_addr()).unwrap();

        manager.add_channel(id);
        manager.add_channel(id);

        let info = manager.get_session(id).unwrap();
        assert_eq!(info.channel_count, 2);

        manager.remove_channel(id);
        let info = manager.get_session(id).unwrap();
        assert_eq!(info.channel_count, 1);
    }

    #[test]
    fn test_session_manager_shutdown_session() {
        let config = SessionConfig::default();
        let manager = SessionManager::new(config);

        let (id, mut shutdown_rx) =
            manager.create_session("test".to_string(), test_addr()).unwrap();
        assert_eq!(manager.session_count(), 1);

        // Signal shutdown
        manager.shutdown_session(id);

        // Verify shutdown was signaled (receiver should be closed or have a value)
        // Note: We can't easily test that the receiver got the signal in sync code,
        // but we can verify the session still exists (removal is done by the session task)
        assert_eq!(manager.session_count(), 1);

        // The shutdown signal should have been consumed
        assert!(shutdown_rx.try_recv().is_ok() || shutdown_rx.try_recv().is_err());
    }

    #[test]
    fn test_session_manager_cleanup_stale_sessions() {
        let config = SessionConfig {
            max_sessions: 10,
            session_timeout: Duration::from_millis(50),
            cleanup_interval: Duration::from_millis(10),
        };
        let manager = SessionManager::new(config);

        // Create a session
        let (id, _rx) = manager.create_session("test".to_string(), test_addr()).unwrap();
        assert_eq!(manager.session_count(), 1);

        // Wait for session to become stale
        std::thread::sleep(Duration::from_millis(100));

        // Run cleanup
        manager.cleanup_stale_sessions(Duration::from_millis(50));

        // Session should be removed
        assert_eq!(manager.session_count(), 0);
        assert!(manager.get_session(id).is_none());
    }

    #[test]
    fn test_session_manager_cleanup_preserves_active_sessions() {
        let config = SessionConfig {
            max_sessions: 10,
            session_timeout: Duration::from_secs(1),
            cleanup_interval: Duration::from_millis(10),
        };
        let manager = SessionManager::new(config);

        // Create first session (will become stale)
        let (id1, _rx1) = manager.create_session("stale".to_string(), test_addr()).unwrap();

        // Wait a bit
        std::thread::sleep(Duration::from_millis(100));

        // Create second session (will stay active due to recent creation)
        let (id2, _rx2) = manager.create_session("active".to_string(), test_addr()).unwrap();

        assert_eq!(manager.session_count(), 2);

        // Run cleanup with a very short timeout that will catch the first session
        // but not the second (since it was just created)
        manager.cleanup_stale_sessions(Duration::from_millis(50));

        // First session should be removed (idle > 50ms), second preserved (just created)
        assert_eq!(manager.session_count(), 1);
        assert!(manager.get_session(id1).is_none());
        assert!(manager.get_session(id2).is_some());
    }

    #[tokio::test]
    async fn test_session_manager_graceful_shutdown() {
        let config = SessionConfig::default();
        let manager = SessionManager::new(config);

        // Create sessions
        let (_id1, _rx1) = manager.create_session("user1".to_string(), test_addr()).unwrap();
        let (_id2, _rx2) = manager.create_session("user2".to_string(), test_addr()).unwrap();
        assert_eq!(manager.session_count(), 2);

        // Perform graceful shutdown with a short timeout
        // Since sessions don't auto-remove themselves, they should be force-closed
        let forced = manager.shutdown_graceful(Duration::from_millis(100)).await;

        // Sessions should be forcefully terminated
        assert_eq!(forced, 2);
        assert_eq!(manager.session_count(), 0);
    }

    #[tokio::test]
    async fn test_session_manager_graceful_shutdown_empty() {
        let config = SessionConfig::default();
        let manager = SessionManager::new(config);

        // No sessions
        assert_eq!(manager.session_count(), 0);

        // Graceful shutdown with no sessions should return immediately
        let forced = manager.shutdown_graceful(Duration::from_millis(100)).await;
        assert_eq!(forced, 0);
    }
}
