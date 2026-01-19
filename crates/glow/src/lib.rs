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

use std::io;
use std::path::Path;

use glamour::{Renderer, Style as GlamourStyle};

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

    fn glamour_style(&self) -> io::Result<GlamourStyle> {
        parse_style(&self.style).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unknown style: {}", self.style),
            )
        })
    }

    fn renderer(&self) -> io::Result<Renderer> {
        let style = self.glamour_style()?;
        let mut renderer = Renderer::new().with_style(style);
        if let Some(width) = self.width {
            renderer = renderer.with_word_wrap(width);
        }
        Ok(renderer)
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
    pub fn read_file<P: AsRef<Path>>(&self, path: P) -> io::Result<String> {
        let markdown = std::fs::read_to_string(path)?;
        self.render_markdown(&markdown)
    }

    /// Renders markdown text using the configured renderer.
    pub fn render_markdown(&self, markdown: &str) -> io::Result<String> {
        let renderer = self.config.renderer()?;
        Ok(renderer.render(markdown))
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

fn parse_style(style: &str) -> Option<GlamourStyle> {
    match style.trim().to_ascii_lowercase().as_str() {
        "dark" => Some(GlamourStyle::Dark),
        "light" => Some(GlamourStyle::Light),
        "ascii" => Some(GlamourStyle::Ascii),
        "pink" => Some(GlamourStyle::Pink),
        "auto" => Some(GlamourStyle::Auto),
        "no-tty" | "notty" | "no_tty" => Some(GlamourStyle::NoTty),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, parse_style};

    #[test]
    fn parse_style_accepts_known_values() {
        let cases = ["dark", "light", "ascii", "pink", "auto", "no-tty", "no_tty"];
        for style in cases {
            assert!(parse_style(style).is_some(), "style {style} should parse");
        }
    }

    #[test]
    fn config_rejects_unknown_style() {
        let config = Config::new().style("unknown");
        let err = config.glamour_style().unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }
}
