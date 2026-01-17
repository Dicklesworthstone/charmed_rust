//! TestRunner - Executing conformance test suites
//!
//! Provides a runner for executing conformance tests with:
//! - Parallel execution
//! - Filtering by crate/category/name
//! - Result aggregation
//! - Report generation
//!
//! TODO: Full implementation pending (charmed_rust-5x5.1.6)

use std::time::{Duration, Instant};

use super::context::TestContext;
use super::traits::{ConformanceTest, TestCategory, TestResult};

/// Summary of test execution results
#[derive(Debug, Clone, Default)]
pub struct TestSummary {
    /// Total number of tests
    pub total: usize,
    /// Number of passed tests
    pub passed: usize,
    /// Number of failed tests
    pub failed: usize,
    /// Number of skipped tests
    pub skipped: usize,
    /// Total execution time
    pub duration: Duration,
    /// Per-test results
    pub results: Vec<TestRunResult>,
}

/// Result of a single test run
#[derive(Debug, Clone)]
pub struct TestRunResult {
    /// Test ID
    pub id: String,
    /// Test name
    pub name: String,
    /// Crate name
    pub crate_name: String,
    /// Test category
    pub category: TestCategory,
    /// Test result
    pub result: TestResult,
    /// Execution duration
    pub duration: Duration,
}

/// Runner for conformance tests
pub struct TestRunner {
    /// Tests to run
    tests: Vec<Box<dyn ConformanceTest>>,
    /// Filter by crate name (if Some)
    crate_filter: Option<String>,
    /// Filter by category (if Some)
    category_filter: Option<TestCategory>,
    /// Filter by test name pattern (if Some)
    name_filter: Option<String>,
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl TestRunner {
    /// Create a new empty test runner
    pub fn new() -> Self {
        Self {
            tests: Vec::new(),
            crate_filter: None,
            category_filter: None,
            name_filter: None,
        }
    }

    /// Add a test to the runner
    pub fn add_test<T: ConformanceTest + 'static>(&mut self, test: T) {
        self.tests.push(Box::new(test));
    }

    /// Filter tests by crate name
    pub fn filter_crate(mut self, crate_name: &str) -> Self {
        self.crate_filter = Some(crate_name.to_string());
        self
    }

    /// Filter tests by category
    pub fn filter_category(mut self, category: TestCategory) -> Self {
        self.category_filter = Some(category);
        self
    }

    /// Filter tests by name pattern
    pub fn filter_name(mut self, pattern: &str) -> Self {
        self.name_filter = Some(pattern.to_string());
        self
    }

    /// Run all registered tests and return a summary
    pub fn run(&self) -> TestSummary {
        let start = Instant::now();
        let mut summary = TestSummary::default();

        for test in &self.tests {
            // Apply filters
            if let Some(ref crate_filter) = self.crate_filter {
                if test.crate_name() != crate_filter {
                    continue;
                }
            }
            if let Some(category_filter) = self.category_filter {
                if test.category() != category_filter {
                    continue;
                }
            }
            if let Some(ref name_filter) = self.name_filter {
                if !test.name().contains(name_filter) {
                    continue;
                }
            }

            // Run the test
            let test_start = Instant::now();
            let mut ctx = TestContext::new().with_test_name(test.name());
            let result = test.run(&mut ctx);
            let test_duration = test_start.elapsed();

            // Record result
            let run_result = TestRunResult {
                id: test.id(),
                name: test.name().to_string(),
                crate_name: test.crate_name().to_string(),
                category: test.category(),
                result: result.clone(),
                duration: test_duration,
            };

            match result {
                TestResult::Pass => summary.passed += 1,
                TestResult::Fail { .. } => summary.failed += 1,
                TestResult::Skipped { .. } => summary.skipped += 1,
            }

            summary.total += 1;
            summary.results.push(run_result);
        }

        summary.duration = start.elapsed();
        summary
    }

    /// Get the number of registered tests
    pub fn test_count(&self) -> usize {
        self.tests.len()
    }
}
