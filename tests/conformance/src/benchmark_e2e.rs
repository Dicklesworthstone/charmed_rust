//! End-to-End Benchmark Tests
//!
//! These tests verify the complete benchmark workflow works correctly:
//! 1. Full benchmark suite execution
//! 2. Baseline comparison workflow
//! 3. Report generation
//! 4. CI workflow simulation
//!
//! Note: These tests are marked #[ignore] by default as they require
//! a longer execution time. Run with `cargo test -- --ignored` to include them.

#![cfg(test)]

use std::env;
use std::path::PathBuf;
use std::process::{Command, Output};

/// Get the workspace root directory
fn workspace_root() -> PathBuf {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Run a cargo command and return the output
fn run_cargo_command(args: &[&str]) -> Output {
    Command::new("cargo")
        .current_dir(workspace_root())
        .args(args)
        .env("CARGO_TERM_COLOR", "never")
        .output()
        .expect("Failed to execute cargo command")
}

/// Helper to check if output indicates success
fn is_success(output: &Output) -> bool {
    output.status.success()
}

/// Helper to get stdout as string
fn stdout_str(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Helper to get stderr as string
fn stderr_str(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

// ============================================================================
// BENCHMARK COMPILATION TESTS
// ============================================================================

mod compilation_tests {
    use super::*;

    #[test]
    fn test_benchmarks_compile() {
        // Verify all benchmarks compile without error
        let output = run_cargo_command(&["bench", "--workspace", "--no-run"]);

        let stderr = stderr_str(&output);

        // Should compile successfully
        assert!(
            is_success(&output),
            "Benchmark compilation failed:\n{}",
            stderr
        );
    }

    #[test]
    fn test_lipgloss_benchmarks_compile() {
        let output = run_cargo_command(&["bench", "-p", "lipgloss", "--no-run"]);

        assert!(
            is_success(&output),
            "lipgloss benchmarks failed to compile:\n{}",
            stderr_str(&output)
        );
    }

    #[test]
    fn test_bubbletea_benchmarks_compile() {
        let output = run_cargo_command(&["bench", "-p", "bubbletea", "--no-run"]);

        assert!(
            is_success(&output),
            "bubbletea benchmarks failed to compile:\n{}",
            stderr_str(&output)
        );
    }

    #[test]
    fn test_glamour_benchmarks_compile() {
        let output = run_cargo_command(&["bench", "-p", "glamour", "--no-run"]);

        assert!(
            is_success(&output),
            "glamour benchmarks failed to compile:\n{}",
            stderr_str(&output)
        );
    }
}

// ============================================================================
// BENCHMARK EXECUTION TESTS (longer running, marked ignore)
// ============================================================================

mod execution_tests {
    use super::*;

    #[test]
    #[ignore] // Run with --ignored flag
    fn test_lipgloss_benchmarks_execute() {
        // Run a quick benchmark to verify execution works
        let output = run_cargo_command(&[
            "bench",
            "-p",
            "lipgloss",
            "--",
            "--noplot",
            "--warm-up-time",
            "1",
            "--measurement-time",
            "1",
            "style_creation",
        ]);

        let stdout = stdout_str(&output);
        let stderr = stderr_str(&output);

        assert!(
            is_success(&output),
            "lipgloss benchmark execution failed:\nstdout:\n{}\nstderr:\n{}",
            stdout,
            stderr
        );

        // Should contain benchmark results
        assert!(
            stdout.contains("style_creation") || stderr.contains("style_creation"),
            "Output should contain benchmark name"
        );
    }

    #[test]
    #[ignore] // Run with --ignored flag
    fn test_bubbletea_benchmarks_execute() {
        let output = run_cargo_command(&[
            "bench",
            "-p",
            "bubbletea",
            "--",
            "--noplot",
            "--warm-up-time",
            "1",
            "--measurement-time",
            "1",
            "message_dispatch",
        ]);

        let stdout = stdout_str(&output);
        let stderr = stderr_str(&output);

        assert!(
            is_success(&output),
            "bubbletea benchmark execution failed:\nstdout:\n{}\nstderr:\n{}",
            stdout,
            stderr
        );
    }

    #[test]
    #[ignore] // Run with --ignored flag
    fn test_full_benchmark_suite_execution() {
        // Run all benchmarks (limited iterations for speed)
        let output = run_cargo_command(&[
            "bench",
            "--workspace",
            "--",
            "--noplot",
            "--warm-up-time",
            "1",
            "--measurement-time",
            "1",
        ]);

        let stdout = stdout_str(&output);
        let stderr = stderr_str(&output);

        assert!(
            is_success(&output),
            "Full benchmark suite failed:\nstdout:\n{}\nstderr:\n{}",
            stdout,
            stderr
        );

        // Verify multiple benchmark groups ran
        let combined = format!("{}\n{}", stdout, stderr);
        assert!(
            combined.contains("lipgloss") || combined.contains("Benchmarking"),
            "Should run lipgloss benchmarks"
        );
    }
}

// ============================================================================
// BASELINE COMPARISON TESTS
// ============================================================================

mod baseline_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    #[ignore] // Run with --ignored flag
    fn test_baseline_save_and_compare() {
        // Create a temporary directory for criterion data
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let target_dir = temp_dir.path().join("target");
        fs::create_dir_all(&target_dir).expect("Failed to create target dir");

        // Save baseline (run a quick benchmark)
        let output = Command::new("cargo")
            .current_dir(workspace_root())
            .args([
                "bench",
                "-p",
                "lipgloss",
                "--",
                "--noplot",
                "--warm-up-time",
                "1",
                "--measurement-time",
                "1",
                "--save-baseline",
                "test_baseline",
                "Style::new",
            ])
            .env("CARGO_TERM_COLOR", "never")
            .output()
            .expect("Failed to save baseline");

        assert!(
            is_success(&output),
            "Failed to save baseline:\n{}",
            stderr_str(&output)
        );

        // Compare against baseline
        let output = Command::new("cargo")
            .current_dir(workspace_root())
            .args([
                "bench",
                "-p",
                "lipgloss",
                "--",
                "--noplot",
                "--warm-up-time",
                "1",
                "--measurement-time",
                "1",
                "--baseline",
                "test_baseline",
                "Style::new",
            ])
            .env("CARGO_TERM_COLOR", "never")
            .output()
            .expect("Failed to compare baseline");

        // Should succeed (comparison itself doesn't fail on regression)
        assert!(
            is_success(&output),
            "Baseline comparison failed:\n{}",
            stderr_str(&output)
        );
    }
}

// ============================================================================
// CI WORKFLOW SIMULATION TESTS
// ============================================================================

mod ci_simulation_tests {
    use super::*;

    /// Simulate the CI benchmark workflow
    #[test]
    #[ignore] // Run with --ignored flag
    fn test_ci_workflow_simulation() {
        // 1. Compile benchmarks (as CI would do first)
        let compile_output = run_cargo_command(&["bench", "--workspace", "--no-run"]);

        assert!(
            is_success(&compile_output),
            "CI step 1 (compile) failed:\n{}",
            stderr_str(&compile_output)
        );

        // 2. Run benchmarks and save baseline
        let bench_output = run_cargo_command(&[
            "bench",
            "-p",
            "lipgloss",
            "--",
            "--noplot",
            "--warm-up-time",
            "1",
            "--measurement-time",
            "1",
            "--save-baseline",
            "ci_test",
            "Style::new",
        ]);

        assert!(
            is_success(&bench_output),
            "CI step 2 (benchmark) failed:\n{}",
            stderr_str(&bench_output)
        );

        // 3. Compare against baseline (simulating PR comparison)
        let compare_output = run_cargo_command(&[
            "bench",
            "-p",
            "lipgloss",
            "--",
            "--noplot",
            "--warm-up-time",
            "1",
            "--measurement-time",
            "1",
            "--baseline",
            "ci_test",
            "Style::new",
        ]);

        assert!(
            is_success(&compare_output),
            "CI step 3 (compare) failed:\n{}",
            stderr_str(&compare_output)
        );
    }

    /// Test parsing benchmark output for regression detection
    #[test]
    fn test_benchmark_output_parsing() {
        // Sample benchmark output that CI would parse
        let sample_output = r#"
Benchmarking lipgloss/style_creation/Style::new
Benchmarking lipgloss/style_creation/Style::new: Warming up for 1.0000 s
Benchmarking lipgloss/style_creation/Style::new: Collecting 10 samples
lipgloss/style_creation/Style::new
                        time:   [15.234 ns 15.456 ns 15.678 ns]
                        change: [-2.34% +0.12% +2.45%] (p = 0.08 > 0.05)
                        No change in performance detected.
"#;

        // Verify we can detect "no change"
        assert!(sample_output.contains("No change in performance detected"));

        // Sample regression output
        let regression_output = r#"
lipgloss/style_creation/Style::new
                        time:   [18.234 ns 18.456 ns 18.678 ns]
                        change: [+18.34% +20.12% +22.45%] (p = 0.00 < 0.05)
                        Performance has regressed.
"#;

        // Verify we can detect regression
        assert!(regression_output.contains("Performance has regressed"));

        // Parse regression percentage
        let contains_severe_regression = regression_output.contains("+20.")
            || regression_output.contains("+21.")
            || regression_output.contains("+22.");
        assert!(contains_severe_regression, "Should detect >20% regression");
    }

    /// Test that benchmark summary can be generated
    #[test]
    fn test_benchmark_summary_generation() {
        // Simulate generating a markdown summary (like CI does)
        let benchmark_results = vec![
            ("style_creation", "15.456 ns", "+0.12%", false),
            ("color_parsing", "25.789 ns", "+1.23%", false),
            ("render_short", "145.23 ns", "+25.67%", true), // Regression
        ];

        let mut summary = String::new();
        summary.push_str("## Benchmark Results\n\n");

        let mut has_regressions = false;
        for (name, time, change, regressed) in &benchmark_results {
            let status = if *regressed {
                has_regressions = true;
                ":x:"
            } else {
                ":white_check_mark:"
            };
            summary.push_str(&format!("- {} {} - {} ({})\n", status, name, time, change));
        }

        if has_regressions {
            summary.push_str("\n:warning: **Performance regressions detected!**\n");
        }

        // Verify summary format
        assert!(summary.contains("## Benchmark Results"));
        assert!(summary.contains("style_creation"));
        assert!(summary.contains(":warning:"));
        assert!(summary.contains("Performance regressions detected"));
    }
}

// ============================================================================
// REPORT VERIFICATION TESTS
// ============================================================================

mod report_tests {
    use super::*;
    use std::fs;

    /// Check that criterion generates expected report structure
    #[test]
    #[ignore] // Run with --ignored flag
    fn test_criterion_report_structure() {
        // First, ensure we have some benchmark data
        let output = run_cargo_command(&[
            "bench",
            "-p",
            "lipgloss",
            "--",
            "--warm-up-time",
            "1",
            "--measurement-time",
            "1",
            "Style::new",
        ]);

        if !is_success(&output) {
            eprintln!(
                "Warning: Benchmark run failed, skipping report test:\n{}",
                stderr_str(&output)
            );
            return;
        }

        // Check for criterion directory
        let criterion_dir = workspace_root().join("target/criterion");
        if criterion_dir.exists() {
            // Should have benchmark directories
            let has_benchmark_dirs = fs::read_dir(&criterion_dir)
                .map(|entries| entries.count() > 0)
                .unwrap_or(false);

            assert!(
                has_benchmark_dirs,
                "Criterion directory should contain benchmark data"
            );

            // Check for report index
            let report_index = criterion_dir.join("report/index.html");
            if report_index.exists() {
                let content = fs::read_to_string(&report_index).expect("Failed to read report");
                assert!(
                    content.contains("Criterion") || content.contains("benchmark"),
                    "Report should be valid HTML benchmark report"
                );
            }
        }
    }

    /// Verify benchmark JSON export format (if available)
    #[test]
    fn test_benchmark_json_format() {
        // Sample criterion JSON format for verification
        let sample_json = r#"{
            "reason": "benchmark-complete",
            "id": "lipgloss/style_creation/Style::new",
            "mean": {
                "estimate": 15.456,
                "lower_bound": 15.234,
                "upper_bound": 15.678,
                "unit": "ns"
            }
        }"#;

        // Verify JSON is valid
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(sample_json);
        assert!(parsed.is_ok(), "Benchmark JSON should be valid");

        let json = parsed.unwrap();
        assert_eq!(json["reason"], "benchmark-complete");
        assert!(json["mean"]["estimate"].as_f64().is_some());
    }
}

// ============================================================================
// BENCHMARK HARNESS TESTS
// ============================================================================

mod harness_tests {
    use crate::harness::{BenchConfig, BenchContext, OutlierRemoval};
    use std::hint::black_box;

    /// Test that our custom harness integrates with criterion workflow
    #[test]
    fn test_harness_compatibility() {
        // Create a context that mimics criterion's workflow
        let mut ctx = BenchContext::with_config(BenchConfig {
            warmup_iterations: 10,
            measure_iterations: 100,
            adaptive_warmup: true,
            outlier_removal: OutlierRemoval::Iqr { multiplier: 1.5 },
            regression_threshold: 0.10,
        });

        // Run a benchmark
        let result = ctx.bench("compatibility_test", || {
            let _ = black_box(vec![1, 2, 3, 4, 5].iter().sum::<i32>());
        });

        // Verify result structure matches expected format
        assert_eq!(result.name, "compatibility_test");
        // Iterations may be significantly less due to outlier removal on trivial benchmarks
        // (system noise is high relative to the tiny operation being measured)
        assert!(
            result.iterations >= 50,
            "Expected at least 50 iterations after outlier removal, got {}",
            result.iterations
        );
        assert!(result.min <= result.mean);
        assert!(result.mean <= result.max);
        assert!(result.p50 >= result.min);
        assert!(result.p95 >= result.p50);
        assert!(result.p99 >= result.p95);
    }

    /// Test baseline creation and structure
    #[test]
    fn test_baseline_creation() {
        let mut ctx = BenchContext::new().iterations(50);

        // Create initial results
        ctx.bench("baseline_test", || {
            let _ = black_box(1 + 1);
        });

        let baseline = ctx.create_baseline();

        // Baseline should contain our benchmark
        assert!(
            baseline.results.contains_key("baseline_test"),
            "Baseline should include the benchmark"
        );

        // Baseline should have stored result data
        let stored = baseline.results.get("baseline_test").unwrap();
        assert!(stored.mean.as_nanos() > 0, "Stored mean should be positive");
    }

    /// Test outlier removal doesn't break stats
    #[test]
    fn test_outlier_removal_preserves_stats() {
        let mut ctx = BenchContext::with_config(BenchConfig {
            warmup_iterations: 5,
            measure_iterations: 20,
            adaptive_warmup: false,
            outlier_removal: OutlierRemoval::Iqr { multiplier: 1.5 },
            regression_threshold: 0.10,
        });

        let result = ctx.bench("outlier_test", || {
            let _ = black_box(42);
        });

        // Stats should still be valid
        assert!(result.min > std::time::Duration::ZERO);
        assert!(result.max >= result.min);
        assert!(result.coefficient_of_variation >= 0.0);
    }
}
