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
//! - Browse local files
//! - Fetch GitHub READMEs (with the `github` feature)
//! - Stash and organize documents
//! - Customizable pager controls
//!
//! ## Quick start (library)
//!
//! ```rust,no_run
//! use glow::{Reader, Config};
//!
//! fn main() -> std::io::Result<()> {
//!     let config = Config::new()
//!         .pager(true)
//!         .width(80)
//!         .style("dark");
//!
//!     let reader = Reader::new(config);
//!     let rendered = reader.read_file("README.md")?;
//!     println!("{rendered}");
//!     Ok(())
//! }
//! ```
//!
//! ## CLI usage
//!
//! ```bash
//! glow README.md
//! glow --style light README.md
//! glow --width 80 README.md
//! glow --no-pager README.md
//! cat README.md | glow -
//! ```
//!
//! ## Feature flags
//!
//! - `github`: enable GitHub README fetching utilities
//! - `default`: core markdown rendering via `glamour`

pub mod browser;

#[cfg(feature = "github")]
pub mod github;

use std::io;
use std::path::Path;

use glamour::{Style as GlamourStyle, TermRenderer};

/// Configuration for the markdown reader.
///
/// Defaults:
/// - pager enabled
/// - width uses glamour's default word wrap
/// - style set to `"dark"`
///
/// # Example
///
/// ```rust
/// use glow::Config;
///
/// let config = Config::new()
///     .pager(false)
///     .width(80)
///     .style("light");
/// ```
#[derive(Debug, Clone)]
pub struct Config {
    pager: bool,
    width: Option<usize>,
    style: String,
    line_numbers: bool,
    preserve_newlines: bool,
}

impl Config {
    /// Creates a new configuration with default settings.
    pub fn new() -> Self {
        Self {
            pager: true,
            width: None,
            style: "dark".to_string(),
            line_numbers: false,
            preserve_newlines: false,
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

    /// Enables or disables line numbers in code blocks.
    pub fn line_numbers(mut self, enabled: bool) -> Self {
        self.line_numbers = enabled;
        self
    }

    /// Enables or disables preserving newlines in output.
    pub fn preserve_newlines(mut self, enabled: bool) -> Self {
        self.preserve_newlines = enabled;
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

    fn renderer(&self) -> io::Result<TermRenderer> {
        let style = self.glamour_style()?;
        let mut renderer = TermRenderer::new()
            .with_style(style)
            .with_preserved_newlines(self.preserve_newlines);
        if let Some(width) = self.width {
            renderer = renderer.with_word_wrap(width);
        }
        // line_numbers is only available with syntax-highlighting feature
        if self.line_numbers {
            #[cfg(feature = "syntax-highlighting")]
            {
                renderer = renderer.set_line_numbers(true);
            }
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
///
/// # Example
///
/// ```rust,no_run
/// use glow::{Config, Reader};
///
/// # fn main() -> std::io::Result<()> {
/// let reader = Reader::new(Config::new().width(80));
/// let output = reader.read_file("README.md")?;
/// println!("{output}");
/// # Ok(())
/// # }
/// ```
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
///
/// # Example
///
/// ```rust
/// use glow::Stash;
///
/// let mut stash = Stash::new();
/// stash.add("README.md");
/// stash.add("docs/guide.md");
/// assert_eq!(stash.documents().len(), 2);
/// ```
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
    pub use crate::browser::{BrowserConfig, Entry, FileBrowser, FileSelectedMsg};
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
    use super::*;

    // =========================================================================
    // Style Parsing Tests
    // =========================================================================

    #[test]
    fn parse_style_accepts_known_values() {
        let cases = ["dark", "light", "ascii", "pink", "auto", "no-tty", "no_tty"];
        for style in cases {
            assert!(parse_style(style).is_some(), "style {style} should parse");
        }
    }

    #[test]
    fn parse_style_is_case_insensitive() {
        assert!(parse_style("DARK").is_some());
        assert!(parse_style("Dark").is_some());
        assert!(parse_style("LIGHT").is_some());
        assert!(parse_style("NoTTY").is_some());
    }

    #[test]
    fn parse_style_trims_whitespace() {
        assert!(parse_style("  dark  ").is_some());
        assert!(parse_style("\tdark\n").is_some());
    }

    #[test]
    fn parse_style_returns_none_for_unknown() {
        assert!(parse_style("unknown").is_none());
        assert!(parse_style("").is_none());
        assert!(parse_style("dracula").is_none());
    }

    // =========================================================================
    // Config Tests
    // =========================================================================

    #[test]
    fn config_default_values() {
        let config = Config::new();
        assert!(config.pager);
        assert!(config.width.is_none());
        assert_eq!(config.style, "dark");
    }

    #[test]
    fn config_default_trait() {
        let config = Config::default();
        assert!(config.pager);
        assert_eq!(config.style, "dark");
    }

    #[test]
    fn config_pager_sets_value() {
        let config = Config::new().pager(false);
        assert!(!config.pager);

        let config = Config::new().pager(true);
        assert!(config.pager);
    }

    #[test]
    fn config_width_sets_value() {
        let config = Config::new().width(80);
        assert_eq!(config.width, Some(80));

        let config = Config::new().width(120);
        assert_eq!(config.width, Some(120));
    }

    #[test]
    fn config_style_sets_value() {
        let config = Config::new().style("light");
        assert_eq!(config.style, "light");

        let config = Config::new().style(String::from("pink"));
        assert_eq!(config.style, "pink");
    }

    #[test]
    fn config_builder_chaining() {
        let config = Config::new().pager(false).width(100).style("ascii");

        assert!(!config.pager);
        assert_eq!(config.width, Some(100));
        assert_eq!(config.style, "ascii");
    }

    #[test]
    fn config_glamour_style_valid() {
        let config = Config::new().style("dark");
        assert!(config.glamour_style().is_ok());
    }

    #[test]
    fn config_rejects_unknown_style() {
        let config = Config::new().style("unknown");
        let err = config.glamour_style().unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn config_renderer_creates_renderer() {
        let config = Config::new().style("dark").width(80);
        let result = config.renderer();
        assert!(result.is_ok());
    }

    #[test]
    fn config_renderer_fails_on_invalid_style() {
        let config = Config::new().style("invalid");
        let result = config.renderer();
        assert!(result.is_err());
    }

    // =========================================================================
    // Reader Tests
    // =========================================================================

    #[test]
    fn reader_new_stores_config() {
        let config = Config::new().style("light").width(100);
        let reader = Reader::new(config);

        assert_eq!(reader.config().style, "light");
        assert_eq!(reader.config().width, Some(100));
    }

    #[test]
    fn reader_render_markdown_basic() {
        let config = Config::new().style("dark");
        let reader = Reader::new(config);

        let result = reader.render_markdown("# Hello World");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_empty());
    }

    #[test]
    fn reader_render_markdown_empty_input() {
        let config = Config::new().style("dark");
        let reader = Reader::new(config);

        let result = reader.render_markdown("");
        assert!(result.is_ok());
    }

    #[test]
    fn reader_render_markdown_complex() {
        let config = Config::new().style("dark").width(80);
        let reader = Reader::new(config);

        let markdown = r#"
# Heading

Some **bold** and *italic* text.

- List item 1
- List item 2

```rust
fn main() {}
```
"#;

        let result = reader.render_markdown(markdown);
        assert!(result.is_ok());
    }

    #[test]
    fn reader_render_fails_on_invalid_style() {
        let config = Config::new().style("invalid");
        let reader = Reader::new(config);

        let result = reader.render_markdown("# Test");
        assert!(result.is_err());
    }

    #[test]
    fn reader_read_file_nonexistent() {
        let config = Config::new().style("dark");
        let reader = Reader::new(config);

        let result = reader.read_file("/nonexistent/path/file.md");
        assert!(result.is_err());
    }

    // =========================================================================
    // Stash Tests
    // =========================================================================

    #[test]
    fn stash_new_is_empty() {
        let stash = Stash::new();
        assert!(stash.documents().is_empty());
    }

    #[test]
    fn stash_default_is_empty() {
        let stash = Stash::default();
        assert!(stash.documents().is_empty());
    }

    #[test]
    fn stash_add_single_document() {
        let mut stash = Stash::new();
        stash.add("/path/to/file.md");

        assert_eq!(stash.documents().len(), 1);
        assert_eq!(stash.documents()[0], "/path/to/file.md");
    }

    #[test]
    fn stash_add_multiple_documents() {
        let mut stash = Stash::new();
        stash.add("file1.md");
        stash.add("file2.md");
        stash.add("file3.md");

        assert_eq!(stash.documents().len(), 3);
        assert_eq!(stash.documents(), &["file1.md", "file2.md", "file3.md"]);
    }

    #[test]
    fn stash_add_accepts_string() {
        let mut stash = Stash::new();
        stash.add(String::from("owned.md"));

        assert_eq!(stash.documents()[0], "owned.md");
    }

    #[test]
    fn stash_add_accepts_str() {
        let mut stash = Stash::new();
        stash.add("borrowed.md");

        assert_eq!(stash.documents()[0], "borrowed.md");
    }
}
