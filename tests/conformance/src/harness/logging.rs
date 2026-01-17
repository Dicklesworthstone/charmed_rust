//! TestLogger - Hierarchical logging for conformance tests
//!
//! Provides detailed, formatted output for test execution including:
//! - Timestamps (ISO8601)
//! - Log level filtering (Debug, Info, Warn, Error)
//! - Hierarchical indentation for nested sections
//! - Colored output for terminal (when enabled)
//! - JSON output format for CI integration
//! - Thread-safe logging via `SharedLogger`
//! - ANSI escape sequence debugging
//!
//! # Example
//!
//! ```rust,ignore
//! use charmed_conformance::harness::logging::{TestLogger, LogLevel, OutputFormat};
//!
//! let mut logger = TestLogger::new()
//!     .with_level(LogLevel::Debug)
//!     .with_timestamps(true)
//!     .with_colors(true);
//!
//! logger.set_test_name("my_crate::my_test");
//! logger.info("Starting test");
//! logger.section("Inputs", |log| {
//!     log.key_value("param1", &42);
//!     log.key_value("param2", &"hello");
//! });
//! ```

use parking_lot::Mutex;
use serde::Serialize;
use std::fmt::{Debug, Display};
use std::io::{self, Write};
use std::sync::Arc;
use std::time::{Duration, Instant};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Log level for test output
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub enum LogLevel {
    /// Detailed trace information
    Trace = 0,
    /// Debug information
    Debug = 1,
    /// Standard informational messages
    #[default]
    Info = 2,
    /// Warning messages
    Warn = 3,
    /// Error messages
    Error = 4,
}

impl LogLevel {
    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }

    /// Get the color for this level
    pub fn color(&self) -> Option<Color> {
        match self {
            LogLevel::Trace => Some(Color::Magenta),
            LogLevel::Debug => Some(Color::Blue),
            LogLevel::Info => Some(Color::Green),
            LogLevel::Warn => Some(Color::Yellow),
            LogLevel::Error => Some(Color::Red),
        }
    }
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Output format for the logger
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// Human-readable format with indentation
    #[default]
    Human,
    /// JSON format for CI parsing
    Json,
}

/// JSON log entry for structured output
#[derive(Debug, Serialize)]
struct JsonLogEntry<'a> {
    timestamp: Option<String>,
    level: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    test: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    section: Option<&'a str>,
    message: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    key: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    passed: Option<bool>,
}

impl<'a> JsonLogEntry<'a> {
    fn new(level: &'a str, message: &'a str) -> Self {
        Self {
            timestamp: None,
            level,
            test: None,
            section: None,
            message,
            key: None,
            value: None,
            duration_ms: None,
            passed: None,
        }
    }
}

/// Output writer that can be colored or plain
enum OutputWriter {
    /// Standard stream with color support
    Colored(StandardStream),
    /// Plain writer (for testing)
    Plain(Box<dyn Write + Send>),
}

impl OutputWriter {
    fn write_colored(&mut self, spec: &ColorSpec, text: &str) -> io::Result<()> {
        match self {
            OutputWriter::Colored(stream) => {
                stream.set_color(spec)?;
                write!(stream, "{}", text)?;
                stream.reset()?;
                Ok(())
            }
            OutputWriter::Plain(writer) => {
                write!(writer, "{}", text)
            }
        }
    }

    fn write_plain(&mut self, text: &str) -> io::Result<()> {
        match self {
            OutputWriter::Colored(stream) => write!(stream, "{}", text),
            OutputWriter::Plain(writer) => write!(writer, "{}", text),
        }
    }

    fn newline(&mut self) -> io::Result<()> {
        match self {
            OutputWriter::Colored(stream) => writeln!(stream),
            OutputWriter::Plain(writer) => writeln!(writer),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            OutputWriter::Colored(stream) => stream.flush(),
            OutputWriter::Plain(writer) => writer.flush(),
        }
    }
}

/// Hierarchical logger for conformance tests
pub struct TestLogger {
    /// Minimum log level to output
    level: LogLevel,
    /// Output destination
    output: OutputWriter,
    /// Output format (Human or JSON)
    format: OutputFormat,
    /// Current indentation level
    indent: usize,
    /// Whether to include timestamps
    timestamps: bool,
    /// Whether to use colors
    colors: bool,
    /// Current test name
    test_name: Option<String>,
    /// Current section path
    section_stack: Vec<String>,
    /// Start time for duration calculation
    start_time: Instant,
    /// Per-test timing start
    timing_start: Option<Instant>,
}

impl Default for TestLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl TestLogger {
    /// Create a new logger with default settings (stdout, Info level)
    pub fn new() -> Self {
        Self {
            level: LogLevel::Info,
            output: OutputWriter::Colored(StandardStream::stdout(ColorChoice::Auto)),
            format: OutputFormat::Human,
            indent: 0,
            timestamps: true,
            colors: true,
            test_name: None,
            section_stack: Vec::new(),
            start_time: Instant::now(),
            timing_start: None,
        }
    }

    /// Set the minimum log level
    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    /// Set the output format
    pub fn with_format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }

    /// Set whether to include timestamps
    pub fn with_timestamps(mut self, timestamps: bool) -> Self {
        self.timestamps = timestamps;
        self
    }

    /// Set whether to use colors
    pub fn with_colors(mut self, colors: bool) -> Self {
        self.colors = colors;
        if colors {
            self.output = OutputWriter::Colored(StandardStream::stdout(ColorChoice::Auto));
        } else {
            self.output = OutputWriter::Colored(StandardStream::stdout(ColorChoice::Never));
        }
        self
    }

    /// Set a custom output destination (disables colors)
    pub fn with_output<W: Write + Send + 'static>(mut self, output: W) -> Self {
        self.output = OutputWriter::Plain(Box::new(output));
        self.colors = false;
        self
    }

    /// Set the current test name (appears in all log lines)
    pub fn set_test_name(&mut self, name: &str) {
        self.test_name = Some(name.to_string());
    }

    /// Clear the current test name
    pub fn clear_test_name(&mut self) {
        self.test_name = None;
    }

    /// Get current timestamp string
    fn timestamp_str(&self) -> Option<String> {
        if self.timestamps {
            let elapsed = self.start_time.elapsed();
            Some(format!("{:>8.3}ms", elapsed.as_secs_f64() * 1000.0))
        } else {
            None
        }
    }

    /// Get current section path as string
    fn section_path(&self) -> Option<String> {
        if self.section_stack.is_empty() {
            None
        } else {
            Some(self.section_stack.join("::"))
        }
    }

    /// Write a log entry
    fn write_log(&mut self, level: LogLevel, message: &str) {
        if level < self.level {
            return;
        }

        match self.format {
            OutputFormat::Human => self.write_human_log(level, message),
            OutputFormat::Json => self.write_json_log(level, message),
        }
    }

    /// Write human-readable log entry
    fn write_human_log(&mut self, level: LogLevel, message: &str) {
        let indent_str = "  ".repeat(self.indent);

        // Timestamp
        if let Some(ts) = self.timestamp_str() {
            let _ = self.output.write_plain(&format!("[{}] ", ts));
        }

        // Level with color
        let level_str = format!("[{}]", level.as_str());
        if self.colors {
            if let Some(color) = level.color() {
                let mut spec = ColorSpec::new();
                spec.set_fg(Some(color)).set_bold(level == LogLevel::Error);
                let _ = self.output.write_colored(&spec, &level_str);
            } else {
                let _ = self.output.write_plain(&level_str);
            }
        } else {
            let _ = self.output.write_plain(&level_str);
        }

        // Test name
        if let Some(ref test_name) = self.test_name {
            let _ = self.output.write_plain(&format!(" {}", test_name));
        }

        // Message with indentation
        let _ = self.output.write_plain(&format!(" {}{}", indent_str, message));
        let _ = self.output.newline();
        let _ = self.output.flush();
    }

    /// Write JSON log entry
    fn write_json_log(&mut self, level: LogLevel, message: &str) {
        let mut entry = JsonLogEntry::new(level.as_str(), message);
        entry.timestamp = self.timestamp_str();
        entry.test = self.test_name.as_deref();
        let section_path = self.section_path();
        entry.section = section_path.as_deref();

        if let Ok(json) = serde_json::to_string(&entry) {
            let _ = self.output.write_plain(&json);
            let _ = self.output.newline();
            let _ = self.output.flush();
        }
    }

    /// Log an info message
    pub fn info(&mut self, message: &str) {
        self.write_log(LogLevel::Info, message);
    }

    /// Log a debug message
    pub fn debug(&mut self, message: &str) {
        self.write_log(LogLevel::Debug, message);
    }

    /// Log a warning message
    pub fn warn(&mut self, message: &str) {
        self.write_log(LogLevel::Warn, message);
    }

    /// Log an error message
    pub fn error(&mut self, message: &str) {
        self.write_log(LogLevel::Error, message);
    }

    /// Log a trace message
    pub fn trace(&mut self, message: &str) {
        self.write_log(LogLevel::Trace, message);
    }

    /// Log a key-value pair
    pub fn key_value<K: Display, V: Debug>(&mut self, key: K, value: &V) {
        let message = format!("{}: {:?}", key, value);
        self.write_log(LogLevel::Info, &message);
    }

    /// Log a key-value pair with raw string value (no Debug formatting)
    pub fn key_value_raw<K: Display>(&mut self, key: K, value: &str) {
        let message = format!("{}: {}", key, value);
        self.write_log(LogLevel::Info, &message);
    }

    /// Log an input value
    pub fn log_input<T: Debug>(&mut self, name: &str, value: &T) {
        self.key_value(format!("Input {}", name), value);
    }

    /// Log an expected value
    pub fn log_expected<T: Debug>(&mut self, name: &str, value: &T) {
        self.key_value(format!("Expected {}", name), value);
    }

    /// Log an actual value
    pub fn log_actual<T: Debug>(&mut self, name: &str, value: &T) {
        self.key_value(format!("Actual {}", name), value);
    }

    /// Log an ANSI string with escape sequence debugging
    pub fn ansi_debug(&mut self, name: &str, ansi_str: &str) {
        self.section(name, |log| {
            // Raw representation
            log.key_value_raw("Raw", &format!("{:?}", ansi_str));

            // Parse and describe escape codes
            let mut codes_desc = String::new();
            let mut in_escape = false;
            let mut escape_buf = String::new();
            let mut text_buf = String::new();

            for c in ansi_str.chars() {
                if c == '\x1b' {
                    if !text_buf.is_empty() {
                        codes_desc.push_str(&format!("\"{}\" ", text_buf));
                        text_buf.clear();
                    }
                    in_escape = true;
                    escape_buf.clear();
                    escape_buf.push(c);
                } else if in_escape {
                    escape_buf.push(c);
                    if c.is_ascii_alphabetic() {
                        // End of escape sequence
                        let desc = Self::describe_ansi_escape(&escape_buf);
                        codes_desc.push_str(&format!("[{}] ", desc));
                        in_escape = false;
                    }
                } else {
                    text_buf.push(c);
                }
            }
            if !text_buf.is_empty() {
                codes_desc.push_str(&format!("\"{}\"", text_buf));
            }

            log.key_value_raw("Codes", &codes_desc);
        });
    }

    /// Describe an ANSI escape sequence
    fn describe_ansi_escape(escape: &str) -> String {
        if escape.starts_with("\x1b[") && escape.ends_with('m') {
            // SGR sequence
            let codes_str = &escape[2..escape.len() - 1];
            let codes: Vec<&str> = codes_str.split(';').collect();
            let descriptions: Vec<String> = codes
                .iter()
                .map(|code| {
                    match *code {
                        "0" => "reset".to_string(),
                        "1" => "bold".to_string(),
                        "2" => "dim".to_string(),
                        "3" => "italic".to_string(),
                        "4" => "underline".to_string(),
                        "5" => "blink".to_string(),
                        "7" => "reverse".to_string(),
                        "9" => "strikethrough".to_string(),
                        "30" => "black".to_string(),
                        "31" => "red".to_string(),
                        "32" => "green".to_string(),
                        "33" => "yellow".to_string(),
                        "34" => "blue".to_string(),
                        "35" => "magenta".to_string(),
                        "36" => "cyan".to_string(),
                        "37" => "white".to_string(),
                        "40" => "bg-black".to_string(),
                        "41" => "bg-red".to_string(),
                        "42" => "bg-green".to_string(),
                        "43" => "bg-yellow".to_string(),
                        "44" => "bg-blue".to_string(),
                        "45" => "bg-magenta".to_string(),
                        "46" => "bg-cyan".to_string(),
                        "47" => "bg-white".to_string(),
                        _ => format!("SGR {}", code),
                    }
                })
                .collect();
            format!("SGR {}", descriptions.join("+"))
        } else {
            format!("ESC{}", &escape[1..])
        }
    }

    /// Increase indentation for a nested section
    pub fn indent(&mut self) {
        self.indent += 1;
    }

    /// Decrease indentation after a nested section
    pub fn dedent(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }

    /// Execute a closure within a named section
    pub fn section<F, R>(&mut self, name: &str, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        // Log section header
        let header = format!("{}:", name);
        if self.colors {
            let mut spec = ColorSpec::new();
            spec.set_fg(Some(Color::Cyan)).set_bold(true);
            let _ = self.output.write_colored(&spec, &header);
            let _ = self.output.newline();
        } else {
            self.info(&header);
        }

        self.section_stack.push(name.to_string());
        self.indent();
        let result = f(self);
        self.dedent();
        self.section_stack.pop();
        result
    }

    /// Start timing for the current test
    pub fn start_timing(&mut self) {
        self.timing_start = Some(Instant::now());
    }

    /// Stop timing and return the duration
    pub fn stop_timing(&mut self) -> Duration {
        self.timing_start
            .take()
            .map(|start| start.elapsed())
            .unwrap_or(Duration::ZERO)
    }

    /// Log a passing result
    pub fn log_pass(&mut self, duration: Duration) {
        match self.format {
            OutputFormat::Human => {
                let msg = format!("Result: PASS ({:.3}ms)", duration.as_secs_f64() * 1000.0);
                if self.colors {
                    let mut spec = ColorSpec::new();
                    spec.set_fg(Some(Color::Green)).set_bold(true);
                    let _ = self.output.write_colored(&spec, &msg);
                    let _ = self.output.newline();
                } else {
                    self.info(&msg);
                }
            }
            OutputFormat::Json => {
                let mut entry = JsonLogEntry::new("INFO", "Result");
                entry.timestamp = self.timestamp_str();
                entry.test = self.test_name.as_deref();
                entry.passed = Some(true);
                entry.duration_ms = Some(duration.as_secs_f64() * 1000.0);
                if let Ok(json) = serde_json::to_string(&entry) {
                    let _ = self.output.write_plain(&json);
                    let _ = self.output.newline();
                }
            }
        }
        let _ = self.output.flush();
    }

    /// Log a failing result
    pub fn log_fail(&mut self, reason: &str, duration: Duration) {
        match self.format {
            OutputFormat::Human => {
                let msg = format!(
                    "Result: FAIL ({:.3}ms) - {}",
                    duration.as_secs_f64() * 1000.0,
                    reason
                );
                if self.colors {
                    let mut spec = ColorSpec::new();
                    spec.set_fg(Some(Color::Red)).set_bold(true);
                    let _ = self.output.write_colored(&spec, &msg);
                    let _ = self.output.newline();
                } else {
                    self.error(&msg);
                }
            }
            OutputFormat::Json => {
                let mut entry = JsonLogEntry::new("ERROR", reason);
                entry.timestamp = self.timestamp_str();
                entry.test = self.test_name.as_deref();
                entry.passed = Some(false);
                entry.duration_ms = Some(duration.as_secs_f64() * 1000.0);
                if let Ok(json) = serde_json::to_string(&entry) {
                    let _ = self.output.write_plain(&json);
                    let _ = self.output.newline();
                }
            }
        }
        let _ = self.output.flush();
    }

    /// Log progress for long test suites
    pub fn progress(&mut self, current: usize, total: usize, test_name: &str) {
        let pct = if total > 0 {
            (current * 100) / total
        } else {
            0
        };
        let msg = format!("[{}/{}] {}% - {}", current, total, pct, test_name);
        self.info(&msg);
    }
}

/// Thread-safe logger wrapper for parallel tests
#[derive(Clone)]
pub struct SharedLogger {
    inner: Arc<Mutex<TestLogger>>,
}

impl SharedLogger {
    /// Create a new shared logger wrapping the given logger
    pub fn new(logger: TestLogger) -> Self {
        Self {
            inner: Arc::new(Mutex::new(logger)),
        }
    }

    /// Lock the logger for exclusive access
    pub fn lock(&self) -> parking_lot::MutexGuard<'_, TestLogger> {
        self.inner.lock()
    }

    /// Convenience method to log info
    pub fn info(&self, message: &str) {
        self.inner.lock().info(message);
    }

    /// Convenience method to log debug
    pub fn debug(&self, message: &str) {
        self.inner.lock().debug(message);
    }

    /// Convenience method to log warning
    pub fn warn(&self, message: &str) {
        self.inner.lock().warn(message);
    }

    /// Convenience method to log error
    pub fn error(&self, message: &str) {
        self.inner.lock().error(message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// A thread-safe buffer for testing that implements Write
    #[derive(Clone)]
    struct TestBuffer {
        inner: Arc<parking_lot::Mutex<Vec<u8>>>,
    }

    impl TestBuffer {
        fn new() -> Self {
            Self {
                inner: Arc::new(parking_lot::Mutex::new(Vec::new())),
            }
        }

        fn to_string(&self) -> String {
            String::from_utf8(self.inner.lock().clone()).unwrap()
        }
    }

    impl Write for TestBuffer {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.inner.lock().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_basic_logging() {
        let buffer = TestBuffer::new();
        let mut logger = TestLogger::new()
            .with_output(buffer.clone())
            .with_timestamps(false);

        logger.info("Test started");
        logger.key_value("input", &42);
        logger.key_value("expected", &"hello");

        let output = buffer.to_string();
        assert!(output.contains("Test started"));
        assert!(output.contains("input: 42"));
        assert!(output.contains("expected: \"hello\""));
    }

    #[test]
    fn test_hierarchical_sections() {
        let buffer = TestBuffer::new();
        let mut logger = TestLogger::new()
            .with_output(buffer.clone())
            .with_timestamps(false);

        logger.section("Outer", |log| {
            log.info("In outer");
            log.section("Inner", |log| {
                log.info("In inner");
            });
        });

        let output = buffer.to_string();
        assert!(output.contains("Outer:"));
        assert!(output.contains("In outer"));
        assert!(output.contains("Inner:"));
        assert!(output.contains("In inner"));
    }

    #[test]
    fn test_json_output() {
        let buffer = TestBuffer::new();
        let mut logger = TestLogger::new()
            .with_output(buffer.clone())
            .with_format(OutputFormat::Json)
            .with_timestamps(false);

        logger.set_test_name("my_test");
        logger.info("test message");

        let output = buffer.to_string();
        assert!(output.contains("\"test\":\"my_test\""));
        assert!(output.contains("\"message\":\"test message\""));
    }

    #[test]
    fn test_timing() {
        let mut logger = TestLogger::new().with_output(std::io::sink());
        logger.start_timing();
        std::thread::sleep(Duration::from_millis(10));
        let duration = logger.stop_timing();
        assert!(duration >= Duration::from_millis(10));
    }

    #[test]
    fn test_level_filtering() {
        let buffer = TestBuffer::new();
        let mut logger = TestLogger::new()
            .with_output(buffer.clone())
            .with_level(LogLevel::Warn)
            .with_timestamps(false);

        logger.debug("debug message");
        logger.info("info message");
        logger.warn("warn message");
        logger.error("error message");

        let output = buffer.to_string();
        assert!(!output.contains("debug message"));
        assert!(!output.contains("info message"));
        assert!(output.contains("warn message"));
        assert!(output.contains("error message"));
    }

    #[test]
    fn test_thread_safety() {
        let logger = SharedLogger::new(TestLogger::new().with_output(std::io::sink()));

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let logger = logger.clone();
                std::thread::spawn(move || {
                    for j in 0..100 {
                        logger.info(&format!("Thread {} msg {}", i, j));
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }
        // No panics = success
    }

    #[test]
    fn test_ansi_debug() {
        let buffer = TestBuffer::new();
        let mut logger = TestLogger::new()
            .with_output(buffer.clone())
            .with_timestamps(false);

        logger.ansi_debug("styled", "\x1b[31;1mHello\x1b[0m");

        let output = buffer.to_string();
        assert!(output.contains("red"));
        assert!(output.contains("bold"));
    }

    #[test]
    fn test_log_pass_fail() {
        let buffer = TestBuffer::new();
        let mut logger = TestLogger::new()
            .with_output(buffer.clone())
            .with_timestamps(false);

        logger.log_pass(Duration::from_millis(5));
        logger.log_fail("assertion failed", Duration::from_millis(10));

        let output = buffer.to_string();
        assert!(output.contains("PASS"));
        assert!(output.contains("FAIL"));
        assert!(output.contains("assertion failed"));
    }

    #[test]
    fn test_progress() {
        let buffer = TestBuffer::new();
        let mut logger = TestLogger::new()
            .with_output(buffer.clone())
            .with_timestamps(false);

        logger.progress(5, 10, "test_foo");

        let output = buffer.to_string();
        assert!(output.contains("[5/10]"));
        assert!(output.contains("50%"));
        assert!(output.contains("test_foo"));
    }
}
