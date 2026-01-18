//! Generate Conformance Report
//!
//! Binary for generating conformance test reports in various formats.
//!
//! Usage:
//!   generate-report [OPTIONS]
//!
//! Options:
//!   --format <FMT>     Output format: summary, markdown, json (default: summary)
//!   --output <FILE>    Write to file instead of stdout
//!   --include-passed   Include passed tests in detailed reports

use std::env;
use std::fs;
use std::io::Write;

/// Test result for a single test
#[derive(Debug, Clone)]
pub struct TestResult {
    pub crate_name: &'static str,
    pub test_name: &'static str,
    pub status: TestStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestStatus {
    Pass,
    Fail,
    Skipped,
}

/// Collect all test results
fn collect_results() -> Vec<TestResult> {
    let mut all_results = Vec::new();

    let crates: Vec<(&str, fn() -> Vec<(&'static str, Result<(), String>)>)> = vec![
        ("harmonica", charmed_conformance::crates::harmonica::run_all_tests),
        ("lipgloss", charmed_conformance::crates::lipgloss::run_all_tests),
        ("bubbletea", charmed_conformance::crates::bubbletea::run_all_tests),
        ("bubbles", charmed_conformance::crates::bubbles::run_all_tests),
        ("charmed_log", charmed_conformance::crates::charmed_log::run_all_tests),
        ("glamour", charmed_conformance::crates::glamour::run_all_tests),
        ("huh", charmed_conformance::crates::huh::run_all_tests),
        ("integration", charmed_conformance::integration::run_all_tests),
    ];

    for (crate_name, run_fn) in crates {
        let tests = run_fn();
        for (name, result) in tests {
            let status = match &result {
                Ok(()) => TestStatus::Pass,
                Err(msg) if msg.starts_with("SKIPPED:") => TestStatus::Skipped,
                Err(_) => TestStatus::Fail,
            };
            all_results.push(TestResult {
                crate_name,
                test_name: name,
                status,
            });
        }
    }

    all_results
}

/// Generate summary report
fn generate_summary(results: &[TestResult]) -> String {
    let mut output = String::new();

    output.push_str("═══════════════════════════════════════════════════════════════\n");
    output.push_str("              CHARMED RUST CONFORMANCE REPORT\n");
    output.push_str("═══════════════════════════════════════════════════════════════\n\n");

    let mut by_crate: std::collections::HashMap<&str, Vec<&TestResult>> =
        std::collections::HashMap::new();

    for result in results {
        by_crate
            .entry(result.crate_name)
            .or_default()
            .push(result);
    }

    // Sort crates
    let mut crate_names: Vec<_> = by_crate.keys().copied().collect();
    crate_names.sort();

    output.push_str("┌────────────────┬──────────┬────────┬──────────┬───────────┐\n");
    output.push_str("│ Crate          │ Tests    │ Pass   │ Fail     │ Skip      │\n");
    output.push_str("├────────────────┼──────────┼────────┼──────────┼───────────┤\n");

    let mut total_tests = 0;
    let mut total_pass = 0;
    let mut total_fail = 0;
    let mut total_skip = 0;

    for crate_name in &crate_names {
        let crate_results = &by_crate[crate_name];
        let pass = crate_results.iter().filter(|r| r.status == TestStatus::Pass).count();
        let fail = crate_results.iter().filter(|r| r.status == TestStatus::Fail).count();
        let skip = crate_results.iter().filter(|r| r.status == TestStatus::Skipped).count();
        let total = pass + fail + skip;

        total_tests += total;
        total_pass += pass;
        total_fail += fail;
        total_skip += skip;

        output.push_str(&format!(
            "│ {:<14} │ {:<8} │ {:<6} │ {:<8} │ {:<9} │\n",
            crate_name, total, pass, fail, skip
        ));
    }

    output.push_str("├────────────────┼──────────┼────────┼──────────┼───────────┤\n");
    output.push_str(&format!(
        "│ TOTAL          │ {:<8} │ {:<6} │ {:<8} │ {:<9} │\n",
        total_tests, total_pass, total_fail, total_skip
    ));
    output.push_str("└────────────────┴──────────┴────────┴──────────┴───────────┘\n\n");

    let status = if total_fail > 0 { "FAILING" } else { "PASSING" };
    output.push_str(&format!(
        "OVERALL STATUS: {} ({}/{} tests passing)\n",
        status, total_pass, total_tests
    ));

    output
}

/// Generate markdown report
fn generate_markdown(results: &[TestResult], include_passed: bool) -> String {
    let mut output = String::new();

    output.push_str("# Charmed Rust Conformance Report\n\n");

    let mut by_crate: std::collections::HashMap<&str, Vec<&TestResult>> =
        std::collections::HashMap::new();

    for result in results {
        by_crate
            .entry(result.crate_name)
            .or_default()
            .push(result);
    }

    // Sort crates
    let mut crate_names: Vec<_> = by_crate.keys().copied().collect();
    crate_names.sort();

    // Summary table
    output.push_str("## Summary\n\n");
    output.push_str("| Crate | Tests | Pass | Fail | Skip |\n");
    output.push_str("|-------|-------|------|------|------|\n");

    let mut total_tests = 0;
    let mut total_pass = 0;
    let mut total_fail = 0;
    let mut total_skip = 0;

    for crate_name in &crate_names {
        let crate_results = &by_crate[crate_name];
        let pass = crate_results.iter().filter(|r| r.status == TestStatus::Pass).count();
        let fail = crate_results.iter().filter(|r| r.status == TestStatus::Fail).count();
        let skip = crate_results.iter().filter(|r| r.status == TestStatus::Skipped).count();
        let total = pass + fail + skip;

        total_tests += total;
        total_pass += pass;
        total_fail += fail;
        total_skip += skip;

        let status = if fail > 0 { "❌" } else { "✅" };
        output.push_str(&format!(
            "| {} {} | {} | {} | {} | {} |\n",
            status, crate_name, total, pass, fail, skip
        ));
    }

    output.push_str(&format!(
        "| **TOTAL** | **{}** | **{}** | **{}** | **{}** |\n\n",
        total_tests, total_pass, total_fail, total_skip
    ));

    // Detailed results by crate
    output.push_str("## Details\n\n");

    for crate_name in &crate_names {
        let crate_results = &by_crate[crate_name];
        let fail_count = crate_results.iter().filter(|r| r.status == TestStatus::Fail).count();
        let skip_count = crate_results.iter().filter(|r| r.status == TestStatus::Skipped).count();

        if fail_count > 0 || skip_count > 0 || include_passed {
            output.push_str(&format!("### {}\n\n", crate_name));

            // Failed tests
            let failed: Vec<_> = crate_results
                .iter()
                .filter(|r| r.status == TestStatus::Fail)
                .collect();
            if !failed.is_empty() {
                output.push_str("**Failed:**\n");
                for result in failed {
                    output.push_str(&format!("- ❌ `{}`\n", result.test_name));
                }
                output.push('\n');
            }

            // Skipped tests
            let skipped: Vec<_> = crate_results
                .iter()
                .filter(|r| r.status == TestStatus::Skipped)
                .collect();
            if !skipped.is_empty() {
                output.push_str("**Skipped:**\n");
                for result in skipped {
                    output.push_str(&format!("- ⏭️ `{}`\n", result.test_name));
                }
                output.push('\n');
            }

            // Passed tests (if requested)
            if include_passed {
                let passed: Vec<_> = crate_results
                    .iter()
                    .filter(|r| r.status == TestStatus::Pass)
                    .collect();
                if !passed.is_empty() {
                    output.push_str("**Passed:**\n");
                    for result in passed {
                        output.push_str(&format!("- ✅ `{}`\n", result.test_name));
                    }
                    output.push('\n');
                }
            }
        }
    }

    output
}

/// Generate JSON report
fn generate_json(results: &[TestResult]) -> String {
    let mut output = String::new();

    let mut by_crate: std::collections::HashMap<&str, Vec<&TestResult>> =
        std::collections::HashMap::new();

    for result in results {
        by_crate
            .entry(result.crate_name)
            .or_default()
            .push(result);
    }

    output.push_str("{\n");
    output.push_str("  \"report_version\": \"1.0\",\n");
    output.push_str("  \"crates\": {\n");

    let crate_names: Vec<_> = by_crate.keys().copied().collect();
    for (i, crate_name) in crate_names.iter().enumerate() {
        let crate_results = &by_crate[crate_name];
        let pass = crate_results.iter().filter(|r| r.status == TestStatus::Pass).count();
        let fail = crate_results.iter().filter(|r| r.status == TestStatus::Fail).count();
        let skip = crate_results.iter().filter(|r| r.status == TestStatus::Skipped).count();

        output.push_str(&format!("    \"{}\": {{\n", crate_name));
        output.push_str("      \"tests\": [\n");

        for (j, result) in crate_results.iter().enumerate() {
            let status = match result.status {
                TestStatus::Pass => "pass",
                TestStatus::Fail => "fail",
                TestStatus::Skipped => "skipped",
            };
            let comma = if j < crate_results.len() - 1 { "," } else { "" };
            output.push_str(&format!(
                "        {{ \"name\": \"{}\", \"status\": \"{}\" }}{}\n",
                result.test_name, status, comma
            ));
        }

        output.push_str("      ],\n");
        output.push_str("      \"summary\": {\n");
        output.push_str(&format!("        \"total\": {},\n", pass + fail + skip));
        output.push_str(&format!("        \"passed\": {},\n", pass));
        output.push_str(&format!("        \"failed\": {},\n", fail));
        output.push_str(&format!("        \"skipped\": {}\n", skip));
        output.push_str("      }\n");

        let comma = if i < crate_names.len() - 1 { "," } else { "" };
        output.push_str(&format!("    }}{}\n", comma));
    }

    let total_pass: usize = results.iter().filter(|r| r.status == TestStatus::Pass).count();
    let total_fail: usize = results.iter().filter(|r| r.status == TestStatus::Fail).count();
    let total_skip: usize = results.iter().filter(|r| r.status == TestStatus::Skipped).count();

    output.push_str("  },\n");
    output.push_str("  \"summary\": {\n");
    output.push_str(&format!("    \"total\": {},\n", total_pass + total_fail + total_skip));
    output.push_str(&format!("    \"passed\": {},\n", total_pass));
    output.push_str(&format!("    \"failed\": {},\n", total_fail));
    output.push_str(&format!("    \"skipped\": {}\n", total_skip));
    output.push_str("  }\n");
    output.push_str("}\n");

    output
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse format
    let format = args
        .iter()
        .position(|a| a == "--format")
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
        .unwrap_or("summary");

    // Parse output file
    let output_file = args
        .iter()
        .position(|a| a == "--output" || a == "-o")
        .and_then(|i| args.get(i + 1).cloned());

    // Include passed tests
    let include_passed = args.contains(&"--include-passed".to_string());

    // Collect results
    eprintln!("Running conformance tests...");
    let results = collect_results();
    eprintln!("Collected {} test results", results.len());

    // Generate report
    let report = match format {
        "markdown" | "md" => generate_markdown(&results, include_passed),
        "json" => generate_json(&results),
        "summary" | _ => generate_summary(&results),
    };

    // Output
    if let Some(path) = output_file {
        let mut file = fs::File::create(&path).expect("Failed to create output file");
        file.write_all(report.as_bytes()).expect("Failed to write report");
        eprintln!("Report written to: {}", path);
    } else {
        print!("{}", report);
    }
}
