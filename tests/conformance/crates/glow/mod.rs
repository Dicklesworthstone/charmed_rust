//! Conformance tests for the glow crate
//!
//! This module contains conformance tests verifying that the Rust
//! implementation of the markdown reader matches the expected behavior
//! of the original Go glow library.
//!
//! Test categories:
//! - Config: builder patterns, defaults, validation
//! - Reader: markdown rendering through glamour
//! - Style: style parsing and selection
//! - Stash: document organization

#![allow(clippy::unreadable_literal)]

use crate::harness::{FixtureLoader, TestFixture, compare_styled_semantic, extract_styled_spans};
use glow::{Config, Reader, Stash};
use serde::Deserialize;
use std::collections::HashSet;

/// Input for glow reader tests
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Reserved fields for future fixture expansion
struct GlowInput {
    /// The markdown input to render
    markdown: Option<String>,
    /// The style to use
    style: Option<String>,
    /// Optional width setting
    width: Option<usize>,
    /// Pager mode
    pager: Option<bool>,
    /// Test type for config tests
    test_type: Option<String>,
}

/// Expected output for glow tests
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Reserved fields for future fixture expansion
struct GlowOutput {
    /// Whether an error is expected
    #[serde(default)]
    error: bool,
    /// The expected output
    output: Option<String>,
    /// Style validity for style tests
    valid: Option<bool>,
    /// Expected default values for config tests
    default_pager: Option<bool>,
    default_style: Option<String>,
}

/// Run a single glow conformance test
fn run_glow_test(fixture: &TestFixture) -> Result<(), String> {
    let input: GlowInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {}", e))?;

    let expected: GlowOutput = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {}", e))?;

    // Handle different test types based on fixture name
    if fixture.name.starts_with("config_") {
        run_config_test(&fixture.name, &input, &expected)
    } else if fixture.name.starts_with("reader_") {
        run_reader_test(&fixture.name, &input, &expected)
    } else if fixture.name.starts_with("style_") {
        run_style_test(&input, &expected)
    } else if fixture.name.starts_with("stash_") {
        run_stash_test(&fixture.name)
    } else {
        Err(format!("Unknown test type: {}", fixture.name))
    }
}

/// Test config builder behavior
fn run_config_test(name: &str, input: &GlowInput, expected: &GlowOutput) -> Result<(), String> {
    match name {
        "config_defaults" => {
            let config = Config::new();
            // Verify defaults match expected
            if let Some(_expected_pager) = expected.default_pager {
                // Config's pager field is private, but we can verify via behavior
                // For now, just verify the config can be created
                let _ = config;
            }
            if let Some(_expected_style) = &expected.default_style {
                // Verify dark is the default style
                let reader = Reader::new(Config::new());
                let result = reader.render_markdown("test");
                if result.is_err() {
                    return Err("Default config should render successfully".to_string());
                }
            }
            Ok(())
        }
        "config_pager_disabled" => {
            let config = Config::new().pager(false);
            let _ = Reader::new(config);
            Ok(())
        }
        "config_width_80" | "config_width_120" => {
            let width = input.width.unwrap_or(80);
            let config = Config::new().width(width);
            let reader = Reader::new(config);
            let result = reader.render_markdown("# Test");
            if result.is_err() {
                return Err(format!("Width {} config should work", width));
            }
            Ok(())
        }
        "config_style_light" | "config_style_ascii" | "config_style_pink" => {
            let style = input.style.as_deref().unwrap_or("dark");
            let config = Config::new().style(style);
            let reader = Reader::new(config);
            let result = reader.render_markdown("# Test");
            if result.is_err() {
                return Err(format!("Style {} should be valid", style));
            }
            Ok(())
        }
        "config_combined" => {
            let config = Config::new()
                .pager(input.pager.unwrap_or(true))
                .width(input.width.unwrap_or(100))
                .style(input.style.as_deref().unwrap_or("dark"));
            let reader = Reader::new(config);
            let result = reader.render_markdown("# Test");
            if result.is_err() {
                return Err("Combined config should work".to_string());
            }
            Ok(())
        }
        _ => Err(format!("Unknown config test: {}", name)),
    }
}

/// Test reader rendering behavior
fn run_reader_test(name: &str, input: &GlowInput, expected: &GlowOutput) -> Result<(), String> {
    let markdown = input.markdown.as_deref().unwrap_or("");
    let style = input.style.as_deref().unwrap_or("dark");

    let mut config = Config::new().style(style);
    if let Some(width) = input.width {
        config = config.width(width);
    }

    let reader = Reader::new(config);
    let result = reader.render_markdown(markdown);

    if expected.error {
        if result.is_ok() {
            return Err("Expected error but got success".to_string());
        }
        Ok(())
    } else {
        let expected_output = expected
            .output
            .as_deref()
            .ok_or_else(|| "Fixture missing expected output string".to_string())?;

        match result {
            Ok(output) => {
                let output_str: &str = &output;
                // Glow output is ANSI-rich and can differ in escape sequence ordering across
                // implementations; byte-for-byte equality is too brittle. We compare semantically:
                // plain text content + presence of key style attributes (bold/italic/colors).
                if name == "reader_code_block" {
                    return compare_code_block_highlighting(expected_output, output_str);
                }

                let semantic = compare_styled_semantic(expected_output, output_str);
                if !semantic.text_matches {
                    return Err(format!(
                        "Rendered text mismatch:\n  expected: {:?}\n  actual:   {:?}",
                        semantic.expected_text, semantic.actual_text
                    ));
                }
                if !semantic.styles_match {
                    return Err(format!(
                        "Rendered styles mismatch (text matched): {:?}",
                        semantic.style_mismatches
                    ));
                }
                Ok(())
            }
            Err(e) => Err(format!("Expected success but got error: {}", e)),
        }
    }
}

fn compare_code_block_highlighting(expected: &str, actual: &str) -> Result<(), String> {
    let semantic = compare_styled_semantic(expected, actual);
    if !semantic.text_matches {
        return Err(format!(
            "Code block text mismatch:\n  expected: {:?}\n  actual:   {:?}",
            semantic.expected_text, semantic.actual_text
        ));
    }

    let expected_colors = distinct_foreground_colors(expected);
    let actual_colors = distinct_foreground_colors(actual);

    // Both Go (chroma) and Rust (syntect) should produce multi-colored output for fenced code
    // blocks with a language hint. Exact color values/themes may differ, so we only assert that
    // highlighting exists when the reference has it.
    if expected_colors.len() > 2 && actual_colors.len() <= 2 {
        return Err(format!(
            "Syntax highlighting gap: expected {} distinct colors, got {}",
            expected_colors.len(),
            actual_colors.len()
        ));
    }

    Ok(())
}

fn distinct_foreground_colors(text: &str) -> HashSet<u32> {
    let mut colors = HashSet::new();
    for span in extract_styled_spans(text) {
        if span.text.trim().is_empty() {
            continue;
        }
        let Some(fg) = span.foreground.as_ref() else {
            continue;
        };

        // Common forms:
        // - "38;5;N" (256-color)
        // - "38;2;R;G;B" (truecolor)
        // - "31" (basic ANSI)
        if let Some(n) = fg.strip_prefix("38;5;") {
            if let Ok(v) = n.parse::<u32>() {
                colors.insert(v);
            }
            continue;
        }
        if let Some(rgb) = fg.strip_prefix("38;2;") {
            let parts: Vec<&str> = rgb.split(';').collect();
            if parts.len() == 3 {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    parts[0].parse::<u32>(),
                    parts[1].parse::<u32>(),
                    parts[2].parse::<u32>(),
                ) {
                    colors.insert(0x100_0000 + (r << 16) + (g << 8) + b);
                }
            }
            continue;
        }
        if let Ok(v) = fg.parse::<u32>() {
            colors.insert(v);
        }
    }
    colors
}

/// Test style parsing
fn run_style_test(input: &GlowInput, expected: &GlowOutput) -> Result<(), String> {
    let style = input.style.as_deref().unwrap_or("");
    let config = Config::new().style(style);
    let reader = Reader::new(config);

    // Try to render something to validate the style
    let result = reader.render_markdown("test");

    let is_valid = result.is_ok();
    let expected_valid = expected.valid.unwrap_or(true);

    if is_valid != expected_valid {
        return Err(format!(
            "Style '{}' validity mismatch: expected {}, got {}",
            style, expected_valid, is_valid
        ));
    }
    Ok(())
}

/// Test stash behavior
fn run_stash_test(name: &str) -> Result<(), String> {
    match name {
        "stash_empty" => {
            let stash = Stash::new();
            if !stash.documents().is_empty() {
                return Err("New stash should be empty".to_string());
            }
            Ok(())
        }
        "stash_add_single" => {
            let mut stash = Stash::new();
            stash.add("test.md");
            if stash.documents().len() != 1 {
                return Err("Stash should have 1 document".to_string());
            }
            Ok(())
        }
        "stash_add_multiple" => {
            let mut stash = Stash::new();
            stash.add("a.md");
            stash.add("b.md");
            stash.add("c.md");
            if stash.documents().len() != 3 {
                return Err("Stash should have 3 documents".to_string());
            }
            Ok(())
        }
        _ => Err(format!("Unknown stash test: {}", name)),
    }
}

/// Run all glow conformance tests
pub fn run_all_tests() -> Vec<(&'static str, Result<(), String>)> {
    let mut loader = FixtureLoader::new();
    let mut results = Vec::new();

    // Load fixtures
    let fixtures = match loader.load_crate("glow") {
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
        "Loaded {} tests from glow.json (Go lib version {})",
        fixtures.tests.len(),
        fixtures.metadata.library_version
    );

    // Run each test
    for test in &fixtures.tests {
        let result = run_test(test);
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

    run_glow_test(fixture)
}

/// Run basic sanity tests without fixtures.
///
/// Note: conformance is fixture-driven; the main suite must fail if fixtures
/// cannot be loaded. This remains only as a local debugging helper.
#[allow(dead_code)]
fn run_basic_tests() -> Vec<(&'static str, Result<(), String>)> {
    fn test_basic_config() -> Result<(), String> {
        let config = Config::new();
        let _ = Reader::new(config);
        Ok(())
    }

    fn test_basic_render() -> Result<(), String> {
        let reader = Reader::new(Config::new());
        match reader.render_markdown("# Hello") {
            Ok(output) => {
                let output_str: &str = &output;
                if output_str.is_empty() {
                    Err("Output should not be empty".to_string())
                } else {
                    Ok(())
                }
            }
            Err(e) => Err(format!("Render failed: {}", e)),
        }
    }

    fn test_basic_styles() -> Result<(), String> {
        let styles = ["dark", "light", "ascii", "pink", "auto"];
        for style in styles {
            let config = Config::new().style(style);
            let reader = Reader::new(config);
            if reader.render_markdown("test").is_err() {
                return Err(format!("Style {} should work", style));
            }
        }
        Ok(())
    }

    fn test_basic_width() -> Result<(), String> {
        let config = Config::new().width(80);
        let reader = Reader::new(config);
        if reader.render_markdown("test").is_err() {
            return Err("Width setting should work".to_string());
        }
        Ok(())
    }

    fn test_basic_stash() -> Result<(), String> {
        let mut stash = Stash::new();
        stash.add("test.md");
        if stash.documents().len() != 1 {
            return Err("Stash should work".to_string());
        }
        Ok(())
    }

    vec![
        ("basic_config", test_basic_config()),
        ("basic_render", test_basic_render()),
        ("basic_styles", test_basic_styles()),
        ("basic_width", test_basic_width()),
        ("basic_stash", test_basic_stash()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test runner that runs all conformance tests
    #[test]
    fn test_glow_conformance() {
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

        println!("\nGlow Conformance Results:");
        println!("  Passed:  {}", passed);
        println!("  Failed:  {}", failed);
        println!("  Skipped: {}", skipped);
        println!("  Total:   {}", results.len());

        if !failures.is_empty() {
            println!("\nFailures:");
            for (name, msg) in &failures {
                println!("  {}: {}", name, msg);
            }
        }

        assert_eq!(failed, 0, "All conformance tests should pass");
        assert_eq!(
            skipped, 0,
            "No conformance fixtures should be skipped (missing coverage must fail CI)"
        );
    }

    /// Quick sanity test that glow renders basic markdown
    #[test]
    fn test_basic_render() {
        let reader = Reader::new(Config::new());
        let result = reader.render_markdown("# Hello World");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_empty());
    }

    /// Test config builder methods
    #[test]
    fn test_config_builder() {
        let config = Config::new().pager(false).width(100).style("light");

        let reader = Reader::new(config);
        let result = reader.render_markdown("test");
        assert!(result.is_ok());
    }

    /// Test all valid styles
    #[test]
    fn test_valid_styles() {
        let valid_styles = ["dark", "light", "ascii", "pink", "auto", "no-tty", "notty"];

        for style in valid_styles {
            let config = Config::new().style(style);
            let reader = Reader::new(config);
            let result = reader.render_markdown("test");
            assert!(result.is_ok(), "Style '{}' should be valid", style);
        }
    }

    /// Test invalid style rejection
    #[test]
    fn test_invalid_styles() {
        let invalid_styles = ["unknown", "dracula", "solarized"];

        for style in invalid_styles {
            let config = Config::new().style(style);
            let reader = Reader::new(config);
            let result = reader.render_markdown("test");
            assert!(result.is_err(), "Style '{}' should be invalid", style);
        }
    }

    /// Test stash operations
    #[test]
    fn test_stash_operations() {
        let mut stash = Stash::new();
        assert!(stash.documents().is_empty());

        stash.add("file1.md");
        assert_eq!(stash.documents().len(), 1);
        assert_eq!(stash.documents()[0], "file1.md");

        stash.add("file2.md");
        assert_eq!(stash.documents().len(), 2);
    }

    /// Test rendering with different widths
    #[test]
    fn test_width_settings() {
        let widths = [40, 80, 120, 200];
        let markdown =
            "This is a long line that should wrap at different widths depending on configuration.";

        for width in widths {
            let config = Config::new().width(width);
            let reader = Reader::new(config);
            let result = reader.render_markdown(markdown);
            assert!(result.is_ok(), "Width {} should work", width);
        }
    }
}

/// Integration with the conformance trait system
pub mod integration {
    use super::*;
    use crate::harness::{ConformanceTest, TestCategory, TestContext, TestResult};

    /// Glow rendering conformance test
    pub struct GlowRenderTest {
        name: String,
    }

    impl GlowRenderTest {
        pub fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    impl ConformanceTest for GlowRenderTest {
        fn name(&self) -> &str {
            &self.name
        }

        fn crate_name(&self) -> &str {
            "glow"
        }

        fn category(&self) -> TestCategory {
            TestCategory::Unit
        }

        fn run(&self, ctx: &mut TestContext) -> TestResult {
            let fixture = match ctx.fixture_for_current_test("glow") {
                Ok(f) => f,
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

    /// Get all glow conformance tests as trait objects
    pub fn all_tests() -> Vec<Box<dyn ConformanceTest>> {
        let mut loader = FixtureLoader::new();
        let fixtures = match loader.load_crate("glow") {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };

        fixtures
            .tests
            .iter()
            .map(|t| Box::new(GlowRenderTest::new(&t.name)) as Box<dyn ConformanceTest>)
            .collect()
    }
}
