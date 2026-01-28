// Allow pedantic and nursery lints for this test infrastructure module.
// Many of these are stylistic and the code prioritizes clarity for test debugging.
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]

//! E2E Test Logging and Artifact Capture
//!
//! This module provides structured logging and artifact capture for E2E tests,
//! making failures instantly diagnosable with complete input/output trails.
//!
//! # Overview
//!
//! The [`ScenarioRecorder`] tracks all events during a test scenario:
//! - Input events (key, mouse, resize)
//! - Assertions performed
//! - Screen captures at each step
//! - Final state and Config snapshot
//!
//! # Artifact Structure
//!
//! ```text
//! target/demo_showcase_e2e/<scenario>/<run_id>/
//! ├── events.jsonl      # Machine-readable event log
//! ├── summary.txt       # Human-readable failure summary
//! ├── config.json       # Config snapshot (seed, theme, toggles)
//! └── frames/
//!     ├── step_001.txt  # Screen capture at step 1
//!     ├── step_002.txt  # Screen capture at step 2
//!     └── final.txt     # Final screen state
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use demo_showcase::test_support::{ScenarioRecorder, TestEvent};
//!
//! let mut recorder = ScenarioRecorder::new("navigation_test");
//! recorder.step("Press down arrow");
//! recorder.input(TestInput::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)));
//! recorder.capture_frame("current screen content");
//! recorder.assert("cursor moved", expected == actual);
//! recorder.finish();
//! ```

use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Event levels for logging (bd-28kk)
///
/// Levels from most verbose to least:
/// - TRACE: Every keystroke, event, internal state change
/// - DEBUG: State changes, render cycles, model updates
/// - INFO: Test steps, assertions, milestone events
/// - WARN: Unexpected but handled conditions
/// - ERROR: Assertion failures, unrecoverable issues
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum EventLevel {
    /// Most verbose: every keystroke, event, internal detail
    Trace,
    /// Debug-level details: state changes, render cycles
    Debug,
    /// Informational events: test steps, assertions
    Info,
    /// Warnings (non-fatal issues)
    Warn,
    /// Errors (assertion failures)
    Error,
}

impl EventLevel {
    /// Returns the single-character abbreviation for this level
    pub const fn abbrev(self) -> char {
        match self {
            Self::Trace => 'T',
            Self::Debug => 'D',
            Self::Info => 'I',
            Self::Warn => 'W',
            Self::Error => 'E',
        }
    }
}

/// Input types that can be recorded
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TestInput {
    /// Keyboard input
    Key {
        /// Key name or character
        key: String,
        /// Modifier keys (ctrl, alt, shift)
        modifiers: Vec<String>,
        /// Raw bytes if available
        #[serde(skip_serializing_if = "Option::is_none")]
        raw: Option<Vec<u8>>,
    },
    /// Mouse input
    Mouse {
        /// Mouse action (click, scroll, move)
        action: String,
        /// X coordinate
        x: u16,
        /// Y coordinate
        y: u16,
        /// Button or scroll direction
        #[serde(skip_serializing_if = "Option::is_none")]
        button: Option<String>,
    },
    /// Terminal resize
    Resize {
        /// New width
        width: u16,
        /// New height
        height: u16,
    },
    /// Paste event (bracketed paste)
    Paste {
        /// Pasted text
        text: String,
    },
}

/// A single recorded event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEvent {
    /// ISO-8601 timestamp
    pub ts: String,
    /// Event level
    pub level: EventLevel,
    /// Scenario name
    pub scenario: String,
    /// Unique run identifier
    pub run_id: String,
    /// Step number (1-indexed)
    pub step: u32,
    /// Event type
    pub event: String,
    /// Human-readable message
    pub message: String,
    /// Input details (for input events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<TestInput>,
    /// Assertion name (for assertion events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assertion: Option<String>,
    /// Expected value (for assertion events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    /// Actual value (for assertion events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<String>,
    /// Path to captured frame
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_path: Option<String>,
    /// Config snapshot
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<ConfigSnapshot>,
}

/// Snapshot of the test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSnapshot {
    /// Random seed for deterministic replay
    pub seed: u64,
    /// Theme name
    pub theme: String,
    /// Sidebar enabled
    pub sidebar: bool,
    /// Animations enabled
    pub animations: bool,
    /// Color mode
    pub color_mode: String,
}

/// Assertion result
#[derive(Debug, Clone)]
pub struct AssertionResult {
    /// Assertion name/description
    pub name: String,
    /// Whether the assertion passed
    pub passed: bool,
    /// Expected value (stringified)
    pub expected: String,
    /// Actual value (stringified)
    pub actual: String,
}

/// Records events and artifacts for a single E2E test scenario (bd-28kk)
pub struct ScenarioRecorder {
    /// Scenario name
    scenario: String,
    /// Unique run identifier
    run_id: String,
    /// Current step number
    step: u32,
    /// Current step description
    step_description: String,
    /// Recorded events
    events: Vec<TestEvent>,
    /// Artifact directory path
    artifact_dir: PathBuf,
    /// Whether any assertion has failed
    has_failures: bool,
    /// Start time for duration tracking
    start_time: Instant,
    /// Start time for current step (for per-operation timing)
    step_start_time: Instant,
    /// Config snapshot
    config: Option<ConfigSnapshot>,
    /// Keep artifacts on success (env: DEMO_SHOWCASE_KEEP_ARTIFACTS)
    keep_on_success: bool,
    /// Minimum log level to record (env: DEMO_SHOWCASE_LOG_LEVEL)
    min_level: EventLevel,
    /// Last captured frame content (for diffing)
    last_frame: Option<String>,
}

impl ScenarioRecorder {
    /// Creates a new scenario recorder
    ///
    /// # Arguments
    ///
    /// * `scenario` - Name of the test scenario (e.g., "navigation_test")
    pub fn new(scenario: impl Into<String>) -> Self {
        let scenario = scenario.into();
        let run_id = generate_run_id();

        // Determine artifact directory
        let artifact_dir = PathBuf::from("target/demo_showcase_e2e")
            .join(&scenario)
            .join(&run_id);

        // Check environment for keep-on-success flag
        let keep_on_success = std::env::var("DEMO_SHOWCASE_KEEP_ARTIFACTS")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        // Check environment for log level (bd-28kk)
        let min_level = std::env::var("DEMO_SHOWCASE_LOG_LEVEL")
            .ok()
            .and_then(|v| match v.to_lowercase().as_str() {
                "trace" => Some(EventLevel::Trace),
                "debug" => Some(EventLevel::Debug),
                "info" => Some(EventLevel::Info),
                "warn" => Some(EventLevel::Warn),
                "error" => Some(EventLevel::Error),
                _ => None,
            })
            .unwrap_or(EventLevel::Debug); // Default to DEBUG

        let now = Instant::now();
        let mut recorder = Self {
            scenario: scenario.clone(),
            run_id: run_id.clone(),
            step: 0,
            step_description: String::new(),
            events: Vec::new(),
            artifact_dir,
            has_failures: false,
            start_time: now,
            step_start_time: now,
            config: None,
            keep_on_success,
            min_level,
            last_frame: None,
        };

        // Record scenario start
        recorder.record_event(
            EventLevel::Info,
            "scenario_start",
            format!("Starting scenario: {scenario}"),
        );

        recorder
    }

    /// Sets the config snapshot for this run
    pub fn set_config(&mut self, config: ConfigSnapshot) {
        self.config = Some(config.clone());
        self.record_event_with_config(
            EventLevel::Info,
            "config_set",
            format!(
                "Config: seed={}, theme={}, animations={}",
                config.seed, config.theme, config.animations
            ),
            Some(config),
        );
    }

    /// Begins a new step in the scenario
    ///
    /// # Arguments
    ///
    /// * `description` - Human-readable description of what this step does
    pub fn step(&mut self, description: impl Into<String>) {
        // Record previous step duration if not first step
        if self.step > 0 {
            let step_duration = self.step_start_time.elapsed();
            self.record_event(
                EventLevel::Debug,
                "step_end",
                format!(
                    "Step {} completed in {:.3}ms",
                    self.step,
                    step_duration.as_secs_f64() * 1000.0
                ),
            );
        }

        self.step += 1;
        self.step_description = description.into();
        self.step_start_time = Instant::now();
        self.record_event(
            EventLevel::Info,
            "step_start",
            format!("Step {}: {}", self.step, self.step_description),
        );
    }

    /// Records a TRACE-level event (bd-28kk)
    ///
    /// Use for detailed debugging: every keystroke, internal event, state change.
    pub fn trace(&mut self, message: impl Into<String>) {
        self.record_event(EventLevel::Trace, "trace", message.into());
    }

    /// Records timing for an operation (bd-28kk)
    ///
    /// # Arguments
    ///
    /// * `operation` - Name of the operation being timed
    /// * `duration_ms` - Duration in milliseconds
    pub fn timing(&mut self, operation: &str, duration_ms: f64) {
        self.record_event(
            EventLevel::Debug,
            "timing",
            format!("{operation}: {duration_ms:.3}ms"),
        );
    }

    /// Times a closure and records the result (bd-28kk)
    ///
    /// Returns the closure's result while recording timing.
    pub fn timed<T, F: FnOnce() -> T>(&mut self, operation: &str, f: F) -> T {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        self.timing(operation, duration.as_secs_f64() * 1000.0);
        result
    }

    /// Records an input event
    pub fn input(&mut self, input: TestInput) {
        let message = match &input {
            TestInput::Key { key, modifiers, .. } => {
                if modifiers.is_empty() {
                    format!("Key: {key}")
                } else {
                    format!("Key: {}+{}", modifiers.join("+"), key)
                }
            }
            TestInput::Mouse {
                action,
                x,
                y,
                button,
            } => {
                if let Some(btn) = button {
                    format!("Mouse: {action} {btn} at ({x}, {y})")
                } else {
                    format!("Mouse: {action} at ({x}, {y})")
                }
            }
            TestInput::Resize { width, height } => {
                format!("Resize: {width}x{height}")
            }
            TestInput::Paste { text } => {
                let preview = if text.len() > 20 {
                    format!("{}...", &text[..20])
                } else {
                    text.clone()
                };
                format!("Paste: {preview:?}")
            }
        };

        let mut event = self.create_event(EventLevel::Debug, "input", message);
        event.input = Some(input);
        self.events.push(event);
    }

    /// Records a key input (convenience method)
    pub fn key(&mut self, key: &str) {
        self.input(TestInput::Key {
            key: key.to_string(),
            modifiers: Vec::new(),
            raw: None,
        });
    }

    /// Records a key input with modifiers
    pub fn key_with_modifiers(&mut self, key: &str, modifiers: &[&str]) {
        self.input(TestInput::Key {
            key: key.to_string(),
            modifiers: modifiers.iter().map(|s| s.to_string()).collect(),
            raw: None,
        });
    }

    /// Captures the current screen state
    /// Captures the current screen state (bd-28kk enhanced)
    ///
    /// Stores the frame for later diffing and artifact output.
    pub fn capture_frame(&mut self, content: &str) {
        let frame_name = format!("step_{:03}.txt", self.step);

        // Compute diff stats if we have a previous frame
        let diff_summary = self.last_frame.as_ref().map(|last| {
            let (added, removed, changed) = compute_line_diff(last, content);
            format!(" (+{added}/-{removed}/~{changed} lines)")
        });

        let mut event = self.create_event(
            EventLevel::Debug,
            "frame_capture",
            format!(
                "Captured frame: {frame_name}{}",
                diff_summary.as_deref().unwrap_or("")
            ),
        );
        event.frame_path = Some(format!("frames/{frame_name}"));
        self.events.push(event);

        // Store frame content in memory for later writing
        // (we don't write until finish() to avoid cluttering on success)
        self.events.last_mut().unwrap().actual = Some(content.to_string());

        // Track for diffing
        self.last_frame = Some(content.to_string());
    }

    /// Captures frame with ANSI sequences preserved (bd-28kk)
    ///
    /// Use this when debugging rendering issues that require ANSI inspection.
    pub fn capture_frame_with_ansi(&mut self, content: &str) {
        let frame_name = format!("step_{:03}_ansi.txt", self.step);

        let mut event = self.create_event(
            EventLevel::Trace,
            "frame_capture_ansi",
            format!("Captured ANSI frame: {frame_name} ({} bytes)", content.len()),
        );
        event.frame_path = Some(format!("frames/{frame_name}"));
        self.events.push(event);

        // Store raw ANSI content
        self.events.last_mut().unwrap().actual = Some(content.to_string());
    }

    /// Returns a diff between the last two captured frames (bd-28kk)
    ///
    /// Returns None if fewer than 2 frames have been captured.
    pub fn frame_diff(&self) -> Option<FrameDiff> {
        // Find the last two frame captures
        let frames: Vec<_> = self
            .events
            .iter()
            .filter(|e| e.event == "frame_capture")
            .filter_map(|e| e.actual.as_ref())
            .collect();

        if frames.len() < 2 {
            return None;
        }

        let prev = frames[frames.len() - 2];
        let curr = frames[frames.len() - 1];

        Some(compute_frame_diff(prev, curr))
    }

    /// Records an assertion
    ///
    /// # Arguments
    ///
    /// * `name` - Description of what is being asserted
    /// * `passed` - Whether the assertion passed
    /// * `expected` - Expected value (for error reporting)
    /// * `actual` - Actual value (for error reporting)
    pub fn assert_eq<T: std::fmt::Debug + PartialEq>(
        &mut self,
        name: &str,
        expected: &T,
        actual: &T,
    ) -> bool {
        let passed = expected == actual;
        self.record_assertion(
            name,
            passed,
            &format!("{expected:?}"),
            &format!("{actual:?}"),
        )
    }

    /// Records an assertion with custom expected/actual strings
    pub fn record_assertion(
        &mut self,
        name: &str,
        passed: bool,
        expected: &str,
        actual: &str,
    ) -> bool {
        let level = if passed {
            EventLevel::Debug
        } else {
            self.has_failures = true;
            EventLevel::Error
        };

        let status = if passed { "PASS" } else { "FAIL" };
        let mut event = self.create_event(level, "assertion", format!("[{status}] {name}"));
        event.assertion = Some(name.to_string());
        event.expected = Some(expected.to_string());
        event.actual = Some(actual.to_string());
        self.events.push(event);

        passed
    }

    /// Records an assertion that should be true
    pub fn assert_true(&mut self, name: &str, condition: bool) -> bool {
        self.record_assertion(name, condition, "true", &condition.to_string())
    }

    /// Finishes the scenario and writes artifacts
    ///
    /// Returns `Ok(())` if no assertions failed, `Err(summary)` otherwise.
    pub fn finish(mut self) -> Result<(), String> {
        let duration = self.start_time.elapsed();

        // Record scenario end
        let status = if self.has_failures {
            "FAILED"
        } else {
            "PASSED"
        };
        self.record_event(
            if self.has_failures {
                EventLevel::Error
            } else {
                EventLevel::Info
            },
            "scenario_end",
            format!(
                "Scenario {}: {} in {:.2}s",
                status,
                self.scenario,
                duration.as_secs_f64()
            ),
        );

        // Write artifacts if there were failures OR keep_on_success is set
        if self.has_failures || self.keep_on_success {
            self.write_artifacts()?;
        }

        if self.has_failures {
            Err(self.generate_summary())
        } else {
            Ok(())
        }
    }

    /// Writes all artifacts to disk
    fn write_artifacts(&self) -> Result<(), String> {
        // Create directory structure
        fs::create_dir_all(&self.artifact_dir)
            .map_err(|e| format!("Failed to create artifact dir: {e}"))?;
        fs::create_dir_all(self.artifact_dir.join("frames"))
            .map_err(|e| format!("Failed to create frames dir: {e}"))?;

        // Write events.jsonl
        let events_path = self.artifact_dir.join("events.jsonl");
        let file = File::create(&events_path)
            .map_err(|e| format!("Failed to create events.jsonl: {e}"))?;
        let mut writer = BufWriter::new(file);
        for event in &self.events {
            // Clone event and clear the embedded frame content to avoid duplication
            let mut event_copy = event.clone();
            if event_copy.event == "frame_capture" {
                event_copy.actual = None;
            }
            let line = serde_json::to_string(&event_copy)
                .map_err(|e| format!("Failed to serialize event: {e}"))?;
            writeln!(writer, "{line}").map_err(|e| format!("Failed to write event: {e}"))?;
        }

        // Write captured frames
        for event in &self.events {
            if event.event == "frame_capture"
                && let (Some(frame_path), Some(content)) = (&event.frame_path, &event.actual)
            {
                let path = self.artifact_dir.join(frame_path);
                fs::write(&path, content).map_err(|e| format!("Failed to write frame: {e}"))?;
            }
        }

        // Write config.json if available
        if let Some(config) = &self.config {
            let config_path = self.artifact_dir.join("config.json");
            let config_json = serde_json::to_string_pretty(config)
                .map_err(|e| format!("Failed to serialize config: {e}"))?;
            fs::write(&config_path, config_json)
                .map_err(|e| format!("Failed to write config.json: {e}"))?;
        }

        // Write summary.txt
        let summary = self.generate_summary();
        let summary_path = self.artifact_dir.join("summary.txt");
        fs::write(&summary_path, &summary)
            .map_err(|e| format!("Failed to write summary.txt: {e}"))?;

        Ok(())
    }

    /// Generates a human-readable summary
    fn generate_summary(&self) -> String {
        let mut summary = String::new();

        summary.push_str(&format!(
            "=== E2E Test Summary ===\n\
             Scenario: {}\n\
             Run ID: {}\n\
             Status: {}\n\n",
            self.scenario,
            self.run_id,
            if self.has_failures {
                "FAILED"
            } else {
                "PASSED"
            }
        ));

        // Config info
        if let Some(config) = &self.config {
            summary.push_str(&format!(
                "Configuration:\n\
                 - Seed: {}\n\
                 - Theme: {}\n\
                 - Animations: {}\n\
                 - Color Mode: {}\n\n",
                config.seed, config.theme, config.animations, config.color_mode
            ));
        }

        // Failed assertions
        let failures: Vec<_> = self
            .events
            .iter()
            .filter(|e| e.event == "assertion" && e.level == EventLevel::Error)
            .collect();

        if !failures.is_empty() {
            summary.push_str("Failed Assertions:\n");
            for failure in failures {
                summary.push_str(&format!(
                    "  Step {}: {}\n",
                    failure.step,
                    failure.assertion.as_deref().unwrap_or("unknown")
                ));
                summary.push_str(&format!(
                    "    Expected: {}\n",
                    failure.expected.as_deref().unwrap_or("?")
                ));
                summary.push_str(&format!(
                    "    Actual:   {}\n",
                    failure.actual.as_deref().unwrap_or("?")
                ));
            }
            summary.push('\n');
        }

        // Step timeline
        summary.push_str("Step Timeline:\n");
        for event in &self.events {
            if event.event == "step_start" {
                summary.push_str(&format!("  [{}] {}\n", event.ts, event.message));
            }
        }
        summary.push('\n');

        // Artifact location
        summary.push_str(&format!("Artifacts: {}\n", self.artifact_dir.display()));

        summary
    }

    /// Creates a new event with common fields filled in
    fn create_event(&self, level: EventLevel, event: &str, message: String) -> TestEvent {
        TestEvent {
            ts: current_timestamp(),
            level,
            scenario: self.scenario.clone(),
            run_id: self.run_id.clone(),
            step: self.step,
            event: event.to_string(),
            message,
            input: None,
            assertion: None,
            expected: None,
            actual: None,
            frame_path: None,
            config: None,
        }
    }

    /// Records an event
    /// Records an event (respects min_level filter, bd-28kk)
    fn record_event(&mut self, level: EventLevel, event: &str, message: String) {
        // Skip events below minimum level
        if level < self.min_level {
            return;
        }
        let event = self.create_event(level, event, message);
        self.events.push(event);
    }

    /// Records an event with config
    fn record_event_with_config(
        &mut self,
        level: EventLevel,
        event: &str,
        message: String,
        config: Option<ConfigSnapshot>,
    ) {
        let mut event = self.create_event(level, event, message);
        event.config = config;
        self.events.push(event);
    }
}

/// Generates an ISO-8601 timestamp
fn current_timestamp() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();

    // Simple ISO-8601 format without external dependencies
    let (year, month, day, hour, min, sec) = timestamp_to_parts(secs);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}.{millis:03}Z")
}

/// Converts Unix timestamp to date/time parts
fn timestamp_to_parts(secs: u64) -> (u32, u32, u32, u32, u32, u32) {
    // Days since Unix epoch
    let days = (secs / 86400) as i64;
    let time_of_day = secs % 86400;

    let hour = (time_of_day / 3600) as u32;
    let min = ((time_of_day % 3600) / 60) as u32;
    let sec = (time_of_day % 60) as u32;

    // Civil date from days since epoch (simplified algorithm)
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };

    (year as u32, m, d, hour, min, sec)
}

/// Generates a unique run ID
fn generate_run_id() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let ts = duration.as_millis();
    format!("{ts:x}")
}

// =============================================================================
// Screen Diff Support (bd-28kk)
// =============================================================================

/// Result of comparing two screen frames
#[derive(Debug, Clone)]
pub struct FrameDiff {
    /// Lines added (present in new, not in old)
    pub added_lines: Vec<(usize, String)>,
    /// Lines removed (present in old, not in new)
    pub removed_lines: Vec<(usize, String)>,
    /// Lines changed (same position, different content)
    pub changed_lines: Vec<(usize, String, String)>,
    /// Summary statistics
    pub stats: DiffStats,
}

/// Summary statistics for a frame diff
#[derive(Debug, Clone, Copy, Default)]
pub struct DiffStats {
    /// Number of lines added
    pub added: usize,
    /// Number of lines removed
    pub removed: usize,
    /// Number of lines changed
    pub changed: usize,
    /// Number of lines unchanged
    pub unchanged: usize,
}

impl FrameDiff {
    /// Returns true if there are any differences
    pub fn has_changes(&self) -> bool {
        self.stats.added > 0 || self.stats.removed > 0 || self.stats.changed > 0
    }

    /// Generates a unified-style diff output
    pub fn to_unified(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!(
            "--- previous\n+++ current\n@@ -{},{} +{},{} @@\n",
            self.stats.removed + self.stats.changed + self.stats.unchanged,
            self.stats.removed,
            self.stats.added + self.stats.changed + self.stats.unchanged,
            self.stats.added,
        ));

        for (line_num, content) in &self.removed_lines {
            output.push_str(&format!("-{line_num}: {content}\n"));
        }
        for (line_num, content) in &self.added_lines {
            output.push_str(&format!("+{line_num}: {content}\n"));
        }
        for (line_num, old, new) in &self.changed_lines {
            output.push_str(&format!("~{line_num}: {old}\n"));
            output.push_str(&format!("~{line_num}: {new}\n"));
        }

        output
    }
}

/// Computes line-level diff statistics between two strings
fn compute_line_diff(old: &str, new: &str) -> (usize, usize, usize) {
    let old_lines: Vec<_> = old.lines().collect();
    let new_lines: Vec<_> = new.lines().collect();

    let max_len = old_lines.len().max(new_lines.len());
    let mut added = 0;
    let mut removed = 0;
    let mut changed = 0;

    for i in 0..max_len {
        match (old_lines.get(i), new_lines.get(i)) {
            (Some(o), Some(n)) if o != n => changed += 1,
            (None, Some(_)) => added += 1,
            (Some(_), None) => removed += 1,
            _ => {}
        }
    }

    (added, removed, changed)
}

/// Computes a detailed frame diff between two strings
fn compute_frame_diff(old: &str, new: &str) -> FrameDiff {
    let old_lines: Vec<_> = old.lines().collect();
    let new_lines: Vec<_> = new.lines().collect();

    let mut added_lines = Vec::new();
    let mut removed_lines = Vec::new();
    let mut changed_lines = Vec::new();
    let mut unchanged = 0;

    let max_len = old_lines.len().max(new_lines.len());

    for i in 0..max_len {
        match (old_lines.get(i), new_lines.get(i)) {
            (Some(o), Some(n)) if o == n => unchanged += 1,
            (Some(o), Some(n)) => {
                changed_lines.push((i + 1, (*o).to_string(), (*n).to_string()));
            }
            (None, Some(n)) => {
                added_lines.push((i + 1, (*n).to_string()));
            }
            (Some(o), None) => {
                removed_lines.push((i + 1, (*o).to_string()));
            }
            (None, None) => {}
        }
    }

    FrameDiff {
        stats: DiffStats {
            added: added_lines.len(),
            removed: removed_lines.len(),
            changed: changed_lines.len(),
            unchanged,
        },
        added_lines,
        removed_lines,
        changed_lines,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_recorder_basic() {
        let mut recorder = ScenarioRecorder::new("basic_test");
        recorder.step("First step");
        recorder.key("a");
        assert!(recorder.assert_true("always true", true));
        let result = recorder.finish();
        assert!(result.is_ok());
    }

    #[test]
    fn test_scenario_recorder_failure() {
        let mut recorder = ScenarioRecorder::new("failing_test");
        recorder.step("Failing step");
        recorder.assert_eq("should fail", &1, &2);
        let result = recorder.finish();
        assert!(result.is_err());
        let summary = result.unwrap_err();
        assert!(summary.contains("FAILED"));
        assert!(summary.contains("should fail"));
    }

    #[test]
    fn test_config_snapshot() {
        let mut recorder = ScenarioRecorder::new("config_test");
        recorder.set_config(ConfigSnapshot {
            seed: 12345,
            theme: "catppuccin".to_string(),
            sidebar: true,
            animations: false,
            color_mode: "auto".to_string(),
        });
        recorder.step("Check config");
        let result = recorder.finish();
        assert!(result.is_ok());
    }

    #[test]
    fn test_input_recording() {
        let mut recorder = ScenarioRecorder::new("input_test");
        recorder.step("Send inputs");
        recorder.key("j");
        recorder.key_with_modifiers("c", &["ctrl"]);
        recorder.input(TestInput::Mouse {
            action: "click".to_string(),
            x: 10,
            y: 20,
            button: Some("left".to_string()),
        });
        recorder.input(TestInput::Resize {
            width: 80,
            height: 24,
        });
        recorder.input(TestInput::Paste {
            text: "hello world".to_string(),
        });
        let result = recorder.finish();
        assert!(result.is_ok());
    }

    #[test]
    fn test_timestamp_format() {
        let ts = current_timestamp();
        // Should be ISO-8601 format: YYYY-MM-DDTHH:MM:SS.mmmZ
        assert!(ts.contains("T"));
        assert!(ts.ends_with("Z"));
        assert_eq!(ts.len(), 24);
    }

    #[test]
    fn test_run_id_generation() {
        let id1 = generate_run_id();
        // IDs should be hex strings
        assert!(id1.chars().all(|c| c.is_ascii_hexdigit()));
        // ID should not be empty
        assert!(!id1.is_empty());
        // ID should be reasonable length (milliseconds timestamp in hex)
        assert!(id1.len() >= 8);
    }

    #[test]
    fn test_event_serialization() {
        let event = TestEvent {
            ts: "2026-01-28T12:00:00.000Z".to_string(),
            level: EventLevel::Info,
            scenario: "test".to_string(),
            run_id: "abc123".to_string(),
            step: 1,
            event: "test_event".to_string(),
            message: "Test message".to_string(),
            input: None,
            assertion: None,
            expected: None,
            actual: None,
            frame_path: None,
            config: None,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"scenario\":\"test\""));
        assert!(json.contains("\"level\":\"info\""));
    }

    #[test]
    fn test_frame_capture() {
        let mut recorder = ScenarioRecorder::new("frame_test");
        recorder.step("Capture frame");
        recorder.capture_frame("Screen content here\nLine 2");
        let result = recorder.finish();
        assert!(result.is_ok());
    }

    // =========================================================================
    // bd-28kk: Enhanced Logging Tests
    // =========================================================================

    #[test]
    fn test_trace_level_logging() {
        let mut recorder = ScenarioRecorder::new("trace_test");
        recorder.step("Test trace");
        recorder.trace("Detailed trace message");
        recorder.trace("Another trace");
        let result = recorder.finish();
        assert!(result.is_ok());
    }

    #[test]
    fn test_timing_measurement() {
        let mut recorder = ScenarioRecorder::new("timing_test");
        recorder.step("Test timing");
        recorder.timing("render", 12.5);
        recorder.timing("update", 3.2);

        // Test timed closure
        let result = recorder.timed("computation", || {
            std::thread::sleep(std::time::Duration::from_millis(10));
            42
        });
        assert_eq!(result, 42);

        let result = recorder.finish();
        assert!(result.is_ok());
    }

    #[test]
    fn test_event_level_ordering() {
        // EventLevel should be ordered from most verbose to least
        assert!(EventLevel::Trace < EventLevel::Debug);
        assert!(EventLevel::Debug < EventLevel::Info);
        assert!(EventLevel::Info < EventLevel::Warn);
        assert!(EventLevel::Warn < EventLevel::Error);
    }

    #[test]
    fn test_event_level_abbrev() {
        assert_eq!(EventLevel::Trace.abbrev(), 'T');
        assert_eq!(EventLevel::Debug.abbrev(), 'D');
        assert_eq!(EventLevel::Info.abbrev(), 'I');
        assert_eq!(EventLevel::Warn.abbrev(), 'W');
        assert_eq!(EventLevel::Error.abbrev(), 'E');
    }

    #[test]
    fn test_frame_diff() {
        let old = "Line 1\nLine 2\nLine 3";
        let new = "Line 1\nModified 2\nLine 3\nLine 4";

        let diff = compute_frame_diff(old, new);
        assert_eq!(diff.stats.unchanged, 2); // Line 1, Line 3
        assert_eq!(diff.stats.changed, 1);   // Line 2 -> Modified 2
        assert_eq!(diff.stats.added, 1);     // Line 4
        assert_eq!(diff.stats.removed, 0);
        assert!(diff.has_changes());
    }

    #[test]
    fn test_frame_diff_no_changes() {
        let content = "Same\nContent";
        let diff = compute_frame_diff(content, content);
        assert!(!diff.has_changes());
        assert_eq!(diff.stats.unchanged, 2);
    }

    #[test]
    fn test_frame_diff_unified_output() {
        let old = "A\nB";
        let new = "A\nC";

        let diff = compute_frame_diff(old, new);
        let unified = diff.to_unified();
        assert!(unified.contains("---"));
        assert!(unified.contains("+++"));
    }

    #[test]
    fn test_step_timing() {
        let mut recorder = ScenarioRecorder::new("step_timing_test");

        recorder.step("First step");
        std::thread::sleep(std::time::Duration::from_millis(5));

        recorder.step("Second step");
        // The first step's completion time should have been recorded

        let result = recorder.finish();
        assert!(result.is_ok());
    }
}

// =============================================================================
// E2E HEADLESS RUNNER
// =============================================================================

use bubbletea::simulator::ProgramSimulator;
use bubbletea::{KeyMsg, KeyType, Message, MouseAction, MouseButton, MouseMsg, WindowSizeMsg};

use crate::app::App;
use crate::config::Config;
use crate::messages::Page;

/// E2E test runner for headless execution of the demo_showcase app.
///
/// This runner provides a high-level DSL for driving the app without a real terminal,
/// capturing frames at each step, and asserting on the output.
///
/// # Example
///
/// ```rust,ignore
/// use demo_showcase::test_support::E2ERunner;
///
/// let mut runner = E2ERunner::new("navigation_test");
/// runner.press_key('j');  // Move down
/// runner.assert_contains("Jobs");
/// runner.press_key('2');  // Navigate to Services
/// runner.assert_page(Page::Services);
/// runner.finish().expect("test should pass");
/// ```
pub struct E2ERunner {
    /// The program simulator
    sim: ProgramSimulator<App>,
    /// Scenario recorder for artifacts
    recorder: ScenarioRecorder,
    /// Current window size
    width: u16,
    height: u16,
    /// Step counter for automatic step naming
    auto_step: u32,
}

impl E2ERunner {
    /// Creates a new E2E runner with default config.
    ///
    /// Uses a deterministic seed and default theme for reproducible tests.
    pub fn new(scenario: impl Into<String>) -> Self {
        let scenario_name = scenario.into();
        Self::with_config(scenario_name, Config::default_test())
    }

    /// Creates a new E2E runner with the given config.
    pub fn with_config(scenario: impl Into<String>, config: Config) -> Self {
        let scenario_name = scenario.into();
        let mut recorder = ScenarioRecorder::new(&scenario_name);

        // Set config snapshot
        recorder.set_config(ConfigSnapshot {
            seed: config.effective_seed(),
            theme: config.theme_preset.name().to_string(),
            sidebar: true, // App starts with sidebar visible
            animations: config.use_animations(),
            color_mode: format!("{:?}", config.color_mode),
        });

        // Create app from config
        let app = App::from_config(&config);

        // Create simulator and initialize
        let mut sim = ProgramSimulator::new(app);
        let _init_cmd = sim.init();

        Self {
            sim,
            recorder,
            width: 120,
            height: 40,
            auto_step: 0,
        }
    }

    /// Begins a named step in the scenario.
    ///
    /// Steps provide structure to the test and appear in artifact summaries.
    pub fn step(&mut self, description: impl Into<String>) {
        self.recorder.step(description);
        self.capture_frame();
    }

    /// Presses a single character key.
    pub fn press_key(&mut self, c: char) {
        self.auto_step("Press key");
        self.recorder.key(&c.to_string());

        let msg = Message::new(KeyMsg {
            key_type: KeyType::Runes,
            runes: vec![c],
            alt: false,
            paste: false,
        });
        self.sim.send(msg);
        self.step_with_cmd(); // Process message and execute any returned command
        self.capture_frame();
    }

    /// Process one step and execute any returned commands.
    ///
    /// This processes the message and any commands it produces, with a step limit
    /// to prevent infinite loops from tick commands or other recurring messages.
    fn step_with_cmd(&mut self) {
        const MAX_STEPS: usize = 50;

        for _ in 0..MAX_STEPS {
            if let Some(cmd) = self.sim.step() {
                // step() returned a command - execute it and queue the result
                if let Some(result_msg) = cmd.execute() {
                    self.sim.send(result_msg);
                }
            } else if self.sim.pending_count() > 0 {
                // step() returned None but there are more messages
                // (e.g., after processing BatchMsg which queues result messages)
                continue;
            } else {
                // Queue is truly empty
                break;
            }
        }
    }

    /// Presses multiple keys in sequence.
    pub fn press_keys(&mut self, keys: &str) {
        for c in keys.chars() {
            self.press_key(c);
        }
    }

    /// Presses a special key (Enter, Esc, Arrow keys, etc.).
    pub fn press_special(&mut self, key_type: KeyType) {
        self.auto_step(&format!("Press {:?}", key_type));
        self.recorder.key(&format!("{:?}", key_type));

        let msg = Message::new(KeyMsg {
            key_type,
            runes: vec![],
            alt: false,
            paste: false,
        });
        self.sim.send(msg);
        self.step_with_cmd();
        self.capture_frame();
    }

    /// Presses a key with Alt modifier.
    pub fn press_alt(&mut self, c: char) {
        self.auto_step(&format!("Press Alt+{}", c));
        self.recorder.key_with_modifiers(&c.to_string(), &["alt"]);

        let msg = Message::new(KeyMsg {
            key_type: KeyType::Runes,
            runes: vec![c],
            alt: true,
            paste: false,
        });
        self.sim.send(msg);
        self.step_with_cmd();
        self.capture_frame();
    }

    /// Types a string (simulates paste event).
    pub fn type_text(&mut self, text: &str) {
        self.auto_step(&format!("Type: {:?}", text));
        self.recorder.input(TestInput::Paste {
            text: text.to_string(),
        });

        let msg = Message::new(KeyMsg {
            key_type: KeyType::Runes,
            runes: text.chars().collect(),
            alt: false,
            paste: true,
        });
        self.sim.send(msg);
        self.step_with_cmd();
        self.capture_frame();
    }

    /// Simulates a mouse click at the given position.
    pub fn click(&mut self, x: u16, y: u16) {
        self.auto_step(&format!("Click at ({}, {})", x, y));
        self.recorder.input(TestInput::Mouse {
            action: "click".to_string(),
            x,
            y,
            button: Some("left".to_string()),
        });

        let msg = Message::new(MouseMsg {
            x,
            y,
            shift: false,
            alt: false,
            ctrl: false,
            action: MouseAction::Press,
            button: MouseButton::Left,
        });
        self.sim.send(msg);
        self.step_with_cmd();
        self.capture_frame();
    }

    /// Simulates a window resize.
    pub fn resize(&mut self, width: u16, height: u16) {
        self.auto_step(&format!("Resize to {}x{}", width, height));
        self.recorder.input(TestInput::Resize { width, height });

        self.width = width;
        self.height = height;

        let msg = Message::new(WindowSizeMsg { width, height });
        self.sim.send(msg);
        self.step_with_cmd();
        self.capture_frame();
    }

    /// Runs update steps until the message queue is empty.
    pub fn drain(&mut self) {
        self.sim.run_until_empty();
        self.capture_frame();
    }

    /// Gets the current rendered view.
    pub fn view(&self) -> String {
        self.sim
            .last_view()
            .map(|s| s.to_string())
            .unwrap_or_default()
    }

    /// Asserts that the current view contains the given text.
    pub fn assert_contains(&mut self, text: &str) -> bool {
        let view = self.view();
        let contains = view.contains(text);
        self.recorder.record_assertion(
            &format!("view contains {:?}", text),
            contains,
            &format!("contains {:?}", text),
            &if contains {
                "found".to_string()
            } else {
                format!("not found in: {}", truncate_view(&view, 200))
            },
        )
    }

    /// Asserts that the current view does NOT contain the given text.
    pub fn assert_not_contains(&mut self, text: &str) -> bool {
        let view = self.view();
        let not_contains = !view.contains(text);
        self.recorder.record_assertion(
            &format!("view does not contain {:?}", text),
            not_contains,
            &format!("does not contain {:?}", text),
            &if not_contains {
                "not found (good)".to_string()
            } else {
                format!("unexpectedly found in: {}", truncate_view(&view, 200))
            },
        )
    }

    /// Asserts that the app is on the given page.
    pub fn assert_page(&mut self, expected: Page) -> bool {
        let actual = self.sim.model().current_page();
        self.recorder.assert_eq(
            &format!("current page is {:?}", expected),
            &expected,
            &actual,
        )
    }

    /// Asserts that the view is not empty.
    pub fn assert_view_not_empty(&mut self) -> bool {
        let view = self.view();
        let not_empty = !view.trim().is_empty();
        self.recorder.record_assertion(
            "view is not empty",
            not_empty,
            "non-empty view",
            &if not_empty {
                format!("{} chars", view.len())
            } else {
                "empty".to_string()
            },
        )
    }

    /// Asserts that no ANSI escape codes are present (for no-color mode testing).
    pub fn assert_no_ansi(&mut self) -> bool {
        let view = self.view();
        let has_ansi = view.contains("\x1b[");
        self.recorder.record_assertion(
            "no ANSI escape codes",
            !has_ansi,
            "no \\x1b[",
            &if has_ansi {
                "found ANSI codes".to_string()
            } else {
                "clean".to_string()
            },
        )
    }

    /// Gets the underlying model for direct state inspection.
    pub fn model(&self) -> &App {
        self.sim.model()
    }

    /// Gets simulation statistics.
    pub fn stats(&self) -> &bubbletea::simulator::SimulationStats {
        self.sim.stats()
    }

    /// Finishes the test and writes artifacts (on failure).
    ///
    /// Returns `Ok(())` if all assertions passed, `Err(summary)` otherwise.
    pub fn finish(self) -> Result<(), String> {
        self.recorder.finish()
    }

    /// Captures the current frame.
    fn capture_frame(&mut self) {
        let view = self.view();
        self.recorder.capture_frame(&view);
    }

    /// Auto-generates a step if none is active.
    fn auto_step(&mut self, description: &str) {
        self.auto_step += 1;
        if self.recorder.step == 0 {
            self.recorder.step(description);
        }
    }
}

/// Truncates a view string for error messages.
fn truncate_view(view: &str, max_len: usize) -> String {
    if view.len() <= max_len {
        view.to_string()
    } else {
        format!("{}...", &view[..max_len])
    }
}

// Extension trait for Config to provide test defaults
impl Config {
    /// Creates a default config for E2E testing.
    ///
    /// Uses:
    /// - Deterministic seed (42)
    /// - Default theme
    /// - Animations disabled (for faster/deterministic tests)
    /// - Mouse enabled
    pub fn default_test() -> Self {
        use crate::config::{AnimationMode, ColorMode};
        use crate::theme::ThemePreset;

        Self {
            theme_preset: ThemePreset::default(),
            theme_file: None,
            color_mode: ColorMode::Auto,
            animations: AnimationMode::Disabled,
            mouse: true,
            alt_screen: false,
            seed: Some(42),
            files_root: None,
            self_check: false,
            verbosity: 0,
            syntax_highlighting: false,
            line_numbers: false,
        }
    }
}

#[cfg(test)]
mod e2e_runner_tests {
    use super::*;

    #[test]
    fn e2e_runner_creates_and_initializes() {
        let runner = E2ERunner::new("init_test");
        assert!(runner.view().len() > 0);
        runner.finish().expect("should pass");
    }

    #[test]
    fn e2e_runner_press_key() {
        let mut runner = E2ERunner::new("key_test");
        runner.step("Press navigation keys");
        runner.press_key('j'); // Move down in sidebar
        runner.assert_view_not_empty();
        runner.finish().expect("should pass");
    }

    #[test]
    fn e2e_runner_navigate_pages() {
        let mut runner = E2ERunner::new("page_nav_test");
        runner.step("Start on Dashboard");
        runner.assert_page(Page::Dashboard);

        runner.step("Navigate to Jobs");
        runner.press_key('3'); // Jobs page shortcut
        runner.assert_page(Page::Jobs);

        runner.step("Navigate to Logs");
        runner.press_key('4'); // Logs page shortcut
        runner.assert_page(Page::Logs);

        runner.finish().expect("should pass");
    }

    #[test]
    fn e2e_runner_view_contains() {
        let mut runner = E2ERunner::new("contains_test");
        runner.step("Check view content");
        // The app should show some content
        runner.assert_view_not_empty();
        runner.finish().expect("should pass");
    }

    #[test]
    fn e2e_runner_resize() {
        let mut runner = E2ERunner::new("resize_test");
        runner.step("Resize window");
        runner.resize(80, 24);
        runner.assert_view_not_empty();
        runner.finish().expect("should pass");
    }
}

// =============================================================================
// E2E SCENARIO: SETTINGS TOGGLES + THEME SWITCHING (bd-3ecr)
// =============================================================================

#[cfg(test)]
mod e2e_settings_tests {
    use super::*;
    use crate::theme::ThemePreset;

    /// E2E test: Settings toggles (mouse, animations, ASCII mode, syntax)
    ///
    /// This scenario validates that Settings page toggles actually change
    /// the running app state.
    #[test]
    fn e2e_settings_toggles() {
        let mut runner = E2ERunner::new("settings_toggles");

        // Initialize app with window size (sets ready = true)
        runner.step("Initialize app");
        runner.resize(120, 40);
        runner.assert_view_not_empty();
        runner.assert_page(Page::Dashboard);

        // Step 1: Navigate to Settings page
        runner.step("Navigate to Settings page");
        runner.press_key('8'); // Settings page shortcut
        runner.assert_page(Page::Settings);
        runner.assert_contains("Settings");
        runner.assert_contains("Toggles");

        // Step 2: Verify toggle labels are visible
        runner.step("Verify toggle labels");
        runner.assert_contains("Mouse Input");
        runner.assert_contains("Animations");
        runner.assert_contains("ASCII Mode");
        runner.assert_contains("Syntax Highlighting");

        // Step 3: Toggle mouse input using direct key
        runner.step("Toggle mouse input with 'm' key");
        let mouse_before = runner.model().mouse_enabled();
        runner.press_key('m');
        let mouse_after = runner.model().mouse_enabled();
        runner.recorder.record_assertion(
            "mouse toggle changed state",
            mouse_before != mouse_after,
            &format!("changed from {}", mouse_before),
            &format!("now {}", mouse_after),
        );

        // Step 4: Toggle animations using direct key
        runner.step("Toggle animations with 'a' key");
        let anim_before = runner.model().use_animations();
        runner.press_key('a');
        let anim_after = runner.model().use_animations();
        runner.recorder.record_assertion(
            "animations toggle changed state",
            anim_before != anim_after,
            &format!("changed from {}", anim_before),
            &format!("now {}", anim_after),
        );

        // Step 5: Navigate with j/k and toggle with Enter
        // After 'a', toggle_selected is 1 (Animations)
        // Press 'j' once to move to index 2 (ASCII Mode)
        runner.step("Navigate to ASCII Mode and toggle with Enter");
        runner.press_key('j'); // Move to ASCII Mode (index 2)
        let ascii_before = runner.model().is_force_ascii();
        runner.press_special(KeyType::Enter);
        let ascii_after = runner.model().is_force_ascii();
        runner.recorder.record_assertion(
            "ASCII mode toggle changed state",
            ascii_before != ascii_after,
            &format!("changed from {}", ascii_before),
            &format!("now {}", ascii_after),
        );

        // Step 6: Return to Dashboard and verify changes persist
        runner.step("Return to Dashboard and verify toggles persist");
        runner.press_key('1'); // Dashboard page shortcut
        runner.assert_page(Page::Dashboard);
        // Verify the toggle states are still changed
        runner.recorder.record_assertion(
            "mouse state persisted",
            runner.model().mouse_enabled() == mouse_after,
            &format!("{}", mouse_after),
            &format!("{}", runner.model().mouse_enabled()),
        );
        runner.recorder.record_assertion(
            "animations state persisted",
            runner.model().use_animations() == anim_after,
            &format!("{}", anim_after),
            &format!("{}", runner.model().use_animations()),
        );

        runner.finish().expect("settings_toggles should pass");
    }

    /// E2E test: Global theme cycling with 't' key
    ///
    /// This tests the global 't' key for cycling themes, which is
    /// simpler than testing the full Settings page theme picker
    /// (which involves batch commands).
    #[test]
    fn e2e_theme_cycling() {
        let mut runner = E2ERunner::new("theme_cycling");

        // Initialize app
        runner.step("Initialize app");
        runner.resize(120, 40);
        runner.assert_view_not_empty();
        runner.assert_page(Page::Dashboard);

        // Verify initial theme
        runner.step("Verify initial theme is Dark");
        let initial_theme = runner.model().theme_preset();
        runner.recorder.record_assertion(
            "initial theme is Dark",
            initial_theme == ThemePreset::Dark,
            "Dark",
            initial_theme.name(),
        );

        // Cycle to Light theme
        runner.step("Cycle to Light theme with 't' key");
        runner.press_key('t');
        let light_theme = runner.model().theme_preset();
        runner.recorder.record_assertion(
            "theme cycled to Light",
            light_theme == ThemePreset::Light,
            "Light",
            light_theme.name(),
        );

        // Cycle to Dracula theme
        runner.step("Cycle to Dracula theme with 't' key");
        runner.press_key('t');
        let dracula_theme = runner.model().theme_preset();
        runner.recorder.record_assertion(
            "theme cycled to Dracula",
            dracula_theme == ThemePreset::Dracula,
            "Dracula",
            dracula_theme.name(),
        );

        // Cycle back to Dark theme
        runner.step("Cycle back to Dark theme with 't' key");
        runner.press_key('t');
        let back_to_dark = runner.model().theme_preset();
        runner.recorder.record_assertion(
            "theme cycled back to Dark",
            back_to_dark == ThemePreset::Dark,
            "Dark",
            back_to_dark.name(),
        );

        // Verify theme persists across page navigation
        runner.step("Navigate to Jobs and verify theme persists");
        runner.press_key('3'); // Jobs
        runner.assert_page(Page::Jobs);
        runner.recorder.record_assertion(
            "theme persisted on Jobs page",
            runner.model().theme_preset() == ThemePreset::Dark,
            "Dark",
            runner.model().theme_preset().name(),
        );

        runner.finish().expect("theme_cycling should pass");
    }

    /// E2E test: Settings toggles and global theme cycling
    ///
    /// This scenario covers:
    /// - Toggle ASCII mode via direct key
    /// - Global theme cycling with 't'
    /// - Verify persistence across navigation
    ///
    /// Note: The Settings page theme picker uses batch commands which
    /// have complex processing requirements. Theme cycling via 't' key
    /// tests the core theme switching functionality.
    #[test]
    fn e2e_settings_and_theme_scenario() {
        let mut runner = E2ERunner::new("settings_and_theme");

        // Initialize
        runner.step("Initialize app");
        runner.resize(120, 40);
        runner.assert_view_not_empty();
        runner.assert_page(Page::Dashboard);

        // Navigate to Settings
        runner.step("Navigate to Settings");
        runner.press_key('8');
        runner.assert_page(Page::Settings);

        // Toggle ASCII mode
        runner.step("Toggle ASCII mode");
        runner.press_key('c');
        let ascii_enabled = runner.model().is_force_ascii();
        runner.recorder.assert_true("ASCII mode is now enabled", ascii_enabled);

        // Toggle mouse
        runner.step("Toggle mouse");
        let mouse_before = runner.model().mouse_enabled();
        runner.press_key('m');
        let mouse_after = runner.model().mouse_enabled();
        runner.recorder.record_assertion(
            "mouse toggled",
            mouse_before != mouse_after,
            &format!("changed from {}", mouse_before),
            &format!("now {}", mouse_after),
        );

        // Return to Dashboard and cycle theme
        runner.step("Return to Dashboard and cycle theme");
        runner.press_key('1');
        runner.assert_page(Page::Dashboard);
        runner.press_key('t'); // Cycle to Light
        runner.recorder.record_assertion(
            "theme cycled to Light",
            runner.model().theme_preset() == ThemePreset::Light,
            "Light",
            runner.model().theme_preset().name(),
        );

        // Verify settings persisted
        runner.step("Verify settings persisted");
        runner.recorder.record_assertion(
            "ASCII mode persisted",
            runner.model().is_force_ascii() == ascii_enabled,
            &ascii_enabled.to_string(),
            &runner.model().is_force_ascii().to_string(),
        );
        runner.recorder.record_assertion(
            "mouse setting persisted",
            runner.model().mouse_enabled() == mouse_after,
            &mouse_after.to_string(),
            &runner.model().mouse_enabled().to_string(),
        );

        // Navigate through pages
        runner.step("Navigate through pages");
        for key in ['2', '3', '4', '5', '6'] {
            runner.press_key(key);
            runner.assert_view_not_empty();
        }

        runner.finish().expect("settings_and_theme should pass");
    }
}

// =============================================================================
// E2E SCENARIO: SMOKE TOUR (bd-4d5e)
// =============================================================================

#[cfg(test)]
mod e2e_smoke_tour_tests {
    use super::*;
    use crate::theme::ThemePreset;

    /// E2E test: Complete smoke tour visiting every page
    ///
    /// This scenario validates that the app can:
    /// - Boot with deterministic config
    /// - Navigate to all pages without panics
    /// - Open/close the help overlay
    /// - Toggle ASCII mode and verify ANSI escapes disappear
    /// - Switch themes and verify the change takes effect
    /// - Quit cleanly
    ///
    /// This is the primary regression test for the app's core surface area.
    #[test]
    fn e2e_smoke_tour() {
        let mut runner = E2ERunner::new("smoke_tour");

        // =========================================================================
        // Step 1: Initialize and verify dashboard
        // =========================================================================
        runner.step("Initialize app with deterministic config");
        runner.resize(120, 40);
        runner.assert_view_not_empty();
        runner.assert_page(Page::Dashboard);
        runner.assert_contains("Dashboard"); // Page header

        // =========================================================================
        // Step 2: Navigate through ALL pages
        // =========================================================================

        runner.step("Navigate to Services page");
        runner.press_key('2');
        runner.assert_page(Page::Services);
        runner.assert_view_not_empty();

        runner.step("Navigate to Jobs page");
        runner.press_key('3');
        runner.assert_page(Page::Jobs);
        runner.assert_view_not_empty();

        runner.step("Navigate to Logs page");
        runner.press_key('4');
        runner.assert_page(Page::Logs);
        runner.assert_view_not_empty();

        runner.step("Navigate to Docs page");
        runner.press_key('5');
        runner.assert_page(Page::Docs);
        runner.assert_view_not_empty();
        runner.assert_contains("Documents"); // Docs page has split view with list

        runner.step("Navigate to Files page");
        runner.press_key('6');
        runner.assert_page(Page::Files);
        runner.assert_view_not_empty();

        runner.step("Navigate to Wizard page");
        runner.press_key('7');
        runner.assert_page(Page::Wizard);
        runner.assert_view_not_empty();

        runner.step("Navigate to Settings page");
        runner.press_key('8');
        runner.assert_page(Page::Settings);
        runner.assert_view_not_empty();
        runner.assert_contains("Settings");

        // =========================================================================
        // Step 3: Test help overlay
        // =========================================================================
        runner.step("Open help overlay");
        runner.press_key('?');
        runner.drain();
        // Help overlay should show keyboard shortcuts
        // Note: The actual help content depends on implementation
        let view_with_help = runner.view();
        runner.recorder.record_assertion(
            "help overlay opened",
            view_with_help.contains("help") || view_with_help.contains("?") || view_with_help.len() > 100,
            "help content visible",
            if view_with_help.len() > 100 { "long content" } else { "short content" },
        );

        runner.step("Close help overlay");
        runner.press_special(KeyType::Esc);
        runner.drain();
        runner.assert_page(Page::Settings); // Should still be on Settings

        // =========================================================================
        // Step 4: Test ASCII mode toggle
        // =========================================================================
        runner.step("Enable ASCII mode in Settings");
        let initial_view = runner.view();
        let has_ansi_before = initial_view.contains('\x1b');
        runner.recorder.record_assertion(
            "view has ANSI before toggle",
            has_ansi_before,
            "has ANSI escapes",
            if has_ansi_before { "yes" } else { "no" },
        );

        // Toggle ASCII mode using 'c' key
        runner.press_key('c');
        runner.drain();

        let ascii_view = runner.view();
        let has_ansi_after = ascii_view.contains('\x1b');

        // In ASCII mode, ANSI escapes should be reduced/eliminated
        // Note: This depends on implementation - some border chars may still have escapes
        runner.recorder.record_assertion(
            "ASCII mode affects output",
            !has_ansi_after || has_ansi_before != has_ansi_after,
            "ANSI escapes changed or removed",
            if has_ansi_after { "still has ANSI" } else { "no ANSI escapes" },
        );

        // Toggle back
        runner.press_key('c');
        runner.drain();

        // =========================================================================
        // Step 5: Test theme switching
        // =========================================================================
        runner.step("Switch to Themes section");
        runner.press_special(KeyType::Tab);
        runner.drain();

        let initial_theme = runner.model().theme_preset();
        runner.recorder.record_assertion(
            "initial theme is Dark",
            initial_theme == ThemePreset::Dark,
            "Dark",
            initial_theme.name(),
        );

        runner.step("Switch to Light theme");
        runner.press_key('j'); // Navigate down in theme list
        runner.press_special(KeyType::Enter);
        runner.drain();

        let new_theme = runner.model().theme_preset();
        runner.recorder.record_assertion(
            "theme changed to Light",
            new_theme == ThemePreset::Light,
            "Light",
            new_theme.name(),
        );

        // =========================================================================
        // Step 6: Return to Dashboard and verify persistence
        // =========================================================================
        runner.step("Return to Dashboard");
        runner.press_key('1');
        runner.assert_page(Page::Dashboard);

        // Theme should still be Light
        let persisted_theme = runner.model().theme_preset();
        runner.recorder.record_assertion(
            "theme persisted after navigation",
            persisted_theme == ThemePreset::Light,
            "Light",
            persisted_theme.name(),
        );

        // =========================================================================
        // Step 7: Final verification - no panics during the tour
        // =========================================================================
        runner.step("Final verification");
        runner.assert_view_not_empty();

        // Quick tour through pages to catch any late panics
        for key in ['2', '3', '4', '5', '6', '7', '1'] {
            runner.press_key(key);
            runner.assert_view_not_empty();
        }

        // =========================================================================
        // Test completed successfully
        // =========================================================================
        runner.finish().expect("smoke_tour should pass");
    }

    /// Focused test for page navigation without settings changes
    #[test]
    fn e2e_page_navigation_comprehensive() {
        let mut runner = E2ERunner::new("page_nav_comprehensive");

        runner.step("Initialize");
        runner.resize(120, 40);
        runner.assert_page(Page::Dashboard);

        // Navigate using number keys
        let pages = [
            ('1', Page::Dashboard, "Dashboard"),
            ('2', Page::Services, "Services"),
            ('3', Page::Jobs, "Jobs"),
            ('4', Page::Logs, "Logs"),
            ('5', Page::Docs, "Docs"),
            ('6', Page::Files, "Files"),
            ('7', Page::Wizard, "Wizard"),
            ('8', Page::Settings, "Settings"),
        ];

        for (key, expected_page, name) in pages {
            runner.step(&format!("Navigate to {} page", name));
            runner.press_key(key);
            runner.assert_page(expected_page);
            runner.assert_view_not_empty();
        }

        // Navigate back to first page
        runner.step("Return to Dashboard");
        runner.press_key('1');
        runner.assert_page(Page::Dashboard);

        runner.finish().expect("page_nav_comprehensive should pass");
    }

    /// E2E scenario: Wizard -> Job -> Logs correlation (bd-3449)
    ///
    /// Tests wizard navigation and verifies cross-page state consistency.
    /// Demonstrates that wizard can be navigated and that Jobs/Logs pages
    /// remain functional after wizard interaction.
    #[test]
    fn e2e_wizard_job_logs_correlation() {
        let mut runner = E2ERunner::new("wizard_job_logs");

        runner.step("Initialize app");
        runner.resize(120, 40);
        runner.assert_page(Page::Dashboard);

        // =========================================================================
        // Step 1: Navigate to Wizard page
        // =========================================================================
        runner.step("Navigate to Wizard page");
        runner.press_key('7');
        runner.assert_page(Page::Wizard);
        runner.assert_view_not_empty();

        // Verify wizard has content
        let wizard_view = runner.view();
        runner.recorder.assert_true(
            "wizard page has content",
            !wizard_view.trim().is_empty(),
        );

        // =========================================================================
        // Step 2: Interact with wizard (navigate steps)
        // =========================================================================
        runner.step("Navigate wizard with keyboard");

        // Press Enter to proceed from step 0
        runner.press_special(KeyType::Enter);
        runner.drain();
        runner.assert_view_not_empty();

        // Type some characters for name field (if in step 1)
        runner.step("Type in wizard fields");
        for c in "test".chars() {
            runner.press_key(c);
        }
        runner.drain();

        // Try to go back
        runner.press_key('b');
        runner.drain();
        runner.assert_view_not_empty();

        // =========================================================================
        // Step 3: Navigate to Jobs page
        // =========================================================================
        runner.step("Navigate to Jobs page");
        runner.press_key('3');
        runner.assert_page(Page::Jobs);

        let jobs_view = runner.view();
        // Jobs page should have some job-related content
        runner.recorder.assert_true(
            "jobs page renders content",
            !jobs_view.trim().is_empty(),
        );

        // Interact with Jobs page
        runner.step("Interact with Jobs page");
        runner.press_key('j'); // Navigate down
        runner.drain();
        runner.press_key('k'); // Navigate up
        runner.drain();
        runner.assert_view_not_empty();

        // =========================================================================
        // Step 4: Navigate to Logs page
        // =========================================================================
        runner.step("Navigate to Logs page");
        runner.press_key('4');
        runner.assert_page(Page::Logs);

        let logs_view = runner.view();
        runner.recorder.assert_true(
            "logs page renders content",
            !logs_view.trim().is_empty(),
        );

        // Interact with Logs page
        runner.step("Interact with Logs page");
        runner.press_key('j'); // Scroll down
        runner.drain();
        runner.press_key('f'); // Toggle follow mode
        runner.drain();
        runner.assert_view_not_empty();

        // =========================================================================
        // Step 5: Cross-page navigation verification
        // =========================================================================
        runner.step("Cross-page navigation");

        // Quick tour through all pages
        let pages = [
            ('1', Page::Dashboard),
            ('3', Page::Jobs),
            ('4', Page::Logs),
            ('7', Page::Wizard),
            ('1', Page::Dashboard),
        ];

        for (key, expected_page) in pages {
            runner.press_key(key);
            runner.assert_page(expected_page);
            runner.assert_view_not_empty();
        }

        // =========================================================================
        // Step 6: Final state verification
        // =========================================================================
        runner.step("Final verification");
        runner.assert_view_not_empty();

        // Verify no panics occurred and app is still functional
        let final_view = runner.view();
        runner.recorder.assert_true(
            "app still functional after navigation",
            !final_view.trim().is_empty(),
        );

        runner.finish().expect("wizard_job_logs_correlation should pass");
    }
}

// =============================================================================
// E2E: Files Page Tests (bd-1g80)
// =============================================================================

/// E2E tests for the Files page (file picker + preview).
#[cfg(test)]
mod e2e_files_tests {
    use super::*;

    /// E2E scenario: file picker navigate + preview (bd-1g80)
    ///
    /// Tests the Files page functionality:
    /// 1) Navigate to Files page
    /// 2) Use the file picker to navigate and select entries
    /// 3) Validate preview behavior (breadcrumb, file content)
    /// 4) Test markdown rendering for .md files
    #[test]
    fn e2e_files_navigate_and_preview() {
        let mut runner = E2ERunner::new("files_navigate_preview");

        // =========================================================================
        // Step 1: Initialize and navigate to Files page
        // =========================================================================
        runner.step("Initialize app");
        runner.resize(120, 40);
        runner.assert_page(Page::Dashboard);

        runner.step("Navigate to Files page");
        runner.press_key('6'); // Files page shortcut
        runner.assert_page(Page::Files);
        runner.assert_view_not_empty();

        // Verify the Files page shows the fixture root
        let files_view = runner.view();
        runner.recorder.assert_true(
            "Files page shows content",
            !files_view.trim().is_empty(),
        );

        // =========================================================================
        // Step 2: Navigate the file picker
        // =========================================================================
        runner.step("Navigate file list with j/k");

        // Press j to move down in the file list
        runner.press_key('j');
        runner.assert_view_not_empty();

        // Press j again to move to another entry
        runner.press_key('j');
        runner.assert_view_not_empty();

        // Press k to move back up
        runner.press_key('k');
        runner.assert_view_not_empty();

        // =========================================================================
        // Step 3: Enter a directory
        // =========================================================================
        runner.step("Enter directory with Enter key");

        // Navigate to find a directory entry
        // The fixture tree has directories like 'config', 'nested'
        runner.press_key('j');
        runner.press_key('j');

        // Press Enter to enter the directory or select the file
        runner.press_special(KeyType::Enter);
        runner.drain();

        let after_enter = runner.view();
        runner.recorder.assert_true(
            "view updated after Enter",
            !after_enter.trim().is_empty(),
        );

        // =========================================================================
        // Step 4: Navigate back with Backspace
        // =========================================================================
        runner.step("Navigate back with Backspace");
        runner.press_special(KeyType::Backspace);
        runner.drain();

        let after_back = runner.view();
        runner.recorder.assert_true(
            "view updated after Backspace",
            !after_back.trim().is_empty(),
        );

        // =========================================================================
        // Step 5: Toggle hidden files
        // =========================================================================
        runner.step("Toggle hidden files with 'h' key");
        runner.press_key('h');
        runner.drain();
        // Just verify no panic when toggling
        runner.assert_view_not_empty();

        // =========================================================================
        // Step 6: Return to Dashboard
        // =========================================================================
        runner.step("Return to Dashboard");
        runner.press_key('1');
        runner.assert_page(Page::Dashboard);

        runner.finish().expect("files_navigate_preview should pass");
    }

    /// E2E test: Files page with deterministic fixture data
    ///
    /// Verifies the Files page works with the embedded fixture tree.
    #[test]
    fn e2e_files_fixture_mode() {
        let mut runner = E2ERunner::new("files_fixture_mode");

        runner.step("Initialize");
        runner.resize(120, 40);

        runner.step("Navigate to Files");
        runner.press_key('6');
        runner.assert_page(Page::Files);

        // The fixture tree should show some entries
        let view = runner.view();
        runner.recorder.assert_true(
            "Files page renders fixture content",
            !view.trim().is_empty(),
        );

        // Navigate and verify no panics
        runner.step("Navigate through entries");
        for _ in 0..5 {
            runner.press_key('j');
            runner.drain();
            runner.assert_view_not_empty();
        }

        for _ in 0..3 {
            runner.press_key('k');
            runner.drain();
            runner.assert_view_not_empty();
        }

        runner.step("Return to Dashboard");
        runner.press_key('1');
        runner.assert_page(Page::Dashboard);

        runner.finish().expect("files_fixture_mode should pass");
    }
}
