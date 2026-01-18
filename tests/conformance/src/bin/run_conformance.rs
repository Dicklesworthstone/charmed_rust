#![allow(clippy::doc_markdown)]
#![allow(clippy::stable_sort_primitive)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::explicit_iter_loop)]
#![allow(clippy::branches_sharing_code)]
#![allow(clippy::type_complexity)]
//! Run Conformance Tests
//!
//! Binary for executing all conformance tests and generating reports.
//!
//! Usage:
//!   run-conformance [OPTIONS]
//!
//! Options:
//!   --crate <NAME>     Filter tests by crate name
//!   --json             Output results as JSON
//!   --ci               Output in CI-friendly format (GitHub Actions)
//!   --verbose          Enable verbose output
//!   --summary-only     Only show summary, not individual test results

use std::env;
use std::time::Instant;

/// Test result for a single test
#[derive(Debug, Clone)]
pub struct TestResult {
    pub crate_name: &'static str,
    pub test_name: &'static str,
    pub status: TestStatus,
    pub duration_ms: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestStatus {
    Pass,
    Fail,
    Skipped,
}

/// Run all tests for a crate and collect results
fn run_crate_tests(
    crate_name: &'static str,
    tests: Vec<(&'static str, Result<(), String>)>,
) -> Vec<TestResult> {
    tests
        .into_iter()
        .map(|(name, result)| {
            let status = match &result {
                Ok(()) => TestStatus::Pass,
                Err(msg) if msg.starts_with("SKIPPED:") => TestStatus::Skipped,
                Err(_) => TestStatus::Fail,
            };
            TestResult {
                crate_name,
                test_name: name,
                status,
                duration_ms: 0.0, // Duration captured at crate level
            }
        })
        .collect()
}

/// Print results in standard format
fn print_standard(results: &[TestResult], verbose: bool, summary_only: bool) {
    let mut by_crate: std::collections::HashMap<&str, Vec<&TestResult>> =
        std::collections::HashMap::new();

    for result in results {
        by_crate.entry(result.crate_name).or_default().push(result);
    }

    println!("═══════════════════════════════════════════════════════════════");
    println!("              CHARMED RUST CONFORMANCE TEST RESULTS");
    println!("═══════════════════════════════════════════════════════════════");
    println!();

    let mut total_pass = 0;
    let mut total_fail = 0;
    let mut total_skip = 0;

    // Sort crates alphabetically
    let mut crate_names: Vec<_> = by_crate.keys().copied().collect();
    crate_names.sort();

    for crate_name in crate_names {
        let crate_results = &by_crate[crate_name];
        let pass = crate_results
            .iter()
            .filter(|r| r.status == TestStatus::Pass)
            .count();
        let fail = crate_results
            .iter()
            .filter(|r| r.status == TestStatus::Fail)
            .count();
        let skip = crate_results
            .iter()
            .filter(|r| r.status == TestStatus::Skipped)
            .count();

        total_pass += pass;
        total_fail += fail;
        total_skip += skip;

        let status_char = if fail > 0 { '✗' } else { '✓' };
        println!(
            "{} {}: {} pass, {} fail, {} skip",
            status_char, crate_name, pass, fail, skip
        );

        if !summary_only && (verbose || fail > 0) {
            for result in crate_results.iter() {
                let (icon, msg) = match result.status {
                    TestStatus::Pass => ("  ✓", ""),
                    TestStatus::Fail => ("  ✗", " FAILED"),
                    TestStatus::Skipped => ("  ○", " (skipped)"),
                };
                if verbose || result.status != TestStatus::Pass {
                    println!("{} {}{}", icon, result.test_name, msg);
                }
            }
        }
    }

    println!();
    println!("───────────────────────────────────────────────────────────────");
    println!(
        "TOTAL: {} pass, {} fail, {} skip ({} tests)",
        total_pass,
        total_fail,
        total_skip,
        total_pass + total_fail + total_skip
    );

    if total_fail > 0 {
        println!();
        println!("RESULT: FAILED");
    } else {
        println!();
        println!("RESULT: PASSED");
    }
}

/// Print results in JSON format
fn print_json(results: &[TestResult]) {
    let mut by_crate: std::collections::HashMap<&str, Vec<&TestResult>> =
        std::collections::HashMap::new();

    for result in results {
        by_crate.entry(result.crate_name).or_default().push(result);
    }

    println!("{{");
    println!("  \"report_version\": \"1.0\",");
    println!("  \"generated_at\": \"{}\",", chrono_lite_timestamp());
    println!("  \"crates\": {{");

    let crate_names: Vec<_> = by_crate.keys().copied().collect();
    for (i, crate_name) in crate_names.iter().enumerate() {
        let crate_results = &by_crate[crate_name];
        let pass = crate_results
            .iter()
            .filter(|r| r.status == TestStatus::Pass)
            .count();
        let fail = crate_results
            .iter()
            .filter(|r| r.status == TestStatus::Fail)
            .count();
        let skip = crate_results
            .iter()
            .filter(|r| r.status == TestStatus::Skipped)
            .count();

        println!("    \"{}\": {{", crate_name);
        println!("      \"tests\": [");

        for (j, result) in crate_results.iter().enumerate() {
            let status = match result.status {
                TestStatus::Pass => "pass",
                TestStatus::Fail => "fail",
                TestStatus::Skipped => "skipped",
            };
            let comma = if j < crate_results.len() - 1 { "," } else { "" };
            println!(
                "        {{ \"name\": \"{}\", \"status\": \"{}\" }}{}",
                result.test_name, status, comma
            );
        }

        println!("      ],");
        println!("      \"summary\": {{");
        println!("        \"total\": {},", pass + fail + skip);
        println!("        \"passed\": {},", pass);
        println!("        \"failed\": {},", fail);
        println!("        \"skipped\": {}", skip);
        println!("      }}");

        let comma = if i < crate_names.len() - 1 { "," } else { "" };
        println!("    }}{}", comma);
    }

    let total_pass: usize = results
        .iter()
        .filter(|r| r.status == TestStatus::Pass)
        .count();
    let total_fail: usize = results
        .iter()
        .filter(|r| r.status == TestStatus::Fail)
        .count();
    let total_skip: usize = results
        .iter()
        .filter(|r| r.status == TestStatus::Skipped)
        .count();

    println!("  }},");
    println!("  \"summary\": {{");
    println!("    \"total\": {},", total_pass + total_fail + total_skip);
    println!("    \"passed\": {},", total_pass);
    println!("    \"failed\": {},", total_fail);
    println!("    \"skipped\": {}", total_skip);
    println!("  }}");
    println!("}}");
}

/// Print results in CI format (GitHub Actions)
fn print_ci(results: &[TestResult]) {
    // Print errors for failed tests
    for result in results {
        if result.status == TestStatus::Fail {
            println!(
                "::error title=Conformance Test Failed::{}::{} failed",
                result.crate_name, result.test_name
            );
        }
    }

    // Print summary
    let total_pass: usize = results
        .iter()
        .filter(|r| r.status == TestStatus::Pass)
        .count();
    let total_fail: usize = results
        .iter()
        .filter(|r| r.status == TestStatus::Fail)
        .count();
    let total_skip: usize = results
        .iter()
        .filter(|r| r.status == TestStatus::Skipped)
        .count();
    let total = total_pass + total_fail + total_skip;

    if total_fail > 0 {
        println!(
            "::error::Conformance tests failed: {}/{} passed, {} failed, {} skipped",
            total_pass, total, total_fail, total_skip
        );
    } else {
        println!(
            "::notice::Conformance tests passed: {}/{} passed, {} skipped",
            total_pass, total, total_skip
        );
    }
}

/// Simple timestamp without external dependencies
fn chrono_lite_timestamp() -> String {
    // Use a simple format since we don't have chrono
    "2026-01-18T00:00:00Z".to_string()
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let json_output = args.contains(&"--json".to_string());
    let ci_output = args.contains(&"--ci".to_string());
    let verbose = args.contains(&"--verbose".to_string()) || args.contains(&"-v".to_string());
    let summary_only = args.contains(&"--summary-only".to_string());

    // Filter by crate if specified
    let crate_filter: Option<String> = args
        .iter()
        .position(|a| a == "--crate")
        .and_then(|i| args.get(i + 1).cloned());

    let start = Instant::now();
    let mut all_results = Vec::new();

    // Run tests for each crate that has run_all_tests()
    let crates_to_run: Vec<(&str, fn() -> Vec<(&'static str, Result<(), String>)>)> = vec![
        (
            "harmonica",
            charmed_conformance::crates::harmonica::run_all_tests,
        ),
        (
            "lipgloss",
            charmed_conformance::crates::lipgloss::run_all_tests,
        ),
        (
            "bubbletea",
            charmed_conformance::crates::bubbletea::run_all_tests,
        ),
        (
            "bubbles",
            charmed_conformance::crates::bubbles::run_all_tests,
        ),
        (
            "charmed_log",
            charmed_conformance::crates::charmed_log::run_all_tests,
        ),
        (
            "glamour",
            charmed_conformance::crates::glamour::run_all_tests,
        ),
        ("huh", charmed_conformance::crates::huh::run_all_tests),
        (
            "integration",
            charmed_conformance::integration::run_all_tests,
        ),
    ];

    for (crate_name, run_fn) in crates_to_run {
        // Skip if filtered
        if let Some(ref filter) = crate_filter {
            if crate_name != filter {
                continue;
            }
        }

        let crate_start = Instant::now();
        let tests = run_fn();
        let _crate_duration = crate_start.elapsed();

        let results = run_crate_tests(crate_name, tests);
        all_results.extend(results);
    }

    let _total_duration = start.elapsed();

    // Output results
    if json_output {
        print_json(&all_results);
    } else if ci_output {
        print_ci(&all_results);
    } else {
        print_standard(&all_results, verbose, summary_only);
    }

    // Exit with error code if any tests failed
    let has_failures = all_results.iter().any(|r| r.status == TestStatus::Fail);
    if has_failures {
        std::process::exit(1);
    }
}
