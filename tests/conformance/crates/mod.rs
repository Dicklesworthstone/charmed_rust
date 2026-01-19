//! Conformance tests for all Charmed Rust crates
//!
//! Each submodule contains conformance tests for a specific crate,
//! verifying that the Rust implementation matches the behavior of
//! the original Go library.

pub mod bubbles;
pub mod bubbletea;
pub mod charmed_log;
pub mod glamour;
pub mod harmonica;
pub mod huh;
pub mod lipgloss;
#[cfg(feature = "wish")]
pub mod wish;
