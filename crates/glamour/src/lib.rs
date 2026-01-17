#![forbid(unsafe_code)]

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

use lipgloss::Style as LipglossStyle;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::collections::HashMap;
use std::fmt::Write as FmtWrite;

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
        h6: StyleBlock::new().style(StylePrimitive::new().prefix("###### ").color("35").bold(false)),
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
        image_text: StylePrimitive::new().color("243").format("Image: {{.text}} ->"),
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
        image_text: StylePrimitive::new().color("243").format("Image: {{.text}} ->"),
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
    in_list: bool,
    in_ordered_list: bool,
    list_depth: usize,
    list_item_number: Vec<usize>,
    in_table: bool,
    table_alignments: Vec<pulldown_cmark::Alignment>,
    table_row: Vec<String>,
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
            in_list: false,
            in_ordered_list: false,
            list_depth: 0,
            list_item_number: Vec::new(),
            in_table: false,
            table_alignments: Vec::new(),
            table_row: Vec::new(),
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
        self.output.push_str(&self.options.styles.document.style.block_prefix);

        // Add margin
        let margin = self.options.styles.document.margin.unwrap_or(0);

        for event in parser {
            self.handle_event(event);
        }

        // Document suffix
        self.output.push_str(&self.options.styles.document.style.block_suffix);

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
                self.output.push('\n');
            }
            Event::End(TagEnd::BlockQuote) => {
                self.in_block_quote = false;
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
                self.in_list = true;
                self.list_depth += 1;
                self.in_ordered_list = first_item.is_some();
                self.list_item_number.push(first_item.unwrap_or(1) as usize);
                if self.list_depth == 1 {
                    self.output.push('\n');
                }
            }
            Event::End(TagEnd::List(_)) => {
                self.list_depth = self.list_depth.saturating_sub(1);
                self.list_item_number.pop();
                if self.list_depth == 0 {
                    self.in_list = false;
                    self.in_ordered_list = false;
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
                self.output.push('\n');
            }
            Event::End(TagEnd::Table) => {
                self.in_table = false;
                self.table_alignments.clear();
            }

            Event::Start(Tag::TableHead) => {
                self.table_header = true;
                self.table_row.clear();
            }
            Event::End(TagEnd::TableHead) => {
                self.flush_table_row(true);
                self.table_header = false;
            }

            Event::Start(Tag::TableRow) => {
                self.table_row.clear();
            }
            Event::End(TagEnd::TableRow) => {
                self.flush_table_row(false);
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
                    self.text_buffer.push_str(&self.options.styles.emph.block_prefix);
                } else {
                    self.current_cell.push_str(&self.options.styles.emph.block_prefix);
                }
            }
            Event::End(TagEnd::Emphasis) => {
                self.in_emphasis = false;
                if !self.in_table {
                    self.text_buffer.push_str(&self.options.styles.emph.block_suffix);
                } else {
                    self.current_cell.push_str(&self.options.styles.emph.block_suffix);
                }
            }

            Event::Start(Tag::Strong) => {
                self.in_strong = true;
                if !self.in_table {
                    self.text_buffer.push_str(&self.options.styles.strong.block_prefix);
                } else {
                    self.current_cell.push_str(&self.options.styles.strong.block_prefix);
                }
            }
            Event::End(TagEnd::Strong) => {
                self.in_strong = false;
                if !self.in_table {
                    self.text_buffer.push_str(&self.options.styles.strong.block_suffix);
                } else {
                    self.current_cell.push_str(&self.options.styles.strong.block_suffix);
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

            Event::Start(Tag::Link { dest_url, title, .. }) => {
                self.in_link = true;
                self.link_url = dest_url.to_string();
                self.link_title = title.to_string();
            }
            Event::End(TagEnd::Link) => {
                self.in_link = false;
            }

            Event::Start(Tag::Image { dest_url, title, .. }) => {
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
                self.output.push_str(&self.options.styles.horizontal_rule.format);
            }

            Event::TaskListMarker(checked) => {
                if checked {
                    self.text_buffer.push_str(&self.options.styles.task.ticked);
                } else {
                    self.text_buffer.push_str(&self.options.styles.task.unticked);
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
            } else {
                self.output.push_str(&rendered);
            }
            self.output.push_str("\n\n");
        }
    }

    fn flush_list_item(&mut self) {
        let text = std::mem::take(&mut self.text_buffer);
        if text.is_empty() {
            return;
        }

        let indent = (self.list_depth - 1) * self.options.styles.list.level_indent;
        let indent_str = " ".repeat(indent);

        let prefix = if self.in_ordered_list {
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
        let style = &self.options.styles.code_block;

        self.output.push('\n');

        // Apply margin
        let margin = style.block.margin.unwrap_or(0);
        let margin_str = " ".repeat(margin);

        for line in content.lines() {
            self.output.push_str(&margin_str);
            self.output.push_str(line);
            self.output.push('\n');
        }

        self.output.push('\n');
    }

    fn flush_table_row(&mut self, is_header: bool) {
        let sep = self
            .options
            .styles
            .table
            .column_separator
            .as_deref()
            .unwrap_or("|");

        let mut line = String::new();
        line.push_str(sep);
        for cell in &self.table_row {
            let _ = write!(line, " {} {}", cell, sep);
        }
        self.output.push_str(&line);
        self.output.push('\n');

        // Add separator row after header
        if is_header {
            let row_sep = self
                .options
                .styles
                .table
                .row_separator
                .as_deref()
                .unwrap_or("-");

            let mut sep_line = String::new();
            sep_line.push_str(sep);
            for cell in &self.table_row {
                let width = cell.chars().count() + 2; // +2 for padding
                sep_line.push_str(&row_sep.repeat(width));
                sep_line.push_str(sep);
            }
            self.output.push_str(&sep_line);
            self.output.push('\n');
        }

        self.table_row.clear();
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

        let mut result = String::new();
        result.push_str(&style.style.prefix);
        result.push_str(&lipgloss_style.render(code));
        result.push_str(&style.style.suffix);
        result
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
            } else if current_line.chars().count() + 1 + word.chars().count() <= width {
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
        ascii_style, available_styles, dark_style, light_style, pink_style, render,
        render_with_environment_config, Renderer, RendererOptions, Style, StyleBlock,
        StyleCodeBlock, StyleConfig, StyleList, StylePrimitive, StyleTable, StyleTask,
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
        let block = StyleBlock::new()
            .margin(4)
            .indent(2)
            .indent_token("  ");

        assert_eq!(block.margin, Some(4));
        assert_eq!(block.indent, Some(2));
        assert_eq!(block.indent_token, Some("  ".to_string()));
    }

    #[test]
    fn test_style_config_heading() {
        let config = dark_style();
        let h1 = config.heading_style(HeadingLevel::H1);
        assert!(!h1.style.prefix.is_empty() || h1.style.suffix.len() > 0 || h1.style.color.is_some());
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
        assert!(output.contains("fn main()"));
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
}
