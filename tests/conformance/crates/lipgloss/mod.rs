//! Conformance tests for the lipgloss crate
//!
//! This module contains conformance tests verifying that the Rust
//! implementation of terminal styling matches the behavior of the
//! original Go library.
//!
//! Test categories:
//! - Styles: text attributes (bold, italic, underline, etc.)
//! - Colors: foreground/background color rendering
//! - Borders: different border styles
//! - Padding: spacing inside borders
//! - Margin: spacing outside borders
//! - Dimensions: width/height constraints
//! - Alignment: horizontal and vertical text alignment
//! - Joins: horizontal and vertical block joining
//! - Place: positioning within containers

use crate::harness::{FixtureLoader, TestFixture};
use lipgloss::{Border, Position, Style, join_horizontal, join_vertical, place};
use serde::Deserialize;

/// Input for style tests (text attributes)
#[derive(Debug, Deserialize, Default)]
struct StyleInput {
    #[serde(default)]
    bold: bool,
    #[serde(default)]
    italic: bool,
    #[serde(default)]
    underline: bool,
    #[serde(default)]
    strikethrough: bool,
    #[serde(default)]
    faint: bool,
    #[serde(default)]
    blink: bool,
    #[serde(default)]
    reverse: bool,
    text: String,
    #[serde(default)]
    foreground: Option<String>,
    #[serde(default)]
    background: Option<String>,
    #[serde(default)]
    padding: Option<Vec<u16>>,
    #[serde(default)]
    margin: Option<Vec<u16>>,
    #[serde(default)]
    width: Option<u16>,
    #[serde(default)]
    height: Option<u16>,
    #[serde(default)]
    max_width: Option<u16>,
    #[serde(default)]
    max_height: Option<u16>,
    #[serde(default)]
    align_horizontal: Option<String>,
    #[serde(default)]
    align_vertical: Option<String>,
}

/// Expected output for style tests
#[derive(Debug, Deserialize)]
struct StyleOutput {
    rendered: String,
    #[serde(default)]
    width: Option<usize>,
    #[serde(default)]
    height: Option<usize>,
}

/// Input for border tests
#[derive(Debug, Deserialize)]
struct BorderInput {
    border_type: String,
    text: String,
    #[serde(default)]
    foreground: Option<String>,
}

/// Expected output for border tests
#[derive(Debug, Deserialize)]
struct BorderOutput {
    rendered: String,
}

/// Input for join tests
#[derive(Debug, Deserialize)]
struct JoinInput {
    blocks: Vec<String>,
    position: String,
}

/// Expected output for join tests
#[derive(Debug, Deserialize)]
struct JoinOutput {
    result: String,
}

/// Input for place tests (single position)
#[derive(Debug, Deserialize)]
struct PlaceInput {
    text: String,
    #[serde(default)]
    width: Option<u16>,
    #[serde(default)]
    height: Option<u16>,
    #[serde(default)]
    position: Option<String>,
    #[serde(default)]
    horizontal_pos: Option<String>,
    #[serde(default)]
    vertical_pos: Option<String>,
}

/// Expected output for place tests
#[derive(Debug, Deserialize)]
struct PlaceOutput {
    result: String,
}

/// Parse position string to Position enum
fn parse_position(pos: &str) -> Position {
    match pos.to_lowercase().as_str() {
        "left" | "top" => Position::Left, // Top = 0.0
        "center" => Position::Center,
        "right" | "bottom" => Position::Right, // Bottom = 1.0
        _ => Position::Left,
    }
}

/// Get border by type name
fn get_border(border_type: &str) -> Border {
    match border_type.to_lowercase().as_str() {
        "normal" => Border::normal(),
        "rounded" => Border::rounded(),
        "double" => Border::double(),
        "thick" => Border::thick(),
        "block" => Border::block(),
        "hidden" => Border::hidden(),
        "ascii" => Border::ascii(),
        _ => Border::normal(),
    }
}

/// Strip ANSI escape codes from string for visual comparison
fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_escape = false;

    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
            continue;
        }
        if in_escape {
            if c == 'm' {
                in_escape = false;
            }
            continue;
        }
        result.push(c);
    }

    result
}

/// Run a style test (text attributes, colors, padding, margin, dimensions, alignment)
fn run_style_test(fixture: &TestFixture) -> Result<(), String> {
    let input: StyleInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {}", e))?;

    let expected: StyleOutput = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {}", e))?;

    let mut style = Style::new();

    // Text attributes
    if input.bold {
        style = style.bold();
    }
    if input.italic {
        style = style.italic();
    }
    if input.underline {
        style = style.underline();
    }
    if input.strikethrough {
        style = style.strikethrough();
    }
    if input.faint {
        style = style.faint();
    }
    if input.blink {
        style = style.blink();
    }
    if input.reverse {
        style = style.reverse();
    }

    // Colors
    if let Some(ref fg) = input.foreground {
        style = style.foreground(fg.as_str());
    }
    if let Some(ref bg) = input.background {
        style = style.background(bg.as_str());
    }

    // Padding (top, right, bottom, left)
    if let Some(ref padding) = input.padding {
        match padding.len() {
            1 => style = style.padding(padding[0]),
            2 => style = style.padding((padding[0], padding[1])),
            4 => style = style.padding((padding[0], padding[1], padding[2], padding[3])),
            _ => {}
        }
    }

    // Margin (top, right, bottom, left)
    if let Some(ref margin) = input.margin {
        match margin.len() {
            1 => style = style.margin(margin[0]),
            2 => style = style.margin((margin[0], margin[1])),
            4 => style = style.margin((margin[0], margin[1], margin[2], margin[3])),
            _ => {}
        }
    }

    // Dimensions
    if let Some(w) = input.width {
        style = style.width(w);
    }
    if let Some(h) = input.height {
        style = style.height(h);
    }
    if let Some(mw) = input.max_width {
        style = style.max_width(mw);
    }
    if let Some(mh) = input.max_height {
        style = style.max_height(mh);
    }

    // Alignment
    if let Some(ref align_h) = input.align_horizontal {
        style = style.align(parse_position(align_h));
    }
    if let Some(ref align_v) = input.align_vertical {
        style = style.align_vertical(parse_position(align_v));
    }

    let rendered = style.render(&input.text);

    // Compare visual content (stripped of ANSI codes) since the test fixtures
    // don't include ANSI escape sequences in expected output
    let actual_stripped = strip_ansi(&rendered);
    if actual_stripped != expected.rendered {
        return Err(format!(
            "Rendered output mismatch:\n  expected: {:?}\n  actual:   {:?}",
            expected.rendered, actual_stripped
        ));
    }

    // Check dimensions if provided
    if let Some(expected_width) = expected.width {
        let actual_width = lipgloss::width(&rendered);
        if actual_width != expected_width {
            return Err(format!(
                "Width mismatch: expected {}, got {}",
                expected_width, actual_width
            ));
        }
    }

    if let Some(expected_height) = expected.height {
        let actual_height = lipgloss::height(&rendered);
        if actual_height != expected_height {
            return Err(format!(
                "Height mismatch: expected {}, got {}",
                expected_height, actual_height
            ));
        }
    }

    Ok(())
}

/// Run a border test
fn run_border_test(fixture: &TestFixture) -> Result<(), String> {
    let input: BorderInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {}", e))?;

    let expected: BorderOutput = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {}", e))?;

    let border = get_border(&input.border_type);
    let mut style = Style::new().border(border);

    // Apply foreground color if specified
    if let Some(ref fg) = input.foreground {
        style = style.foreground(fg.as_str());
    }

    let rendered = style.render(&input.text);

    // Compare visual content (stripped of ANSI codes)
    let actual_stripped = strip_ansi(&rendered);
    if actual_stripped != expected.rendered {
        return Err(format!(
            "Border rendered output mismatch:\n  expected: {:?}\n  actual:   {:?}",
            expected.rendered, actual_stripped
        ));
    }

    Ok(())
}

/// Run a join test (horizontal or vertical)
fn run_join_test(fixture: &TestFixture, is_horizontal: bool) -> Result<(), String> {
    let input: JoinInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {}", e))?;

    let expected: JoinOutput = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {}", e))?;

    let position = parse_position(&input.position);
    let blocks: Vec<&str> = input.blocks.iter().map(|s| s.as_str()).collect();

    let result = if is_horizontal {
        join_horizontal(position, &blocks)
    } else {
        join_vertical(position, &blocks)
    };

    if result != expected.result {
        return Err(format!(
            "Join result mismatch:\n  expected: {:?}\n  actual:   {:?}",
            expected.result, result
        ));
    }

    Ok(())
}

/// Run a place test
fn run_place_test(fixture: &TestFixture) -> Result<(), String> {
    let input: PlaceInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {}", e))?;

    let expected: PlaceOutput = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {}", e))?;

    // Determine positions - use specific or combined position
    let h_pos = input
        .horizontal_pos
        .as_ref()
        .map(|p| parse_position(p))
        .or_else(|| input.position.as_ref().map(|p| parse_position(p)))
        .unwrap_or(Position::Left);

    let v_pos = input
        .vertical_pos
        .as_ref()
        .map(|p| parse_position(p))
        .or_else(|| input.position.as_ref().map(|p| parse_position(p)))
        .unwrap_or(Position::Top);

    let width = input
        .width
        .map(|w| w as usize)
        .unwrap_or_else(|| lipgloss::width(&input.text));
    let height = input
        .height
        .map(|h| h as usize)
        .unwrap_or_else(|| lipgloss::height(&input.text));

    let result = place(width, height, h_pos, v_pos, &input.text);

    if result != expected.result {
        return Err(format!(
            "Place result mismatch:\n  expected: {:?}\n  actual:   {:?}",
            expected.result, result
        ));
    }

    Ok(())
}

/// Run a single test fixture
fn run_test(fixture: &TestFixture) -> Result<(), String> {
    // Skip if marked
    if let Some(reason) = fixture.should_skip() {
        return Err(format!("SKIPPED: {}", reason));
    }

    // Route to appropriate test runner based on test name
    let name = &fixture.name;

    if name.starts_with("style_") || name.starts_with("color_") {
        run_style_test(fixture)
    } else if name.starts_with("padding_") || name.starts_with("margin_") {
        run_style_test(fixture)
    } else if name.starts_with("dimension_") {
        run_style_test(fixture)
    } else if name.starts_with("align_") {
        run_style_test(fixture)
    } else if name.starts_with("border_") {
        // Special case: border_partial_top_bottom has different input structure
        if name == "border_partial_top_bottom" {
            // This test uses partial borders - skip for now as it requires different API
            return Err("SKIPPED: Partial border edges not yet implemented".to_string());
        }
        run_border_test(fixture)
    } else if name.starts_with("join_horizontal") {
        run_join_test(fixture, true)
    } else if name.starts_with("join_vertical") {
        run_join_test(fixture, false)
    } else if name.starts_with("place_") {
        run_place_test(fixture)
    } else {
        Err(format!("Unknown test type: {}", name))
    }
}

/// Run all lipgloss conformance tests
pub fn run_all_tests() -> Vec<(&'static str, Result<(), String>)> {
    let mut loader = FixtureLoader::new();
    let mut results = Vec::new();

    // Load fixtures
    let fixtures = match loader.load_crate("lipgloss") {
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
        "Loaded {} tests from lipgloss.json (Go lib version {})",
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Test runner that loads fixtures and runs all conformance tests
    #[test]
    fn test_lipgloss_conformance() {
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

        println!("\nLipgloss Conformance Results:");
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
                "Lipgloss conformance tests failed: {} of {} tests failed",
                failed,
                results.len()
            );
        }

        assert_eq!(failed, 0, "All conformance tests should pass");
    }

    /// Basic style rendering test
    #[test]
    fn test_style_plain() {
        let style = Style::new();
        let rendered = style.render("Hello");
        assert_eq!(strip_ansi(&rendered), "Hello");
    }

    /// Bold style test
    #[test]
    fn test_style_bold() {
        let style = Style::new().bold();
        let rendered = style.render("Bold");
        assert_eq!(strip_ansi(&rendered), "Bold");
    }

    /// Border test - normal
    #[test]
    fn test_border_normal() {
        let style = Style::new().border(Border::normal());
        let rendered = style.render("Normal");
        let stripped = strip_ansi(&rendered);
        assert!(
            stripped.contains("Normal"),
            "Should contain 'Normal': {}",
            stripped
        );
        assert!(
            stripped.contains("┌") || stripped.contains("+"),
            "Should have border chars: {}",
            stripped
        );
    }

    /// Border test - rounded
    #[test]
    fn test_border_rounded() {
        let style = Style::new().border(Border::rounded());
        let rendered = style.render("Rounded");
        let stripped = strip_ansi(&rendered);
        assert!(
            stripped.contains("Rounded"),
            "Should contain 'Rounded': {}",
            stripped
        );
        assert!(
            stripped.contains("╭"),
            "Should have rounded corner: {}",
            stripped
        );
    }

    /// Join horizontal test
    #[test]
    fn test_join_horizontal_basic() {
        let result = join_horizontal(Position::Top, &["A", "B"]);
        assert_eq!(result, "AB");
    }

    /// Join vertical test
    #[test]
    fn test_join_vertical_basic() {
        let result = join_vertical(Position::Left, &["A", "B"]);
        assert_eq!(result, "A\nB");
    }

    /// Place horizontal center test
    #[test]
    fn test_place_center() {
        let result = place(10, 1, Position::Center, Position::Top, "Hi");
        assert_eq!(result, "    Hi    ");
    }

    /// Width calculation test
    #[test]
    fn test_width_calculation() {
        assert_eq!(lipgloss::width("Hello"), 5);
        assert_eq!(lipgloss::width("Line1\nLongerLine2"), 11);
    }

    /// Height calculation test
    #[test]
    fn test_height_calculation() {
        assert_eq!(lipgloss::height("Hello"), 1);
        assert_eq!(lipgloss::height("Line1\nLine2\nLine3"), 3);
    }
}

/// Integration with the conformance trait system
pub mod integration {
    use super::*;
    use crate::harness::{ConformanceTest, TestCategory, TestContext, TestResult};

    /// Lipgloss conformance test
    pub struct LipglossStyleTest {
        name: String,
    }

    impl LipglossStyleTest {
        pub fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    impl ConformanceTest for LipglossStyleTest {
        fn name(&self) -> &str {
            &self.name
        }

        fn crate_name(&self) -> &str {
            "lipgloss"
        }

        fn category(&self) -> TestCategory {
            TestCategory::Unit
        }

        fn run(&self, ctx: &mut TestContext) -> TestResult {
            let fixture = match ctx.fixture_for_current_test("lipgloss") {
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

    /// Get all lipgloss conformance tests as trait objects
    pub fn all_tests() -> Vec<Box<dyn ConformanceTest>> {
        let mut loader = FixtureLoader::new();
        let fixtures = match loader.load_crate("lipgloss") {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };

        fixtures
            .tests
            .iter()
            .map(|t| Box::new(LipglossStyleTest::new(&t.name)) as Box<dyn ConformanceTest>)
            .collect()
    }
}
