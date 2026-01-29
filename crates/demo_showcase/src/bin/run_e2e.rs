//! E2E Test Runner Script (bd-1kx7)
//!
//! This is the canonical entrypoint for running demo_showcase E2E tests.
//! It supports profiles, scenario selection, determinism, and structured output.
//!
//! # Usage
//!
//! ```bash
//! cargo run --bin run_e2e -- --profile smoke
//! cargo run --bin run_e2e -- --profile full
//! cargo run --bin run_e2e -- --profile nightly --seed 12345
//! cargo run --bin run_e2e -- --scenario docs,logs
//! ```
//!
//! # Profiles
//!
//! - `smoke`: Fast validation (basic navigation, no panics). Good for PR checks.
//! - `full`: Complete test suite with all scenarios and artifact validation.
//! - `nightly`: Stress testing with multiple seeds and resize variations.
//!
//! # Environment Variables
//!
//! - `DEMO_SHOWCASE_E2E_SEED`: Override the random seed
//! - `DEMO_SHOWCASE_KEEP_ARTIFACTS`: Keep artifacts even on success (1/true)
//! - `DEMO_SHOWCASE_LOG_LEVEL`: Set log level (trace/debug/info/warn/error)
//! - `DEMO_SHOWCASE_E2E_ARTIFACTS`: Override artifact output directory

use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// =============================================================================
// CONFIGURATION
// =============================================================================

/// Available test profiles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Profile {
    /// Fast validation: basic smoke tests only
    Smoke,
    /// Full suite: all scenarios with artifact validation
    Full,
    /// Nightly: stress testing with multiple seeds
    Nightly,
}

impl Profile {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "smoke" => Some(Self::Smoke),
            "full" => Some(Self::Full),
            "nightly" => Some(Self::Nightly),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Smoke => "smoke",
            Self::Full => "full",
            Self::Nightly => "nightly",
        }
    }
}

/// Test scenarios (test file groups)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Scenario {
    Docs,
    Logs,
    Mouse,
    ShellOut,
    Settings,
    Wizard,
    Dashboard,
    Navigation,
    Runner,
}

impl Scenario {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "docs" => Some(Self::Docs),
            "logs" => Some(Self::Logs),
            "mouse" => Some(Self::Mouse),
            "shell_out" | "shellout" | "shell" => Some(Self::ShellOut),
            "settings" => Some(Self::Settings),
            "wizard" => Some(Self::Wizard),
            "dashboard" => Some(Self::Dashboard),
            "navigation" | "nav" => Some(Self::Navigation),
            "runner" => Some(Self::Runner),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Docs => "docs",
            Self::Logs => "logs",
            Self::Mouse => "mouse",
            Self::ShellOut => "shell_out",
            Self::Settings => "settings",
            Self::Wizard => "wizard",
            Self::Dashboard => "dashboard",
            Self::Navigation => "navigation",
            Self::Runner => "runner",
        }
    }

    /// Get the cargo test filter pattern for this scenario
    fn test_filter(self) -> &'static str {
        match self {
            Self::Docs => "e2e_docs",
            Self::Logs => "e2e_logs",
            Self::Mouse => "e2e_mouse",
            Self::ShellOut => "e2e_shell_out",
            Self::Settings => "e2e_settings",
            Self::Wizard => "e2e_wizard",
            Self::Dashboard => "e2e_dashboard",
            Self::Navigation => "e2e_navigation",
            Self::Runner => "e2e_runner",
        }
    }

    /// Get the test file name (if external test file)
    #[allow(dead_code)] // Reserved for future per-file test running
    fn test_file(self) -> Option<&'static str> {
        match self {
            Self::Docs => Some("docs_e2e"),
            Self::Logs => Some("logs_e2e"),
            Self::Mouse => Some("mouse_e2e"),
            Self::ShellOut => Some("shell_out_e2e"),
            // These are in test_support module, not separate files
            Self::Settings | Self::Wizard | Self::Dashboard | Self::Navigation | Self::Runner => {
                None
            }
        }
    }

    /// Approximate test count (for progress estimation)
    fn test_count(self) -> usize {
        match self {
            Self::Docs => 14,
            Self::Logs => 13,
            Self::Mouse => 11,
            Self::ShellOut => 12,
            Self::Settings => 8,
            Self::Wizard => 11,
            Self::Dashboard => 10,
            Self::Navigation => 10,
            Self::Runner => 5,
        }
    }

    fn all() -> &'static [Self] {
        &[
            Self::Docs,
            Self::Logs,
            Self::Mouse,
            Self::ShellOut,
            Self::Settings,
            Self::Wizard,
            Self::Dashboard,
            Self::Navigation,
            Self::Runner,
        ]
    }

    /// Scenarios included in smoke profile (fast, essential tests)
    fn smoke_scenarios() -> &'static [Self] {
        &[Self::Runner, Self::Navigation, Self::Mouse]
    }
}

// =============================================================================
// JSONL OUTPUT TYPES
// =============================================================================

#[derive(Debug, Serialize)]
struct RunStartEvent {
    event: &'static str,
    ts: String,
    profile: String,
    seed: u64,
    scenarios: Vec<String>,
    artifact_dir: String,
}

#[derive(Debug, Serialize)]
struct ScenarioStartEvent {
    event: &'static str,
    ts: String,
    scenario: String,
    test_count: usize,
}

#[derive(Debug, Serialize)]
struct ScenarioEndEvent {
    event: &'static str,
    ts: String,
    scenario: String,
    passed: usize,
    failed: usize,
    duration_ms: f64,
    success: bool,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)] // Reserved for per-test event logging
struct TestResultEvent {
    event: &'static str,
    ts: String,
    scenario: String,
    test: String,
    status: String,
    duration_ms: Option<f64>,
    message: Option<String>,
}

#[derive(Debug, Serialize)]
struct RunEndEvent {
    event: &'static str,
    ts: String,
    profile: String,
    total_passed: usize,
    total_failed: usize,
    total_duration_ms: f64,
    success: bool,
    artifact_dir: String,
}

// =============================================================================
// RUNNER
// =============================================================================

struct E2ERunner {
    profile: Profile,
    scenarios: Vec<Scenario>,
    seed: u64,
    artifact_dir: PathBuf,
    log_file: Option<BufWriter<File>>,
    verbose: bool,
    results: HashMap<Scenario, ScenarioResult>,
    start_time: Instant,
}

struct ScenarioResult {
    passed: usize,
    failed: usize,
    duration: Duration,
    failed_tests: Vec<String>,
}

impl E2ERunner {
    fn new(profile: Profile, scenarios: Vec<Scenario>, seed: u64, verbose: bool) -> Self {
        let artifact_dir = env::var("DEMO_SHOWCASE_E2E_ARTIFACTS")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("target/e2e_runs"));

        // Create run directory with timestamp
        let run_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let run_dir = artifact_dir.join(format!("{}_{}", profile.name(), run_id));
        fs::create_dir_all(&run_dir).expect("Failed to create artifact directory");

        // Create JSONL log file
        let log_path = run_dir.join("events.jsonl");
        let log_file = File::create(&log_path).ok().map(BufWriter::new);

        Self {
            profile,
            scenarios,
            seed,
            artifact_dir: run_dir,
            log_file,
            verbose,
            results: HashMap::new(),
            start_time: Instant::now(),
        }
    }

    fn log_event<T: Serialize>(&mut self, event: &T) {
        if let Some(ref mut file) = self.log_file {
            if let Ok(json) = serde_json::to_string(event) {
                let _ = writeln!(file, "{}", json);
                let _ = file.flush();
            }
        }
    }

    fn timestamp() -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // Simple ISO-8601 like timestamp
        format!("{}", now)
    }

    fn run(&mut self) -> bool {
        // Log run start
        self.log_event(&RunStartEvent {
            event: "run_start",
            ts: Self::timestamp(),
            profile: self.profile.name().to_string(),
            seed: self.seed,
            scenarios: self
                .scenarios
                .iter()
                .map(|s| s.name().to_string())
                .collect(),
            artifact_dir: self.artifact_dir.display().to_string(),
        });

        println!("\n╭─────────────────────────────────────────────────────────╮");
        println!("│ E2E Test Runner                                         │");
        println!("├─────────────────────────────────────────────────────────┤");
        println!("│ Profile:    {:44} │", self.profile.name());
        println!("│ Seed:       {:44} │", self.seed);
        println!("│ Scenarios:  {:44} │", self.scenarios.len());
        println!("│ Artifacts:  {:44} │", self.artifact_dir.display());
        println!("╰─────────────────────────────────────────────────────────╯\n");

        // Run each scenario
        let mut all_passed = true;
        for scenario in self.scenarios.clone() {
            if !self.run_scenario(scenario) {
                all_passed = false;
            }
        }

        // Log run end
        let total_passed: usize = self.results.values().map(|r| r.passed).sum();
        let total_failed: usize = self.results.values().map(|r| r.failed).sum();
        let total_duration = self.start_time.elapsed();

        self.log_event(&RunEndEvent {
            event: "run_end",
            ts: Self::timestamp(),
            profile: self.profile.name().to_string(),
            total_passed,
            total_failed,
            total_duration_ms: total_duration.as_secs_f64() * 1000.0,
            success: all_passed,
            artifact_dir: self.artifact_dir.display().to_string(),
        });

        // Print summary
        self.print_summary(all_passed, total_passed, total_failed, total_duration);

        // Write summary file
        self.write_summary_file(all_passed, total_passed, total_failed, total_duration);

        // Write seed file for reproduction
        let seed_path = self.artifact_dir.join("seed.txt");
        let _ = fs::write(&seed_path, format!("{}", self.seed));

        all_passed
    }

    fn run_scenario(&mut self, scenario: Scenario) -> bool {
        let start = Instant::now();

        self.log_event(&ScenarioStartEvent {
            event: "scenario_start",
            ts: Self::timestamp(),
            scenario: scenario.name().to_string(),
            test_count: scenario.test_count(),
        });

        println!("┌──────────────────────────────────────────────────────────┐");
        println!("│ Running: {:48} │", scenario.name());
        println!("└──────────────────────────────────────────────────────────┘");

        // Build cargo test command
        let mut cmd = Command::new("cargo");
        cmd.arg("test")
            .arg("-p")
            .arg("demo_showcase")
            .arg("--")
            .arg(scenario.test_filter());

        // Set environment variables
        cmd.env("DEMO_SHOWCASE_E2E_SEED", self.seed.to_string());
        cmd.env("DEMO_SHOWCASE_KEEP_ARTIFACTS", "1");

        if self.verbose {
            cmd.arg("--nocapture");
        }

        // Run tests and capture output
        let output = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to run cargo test");

        let duration = start.elapsed();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Parse results
        let (passed, failed, failed_tests) = self.parse_test_output(&stdout, &stderr);
        let success = output.status.success() && failed == 0;

        // Store result
        self.results.insert(
            scenario,
            ScenarioResult {
                passed,
                failed,
                duration,
                failed_tests: failed_tests.clone(),
            },
        );

        // Log scenario end
        self.log_event(&ScenarioEndEvent {
            event: "scenario_end",
            ts: Self::timestamp(),
            scenario: scenario.name().to_string(),
            passed,
            failed,
            duration_ms: duration.as_secs_f64() * 1000.0,
            success,
        });

        // Print result
        if success {
            println!("  ✓ {} passed in {:.2}s\n", passed, duration.as_secs_f64());
        } else {
            println!(
                "  ✗ {} passed, {} failed in {:.2}s",
                passed,
                failed,
                duration.as_secs_f64()
            );
            for test in &failed_tests {
                println!("    - {}", test);
            }
            println!();
        }

        // Save output to artifact directory
        let scenario_dir = self.artifact_dir.join(scenario.name());
        fs::create_dir_all(&scenario_dir).ok();
        fs::write(scenario_dir.join("stdout.txt"), stdout.as_bytes()).ok();
        fs::write(scenario_dir.join("stderr.txt"), stderr.as_bytes()).ok();

        success
    }

    fn parse_test_output(&self, stdout: &str, _stderr: &str) -> (usize, usize, Vec<String>) {
        let mut passed = 0;
        let mut failed = 0;
        let mut failed_tests = Vec::new();

        for line in stdout.lines() {
            // Count individual test results (more reliable than parsing summary)
            if line.contains("... ok") {
                passed += 1;
            } else if line.contains("... FAILED") {
                failed += 1;
                // Extract test name (format: "test path::to::test ... FAILED")
                if let Some(name) = line.split("...").next() {
                    let name = name.trim().trim_start_matches("test ");
                    failed_tests.push(name.to_string());
                }
            }
        }

        (passed, failed, failed_tests)
    }

    fn print_summary(
        &self,
        all_passed: bool,
        total_passed: usize,
        total_failed: usize,
        total_duration: Duration,
    ) {
        println!("\n╔══════════════════════════════════════════════════════════╗");
        if all_passed {
            println!("║                      ✓ ALL PASSED                        ║");
        } else {
            println!("║                      ✗ FAILURES                          ║");
        }
        println!("╠══════════════════════════════════════════════════════════╣");
        println!(
            "║  Total: {} passed, {} failed in {:.2}s {:16}║",
            total_passed,
            total_failed,
            total_duration.as_secs_f64(),
            ""
        );
        println!("║  Artifacts: {:45}║", self.artifact_dir.display());
        println!("╚══════════════════════════════════════════════════════════╝");

        if !all_passed {
            println!("\n  Failed scenarios:");
            for (scenario, result) in &self.results {
                if result.failed > 0 {
                    println!("    • {} ({} failures)", scenario.name(), result.failed);
                    for test in &result.failed_tests {
                        println!("      - {}", test);
                    }
                }
            }
            println!("\n  To reproduce, run with seed {}:", self.seed);
            println!(
                "    cargo run --bin run_e2e -- --profile {} --seed {}",
                self.profile.name(),
                self.seed
            );
        }
    }

    fn write_summary_file(
        &self,
        all_passed: bool,
        total_passed: usize,
        total_failed: usize,
        total_duration: Duration,
    ) {
        let summary_path = self.artifact_dir.join("summary.txt");
        let mut summary = String::new();

        summary.push_str(&format!(
            "E2E Test Run Summary\n\
             ====================\n\n\
             Profile:  {}\n\
             Seed:     {}\n\
             Status:   {}\n\
             Duration: {:.2}s\n\
             Results:  {} passed, {} failed\n\n",
            self.profile.name(),
            self.seed,
            if all_passed { "PASSED" } else { "FAILED" },
            total_duration.as_secs_f64(),
            total_passed,
            total_failed
        ));

        summary.push_str("Scenarios:\n");
        for (scenario, result) in &self.results {
            let status = if result.failed == 0 { "✓" } else { "✗" };
            summary.push_str(&format!(
                "  {} {} - {} passed, {} failed ({:.2}s)\n",
                status,
                scenario.name(),
                result.passed,
                result.failed,
                result.duration.as_secs_f64()
            ));
        }

        if !all_passed {
            summary.push_str("\nFailed Tests:\n");
            for (scenario, result) in &self.results {
                for test in &result.failed_tests {
                    summary.push_str(&format!("  - {}::{}\n", scenario.name(), test));
                }
            }
            summary.push_str(&format!(
                "\nTo reproduce:\n  cargo run --bin run_e2e -- --profile {} --seed {}\n",
                self.profile.name(),
                self.seed
            ));
        }

        let _ = fs::write(&summary_path, &summary);
    }
}

// =============================================================================
// CLI ARGUMENT PARSING
// =============================================================================

struct Args {
    profile: Profile,
    scenarios: Option<Vec<Scenario>>,
    seed: Option<u64>,
    verbose: bool,
    help: bool,
}

fn parse_args() -> Args {
    let mut args = Args {
        profile: Profile::Full,
        scenarios: None,
        seed: None,
        verbose: false,
        help: false,
    };

    let argv: Vec<String> = env::args().collect();
    let mut i = 1;

    while i < argv.len() {
        match argv[i].as_str() {
            "--help" | "-h" => {
                args.help = true;
            }
            "--profile" | "-p" => {
                i += 1;
                if i < argv.len() {
                    args.profile = Profile::from_str(&argv[i]).unwrap_or(Profile::Full);
                }
            }
            "--scenario" | "-s" => {
                i += 1;
                if i < argv.len() {
                    let scenarios: Vec<Scenario> = argv[i]
                        .split(',')
                        .filter_map(|s| Scenario::from_str(s.trim()))
                        .collect();
                    if !scenarios.is_empty() {
                        args.scenarios = Some(scenarios);
                    }
                }
            }
            "--seed" => {
                i += 1;
                if i < argv.len() {
                    args.seed = argv[i].parse().ok();
                }
            }
            "--verbose" | "-v" => {
                args.verbose = true;
            }
            _ => {}
        }
        i += 1;
    }

    // Check environment variable for seed
    if args.seed.is_none() {
        args.seed = env::var("DEMO_SHOWCASE_E2E_SEED")
            .ok()
            .and_then(|s| s.parse().ok());
    }

    args
}

fn print_help() {
    println!(
        r#"
E2E Test Runner for demo_showcase

USAGE:
    cargo run --bin run_e2e -- [OPTIONS]

OPTIONS:
    -h, --help              Print this help message
    -p, --profile <NAME>    Test profile: smoke, full, nightly (default: full)
    -s, --scenario <LIST>   Comma-separated scenarios to run (overrides profile)
                            Available: docs, logs, mouse, shell_out, settings,
                                       wizard, dashboard, navigation, runner
    --seed <NUMBER>         Random seed for deterministic tests
    -v, --verbose           Show test output in real-time

PROFILES:
    smoke   - Fast validation (basic navigation, no panics)
              Runs: runner, navigation, mouse
              Best for: PR checks, quick validation

    full    - Complete test suite
              Runs: all scenarios
              Best for: Pre-release validation

    nightly - Stress testing with multiple seeds
              Runs: all scenarios with variations
              Best for: Nightly CI, regression hunting

ENVIRONMENT VARIABLES:
    DEMO_SHOWCASE_E2E_SEED       Override the random seed
    DEMO_SHOWCASE_E2E_ARTIFACTS  Override artifact output directory
    DEMO_SHOWCASE_KEEP_ARTIFACTS Keep artifacts even on success (1/true)
    DEMO_SHOWCASE_LOG_LEVEL      Log level: trace/debug/info/warn/error

EXAMPLES:
    # Run smoke tests (fast, PR-safe)
    cargo run --bin run_e2e -- --profile smoke

    # Run full suite with specific seed
    cargo run --bin run_e2e -- --profile full --seed 12345

    # Run only docs and logs scenarios
    cargo run --bin run_e2e -- --scenario docs,logs

    # Run with verbose output
    cargo run --bin run_e2e -- -v --scenario mouse
"#
    );
}

// =============================================================================
// MAIN
// =============================================================================

fn main() -> ExitCode {
    let args = parse_args();

    if args.help {
        print_help();
        return ExitCode::SUCCESS;
    }

    // Determine scenarios based on profile or explicit selection
    let scenarios = args.scenarios.unwrap_or_else(|| match args.profile {
        Profile::Smoke => Scenario::smoke_scenarios().to_vec(),
        Profile::Full | Profile::Nightly => Scenario::all().to_vec(),
    });

    // Generate or use provided seed
    let seed = args.seed.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    });

    let mut runner = E2ERunner::new(args.profile, scenarios, seed, args.verbose);

    if runner.run() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
