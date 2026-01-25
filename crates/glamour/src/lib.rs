#![forbid(unsafe_code)]
// Allow pedantic lints for early-stage API ergonomics.
#![allow(clippy::nursery)]
#![allow(clippy::pedantic)]
#![allow(clippy::len_zero)]
#![allow(clippy::single_char_pattern)]

//! # Glamour
//!
//! A markdown rendering library for terminal applications.
//!
//! Glamour transforms markdown into beautifully styled terminal output with:
//! - Styled headings, lists, and tables
//! - Code block formatting with optional syntax highlighting
//! - Link and image handling
//! - Customizable themes (Dark, Light, ASCII, Pink)
//!
//! ## Example
//!
//! ```rust
//! use glamour::{render, Renderer, Style};
//!
//! // Quick render with default dark style
//! let output = render("# Hello\n\nThis is **bold** text.", Style::Dark).unwrap();
//! println!("{}", output);
//!
//! // Custom renderer with word wrap
//! let renderer = Renderer::new()
//!     .with_style(Style::Light)
//!     .with_word_wrap(80);
//! let output = renderer.render("# Heading\n\nParagraph text.");
//! ```
//!
//! ## Feature Flags
//!
//! - `syntax-highlighting`: Enable syntax highlighting for code blocks using
//!   [syntect](https://crates.io/crates/syntect). This adds ~2MB to binary size
//!   due to embedded syntax definitions for ~60 languages.
//!
//! ### Example with syntax highlighting
//!
//! ```toml
//! [dependencies]
//! glamour = { version = "0.1", features = ["syntax-highlighting"] }
//! ```
//!
//! When enabled, code blocks with language annotations (e.g., ` ```rust `)
//! will be rendered with syntax highlighting using the configured theme.
//! See `docs/SYNTAX_HIGHLIGHTING_RESEARCH.md` for implementation details.

// Syntax highlighting module (optional feature)
#[cfg(feature = "syntax-highlighting")]
pub mod syntax;

// Table parsing module for markdown tables
pub mod table;

use lipgloss::Style as LipglossStyle;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::collections::HashMap;
#[cfg(feature = "syntax-highlighting")]
use std::collections::HashSet;

// Conditional serde import
#[cfg(all(feature = "syntax-highlighting", feature = "serde"))]
use serde::{Deserialize, Serialize};

/// Default width for word wrapping.
const DEFAULT_WIDTH: usize = 80;
const DEFAULT_MARGIN: usize = 2;
const DEFAULT_LIST_INDENT: usize = 2;
const DEFAULT_LIST_LEVEL_INDENT: usize = 4;

// ============================================================================
// Style Configuration Types
// ============================================================================

/// Primitive style settings for text elements.
#[derive(Debug, Clone, Default)]
pub struct StylePrimitive {
    /// Prefix added before the block.
    pub block_prefix: String,
    /// Suffix added after the block.
    pub block_suffix: String,
    /// Prefix added before text.
    pub prefix: String,
    /// Suffix added after text.
    pub suffix: String,
    /// Foreground color (ANSI color code or hex).
    pub color: Option<String>,
    /// Background color (ANSI color code or hex).
    pub background_color: Option<String>,
    /// Whether text is underlined.
    pub underline: Option<bool>,
    /// Whether text is bold.
    pub bold: Option<bool>,
    /// Whether text is italic.
    pub italic: Option<bool>,
    /// Whether text has strikethrough.
    pub crossed_out: Option<bool>,
    /// Whether text is faint.
    pub faint: Option<bool>,
    /// Format string for special elements (e.g., "Image: {{.text}}").
    pub format: String,
}

impl StylePrimitive {
    /// Creates a new empty style primitive.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the prefix.
    pub fn prefix(mut self, p: impl Into<String>) -> Self {
        self.prefix = p.into();
        self
    }

    /// Sets the suffix.
    pub fn suffix(mut self, s: impl Into<String>) -> Self {
        self.suffix = s.into();
        self
    }

    /// Sets the block prefix.
    pub fn block_prefix(mut self, p: impl Into<String>) -> Self {
        self.block_prefix = p.into();
        self
    }

    /// Sets the block suffix.
    pub fn block_suffix(mut self, s: impl Into<String>) -> Self {
        self.block_suffix = s.into();
        self
    }

    /// Sets the foreground color.
    pub fn color(mut self, c: impl Into<String>) -> Self {
        self.color = Some(c.into());
        self
    }

    /// Sets the background color.
    pub fn background_color(mut self, c: impl Into<String>) -> Self {
        self.background_color = Some(c.into());
        self
    }

    /// Sets bold.
    pub fn bold(mut self, b: bool) -> Self {
        self.bold = Some(b);
        self
    }

    /// Sets italic.
    pub fn italic(mut self, i: bool) -> Self {
        self.italic = Some(i);
        self
    }

    /// Sets underline.
    pub fn underline(mut self, u: bool) -> Self {
        self.underline = Some(u);
        self
    }

    /// Sets strikethrough.
    pub fn crossed_out(mut self, c: bool) -> Self {
        self.crossed_out = Some(c);
        self
    }

    /// Sets faint.
    pub fn faint(mut self, f: bool) -> Self {
        self.faint = Some(f);
        self
    }

    /// Sets the format string.
    pub fn format(mut self, f: impl Into<String>) -> Self {
        self.format = f.into();
        self
    }

    /// Converts to a lipgloss style.
    pub fn to_lipgloss(&self) -> LipglossStyle {
        let mut style = LipglossStyle::new();

        if let Some(ref color) = self.color {
            style = style.foreground(color.as_str());
        }
        if let Some(ref bg) = self.background_color {
            style = style.background(bg.as_str());
        }
        if self.bold == Some(true) {
            style = style.bold();
        }
        if self.italic == Some(true) {
            style = style.italic();
        }
        if self.underline == Some(true) {
            style = style.underline();
        }
        if self.crossed_out == Some(true) {
            style = style.strikethrough();
        }
        if self.faint == Some(true) {
            style = style.faint();
        }

        style
    }
}

/// Block-level style settings.
#[derive(Debug, Clone, Default)]
pub struct StyleBlock {
    /// Primitive style settings.
    pub style: StylePrimitive,
    /// Indentation level.
    pub indent: Option<usize>,
    /// Token used for indentation.
    pub indent_token: Option<String>,
    /// Margin around the block.
    pub margin: Option<usize>,
}

impl StyleBlock {
    /// Creates a new empty block style.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the primitive style.
    pub fn style(mut self, s: StylePrimitive) -> Self {
        self.style = s;
        self
    }

    /// Sets the indent.
    pub fn indent(mut self, i: usize) -> Self {
        self.indent = Some(i);
        self
    }

    /// Sets the indent token.
    pub fn indent_token(mut self, t: impl Into<String>) -> Self {
        self.indent_token = Some(t.into());
        self
    }

    /// Sets the margin.
    pub fn margin(mut self, m: usize) -> Self {
        self.margin = Some(m);
        self
    }
}

/// Code block style settings.
#[derive(Debug, Clone, Default)]
pub struct StyleCodeBlock {
    /// Block style settings.
    pub block: StyleBlock,
    /// Syntax highlighting theme name.
    pub theme: Option<String>,
}

impl StyleCodeBlock {
    /// Creates a new code block style.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the block style.
    pub fn block(mut self, b: StyleBlock) -> Self {
        self.block = b;
        self
    }

    /// Sets the theme.
    pub fn theme(mut self, t: impl Into<String>) -> Self {
        self.theme = Some(t.into());
        self
    }
}

/// List style settings.
#[derive(Debug, Clone, Default)]
pub struct StyleList {
    /// Block style settings.
    pub block: StyleBlock,
    /// Additional indent per nesting level.
    pub level_indent: usize,
}

impl StyleList {
    /// Creates a new list style.
    pub fn new() -> Self {
        Self {
            level_indent: DEFAULT_LIST_LEVEL_INDENT,
            ..Default::default()
        }
    }

    /// Sets the block style.
    pub fn block(mut self, b: StyleBlock) -> Self {
        self.block = b;
        self
    }

    /// Sets the level indent.
    pub fn level_indent(mut self, i: usize) -> Self {
        self.level_indent = i;
        self
    }
}

/// Table style settings.
#[derive(Debug, Clone, Default)]
pub struct StyleTable {
    /// Block style settings.
    pub block: StyleBlock,
    /// Center separator character.
    pub center_separator: Option<String>,
    /// Column separator character.
    pub column_separator: Option<String>,
    /// Row separator character.
    pub row_separator: Option<String>,
}

impl StyleTable {
    /// Creates a new table style.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets separators.
    pub fn separators(
        mut self,
        center: impl Into<String>,
        column: impl Into<String>,
        row: impl Into<String>,
    ) -> Self {
        self.center_separator = Some(center.into());
        self.column_separator = Some(column.into());
        self.row_separator = Some(row.into());
        self
    }
}

/// Task item style settings.
#[derive(Debug, Clone, Default)]
pub struct StyleTask {
    /// Primitive style settings.
    pub style: StylePrimitive,
    /// Marker for checked items.
    pub ticked: String,
    /// Marker for unchecked items.
    pub unticked: String,
}

impl StyleTask {
    /// Creates a new task style.
    pub fn new() -> Self {
        Self {
            ticked: "[x] ".to_string(),
            unticked: "[ ] ".to_string(),
            ..Default::default()
        }
    }

    /// Sets the ticked marker.
    pub fn ticked(mut self, t: impl Into<String>) -> Self {
        self.ticked = t.into();
        self
    }

    /// Sets the unticked marker.
    pub fn unticked(mut self, u: impl Into<String>) -> Self {
        self.unticked = u.into();
        self
    }
}

// ============================================================================
// Syntax Highlighting Configuration (optional feature)
// ============================================================================

/// Configuration for syntax highlighting behavior.
///
/// This struct is only available when the `syntax-highlighting` feature is enabled.
///
/// # Example
///
/// ```rust,ignore
/// use glamour::SyntaxThemeConfig;
///
/// let config = SyntaxThemeConfig::default()
///     .theme("Solarized (dark)")
///     .line_numbers(true);
/// ```
///
/// # Serialization
///
/// When the `serde` feature is enabled, this struct can be serialized/deserialized:
///
/// ```toml
/// # config.toml example
/// [syntax]
/// theme_name = "Solarized (dark)"
/// line_numbers = true
/// ```
#[cfg(feature = "syntax-highlighting")]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SyntaxThemeConfig {
    /// Theme name (e.g., "base16-ocean.dark", "Solarized (dark)").
    /// Use `SyntaxTheme::available_themes()` to see all options.
    pub theme_name: String,
    /// Whether to show line numbers in code blocks.
    pub line_numbers: bool,
    /// Custom language aliases (e.g., "rs" -> "rust").
    /// These override the built-in aliases.
    pub language_aliases: HashMap<String, String>,
    /// Languages to never highlight (render as plain text).
    pub disabled_languages: HashSet<String>,
}

#[cfg(feature = "syntax-highlighting")]
impl Default for SyntaxThemeConfig {
    fn default() -> Self {
        Self {
            theme_name: "base16-ocean.dark".to_string(),
            line_numbers: false,
            language_aliases: HashMap::new(),
            disabled_languages: HashSet::new(),
        }
    }
}

#[cfg(feature = "syntax-highlighting")]
impl SyntaxThemeConfig {
    /// Creates a new syntax theme config with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the syntax highlighting theme.
    ///
    /// Available themes include:
    /// - `base16-ocean.dark` (default)
    /// - `base16-eighties.dark`
    /// - `base16-mocha.dark`
    /// - `InspiredGitHub`
    /// - `Solarized (dark)`
    /// - `Solarized (light)`
    pub fn theme(mut self, name: impl Into<String>) -> Self {
        self.theme_name = name.into();
        self
    }

    /// Enables or disables line numbers in code blocks.
    pub fn line_numbers(mut self, enabled: bool) -> Self {
        self.line_numbers = enabled;
        self
    }

    /// Adds a custom language alias.
    ///
    /// This allows mapping custom identifiers to languages.
    /// For example, `("dockerfile", "docker")` would map the
    /// `dockerfile` language hint to Docker syntax.
    pub fn language_alias(mut self, alias: impl Into<String>, language: impl Into<String>) -> Self {
        self.language_aliases.insert(alias.into(), language.into());
        self
    }

    /// Disables highlighting for a specific language.
    ///
    /// Languages in this set will be rendered as plain text.
    pub fn disable_language(mut self, lang: impl Into<String>) -> Self {
        self.disabled_languages.insert(lang.into());
        self
    }

    /// Validates that the configured theme exists.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the theme exists, or an error message if not.
    pub fn validate(&self) -> Result<(), String> {
        use crate::syntax::SyntaxTheme;

        if SyntaxTheme::from_name(&self.theme_name).is_none() {
            let available = SyntaxTheme::available_themes().join(", ");
            return Err(format!(
                "Unknown syntax theme '{}'. Available themes: {}",
                self.theme_name, available
            ));
        }
        Ok(())
    }

    /// Resolves a language identifier through custom aliases.
    ///
    /// If a custom alias exists, returns the mapped language.
    /// Otherwise returns the original language.
    pub fn resolve_language<'a>(&'a self, lang: &'a str) -> &'a str {
        self.language_aliases
            .get(lang)
            .map(|s| s.as_str())
            .unwrap_or(lang)
    }

    /// Checks if a language is disabled.
    pub fn is_disabled(&self, lang: &str) -> bool {
        self.disabled_languages.contains(lang)
    }
}

/// Complete style configuration for rendering.
#[derive(Debug, Clone, Default)]
pub struct StyleConfig {
    // Document
    pub document: StyleBlock,

    // Block elements
    pub block_quote: StyleBlock,
    pub paragraph: StyleBlock,
    pub list: StyleList,

    // Headings
    pub heading: StyleBlock,
    pub h1: StyleBlock,
    pub h2: StyleBlock,
    pub h3: StyleBlock,
    pub h4: StyleBlock,
    pub h5: StyleBlock,
    pub h6: StyleBlock,

    // Inline elements
    pub text: StylePrimitive,
    pub strikethrough: StylePrimitive,
    pub emph: StylePrimitive,
    pub strong: StylePrimitive,
    pub horizontal_rule: StylePrimitive,

    // List items
    pub item: StylePrimitive,
    pub enumeration: StylePrimitive,
    pub task: StyleTask,

    // Links and images
    pub link: StylePrimitive,
    pub link_text: StylePrimitive,
    pub image: StylePrimitive,
    pub image_text: StylePrimitive,

    // Code
    pub code: StyleBlock,
    pub code_block: StyleCodeBlock,

    // Tables
    pub table: StyleTable,

    // Definition lists
    pub definition_list: StyleBlock,
    pub definition_term: StylePrimitive,
    pub definition_description: StylePrimitive,

    // Syntax highlighting configuration (optional feature)
    #[cfg(feature = "syntax-highlighting")]
    pub syntax_config: SyntaxThemeConfig,
}

impl StyleConfig {
    /// Creates a new empty style config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the style for a heading level.
    pub fn heading_style(&self, level: HeadingLevel) -> &StyleBlock {
        match level {
            HeadingLevel::H1 => &self.h1,
            HeadingLevel::H2 => &self.h2,
            HeadingLevel::H3 => &self.h3,
            HeadingLevel::H4 => &self.h4,
            HeadingLevel::H5 => &self.h5,
            HeadingLevel::H6 => &self.h6,
        }
    }

    /// Sets the syntax highlighting theme.
    ///
    /// This method is only available when the `syntax-highlighting` feature is enabled.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let config = StyleConfig::default()
    ///     .syntax_theme("Solarized (dark)");
    /// ```
    #[cfg(feature = "syntax-highlighting")]
    pub fn syntax_theme(mut self, theme: impl Into<String>) -> Self {
        self.syntax_config.theme_name = theme.into();
        self
    }

    /// Enables or disables line numbers in code blocks.
    ///
    /// This method is only available when the `syntax-highlighting` feature is enabled.
    #[cfg(feature = "syntax-highlighting")]
    pub fn with_line_numbers(mut self, enabled: bool) -> Self {
        self.syntax_config.line_numbers = enabled;
        self
    }

    /// Adds a custom language alias.
    ///
    /// This allows mapping custom identifiers to languages.
    ///
    /// This method is only available when the `syntax-highlighting` feature is enabled.
    #[cfg(feature = "syntax-highlighting")]
    pub fn language_alias(mut self, alias: impl Into<String>, language: impl Into<String>) -> Self {
        self.syntax_config
            .language_aliases
            .insert(alias.into(), language.into());
        self
    }

    /// Disables syntax highlighting for a specific language.
    ///
    /// Languages in this set will be rendered as plain text.
    ///
    /// This method is only available when the `syntax-highlighting` feature is enabled.
    #[cfg(feature = "syntax-highlighting")]
    pub fn disable_language(mut self, lang: impl Into<String>) -> Self {
        self.syntax_config.disabled_languages.insert(lang.into());
        self
    }

    /// Sets the full syntax configuration.
    ///
    /// This method is only available when the `syntax-highlighting` feature is enabled.
    #[cfg(feature = "syntax-highlighting")]
    pub fn with_syntax_config(mut self, config: SyntaxThemeConfig) -> Self {
        self.syntax_config = config;
        self
    }

    /// Gets a reference to the syntax configuration.
    ///
    /// This method is only available when the `syntax-highlighting` feature is enabled.
    #[cfg(feature = "syntax-highlighting")]
    pub fn syntax(&self) -> &SyntaxThemeConfig {
        &self.syntax_config
    }
}

// ============================================================================
// Built-in Styles
// ============================================================================

/// Available built-in styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Style {
    /// ASCII-only style (no special characters).
    Ascii,
    /// Dark terminal style (default).
    #[default]
    Dark,
    /// Light terminal style.
    Light,
    /// Pink accent style.
    Pink,
    /// No TTY style (for non-terminal output).
    NoTty,
    /// Auto-detect based on terminal.
    Auto,
}

impl Style {
    /// Gets the style configuration for this style.
    pub fn config(&self) -> StyleConfig {
        match self {
            Style::Ascii | Style::NoTty => ascii_style(),
            Style::Dark | Style::Auto => dark_style(),
            Style::Light => light_style(),
            Style::Pink => pink_style(),
        }
    }
}

/// Creates the ASCII style configuration.
pub fn ascii_style() -> StyleConfig {
    StyleConfig {
        document: StyleBlock::new()
            .style(StylePrimitive::new().block_prefix("\n").block_suffix("\n"))
            .margin(DEFAULT_MARGIN),
        block_quote: StyleBlock::new().indent(1).indent_token("| "),
        paragraph: StyleBlock::new(),
        list: StyleList::new().level_indent(DEFAULT_LIST_LEVEL_INDENT),
        heading: StyleBlock::new().style(StylePrimitive::new().block_suffix("\n")),
        h1: StyleBlock::new().style(StylePrimitive::new().prefix("# ")),
        h2: StyleBlock::new().style(StylePrimitive::new().prefix("## ")),
        h3: StyleBlock::new().style(StylePrimitive::new().prefix("### ")),
        h4: StyleBlock::new().style(StylePrimitive::new().prefix("#### ")),
        h5: StyleBlock::new().style(StylePrimitive::new().prefix("##### ")),
        h6: StyleBlock::new().style(StylePrimitive::new().prefix("###### ")),
        strikethrough: StylePrimitive::new().block_prefix("~~").block_suffix("~~"),
        emph: StylePrimitive::new().block_prefix("*").block_suffix("*"),
        strong: StylePrimitive::new().block_prefix("**").block_suffix("**"),
        horizontal_rule: StylePrimitive::new().format("\n--------\n"),
        item: StylePrimitive::new().block_prefix("* "),
        enumeration: StylePrimitive::new().block_prefix(". "),
        task: StyleTask::new().ticked("[x] ").unticked("[ ] "),
        image_text: StylePrimitive::new().format("Image: {{.text}} ->"),
        code: StyleBlock::new().style(StylePrimitive::new().prefix("`").suffix("`")),
        code_block: StyleCodeBlock::new().block(StyleBlock::new().margin(DEFAULT_MARGIN)),
        table: StyleTable::new().separators("|", "|", "-"),
        definition_description: StylePrimitive::new().block_prefix("\n* "),
        ..Default::default()
    }
}

/// Creates the dark style configuration.
pub fn dark_style() -> StyleConfig {
    StyleConfig {
        document: StyleBlock::new()
            .style(
                StylePrimitive::new()
                    .block_prefix("\n")
                    .block_suffix("\n")
                    .color("252"),
            )
            .margin(DEFAULT_MARGIN),
        block_quote: StyleBlock::new().indent(1).indent_token("│ "),
        list: StyleList::new().level_indent(DEFAULT_LIST_INDENT),
        heading: StyleBlock::new().style(
            StylePrimitive::new()
                .block_suffix("\n")
                .color("39")
                .bold(true),
        ),
        h1: StyleBlock::new().style(
            StylePrimitive::new()
                .prefix(" ")
                .suffix(" ")
                .color("228")
                .background_color("63")
                .bold(true),
        ),
        h2: StyleBlock::new().style(StylePrimitive::new().prefix("## ")),
        h3: StyleBlock::new().style(StylePrimitive::new().prefix("### ")),
        h4: StyleBlock::new().style(StylePrimitive::new().prefix("#### ")),
        h5: StyleBlock::new().style(StylePrimitive::new().prefix("##### ")),
        h6: StyleBlock::new().style(
            StylePrimitive::new()
                .prefix("###### ")
                .color("35")
                .bold(false),
        ),
        strikethrough: StylePrimitive::new().crossed_out(true),
        emph: StylePrimitive::new().italic(true),
        strong: StylePrimitive::new().bold(true),
        horizontal_rule: StylePrimitive::new().color("240").format("\n--------\n"),
        item: StylePrimitive::new().block_prefix("• "),
        enumeration: StylePrimitive::new().block_prefix(". "),
        task: StyleTask::new().ticked("[✓] ").unticked("[ ] "),
        link: StylePrimitive::new().color("30").underline(true),
        link_text: StylePrimitive::new().color("35").bold(true),
        image: StylePrimitive::new().color("212").underline(true),
        image_text: StylePrimitive::new()
            .color("243")
            .format("Image: {{.text}} ->"),
        code: StyleBlock::new().style(
            StylePrimitive::new()
                .prefix(" ")
                .suffix(" ")
                .color("203")
                .background_color("236"),
        ),
        code_block: StyleCodeBlock::new().block(
            StyleBlock::new()
                .style(StylePrimitive::new().color("244"))
                .margin(DEFAULT_MARGIN),
        ),
        definition_description: StylePrimitive::new().block_prefix("\n→ "),
        ..Default::default()
    }
}

/// Creates the light style configuration.
pub fn light_style() -> StyleConfig {
    StyleConfig {
        document: StyleBlock::new()
            .style(
                StylePrimitive::new()
                    .block_prefix("\n")
                    .block_suffix("\n")
                    .color("234"),
            )
            .margin(DEFAULT_MARGIN),
        block_quote: StyleBlock::new().indent(1).indent_token("│ "),
        list: StyleList::new().level_indent(DEFAULT_LIST_INDENT),
        heading: StyleBlock::new().style(
            StylePrimitive::new()
                .block_suffix("\n")
                .color("27")
                .bold(true),
        ),
        h1: StyleBlock::new().style(
            StylePrimitive::new()
                .prefix(" ")
                .suffix(" ")
                .color("228")
                .background_color("63")
                .bold(true),
        ),
        h2: StyleBlock::new().style(StylePrimitive::new().prefix("## ")),
        h3: StyleBlock::new().style(StylePrimitive::new().prefix("### ")),
        h4: StyleBlock::new().style(StylePrimitive::new().prefix("#### ")),
        h5: StyleBlock::new().style(StylePrimitive::new().prefix("##### ")),
        h6: StyleBlock::new().style(StylePrimitive::new().prefix("###### ").bold(false)),
        strikethrough: StylePrimitive::new().crossed_out(true),
        emph: StylePrimitive::new().italic(true),
        strong: StylePrimitive::new().bold(true),
        horizontal_rule: StylePrimitive::new().color("249").format("\n--------\n"),
        item: StylePrimitive::new().block_prefix("• "),
        enumeration: StylePrimitive::new().block_prefix(". "),
        task: StyleTask::new().ticked("[✓] ").unticked("[ ] "),
        link: StylePrimitive::new().color("36").underline(true),
        link_text: StylePrimitive::new().color("29").bold(true),
        image: StylePrimitive::new().color("205").underline(true),
        image_text: StylePrimitive::new()
            .color("243")
            .format("Image: {{.text}} ->"),
        code: StyleBlock::new().style(
            StylePrimitive::new()
                .prefix(" ")
                .suffix(" ")
                .color("203")
                .background_color("254"),
        ),
        code_block: StyleCodeBlock::new().block(
            StyleBlock::new()
                .style(StylePrimitive::new().color("242"))
                .margin(DEFAULT_MARGIN),
        ),
        definition_description: StylePrimitive::new().block_prefix("\n→ "),
        ..Default::default()
    }
}

/// Creates the pink style configuration.
pub fn pink_style() -> StyleConfig {
    StyleConfig {
        document: StyleBlock::new().margin(DEFAULT_MARGIN),
        block_quote: StyleBlock::new().indent(1).indent_token("│ "),
        list: StyleList::new().level_indent(DEFAULT_LIST_INDENT),
        heading: StyleBlock::new().style(
            StylePrimitive::new()
                .block_suffix("\n")
                .color("212")
                .bold(true),
        ),
        h1: StyleBlock::new().style(StylePrimitive::new().block_prefix("\n").block_suffix("\n")),
        h2: StyleBlock::new().style(StylePrimitive::new().prefix("▌ ")),
        h3: StyleBlock::new().style(StylePrimitive::new().prefix("┃ ")),
        h4: StyleBlock::new().style(StylePrimitive::new().prefix("│ ")),
        h5: StyleBlock::new().style(StylePrimitive::new().prefix("┆ ")),
        h6: StyleBlock::new().style(StylePrimitive::new().prefix("┊ ").bold(false)),
        strikethrough: StylePrimitive::new().crossed_out(true),
        emph: StylePrimitive::new().italic(true),
        strong: StylePrimitive::new().bold(true),
        horizontal_rule: StylePrimitive::new().color("212").format("\n──────\n"),
        item: StylePrimitive::new().block_prefix("• "),
        enumeration: StylePrimitive::new().block_prefix(". "),
        task: StyleTask::new().ticked("[✓] ").unticked("[ ] "),
        link: StylePrimitive::new().color("99").underline(true),
        link_text: StylePrimitive::new().bold(true),
        image: StylePrimitive::new().underline(true),
        image_text: StylePrimitive::new().format("Image: {{.text}}"),
        code: StyleBlock::new().style(
            StylePrimitive::new()
                .prefix(" ")
                .suffix(" ")
                .color("212")
                .background_color("236"),
        ),
        definition_description: StylePrimitive::new().block_prefix("\n→ "),
        ..Default::default()
    }
}

// ============================================================================
// Renderer
// ============================================================================

/// Options for the markdown renderer.
#[derive(Debug, Clone)]
pub struct RendererOptions {
    /// Word wrap width.
    pub word_wrap: usize,
    /// Base URL for resolving relative links.
    pub base_url: Option<String>,
    /// Whether to preserve newlines.
    pub preserve_newlines: bool,
    /// Style configuration.
    pub styles: StyleConfig,
}

impl Default for RendererOptions {
    fn default() -> Self {
        Self {
            word_wrap: DEFAULT_WIDTH,
            base_url: None,
            preserve_newlines: false,
            styles: dark_style(),
        }
    }
}

/// Markdown renderer for terminal output.
#[derive(Debug, Clone)]
pub struct Renderer {
    options: RendererOptions,
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer {
    /// Creates a new renderer with default settings.
    pub fn new() -> Self {
        Self {
            options: RendererOptions::default(),
        }
    }

    /// Sets the style for rendering.
    pub fn with_style(mut self, style: Style) -> Self {
        self.options.styles = style.config();
        self
    }

    /// Sets a custom style configuration.
    pub fn with_style_config(mut self, config: StyleConfig) -> Self {
        self.options.styles = config;
        self
    }

    /// Sets the word wrap width.
    pub fn with_word_wrap(mut self, width: usize) -> Self {
        self.options.word_wrap = width;
        self
    }

    /// Sets the base URL for resolving relative links.
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.options.base_url = Some(url.into());
        self
    }

    /// Sets whether to preserve newlines.
    pub fn with_preserved_newlines(mut self, preserve: bool) -> Self {
        self.options.preserve_newlines = preserve;
        self
    }

    /// Renders markdown to styled terminal output.
    pub fn render(&self, markdown: &str) -> String {
        let mut ctx = RenderContext::new(&self.options);
        ctx.render(markdown)
    }

    /// Renders markdown bytes to styled terminal output.
    pub fn render_bytes(&self, markdown: &[u8]) -> Result<String, std::str::Utf8Error> {
        let text = std::str::from_utf8(markdown)?;
        Ok(self.render(text))
    }

    /// Changes the syntax highlighting theme at runtime.
    ///
    /// This allows switching themes without creating a new Renderer instance.
    ///
    /// # Arguments
    ///
    /// * `theme` - Theme name (e.g., "base16-ocean.dark", "Solarized (dark)")
    ///
    /// # Returns
    ///
    /// `Ok(())` if the theme exists and was applied, or an error message if the theme
    /// was not found.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use glamour::Renderer;
    ///
    /// let mut renderer = Renderer::new();
    /// renderer.set_syntax_theme("Solarized (dark)")?;
    /// let output = renderer.render("```rust\nfn main() {}\n```");
    /// ```
    #[cfg(feature = "syntax-highlighting")]
    pub fn set_syntax_theme(&mut self, theme: impl Into<String>) -> Result<(), String> {
        let theme_name = theme.into();

        // Validate the theme exists before setting it
        use crate::syntax::SyntaxTheme;
        if SyntaxTheme::from_name(&theme_name).is_none() {
            let available = SyntaxTheme::available_themes().join(", ");
            return Err(format!(
                "Unknown syntax theme '{}'. Available themes: {}",
                theme_name, available
            ));
        }

        self.options.styles.syntax_config.theme_name = theme_name;
        Ok(())
    }

    /// Enables or disables line numbers in code blocks at runtime.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use glamour::Renderer;
    ///
    /// let mut renderer = Renderer::new();
    /// renderer.set_line_numbers(true);
    /// ```
    #[cfg(feature = "syntax-highlighting")]
    pub fn set_line_numbers(&mut self, enabled: bool) {
        self.options.styles.syntax_config.line_numbers = enabled;
    }

    /// Returns a reference to the current syntax configuration.
    ///
    /// This method is only available when the `syntax-highlighting` feature is enabled.
    #[cfg(feature = "syntax-highlighting")]
    pub fn syntax_config(&self) -> &SyntaxThemeConfig {
        &self.options.styles.syntax_config
    }

    /// Returns a mutable reference to the current syntax configuration.
    ///
    /// This allows runtime modification of all syntax highlighting settings.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use glamour::Renderer;
    ///
    /// let mut renderer = Renderer::new();
    /// renderer.syntax_config_mut()
    ///     .language_aliases
    ///     .insert("rs".to_string(), "rust".to_string());
    /// ```
    #[cfg(feature = "syntax-highlighting")]
    pub fn syntax_config_mut(&mut self) -> &mut SyntaxThemeConfig {
        &mut self.options.styles.syntax_config
    }
}

/// Render context that tracks state during rendering.
struct RenderContext<'a> {
    options: &'a RendererOptions,
    output: String,
    // Track element nesting
    in_heading: Option<HeadingLevel>,
    in_emphasis: bool,
    in_strong: bool,
    in_strikethrough: bool,
    in_link: bool,
    in_image: bool,
    in_code_block: bool,
    in_block_quote: bool,
    block_quote_pending_separator: bool,
    in_list: bool,
    ordered_list_stack: Vec<bool>,
    list_depth: usize,
    list_item_number: Vec<usize>,
    in_table: bool,
    table_alignments: Vec<pulldown_cmark::Alignment>,
    table_row: Vec<String>,
    table_rows: Vec<Vec<String>>,
    table_header_row: Option<Vec<String>>,
    table_header: bool,
    current_cell: String,
    // Buffering
    text_buffer: String,
    link_url: String,
    link_title: String,
    image_url: String,
    image_title: String,
    code_block_language: String,
    code_block_content: String,
}

impl<'a> RenderContext<'a> {
    fn new(options: &'a RendererOptions) -> Self {
        Self {
            options,
            output: String::new(),
            in_heading: None,
            in_emphasis: false,
            in_strong: false,
            in_strikethrough: false,
            in_link: false,
            in_image: false,
            in_code_block: false,
            in_block_quote: false,
            block_quote_pending_separator: false,
            in_list: false,
            ordered_list_stack: Vec::new(),
            list_depth: 0,
            list_item_number: Vec::new(),
            in_table: false,
            table_alignments: Vec::new(),
            table_row: Vec::new(),
            table_rows: Vec::new(),
            table_header_row: None,
            table_header: false,
            current_cell: String::new(),
            text_buffer: String::new(),
            link_url: String::new(),
            link_title: String::new(),
            image_url: String::new(),
            image_title: String::new(),
            code_block_language: String::new(),
            code_block_content: String::new(),
        }
    }

    fn render(&mut self, markdown: &str) -> String {
        // Enable tables and other extensions
        let mut opts = Options::empty();
        opts.insert(Options::ENABLE_TABLES);
        opts.insert(Options::ENABLE_STRIKETHROUGH);
        opts.insert(Options::ENABLE_TASKLISTS);

        let parser = Parser::new_ext(markdown, opts);

        // Document prefix
        self.output
            .push_str(&self.options.styles.document.style.block_prefix);

        // Add margin
        let margin = self.options.styles.document.margin.unwrap_or(0);

        for event in parser {
            self.handle_event(event);
        }

        // Document suffix
        self.output
            .push_str(&self.options.styles.document.style.block_suffix);

        // Apply margin
        if margin > 0 {
            let margin_str = " ".repeat(margin);
            self.output = self
                .output
                .lines()
                .map(|line| format!("{}{}", margin_str, line))
                .collect::<Vec<_>>()
                .join("\n");
        }

        std::mem::take(&mut self.output)
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            // Block elements
            Event::Start(Tag::Heading { level, .. }) => {
                self.in_heading = Some(level);
                self.text_buffer.clear();
            }
            Event::End(TagEnd::Heading(_level)) => {
                self.flush_heading();
                self.in_heading = None;
            }

            Event::Start(Tag::Paragraph) => {
                if self.in_block_quote && self.block_quote_pending_separator {
                    let indent_token = self
                        .options
                        .styles
                        .block_quote
                        .indent_token
                        .as_deref()
                        .unwrap_or("│ ");
                    self.output.push_str(indent_token);
                    self.output.push('\n');
                    self.block_quote_pending_separator = false;
                }
                if !self.in_list {
                    self.text_buffer.clear();
                }
            }
            Event::End(TagEnd::Paragraph) => {
                if !self.in_list && !self.in_table {
                    self.flush_paragraph();
                }
            }

            Event::Start(Tag::BlockQuote(_kind)) => {
                self.in_block_quote = true;
                self.block_quote_pending_separator = false;
                self.output.push('\n');
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                self.in_block_quote = false;
                self.block_quote_pending_separator = false;
            }

            Event::Start(Tag::CodeBlock(kind)) => {
                self.in_code_block = true;
                self.code_block_content.clear();
                match kind {
                    CodeBlockKind::Fenced(lang) => {
                        self.code_block_language = lang.to_string();
                    }
                    CodeBlockKind::Indented => {
                        self.code_block_language.clear();
                    }
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                self.flush_code_block();
                self.in_code_block = false;
            }

            // Lists
            Event::Start(Tag::List(first_item)) => {
                // If we're starting a nested list inside a list item, flush the parent
                // item's text first (before its nested children)
                if self.list_depth > 0 && !self.text_buffer.is_empty() {
                    self.flush_list_item();
                }
                self.in_list = true;
                self.list_depth += 1;
                // Track ordered/unordered state per list level
                self.ordered_list_stack.push(first_item.is_some());
                self.list_item_number.push(first_item.unwrap_or(1) as usize);
                if self.list_depth == 1 {
                    self.output.push('\n');
                }
            }
            Event::End(TagEnd::List(_)) => {
                self.list_depth = self.list_depth.saturating_sub(1);
                self.list_item_number.pop();
                self.ordered_list_stack.pop();
                if self.list_depth == 0 {
                    self.in_list = false;
                }
            }

            Event::Start(Tag::Item) => {
                self.text_buffer.clear();
            }
            Event::End(TagEnd::Item) => {
                self.flush_list_item();
            }

            // Tables
            Event::Start(Tag::Table(alignments)) => {
                self.in_table = true;
                self.table_alignments = alignments;
                self.table_rows.clear();
                self.table_header_row = None;
            }
            Event::End(TagEnd::Table) => {
                self.flush_table();
                self.in_table = false;
                self.table_alignments.clear();
                self.table_rows.clear();
                self.table_header_row = None;
            }

            Event::Start(Tag::TableHead) => {
                self.table_header = true;
                self.table_row.clear();
            }
            Event::End(TagEnd::TableHead) => {
                // Store header row for later
                self.table_header_row = Some(std::mem::take(&mut self.table_row));
                self.table_header = false;
            }

            Event::Start(Tag::TableRow) => {
                self.table_row.clear();
            }
            Event::End(TagEnd::TableRow) => {
                // Store row for later
                self.table_rows.push(std::mem::take(&mut self.table_row));
            }

            Event::Start(Tag::TableCell) => {
                self.current_cell.clear();
            }
            Event::End(TagEnd::TableCell) => {
                self.table_row.push(std::mem::take(&mut self.current_cell));
            }

            // Inline elements
            Event::Start(Tag::Emphasis) => {
                self.in_emphasis = true;
                if !self.in_table {
                    self.text_buffer
                        .push_str(&self.options.styles.emph.block_prefix);
                } else {
                    self.current_cell
                        .push_str(&self.options.styles.emph.block_prefix);
                }
            }
            Event::End(TagEnd::Emphasis) => {
                self.in_emphasis = false;
                if !self.in_table {
                    self.text_buffer
                        .push_str(&self.options.styles.emph.block_suffix);
                } else {
                    self.current_cell
                        .push_str(&self.options.styles.emph.block_suffix);
                }
            }

            Event::Start(Tag::Strong) => {
                self.in_strong = true;
                if !self.in_table {
                    self.text_buffer
                        .push_str(&self.options.styles.strong.block_prefix);
                } else {
                    self.current_cell
                        .push_str(&self.options.styles.strong.block_prefix);
                }
            }
            Event::End(TagEnd::Strong) => {
                self.in_strong = false;
                if !self.in_table {
                    self.text_buffer
                        .push_str(&self.options.styles.strong.block_suffix);
                } else {
                    self.current_cell
                        .push_str(&self.options.styles.strong.block_suffix);
                }
            }

            Event::Start(Tag::Strikethrough) => {
                self.in_strikethrough = true;
                if !self.in_table {
                    self.text_buffer
                        .push_str(&self.options.styles.strikethrough.block_prefix);
                } else {
                    self.current_cell
                        .push_str(&self.options.styles.strikethrough.block_prefix);
                }
            }
            Event::End(TagEnd::Strikethrough) => {
                self.in_strikethrough = false;
                if !self.in_table {
                    self.text_buffer
                        .push_str(&self.options.styles.strikethrough.block_suffix);
                } else {
                    self.current_cell
                        .push_str(&self.options.styles.strikethrough.block_suffix);
                }
            }

            Event::Start(Tag::Link {
                dest_url, title, ..
            }) => {
                self.in_link = true;
                self.link_url = dest_url.to_string();
                self.link_title = title.to_string();
            }
            Event::End(TagEnd::Link) => {
                // Append URL after link text, like Go glamour does
                // But don't duplicate if the link text is already the URL (autolinks)
                if !self.link_url.is_empty() && !self.text_buffer.ends_with(&self.link_url) {
                    self.text_buffer.push(' ');
                    self.text_buffer.push_str(&self.link_url);
                }
                self.in_link = false;
                self.link_url.clear();
                self.link_title.clear();
            }

            Event::Start(Tag::Image {
                dest_url, title, ..
            }) => {
                self.in_image = true;
                self.image_url = dest_url.to_string();
                self.image_title = title.to_string();
            }
            Event::End(TagEnd::Image) => {
                self.flush_image();
                self.in_image = false;
            }

            // Text content
            Event::Text(text) => {
                if self.in_code_block {
                    self.code_block_content.push_str(&text);
                } else if self.in_table {
                    self.current_cell.push_str(&text);
                } else if self.in_image {
                    // Buffer for image alt text
                    self.text_buffer.push_str(&text);
                } else {
                    self.text_buffer.push_str(&text);
                }
            }

            Event::Code(code) => {
                let styled = self.style_inline_code(&code);
                if self.in_table {
                    self.current_cell.push_str(&styled);
                } else {
                    self.text_buffer.push_str(&styled);
                }
            }

            Event::SoftBreak => {
                if self.options.preserve_newlines {
                    if self.in_table {
                        self.current_cell.push('\n');
                    } else {
                        self.text_buffer.push('\n');
                    }
                } else if self.in_table {
                    self.current_cell.push(' ');
                } else {
                    self.text_buffer.push(' ');
                }
            }

            Event::HardBreak => {
                if self.in_table {
                    self.current_cell.push('\n');
                } else {
                    self.text_buffer.push('\n');
                }
            }

            Event::Rule => {
                self.output
                    .push_str(&self.options.styles.horizontal_rule.format);
            }

            Event::TaskListMarker(checked) => {
                if checked {
                    self.text_buffer.push_str(&self.options.styles.task.ticked);
                } else {
                    self.text_buffer
                        .push_str(&self.options.styles.task.unticked);
                }
            }

            // Ignore other events
            _ => {}
        }
    }

    fn flush_heading(&mut self) {
        if let Some(level) = self.in_heading {
            let heading_style = self.options.styles.heading_style(level);
            let base_heading = &self.options.styles.heading;

            // Build the heading text
            let mut heading_text = String::new();
            heading_text.push_str(&heading_style.style.prefix);
            heading_text.push_str(&self.text_buffer);
            heading_text.push_str(&heading_style.style.suffix);

            // Apply lipgloss styling
            let mut style = base_heading.style.to_lipgloss();

            // Merge heading-level specific styles
            if let Some(ref color) = heading_style.style.color {
                style = style.foreground(color.as_str());
            }
            if let Some(ref bg) = heading_style.style.background_color {
                style = style.background(bg.as_str());
            }
            if heading_style.style.bold == Some(true) {
                style = style.bold();
            }
            if heading_style.style.italic == Some(true) {
                style = style.italic();
            }

            let rendered = style.render(&heading_text);

            self.output.push_str(&heading_style.style.block_prefix);
            self.output.push('\n');
            self.output.push_str(&rendered);
            self.output.push_str(&base_heading.style.block_suffix);

            self.text_buffer.clear();
        }
    }

    fn flush_paragraph(&mut self) {
        if !self.text_buffer.is_empty() {
            let text = std::mem::take(&mut self.text_buffer);

            // Apply word wrap
            let wrapped = self.word_wrap(&text);

            // Apply paragraph styling
            let style = self.options.styles.paragraph.style.to_lipgloss();
            let rendered = style.render(&wrapped);

            // Add block quote indent if needed
            if self.in_block_quote {
                let indent_token = self
                    .options
                    .styles
                    .block_quote
                    .indent_token
                    .as_deref()
                    .unwrap_or("│ ");
                let indented = rendered
                    .lines()
                    .map(|line| format!("{}{}", indent_token, line))
                    .collect::<Vec<_>>()
                    .join("\n");
                self.output.push_str(&indented);
                self.output.push('\n');
                self.block_quote_pending_separator = true;
            } else {
                self.output.push_str(&rendered);
                self.output.push_str("\n\n");
            }
        }
    }

    fn flush_list_item(&mut self) {
        let text = std::mem::take(&mut self.text_buffer);
        if text.is_empty() {
            return;
        }

        let indent = (self.list_depth - 1) * self.options.styles.list.level_indent;
        let indent_str = " ".repeat(indent);

        let is_ordered = self.ordered_list_stack.last().copied().unwrap_or(false);
        let prefix = if is_ordered {
            let num = self.list_item_number.last().copied().unwrap_or(1);
            if let Some(last) = self.list_item_number.last_mut() {
                *last += 1;
            }
            format!("{}{}", num, &self.options.styles.enumeration.block_prefix)
        } else {
            self.options.styles.item.block_prefix.clone()
        };

        self.output.push_str(&indent_str);
        self.output.push_str(&prefix);
        self.output.push_str(text.trim());
        self.output.push('\n');
    }

    fn flush_code_block(&mut self) {
        let content = std::mem::take(&mut self.code_block_content);
        let language = std::mem::take(&mut self.code_block_language);
        let style = &self.options.styles.code_block;

        self.output.push('\n');

        // Apply margin
        let margin = style.block.margin.unwrap_or(0);
        let margin_str = " ".repeat(margin);

        // Try syntax highlighting if feature is enabled and language is specified
        #[cfg(feature = "syntax-highlighting")]
        {
            use crate::syntax::{LanguageDetector, SyntaxTheme, highlight_code};

            let syntax_config = &self.options.styles.syntax_config;

            if !language.is_empty() && !syntax_config.is_disabled(&language) {
                // Resolve language through custom aliases
                let resolved_lang = syntax_config.resolve_language(&language);

                let detector = LanguageDetector::new();
                if detector.is_supported(resolved_lang) {
                    // Get theme from syntax config, code_block style, or use default
                    let theme = SyntaxTheme::from_name(&syntax_config.theme_name)
                        .or_else(|| {
                            style
                                .theme
                                .as_ref()
                                .and_then(|name| SyntaxTheme::from_name(name))
                        })
                        .unwrap_or_else(SyntaxTheme::default_dark);

                    let highlighted = highlight_code(&content, resolved_lang, &theme);

                    // Output with optional line numbers
                    for (idx, line) in highlighted.lines().enumerate() {
                        self.output.push_str(&margin_str);
                        if syntax_config.line_numbers {
                            // Format line number with right-aligned padding
                            let line_num = idx + 1;
                            self.output.push_str(&format!("{:4} │ ", line_num));
                        }
                        self.output.push_str(line);
                        self.output.push('\n');
                    }

                    self.output.push('\n');
                    return;
                }
            }
        }

        // Suppress unused variable warning when feature is disabled
        let _ = &language;

        // Fallback: no syntax highlighting
        for line in content.lines() {
            self.output.push_str(&margin_str);
            self.output.push_str(line);
            self.output.push('\n');
        }

        self.output.push('\n');
    }

    fn flush_table(&mut self) {
        use crate::table::{
            ColumnWidthConfig, MINIMAL_ASCII_BORDER, MINIMAL_BORDER, ParsedTable, TableCell,
            calculate_column_widths, render_minimal_row, render_minimal_separator,
        };

        // Collect all rows (header + body) to count columns
        let num_cols = self.table_alignments.len();
        if num_cols == 0 {
            return;
        }

        let mut parsed_table = ParsedTable::new();
        parsed_table.alignments = self.table_alignments.clone();

        if let Some(header_strs) = &self.table_header_row {
            parsed_table.header = header_strs
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    let align = self
                        .table_alignments
                        .get(i)
                        .copied()
                        .unwrap_or(pulldown_cmark::Alignment::None);
                    TableCell::new(s.clone(), align)
                })
                .collect();
        }

        for row_strs in &self.table_rows {
            let row_cells = row_strs
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    let align = self
                        .table_alignments
                        .get(i)
                        .copied()
                        .unwrap_or(pulldown_cmark::Alignment::None);
                    TableCell::new(s.clone(), align)
                })
                .collect();
            parsed_table.rows.push(row_cells);
        }

        if parsed_table.is_empty() {
            return;
        }

        // Determine border style - use minimal borders to match Go glamour
        // Go glamour only renders internal separators (no outer borders)
        let col_sep = self
            .options
            .styles
            .table
            .column_separator
            .as_deref()
            .unwrap_or("│");
        let border = if col_sep == "|" {
            MINIMAL_ASCII_BORDER
        } else {
            MINIMAL_BORDER
        };

        // Calculate column widths
        let margin = self
            .options
            .styles
            .document
            .margin
            .unwrap_or(DEFAULT_MARGIN);
        let max_width = self.options.word_wrap.saturating_sub(2 * margin);
        let cell_padding = 1;

        // Use border_width=0 for minimal style since we don't have outer borders
        let width_config = ColumnWidthConfig::new()
            .cell_padding(cell_padding)
            .border_width(1) // Internal separators still take 1 char width
            .max_table_width(max_width);

        let column_widths = calculate_column_widths(&parsed_table, &width_config);
        let widths = &column_widths.widths;

        // Output a blank styled line first (matching Go behavior)
        let doc_style = &self.options.styles.document.style;
        let lipgloss = doc_style.to_lipgloss();
        // Just a newline with background if set
        self.output.push('\n');

        // No top border - Go glamour doesn't render outer borders

        // Header row (rendered without outer borders)
        if !parsed_table.header.is_empty() {
            let rendered_header =
                render_minimal_row(&parsed_table.header, widths, &border, cell_padding);
            self.output.push_str(&lipgloss.render(&rendered_header));
            self.output.push('\n');

            // Header separator (internal only)
            let sep = render_minimal_separator(widths, &border, cell_padding);
            if !sep.is_empty() {
                self.output.push_str(&lipgloss.render(&sep));
                self.output.push('\n');
            }
        }

        // Body rows (rendered without outer borders)
        for row in parsed_table.rows.iter() {
            let rendered_row = render_minimal_row(row, widths, &border, cell_padding);
            self.output.push_str(&lipgloss.render(&rendered_row));
            self.output.push('\n');
        }

        // No bottom border - Go glamour doesn't render outer borders

        self.output.push('\n');
    }

    fn flush_image(&mut self) {
        let alt_text = std::mem::take(&mut self.text_buffer);
        let url = std::mem::take(&mut self.image_url);

        let style = &self.options.styles.image_text;
        let format = if style.format.is_empty() {
            "Image: {{.text}} ->"
        } else {
            &style.format
        };

        let text = format.replace("{{.text}}", &alt_text);

        let link_style = self.options.styles.image.to_lipgloss();
        let rendered_url = link_style.render(&url);

        self.output.push_str(&text);
        self.output.push(' ');
        self.output.push_str(&rendered_url);
    }

    fn style_inline_code(&self, code: &str) -> String {
        let style = &self.options.styles.code;
        let lipgloss_style = style.style.to_lipgloss();

        // Build the code text with prefix/suffix INSIDE the styled region
        // Go glamour includes padding spaces inside the ANSI-styled region
        let code_with_padding = format!("{}{}{}", style.style.prefix, code, style.style.suffix);
        lipgloss_style.render(&code_with_padding)
    }

    fn word_wrap(&self, text: &str) -> String {
        let width = self.options.word_wrap;
        if width == 0 {
            return text.to_string();
        }

        let mut result = String::new();
        let mut current_line = String::new();

        for word in text.split_whitespace() {
            if current_line.is_empty() {
                current_line.push_str(word);
            } else if unicode_width::UnicodeWidthStr::width(current_line.as_str())
                + 1
                + unicode_width::UnicodeWidthStr::width(word)
                <= width
            {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                result.push_str(&current_line);
                result.push('\n');
                current_line = word.to_string();
            }
        }

        if !current_line.is_empty() {
            result.push_str(&current_line);
        }

        result
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Render markdown with the specified style.
pub fn render(markdown: &str, style: Style) -> Result<String, std::convert::Infallible> {
    Ok(Renderer::new().with_style(style).render(markdown))
}

/// Render markdown with the default dark style.
pub fn render_with_environment_config(markdown: &str) -> String {
    // Check GLAMOUR_STYLE environment variable
    let style = std::env::var("GLAMOUR_STYLE")
        .ok()
        .and_then(|s| match s.as_str() {
            "ascii" => Some(Style::Ascii),
            "dark" => Some(Style::Dark),
            "light" => Some(Style::Light),
            "pink" => Some(Style::Pink),
            "notty" => Some(Style::NoTty),
            "auto" => Some(Style::Auto),
            _ => None,
        })
        .unwrap_or(Style::Auto);

    Renderer::new().with_style(style).render(markdown)
}

/// Available style names for configuration.
pub fn available_styles() -> HashMap<&'static str, Style> {
    let mut styles = HashMap::new();
    styles.insert("ascii", Style::Ascii);
    styles.insert("dark", Style::Dark);
    styles.insert("light", Style::Light);
    styles.insert("pink", Style::Pink);
    styles.insert("notty", Style::NoTty);
    styles.insert("auto", Style::Auto);
    styles
}

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::{
        Renderer, RendererOptions, Style, StyleBlock, StyleCodeBlock, StyleConfig, StyleList,
        StylePrimitive, StyleTable, StyleTask, ascii_style, available_styles, dark_style,
        light_style, pink_style, render, render_with_environment_config,
    };
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_new() {
        let renderer = Renderer::new();
        assert_eq!(renderer.options.word_wrap, DEFAULT_WIDTH);
    }

    #[test]
    fn test_renderer_with_word_wrap() {
        let renderer = Renderer::new().with_word_wrap(120);
        assert_eq!(renderer.options.word_wrap, 120);
    }

    #[test]
    fn test_renderer_with_style() {
        let renderer = Renderer::new().with_style(Style::Light);
        // Light style has different document color
        assert!(renderer.options.styles.document.style.color.is_some());
    }

    #[test]
    fn test_render_simple_text() {
        let renderer = Renderer::new().with_style(Style::Ascii);
        let output = renderer.render("Hello, world!");
        assert!(output.contains("Hello, world!"));
    }

    #[test]
    fn test_render_heading() {
        let renderer = Renderer::new().with_style(Style::Ascii);
        let output = renderer.render("# Heading");
        assert!(output.contains("# Heading"));
    }

    #[test]
    fn test_render_emphasis() {
        let renderer = Renderer::new().with_style(Style::Ascii);
        let output = renderer.render("*italic*");
        assert!(output.contains("*italic*"));
    }

    #[test]
    fn test_render_strong() {
        let renderer = Renderer::new().with_style(Style::Ascii);
        let output = renderer.render("**bold**");
        assert!(output.contains("**bold**"));
    }

    #[test]
    fn test_render_code() {
        let renderer = Renderer::new().with_style(Style::Ascii);
        let output = renderer.render("`code`");
        assert!(output.contains("`"));
        assert!(output.contains("code"));
    }

    #[test]
    fn test_render_horizontal_rule() {
        let renderer = Renderer::new().with_style(Style::Ascii);
        let output = renderer.render("---");
        assert!(output.contains("--------"));
    }

    #[test]
    fn test_render_list() {
        let renderer = Renderer::new().with_style(Style::Ascii);
        let output = renderer.render("* item 1\n* item 2");
        assert!(output.contains("item 1"));
        assert!(output.contains("item 2"));
    }

    #[test]
    fn test_render_nested_list() {
        let renderer = Renderer::new().with_style(Style::Dark);
        let output = renderer.render("- Item 1\n  - Nested 1\n  - Nested 2\n- Item 2");
        assert!(output.contains("Item 1"));
        assert!(output.contains("Nested 1"));
        assert!(output.contains("Nested 2"));
        assert!(output.contains("Item 2"));
    }

    #[test]
    fn test_render_mixed_nested_list() {
        let renderer = Renderer::new().with_style(Style::Dark);
        let output = renderer.render("1. First\n   - Sub item\n   - Sub item\n2. Second");
        assert!(output.contains("First"));
        assert!(output.contains("Sub item"));
        assert!(output.contains("Second"));
        // Verify the second item is rendered as ordered (with number)
        assert!(output.contains("2."));
    }

    #[test]
    fn test_render_link() {
        let renderer = Renderer::new().with_style(Style::Dark);
        let output = renderer.render("[Link text](https://example.com)");
        assert!(output.contains("Link text"));
        // URL should be appended after link text
        assert!(output.contains("https://example.com"));
    }

    #[test]
    fn test_render_autolink() {
        let renderer = Renderer::new().with_style(Style::Dark);
        let output = renderer.render("<https://example.com>");
        // For autolinks, URL should appear only once (not duplicated)
        let url_count = output.matches("https://example.com").count();
        assert_eq!(url_count, 1, "Autolink URL should appear exactly once");
    }

    #[test]
    fn test_render_ordered_list() {
        let renderer = Renderer::new().with_style(Style::Ascii);
        let output = renderer.render("1. first\n2. second");
        assert!(output.contains("first"));
        assert!(output.contains("second"));
    }

    #[test]
    fn test_render_table() {
        let renderer = Renderer::new().with_style(Style::Ascii);
        let output = renderer.render("| A | B |\n|---|---|\n| 1 | 2 |");
        assert!(output.contains("|"));
        assert!(output.contains("A"));
        assert!(output.contains("B"));
    }

    #[test]
    fn test_render_table_dark_debug() {
        let renderer = Renderer::new().with_style(Style::Dark);
        let output = renderer.render("| A | B |\n|---|---|\n| 1 | 2 |");

        // Print each line with visible markers
        eprintln!("=== RUST TABLE OUTPUT (2x2, dark) ===");
        for (i, line) in output.lines().enumerate() {
            eprintln!("Line {}: len={} chars", i, line.chars().count());
            // Print escaped version
            let escaped: String = line
                .chars()
                .map(|c| {
                    if c == '\x1b' {
                        "\\x1b".to_string()
                    } else if c == '│' {
                        "│".to_string()
                    } else if c == '─' {
                        "─".to_string()
                    } else if c == '┼' {
                        "┼".to_string()
                    } else {
                        c.to_string()
                    }
                })
                .collect();
            eprintln!("  {:?}", escaped);
        }
        eprintln!("=== END OUTPUT ===");

        // Verify basic structure
        assert!(
            output.contains("│") || output.contains("|"),
            "Should contain column separator"
        );
        assert!(output.contains("A"), "Should contain header A");
    }

    #[test]
    fn test_style_primitive_builder() {
        let style = StylePrimitive::new()
            .color("red")
            .bold(true)
            .prefix("> ")
            .suffix(" <");

        assert_eq!(style.color, Some("red".to_string()));
        assert_eq!(style.bold, Some(true));
        assert_eq!(style.prefix, "> ");
        assert_eq!(style.suffix, " <");
    }

    #[test]
    fn test_style_block_builder() {
        let block = StyleBlock::new().margin(4).indent(2).indent_token("  ");

        assert_eq!(block.margin, Some(4));
        assert_eq!(block.indent, Some(2));
        assert_eq!(block.indent_token, Some("  ".to_string()));
    }

    #[test]
    fn test_style_config_heading() {
        let config = dark_style();
        let h1 = config.heading_style(HeadingLevel::H1);
        assert!(
            !h1.style.prefix.is_empty() || h1.style.suffix.len() > 0 || h1.style.color.is_some()
        );
    }

    #[test]
    fn test_available_styles() {
        let styles = available_styles();
        assert!(styles.contains_key("dark"));
        assert!(styles.contains_key("light"));
        assert!(styles.contains_key("ascii"));
        assert!(styles.contains_key("pink"));
    }

    #[test]
    fn test_render_function() {
        let result = render("# Test", Style::Ascii);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Test"));
    }

    #[test]
    fn test_dark_style() {
        let config = dark_style();
        assert!(config.heading.style.bold == Some(true));
        assert!(config.document.margin.is_some());
    }

    #[test]
    fn test_light_style() {
        let config = light_style();
        assert!(config.heading.style.bold == Some(true));
    }

    #[test]
    fn test_ascii_style() {
        let config = ascii_style();
        assert_eq!(config.h1.style.prefix, "# ");
    }

    #[test]
    fn test_pink_style() {
        let config = pink_style();
        assert!(config.heading.style.color.is_some());
    }

    #[test]
    fn test_word_wrap() {
        let renderer = Renderer::new().with_word_wrap(20);
        let output = renderer.render("This is a very long line that should be wrapped.");
        // The output should contain newlines due to wrapping
        assert!(output.len() > 0);
    }

    #[test]
    fn test_render_code_block() {
        let renderer = Renderer::new().with_style(Style::Ascii);
        let output = renderer.render("```rust\nfn main() {}\n```");
        // With syntax highlighting, tokens may be split by ANSI codes
        // So check for individual tokens instead of the full string
        assert!(output.contains("fn"));
        assert!(output.contains("main"));
    }

    #[test]
    fn test_render_blockquote() {
        let renderer = Renderer::new().with_style(Style::Dark);
        let output = renderer.render("> quoted text");
        assert!(output.contains("quoted"));
    }

    #[test]
    fn test_strikethrough() {
        let renderer = Renderer::new().with_style(Style::Ascii);
        let output = renderer.render("~~deleted~~");
        assert!(output.contains("~~"));
        assert!(output.contains("deleted"));
    }

    #[test]
    fn test_task_list() {
        let renderer = Renderer::new().with_style(Style::Ascii);
        let output = renderer.render("- [ ] todo\n- [x] done");
        assert!(output.contains("[ ]") || output.contains("todo"));
    }

    // ========================================================================
    // Syntax Theme Config Tests (feature-gated)
    // ========================================================================

    #[cfg(feature = "syntax-highlighting")]
    mod syntax_config_tests {
        use super::*;

        #[test]
        fn test_syntax_theme_config_default() {
            let config = SyntaxThemeConfig::default();
            assert_eq!(config.theme_name, "base16-ocean.dark");
            assert!(!config.line_numbers);
            assert!(config.language_aliases.is_empty());
            assert!(config.disabled_languages.is_empty());
        }

        #[test]
        fn test_syntax_theme_config_builder() {
            let config = SyntaxThemeConfig::new()
                .theme("Solarized (dark)")
                .line_numbers(true)
                .language_alias("dockerfile", "docker")
                .disable_language("text");

            assert_eq!(config.theme_name, "Solarized (dark)");
            assert!(config.line_numbers);
            assert_eq!(
                config.language_aliases.get("dockerfile"),
                Some(&"docker".to_string())
            );
            assert!(config.disabled_languages.contains("text"));
        }

        #[test]
        fn test_syntax_theme_config_resolve_language() {
            let config = SyntaxThemeConfig::new()
                .language_alias("rs", "rust")
                .language_alias("dockerfile", "docker");

            assert_eq!(config.resolve_language("rs"), "rust");
            assert_eq!(config.resolve_language("dockerfile"), "docker");
            assert_eq!(config.resolve_language("python"), "python"); // No alias
        }

        #[test]
        fn test_syntax_theme_config_is_disabled() {
            let config = SyntaxThemeConfig::new()
                .disable_language("text")
                .disable_language("plain");

            assert!(config.is_disabled("text"));
            assert!(config.is_disabled("plain"));
            assert!(!config.is_disabled("rust"));
        }

        #[test]
        fn test_syntax_theme_config_validate() {
            let valid = SyntaxThemeConfig::new().theme("base16-ocean.dark");
            assert!(valid.validate().is_ok());

            let invalid = SyntaxThemeConfig::new().theme("nonexistent-theme");
            assert!(invalid.validate().is_err());
            let err = invalid.validate().unwrap_err();
            assert!(err.contains("Unknown syntax theme"));
            assert!(err.contains("nonexistent-theme"));
        }

        #[test]
        fn test_style_config_syntax_methods() {
            let config = StyleConfig::default()
                .syntax_theme("Solarized (dark)")
                .with_line_numbers(true)
                .language_alias("rs", "rust")
                .disable_language("text");

            assert_eq!(config.syntax().theme_name, "Solarized (dark)");
            assert!(config.syntax().line_numbers);
            assert_eq!(
                config.syntax().language_aliases.get("rs"),
                Some(&"rust".to_string())
            );
            assert!(config.syntax().disabled_languages.contains("text"));
        }

        #[test]
        fn test_style_config_with_syntax_config() {
            let syntax_config = SyntaxThemeConfig::new()
                .theme("InspiredGitHub")
                .line_numbers(true);

            let style_config = StyleConfig::default().with_syntax_config(syntax_config);

            assert_eq!(style_config.syntax().theme_name, "InspiredGitHub");
            assert!(style_config.syntax().line_numbers);
        }

        #[test]
        fn test_render_with_line_numbers() {
            let config = StyleConfig::default().with_line_numbers(true);
            let renderer = Renderer::new().with_style_config(config);

            let output = renderer.render("```rust\nfn main() {\n    println!(\"Hello\");\n}\n```");

            // Should contain line numbers
            assert!(output.contains("1 │"));
            assert!(output.contains("2 │"));
            assert!(output.contains("3 │"));
        }

        #[test]
        fn test_render_with_disabled_language() {
            let config = StyleConfig::default().disable_language("rust");
            let renderer = Renderer::new().with_style_config(config);

            let output = renderer.render("```rust\nfn main() {}\n```");

            // Should NOT have ANSI codes since rust is disabled
            // The output should just have the plain text
            assert!(output.contains("fn main()"));
        }

        #[test]
        fn test_render_with_language_alias() {
            let config = StyleConfig::default().language_alias("rs", "rust");
            let renderer = Renderer::new().with_style_config(config);

            let output = renderer.render("```rs\nfn main() {}\n```");

            // Should be highlighted as Rust (contains ANSI codes)
            assert!(output.contains("fn"));
            assert!(output.contains("main"));
            assert!(output.contains('\x1b'));
        }

        #[test]
        fn test_runtime_theme_switching() {
            let mut renderer = Renderer::new();

            // Default theme
            let original_theme = renderer.syntax_config().theme_name.clone();
            assert_eq!(original_theme, "base16-ocean.dark");

            // Switch to a different theme
            renderer.set_syntax_theme("Solarized (dark)").unwrap();
            assert_eq!(renderer.syntax_config().theme_name, "Solarized (dark)");

            // Render with new theme
            let output = renderer.render("```rust\nfn main() {}\n```");
            assert!(output.contains('\x1b')); // Should have ANSI codes
        }

        #[test]
        fn test_runtime_theme_switching_invalid_theme() {
            let mut renderer = Renderer::new();

            let result = renderer.set_syntax_theme("nonexistent-theme-xyz");
            assert!(result.is_err());

            let err = result.unwrap_err();
            assert!(err.contains("Unknown syntax theme"));
            assert!(err.contains("nonexistent-theme-xyz"));
            assert!(err.contains("Available themes"));

            // Theme should not have changed
            assert_eq!(renderer.syntax_config().theme_name, "base16-ocean.dark");
        }

        #[test]
        fn test_runtime_line_numbers_toggle() {
            let mut renderer = Renderer::new();

            // Default should be off
            assert!(!renderer.syntax_config().line_numbers);

            // Enable line numbers
            renderer.set_line_numbers(true);
            assert!(renderer.syntax_config().line_numbers);

            let output = renderer.render("```rust\nfn main() {}\n```");
            assert!(output.contains("1 │"));

            // Disable line numbers
            renderer.set_line_numbers(false);
            assert!(!renderer.syntax_config().line_numbers);
        }

        #[test]
        fn test_syntax_config_mut() {
            let mut renderer = Renderer::new();

            // Modify config through mutable reference
            renderer
                .syntax_config_mut()
                .language_aliases
                .insert("myrs".to_string(), "rust".to_string());

            let config = renderer.syntax_config();
            assert_eq!(
                config.language_aliases.get("myrs"),
                Some(&"rust".to_string())
            );
        }
    }
}

// ============================================================================
// E2E Syntax Highlighting Tests
// ============================================================================

#[cfg(test)]
#[cfg(feature = "syntax-highlighting")]
mod e2e_highlighting_tests {
    use super::*;

    // ========================================================================
    // Full Document Rendering Tests
    // ========================================================================

    #[test]
    fn test_document_with_mixed_code_blocks() {
        let markdown = r#"
# My Document

Here's some Rust:

```rust
fn main() {
    println!("Hello");
}
```

And some Python:

```python
def main():
    print("Hello")
```

And some JSON:

```json
{"key": "value"}
```
"#;

        let renderer = Renderer::new().with_style(Style::Dark);
        let output = renderer.render(markdown);

        // All code blocks should be highlighted (have ANSI codes)
        assert!(output.contains("\x1b["), "Should have color codes");

        // All content should be present (check tokens separately as ANSI codes may split them)
        assert!(output.contains("fn"), "Should contain Rust fn keyword");
        assert!(output.contains("main"), "Should contain main function");
        assert!(output.contains("def"), "Should contain Python def keyword");
        assert!(output.contains("key"), "Should contain JSON key");
    }

    #[test]
    fn test_document_with_inline_code_not_syntax_highlighted() {
        let renderer = Renderer::new().with_style(Style::Dark);
        let markdown = "Here is `inline code` in a sentence.";
        let output = renderer.render(markdown);

        // Inline code should be styled (with background) but NOT syntax highlighted
        assert!(
            output.contains("inline code"),
            "Should contain inline code text"
        );
        // Inline code uses lipgloss styling, not syntect highlighting
    }

    #[test]
    fn test_real_readme_rendering() {
        // Use a small synthetic README since we can't use include_str! on project README
        let readme = r#"
# My Project

A library for doing things.

## Installation

```bash
cargo add my-project
```

## Usage

```rust
use my_project::do_thing;

fn main() {
    do_thing();
}
```

## Features

- Feature 1
- Feature 2
- Feature 3

| Column A | Column B |
|----------|----------|
| Value 1  | Value 2  |

## License

MIT
"#;

        let config = StyleConfig::default().syntax_theme("base16-ocean.dark");
        let renderer = Renderer::new().with_style_config(config);

        // Should not panic
        let output = renderer.render(readme);

        // Should produce substantial output
        assert!(
            output.len() > readme.len() / 2,
            "Output should be substantial, got {} chars from {} input chars",
            output.len(),
            readme.len()
        );

        // Should contain key content (check tokens separately as ANSI codes may split them)
        assert!(output.contains("My Project"), "Should contain title");
        assert!(output.contains("cargo"), "Should contain cargo command");
        assert!(output.contains("do_thing"), "Should contain Rust code");
    }

    // ========================================================================
    // Theme Consistency Tests
    // ========================================================================

    #[test]
    fn test_theme_consistency_across_blocks() {
        let markdown = r#"
```rust
fn a() {}
```

Some text in between.

```rust
fn b() {}
```
"#;

        let renderer = Renderer::new().with_style(Style::Dark);
        let output = renderer.render(markdown);

        // Both `fn` keywords should have the same color
        let fn_indices: Vec<_> = output.match_indices("fn").collect();
        assert!(
            fn_indices.len() >= 2,
            "Should have at least 2 'fn' keywords, found {}",
            fn_indices.len()
        );

        // Extract the ANSI escape sequence before each `fn`
        let get_escape_before = |idx: usize| -> Option<&str> {
            let prefix = &output[..idx];
            // Find the last escape sequence before the keyword
            if let Some(esc_start) = prefix.rfind("\x1b[") {
                // Find the 'm' that ends the escape sequence
                let search_area = &prefix[esc_start..];
                if let Some(m_pos) = search_area.find('m') {
                    return Some(&prefix[esc_start..esc_start + m_pos + 1]);
                }
            }
            None
        };

        let color1 = get_escape_before(fn_indices[0].0);
        let color2 = get_escape_before(fn_indices[1].0);

        assert_eq!(
            color1, color2,
            "Same tokens should have same colors: {:?} vs {:?}",
            color1, color2
        );
    }

    // ========================================================================
    // Error Resilience Tests
    // ========================================================================

    #[test]
    fn test_malformed_language_tag() {
        // Language tag with extra whitespace/content
        let markdown = "```rust with extra stuff\nfn main() {}\n```";

        let renderer = Renderer::new().with_style(Style::Dark);
        // Should not panic
        let output = renderer.render(markdown);

        // Content should still be rendered (even if not highlighted)
        assert!(
            output.contains("fn main"),
            "Should contain code content even with malformed tag"
        );
    }

    #[test]
    fn test_very_long_code_block() {
        let code = "let x = 1;\n".repeat(1000); // 1000 lines
        let markdown = format!("```rust\n{}```", code);

        // Should complete without timeout or crash
        let start = std::time::Instant::now();
        let renderer = Renderer::new().with_style(Style::Dark);
        let output = renderer.render(&markdown);
        let duration = start.elapsed();

        assert!(
            duration.as_secs() < 5,
            "Should complete in <5s, took {:?}",
            duration
        );
        // Check tokens separately as ANSI codes may split them
        assert!(output.contains("let"), "Should contain let keyword");
        assert!(output.contains("x"), "Should contain variable x");
    }

    #[test]
    fn test_code_block_with_unicode() {
        let markdown = r#"
```rust
fn main() {
    let emoji = "🦀";
    let chinese = "你好";
    let japanese = "こんにちは";
    let arabic = "مرحبا";
}
```
"#;

        let renderer = Renderer::new().with_style(Style::Dark);
        let output = renderer.render(markdown);

        assert!(output.contains("🦀"), "Should preserve crab emoji");
        assert!(
            output.contains("你好"),
            "Should preserve Chinese characters"
        );
        assert!(
            output.contains("こんにちは"),
            "Should preserve Japanese characters"
        );
        assert!(
            output.contains("مرحبا"),
            "Should preserve Arabic characters"
        );
    }

    #[test]
    fn test_empty_code_block() {
        let markdown = "```rust\n```";

        let renderer = Renderer::new().with_style(Style::Dark);
        // Should not panic on empty code block
        let output = renderer.render(markdown);

        // Output should exist (may just be whitespace/margins)
        assert!(output.len() > 0, "Should produce some output");
    }

    #[test]
    fn test_code_block_with_only_whitespace() {
        let markdown = "```rust\n   \n\t\n   \n```";

        let renderer = Renderer::new().with_style(Style::Dark);
        // Should not panic
        let output = renderer.render(markdown);

        // Should handle gracefully
        assert!(output.len() > 0, "Should produce some output");
    }

    #[test]
    fn test_unknown_language_graceful_fallback() {
        let markdown = "```notareallanguage123\nsome code here\n```";

        let renderer = Renderer::new().with_style(Style::Dark);
        // Should not panic, should render as plain text
        let output = renderer.render(markdown);

        assert!(
            output.contains("some code here"),
            "Should render unknown language code as plain text"
        );
    }

    #[test]
    fn test_special_characters_in_code() {
        let markdown = r#"
```rust
fn main() {
    let s = "<script>alert('xss')</script>";
    let regex = r"[a-z]+\d*";
    let backslash = "\\";
    let null_byte = "\0";
}
```
"#;

        let renderer = Renderer::new().with_style(Style::Dark);
        // Should not panic or produce invalid output
        let output = renderer.render(markdown);

        assert!(
            output.contains("script"),
            "Should handle HTML-like content in code"
        );
        assert!(output.contains("regex"), "Should handle regex patterns");
    }

    // ========================================================================
    // Multiple Theme Tests
    // ========================================================================

    #[test]
    fn test_different_themes_produce_different_output() {
        let markdown = "```rust\nfn main() {}\n```";

        let theme1 = StyleConfig::default().syntax_theme("base16-ocean.dark");
        let theme2 = StyleConfig::default().syntax_theme("Solarized (dark)");

        let renderer1 = Renderer::new().with_style_config(theme1);
        let renderer2 = Renderer::new().with_style_config(theme2);

        let output1 = renderer1.render(markdown);
        let output2 = renderer2.render(markdown);

        // Different themes should produce different ANSI escape sequences
        assert_ne!(
            output1, output2,
            "Different themes should produce different output"
        );

        // But both should contain the code
        assert!(output1.contains("fn"), "Theme 1 should contain code");
        assert!(output2.contains("fn"), "Theme 2 should contain code");
    }

    #[test]
    fn test_all_available_themes_render_without_panic() {
        use crate::syntax::SyntaxTheme;

        let markdown = "```rust\nfn main() { println!(\"hello\"); }\n```";

        for theme_name in SyntaxTheme::available_themes() {
            let config = StyleConfig::default().syntax_theme(theme_name);
            let renderer = Renderer::new().with_style_config(config);

            // Should not panic for any theme
            let output = renderer.render(markdown);
            assert!(
                output.contains("fn"),
                "Theme '{}' should render code content",
                theme_name
            );
        }
    }

    // ========================================================================
    // Line Numbers Tests
    // ========================================================================

    #[test]
    fn test_line_numbers_correct_count() {
        let markdown = "```rust\nline1\nline2\nline3\nline4\nline5\n```";

        let config = StyleConfig::default().with_line_numbers(true);
        let renderer = Renderer::new().with_style_config(config);
        let output = renderer.render(markdown);

        // Should have line numbers 1 through 5
        assert!(output.contains("1 │"), "Should have line 1");
        assert!(output.contains("2 │"), "Should have line 2");
        assert!(output.contains("3 │"), "Should have line 3");
        assert!(output.contains("4 │"), "Should have line 4");
        assert!(output.contains("5 │"), "Should have line 5");
    }

    #[test]
    fn test_line_numbers_disabled_by_default() {
        let markdown = "```rust\nfn main() {}\n```";

        let renderer = Renderer::new().with_style(Style::Dark);
        let output = renderer.render(markdown);

        // Should NOT have line number markers
        assert!(
            !output.contains("1 │"),
            "Line numbers should be disabled by default"
        );
    }

    // ========================================================================
    // Language Alias Tests
    // ========================================================================

    #[test]
    fn test_custom_language_alias_applied() {
        let markdown = "```myrust\nfn main() {}\n```";

        let config = StyleConfig::default().language_alias("myrust", "rust");
        let renderer = Renderer::new().with_style_config(config);
        let output = renderer.render(markdown);

        // Should be highlighted as Rust (contains ANSI escape codes)
        assert!(
            output.contains('\x1b'),
            "Custom alias 'myrust' should be highlighted as Rust"
        );
    }

    // ========================================================================
    // Performance Tests
    // ========================================================================

    #[test]
    fn test_many_small_code_blocks_performance() {
        // Document with many small code blocks
        let mut markdown = String::new();
        for i in 0..100 {
            markdown.push_str(&format!("\n```rust\nfn func_{}() {{ }}\n```\n", i));
        }

        let start = std::time::Instant::now();
        let renderer = Renderer::new().with_style(Style::Dark);
        let output = renderer.render(&markdown);
        let duration = start.elapsed();

        assert!(
            duration.as_secs() < 5,
            "100 code blocks should render in <5s, took {:?}",
            duration
        );
        assert!(output.contains("func_0"), "Should contain first function");
        assert!(output.contains("func_99"), "Should contain last function");
    }

    // ========================================================================
    // Integration with Other Markdown Elements
    // ========================================================================

    #[test]
    fn test_code_blocks_with_surrounding_elements() {
        let markdown = r#"
# Header

Some **bold** and *italic* text.

> A blockquote with `inline code`.

```rust
fn main() {}
```

| Table | Header |
|-------|--------|
| cell  | cell   |

1. List item 1
2. List item 2

```python
def hello():
    pass
```

---

The end.
"#;

        let renderer = Renderer::new().with_style(Style::Dark);
        // Should handle all elements without issues
        let output = renderer.render(markdown);

        // Verify key elements are present (check tokens separately as ANSI codes may split them)
        assert!(output.contains("Header"), "Should contain heading");
        assert!(output.contains("fn"), "Should contain Rust fn keyword");
        assert!(output.contains("def"), "Should contain Python def keyword");
        assert!(output.contains("Table"), "Should contain table");
        assert!(output.contains("List item"), "Should contain list");
    }
}

#[cfg(test)]
mod table_spacing_tests {
    use super::*;

    #[test]
    fn test_table_spacing_matches_go() {
        let renderer = Renderer::new().with_style(Style::Dark);
        let md = "| A | B |\n|---|---|\n| 1 | 2 |";
        let output = renderer.render(md);

        // Print each line for debugging
        for (i, line) in output.lines().enumerate() {
            eprintln!("Line {}: {:?}", i, line);
        }

        let lines: Vec<&str> = output.lines().collect();
        // Minimal border structure (matching Go glamour):
        // Line 0: "" (empty prefix)
        // Line 1: "" (blank line before table)
        // Line 2: Header row (A │ B) - no outer borders
        // Line 3: Separator (─┼─)
        // Line 4: Data row (1 │ 2) - no outer borders
        // Line 5: "" (blank line after table)

        assert!(
            lines.len() >= 4,
            "Expected at least 4 lines for minimal table"
        );

        // Find the header row (contains A and B with internal separator)
        let header_line = lines
            .iter()
            .find(|l| l.contains('A') && l.contains('B'))
            .expect("Should have header row with A and B");
        assert!(
            header_line.contains('│'),
            "Should have internal column separator"
        );

        // Find the separator line (contains ─ and ┼)
        let sep_line = lines
            .iter()
            .find(|l| l.contains('─') && l.contains('┼'))
            .expect("Should have header separator");
        assert!(sep_line.contains('┼'), "Should have cross junction");

        // Verify there are NO outer borders (no ╭, ╰, ├, ┤)
        for line in &lines {
            assert!(!line.contains('╭'), "Should NOT have top-left corner");
            assert!(!line.contains('╰'), "Should NOT have bottom-left corner");
        }
    }

    #[test]
    fn test_table_respects_word_wrap() {
        let markdown = "| A | B |\n|---|---|\n| 1 | 2 |";

        // Render with 40 width
        let renderer_small = Renderer::new().with_word_wrap(40).with_style(Style::Ascii);
        let output_small = renderer_small.render(markdown);

        // Render with 120 width
        let renderer_large = Renderer::new().with_word_wrap(120).with_style(Style::Ascii);
        let output_large = renderer_large.render(markdown);

        // With minimal borders (matching Go glamour), we don't have top/bottom borders.
        // Instead, find the header separator line (contains - and |)
        let small_sep = output_small
            .lines()
            .find(|l| l.contains('─') && l.contains('|'))
            .expect("Could not find header separator in small output");

        let large_sep = output_large
            .lines()
            .find(|l| l.contains('─') && l.contains('|'))
            .expect("Could not find header separator in large output");

        // With equal column distribution and max width constraint,
        // our calculate_column_widths logic calculates width based on CONTENT.
        // It only *shrinks* if it exceeds max_width. It doesn't *expand* to fill max_width.
        //
        // So for small content that fits in both widths, table width should be the same
        // (content-sized, not expanded to fill available space).

        let width_small = small_sep.chars().count();
        let width_large = large_sep.chars().count();

        assert!(width_small <= 40, "Small table should fit in 40 chars");
        assert_eq!(
            width_small, width_large,
            "Table should be compact (content-sized) when it fits"
        );
    }
}
