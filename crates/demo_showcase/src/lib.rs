#![forbid(unsafe_code)]

//! # Demo Showcase Library
//!
//! Flagship demonstration of all `charmed_rust` TUI capabilities.
//!
//! This module exposes the core types and utilities for the `demo_showcase`
//! application, enabling both the binary and integration tests to share code.
//!
//! ## Role in `charmed_rust`
//!
//! `demo_showcase` is the integration surface for the entire ecosystem:
//! - **bubbletea** drives the app runtime and input handling.
//! - **bubbles** and **huh** provide core UI components.
//! - **glamour** renders Markdown docs pages.
//! - **harmonica** powers smooth animations.
//! - **wish** (optional) runs the demo over SSH.
//!
//! ## Public Modules
//!
//! - [`app`] - Main application state and update logic
//! - [`config`] - Runtime configuration
//! - [`messages`] - Message types for event handling
//! - [`test_support`] - E2E test infrastructure
//! - [`shell_action`] - Terminal release/restore for pagers
//! - [`theme`] - Theme system and presets

pub mod app;
pub mod assets;
#[allow(clippy::doc_markdown)]
pub mod cli;
pub mod components;
pub mod config;
pub mod content;
#[allow(clippy::needless_pass_by_value, clippy::redundant_clone)]
pub mod data;
pub mod keymap;
pub mod messages;
pub mod pages;
pub mod shell_action;
#[cfg(feature = "ssh")]
#[allow(
    clippy::collapsible_if,
    clippy::match_wildcard_for_single_variants
)]
pub mod ssh;
pub mod test_support;
pub mod theme;
