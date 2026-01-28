#![forbid(unsafe_code)]

//! # Demo Showcase Library
//!
//! Flagship demonstration of all `charmed_rust` TUI capabilities.
//!
//! This module exposes the core types and utilities for the demo_showcase
//! application, enabling both the binary and integration tests to share code.
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
pub mod cli;
pub mod components;
pub mod config;
pub mod content;
pub mod data;
pub mod keymap;
pub mod messages;
pub mod pages;
pub mod shell_action;
#[cfg(feature = "ssh")]
pub mod ssh;
pub mod test_support;
pub mod theme;
