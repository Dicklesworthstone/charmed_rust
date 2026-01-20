//! Syntax highlighting support for code blocks.
//!
//! This module provides language detection and syntax highlighting
//! using the [syntect](https://crates.io/crates/syntect) library.
//!
//! # Example
//!
//! ```rust,ignore
//! use glamour::syntax::LanguageDetector;
//!
//! let detector = LanguageDetector::new();
//! let syntax = detector.detect("rust");
//! assert!(detector.is_supported("rust"));
//! assert!(detector.is_supported("rs")); // Alias works too
//! ```

use lipgloss::{RgbColor, Style as LipglossStyle};
use std::sync::LazyLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle as SynFontStyle, Style as SynStyle, Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

/// Lazily loaded syntax set containing all default language definitions.
///
/// This is loaded on first use to avoid startup overhead when syntax
/// highlighting is not used.
pub static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);

/// Lazily loaded theme set containing all default syntax themes.
///
/// This is loaded on first use to avoid startup overhead when syntax
/// highlighting is not used.
pub static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

/// Maps markdown language identifiers to syntect syntax definitions.
///
/// Handles common language aliases (e.g., "js" → "javascript", "rs" → "rust")
/// and provides fallback to plain text for unknown languages.
#[derive(Debug, Clone)]
pub struct LanguageDetector;

impl Default for LanguageDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageDetector {
    /// Creates a new language detector.
    ///
    /// The underlying syntax set is lazily loaded on first use.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Finds the syntax definition for a language identifier.
    ///
    /// This method handles common aliases and performs case-insensitive matching.
    /// If the language is not recognized, returns the plain text syntax.
    ///
    /// # Arguments
    ///
    /// * `lang` - Language identifier from markdown code fence (e.g., "rust", "js", "py")
    ///
    /// # Returns
    ///
    /// A reference to the syntax definition. Never panics.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let detector = LanguageDetector::new();
    /// let syntax = detector.detect("rs"); // Returns Rust syntax
    /// assert_eq!(syntax.name, "Rust");
    /// ```
    #[must_use]
    pub fn detect(&self, lang: &str) -> &'static SyntaxReference {
        let lang_lower = lang.to_lowercase().trim().to_string();

        // Handle empty language string
        if lang_lower.is_empty() {
            return SYNTAX_SET.find_syntax_plain_text();
        }

        // Try direct match first (syntect's find_syntax_by_token is case-insensitive)
        if let Some(syntax) = SYNTAX_SET.find_syntax_by_token(&lang_lower) {
            return syntax;
        }

        // Try common aliases
        let canonical = Self::resolve_alias(&lang_lower);

        if canonical != lang_lower
            && let Some(syntax) = SYNTAX_SET.find_syntax_by_token(canonical)
        {
            return syntax;
        }

        // Try by file extension
        if let Some(syntax) = SYNTAX_SET.find_syntax_by_extension(&lang_lower) {
            return syntax;
        }

        // Fallback to plain text
        SYNTAX_SET.find_syntax_plain_text()
    }

    /// Resolves common language aliases to their canonical names.
    ///
    /// # Arguments
    ///
    /// * `lang` - Lowercase language identifier
    ///
    /// # Returns
    ///
    /// The canonical language name if an alias is found, otherwise the original.
    fn resolve_alias(lang: &str) -> &str {
        match lang {
            // JavaScript/TypeScript
            "js" | "mjs" | "cjs" => "javascript",
            "ts" | "mts" | "cts" => "typescript",
            "jsx" => "javascript",
            "tsx" => "typescript",

            // Rust
            "rs" => "rust",

            // Python
            "py" | "python3" | "py3" => "python",
            "pyw" => "python",

            // Ruby
            "rb" => "ruby",

            // Shell
            "sh" | "bash" | "zsh" | "fish" | "ksh" => "shell",
            "shell" => "bash",
            "shellscript" => "bash",

            // Markup
            "md" | "markdown" => "markdown",
            "htm" => "html",

            // Config files
            "yml" => "yaml",
            "dockerfile" => "docker",

            // C family
            "c++" | "cxx" | "hpp" | "hxx" | "cc" | "hh" => "cpp",
            "h" => "c",
            "objc" => "objective-c",
            "objcpp" | "objc++" => "objective-c++",

            // JVM languages
            "kt" | "kts" => "kotlin",
            "scala" => "scala",
            "groovy" => "groovy",
            "clj" | "cljs" | "cljc" => "clojure",

            // .NET languages
            "cs" | "csharp" => "c#",
            "fs" | "fsharp" => "f#",
            "vb" => "visual basic",

            // Go
            "go" | "golang" => "go",

            // Erlang/Elixir
            "ex" | "exs" => "elixir",
            "erl" | "hrl" => "erlang",

            // Haskell
            "hs" | "lhs" => "haskell",

            // Lisp family
            "el" | "elisp" | "emacs-lisp" => "lisp",
            "rkt" | "scm" | "ss" => "scheme",

            // ML family
            "ml" | "mli" => "ocaml",
            "sml" => "standard ml",

            // Data formats
            "jsonc" => "json",
            "json5" => "json",

            // Misc
            "tf" | "hcl" => "terraform",
            "tex" | "latex" => "latex",
            "r" => "r",
            "pl" | "pm" => "perl",
            "php" | "php3" | "php4" | "php5" | "php7" | "php8" | "phtml" => "php",
            "lua" => "lua",
            "swift" => "swift",
            "dart" => "dart",
            "vim" | "vimscript" => "viml",
            "ps1" | "psm1" | "psd1" => "powershell",
            "bat" | "cmd" => "batch file",
            "asm" | "s" | "S" => "assembly",
            "nim" => "nim",
            "zig" => "zig",
            "v" => "v",
            "crystal" | "cr" => "crystal",
            "d" => "d",
            "ada" | "adb" | "ads" => "ada",
            "fortran" | "f" | "f90" | "f95" | "f03" | "f08" => "fortran",
            "cobol" | "cob" | "cbl" => "cobol",
            "pascal" | "pas" => "pascal",
            "makefile" | "make" | "mk" => "makefile",
            "cmake" => "cmake",
            "nginx" => "nginx",
            "apache" => "apacheconf",
            "diff" | "patch" => "diff",
            "graphql" | "gql" => "graphql",
            "proto" | "protobuf" => "protocol buffers",
            "thrift" => "thrift",
            "svg" => "xml",
            "xslt" | "xsl" => "xml",
            "vue" => "vue",
            "svelte" => "svelte",
            "scss" => "scss",
            "sass" => "sass",
            "less" => "less",
            "styl" | "stylus" => "stylus",
            "pug" | "jade" => "pug",
            "haml" => "haml",
            "slim" => "slim",
            "erb" => "html (rails)",
            "ejs" => "ejs",
            "jinja" | "jinja2" | "j2" => "jinja2",
            "handlebars" | "hbs" => "handlebars",
            "mustache" => "mustache",
            "twig" => "twig",
            "nunjucks" | "njk" => "nunjucks",
            "liquid" => "liquid",

            // No alias found
            _ => lang,
        }
    }

    /// Checks if a language identifier is supported for syntax highlighting.
    ///
    /// A language is considered supported if it resolves to something other
    /// than plain text.
    ///
    /// # Arguments
    ///
    /// * `lang` - Language identifier to check
    ///
    /// # Returns
    ///
    /// `true` if the language has syntax highlighting support, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let detector = LanguageDetector::new();
    /// assert!(detector.is_supported("rust"));
    /// assert!(detector.is_supported("rs"));
    /// assert!(!detector.is_supported("unknown-lang"));
    /// ```
    #[must_use]
    pub fn is_supported(&self, lang: &str) -> bool {
        let syntax = self.detect(lang);
        syntax.name != "Plain Text"
    }

    /// Returns the number of supported language syntaxes.
    ///
    /// This counts the total number of syntax definitions available,
    /// not including aliases.
    #[must_use]
    pub fn syntax_count() -> usize {
        SYNTAX_SET.syntaxes().len()
    }

    /// Returns a list of all supported language identifiers.
    ///
    /// This includes both the canonical names from syntect and
    /// common aliases.
    #[must_use]
    pub fn supported_languages() -> Vec<&'static str> {
        vec![
            // Rust
            "rust",
            "rs",
            // Python
            "python",
            "py",
            "py3",
            // JavaScript/TypeScript
            "javascript",
            "js",
            "mjs",
            "cjs",
            "typescript",
            "ts",
            "mts",
            "jsx",
            "tsx",
            // Go
            "go",
            "golang",
            // C family
            "c",
            "cpp",
            "c++",
            "cxx",
            "h",
            "hpp",
            // Java/JVM
            "java",
            "kotlin",
            "kt",
            "scala",
            "groovy",
            "clojure",
            "clj",
            // .NET
            "csharp",
            "cs",
            "c#",
            "fsharp",
            "fs",
            "f#",
            // Ruby
            "ruby",
            "rb",
            // Shell
            "bash",
            "sh",
            "zsh",
            "shell",
            "fish",
            // Web
            "html",
            "htm",
            "css",
            "scss",
            "sass",
            "less",
            // Data formats
            "json",
            "jsonc",
            "yaml",
            "yml",
            "toml",
            "xml",
            "csv",
            // Markdown
            "markdown",
            "md",
            // SQL
            "sql",
            // Other
            "php",
            "perl",
            "pl",
            "lua",
            "swift",
            "objective-c",
            "objc",
            "r",
            "haskell",
            "hs",
            "elixir",
            "ex",
            "erlang",
            "erl",
            "ocaml",
            "ml",
            "lisp",
            "scheme",
            "makefile",
            "make",
            "dockerfile",
            "docker",
            "nginx",
            "diff",
            "patch",
            "graphql",
            "gql",
            "protobuf",
            "proto",
            "terraform",
            "tf",
            "hcl",
            "powershell",
            "ps1",
            "batch",
            "bat",
            "cmd",
            "vim",
            "viml",
            "latex",
            "tex",
            "asm",
            "assembly",
        ]
    }
}

// ============================================================================
// Theme Mapping: syntect -> lipgloss
// ============================================================================

/// Converts a syntect highlighting style to a lipgloss terminal style.
///
/// This function maps syntect's GUI-oriented style (designed for text editors)
/// to lipgloss's terminal-oriented style with ANSI escape sequences.
///
/// # Arguments
///
/// * `syn_style` - A syntect `Style` containing foreground, background, and font attributes
///
/// # Returns
///
/// A lipgloss `Style` with the corresponding colors and text attributes.
///
/// # Example
///
/// ```rust,ignore
/// use syntect::highlighting::Style as SynStyle;
/// use glamour::syntax::syntect_to_lipgloss;
///
/// let syn_style = SynStyle::default();
/// let lip_style = syntect_to_lipgloss(syn_style);
/// ```
#[must_use]
pub fn syntect_to_lipgloss(syn_style: SynStyle) -> LipglossStyle {
    let mut style = LipglossStyle::new();

    // Map foreground color (RGBA → RGB)
    let fg = syn_style.foreground;
    style = style.foreground_color(RgbColor::new(fg.r, fg.g, fg.b));

    // Map background color (if not transparent)
    // Transparent backgrounds (a=0) are common in themes for "inherit from editor"
    let bg = syn_style.background;
    if bg.a > 0 {
        style = style.background_color(RgbColor::new(bg.r, bg.g, bg.b));
    }

    // Map font styles (bitflags)
    let font = syn_style.font_style;
    if font.contains(SynFontStyle::BOLD) {
        style = style.bold();
    }
    if font.contains(SynFontStyle::ITALIC) {
        style = style.italic();
    }
    if font.contains(SynFontStyle::UNDERLINE) {
        style = style.underline();
    }

    style
}

/// A wrapper around syntect themes providing terminal-appropriate defaults
/// and lipgloss style conversion.
///
/// # Example
///
/// ```rust,ignore
/// use glamour::syntax::SyntaxTheme;
///
/// let theme = SyntaxTheme::from_name("base16-ocean.dark").unwrap();
/// println!("Using theme: {}", theme.name());
/// ```
#[derive(Debug, Clone)]
pub struct SyntaxTheme {
    name: String,
    inner: Theme,
}

impl SyntaxTheme {
    /// Loads a built-in theme by name.
    ///
    /// # Arguments
    ///
    /// * `name` - Theme name (e.g., "base16-ocean.dark", "Solarized (dark)")
    ///
    /// # Returns
    ///
    /// `Some(SyntaxTheme)` if the theme exists, `None` otherwise.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        THEME_SET.themes.get(name).map(|theme| Self {
            name: name.to_string(),
            inner: theme.clone(),
        })
    }

    /// Returns the default dark theme (base16-ocean.dark).
    #[must_use]
    pub fn default_dark() -> Self {
        Self::from_name("base16-ocean.dark").expect("base16-ocean.dark should be a built-in theme")
    }

    /// Returns the default light theme (InspiredGitHub).
    #[must_use]
    pub fn default_light() -> Self {
        Self::from_name("InspiredGitHub").expect("InspiredGitHub should be a built-in theme")
    }

    /// Returns the theme name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns a reference to the underlying syntect theme.
    #[must_use]
    pub fn inner(&self) -> &Theme {
        &self.inner
    }

    /// Returns a list of all available built-in theme names.
    #[must_use]
    pub fn available_themes() -> Vec<&'static str> {
        vec![
            "base16-ocean.dark",
            "base16-eighties.dark",
            "base16-mocha.dark",
            "InspiredGitHub",
            "Solarized (dark)",
            "Solarized (light)",
        ]
    }

    /// Returns the background color of this theme, if set.
    #[must_use]
    pub fn background_color(&self) -> Option<(u8, u8, u8)> {
        self.inner.settings.background.map(|c| (c.r, c.g, c.b))
    }

    /// Returns the default foreground color of this theme, if set.
    #[must_use]
    pub fn foreground_color(&self) -> Option<(u8, u8, u8)> {
        self.inner.settings.foreground.map(|c| (c.r, c.g, c.b))
    }
}

impl Default for SyntaxTheme {
    fn default() -> Self {
        Self::default_dark()
    }
}

/// A cache for converted lipgloss styles to avoid repeated conversions.
///
/// Syntect styles are converted to lipgloss styles on-demand and cached
/// for future use, improving performance when highlighting large code blocks.
///
/// # Example
///
/// ```rust,ignore
/// use glamour::syntax::StyleCache;
/// use syntect::highlighting::Style;
///
/// let mut cache = StyleCache::new();
/// let syn_style = Style::default();
/// let lip_style = cache.get_or_convert(syn_style);
/// ```
#[derive(Debug, Default)]
pub struct StyleCache {
    /// Maps syntect styles (by their debug representation) to lipgloss styles.
    /// Note: SynStyle doesn't implement Hash, so we use a Vec for simplicity.
    /// For most code blocks, the number of unique styles is small (<20).
    cache: Vec<(SynStyle, LipglossStyle)>,
}

impl StyleCache {
    /// Creates a new empty style cache.
    #[must_use]
    pub fn new() -> Self {
        Self { cache: Vec::new() }
    }

    /// Gets the lipgloss style for a syntect style, converting and caching if needed.
    ///
    /// # Arguments
    ///
    /// * `syn_style` - The syntect style to convert
    ///
    /// # Returns
    ///
    /// A reference to the cached lipgloss style.
    pub fn get_or_convert(&mut self, syn_style: SynStyle) -> &LipglossStyle {
        // Check if we already have this style cached
        let pos = self
            .cache
            .iter()
            .position(|(s, _)| styles_equal(s, &syn_style));

        if let Some(idx) = pos {
            &self.cache[idx].1
        } else {
            // Convert and cache
            let lip_style = syntect_to_lipgloss(syn_style);
            self.cache.push((syn_style, lip_style));
            &self.cache.last().unwrap().1
        }
    }

    /// Clears the cache, freeing memory.
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Returns the number of cached styles.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Returns true if the cache is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

/// Compares two syntect styles for equality.
///
/// SynStyle doesn't implement Eq, so we compare field by field.
fn styles_equal(a: &SynStyle, b: &SynStyle) -> bool {
    a.foreground.r == b.foreground.r
        && a.foreground.g == b.foreground.g
        && a.foreground.b == b.foreground.b
        && a.foreground.a == b.foreground.a
        && a.background.r == b.background.r
        && a.background.g == b.background.g
        && a.background.b == b.background.b
        && a.background.a == b.background.a
        && a.font_style == b.font_style
}

fn is_json_language(language: &str) -> bool {
    matches!(
        language.trim().to_ascii_lowercase().as_str(),
        "json" | "jsonc" | "json5"
    )
}

fn is_default_foreground(style: &SynStyle, theme: &SyntaxTheme) -> bool {
    if style.foreground.a == 0 {
        return true;
    }

    match theme.foreground_color() {
        Some((r, g, b)) => {
            style.foreground.r == r && style.foreground.g == g && style.foreground.b == b
        }
        None => false,
    }
}

fn adjust_channel(value: u8, delta: i16) -> u8 {
    let shifted = value as i16 + delta;
    if shifted < 0 {
        0
    } else if shifted > 255 {
        255
    } else {
        shifted as u8
    }
}

fn relative_luminance(r: u8, g: u8, b: u8) -> f32 {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn json_punctuation_style(theme: &SyntaxTheme) -> LipglossStyle {
    let base = theme.foreground_color().unwrap_or((180, 180, 180));
    let delta = theme
        .background_color()
        .map(|(r, g, b)| {
            if relative_luminance(r, g, b) < 0.5 {
                40
            } else {
                -40
            }
        })
        .unwrap_or(-40);

    let mut adjusted = (
        adjust_channel(base.0, delta),
        adjust_channel(base.1, delta),
        adjust_channel(base.2, delta),
    );

    if adjusted == base {
        let alt = if delta > 0 { -60 } else { 60 };
        adjusted = (
            adjust_channel(base.0, alt),
            adjust_channel(base.1, alt),
            adjust_channel(base.2, alt),
        );
    }

    LipglossStyle::new().foreground_color(RgbColor::new(adjusted.0, adjusted.1, adjusted.2))
}

fn is_json_punctuation(ch: char) -> bool {
    matches!(ch, '{' | '}' | '[' | ']' | ':' | ',' | '"')
}

fn render_with_json_punctuation(
    default_style: &LipglossStyle,
    punctuation_style: &LipglossStyle,
    text: &str,
    output: &mut String,
) {
    let mut start = 0;
    for (idx, ch) in text.char_indices() {
        if is_json_punctuation(ch) {
            if start < idx {
                output.push_str(&default_style.render(&text[start..idx]));
            }
            let mut buf = [0u8; 4];
            let encoded = ch.encode_utf8(&mut buf);
            output.push_str(&punctuation_style.render(encoded));
            start = idx + ch.len_utf8();
        }
    }
    if start < text.len() {
        output.push_str(&default_style.render(&text[start..]));
    }
}

/// Highlights code with syntax highlighting and returns styled text.
///
/// This is the main entry point for syntax highlighting. It takes source code,
/// a language identifier, and a theme, and returns the code with ANSI escape
/// sequences for terminal rendering.
///
/// # Arguments
///
/// * `code` - The source code to highlight
/// * `language` - Language identifier (e.g., "rust", "python", "js")
/// * `theme` - The syntax theme to use
///
/// # Returns
///
/// A string with ANSI escape sequences for terminal rendering.
///
/// # Example
///
/// ```rust,ignore
/// use glamour::syntax::{highlight_code, SyntaxTheme};
///
/// let code = "fn main() { println!(\"Hello!\"); }";
/// let theme = SyntaxTheme::default_dark();
/// let highlighted = highlight_code(code, "rust", &theme);
/// println!("{}", highlighted);
/// ```
#[must_use]
pub fn highlight_code(code: &str, language: &str, theme: &SyntaxTheme) -> String {
    let detector = LanguageDetector::new();
    let syntax = detector.detect(language);

    let mut highlighter = HighlightLines::new(syntax, theme.inner());
    let mut cache = StyleCache::new();
    let mut output = String::with_capacity(code.len() * 2);
    let json_punct_style = is_json_language(language).then(|| json_punctuation_style(theme));

    for line in LinesWithEndings::from(code) {
        match highlighter.highlight_line(line, &SYNTAX_SET) {
            Ok(regions) => {
                for (syn_style, text) in regions {
                    let lip_style = cache.get_or_convert(syn_style);
                    // Check if text ends with newline before rendering
                    // (lipgloss render may strip trailing whitespace)
                    let ends_with_newline = text.ends_with('\n');
                    let trimmed = text.trim_end_matches('\n');
                    if let Some(punctuation_style) = json_punct_style.as_ref()
                        && is_default_foreground(&syn_style, theme)
                        && trimmed.chars().any(is_json_punctuation)
                    {
                        render_with_json_punctuation(
                            &lip_style,
                            punctuation_style,
                            trimmed,
                            &mut output,
                        );
                    } else {
                        output.push_str(&lip_style.render(trimmed));
                    }
                    if ends_with_newline {
                        output.push('\n');
                    }
                }
            }
            Err(_) => {
                // On error, output plain text
                output.push_str(line);
            }
        }
    }

    output
}

/// Generates a preview of a theme with sample Rust code.
///
/// Useful for displaying available themes to users.
///
/// # Arguments
///
/// * `theme_name` - Name of the theme to preview
///
/// # Returns
///
/// `Some(String)` with highlighted sample code, or `None` if theme not found.
///
/// # Example
///
/// ```rust,ignore
/// use glamour::syntax::preview_theme;
///
/// if let Some(preview) = preview_theme("base16-ocean.dark") {
///     println!("{}", preview);
/// }
/// ```
#[must_use]
pub fn preview_theme(theme_name: &str) -> Option<String> {
    let theme = SyntaxTheme::from_name(theme_name)?;

    let sample = r#"// Sample Rust code
fn main() {
    let greeting = "Hello, World!";
    let numbers: Vec<i32> = (1..=5).collect();

    for n in &numbers {
        println!("{}: {}", n, greeting);
    }
}
"#;

    Some(highlight_code(sample, "rust", &theme))
}

/// Converts an RGB color to the nearest xterm-256 color code.
///
/// This is useful for terminals that don't support true color (24-bit RGB).
///
/// # Arguments
///
/// * `r`, `g`, `b` - RGB color components (0-255)
///
/// # Returns
///
/// The nearest xterm-256 color code (0-255).
#[must_use]
pub fn rgb_to_256(r: u8, g: u8, b: u8) -> u8 {
    // Check for grayscale
    if r == g && g == b {
        if r < 8 {
            return 16; // Black
        }
        if r > 248 {
            return 231; // White
        }
        // Grayscale ramp (232-255)
        return 232 + ((r as u16 - 8) / 10) as u8;
    }

    // Color cube (16-231)
    // Each axis has 6 values: 0, 95, 135, 175, 215, 255
    let r_idx = if r < 48 {
        0
    } else {
        ((r as u16 - 35) / 40) as u8
    };
    let g_idx = if g < 48 {
        0
    } else {
        ((g as u16 - 35) / 40) as u8
    };
    let b_idx = if b < 48 {
        0
    } else {
        ((b as u16 - 35) / 40) as u8
    };

    16 + 36 * r_idx.min(5) + 6 * g_idx.min(5) + b_idx.min(5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let detector = LanguageDetector::new();
        // Should not panic
        let _ = detector.detect("rust");
    }

    #[test]
    fn test_direct_language_match() {
        let detector = LanguageDetector::new();

        // These should match directly
        assert!(detector.is_supported("rust"));
        assert!(detector.is_supported("python"));
        assert!(detector.is_supported("javascript"));
        assert!(detector.is_supported("html"));
        assert!(detector.is_supported("css"));
        assert!(detector.is_supported("json"));
        assert!(detector.is_supported("yaml"));
    }

    #[test]
    fn test_rust_aliases() {
        let detector = LanguageDetector::new();
        let rust = detector.detect("rust");

        assert_eq!(detector.detect("rs").name, rust.name);
        assert_eq!(detector.detect("RS").name, rust.name);
        assert_eq!(detector.detect("Rust").name, rust.name);
    }

    #[test]
    fn test_javascript_aliases() {
        let detector = LanguageDetector::new();
        let js = detector.detect("javascript");

        assert_eq!(detector.detect("js").name, js.name);
        assert_eq!(detector.detect("JS").name, js.name);
        assert_eq!(detector.detect("mjs").name, js.name);
        assert_eq!(detector.detect("cjs").name, js.name);
    }

    #[test]
    fn test_typescript_aliases() {
        let detector = LanguageDetector::new();
        let ts = detector.detect("typescript");

        assert_eq!(detector.detect("ts").name, ts.name);
        assert_eq!(detector.detect("TS").name, ts.name);
        assert_eq!(detector.detect("mts").name, ts.name);
        assert_eq!(detector.detect("cts").name, ts.name);
    }

    #[test]
    fn test_python_aliases() {
        let detector = LanguageDetector::new();
        let py = detector.detect("python");

        assert_eq!(detector.detect("py").name, py.name);
        assert_eq!(detector.detect("PY").name, py.name);
        assert_eq!(detector.detect("python3").name, py.name);
        assert_eq!(detector.detect("py3").name, py.name);
    }

    #[test]
    fn test_ruby_aliases() {
        let detector = LanguageDetector::new();
        let rb = detector.detect("ruby");

        assert_eq!(detector.detect("rb").name, rb.name);
        assert_eq!(detector.detect("RB").name, rb.name);
    }

    #[test]
    fn test_shell_aliases() {
        let detector = LanguageDetector::new();

        // These should all resolve to a shell-like syntax
        assert!(detector.is_supported("bash"));
        assert!(detector.is_supported("sh"));
        assert!(detector.is_supported("zsh"));
        assert!(detector.is_supported("shell"));
    }

    #[test]
    fn test_go_aliases() {
        let detector = LanguageDetector::new();
        let go = detector.detect("go");

        assert_eq!(detector.detect("golang").name, go.name);
        assert_eq!(detector.detect("Go").name, go.name);
    }

    #[test]
    fn test_cpp_aliases() {
        let detector = LanguageDetector::new();
        let cpp = detector.detect("cpp");

        assert_eq!(detector.detect("c++").name, cpp.name);
        assert_eq!(detector.detect("cxx").name, cpp.name);
        assert_eq!(detector.detect("CPP").name, cpp.name);
    }

    #[test]
    fn test_yaml_aliases() {
        let detector = LanguageDetector::new();
        let yaml = detector.detect("yaml");

        assert_eq!(detector.detect("yml").name, yaml.name);
        assert_eq!(detector.detect("YML").name, yaml.name);
    }

    #[test]
    fn test_markdown_aliases() {
        let detector = LanguageDetector::new();
        let md = detector.detect("markdown");

        assert_eq!(detector.detect("md").name, md.name);
        assert_eq!(detector.detect("MD").name, md.name);
    }

    #[test]
    fn test_case_insensitive() {
        let detector = LanguageDetector::new();

        // All of these should resolve to the same syntax
        let lower = detector.detect("rust");
        let upper = detector.detect("RUST");
        let mixed = detector.detect("Rust");

        assert_eq!(lower.name, upper.name);
        assert_eq!(lower.name, mixed.name);
    }

    #[test]
    fn test_unknown_language_fallback() {
        let detector = LanguageDetector::new();

        // Unknown languages should fall back to plain text
        let plain = detector.detect("totally-unknown-language-xyz123");
        assert_eq!(plain.name, "Plain Text");

        // is_supported should return false
        assert!(!detector.is_supported("totally-unknown-language-xyz123"));
    }

    #[test]
    fn test_empty_language() {
        let detector = LanguageDetector::new();

        let plain = detector.detect("");
        assert_eq!(plain.name, "Plain Text");

        assert!(!detector.is_supported(""));
    }

    #[test]
    fn test_whitespace_handling() {
        let detector = LanguageDetector::new();

        // Whitespace should be trimmed
        let rust = detector.detect("rust");
        assert_eq!(detector.detect("  rust  ").name, rust.name);
        assert_eq!(detector.detect("\trust\n").name, rust.name);
    }

    #[test]
    fn test_no_panic_on_any_input() {
        let detector = LanguageDetector::new();

        // Test various edge cases that might cause panics
        let _ = detector.detect("");
        let _ = detector.detect("   ");
        let _ = detector.detect("\n\n\n");
        let _ = detector.detect("a".repeat(1000).as_str());
        let _ = detector.detect("!@#$%^&*()");
        let _ = detector.detect("🦀");
        let _ = detector.detect("日本語");
        let _ = detector.detect("null");
        let _ = detector.detect("undefined");
        let _ = detector.detect("NaN");

        // None of these should panic
    }

    #[test]
    fn test_syntax_count() {
        let count = LanguageDetector::syntax_count();
        // syntect includes ~60 languages by default
        assert!(count >= 50, "Expected at least 50 syntaxes, got {}", count);
    }

    #[test]
    fn test_supported_languages_list() {
        let langs = LanguageDetector::supported_languages();

        // Should have at least 30 entries
        assert!(
            langs.len() >= 30,
            "Expected at least 30 languages, got {}",
            langs.len()
        );

        // Check some expected languages are in the list
        assert!(langs.contains(&"rust"));
        assert!(langs.contains(&"rs"));
        assert!(langs.contains(&"python"));
        assert!(langs.contains(&"py"));
        assert!(langs.contains(&"javascript"));
        assert!(langs.contains(&"js"));
    }

    #[test]
    fn test_csharp_aliases() {
        let detector = LanguageDetector::new();

        assert!(detector.is_supported("c#"));
        assert!(detector.is_supported("cs"));
        assert!(detector.is_supported("csharp"));
    }

    #[test]
    fn test_kotlin_aliases() {
        let detector = LanguageDetector::new();
        let kotlin = detector.detect("kotlin");

        assert_eq!(detector.detect("kt").name, kotlin.name);
        assert_eq!(detector.detect("kts").name, kotlin.name);
    }

    #[test]
    fn test_html_aliases() {
        let detector = LanguageDetector::new();
        let html = detector.detect("html");

        assert_eq!(detector.detect("htm").name, html.name);
    }

    #[test]
    fn test_docker_aliases() {
        let detector = LanguageDetector::new();

        // Dockerfile maps to YAML-like syntax if available, otherwise plain text
        // Note: syntect's default set doesn't include Dockerfile syntax
        let dockerfile = detector.detect("dockerfile");
        let docker = detector.detect("docker");
        // Both should resolve to the same thing (even if it's plain text)
        assert_eq!(dockerfile.name, docker.name);
    }

    #[test]
    fn test_json_aliases() {
        let detector = LanguageDetector::new();
        let json = detector.detect("json");

        assert_eq!(detector.detect("jsonc").name, json.name);
        assert_eq!(detector.detect("json5").name, json.name);
    }

    #[test]
    fn test_default_impl() {
        let detector = LanguageDetector::default();
        assert!(detector.is_supported("rust"));
    }

    #[test]
    fn test_elixir_aliases() {
        let detector = LanguageDetector::new();
        let elixir = detector.detect("elixir");

        assert_eq!(detector.detect("ex").name, elixir.name);
        assert_eq!(detector.detect("exs").name, elixir.name);
    }

    #[test]
    fn test_haskell_aliases() {
        let detector = LanguageDetector::new();
        let haskell = detector.detect("haskell");

        assert_eq!(detector.detect("hs").name, haskell.name);
    }

    #[test]
    fn test_ocaml_aliases() {
        let detector = LanguageDetector::new();
        let ocaml = detector.detect("ocaml");

        assert_eq!(detector.detect("ml").name, ocaml.name);
        assert_eq!(detector.detect("mli").name, ocaml.name);
    }

    #[test]
    fn test_perl_aliases() {
        let detector = LanguageDetector::new();
        let perl = detector.detect("perl");

        assert_eq!(detector.detect("pl").name, perl.name);
        assert_eq!(detector.detect("pm").name, perl.name);
    }

    #[test]
    fn test_php_aliases() {
        let detector = LanguageDetector::new();
        let php = detector.detect("php");

        assert_eq!(detector.detect("php3").name, php.name);
        assert_eq!(detector.detect("php7").name, php.name);
        assert_eq!(detector.detect("phtml").name, php.name);
    }

    #[test]
    fn test_powershell_aliases() {
        let detector = LanguageDetector::new();
        let ps = detector.detect("powershell");

        assert_eq!(detector.detect("ps1").name, ps.name);
        assert_eq!(detector.detect("psm1").name, ps.name);
    }

    #[test]
    fn test_terraform_aliases() {
        let detector = LanguageDetector::new();

        // Terraform/HCL aliases should all resolve consistently
        // Note: syntect's default set may not include Terraform syntax
        let terraform = detector.detect("terraform");
        let tf = detector.detect("tf");
        let hcl = detector.detect("hcl");

        // All should resolve to the same syntax
        assert_eq!(terraform.name, tf.name);
        assert_eq!(terraform.name, hcl.name);
    }

    #[test]
    fn test_latex_aliases() {
        let detector = LanguageDetector::new();
        let latex = detector.detect("latex");

        assert_eq!(detector.detect("tex").name, latex.name);
    }

    #[test]
    fn test_makefile_aliases() {
        let detector = LanguageDetector::new();
        let makefile = detector.detect("makefile");

        assert_eq!(detector.detect("make").name, makefile.name);
        assert_eq!(detector.detect("mk").name, makefile.name);
    }

    #[test]
    fn test_diff_aliases() {
        let detector = LanguageDetector::new();
        let diff = detector.detect("diff");

        assert_eq!(detector.detect("patch").name, diff.name);
    }

    #[test]
    fn test_clojure_aliases() {
        let detector = LanguageDetector::new();
        let clj = detector.detect("clojure");

        assert_eq!(detector.detect("clj").name, clj.name);
        assert_eq!(detector.detect("cljs").name, clj.name);
        assert_eq!(detector.detect("cljc").name, clj.name);
    }

    #[test]
    fn test_erlang_aliases() {
        let detector = LanguageDetector::new();
        let erl = detector.detect("erlang");

        assert_eq!(detector.detect("erl").name, erl.name);
        assert_eq!(detector.detect("hrl").name, erl.name);
    }

    // ========================================================================
    // Theme Mapping Tests
    // ========================================================================

    #[test]
    fn test_syntect_to_lipgloss_basic() {
        use syntect::highlighting::Color as SynColor;

        let syn_style = SynStyle {
            foreground: SynColor {
                r: 255,
                g: 128,
                b: 64,
                a: 255,
            },
            background: SynColor {
                r: 0,
                g: 0,
                b: 0,
                a: 0, // transparent
            },
            font_style: SynFontStyle::empty(),
        };

        let lip_style = syntect_to_lipgloss(syn_style);
        let rendered = lip_style.render("test");
        assert!(rendered.contains("test"));
        assert!(rendered.contains('\x1b'));
    }

    #[test]
    fn test_syntect_to_lipgloss_with_background() {
        use syntect::highlighting::Color as SynColor;

        let syn_style = SynStyle {
            foreground: SynColor {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
            background: SynColor {
                r: 40,
                g: 44,
                b: 52,
                a: 255, // opaque background
            },
            font_style: SynFontStyle::empty(),
        };

        let lip_style = syntect_to_lipgloss(syn_style);
        let rendered = lip_style.render("text");
        assert!(rendered.contains("text"));
        assert!(rendered.contains('\x1b'));
    }

    #[test]
    fn test_syntect_to_lipgloss_font_styles() {
        use syntect::highlighting::Color as SynColor;

        let bold_style = SynStyle {
            foreground: SynColor {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
            background: SynColor {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
            font_style: SynFontStyle::BOLD,
        };
        let lip_bold = syntect_to_lipgloss(bold_style);
        let rendered = lip_bold.render("bold");
        assert!(rendered.contains('\x1b'));

        let combined_style = SynStyle {
            foreground: SynColor {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
            background: SynColor {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
            font_style: SynFontStyle::BOLD | SynFontStyle::ITALIC | SynFontStyle::UNDERLINE,
        };
        let lip_combined = syntect_to_lipgloss(combined_style);
        let rendered = lip_combined.render("styled");
        assert!(rendered.contains('\x1b'));
    }

    #[test]
    fn test_syntax_theme_from_name() {
        let theme = SyntaxTheme::from_name("base16-ocean.dark");
        assert!(theme.is_some());
        assert_eq!(theme.unwrap().name(), "base16-ocean.dark");

        let invalid = SyntaxTheme::from_name("nonexistent-theme-xyz");
        assert!(invalid.is_none());
    }

    #[test]
    fn test_syntax_theme_defaults() {
        let dark = SyntaxTheme::default_dark();
        assert_eq!(dark.name(), "base16-ocean.dark");

        let light = SyntaxTheme::default_light();
        assert_eq!(light.name(), "InspiredGitHub");

        let default = SyntaxTheme::default();
        assert_eq!(default.name(), "base16-ocean.dark");
    }

    #[test]
    fn test_syntax_theme_colors() {
        let theme = SyntaxTheme::default_dark();
        assert!(theme.background_color().is_some());
        assert!(theme.foreground_color().is_some());
    }

    #[test]
    fn test_syntax_theme_available_themes() {
        let themes = SyntaxTheme::available_themes();
        assert!(themes.len() >= 5);
        assert!(themes.contains(&"base16-ocean.dark"));
        assert!(themes.contains(&"InspiredGitHub"));
    }

    #[test]
    fn test_style_cache_basic() {
        use syntect::highlighting::Color as SynColor;

        let mut cache = StyleCache::new();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);

        let style1 = SynStyle {
            foreground: SynColor {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            },
            background: SynColor {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
            font_style: SynFontStyle::empty(),
        };

        let _ = cache.get_or_convert(style1);
        assert_eq!(cache.len(), 1);

        let _ = cache.get_or_convert(style1);
        assert_eq!(cache.len(), 1); // Still 1, reused cache

        let style2 = SynStyle {
            foreground: SynColor {
                r: 0,
                g: 255,
                b: 0,
                a: 255,
            },
            background: SynColor {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
            font_style: SynFontStyle::empty(),
        };
        let _ = cache.get_or_convert(style2);
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_style_cache_clear() {
        use syntect::highlighting::Color as SynColor;

        let mut cache = StyleCache::new();
        let style = SynStyle {
            foreground: SynColor {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            },
            background: SynColor {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
            font_style: SynFontStyle::empty(),
        };

        let _ = cache.get_or_convert(style);
        assert_eq!(cache.len(), 1);

        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_highlight_code_rust() {
        let code = "fn main() { println!(\"Hello\"); }";
        let theme = SyntaxTheme::default_dark();
        let highlighted = highlight_code(code, "rust", &theme);

        assert!(highlighted.contains("fn"));
        assert!(highlighted.contains("main"));
        assert!(highlighted.contains('\x1b'));
    }

    #[test]
    fn test_highlight_code_unknown_language() {
        let code = "some random text";
        let theme = SyntaxTheme::default_dark();
        let highlighted = highlight_code(code, "unknown-lang-xyz", &theme);
        assert!(highlighted.contains("some random text"));
    }

    #[test]
    fn test_highlight_code_multiline() {
        let code = "fn foo() {\n    let x = 1;\n    x + 1\n}";
        let theme = SyntaxTheme::default_dark();
        let highlighted = highlight_code(code, "rust", &theme);

        // Check that the output contains the code content
        assert!(highlighted.contains("fn"));
        assert!(highlighted.contains("foo"));

        // LinesWithEndings preserves line endings in each line's text.
        // The output should be longer than input due to ANSI escape sequences.
        assert!(highlighted.len() > code.len());

        // The highlighted output should contain multiple distinct lines
        // when the ANSI escape codes are stripped
        let line_count = highlighted.matches('\n').count();
        assert!(
            line_count >= 3,
            "Expected at least 3 newlines, got {}. Output: {:?}",
            line_count,
            highlighted
        );
    }

    #[test]
    fn test_preview_theme_valid() {
        let preview = preview_theme("base16-ocean.dark");
        assert!(preview.is_some());
        let content = preview.unwrap();
        assert!(content.contains("fn"));
        assert!(content.contains('\x1b'));
    }

    #[test]
    fn test_preview_theme_invalid() {
        let preview = preview_theme("nonexistent-theme");
        assert!(preview.is_none());
    }

    #[test]
    fn test_rgb_to_256_black() {
        assert_eq!(rgb_to_256(0, 0, 0), 16);
    }

    #[test]
    fn test_rgb_to_256_white() {
        assert_eq!(rgb_to_256(255, 255, 255), 231);
    }

    #[test]
    fn test_rgb_to_256_grayscale() {
        let gray = rgb_to_256(128, 128, 128);
        assert!(gray >= 232 || gray == 16 || gray == 231);
    }

    #[test]
    fn test_rgb_to_256_primary_colors() {
        let red = rgb_to_256(255, 0, 0);
        assert!(red >= 16 && red <= 231);

        let green = rgb_to_256(0, 255, 0);
        assert!(green >= 16 && green <= 231);

        let blue = rgb_to_256(0, 0, 255);
        assert!(blue >= 16 && blue <= 231);
    }

    #[test]
    fn test_rgb_to_256_range() {
        // Verify rgb_to_256 doesn't panic for various inputs
        // and returns values in valid xterm-256 range (16-255)
        for r in [0u8, 64, 128, 192, 255] {
            for g in [0u8, 64, 128, 192, 255] {
                for b in [0u8, 64, 128, 192, 255] {
                    let result = rgb_to_256(r, g, b);
                    // Result should be in xterm-256 range (16=black through 255=white)
                    assert!(result >= 16, "Color code {} should be >= 16", result);
                }
            }
        }
    }

    #[test]
    fn test_styles_equal() {
        use syntect::highlighting::Color as SynColor;

        let style1 = SynStyle {
            foreground: SynColor {
                r: 255,
                g: 128,
                b: 64,
                a: 255,
            },
            background: SynColor {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
            font_style: SynFontStyle::BOLD,
        };

        let style2 = style1;
        let style3 = SynStyle {
            foreground: SynColor {
                r: 0,
                g: 128,
                b: 64,
                a: 255,
            },
            background: SynColor {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
            font_style: SynFontStyle::BOLD,
        };

        assert!(styles_equal(&style1, &style2));
        assert!(!styles_equal(&style1, &style3));
    }

    #[test]
    fn test_theme_set_lazy_loading() {
        let theme_count = THEME_SET.themes.len();
        assert!(theme_count >= 5);
    }

    // ========================================================================
    // Additional Tests for charmed_rust-417
    // ========================================================================

    #[test]
    fn test_all_builtin_themes_load() {
        for theme_name in SyntaxTheme::available_themes() {
            let theme = SyntaxTheme::from_name(theme_name);
            assert!(
                theme.is_some(),
                "Theme '{}' should load successfully",
                theme_name
            );
        }
    }

    #[test]
    fn test_themes_produce_different_output() {
        let code = "fn main() { let x = 42; }";
        let output_ocean = highlight_code(
            code,
            "rust",
            &SyntaxTheme::from_name("base16-ocean.dark").unwrap(),
        );
        let output_solarized = highlight_code(
            code,
            "rust",
            &SyntaxTheme::from_name("Solarized (dark)").unwrap(),
        );

        // Different themes should produce different ANSI sequences
        assert_ne!(
            output_ocean, output_solarized,
            "Different themes should produce different output"
        );

        // Both should still contain the code content
        assert!(output_ocean.contains("fn"));
        assert!(output_solarized.contains("fn"));
    }

    #[test]
    fn test_highlight_empty_code() {
        let theme = SyntaxTheme::default_dark();
        let highlighted = highlight_code("", "rust", &theme);
        assert_eq!(highlighted, "");
    }

    #[test]
    fn test_highlight_whitespace_only() {
        let theme = SyntaxTheme::default_dark();

        // Test single space
        let single_space = highlight_code(" ", "rust", &theme);
        assert!(single_space.contains(' ') || single_space.is_empty());

        // Test multiple newlines
        let newlines = highlight_code("\n\n\n", "rust", &theme);
        // Should preserve newlines
        let newline_count = newlines.matches('\n').count();
        assert!(
            newline_count >= 2,
            "Expected at least 2 newlines, got {}",
            newline_count
        );
    }

    #[test]
    fn test_highlight_preserves_trailing_newline() {
        let theme = SyntaxTheme::default_dark();
        let code = "fn main() {}\n";
        let highlighted = highlight_code(code, "rust", &theme);
        assert!(
            highlighted.ends_with('\n'),
            "Highlighted code should preserve trailing newline"
        );
    }

    /// Helper to strip ANSI escape codes from a string for content verification.
    fn strip_ansi(s: &str) -> String {
        // Matches ANSI escape sequences: ESC [ ... m (SGR sequences)
        let mut result = String::with_capacity(s.len());
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\x1b' {
                // Skip the escape sequence
                if chars.peek() == Some(&'[') {
                    chars.next(); // consume '['
                    // Skip until we hit 'm'
                    while let Some(&next) = chars.peek() {
                        chars.next();
                        if next == 'm' {
                            break;
                        }
                    }
                } else {
                    result.push(c);
                }
            } else {
                result.push(c);
            }
        }
        result
    }

    #[test]
    fn test_highlight_handles_tabs() {
        // Note: Lipgloss normalizes tabs to spaces (4 spaces per tab) for
        // consistent terminal rendering. This test verifies that tab-indented
        // code is correctly processed and the indentation is preserved as spaces.
        let theme = SyntaxTheme::default_dark();
        let code = "fn main() {\n\tlet x = 1;\n}";
        let highlighted = highlight_code(code, "rust", &theme);
        // Strip ANSI codes to verify content
        let stripped = strip_ansi(&highlighted);

        // Tabs are normalized to 4 spaces by lipgloss
        assert!(
            stripped.contains("    let x"),
            "Tab indentation should be converted to spaces"
        );
        // Content should still be present
        assert!(stripped.contains("fn main()"));
        assert!(stripped.contains("let x = 1"));
    }

    #[test]
    fn test_highlight_unicode_in_strings() {
        let theme = SyntaxTheme::default_dark();
        let code = r#"let emoji = "🦀";"#;
        let highlighted = highlight_code(code, "rust", &theme);
        assert!(highlighted.contains("🦀"), "Unicode should be preserved");
    }

    // ========================================================================
    // Performance Tests
    // ========================================================================

    #[test]
    fn test_large_file_highlighting_completes() {
        let theme = SyntaxTheme::default_dark();

        // Generate 100 lines of code (not 1000 to keep test fast)
        let code: String = (0..100)
            .map(|i| format!("fn func_{}() {{ let x = {}; }}\n", i, i))
            .collect();

        let start = std::time::Instant::now();
        let highlighted = highlight_code(&code, "rust", &theme);
        let duration = start.elapsed();

        // Should complete in under 5 seconds (generous for CI)
        assert!(
            duration.as_secs() < 5,
            "Highlighting took too long: {:?}",
            duration
        );

        // Output should contain all function names
        assert!(highlighted.contains("func_0"));
        assert!(highlighted.contains("func_99"));
    }

    #[test]
    fn test_style_cache_reduces_allocations() {
        use syntect::highlighting::Color as SynColor;

        let mut cache = StyleCache::new();

        // Create the same style 100 times
        let style = SynStyle {
            foreground: SynColor {
                r: 255,
                g: 128,
                b: 64,
                a: 255,
            },
            background: SynColor {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
            font_style: SynFontStyle::BOLD,
        };

        // First call adds to cache
        let _ = cache.get_or_convert(style);
        assert_eq!(cache.len(), 1);

        // Subsequent calls should reuse cache
        for _ in 0..99 {
            let _ = cache.get_or_convert(style);
        }
        assert_eq!(
            cache.len(),
            1,
            "Cache should have only 1 entry for identical styles"
        );
    }

    #[test]
    fn test_multiple_language_detection_performance() {
        let detector = LanguageDetector::new();

        let start = std::time::Instant::now();
        // Detect 1000 languages (mix of known and unknown)
        for _ in 0..100 {
            let _ = detector.detect("rust");
            let _ = detector.detect("python");
            let _ = detector.detect("javascript");
            let _ = detector.detect("unknown-lang");
            let _ = detector.detect("go");
            let _ = detector.detect("ts");
            let _ = detector.detect("cpp");
            let _ = detector.detect("rb");
            let _ = detector.detect("yaml");
            let _ = detector.detect("json");
        }
        let duration = start.elapsed();

        // 1000 detections should complete in under 1 second
        assert!(
            duration.as_millis() < 1000,
            "Language detection took too long: {:?}",
            duration
        );
    }

    // ========================================================================
    // Edge Case Tests
    // ========================================================================

    #[test]
    fn test_highlight_very_long_line() {
        let theme = SyntaxTheme::default_dark();
        let long_line = format!("let x = \"{}\";", "a".repeat(1000));
        let highlighted = highlight_code(&long_line, "rust", &theme);
        assert!(
            highlighted.len() > long_line.len(),
            "Output should have ANSI codes"
        );
    }

    #[test]
    fn test_highlight_mixed_indentation() {
        // Note: Lipgloss normalizes tabs to 4 spaces. Mixed indentation
        // (spaces + tabs) will all be rendered as spaces.
        let theme = SyntaxTheme::default_dark();
        let code = "fn main() {\n  let a = 1;\n\tlet b = 2;\n    let c = 3;\n}";
        let highlighted = highlight_code(code, "rust", &theme);
        // Strip ANSI codes to verify content
        let stripped = strip_ansi(&highlighted);

        // 2-space indent preserved
        assert!(
            stripped.contains("  let a"),
            "Two-space indentation should be preserved"
        );
        // Tab converted to 4 spaces
        assert!(
            stripped.contains("    let b"),
            "Tab should be converted to 4-space indentation"
        );
        // 4-space indent preserved
        assert!(
            stripped.contains("    let c"),
            "Four-space indentation should be preserved"
        );
        // All content present
        assert!(stripped.contains("let a = 1"));
        assert!(stripped.contains("let b = 2"));
        assert!(stripped.contains("let c = 3"));
    }

    #[test]
    fn test_highlight_code_with_errors() {
        // Code that's syntactically incomplete but should still highlight
        let theme = SyntaxTheme::default_dark();
        let code = "fn main( { let x =";
        let highlighted = highlight_code(code, "rust", &theme);
        // Should not panic and should contain the tokens
        assert!(highlighted.contains("fn"));
        assert!(highlighted.contains("main"));
    }

    #[test]
    fn test_canonical_language_names_match_expected() {
        let detector = LanguageDetector::new();

        // Verify canonical names for common languages
        assert_eq!(detector.detect("rust").name, "Rust");
        assert_eq!(detector.detect("python").name, "Python");
        assert_eq!(detector.detect("javascript").name, "JavaScript");
        assert_eq!(detector.detect("go").name, "Go");
        assert_eq!(detector.detect("html").name, "HTML");
        assert_eq!(detector.detect("css").name, "CSS");
        assert_eq!(detector.detect("json").name, "JSON");
    }

    #[test]
    fn test_supported_languages_complete() {
        let supported = LanguageDetector::supported_languages();

        // Must support at least 30 languages (as per requirements)
        assert!(
            supported.len() >= 30,
            "Expected at least 30 languages, got {}",
            supported.len()
        );

        // Must include common languages
        assert!(supported.contains(&"rust"), "Must support rust");
        assert!(supported.contains(&"python"), "Must support python");
        assert!(supported.contains(&"javascript"), "Must support javascript");
        assert!(supported.contains(&"go"), "Must support go");
        assert!(supported.contains(&"java"), "Must support java");
    }
}
