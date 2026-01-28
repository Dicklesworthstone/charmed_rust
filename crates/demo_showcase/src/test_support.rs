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

/// Event levels for logging
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EventLevel {
    /// Informational events
    Info,
    /// Debug-level details
    Debug,
    /// Warnings (non-fatal issues)
    Warn,
    /// Errors (assertion failures)
    Error,
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

/// Records events and artifacts for a single E2E test scenario
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
    /// Config snapshot
    config: Option<ConfigSnapshot>,
    /// Keep artifacts on success (env: DEMO_SHOWCASE_KEEP_ARTIFACTS)
    keep_on_success: bool,
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

        let mut recorder = Self {
            scenario: scenario.clone(),
            run_id: run_id.clone(),
            step: 0,
            step_description: String::new(),
            events: Vec::new(),
            artifact_dir,
            has_failures: false,
            start_time: Instant::now(),
            config: None,
            keep_on_success,
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
        self.step += 1;
        self.step_description = description.into();
        self.record_event(
            EventLevel::Info,
            "step_start",
            format!("Step {}: {}", self.step, self.step_description),
        );
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
            TestInput::Mouse { action, x, y, button } => {
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
    pub fn capture_frame(&mut self, content: &str) {
        let frame_name = format!("step_{:03}.txt", self.step);

        let mut event = self.create_event(
            EventLevel::Debug,
            "frame_capture",
            format!("Captured frame: {frame_name}"),
        );
        event.frame_path = Some(format!("frames/{frame_name}"));
        self.events.push(event);

        // Store frame content in memory for later writing
        // (we don't write until finish() to avoid cluttering on success)
        self.events.last_mut().unwrap().actual = Some(content.to_string());
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
        self.record_assertion(name, passed, &format!("{expected:?}"), &format!("{actual:?}"))
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
        let mut event = self.create_event(
            level,
            "assertion",
            format!("[{status}] {name}"),
        );
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
        let status = if self.has_failures { "FAILED" } else { "PASSED" };
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
                fs::write(&path, content)
                    .map_err(|e| format!("Failed to write frame: {e}"))?;
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
            if self.has_failures { "FAILED" } else { "PASSED" }
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
        summary.push_str(&format!(
            "Artifacts: {}\n",
            self.artifact_dir.display()
        ));

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
    fn record_event(&mut self, level: EventLevel, event: &str, message: String) {
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
}
