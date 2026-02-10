//! Session management for SSH connections.
//!
//! This module provides session lifecycle management including:
//! - Session tracking with unique IDs
//! - Activity tracking to prevent premature timeouts
//! - Idle session cleanup
//! - Graceful shutdown with session draining
//!
//! # Example
//!
//! ```rust,ignore
//! use wish::session::{SessionManager, SessionConfig};
//! use std::time::Duration;
//!
//! let config = SessionConfig {
//!     max_sessions: 100,
//!     session_timeout: Duration::from_secs(3600),
//!     cleanup_interval: Duration::from_secs(30),
//! };
//! let manager = SessionManager::new(config);
//!
//! // Create a session
//! let (id, shutdown_rx) = manager.create_session("user".to_string(), addr)?;
//!
//! // Track activity
//! manager.update_activity(id);
//!
//! // Clean up
//! manager.remove_session(id);
//! ```

mod handle;
mod manager;

pub use handle::{SessionHandle, SessionInfo};
pub use manager::{SessionConfig, SessionManager};
