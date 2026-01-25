//! Rune/character sanitization utilities.
//!
//! This module provides utilities for sanitizing input characters, removing
//! control characters, and replacing tabs and newlines with configurable strings.
//!
//! # Example
//!
//! ```rust
//! use bubbles::runeutil::Sanitizer;
//!
//! let sanitizer = Sanitizer::new();
//! let input: Vec<char> = "hello\tworld\n".chars().collect();
//! let output = sanitizer.sanitize(&input);
//! // Tabs are replaced with 4 spaces, newlines preserved by default
//! assert_eq!(output.iter().collect::<String>(), "hello    world\n");
//! ```

/// A sanitizer for input characters.
///
/// The sanitizer removes control characters and optionally replaces
/// tabs and newlines with configurable strings.
#[derive(Debug, Clone)]
pub struct Sanitizer {
    replace_newline: Vec<char>,
    replace_tab: Vec<char>,
}

impl Default for Sanitizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Sanitizer {
    /// Creates a new sanitizer with default settings.
    ///
    /// Default settings:
    /// - Newlines are preserved as-is (`\n`)
    /// - Tabs are replaced with 4 spaces
    #[must_use]
    pub fn new() -> Self {
        Self {
            replace_newline: vec!['\n'],
            replace_tab: vec![' ', ' ', ' ', ' '],
        }
    }

    /// Creates a sanitizer builder for custom configuration.
    #[must_use]
    pub fn builder() -> SanitizerBuilder {
        SanitizerBuilder::new()
    }

    /// Sets the replacement string for tabs.
    #[must_use]
    pub fn with_tab_replacement(mut self, replacement: &str) -> Self {
        self.replace_tab = replacement.chars().collect();
        self
    }

    /// Sets the replacement string for newlines.
    #[must_use]
    pub fn with_newline_replacement(mut self, replacement: &str) -> Self {
        self.replace_newline = replacement.chars().collect();
        self
    }

    /// Sanitizes the input characters.
    ///
    /// This method:
    /// - Removes Unicode replacement characters (`U+FFFD`)
    /// - Replaces `\r\n` (CRLF), `\r`, and `\n` with the configured newline replacement
    /// - Replaces `\t` with the configured tab replacement
    /// - Removes other control characters
    ///
    /// # Example
    ///
    /// ```rust
    /// use bubbles::runeutil::Sanitizer;
    ///
    /// let sanitizer = Sanitizer::new()
    ///     .with_tab_replacement("  ")
    ///     .with_newline_replacement(" ");
    ///
    /// let input: Vec<char> = "a\tb\nc\r\nd".chars().collect();
    /// let output = sanitizer.sanitize(&input);
    /// assert_eq!(output.iter().collect::<String>(), "a  b c d");
    /// ```
    #[must_use]
    pub fn sanitize(&self, runes: &[char]) -> Vec<char> {
        let mut result = Vec::with_capacity(runes.len());
        let mut iter = runes.iter().peekable();

        while let Some(&r) = iter.next() {
            match r {
                // Skip Unicode replacement character
                '\u{FFFD}' => {}

                // Handle Carriage Return
                '\r' => {
                    // Check if next char is Newline (CRLF)
                    if let Some(&&next) = iter.peek()
                        && next == '\n'
                    {
                        continue; // Skip \r, let \n be processed
                    }
                    // Lone \r becomes newline
                    result.extend(&self.replace_newline);
                }

                // Handle Newline
                '\n' => {
                    result.extend(&self.replace_newline);
                }

                // Replace tab
                '\t' => {
                    result.extend(&self.replace_tab);
                }

                // Skip other control characters
                c if c.is_control() => {}

                // Keep regular characters
                c => {
                    result.push(c);
                }
            }
        }

        result
    }

    /// Sanitizes a string, returning a new sanitized string.
    ///
    /// Convenience method that converts to/from chars internally.
    #[must_use]
    pub fn sanitize_string(&self, s: &str) -> String {
        let chars: Vec<char> = s.chars().collect();
        self.sanitize(&chars).into_iter().collect()
    }
}

/// Builder for creating a [`Sanitizer`] with custom settings.
#[derive(Debug, Clone, Default)]
pub struct SanitizerBuilder {
    replace_newline: Option<Vec<char>>,
    replace_tab: Option<Vec<char>>,
}

impl SanitizerBuilder {
    /// Creates a new builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the replacement string for tabs.
    #[must_use]
    pub fn replace_tabs(mut self, replacement: &str) -> Self {
        self.replace_tab = Some(replacement.chars().collect());
        self
    }

    /// Sets the replacement string for newlines.
    #[must_use]
    pub fn replace_newlines(mut self, replacement: &str) -> Self {
        self.replace_newline = Some(replacement.chars().collect());
        self
    }

    /// Builds the sanitizer.
    #[must_use]
    pub fn build(self) -> Sanitizer {
        Sanitizer {
            replace_newline: self.replace_newline.unwrap_or_else(|| vec!['\n']),
            replace_tab: self.replace_tab.unwrap_or_else(|| vec![' ', ' ', ' ', ' ']),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_basic() {
        let sanitizer = Sanitizer::new();
        let input: Vec<char> = "hello world".chars().collect();
        let output = sanitizer.sanitize(&input);
        assert_eq!(output.iter().collect::<String>(), "hello world");
    }

    #[test]
    fn test_sanitize_tabs() {
        let sanitizer = Sanitizer::new();
        let input: Vec<char> = "hello\tworld".chars().collect();
        let output = sanitizer.sanitize(&input);
        assert_eq!(output.iter().collect::<String>(), "hello    world");
    }

    #[test]
    fn test_sanitize_custom_tabs() {
        let sanitizer = Sanitizer::new().with_tab_replacement("  ");
        let input: Vec<char> = "a\tb".chars().collect();
        let output = sanitizer.sanitize(&input);
        assert_eq!(output.iter().collect::<String>(), "a  b");
    }

    #[test]
    fn test_sanitize_newlines() {
        let sanitizer = Sanitizer::new();
        let input: Vec<char> = "hello\nworld".chars().collect();
        let output = sanitizer.sanitize(&input);
        assert_eq!(output.iter().collect::<String>(), "hello\nworld");
    }

    #[test]
    fn test_sanitize_custom_newlines() {
        let sanitizer = Sanitizer::new().with_newline_replacement(" ");
        let input: Vec<char> = "hello\nworld".chars().collect();
        let output = sanitizer.sanitize(&input);
        assert_eq!(output.iter().collect::<String>(), "hello world");
    }

    #[test]
    fn test_sanitize_carriage_return() {
        let sanitizer = Sanitizer::new().with_newline_replacement("");
        let input: Vec<char> = "hello\r\nworld".chars().collect();
        let output = sanitizer.sanitize(&input);
        assert_eq!(output.iter().collect::<String>(), "helloworld");
    }

    #[test]
    fn test_sanitize_control_chars() {
        let sanitizer = Sanitizer::new();
        // Include some control characters (ASCII 0x01, 0x02, etc.)
        let input: Vec<char> = "hello\x01\x02world".chars().collect();
        let output = sanitizer.sanitize(&input);
        assert_eq!(output.iter().collect::<String>(), "helloworld");
    }

    #[test]
    fn test_sanitize_unicode_replacement() {
        let sanitizer = Sanitizer::new();
        let input: Vec<char> = "hello\u{FFFD}world".chars().collect();
        let output = sanitizer.sanitize(&input);
        assert_eq!(output.iter().collect::<String>(), "helloworld");
    }

    #[test]
    fn test_sanitize_string() {
        let sanitizer = Sanitizer::new().with_tab_replacement("--");
        let output = sanitizer.sanitize_string("a\tb");
        assert_eq!(output, "a--b");
    }

    #[test]
    fn test_builder() {
        let sanitizer = Sanitizer::builder()
            .replace_tabs("  ")
            .replace_newlines("")
            .build();

        let output = sanitizer.sanitize_string("a\tb\nc");
        assert_eq!(output, "a  bc");
    }

    #[test]
    fn test_unicode_preserved() {
        let sanitizer = Sanitizer::new();
        let input: Vec<char> = "hello 世界 🌍".chars().collect();
        let output = sanitizer.sanitize(&input);
        assert_eq!(output.iter().collect::<String>(), "hello 世界 🌍");
    }
}
