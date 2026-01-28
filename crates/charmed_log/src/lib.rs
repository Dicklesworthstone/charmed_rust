#![forbid(unsafe_code)]
// Allow pedantic lints for early-stage API ergonomics.
#![allow(clippy::nursery)]
#![allow(clippy::pedantic)]

//! # Charmed Log
//!
//! A structured logging library designed for terminal applications.
//!
//! Charmed Log provides beautiful, structured logging output with support for:
//! - Multiple log levels (trace, debug, info, warn, error, fatal)
//! - Structured key-value pairs
//! - Multiple output formatters (text, JSON, logfmt)
//! - Integration with lipgloss for styled output
//!
//! ## Example
//!
//! ```rust
//! use charmed_log::{Logger, Level};
//!
//! let logger = Logger::new();
//! logger.info("Application started", &[("version", "1.0.0")]);
//! ```
//!
//! ## Formatters
//!
//! - **Text**: Human-readable colored output (default)
//! - **JSON**: Machine-readable JSON output
//! - **Logfmt**: Key=value format for log aggregation

use backtrace::Backtrace;
use lipgloss::{Color, Style};
use std::collections::HashMap;
use std::fmt;
use std::io::{self, Write};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Log level for filtering messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum Level {
    /// Debug level (most verbose).
    Debug = -4,
    /// Info level (default).
    Info = 0,
    /// Warning level.
    Warn = 4,
    /// Error level.
    Error = 8,
    /// Fatal level (least verbose).
    Fatal = 12,
}

impl Level {
    /// Returns the string representation of the level.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
            Self::Fatal => "fatal",
        }
    }

    /// Returns the uppercase string representation of the level.
    #[must_use]
    pub fn as_upper_str(&self) -> &'static str {
        match self {
            Self::Debug => "DEBU",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERRO",
            Self::Fatal => "FATA",
        }
    }
}

impl PartialOrd for Level {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Level {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (*self as i32).cmp(&(*other as i32))
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for Level {
    type Err = ParseLevelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "debug" => Ok(Self::Debug),
            "info" => Ok(Self::Info),
            "warn" => Ok(Self::Warn),
            "error" => Ok(Self::Error),
            "fatal" => Ok(Self::Fatal),
            _ => Err(ParseLevelError(s.to_string())),
        }
    }
}

/// Error returned when parsing an invalid log level string.
///
/// This error occurs when calling [`Level::from_str`] with a string
/// that doesn't match any known log level.
///
/// # Valid Level Strings
///
/// The following strings are accepted (case-insensitive):
/// - `"debug"`
/// - `"info"`
/// - `"warn"`
/// - `"error"`
/// - `"fatal"`
///
/// # Example
///
/// ```rust
/// use charmed_log::Level;
/// use std::str::FromStr;
///
/// assert!(Level::from_str("info").is_ok());
/// assert!(Level::from_str("INFO").is_ok());
/// assert!(Level::from_str("invalid").is_err());
/// ```
#[derive(Error, Debug, Clone)]
#[error("invalid level: {0:?}")]
pub struct ParseLevelError(String);

/// A specialized [`Result`] type for level parsing operations.
pub type ParseResult<T> = std::result::Result<T, ParseLevelError>;

/// Output formatter type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Formatter {
    /// Human-readable text format (default).
    #[default]
    Text,
    /// JSON format.
    Json,
    /// Logfmt key=value format.
    Logfmt,
}

/// Standard keys used in log records.
pub mod keys {
    /// Key for timestamp.
    pub const TIMESTAMP: &str = "time";
    /// Key for message.
    pub const MESSAGE: &str = "msg";
    /// Key for level.
    pub const LEVEL: &str = "level";
    /// Key for caller location.
    pub const CALLER: &str = "caller";
    /// Key for prefix.
    pub const PREFIX: &str = "prefix";
}

/// Default time format.
pub const DEFAULT_TIME_FORMAT: &str = "%Y/%m/%d %H:%M:%S";

/// Styles for the text formatter.
#[derive(Debug, Clone)]
pub struct Styles {
    /// Style for timestamps.
    pub timestamp: Style,
    /// Style for caller location.
    pub caller: Style,
    /// Style for prefix.
    pub prefix: Style,
    /// Style for messages.
    pub message: Style,
    /// Style for keys.
    pub key: Style,
    /// Style for values.
    pub value: Style,
    /// Style for separators.
    pub separator: Style,
    /// Styles for each level.
    pub levels: HashMap<Level, Style>,
    /// Custom styles for specific keys.
    pub keys: HashMap<String, Style>,
    /// Custom styles for specific values.
    pub values: HashMap<String, Style>,
}

impl Default for Styles {
    fn default() -> Self {
        Self::new()
    }
}

impl Styles {
    /// Creates a new Styles with default values.
    #[must_use]
    pub fn new() -> Self {
        let mut levels = HashMap::new();
        levels.insert(
            Level::Debug,
            Style::new().bold().foreground_color(Color::from("63")),
        );
        levels.insert(
            Level::Info,
            Style::new().bold().foreground_color(Color::from("86")),
        );
        levels.insert(
            Level::Warn,
            Style::new().bold().foreground_color(Color::from("192")),
        );
        levels.insert(
            Level::Error,
            Style::new().bold().foreground_color(Color::from("204")),
        );
        levels.insert(
            Level::Fatal,
            Style::new().bold().foreground_color(Color::from("134")),
        );

        Self {
            timestamp: Style::new(),
            caller: Style::new().faint(),
            prefix: Style::new().bold().faint(),
            message: Style::new(),
            key: Style::new().faint(),
            value: Style::new(),
            separator: Style::new().faint(),
            levels,
            keys: HashMap::new(),
            values: HashMap::new(),
        }
    }
}

/// Type alias for time function.
pub type TimeFunction = fn(std::time::SystemTime) -> std::time::SystemTime;

/// Returns the time in UTC.
#[must_use]
pub fn now_utc(t: SystemTime) -> SystemTime {
    t // SystemTime is already timezone-agnostic
}

/// Type alias for caller formatter.
pub type CallerFormatter = fn(&str, u32, &str) -> String;

/// Type alias for error handler callback.
///
/// The error handler is called when an I/O error occurs during log writing.
/// This allows applications to respond to logging failures (e.g., disk full,
/// pipe closed, permission denied) instead of silently losing log messages.
///
/// # Example
///
/// ```rust
/// use charmed_log::Logger;
///
/// let logger = Logger::new().with_error_handler(|err| {
///     // Alert monitoring system, attempt fallback, etc.
///     eprintln!("charmed_log: write failed: {}", err);
/// });
/// ```
pub type ErrorHandler = Arc<dyn Fn(io::Error) + Send + Sync>;

/// Short caller formatter - returns last 2 path segments and line.
#[must_use]
pub fn short_caller_formatter(file: &str, line: u32, _fn_name: &str) -> String {
    let trimmed = trim_caller_path(file, 2);
    format!("{trimmed}:{line}")
}

/// Long caller formatter - returns full path and line.
#[must_use]
pub fn long_caller_formatter(file: &str, line: u32, _fn_name: &str) -> String {
    format!("{file}:{line}")
}

/// Trims a path to the last n segments.
fn trim_caller_path(path: &str, n: usize) -> &str {
    if n == 0 {
        return path;
    }

    let mut last_idx = path.len();
    for _ in 0..n {
        if let Some(idx) = path[..last_idx].rfind('/') {
            last_idx = idx;
        } else {
            return path;
        }
    }

    &path[last_idx + 1..]
}

/// Caller information extracted from the call stack.
#[derive(Debug, Clone)]
pub struct CallerInfo {
    /// Source file path.
    pub file: String,
    /// Line number.
    pub line: u32,
    /// Function name.
    pub function: String,
}

impl CallerInfo {
    /// Extracts caller information from the current call stack.
    ///
    /// The `skip` parameter indicates how many frames to skip from the
    /// logging infrastructure to find the actual caller.
    ///
    /// # Performance Warning
    ///
    /// This method captures a full stack backtrace and performs symbol
    /// resolution, which is a very expensive operation (~100μs or more).
    /// Avoid calling this in hot paths or production code.
    ///
    /// Typical overhead: **100-1000x** slower than a normal log call.
    #[must_use]
    pub fn capture(skip: usize) -> Option<Self> {
        let bt = Backtrace::new();
        let frames: Vec<_> = bt.frames().iter().collect();

        // Skip frames from backtrace crate + our own logging infrastructure
        // Typical stack: backtrace::capture -> CallerInfo::capture -> log -> debug/info/etc -> user code
        let skip_total = skip + 4;

        for frame in frames.iter().skip(skip_total) {
            for symbol in frame.symbols() {
                // Get function name and filter out internal frames
                let fn_name = symbol
                    .name()
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "<unknown>".to_string());

                // Skip frames from logging crate itself
                if fn_name.contains("charmed_log::") || fn_name.contains("backtrace::") {
                    continue;
                }

                let file = symbol
                    .filename()
                    .and_then(|p| p.to_str())
                    .unwrap_or("<unknown>")
                    .to_string();

                let line = symbol.lineno().unwrap_or(0);

                return Some(Self {
                    file,
                    line,
                    function: fn_name,
                });
            }
        }

        None
    }
}

/// Logger options.
#[derive(Clone)]
pub struct Options {
    /// Time function for the logger.
    pub time_function: TimeFunction,
    /// Time format string.
    pub time_format: String,
    /// Minimum log level.
    pub level: Level,
    /// Log prefix.
    pub prefix: String,
    /// Whether to report timestamps.
    pub report_timestamp: bool,
    /// Whether to report caller location.
    ///
    /// # Performance Warning
    ///
    /// When enabled, captures a full stack backtrace on **every** log call,
    /// which is approximately **100-1000x slower** than normal logging.
    /// Only enable during active debugging sessions. Do NOT enable in production.
    pub report_caller: bool,
    /// Caller formatter function.
    pub caller_formatter: CallerFormatter,
    /// Caller offset for stack trace.
    pub caller_offset: usize,
    /// Default fields to include in all logs.
    pub fields: Vec<(String, String)>,
    /// Output formatter.
    pub formatter: Formatter,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            time_function: now_utc,
            time_format: DEFAULT_TIME_FORMAT.to_string(),
            level: Level::Info,
            prefix: String::new(),
            report_timestamp: false,
            report_caller: false,
            caller_formatter: short_caller_formatter,
            caller_offset: 0,
            fields: Vec::new(),
            formatter: Formatter::Text,
        }
    }
}

/// Internal logger state.
struct LoggerInner {
    writer: Box<dyn Write + Send + Sync>,
    level: Level,
    prefix: String,
    time_function: TimeFunction,
    time_format: String,
    caller_offset: usize,
    caller_formatter: CallerFormatter,
    formatter: Formatter,
    report_timestamp: bool,
    report_caller: bool,
    fields: Vec<(String, String)>,
    styles: Styles,
    /// Optional error handler for I/O failures during logging.
    error_handler: Option<ErrorHandler>,
    /// Whether we've already warned about I/O failures (to prevent infinite loops).
    has_warned_io_failure: bool,
    /// Whether we've already warned about caller reporting overhead.
    warned_caller_overhead: bool,
    /// Whether to suppress the caller overhead warning.
    suppress_caller_warning: bool,
}

/// A structured logger instance.
pub struct Logger {
    inner: Arc<RwLock<LoggerInner>>,
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Logger {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl fmt::Debug for Logger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = self.inner.read().unwrap_or_else(|e| e.into_inner());
        f.debug_struct("Logger")
            .field("level", &inner.level)
            .field("prefix", &inner.prefix)
            .field("formatter", &inner.formatter)
            .field("report_timestamp", &inner.report_timestamp)
            .field("report_caller", &inner.report_caller)
            .finish()
    }
}

impl Logger {
    /// Creates a new logger with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::with_options(Options::default())
    }

    /// Creates a new logger with the given options.
    #[must_use]
    pub fn with_options(opts: Options) -> Self {
        Self {
            inner: Arc::new(RwLock::new(LoggerInner {
                writer: Box::new(io::stderr()),
                level: opts.level,
                prefix: opts.prefix,
                time_function: opts.time_function,
                time_format: opts.time_format,
                caller_offset: opts.caller_offset,
                caller_formatter: opts.caller_formatter,
                formatter: opts.formatter,
                report_timestamp: opts.report_timestamp,
                report_caller: opts.report_caller,
                fields: opts.fields,
                styles: Styles::new(),
                error_handler: None,
                has_warned_io_failure: false,
                warned_caller_overhead: false,
                suppress_caller_warning: false,
            })),
        }
    }

    /// Sets the minimum log level.
    pub fn set_level(&self, level: Level) {
        let mut inner = self.inner.write().unwrap_or_else(|e| e.into_inner());
        inner.level = level;
    }

    /// Returns the current log level.
    #[must_use]
    pub fn level(&self) -> Level {
        let inner = self.inner.read().unwrap_or_else(|e| e.into_inner());
        inner.level
    }

    /// Sets the log prefix.
    pub fn set_prefix(&self, prefix: impl Into<String>) {
        let mut inner = self.inner.write().unwrap_or_else(|e| e.into_inner());
        inner.prefix = prefix.into();
    }

    /// Returns the current prefix.
    #[must_use]
    pub fn prefix(&self) -> String {
        let inner = self.inner.read().unwrap_or_else(|e| e.into_inner());
        inner.prefix.clone()
    }

    /// Sets whether to report timestamps.
    pub fn set_report_timestamp(&self, report: bool) {
        let mut inner = self.inner.write().unwrap_or_else(|e| e.into_inner());
        inner.report_timestamp = report;
    }

    /// Sets whether to report caller location.
    ///
    /// # Performance Warning
    ///
    /// When enabled, this captures a full stack backtrace on **every** log call,
    /// which is approximately **100-1000x slower** than normal logging.
    ///
    /// | Configuration        | Typical Latency | Use Case       |
    /// |---------------------|-----------------|----------------|
    /// | Default (no caller) | ~100 ns         | Production     |
    /// | With caller         | ~100 μs         | Debug only     |
    ///
    /// **Only enable during active debugging sessions.**
    /// **Do NOT enable in production.**
    ///
    /// A runtime warning will be emitted on the first log call with caller
    /// reporting enabled (unless suppressed via [`suppress_caller_warning`]).
    ///
    /// [`suppress_caller_warning`]: Logger::suppress_caller_warning
    pub fn set_report_caller(&self, report: bool) {
        let mut inner = self.inner.write().unwrap_or_else(|e| e.into_inner());
        inner.report_caller = report;
    }

    /// Suppresses the runtime performance warning for caller reporting.
    ///
    /// By default, when caller reporting is enabled, a warning is emitted to
    /// stderr on the first log call to alert developers about the significant
    /// performance overhead (~100-1000x slower).
    ///
    /// Call this method to suppress the warning when you have intentionally
    /// enabled caller reporting and understand the performance implications.
    ///
    /// # Example
    ///
    /// ```rust
    /// use charmed_log::Logger;
    ///
    /// let logger = Logger::new();
    /// logger.set_report_caller(true);
    /// logger.suppress_caller_warning();
    /// // No warning will be emitted on first log call
    /// logger.info("debug message", &[]);
    /// ```
    pub fn suppress_caller_warning(&self) {
        let mut inner = self.inner.write().unwrap_or_else(|e| e.into_inner());
        inner.suppress_caller_warning = true;
    }

    /// Sets the time format.
    pub fn set_time_format(&self, format: impl Into<String>) {
        let mut inner = self.inner.write().unwrap_or_else(|e| e.into_inner());
        inner.time_format = format.into();
    }

    /// Sets the formatter.
    pub fn set_formatter(&self, formatter: Formatter) {
        let mut inner = self.inner.write().unwrap_or_else(|e| e.into_inner());
        inner.formatter = formatter;
    }

    /// Sets the styles.
    pub fn set_styles(&self, styles: Styles) {
        let mut inner = self.inner.write().unwrap_or_else(|e| e.into_inner());
        inner.styles = styles;
    }

    /// Creates a new logger with additional fields.
    ///
    /// This is the idiomatic Rust method name. For Go API compatibility,
    /// use [`with`](Logger::with) instead.
    #[must_use]
    pub fn with_fields(&self, fields: &[(&str, &str)]) -> Self {
        let inner = self.inner.read().unwrap_or_else(|e| e.into_inner());
        let mut new_fields = inner.fields.clone();
        new_fields.extend(fields.iter().map(|(k, v)| (k.to_string(), v.to_string())));

        Self {
            inner: Arc::new(RwLock::new(LoggerInner {
                writer: Box::new(io::stderr()),
                level: inner.level,
                prefix: inner.prefix.clone(),
                time_function: inner.time_function,
                time_format: inner.time_format.clone(),
                caller_offset: inner.caller_offset,
                caller_formatter: inner.caller_formatter,
                formatter: inner.formatter,
                report_timestamp: inner.report_timestamp,
                report_caller: inner.report_caller,
                fields: new_fields,
                styles: inner.styles.clone(),
                error_handler: inner.error_handler.clone(),
                has_warned_io_failure: false, // Reset warning state for new logger
                warned_caller_overhead: false, // Reset for new logger
                suppress_caller_warning: inner.suppress_caller_warning, // Inherit suppression
            })),
        }
    }

    /// Creates a new logger with additional fields (Go API compatibility).
    ///
    /// This method matches the Go `log.With()` API. It is equivalent to
    /// [`with_fields`](Logger::with_fields).
    ///
    /// # Example
    ///
    /// ```rust
    /// use charmed_log::Logger;
    ///
    /// let logger = Logger::new();
    /// let ctx_logger = logger.with(&[("request_id", "abc123"), ("user", "alice")]);
    /// ctx_logger.info("Processing request", &[]);
    /// ```
    #[must_use]
    pub fn with(&self, fields: &[(&str, &str)]) -> Self {
        self.with_fields(fields)
    }

    /// Creates a new logger with a different prefix.
    #[must_use]
    pub fn with_prefix(&self, prefix: impl Into<String>) -> Self {
        let new_logger = self.with_fields(&[]);
        new_logger.set_prefix(prefix);
        new_logger
    }

    /// Sets an error handler for I/O failures during logging.
    ///
    /// When writing log output fails (e.g., disk full, pipe closed, permission
    /// denied), the handler is called with the I/O error. This allows applications
    /// to respond appropriately instead of silently losing log messages.
    ///
    /// # Default Behavior
    ///
    /// If no error handler is configured:
    /// - First failure: A warning is printed to stderr (if available)
    /// - Subsequent failures: Silent (to avoid infinite loops)
    ///
    /// # Example
    ///
    /// ```rust
    /// use charmed_log::Logger;
    /// use std::sync::atomic::{AtomicUsize, Ordering};
    /// use std::sync::Arc;
    ///
    /// let error_count = Arc::new(AtomicUsize::new(0));
    /// let counter = error_count.clone();
    ///
    /// let logger = Logger::new().with_error_handler(move |err| {
    ///     counter.fetch_add(1, Ordering::Relaxed);
    ///     eprintln!("Log write failed: {}", err);
    /// });
    /// ```
    #[must_use]
    pub fn with_error_handler<F>(self, handler: F) -> Self
    where
        F: Fn(io::Error) + Send + Sync + 'static,
    {
        let mut inner = self.inner.write().unwrap_or_else(|e| e.into_inner());
        inner.error_handler = Some(Arc::new(handler));
        drop(inner);
        self
    }

    /// Logs a message at the specified level.
    ///
    /// This method uses a single write lock for the entire operation (format + write)
    /// to ensure configuration consistency. The only exception is caller info capture,
    /// which happens inside formatting but is atomic with respect to the log entry.
    pub fn log(&self, level: Level, msg: &str, keyvals: &[(&str, &str)]) {
        // Use a single write lock for the entire operation to avoid race conditions
        // between configuration reads and writes. This eliminates the window where
        // another thread could modify settings between formatting and writing.
        let mut inner = self.inner.write().unwrap_or_else(|e| e.into_inner());

        // Check if we need to emit caller overhead warning
        if inner.report_caller && !inner.warned_caller_overhead && !inner.suppress_caller_warning {
            inner.warned_caller_overhead = true;
            // Emit warning to stderr (separate from the log output)
            let _ = io::stderr().write_all(
                b"[charmed_log] PERF WARNING: caller reporting enabled - expect 100-1000x slowdown\n",
            );
        }

        // Check level - early return while holding the lock is fine
        if level < inner.level {
            return;
        }

        // Format output using current configuration (atomic snapshot)
        let mut output = String::new();
        match inner.formatter {
            Formatter::Text => Self::format_text_inner(&inner, level, msg, keyvals, &mut output),
            Formatter::Json => Self::format_json_inner(&inner, level, msg, keyvals, &mut output),
            Formatter::Logfmt => {
                Self::format_logfmt_inner(&inner, level, msg, keyvals, &mut output);
            }
        }

        // Write output with error handling (still holding the lock for consistency)
        if let Err(e) = inner.writer.write_all(output.as_bytes()) {
            // Call user-provided error handler if available
            if let Some(ref handler) = inner.error_handler {
                // Clone the handler to avoid holding the lock while calling it
                let handler = Arc::clone(handler);
                drop(inner);
                handler(e);
            } else if !inner.has_warned_io_failure {
                // Default behavior: warn once to stderr, then go silent
                inner.has_warned_io_failure = true;
                drop(inner);
                // Attempt to write warning to stderr (may also fail, but we try)
                let _ =
                    io::stderr().write_all(format!("charmed_log: write failed: {e}\n").as_bytes());
            }
        }
    }

    // =========================================================================
    // Associated functions for formatting (no &self, used by log() for atomicity)
    // =========================================================================

    /// Format text output without requiring &self (for use in atomic log operation).
    fn format_text_inner(
        inner: &LoggerInner,
        level: Level,
        msg: &str,
        keyvals: &[(&str, &str)],
        output: &mut String,
    ) {
        let styles = &inner.styles;
        let mut first = true;

        // Timestamp
        if inner.report_timestamp {
            let ts = (inner.time_function)(SystemTime::now());
            if let Ok(duration) = ts.duration_since(UNIX_EPOCH) {
                let secs = duration.as_secs();
                let ts_str = format_timestamp(secs, &inner.time_format);
                let styled = styles.timestamp.render(&ts_str);
                if !first {
                    output.push(' ');
                }
                output.push_str(&styled);
                first = false;
            }
        }

        // Level
        if let Some(level_style) = styles.levels.get(&level) {
            let lvl = level_style.render(level.as_upper_str());
            if !first {
                output.push(' ');
            }
            output.push_str(&lvl);
            first = false;
        }

        // Caller - extract actual caller info from backtrace
        if inner.report_caller {
            let caller_str = if let Some(info) = CallerInfo::capture(inner.caller_offset) {
                (inner.caller_formatter)(&info.file, info.line, &info.function)
            } else {
                (inner.caller_formatter)("unknown", 0, "unknown")
            };
            let styled = styles.caller.render(&format!("<{caller_str}>"));
            if !first {
                output.push(' ');
            }
            output.push_str(&styled);
            first = false;
        }

        // Prefix
        if !inner.prefix.is_empty() {
            let styled = styles.prefix.render(&format!("{}:", inner.prefix));
            if !first {
                output.push(' ');
            }
            output.push_str(&styled);
            first = false;
        }

        // Message
        if !msg.is_empty() {
            let styled = styles.message.render(msg);
            if !first {
                output.push(' ');
            }
            output.push_str(&styled);
            first = false;
        }

        // Default fields
        for (key, value) in &inner.fields {
            Self::format_text_keyval_inner(styles, key, value, &mut first, output);
        }

        // Additional keyvals
        for (key, value) in keyvals {
            Self::format_text_keyval_inner(styles, key, value, &mut first, output);
        }

        output.push('\n');
    }

    /// Format a key-value pair for text output.
    fn format_text_keyval_inner(
        styles: &Styles,
        key: &str,
        value: &str,
        first: &mut bool,
        output: &mut String,
    ) {
        let sep = styles.separator.render("=");
        let key_styled = if let Some(style) = styles.keys.get(key) {
            style.render(key)
        } else {
            styles.key.render(key)
        };
        let value_styled = if let Some(style) = styles.values.get(key) {
            style.render(value)
        } else {
            styles.value.render(value)
        };

        if !*first {
            output.push(' ');
        }
        output.push_str(&key_styled);
        output.push_str(&sep);
        output.push_str(&value_styled);
        *first = false;
    }

    /// Format JSON output without requiring &self.
    fn format_json_inner(
        inner: &LoggerInner,
        level: Level,
        msg: &str,
        keyvals: &[(&str, &str)],
        output: &mut String,
    ) {
        output.push('{');
        let mut first = true;

        // Timestamp
        if inner.report_timestamp {
            let ts = (inner.time_function)(SystemTime::now());
            if let Ok(duration) = ts.duration_since(UNIX_EPOCH) {
                let secs = duration.as_secs();
                let ts_str = format_timestamp(secs, &inner.time_format);
                write_json_field(output, keys::TIMESTAMP, &ts_str, &mut first);
            }
        }

        // Level
        write_json_field(output, keys::LEVEL, level.as_str(), &mut first);

        // Prefix
        if !inner.prefix.is_empty() {
            write_json_field(output, keys::PREFIX, &inner.prefix, &mut first);
        }

        // Message
        if !msg.is_empty() {
            write_json_field(output, keys::MESSAGE, msg, &mut first);
        }

        // Default fields
        for (key, value) in &inner.fields {
            write_json_field(output, key, value, &mut first);
        }

        // Additional keyvals
        for (key, value) in keyvals {
            write_json_field(output, key, value, &mut first);
        }

        output.push_str("}\n");
    }

    /// Format logfmt output without requiring &self.
    fn format_logfmt_inner(
        inner: &LoggerInner,
        level: Level,
        msg: &str,
        keyvals: &[(&str, &str)],
        output: &mut String,
    ) {
        let mut first = true;

        // Timestamp
        if inner.report_timestamp {
            let ts = (inner.time_function)(SystemTime::now());
            if let Ok(duration) = ts.duration_since(UNIX_EPOCH) {
                let secs = duration.as_secs();
                let ts_str = format_timestamp(secs, &inner.time_format);
                write_logfmt_field(output, keys::TIMESTAMP, &ts_str, &mut first);
            }
        }

        // Level
        write_logfmt_field(output, keys::LEVEL, level.as_str(), &mut first);

        // Prefix
        if !inner.prefix.is_empty() {
            write_logfmt_field(output, keys::PREFIX, &inner.prefix, &mut first);
        }

        // Message
        if !msg.is_empty() {
            write_logfmt_field(output, keys::MESSAGE, msg, &mut first);
        }

        // Default fields
        for (key, value) in &inner.fields {
            write_logfmt_field(output, key, value, &mut first);
        }

        // Additional keyvals
        for (key, value) in keyvals {
            write_logfmt_field(output, key, value, &mut first);
        }

        output.push('\n');
    }

    // =========================================================================
    // Instance method wrappers (for backward compatibility)
    // =========================================================================

    #[expect(dead_code, reason = "Kept for API compatibility")]
    fn format_text(
        &self,
        inner: &LoggerInner,
        level: Level,
        msg: &str,
        keyvals: &[(&str, &str)],
        output: &mut String,
    ) {
        Self::format_text_inner(inner, level, msg, keyvals, output);
    }

    #[expect(dead_code, reason = "Kept for API compatibility")]
    fn format_text_keyval(
        &self,
        styles: &Styles,
        key: &str,
        value: &str,
        first: &mut bool,
        output: &mut String,
    ) {
        Self::format_text_keyval_inner(styles, key, value, first, output);
    }

    #[expect(dead_code, reason = "Kept for API compatibility")]
    fn format_json(
        &self,
        inner: &LoggerInner,
        level: Level,
        msg: &str,
        keyvals: &[(&str, &str)],
        output: &mut String,
    ) {
        Self::format_json_inner(inner, level, msg, keyvals, output);
    }

    #[expect(dead_code, reason = "Kept for API compatibility")]
    fn format_logfmt(
        &self,
        inner: &LoggerInner,
        level: Level,
        msg: &str,
        keyvals: &[(&str, &str)],
        output: &mut String,
    ) {
        Self::format_logfmt_inner(inner, level, msg, keyvals, output);
    }

    /// Logs a debug message.
    pub fn debug(&self, msg: &str, keyvals: &[(&str, &str)]) {
        self.log(Level::Debug, msg, keyvals);
    }

    /// Logs an info message.
    pub fn info(&self, msg: &str, keyvals: &[(&str, &str)]) {
        self.log(Level::Info, msg, keyvals);
    }

    /// Logs a warning message.
    pub fn warn(&self, msg: &str, keyvals: &[(&str, &str)]) {
        self.log(Level::Warn, msg, keyvals);
    }

    /// Logs an error message.
    pub fn error(&self, msg: &str, keyvals: &[(&str, &str)]) {
        self.log(Level::Error, msg, keyvals);
    }

    /// Logs a fatal message.
    pub fn fatal(&self, msg: &str, keyvals: &[(&str, &str)]) {
        self.log(Level::Fatal, msg, keyvals);
    }

    /// Logs a message with formatting.
    pub fn logf(&self, level: Level, format: &str, args: &[&dyn fmt::Display]) {
        let msg = format_args_simple(format, args);
        self.log(level, &msg, &[]);
    }

    /// Logs a debug message with formatting.
    pub fn debugf(&self, format: &str, args: &[&dyn fmt::Display]) {
        self.logf(Level::Debug, format, args);
    }

    /// Logs an info message with formatting.
    pub fn infof(&self, format: &str, args: &[&dyn fmt::Display]) {
        self.logf(Level::Info, format, args);
    }

    /// Logs a warning message with formatting.
    pub fn warnf(&self, format: &str, args: &[&dyn fmt::Display]) {
        self.logf(Level::Warn, format, args);
    }

    /// Logs an error message with formatting.
    pub fn errorf(&self, format: &str, args: &[&dyn fmt::Display]) {
        self.logf(Level::Error, format, args);
    }

    /// Logs a fatal message with formatting.
    pub fn fatalf(&self, format: &str, args: &[&dyn fmt::Display]) {
        self.logf(Level::Fatal, format, args);
    }
}

/// Simple format string replacement.
fn format_args_simple(format: &str, args: &[&dyn fmt::Display]) -> String {
    let mut result = format.to_string();
    for arg in args {
        if let Some(pos) = result.find("{}") {
            result = format!("{}{}{}", &result[..pos], arg, &result[pos + 2..]);
        }
    }
    result
}

/// Formats a Unix timestamp.
fn format_timestamp(secs: u64, format: &str) -> String {
    use chrono::{DateTime, Utc};

    if let Some(datetime) = DateTime::from_timestamp(secs as i64, 0) {
        datetime.with_timezone(&Utc).format(format).to_string()
    } else {
        "INVALID TIMESTAMP".to_string()
    }
}

/// Writes a JSON field.
fn write_json_field(output: &mut String, key: &str, value: &str, first: &mut bool) {
    if !*first {
        output.push(',');
    }
    output.push('"');
    output.push_str(&escape_json(key));
    output.push_str("\":\"");
    output.push_str(&escape_json(value));
    output.push('"');
    *first = false;
}

/// Escapes a string for JSON.
fn escape_json(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result
}

/// Writes a logfmt field.
fn write_logfmt_field(output: &mut String, key: &str, value: &str, first: &mut bool) {
    if !*first {
        output.push(' ');
    }
    output.push_str(key);
    output.push('=');
    if needs_quoting(value) {
        output.push('"');
        output.push_str(&escape_logfmt(value));
        output.push('"');
    } else {
        output.push_str(value);
    }
    *first = false;
}

/// Checks if a value needs quoting in logfmt.
fn needs_quoting(s: &str) -> bool {
    s.is_empty()
        || s.chars()
            .any(|c| c.is_whitespace() || c == '"' || c == '=' || c.is_control())
}

/// Escapes a string for logfmt.
fn escape_logfmt(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c => result.push(c),
        }
    }
    result
}

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::{
        CallerInfo, DEFAULT_TIME_FORMAT, ErrorHandler, Formatter, Level, Logger, Options,
        ParseLevelError, ParseResult, Styles, keys, long_caller_formatter, now_utc,
        short_caller_formatter,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_ordering() {
        assert!(Level::Debug < Level::Info);
        assert!(Level::Info < Level::Warn);
        assert!(Level::Warn < Level::Error);
        assert!(Level::Error < Level::Fatal);
    }

    #[test]
    fn test_level_display() {
        assert_eq!(Level::Debug.to_string(), "debug");
        assert_eq!(Level::Info.to_string(), "info");
        assert_eq!(Level::Warn.to_string(), "warn");
        assert_eq!(Level::Error.to_string(), "error");
        assert_eq!(Level::Fatal.to_string(), "fatal");
    }

    #[test]
    fn test_level_parse() {
        assert_eq!("debug".parse::<Level>().unwrap(), Level::Debug);
        assert_eq!("INFO".parse::<Level>().unwrap(), Level::Info);
        assert_eq!("WARN".parse::<Level>().unwrap(), Level::Warn);
        // Note: "warning" is NOT accepted - only "warn" (matching Go behavior)
        assert!("warning".parse::<Level>().is_err());
        assert!("invalid".parse::<Level>().is_err());
    }

    #[test]
    fn test_logger_new() {
        let logger = Logger::new();
        assert_eq!(logger.level(), Level::Info);
        assert!(logger.prefix().is_empty());
    }

    #[test]
    fn test_logger_set_level() {
        let logger = Logger::new();
        logger.set_level(Level::Debug);
        assert_eq!(logger.level(), Level::Debug);
    }

    #[test]
    fn test_logger_set_prefix() {
        let logger = Logger::new();
        logger.set_prefix("myapp");
        assert_eq!(logger.prefix(), "myapp");
    }

    #[test]
    fn test_logger_with_prefix() {
        let logger = Logger::new();
        let prefixed = logger.with_prefix("myapp");
        assert_eq!(prefixed.prefix(), "myapp");
        assert!(logger.prefix().is_empty()); // Original unchanged
    }

    #[test]
    fn test_logger_with_fields() {
        let logger = Logger::new();
        let with_fields = logger.with_fields(&[("app", "test"), ("version", "1.0")]);
        // Fields are internal, just verify it doesn't panic
        drop(with_fields);
    }

    #[test]
    fn test_logger_with_method() {
        // Test the Go API compatible `with()` method
        let logger = Logger::new();
        let ctx_logger = logger.with(&[("request_id", "abc123"), ("user", "alice")]);
        // Verify it creates a new logger (not the same instance)
        // Both should work independently
        ctx_logger.info("test message", &[]);
        logger.info("another message", &[]);
    }

    #[test]
    fn test_caller_info_capture() {
        // CallerInfo::capture should return Some when called from a test
        let info = CallerInfo::capture(0);
        // The capture might return None in optimized builds, so we just
        // verify it doesn't panic
        if let Some(caller) = info {
            // In debug builds, we should get meaningful info
            assert!(!caller.function.is_empty());
        }
    }

    #[test]
    fn test_styles_default() {
        let styles = Styles::new();
        assert!(styles.levels.contains_key(&Level::Debug));
        assert!(styles.levels.contains_key(&Level::Info));
        assert!(styles.levels.contains_key(&Level::Warn));
        assert!(styles.levels.contains_key(&Level::Error));
        assert!(styles.levels.contains_key(&Level::Fatal));
    }

    #[test]
    fn test_trim_caller_path() {
        assert_eq!(trim_caller_path("src/lib.rs", 1), "lib.rs");
        assert_eq!(trim_caller_path("foo/bar/baz.rs", 2), "bar/baz.rs");
        assert_eq!(trim_caller_path("baz.rs", 2), "baz.rs");
        assert_eq!(trim_caller_path("foo/bar/baz.rs", 0), "foo/bar/baz.rs");
    }

    #[test]
    fn test_short_caller_formatter() {
        let result = short_caller_formatter("/home/user/project/src/main.rs", 42, "main");
        assert!(result.contains(":42"));
    }

    #[test]
    fn test_long_caller_formatter() {
        let result = long_caller_formatter("/home/user/project/src/main.rs", 42, "main");
        assert_eq!(result, "/home/user/project/src/main.rs:42");
    }

    #[test]
    fn test_escape_json() {
        assert_eq!(escape_json("hello"), "hello");
        assert_eq!(escape_json("hello \"world\""), "hello \\\"world\\\"");
        assert_eq!(escape_json("line1\nline2"), "line1\\nline2");
    }

    #[test]
    fn test_needs_quoting() {
        assert!(needs_quoting(""));
        assert!(needs_quoting("hello world"));
        assert!(needs_quoting("key=value"));
        assert!(needs_quoting("has\"quote"));
        assert!(!needs_quoting("simple"));
    }

    #[test]
    fn test_escape_logfmt() {
        assert_eq!(escape_logfmt("hello"), "hello");
        assert_eq!(escape_logfmt("hello \"world\""), "hello \\\"world\\\"");
        assert_eq!(escape_logfmt("line1\nline2"), "line1\\nline2");
    }

    #[test]
    fn test_formatter_default() {
        assert_eq!(Formatter::default(), Formatter::Text);
    }

    #[test]
    fn test_options_default() {
        let opts = Options::default();
        assert_eq!(opts.level, Level::Info);
        assert_eq!(opts.formatter, Formatter::Text);
        assert!(!opts.report_timestamp);
        assert!(!opts.report_caller);
    }

    #[test]
    fn test_logger_with_options() {
        let opts = Options {
            level: Level::Debug,
            prefix: "test".to_string(),
            report_timestamp: true,
            ..Default::default()
        };
        let logger = Logger::with_options(opts);
        assert_eq!(logger.level(), Level::Debug);
        assert_eq!(logger.prefix(), "test");
    }

    /// A writer that always fails for testing error handling.
    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("simulated failure"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::other("simulated failure"))
        }
    }

    #[test]
    fn test_error_handler_called_on_io_failure() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let error_count = Arc::new(AtomicUsize::new(0));
        let counter = error_count.clone();

        // Create logger with custom error handler
        let logger = Logger::new().with_error_handler(move |_err| {
            counter.fetch_add(1, Ordering::Relaxed);
        });

        // Replace writer with failing writer
        {
            let mut inner = logger.inner.write().unwrap();
            inner.writer = Box::new(FailingWriter);
        }

        // Log a message - should trigger error handler
        logger.info("test message", &[]);

        // Verify error handler was called
        assert_eq!(error_count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_error_handler_receives_correct_error() {
        use std::sync::Mutex;

        let captured_error = Arc::new(Mutex::new(None::<String>));
        let error_capture = captured_error.clone();

        let logger = Logger::new().with_error_handler(move |err| {
            *error_capture.lock().unwrap() = Some(err.to_string());
        });

        // Replace writer with failing writer
        {
            let mut inner = logger.inner.write().unwrap();
            inner.writer = Box::new(FailingWriter);
        }

        logger.info("test", &[]);

        let error_msg = captured_error.lock().unwrap();
        assert!(error_msg.is_some());
        assert!(error_msg.as_ref().unwrap().contains("simulated failure"));
    }

    #[test]
    fn test_default_behavior_warns_once() {
        // Without an error handler, the default behavior is to warn once
        // We can't easily test stderr output, but we can verify it doesn't panic
        let logger = Logger::new();

        // Replace writer with failing writer
        {
            let mut inner = logger.inner.write().unwrap();
            inner.writer = Box::new(FailingWriter);
        }

        // Log multiple messages - should not panic
        logger.info("first message", &[]);
        logger.info("second message", &[]);
        logger.info("third message", &[]);

        // Verify has_warned_io_failure is set
        let inner = logger.inner.read().unwrap();
        assert!(inner.has_warned_io_failure);
    }

    #[test]
    fn test_error_handler_inherited_by_with_fields() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let error_count = Arc::new(AtomicUsize::new(0));
        let counter = error_count.clone();

        let logger = Logger::new().with_error_handler(move |_err| {
            counter.fetch_add(1, Ordering::Relaxed);
        });

        // Create a child logger with additional fields
        let child_logger = logger.with_fields(&[("component", "test")]);

        // Replace writer with failing writer on child
        {
            let mut inner = child_logger.inner.write().unwrap();
            inner.writer = Box::new(FailingWriter);
        }

        // Log a message on child - should use inherited error handler
        child_logger.info("test message", &[]);

        assert_eq!(error_count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_with_error_handler_returns_same_logger() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let called = Arc::new(AtomicBool::new(false));
        let flag = called.clone();

        let logger = Logger::new().with_error_handler(move |_| {
            flag.store(true, Ordering::Relaxed);
        });

        // Verify the logger is usable after setting error handler
        assert_eq!(logger.level(), Level::Info);

        // The error handler should be set
        let inner = logger.inner.read().unwrap();
        assert!(inner.error_handler.is_some());
    }

    #[test]
    fn test_caller_warning_flag_set_on_first_log() {
        let logger = Logger::new();
        logger.set_report_caller(true);

        // Verify warning flag is not set initially
        {
            let inner = logger.inner.read().unwrap();
            assert!(!inner.warned_caller_overhead);
        }

        // Log a message (this should set the warning flag)
        logger.info("test message", &[]);

        // Verify warning flag is now set
        {
            let inner = logger.inner.read().unwrap();
            assert!(inner.warned_caller_overhead);
        }
    }

    #[test]
    fn test_caller_warning_suppressed() {
        let logger = Logger::new();
        logger.set_report_caller(true);
        logger.suppress_caller_warning();

        // Verify suppression flag is set
        {
            let inner = logger.inner.read().unwrap();
            assert!(inner.suppress_caller_warning);
        }

        // Log a message (warning flag should still be false since suppressed)
        logger.info("test message", &[]);

        // Verify warning flag remains false when suppressed
        {
            let inner = logger.inner.read().unwrap();
            assert!(!inner.warned_caller_overhead);
        }
    }

    #[test]
    fn test_caller_warning_not_triggered_when_caller_disabled() {
        let logger = Logger::new();
        // report_caller is false by default

        // Log a message
        logger.info("test message", &[]);

        // Verify warning flag is not set when caller reporting is disabled
        {
            let inner = logger.inner.read().unwrap();
            assert!(!inner.warned_caller_overhead);
        }
    }

    #[test]
    fn test_caller_warning_inherits_suppression_via_with_fields() {
        let logger = Logger::new();
        logger.set_report_caller(true);
        logger.suppress_caller_warning();

        // Create child logger
        let child = logger.with_fields(&[("key", "value")]);

        // Verify child inherits suppression setting
        {
            let inner = child.inner.read().unwrap();
            assert!(inner.suppress_caller_warning);
        }
    }

    #[test]
    fn test_caller_warning_resets_for_child_logger() {
        let logger = Logger::new();
        logger.set_report_caller(true);

        // Trigger warning on parent
        logger.info("parent message", &[]);

        // Create child logger
        let child = logger.with_fields(&[("key", "value")]);

        // Verify child has fresh warning state
        {
            let inner = child.inner.read().unwrap();
            assert!(!inner.warned_caller_overhead);
        }
    }
}
