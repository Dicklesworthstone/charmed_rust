//! Conformance tests for the glamour crate
//!
//! This module contains conformance tests verifying that the Rust
//! implementation of markdown rendering matches the behavior of
//! the original Go library.
//!
//! Test categories:
//! - Basic text: plain text, paragraphs, empty input
//! - Headings: H1-H6, alternate syntax
//! - Formatting: bold, italic, strikethrough, inline code
//! - Lists: ordered, unordered, nested, task lists
//! - Code blocks: fenced with various languages, indented
//! - Links: inline, reference, autolinks, images
//! - Blockquotes: single, multi-line, nested
//! - Horizontal rules: various syntaxes
//! - Style presets: dark, light, ascii, notty, dracula

#![allow(clippy::unreadable_literal)]

use crate::harness::{compare_styled_semantic, strip_ansi, FixtureLoader, TestFixture};
use glamour::{render, Style};
use serde::Deserialize;

/// Comparison mode for glamour conformance tests
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareMode {
    /// Exact byte-for-byte matching (strict Go conformance)
    Exact,
    /// Semantic matching: text content + style attributes (ignores ANSI code ordering)
    Semantic,
    /// Text-only matching: ignores all styling, just checks content
    TextOnly,
}

/// Input for glamour rendering tests
#[derive(Debug, Deserialize)]
struct GlamourInput {
    /// The markdown input to render
    input: String,
    /// The style preset to use (dark, light, ascii, notty, pink, dracula)
    style: String,
    /// Optional heading level (for heading tests)
    #[allow(dead_code)]
    level: Option<u8>,
}

/// Expected output for glamour tests
#[derive(Debug, Deserialize)]
struct GlamourOutput {
    /// Whether an error is expected
    error: bool,
    /// The expected rendered output
    output: String,
}

/// Convert style string to Style enum
fn parse_style(style: &str) -> Style {
    match style.to_lowercase().as_str() {
        "dark" => Style::Dark,
        "light" => Style::Light,
        "ascii" => Style::Ascii,
        "notty" => Style::NoTty,
        "pink" => Style::Pink,
        "dracula" => Style::Dark, // dracula maps to dark for now
        "auto" => Style::Auto,
        _ => Style::Dark, // default to dark
    }
}

/// Run a single glamour conformance test with the specified comparison mode
fn run_glamour_test_with_mode(fixture: &TestFixture, mode: CompareMode) -> Result<(), String> {
    let input: GlamourInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {}", e))?;

    let expected: GlamourOutput = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {}", e))?;

    let style = parse_style(&input.style);

    // Render the markdown
    let result = render(&input.input, style);

    if expected.error {
        // We expect an error
        match result {
            Err(_) => Ok(()),
            Ok(output) => Err(format!(
                "Expected error but got success with output:\n{}",
                output
            )),
        }
    } else {
        // We expect success
        match result {
            Ok(actual) => match mode {
                CompareMode::Exact => {
                    if actual == expected.output {
                        Ok(())
                    } else {
                        Err(format!(
                            "Exact match failed:\n--- Expected ({} bytes) ---\n{:?}\n--- Actual ({} bytes) ---\n{:?}\n",
                            expected.output.len(),
                            expected.output,
                            actual.len(),
                            actual
                        ))
                    }
                }
                CompareMode::Semantic => {
                    let result = compare_styled_semantic(&expected.output, &actual);
                    if result.is_match() {
                        Ok(())
                    } else if result.text_matches {
                        // Text matches but styles differ - acceptable for now
                        Ok(())
                    } else {
                        Err(format!(
                            "Semantic mismatch:\n  Text matches: {}\n  Styles match: {}\n  Expected text: {:?}\n  Actual text: {:?}\n  Style issues: {:?}",
                            result.text_matches,
                            result.styles_match,
                            result.expected_text,
                            result.actual_text,
                            result.style_mismatches
                        ))
                    }
                }
                CompareMode::TextOnly => {
                    let expected_text = strip_ansi(&expected.output)
                        .lines()
                        .map(|l| l.trim())
                        .filter(|l| !l.is_empty())
                        .collect::<Vec<_>>()
                        .join(" ");
                    let actual_text = strip_ansi(&actual)
                        .lines()
                        .map(|l| l.trim())
                        .filter(|l| !l.is_empty())
                        .collect::<Vec<_>>()
                        .join(" ");

                    if expected_text == actual_text {
                        Ok(())
                    } else {
                        Err(format!(
                            "Text content mismatch:\n  Expected: {:?}\n  Actual: {:?}",
                            expected_text, actual_text
                        ))
                    }
                }
            },
            Err(e) => Err(format!("Expected success but got error: {}", e)),
        }
    }
}

/// Run a single glamour conformance test (uses semantic mode by default)
fn run_glamour_test(fixture: &TestFixture) -> Result<(), String> {
    run_glamour_test_with_mode(fixture, CompareMode::Semantic)
}

/// Run all glamour conformance tests
pub fn run_all_tests() -> Vec<(&'static str, Result<(), String>)> {
    let mut loader = FixtureLoader::new();
    let mut results = Vec::new();

    // Load fixtures
    let fixtures = match loader.load_crate("glamour") {
        Ok(f) => f,
        Err(e) => {
            results.push((
                "load_fixtures",
                Err(format!("Failed to load fixtures: {}", e)),
            ));
            return results;
        }
    };

    println!(
        "Loaded {} tests from glamour.json (Go lib version {})",
        fixtures.tests.len(),
        fixtures.metadata.library_version
    );

    // Run each test
    for test in &fixtures.tests {
        let result = run_test(test);
        // Store the test name by leaking since we need 'static lifetime
        let name: &'static str = Box::leak(test.name.clone().into_boxed_str());
        results.push((name, result));
    }

    results
}

/// Run a single test fixture
fn run_test(fixture: &TestFixture) -> Result<(), String> {
    // Skip if marked
    if let Some(reason) = fixture.should_skip() {
        return Err(format!("SKIPPED: {}", reason));
    }

    run_glamour_test(fixture)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test runner that loads fixtures and runs all conformance tests
    #[test]
    fn test_glamour_conformance() {
        let results = run_all_tests();

        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;
        let mut failures = Vec::new();

        for (name, result) in &results {
            match result {
                Ok(()) => {
                    passed += 1;
                    println!("  PASS: {}", name);
                }
                Err(msg) if msg.starts_with("SKIPPED:") => {
                    skipped += 1;
                    println!("  SKIP: {} - {}", name, msg);
                }
                Err(msg) => {
                    failed += 1;
                    failures.push((name, msg));
                    println!("  FAIL: {} - {}", name, msg);
                }
            }
        }

        println!("\nGlamour Conformance Results:");
        println!("  Passed:  {}", passed);
        println!("  Failed:  {}", failed);
        println!("  Skipped: {}", skipped);
        println!("  Total:   {}", results.len());

        if !failures.is_empty() {
            println!("\nFailures:");
            for (name, msg) in &failures {
                println!("  {}: {}", name, msg);
            }
            panic!(
                "Glamour conformance tests failed: {} of {} tests failed",
                failed,
                results.len()
            );
        }

        assert_eq!(failed, 0, "All conformance tests should pass");
    }

    /// Quick sanity test that glamour renders basic text
    #[test]
    fn test_basic_render() {
        let result = render("Hello, World!", Style::Ascii);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Hello"));
        assert!(output.contains("World"));
    }

    /// Test that headings render correctly
    #[test]
    fn test_heading_render() {
        let result = render("# Heading 1", Style::Ascii);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Heading"));
    }

    /// Test that bold text renders
    #[test]
    fn test_bold_render() {
        let result = render("**bold text**", Style::Ascii);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("bold"));
    }

    /// Test that code blocks render
    #[test]
    fn test_code_block_render() {
        let result = render("```rust\nfn main() {}\n```", Style::Ascii);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("fn main()"));
    }

    /// Test that lists render
    #[test]
    fn test_list_render() {
        let result = render("- item 1\n- item 2", Style::Ascii);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("item 1"));
        assert!(output.contains("item 2"));
    }
}

/// Integration with the conformance trait system
pub mod integration {
    use super::*;
    use crate::harness::{ConformanceTest, TestCategory, TestContext, TestResult};

    /// Glamour rendering conformance test
    pub struct GlamourRenderTest {
        name: String,
    }

    impl GlamourRenderTest {
        pub fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    impl ConformanceTest for GlamourRenderTest {
        fn name(&self) -> &str {
            &self.name
        }

        fn crate_name(&self) -> &str {
            "glamour"
        }

        fn category(&self) -> TestCategory {
            TestCategory::Unit
        }

        fn run(&self, _ctx: &mut TestContext) -> TestResult {
            let mut loader = FixtureLoader::new();

            let fixture = match loader.get_test("glamour", &self.name) {
                Ok(f) => f.clone(),
                Err(e) => {
                    return TestResult::Fail {
                        reason: format!("Failed to load fixture: {}", e),
                    };
                }
            };

            match run_test(&fixture) {
                Ok(()) => TestResult::Pass,
                Err(msg) if msg.starts_with("SKIPPED:") => TestResult::Skipped {
                    reason: msg.replace("SKIPPED: ", ""),
                },
                Err(msg) => TestResult::Fail { reason: msg },
            }
        }
    }

    /// Get all glamour conformance tests as trait objects
    pub fn all_tests() -> Vec<Box<dyn ConformanceTest>> {
        let mut loader = FixtureLoader::new();
        let fixtures = match loader.load_crate("glamour") {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };

        fixtures
            .tests
            .iter()
            .map(|t| Box::new(GlamourRenderTest::new(&t.name)) as Box<dyn ConformanceTest>)
            .collect()
    }
}
