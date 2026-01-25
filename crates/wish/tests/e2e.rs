//! End-to-end SSH integration tests for wish.
//!
//! These tests require the `ssh` command to be available on the system.
//! Run with: `cargo test -p wish --test e2e -- --ignored` for stress tests.

#[path = "e2e/mod.rs"]
mod tests;
