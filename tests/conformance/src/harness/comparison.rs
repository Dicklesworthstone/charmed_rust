//! OutputComparator - Diff generation for comparing expected vs actual outputs
//!
//! Provides utilities for comparing test outputs with:
//! - ANSI escape sequence normalization
//! - Whitespace normalization (trailing, newlines)
//! - Unicode normalization (NFC)
//! - Floating point comparison with epsilon
//! - Case-insensitive comparison
//! - Detailed diff generation for mismatches

use similar::TextDiff;
use std::fmt::Debug;
use unicode_normalization::UnicodeNormalization;

/// Type of difference detected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffType {
    /// Strings differ in individual characters
    CharacterDiff,
    /// Multi-line strings differ in lines
    LineDiff,
    /// Strings have different lengths
    LengthDiff,
    /// Types differ (for Debug comparison)
    TypeDiff,
    /// Float values exceed epsilon
    FloatDiff,
}

/// Detailed diff information
#[derive(Debug, Clone, PartialEq)]
pub struct Diff {
    /// The expected value as string
    pub expected: String,
    /// The actual value as string
    pub actual: String,
    /// Position of first difference (byte offset)
    pub first_diff_pos: Option<usize>,
    /// Line number of first difference (1-indexed)
    pub first_diff_line: Option<usize>,
    /// Human-readable inline diff showing exact position
    pub inline_diff: String,
    /// Unified diff format for multi-line content
    pub unified_diff: String,
    /// Type of difference
    pub diff_type: DiffType,
}

impl Diff {
    /// Get a human-readable description of the difference
    pub fn describe(&self) -> String {
        let mut desc = String::new();

        match self.diff_type {
            DiffType::CharacterDiff => {
                if let Some(pos) = self.first_diff_pos {
                    desc.push_str(&format!("Character difference at position {}", pos));
                } else {
                    desc.push_str("Character difference detected");
                }
            }
            DiffType::LineDiff => {
                if let Some(line) = self.first_diff_line {
                    desc.push_str(&format!("Line difference at line {}", line));
                } else {
                    desc.push_str("Line difference detected");
                }
            }
            DiffType::LengthDiff => {
                desc.push_str(&format!(
                    "Length difference: expected {} bytes, got {} bytes",
                    self.expected.len(),
                    self.actual.len()
                ));
            }
            DiffType::TypeDiff => {
                desc.push_str("Type representation differs");
            }
            DiffType::FloatDiff => {
                desc.push_str("Floating point values differ beyond epsilon");
            }
        }

        desc
    }

    /// Format for plain text output
    pub fn format_plain(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("Expected: {:?}\n", self.expected));
        output.push_str(&format!("Actual:   {:?}\n", self.actual));
        output.push_str(&self.describe());
        output.push('\n');
        if !self.inline_diff.is_empty() {
            output.push_str(&format!("Inline: {}\n", self.inline_diff));
        }
        output
    }
}

/// Result of comparing two values
#[derive(Debug, Clone, PartialEq)]
pub enum CompareResult {
    /// Values are exactly equal
    Equal,
    /// Values are different, with detailed diff
    Different(Diff),
    /// Floating point values are approximately equal within epsilon
    ApproximatelyEqual {
        delta: f64,
        epsilon: f64,
        values: (f64, f64),
    },
}

impl CompareResult {
    /// Returns true if the comparison passed (Equal or ApproximatelyEqual)
    pub fn is_pass(&self) -> bool {
        matches!(
            self,
            CompareResult::Equal | CompareResult::ApproximatelyEqual { .. }
        )
    }

    /// Returns true if the comparison failed
    pub fn is_fail(&self) -> bool {
        matches!(self, CompareResult::Different(_))
    }

    /// Get the diff if this is a Different result
    pub fn diff(&self) -> Option<&Diff> {
        match self {
            CompareResult::Different(diff) => Some(diff),
            _ => None,
        }
    }
}

/// Options for whitespace normalization
#[derive(Debug, Clone, Copy, Default)]
pub struct WhitespaceOptions {
    /// Remove trailing whitespace from each line
    pub trim_trailing: bool,
    /// Convert \r\n to \n
    pub normalize_newlines: bool,
    /// Collapse multiple consecutive blank lines into one
    pub collapse_blank_lines: bool,
    /// Remove final trailing newline
    pub trim_final_newline: bool,
}

/// Options for comparison
#[derive(Debug, Clone, Default)]
pub struct CompareOptions {
    /// Normalize ANSI escape sequences (sort SGR parameters)
    pub ansi_normalize: bool,
    /// Whitespace normalization options
    pub whitespace_options: WhitespaceOptions,
    /// Enable general whitespace normalization
    pub whitespace_normalize: bool,
    /// Unicode NFC normalization
    pub unicode_normalize: bool,
    /// Default epsilon for float comparisons
    pub float_epsilon: Option<f64>,
    /// Case-insensitive comparison
    pub ignore_case: bool,
}

/// Comparator for test outputs with various comparison modes
#[derive(Debug, Clone, Default)]
pub struct OutputComparator {
    options: CompareOptions,
}

impl OutputComparator {
    /// Create a new comparator with default settings
    pub fn new() -> Self {
        Self {
            options: CompareOptions::default(),
        }
    }

    /// Create a comparator with custom options
    pub fn with_options(options: CompareOptions) -> Self {
        Self { options }
    }

    /// Enable ANSI escape sequence normalization
    pub fn ansi_normalize(mut self, enabled: bool) -> Self {
        self.options.ansi_normalize = enabled;
        self
    }

    /// Enable whitespace normalization
    pub fn whitespace_normalize(mut self, enabled: bool) -> Self {
        self.options.whitespace_normalize = enabled;
        if enabled {
            self.options.whitespace_options = WhitespaceOptions {
                trim_trailing: true,
                normalize_newlines: true,
                collapse_blank_lines: false,
                trim_final_newline: true,
            };
        }
        self
    }

    /// Enable Unicode NFC normalization
    pub fn unicode_normalize(mut self, enabled: bool) -> Self {
        self.options.unicode_normalize = enabled;
        self
    }

    /// Set default float epsilon
    pub fn float_epsilon(mut self, epsilon: f64) -> Self {
        self.options.float_epsilon = Some(epsilon);
        self
    }

    /// Enable case-insensitive comparison
    pub fn ignore_case(mut self, enabled: bool) -> Self {
        self.options.ignore_case = enabled;
        self
    }

    /// Normalize a string according to configured options
    fn normalize(&self, s: &str) -> String {
        let mut result = s.to_string();

        // Unicode normalization first
        if self.options.unicode_normalize {
            result = result.nfc().collect();
        }

        // ANSI normalization
        if self.options.ansi_normalize {
            result = normalize_ansi(&result);
        }

        // Whitespace normalization
        if self.options.whitespace_normalize {
            let opts = &self.options.whitespace_options;

            // Normalize newlines first
            if opts.normalize_newlines {
                result = result.replace("\r\n", "\n").replace('\r', "\n");
            }

            // Trim trailing whitespace per line
            if opts.trim_trailing {
                result = result
                    .lines()
                    .map(|line| line.trim_end())
                    .collect::<Vec<_>>()
                    .join("\n");
            }

            // Collapse blank lines
            if opts.collapse_blank_lines {
                let mut new_result = String::new();
                let mut prev_blank = false;
                for line in result.lines() {
                    let is_blank = line.trim().is_empty();
                    if is_blank && prev_blank {
                        continue;
                    }
                    if !new_result.is_empty() {
                        new_result.push('\n');
                    }
                    new_result.push_str(line);
                    prev_blank = is_blank;
                }
                result = new_result;
            }

            // Trim final newline
            if opts.trim_final_newline {
                while result.ends_with('\n') {
                    result.pop();
                }
            }
        }

        // Case normalization
        if self.options.ignore_case {
            result = result.to_lowercase();
        }

        result
    }

    /// Find the position of the first difference between two strings
    fn find_first_diff(expected: &str, actual: &str) -> Option<usize> {
        expected
            .chars()
            .zip(actual.chars())
            .position(|(e, a)| e != a)
            .or_else(|| {
                if expected.len() != actual.len() {
                    Some(expected.len().min(actual.len()))
                } else {
                    None
                }
            })
    }

    /// Find the line number of the first difference
    fn find_first_diff_line(expected: &str, actual: &str) -> Option<usize> {
        let expected_lines: Vec<&str> = expected.lines().collect();
        let actual_lines: Vec<&str> = actual.lines().collect();

        for (i, (e, a)) in expected_lines.iter().zip(actual_lines.iter()).enumerate() {
            if e != a {
                return Some(i + 1); // 1-indexed
            }
        }

        if expected_lines.len() != actual_lines.len() {
            return Some(expected_lines.len().min(actual_lines.len()) + 1);
        }

        None
    }

    /// Generate inline diff showing the exact position of difference
    fn generate_inline_diff(expected: &str, actual: &str) -> String {
        if let Some(pos) = Self::find_first_diff(expected, actual) {
            let exp_char = expected.chars().nth(pos);
            let act_char = actual.chars().nth(pos);

            match (exp_char, act_char) {
                (Some(e), Some(a)) => {
                    format!(
                        "At position {}: expected {:?} (0x{:02x}), got {:?} (0x{:02x})",
                        pos,
                        e,
                        e as u32,
                        a,
                        a as u32
                    )
                }
                (Some(e), None) => {
                    format!(
                        "At position {}: expected {:?}, but actual string ended",
                        pos, e
                    )
                }
                (None, Some(a)) => {
                    format!(
                        "At position {}: expected end, but got {:?}",
                        pos, a
                    )
                }
                (None, None) => String::new(),
            }
        } else {
            String::new()
        }
    }

    /// Generate unified diff format
    fn generate_unified_diff(expected: &str, actual: &str) -> String {
        let diff = TextDiff::from_lines(expected, actual);
        let mut result = String::new();

        result.push_str("--- expected\n");
        result.push_str("+++ actual\n");

        for hunk in diff.unified_diff().iter_hunks() {
            result.push_str(&format!("{}", hunk));
        }

        result
    }

    /// Create a Diff struct from two strings
    fn create_diff(&self, expected: &str, actual: &str, diff_type: DiffType) -> Diff {
        Diff {
            expected: expected.to_string(),
            actual: actual.to_string(),
            first_diff_pos: Self::find_first_diff(expected, actual),
            first_diff_line: Self::find_first_diff_line(expected, actual),
            inline_diff: Self::generate_inline_diff(expected, actual),
            unified_diff: Self::generate_unified_diff(expected, actual),
            diff_type,
        }
    }

    /// Determine the diff type based on string comparison
    fn determine_diff_type(expected: &str, actual: &str) -> DiffType {
        if expected.len() != actual.len() {
            DiffType::LengthDiff
        } else if expected.contains('\n') || actual.contains('\n') {
            DiffType::LineDiff
        } else {
            DiffType::CharacterDiff
        }
    }

    /// Compare two strings with all configured normalizations
    pub fn compare_str(&self, expected: &str, actual: &str) -> CompareResult {
        let norm_expected = self.normalize(expected);
        let norm_actual = self.normalize(actual);

        if norm_expected == norm_actual {
            return CompareResult::Equal;
        }

        let diff_type = Self::determine_diff_type(&norm_expected, &norm_actual);
        CompareResult::Different(self.create_diff(&norm_expected, &norm_actual, diff_type))
    }

    /// Compare two strings with ANSI escape sequence normalization
    pub fn compare_ansi(&self, expected: &str, actual: &str) -> CompareResult {
        let norm_expected = normalize_ansi(expected);
        let norm_actual = normalize_ansi(actual);

        if norm_expected == norm_actual {
            return CompareResult::Equal;
        }

        let diff_type = Self::determine_diff_type(&norm_expected, &norm_actual);
        CompareResult::Different(self.create_diff(&norm_expected, &norm_actual, diff_type))
    }

    /// Compare multi-line strings with line-based diff
    pub fn compare_lines(&self, expected: &str, actual: &str) -> CompareResult {
        let norm_expected = self.normalize(expected);
        let norm_actual = self.normalize(actual);

        if norm_expected == norm_actual {
            return CompareResult::Equal;
        }

        CompareResult::Different(self.create_diff(&norm_expected, &norm_actual, DiffType::LineDiff))
    }

    /// Compare two floating point values with epsilon tolerance
    pub fn compare_f64(&self, expected: f64, actual: f64, epsilon: f64) -> CompareResult {
        // Handle special cases
        if expected.is_nan() && actual.is_nan() {
            // NaN == NaN for testing purposes (both are "not a number")
            return CompareResult::Equal;
        }
        if expected.is_nan() || actual.is_nan() {
            return CompareResult::Different(Diff {
                expected: expected.to_string(),
                actual: actual.to_string(),
                first_diff_pos: None,
                first_diff_line: None,
                inline_diff: "One value is NaN".to_string(),
                unified_diff: String::new(),
                diff_type: DiffType::FloatDiff,
            });
        }

        // Handle infinity
        if expected.is_infinite() && actual.is_infinite() {
            if expected.signum() == actual.signum() {
                return CompareResult::Equal;
            }
        }

        let delta = (expected - actual).abs();
        if delta <= epsilon {
            if delta == 0.0 {
                CompareResult::Equal
            } else {
                CompareResult::ApproximatelyEqual {
                    delta,
                    epsilon,
                    values: (expected, actual),
                }
            }
        } else {
            CompareResult::Different(Diff {
                expected: expected.to_string(),
                actual: actual.to_string(),
                first_diff_pos: None,
                first_diff_line: None,
                inline_diff: format!(
                    "delta {} exceeds epsilon {} (expected: {}, actual: {})",
                    delta, epsilon, expected, actual
                ),
                unified_diff: String::new(),
                diff_type: DiffType::FloatDiff,
            })
        }
    }

    /// Compare two values using their Debug representation
    pub fn compare_debug<T: Debug>(&self, expected: &T, actual: &T) -> CompareResult {
        let expected_str = format!("{:?}", expected);
        let actual_str = format!("{:?}", actual);

        if expected_str == actual_str {
            return CompareResult::Equal;
        }

        CompareResult::Different(self.create_diff(&expected_str, &actual_str, DiffType::TypeDiff))
    }

    /// Compare bytes
    pub fn compare_bytes(&self, expected: &[u8], actual: &[u8]) -> CompareResult {
        if expected == actual {
            return CompareResult::Equal;
        }

        // Find first differing byte
        let first_diff = expected
            .iter()
            .zip(actual.iter())
            .position(|(e, a)| e != a)
            .unwrap_or(expected.len().min(actual.len()));

        let diff = Diff {
            expected: format!("{:?}", expected),
            actual: format!("{:?}", actual),
            first_diff_pos: Some(first_diff),
            first_diff_line: None,
            inline_diff: format!(
                "First byte difference at position {}: expected 0x{:02x}, got 0x{:02x}",
                first_diff,
                expected.get(first_diff).copied().unwrap_or(0),
                actual.get(first_diff).copied().unwrap_or(0)
            ),
            unified_diff: String::new(),
            diff_type: if expected.len() != actual.len() {
                DiffType::LengthDiff
            } else {
                DiffType::CharacterDiff
            },
        };

        CompareResult::Different(diff)
    }
}

/// Normalize ANSI escape sequences by sorting SGR parameters
///
/// ANSI escape sequences can be ordered differently but produce the same visual output:
/// - `\x1b[31;1m` == `\x1b[1;31m` (red bold vs bold red)
///
/// This function normalizes by sorting the numeric parameters.
fn normalize_ansi(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Start of escape sequence
            let mut escape = String::new();
            escape.push(c);

            // Collect the rest of the escape sequence
            while let Some(&next) = chars.peek() {
                escape.push(chars.next().unwrap());
                if next.is_ascii_alphabetic() {
                    break;
                }
            }

            // Normalize if it's an SGR sequence (ends with 'm')
            if escape.starts_with("\x1b[") && escape.ends_with('m') {
                let normalized = normalize_sgr(&escape);
                result.push_str(&normalized);
            } else {
                result.push_str(&escape);
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Normalize an SGR (Select Graphic Rendition) sequence by sorting parameters
fn normalize_sgr(escape: &str) -> String {
    // Extract parameters between \x1b[ and m
    let params_str = &escape[2..escape.len() - 1];

    if params_str.is_empty() {
        return escape.to_string();
    }

    // Parse parameters
    let mut params: Vec<u32> = params_str
        .split(';')
        .filter_map(|p| p.parse().ok())
        .collect();

    // Sort parameters (this normalizes "31;1" and "1;31" to the same form)
    params.sort();

    // Reconstruct
    let sorted_params: Vec<String> = params.iter().map(|p| p.to_string()).collect();
    format!("\x1b[{}m", sorted_params.join(";"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let cmp = OutputComparator::new();
        let result = cmp.compare_str("hello", "hello");
        assert!(matches!(result, CompareResult::Equal));
    }

    #[test]
    fn test_difference_detection() {
        let cmp = OutputComparator::new();
        let result = cmp.compare_str("hello", "hallo");

        match result {
            CompareResult::Different(diff) => {
                assert_eq!(diff.first_diff_pos, Some(1));
                assert!(diff.inline_diff.contains("position 1"));
                assert_eq!(diff.diff_type, DiffType::CharacterDiff);
            }
            _ => panic!("Should detect difference"),
        }
    }

    #[test]
    fn test_ansi_normalization() {
        let cmp = OutputComparator::new();

        // Same visual output, different sequence order
        let result = cmp.compare_ansi("\x1b[31;1mHello\x1b[0m", "\x1b[1;31mHello\x1b[0m");
        assert!(
            matches!(result, CompareResult::Equal),
            "ANSI sequences with same params should normalize to equal"
        );
    }

    #[test]
    fn test_ansi_different_codes() {
        let cmp = OutputComparator::new();

        // Different colors should still be different
        let result = cmp.compare_ansi(
            "\x1b[31mHello\x1b[0m", // red
            "\x1b[32mHello\x1b[0m", // green
        );
        assert!(matches!(result, CompareResult::Different(_)));
    }

    #[test]
    fn test_whitespace_normalization() {
        let cmp = OutputComparator::new().whitespace_normalize(true);

        let result = cmp.compare_str("hello  \nworld\r\n", "hello\nworld");
        assert!(
            matches!(result, CompareResult::Equal),
            "Whitespace normalized strings should be equal"
        );
    }

    #[test]
    fn test_unicode_normalization() {
        let cmp = OutputComparator::new().unicode_normalize(true);

        // e with combining accent vs precomposed e
        let result = cmp.compare_str("cafe\u{0301}", "caf\u{00e9}");
        assert!(
            matches!(result, CompareResult::Equal),
            "NFC normalized unicode should be equal"
        );
    }

    #[test]
    fn test_float_epsilon_pass() {
        let cmp = OutputComparator::new();

        let result = cmp.compare_f64(1.0, 1.0005, 0.001);
        match result {
            CompareResult::ApproximatelyEqual { delta, epsilon, .. } => {
                assert!(delta < epsilon);
            }
            CompareResult::Equal => {
                // Also acceptable if exactly equal
            }
            _ => panic!("Should be approximately equal"),
        }
    }

    #[test]
    fn test_float_epsilon_fail() {
        let cmp = OutputComparator::new();

        let result = cmp.compare_f64(1.0, 1.01, 0.001);
        assert!(matches!(result, CompareResult::Different(_)));
    }

    #[test]
    fn test_float_nan() {
        let cmp = OutputComparator::new();

        // NaN == NaN for testing
        let result = cmp.compare_f64(f64::NAN, f64::NAN, 0.001);
        assert!(matches!(result, CompareResult::Equal));
    }

    #[test]
    fn test_float_infinity() {
        let cmp = OutputComparator::new();

        let result = cmp.compare_f64(f64::INFINITY, f64::INFINITY, 0.001);
        assert!(matches!(result, CompareResult::Equal));

        let result = cmp.compare_f64(f64::NEG_INFINITY, f64::NEG_INFINITY, 0.001);
        assert!(matches!(result, CompareResult::Equal));

        let result = cmp.compare_f64(f64::INFINITY, f64::NEG_INFINITY, 0.001);
        assert!(matches!(result, CompareResult::Different(_)));
    }

    #[test]
    fn test_multiline_unified_diff() {
        let cmp = OutputComparator::new();

        let expected = "line 1\nline 2\nline 3";
        let actual = "line 1\nmodified\nline 3";

        let result = cmp.compare_lines(expected, actual);
        match result {
            CompareResult::Different(diff) => {
                assert!(diff.unified_diff.contains("-line 2"));
                assert!(diff.unified_diff.contains("+modified"));
                assert_eq!(diff.first_diff_line, Some(2));
            }
            _ => panic!("Should be different"),
        }
    }

    #[test]
    fn test_empty_strings() {
        let cmp = OutputComparator::new();

        assert!(matches!(cmp.compare_str("", ""), CompareResult::Equal));
        assert!(matches!(
            cmp.compare_str("", "x"),
            CompareResult::Different(_)
        ));
    }

    #[test]
    fn test_length_diff_reported() {
        let cmp = OutputComparator::new();

        let result = cmp.compare_str("hello", "hello world");
        match result {
            CompareResult::Different(diff) => {
                assert_eq!(diff.diff_type, DiffType::LengthDiff);
            }
            _ => panic!("Should be different"),
        }
    }

    #[test]
    fn test_debug_comparison() {
        #[derive(Debug)]
        struct Point {
            x: i32,
            y: i32,
        }

        let cmp = OutputComparator::new();
        let result = cmp.compare_debug(&Point { x: 1, y: 2 }, &Point { x: 1, y: 3 });

        match result {
            CompareResult::Different(diff) => {
                assert!(diff.inline_diff.contains("y: 2") || diff.expected.contains("y: 2"));
                assert_eq!(diff.diff_type, DiffType::TypeDiff);
            }
            _ => panic!("Should be different"),
        }
    }

    #[test]
    fn test_case_insensitive() {
        let cmp = OutputComparator::new().ignore_case(true);

        assert!(matches!(
            cmp.compare_str("Hello", "hello"),
            CompareResult::Equal
        ));
        assert!(matches!(
            cmp.compare_str("WORLD", "world"),
            CompareResult::Equal
        ));
    }

    #[test]
    fn test_bytes_comparison() {
        let cmp = OutputComparator::new();

        assert!(matches!(
            cmp.compare_bytes(b"hello", b"hello"),
            CompareResult::Equal
        ));

        let result = cmp.compare_bytes(b"hello", b"hallo");
        match result {
            CompareResult::Different(diff) => {
                assert_eq!(diff.first_diff_pos, Some(1));
            }
            _ => panic!("Should be different"),
        }
    }

    #[test]
    fn test_diff_description() {
        let diff = Diff {
            expected: "hello".to_string(),
            actual: "hallo".to_string(),
            first_diff_pos: Some(1),
            first_diff_line: None,
            inline_diff: "At position 1: expected 'e', got 'a'".to_string(),
            unified_diff: String::new(),
            diff_type: DiffType::CharacterDiff,
        };

        let desc = diff.describe();
        assert!(desc.contains("position 1"));
    }

    #[test]
    fn test_normalize_ansi_function() {
        // Test the internal normalization function
        let input = "\x1b[31;1mHello\x1b[0m";
        let normalized = normalize_ansi(input);
        assert!(normalized.contains("1;31")); // Sorted order

        let input2 = "\x1b[1;31mHello\x1b[0m";
        let normalized2 = normalize_ansi(input2);
        assert_eq!(normalized, normalized2);
    }
}
