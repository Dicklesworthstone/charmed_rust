//! TestRunner - Executing conformance test suites
//!
//! Provides a runner for executing conformance tests with:
//! - Parallel execution via rayon
//! - Filtering by crate/category/name
//! - Result aggregation
//! - Report generation (text, JSON, JUnit XML)

use std::collections::HashMap;
use std::io::Write;
use std::time::{Duration, Instant};

use rayon::prelude::*;
use serde::Serialize;

use super::context::TestContext;
use super::traits::{ConformanceTest, TestCategory, TestResult};

/// Summary of test execution results
#[derive(Debug, Clone, Default, Serialize)]
pub struct TestSummary {
    /// Total number of tests
    pub total: usize,
    /// Number of passed tests
    pub passed: usize,
    /// Number of failed tests
    pub failed: usize,
    /// Number of skipped tests
    pub skipped: usize,
    /// Total execution time in milliseconds
    pub duration_ms: u64,
    /// Per-test results
    pub results: Vec<TestRunResult>,
}

impl TestSummary {
    /// Check if all tests passed
    pub fn is_success(&self) -> bool {
        self.failed == 0
    }

    /// Get results grouped by crate
    pub fn by_crate(&self) -> HashMap<&str, Vec<&TestRunResult>> {
        let mut grouped: HashMap<&str, Vec<&TestRunResult>> = HashMap::new();
        for result in &self.results {
            grouped.entry(&result.crate_name).or_default().push(result);
        }
        grouped
    }

    /// Get results grouped by category
    pub fn by_category(&self) -> HashMap<TestCategory, Vec<&TestRunResult>> {
        let mut grouped: HashMap<TestCategory, Vec<&TestRunResult>> = HashMap::new();
        for result in &self.results {
            grouped.entry(result.category).or_default().push(result);
        }
        grouped
    }
}

/// Result of a single test run
#[derive(Debug, Clone, Serialize)]
pub struct TestRunResult {
    /// Test ID in format "crate::name"
    pub id: String,
    /// Test name
    pub name: String,
    /// Crate name
    pub crate_name: String,
    /// Test category
    pub category: TestCategory,
    /// Test result status
    #[serde(flatten)]
    pub result: TestResult,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
}

/// Configuration for report generation
#[derive(Debug, Clone, Default)]
pub struct ReportConfig {
    /// Include verbose output
    pub verbose: bool,
    /// Show only summary, not individual results
    pub summary_only: bool,
    /// Include timing information
    pub show_timing: bool,
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
    /// Whether to run tests in parallel
    parallel: bool,
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
            parallel: true, // Parallel by default
        }
    }

    /// Add a test to the runner
    pub fn add_test<T: ConformanceTest + 'static>(&mut self, test: T) {
        self.tests.push(Box::new(test));
    }

    /// Add multiple tests to the runner
    pub fn add_tests<I, T>(&mut self, tests: I)
    where
        I: IntoIterator<Item = T>,
        T: ConformanceTest + 'static,
    {
        for test in tests {
            self.tests.push(Box::new(test));
        }
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

    /// Filter tests by name pattern (substring match)
    pub fn filter_name(mut self, pattern: &str) -> Self {
        self.name_filter = Some(pattern.to_string());
        self
    }

    /// Enable or disable parallel execution
    pub fn parallel(mut self, enabled: bool) -> Self {
        self.parallel = enabled;
        self
    }

    /// Check if a test passes all filters
    fn passes_filters(&self, test: &dyn ConformanceTest) -> bool {
        if let Some(ref crate_filter) = self.crate_filter {
            if test.crate_name() != crate_filter {
                return false;
            }
        }
        if let Some(category_filter) = self.category_filter {
            if test.category() != category_filter {
                return false;
            }
        }
        if let Some(ref name_filter) = self.name_filter {
            if !test.name().contains(name_filter) {
                return false;
            }
        }
        true
    }

    /// Run a single test and return its result
    fn run_single_test(test: &dyn ConformanceTest) -> TestRunResult {
        let test_start = Instant::now();
        let mut ctx = TestContext::new()
            .with_test_name(test.name())
            .with_logger_test_name(&test.id());
        let result = test.run(&mut ctx);
        let duration = test_start.elapsed();

        TestRunResult {
            id: test.id(),
            name: test.name().to_string(),
            crate_name: test.crate_name().to_string(),
            category: test.category(),
            result,
            duration_ms: duration.as_millis() as u64,
        }
    }

    /// Run all registered tests sequentially and return a summary
    fn run_sequential(&self) -> TestSummary {
        let start = Instant::now();
        let mut results = Vec::new();

        for test in &self.tests {
            if !self.passes_filters(test.as_ref()) {
                continue;
            }
            results.push(Self::run_single_test(test.as_ref()));
        }

        Self::aggregate_results(results, start.elapsed())
    }

    /// Run all registered tests in parallel and return a summary
    fn run_parallel(&self) -> TestSummary {
        let start = Instant::now();

        // Filter tests first
        let filtered_tests: Vec<_> = self
            .tests
            .iter()
            .filter(|t| self.passes_filters(t.as_ref()))
            .collect();

        // Run in parallel using rayon
        let results: Vec<TestRunResult> = filtered_tests
            .par_iter()
            .map(|test| Self::run_single_test(test.as_ref()))
            .collect();

        Self::aggregate_results(results, start.elapsed())
    }

    /// Aggregate test results into a summary
    fn aggregate_results(results: Vec<TestRunResult>, duration: Duration) -> TestSummary {
        let mut summary = TestSummary {
            duration_ms: duration.as_millis() as u64,
            ..Default::default()
        };

        for result in &results {
            summary.total += 1;
            match &result.result {
                TestResult::Pass => summary.passed += 1,
                TestResult::Fail { .. } => summary.failed += 1,
                TestResult::Skipped { .. } => summary.skipped += 1,
            }
        }

        summary.results = results;
        summary
    }

    /// Run all registered tests and return a summary
    ///
    /// Uses parallel execution if enabled (default), otherwise sequential.
    pub fn run(&self) -> TestSummary {
        if self.parallel {
            self.run_parallel()
        } else {
            self.run_sequential()
        }
    }

    /// Get the number of registered tests
    pub fn test_count(&self) -> usize {
        self.tests.len()
    }

    /// Get the number of tests that will run after filtering
    pub fn filtered_count(&self) -> usize {
        self.tests
            .iter()
            .filter(|t| self.passes_filters(t.as_ref()))
            .count()
    }
}

/// Report generator for test results
pub struct ReportGenerator;

impl ReportGenerator {
    /// Generate a text report to the given writer
    pub fn text<W: Write>(
        writer: &mut W,
        summary: &TestSummary,
        config: &ReportConfig,
    ) -> std::io::Result<()> {
        writeln!(
            writer,
            "═══════════════════════════════════════════════════════════════"
        )?;
        writeln!(
            writer,
            "              CHARMED RUST CONFORMANCE TEST RESULTS"
        )?;
        writeln!(
            writer,
            "═══════════════════════════════════════════════════════════════"
        )?;
        writeln!(writer)?;

        // Group by crate
        let by_crate = summary.by_crate();
        let mut crate_names: Vec<_> = by_crate.keys().copied().collect();
        crate_names.sort();

        for crate_name in crate_names {
            let crate_results = &by_crate[crate_name];
            let pass = crate_results.iter().filter(|r| r.result.is_pass()).count();
            let fail = crate_results.iter().filter(|r| r.result.is_fail()).count();
            let skip = crate_results
                .iter()
                .filter(|r| r.result.is_skipped())
                .count();

            let status_char = if fail > 0 { '✗' } else { '✓' };
            write!(
                writer,
                "{} {}: {} pass, {} fail, {} skip",
                status_char, crate_name, pass, fail, skip
            )?;

            if config.show_timing {
                let total_ms: u64 = crate_results.iter().map(|r| r.duration_ms).sum();
                write!(writer, " ({}ms)", total_ms)?;
            }
            writeln!(writer)?;

            if !config.summary_only && (config.verbose || fail > 0) {
                for result in crate_results.iter() {
                    let (icon, msg) = match &result.result {
                        TestResult::Pass => ("  ✓", String::new()),
                        TestResult::Fail { reason } => ("  ✗", format!(" FAILED: {}", reason)),
                        TestResult::Skipped { reason } => {
                            ("  ○", format!(" (skipped: {})", reason))
                        }
                    };
                    if config.verbose || result.result.is_fail() {
                        write!(writer, "{} {}{}", icon, result.name, msg)?;
                        if config.show_timing {
                            write!(writer, " ({}ms)", result.duration_ms)?;
                        }
                        writeln!(writer)?;
                    }
                }
            }
        }

        writeln!(writer)?;
        writeln!(
            writer,
            "───────────────────────────────────────────────────────────────"
        )?;
        write!(
            writer,
            "TOTAL: {} pass, {} fail, {} skip ({} tests)",
            summary.passed, summary.failed, summary.skipped, summary.total
        )?;
        if config.show_timing {
            write!(writer, " in {}ms", summary.duration_ms)?;
        }
        writeln!(writer)?;

        writeln!(writer)?;
        if summary.is_success() {
            writeln!(writer, "RESULT: PASSED")?;
        } else {
            writeln!(writer, "RESULT: FAILED")?;
        }

        Ok(())
    }

    /// Generate a JSON report
    pub fn json<W: Write>(writer: &mut W, summary: &TestSummary) -> std::io::Result<()> {
        let report = serde_json::json!({
            "report_version": "1.0",
            "generated_at": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "summary": {
                "total": summary.total,
                "passed": summary.passed,
                "failed": summary.failed,
                "skipped": summary.skipped,
                "duration_ms": summary.duration_ms,
                "success": summary.is_success(),
            },
            "results": summary.results,
        });

        writeln!(writer, "{}", serde_json::to_string_pretty(&report).unwrap())?;
        Ok(())
    }

    /// Generate a JUnit XML report for CI integration
    pub fn junit_xml<W: Write>(writer: &mut W, summary: &TestSummary) -> std::io::Result<()> {
        writeln!(writer, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
        writeln!(
            writer,
            r#"<testsuites name="charmed_rust_conformance" tests="{}" failures="{}" skipped="{}" time="{:.3}">"#,
            summary.total,
            summary.failed,
            summary.skipped,
            summary.duration_ms as f64 / 1000.0
        )?;

        // Group by crate for test suites
        let by_crate = summary.by_crate();
        let mut crate_names: Vec<_> = by_crate.keys().copied().collect();
        crate_names.sort();

        for crate_name in crate_names {
            let crate_results = &by_crate[crate_name];
            let failures = crate_results.iter().filter(|r| r.result.is_fail()).count();
            let skipped = crate_results
                .iter()
                .filter(|r| r.result.is_skipped())
                .count();
            let total_time_ms: u64 = crate_results.iter().map(|r| r.duration_ms).sum();

            writeln!(
                writer,
                r#"  <testsuite name="{}" tests="{}" failures="{}" skipped="{}" time="{:.3}">"#,
                crate_name,
                crate_results.len(),
                failures,
                skipped,
                total_time_ms as f64 / 1000.0
            )?;

            for result in crate_results.iter() {
                writeln!(
                    writer,
                    r#"    <testcase name="{}" classname="{}" time="{:.3}">"#,
                    xml_escape(&result.name),
                    crate_name,
                    result.duration_ms as f64 / 1000.0
                )?;

                match &result.result {
                    TestResult::Pass => {}
                    TestResult::Fail { reason } => {
                        writeln!(
                            writer,
                            r#"      <failure message="{}">{}</failure>"#,
                            xml_escape(reason),
                            xml_escape(reason)
                        )?;
                    }
                    TestResult::Skipped { reason } => {
                        writeln!(
                            writer,
                            r#"      <skipped message="{}"/>"#,
                            xml_escape(reason)
                        )?;
                    }
                }

                writeln!(writer, "    </testcase>")?;
            }

            writeln!(writer, "  </testsuite>")?;
        }

        writeln!(writer, "</testsuites>")?;
        Ok(())
    }

    /// Generate a GitHub Actions formatted report
    pub fn github_actions<W: Write>(writer: &mut W, summary: &TestSummary) -> std::io::Result<()> {
        // Print errors for failed tests
        for result in &summary.results {
            if let TestResult::Fail { reason } = &result.result {
                writeln!(
                    writer,
                    "::error title=Conformance Test Failed::{}::{} - {}",
                    result.crate_name, result.name, reason
                )?;
            }
        }

        // Print summary
        if summary.is_success() {
            writeln!(
                writer,
                "::notice::Conformance tests passed: {}/{} passed, {} skipped in {}ms",
                summary.passed, summary.total, summary.skipped, summary.duration_ms
            )?;
        } else {
            writeln!(
                writer,
                "::error::Conformance tests failed: {}/{} passed, {} failed, {} skipped",
                summary.passed, summary.total, summary.failed, summary.skipped
            )?;
        }

        Ok(())
    }
}

/// Escape special XML characters
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyPassTest;
    impl ConformanceTest for DummyPassTest {
        fn name(&self) -> &str {
            "dummy_pass"
        }
        fn crate_name(&self) -> &str {
            "test_crate"
        }
        fn category(&self) -> TestCategory {
            TestCategory::Unit
        }
        fn run(&self, _ctx: &mut TestContext) -> TestResult {
            TestResult::Pass
        }
    }

    struct DummyFailTest;
    impl ConformanceTest for DummyFailTest {
        fn name(&self) -> &str {
            "dummy_fail"
        }
        fn crate_name(&self) -> &str {
            "test_crate"
        }
        fn category(&self) -> TestCategory {
            TestCategory::Unit
        }
        fn run(&self, _ctx: &mut TestContext) -> TestResult {
            TestResult::Fail {
                reason: "intentional failure".to_string(),
            }
        }
    }

    #[test]
    fn test_runner_creation() {
        let runner = TestRunner::new();
        assert_eq!(runner.test_count(), 0);
    }

    #[test]
    fn test_runner_add_test() {
        let mut runner = TestRunner::new();
        runner.add_test(DummyPassTest);
        assert_eq!(runner.test_count(), 1);
    }

    #[test]
    fn test_runner_run_passing() {
        let mut runner = TestRunner::new();
        runner.add_test(DummyPassTest);
        let summary = runner.run();
        assert_eq!(summary.total, 1);
        assert_eq!(summary.passed, 1);
        assert_eq!(summary.failed, 0);
        assert!(summary.is_success());
    }

    #[test]
    fn test_runner_run_failing() {
        let mut runner = TestRunner::new();
        runner.add_test(DummyFailTest);
        let summary = runner.run();
        assert_eq!(summary.total, 1);
        assert_eq!(summary.passed, 0);
        assert_eq!(summary.failed, 1);
        assert!(!summary.is_success());
    }

    #[test]
    fn test_runner_mixed_results() {
        let mut runner = TestRunner::new();
        runner.add_test(DummyPassTest);
        runner.add_test(DummyFailTest);
        let summary = runner.run();
        assert_eq!(summary.total, 2);
        assert_eq!(summary.passed, 1);
        assert_eq!(summary.failed, 1);
        assert!(!summary.is_success());
    }

    #[test]
    fn test_runner_filter_by_name() {
        let mut runner = TestRunner::new();
        runner.add_test(DummyPassTest);
        runner.add_test(DummyFailTest);
        let runner = runner.filter_name("pass");
        assert_eq!(runner.filtered_count(), 1);
        let summary = runner.run();
        assert_eq!(summary.total, 1);
        assert_eq!(summary.passed, 1);
    }

    #[test]
    fn test_runner_sequential() {
        let mut runner = TestRunner::new().parallel(false);
        runner.add_test(DummyPassTest);
        runner.add_test(DummyPassTest);
        let summary = runner.run();
        assert_eq!(summary.total, 2);
        assert_eq!(summary.passed, 2);
    }

    #[test]
    fn test_runner_parallel() {
        let mut runner = TestRunner::new().parallel(true);
        runner.add_test(DummyPassTest);
        runner.add_test(DummyPassTest);
        let summary = runner.run();
        assert_eq!(summary.total, 2);
        assert_eq!(summary.passed, 2);
    }

    #[test]
    fn test_report_text() {
        let mut runner = TestRunner::new();
        runner.add_test(DummyPassTest);
        let summary = runner.run();

        let mut output = Vec::new();
        ReportGenerator::text(&mut output, &summary, &ReportConfig::default()).unwrap();
        let text = String::from_utf8(output).unwrap();

        assert!(text.contains("CHARMED RUST CONFORMANCE"));
        assert!(text.contains("1 pass"));
        assert!(text.contains("RESULT: PASSED"));
    }

    #[test]
    fn test_report_json() {
        let mut runner = TestRunner::new();
        runner.add_test(DummyPassTest);
        let summary = runner.run();

        let mut output = Vec::new();
        ReportGenerator::json(&mut output, &summary).unwrap();
        let text = String::from_utf8(output).unwrap();

        assert!(text.contains("\"report_version\": \"1.0\""));
        assert!(text.contains("\"passed\": 1"));
        assert!(text.contains("\"success\": true"));
    }

    #[test]
    fn test_report_junit_xml() {
        let mut runner = TestRunner::new();
        runner.add_test(DummyPassTest);
        runner.add_test(DummyFailTest);
        let summary = runner.run();

        let mut output = Vec::new();
        ReportGenerator::junit_xml(&mut output, &summary).unwrap();
        let text = String::from_utf8(output).unwrap();

        assert!(text.contains(r#"<?xml version="1.0""#));
        assert!(text.contains("testsuites"));
        assert!(text.contains("testsuite"));
        assert!(text.contains("testcase"));
        assert!(text.contains("<failure"));
    }

    #[test]
    fn test_by_crate_grouping() {
        let mut runner = TestRunner::new();
        runner.add_test(DummyPassTest);
        runner.add_test(DummyFailTest);
        let summary = runner.run();

        let by_crate = summary.by_crate();
        assert_eq!(by_crate.len(), 1);
        assert_eq!(by_crate["test_crate"].len(), 2);
    }
}
