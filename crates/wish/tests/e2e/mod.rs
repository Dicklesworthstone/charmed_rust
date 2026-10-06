//! End-to-end SSH integration tests for wish.

// Shared helpers; the russh exec client is used by other test suites.
#[allow(dead_code)]
#[path = "../common/mod.rs"]
mod common;

mod auth;
mod bubbletea;
mod connection;
mod perf;
mod pty;
