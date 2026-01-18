#![forbid(unsafe_code)]
// Allow pedantic lints for early-stage API ergonomics.
#![allow(clippy::nursery)]
#![allow(clippy::pedantic)]

//! # Glow
//!
//! A terminal-based markdown reader and browser.
//!
//! Glow provides a beautiful way to read markdown files directly in the terminal:
//! - Render local markdown files
//! - Browse and read GitHub READMEs
//! - Stash and organize documents
//! - Customizable pager controls
//!
//! ## Example
//!
//! ```rust,ignore
//! use glow::{Reader, Config};
//!
//! let config = Config::new()
//!     .pager(true)
//!     .width(80);
//!
//! let reader = Reader::new(config);
//! reader.read_file("README.md")?;
//! ```

/// Configuration for the markdown reader.
#[derive(Debug, Clone)]
pub struct Config {
    pager: bool,
    width: Option<usize>,
    style: String,
}

impl Config {
    /// Creates a new configuration with default settings.
    pub fn new() -> Self {
        Self {
            pager: true,
            width: None,
            style: "dark".to_string(),
        }
    }

    /// Enables or disables pager mode.
    pub fn pager(mut self, enabled: bool) -> Self {
        self.pager = enabled;
        self
    }

    /// Sets the output width.
    pub fn width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets the style theme.
    pub fn style(mut self, style: impl Into<String>) -> Self {
        self.style = style.into();
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Markdown file reader.
#[derive(Debug)]
pub struct Reader {
    config: Config,
}

impl Reader {
    /// Creates a new reader with the given configuration.
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Returns the reader configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Reads and renders a markdown file.
    pub fn read_file(&self, _path: &str) -> Result<String, std::io::Error> {
        // Placeholder implementation
        Ok(String::new())
    }
}

/// Stash for saving and organizing documents.
#[derive(Debug, Default)]
pub struct Stash {
    documents: Vec<String>,
}

impl Stash {
    /// Creates a new empty stash.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a document to the stash.
    pub fn add(&mut self, path: impl Into<String>) {
        self.documents.push(path.into());
    }

    /// Returns all stashed documents.
    pub fn documents(&self) -> &[String] {
        &self.documents
    }
}

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::{Config, Reader, Stash};
}
