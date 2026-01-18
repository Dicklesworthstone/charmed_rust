//! Conformance tests for the charmed_log crate
//!
//! This module contains conformance tests verifying that the Rust
//! implementation of structured logging matches the behavior of
//! the original Go library.
//!
//! Test categories:
//! - Log levels: numeric values and string representations
//! - Level parsing: parsing string inputs to levels
//! - Level comparisons: ordering and enablement checks

use crate::harness::{FixtureLoader, TestFixture};
use charmed_log::Level;
use serde::Deserialize;
use std::str::FromStr;

/// Input for level constant tests
#[derive(Debug, Deserialize)]
struct LevelInput {
    level: String,
}

/// Expected output for level constant tests
#[derive(Debug, Deserialize)]
struct LevelOutput {
    string_name: String,
    value: i32,
}

/// Input for level parsing tests
#[derive(Debug, Deserialize)]
struct ParseLevelInput {
    input: String,
}

/// Expected output for level parsing tests
#[derive(Debug, Deserialize)]
struct ParseLevelOutput {
    is_valid: bool,
    level: i32,
}

/// Input for level string tests
#[derive(Debug, Deserialize)]
struct LevelStringInput {
    value: i32,
}

/// Expected output for level string tests
#[derive(Debug, Deserialize)]
struct LevelStringOutput {
    string: String,
}

/// Input for level comparison tests
#[derive(Debug, Deserialize)]
struct LevelCompareInput {
    level1: i32,
    level2: i32,
    #[allow(dead_code)]
    level1_name: String,
    #[allow(dead_code)]
    level2_name: String,
}

/// Expected output for level comparison tests
#[derive(Debug, Deserialize)]
struct LevelCompareOutput {
    equal: bool,
    greater_than: bool,
    less_than: bool,
    level1_enabled_at_level2: bool,
}

fn level_from_name(name: &str) -> Option<Level> {
    match name {
        "DebugLevel" => Some(Level::Debug),
        "InfoLevel" => Some(Level::Info),
        "WarnLevel" => Some(Level::Warn),
        "ErrorLevel" => Some(Level::Error),
        "FatalLevel" => Some(Level::Fatal),
        _ => None,
    }
}

fn level_from_value(value: i32) -> Option<Level> {
    match value {
        -4 => Some(Level::Debug),
        0 => Some(Level::Info),
        4 => Some(Level::Warn),
        8 => Some(Level::Error),
        12 => Some(Level::Fatal),
        _ => None,
    }
}

fn level_string_from_value(value: i32) -> String {
    level_from_value(value)
        .map(|level| level.as_str().to_string())
        .unwrap_or_default()
}

fn run_level_test(fixture: &TestFixture) -> Result<(), String> {
    let input: LevelInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {}", e))?;

    let expected: LevelOutput = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {}", e))?;

    let level = level_from_name(&input.level)
        .ok_or_else(|| format!("Unknown level name: {}", input.level))?;

    let actual_string = level.as_str();
    let actual_value = level as i32;

    if actual_string != expected.string_name {
        return Err(format!(
            "String mismatch: expected {}, got {}",
            expected.string_name, actual_string
        ));
    }

    if actual_value != expected.value {
        return Err(format!(
            "Value mismatch: expected {}, got {}",
            expected.value, actual_value
        ));
    }

    Ok(())
}

fn run_parse_level_test(fixture: &TestFixture) -> Result<(), String> {
    let input: ParseLevelInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {}", e))?;

    let expected: ParseLevelOutput = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {}", e))?;

    let parsed = Level::from_str(&input.input);
    let (is_valid, level) = match parsed {
        Ok(level) => (true, level),
        Err(_) => (false, Level::Info),
    };

    if is_valid != expected.is_valid {
        return Err(format!(
            "Validity mismatch for {:?}: expected {}, got {}",
            input.input, expected.is_valid, is_valid
        ));
    }

    if level as i32 != expected.level {
        return Err(format!(
            "Parsed level mismatch for {:?}: expected {}, got {}",
            input.input, expected.level, level as i32
        ));
    }

    Ok(())
}

fn run_level_string_test(fixture: &TestFixture) -> Result<(), String> {
    let input: LevelStringInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {}", e))?;

    let expected: LevelStringOutput = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {}", e))?;

    let actual = level_string_from_value(input.value);

    if actual != expected.string {
        return Err(format!(
            "String mismatch for value {}: expected {:?}, got {:?}",
            input.value, expected.string, actual
        ));
    }

    Ok(())
}

fn run_level_compare_test(fixture: &TestFixture) -> Result<(), String> {
    let input: LevelCompareInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {}", e))?;

    let expected: LevelCompareOutput = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {}", e))?;

    let level1 = level_from_value(input.level1)
        .ok_or_else(|| format!("Unknown level value: {}", input.level1))?;
    let level2 = level_from_value(input.level2)
        .ok_or_else(|| format!("Unknown level value: {}", input.level2))?;

    let actual_equal = level1 == level2;
    let actual_greater = level1 > level2;
    let actual_less = level1 < level2;
    let actual_enabled = level1 >= level2;

    if actual_equal != expected.equal {
        return Err(format!(
            "Equal mismatch: expected {}, got {}",
            expected.equal, actual_equal
        ));
    }
    if actual_greater != expected.greater_than {
        return Err(format!(
            "Greater mismatch: expected {}, got {}",
            expected.greater_than, actual_greater
        ));
    }
    if actual_less != expected.less_than {
        return Err(format!(
            "Less mismatch: expected {}, got {}",
            expected.less_than, actual_less
        ));
    }
    if actual_enabled != expected.level1_enabled_at_level2 {
        return Err(format!(
            "Enablement mismatch: expected {}, got {}",
            expected.level1_enabled_at_level2, actual_enabled
        ));
    }

    Ok(())
}

fn run_test(fixture: &TestFixture) -> Result<(), String> {
    if let Some(reason) = fixture.should_skip() {
        return Err(format!("SKIPPED: {}", reason));
    }

    if fixture.name.starts_with("level_compare_") {
        run_level_compare_test(fixture)
    } else if fixture.name.starts_with("level_string_") {
        run_level_string_test(fixture)
    } else if fixture.name.starts_with("parse_level_") {
        run_parse_level_test(fixture)
    } else if fixture.name.starts_with("level_") {
        run_level_test(fixture)
    } else {
        Err(format!("Unknown test type: {}", fixture.name))
    }
}

/// Run all charmed_log conformance tests
pub fn run_all_tests() -> Vec<(&'static str, Result<(), String>)> {
    let mut loader = FixtureLoader::new();
    let mut results = Vec::new();

    let fixtures = match loader.load_crate("charmed_log") {
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
        "Loaded {} tests from charmed_log.json (Go lib version {})",
        fixtures.tests.len(),
        fixtures.metadata.library_version
    );

    for test in &fixtures.tests {
        let result = run_test(test);
        let name: &'static str = Box::leak(test.name.clone().into_boxed_str());
        results.push((name, result));
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_charmed_log_conformance() {
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

        println!("\nCharmed Log Conformance Results:");
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
                "Charmed Log conformance tests failed: {} of {} tests failed",
                failed,
                results.len()
            );
        }

        assert_eq!(failed, 0, "All conformance tests should pass");
    }
}

/// Integration with the conformance trait system
pub mod integration {
    use super::*;
    use crate::harness::{ConformanceTest, TestCategory, TestContext, TestResult};

    pub struct CharmedLogTest {
        name: String,
    }

    impl CharmedLogTest {
        pub fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    impl ConformanceTest for CharmedLogTest {
        fn name(&self) -> &str {
            &self.name
        }

        fn crate_name(&self) -> &str {
            "charmed_log"
        }

        fn category(&self) -> TestCategory {
            TestCategory::Unit
        }

        fn run(&self, _ctx: &mut TestContext) -> TestResult {
            let mut loader = FixtureLoader::new();
            let fixture = match loader.get_test("charmed_log", &self.name) {
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

    pub fn all_tests() -> Vec<Box<dyn ConformanceTest>> {
        let mut loader = FixtureLoader::new();
        let fixtures = match loader.load_crate("charmed_log") {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };

        fixtures
            .tests
            .iter()
            .map(|t| Box::new(CharmedLogTest::new(&t.name)) as Box<dyn ConformanceTest>)
            .collect()
    }
}
