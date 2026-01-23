//! TestContext - Integration layer for conformance tests
//!
//! Combines logging, comparison, and fixture loading into a unified
//! interface for running conformance tests.
//!
//! TODO: Full implementation pending (charmed_rust-5x5.1.4)

use std::fmt::Debug;

use super::comparison::{CompareResult, OutputComparator};
use super::fixtures::{FixtureError, FixtureLoader, FixtureResult, TestFixture};
use super::logging::TestLogger;
use super::traits::TestResult;

/// Context for running conformance tests
///
/// Provides a unified interface for:
/// - Logging test inputs, expected outputs, and actual outputs
/// - Comparing values and generating diffs
/// - Loading fixtures and expected outputs
/// - Tracking test results
pub struct TestContext {
    /// Logger for test output
    logger: TestLogger,
    /// Fixture loader for test data
    fixtures: FixtureLoader,
    /// Output comparator for assertions
    comparator: OutputComparator,
    /// Whether any assertion has failed
    has_failures: bool,
    /// Current test name
    test_name: String,
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TestContext {
    /// Create a new test context with default settings
    pub fn new() -> Self {
        Self {
            logger: TestLogger::new(),
            fixtures: FixtureLoader::new(),
            comparator: OutputComparator::new(),
            has_failures: false,
            test_name: String::new(),
        }
    }

    /// Set the current test name
    pub fn with_test_name(mut self, name: &str) -> Self {
        self.test_name = name.to_string();
        self.logger.set_test_name(name);
        self
    }

    /// Update the current test name (updates logger context too)
    pub fn set_test_name(&mut self, name: &str) {
        self.test_name = name.to_string();
        self.logger.set_test_name(name);
    }

    /// Clear the current test name (clears logger context)
    pub fn clear_test_name(&mut self) {
        self.test_name.clear();
        self.logger.clear_test_name();
    }

    /// Get a reference to the logger
    pub fn logger(&mut self) -> &mut TestLogger {
        &mut self.logger
    }

    /// Get a reference to the fixture loader
    pub fn fixtures(&self) -> &FixtureLoader {
        &self.fixtures
    }

    /// Get a mutable reference to the fixture loader
    pub fn fixtures_mut(&mut self) -> &mut FixtureLoader {
        &mut self.fixtures
    }

    /// Get a reference to the comparator
    pub fn comparator(&self) -> &OutputComparator {
        &self.comparator
    }

    /// Load a specific fixture by crate and test name
    pub fn fixture(&mut self, crate_name: &str, test_name: &str) -> FixtureResult<&TestFixture> {
        self.fixtures.get_test(crate_name, test_name)
    }

    /// Load the fixture matching the current test name
    pub fn fixture_for_current_test(&mut self, crate_name: &str) -> FixtureResult<&TestFixture> {
        if self.test_name.is_empty() {
            return Err(FixtureError::TestNotFound {
                crate_name: crate_name.to_string(),
                test_name: "<unset>".to_string(),
            });
        }
        self.fixtures.get_test(crate_name, &self.test_name)
    }

    /// Log an input being tested
    pub fn log_input<T: Debug>(&mut self, name: &str, value: &T) {
        self.logger.log_input(name, value);
    }

    /// Log expected output (from Go reference)
    pub fn log_expected<T: Debug>(&mut self, name: &str, value: &T) {
        self.logger.log_expected(name, value);
    }

    /// Log actual output (from Rust)
    pub fn log_actual<T: Debug>(&mut self, name: &str, value: &T) {
        self.logger.log_actual(name, value);
    }

    /// Assert that two values are equal
    pub fn assert_eq<T: PartialEq + Debug>(&mut self, expected: &T, actual: &T) -> bool {
        let result = self.comparator.compare_debug(expected, actual);
        self.log_comparison_result(&result);
        if result.is_fail() {
            self.has_failures = true;
            false
        } else {
            true
        }
    }

    /// Assert that two strings are equal
    pub fn assert_str_eq(&mut self, expected: &str, actual: &str) -> bool {
        let result = self.comparator.compare_str(expected, actual);
        self.log_comparison_result(&result);
        if result.is_fail() {
            self.has_failures = true;
            false
        } else {
            true
        }
    }

    /// Assert that two f64 values are approximately equal
    pub fn assert_f64_eq(&mut self, expected: f64, actual: f64, epsilon: f64) -> bool {
        let result = self.comparator.compare_f64(expected, actual, epsilon);
        self.log_comparison_result(&result);
        if result.is_fail() {
            self.has_failures = true;
            false
        } else {
            true
        }
    }

    /// Execute a nested section with its own logging context
    pub fn section<F>(&mut self, name: &str, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.logger.info(&format!("Section: {}", name));
        self.logger.indent();
        f(self);
        self.logger.dedent();
    }

    /// Get the final test result
    pub fn result(&self) -> TestResult {
        if self.has_failures {
            TestResult::Fail {
                reason: "One or more assertions failed".to_string(),
            }
        } else {
            TestResult::Pass
        }
    }

    /// Log the result of a comparison
    fn log_comparison_result(&mut self, result: &CompareResult) {
        match result {
            CompareResult::Equal => {
                self.logger.info("Assertion: PASS (equal)");
            }
            CompareResult::ApproximatelyEqual { delta, epsilon, .. } => {
                self.logger.info(&format!(
                    "Assertion: PASS (approximately equal, delta={} <= epsilon={})",
                    delta, epsilon
                ));
            }
            CompareResult::Different(diff) => {
                self.logger.error("Assertion: FAIL");
                self.logger.section("Diff", |logger| {
                    logger.info(&format!("Expected: {}", diff.expected));
                    logger.info(&format!("Actual: {}", diff.actual));
                    logger.info(&format!("{}", diff.describe()));
                    if !diff.inline_diff.is_empty() {
                        logger.info(&format!("Inline: {}", diff.inline_diff));
                    }
                    if !diff.unified_diff.is_empty() {
                        logger.info(&format!("Unified diff:\n{}", diff.unified_diff));
                    }
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = TestContext::new();
        assert!(!ctx.has_failures);
        assert!(ctx.test_name.is_empty());
    }

    #[test]
    fn test_context_with_test_name() {
        let ctx = TestContext::new().with_test_name("my_test");
        assert_eq!(ctx.test_name, "my_test");
    }

    #[test]
    fn test_assertion_pass() {
        let mut ctx = TestContext::new();
        let result = ctx.assert_eq(&42, &42);
        assert!(result);
        assert!(!ctx.has_failures);
        assert!(ctx.result().is_pass());
    }

    #[test]
    fn test_assertion_fail() {
        let mut ctx = TestContext::new();
        let result = ctx.assert_eq(&42, &43);
        assert!(!result);
        assert!(ctx.has_failures);
        assert!(ctx.result().is_fail());
    }

    #[test]
    fn test_string_assertion() {
        let mut ctx = TestContext::new();
        assert!(ctx.assert_str_eq("hello", "hello"));
        assert!(!ctx.assert_str_eq("hello", "world"));
    }

    #[test]
    fn test_float_assertion() {
        let mut ctx = TestContext::new();
        assert!(ctx.assert_f64_eq(1.0, 1.0001, 0.001));
        assert!(!ctx.assert_f64_eq(1.0, 2.0, 0.001));
    }

    #[test]
    fn test_multiple_assertions_first_failure_recorded() {
        let mut ctx = TestContext::new();
        ctx.assert_eq(&1, &1); // pass
        ctx.assert_eq(&2, &3); // fail
        ctx.assert_eq(&4, &4); // pass

        assert!(ctx.has_failures);
        assert!(ctx.result().is_fail());
    }

    #[test]
    fn test_section_logging() {
        let mut ctx = TestContext::new();
        ctx.section("TestSection", |inner| {
            inner.log_input("value", &42);
            inner.assert_eq(&1, &1);
        });
        assert!(!ctx.has_failures);
    }
}
