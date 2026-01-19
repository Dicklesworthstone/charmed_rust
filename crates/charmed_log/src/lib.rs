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

use lipgloss::{Color, Style};
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;
use std::io::{self, Write};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

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
        let inner = self.inner.read().unwrap();
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
            })),
        }
    }

    /// Sets the minimum log level.
    pub fn set_level(&self, level: Level) {
        let mut inner = self.inner.write().unwrap();
        inner.level = level;
    }

    /// Returns the current log level.
    #[must_use]
    pub fn level(&self) -> Level {
        let inner = self.inner.read().unwrap();
        inner.level
    }

    /// Sets the log prefix.
    pub fn set_prefix(&self, prefix: impl Into<String>) {
        let mut inner = self.inner.write().unwrap();
        inner.prefix = prefix.into();
    }

    /// Returns the current prefix.
    #[must_use]
    pub fn prefix(&self) -> String {
        let inner = self.inner.read().unwrap();
        inner.prefix.clone()
    }

    /// Sets whether to report timestamps.
    pub fn set_report_timestamp(&self, report: bool) {
        let mut inner = self.inner.write().unwrap();
        inner.report_timestamp = report;
    }

    /// Sets whether to report caller location.
    pub fn set_report_caller(&self, report: bool) {
        let mut inner = self.inner.write().unwrap();
        inner.report_caller = report;
    }

    /// Sets the time format.
    pub fn set_time_format(&self, format: impl Into<String>) {
        let mut inner = self.inner.write().unwrap();
        inner.time_format = format.into();
    }

    /// Sets the formatter.
    pub fn set_formatter(&self, formatter: Formatter) {
        let mut inner = self.inner.write().unwrap();
        inner.formatter = formatter;
    }

    /// Sets the styles.
    pub fn set_styles(&self, styles: Styles) {
        let mut inner = self.inner.write().unwrap();
        inner.styles = styles;
    }

    /// Creates a new logger with additional fields.
    #[must_use]
    pub fn with_fields(&self, fields: &[(&str, &str)]) -> Self {
        let inner = self.inner.read().unwrap();
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
            })),
        }
    }

    /// Creates a new logger with a different prefix.
    #[must_use]
    pub fn with_prefix(&self, prefix: impl Into<String>) -> Self {
        let new_logger = self.with_fields(&[]);
        new_logger.set_prefix(prefix);
        new_logger
    }

    /// Logs a message at the specified level.
    pub fn log(&self, level: Level, msg: &str, keyvals: &[(&str, &str)]) {
        let inner = self.inner.read().unwrap();

        // Check level
        if level < inner.level {
            return;
        }

        let mut output = String::new();

        match inner.formatter {
            Formatter::Text => self.format_text(&inner, level, msg, keyvals, &mut output),
            Formatter::Json => self.format_json(&inner, level, msg, keyvals, &mut output),
            Formatter::Logfmt => self.format_logfmt(&inner, level, msg, keyvals, &mut output),
        }

        drop(inner);

        // Write output
        let mut inner = self.inner.write().unwrap();
        let _ = inner.writer.write_all(output.as_bytes());
    }

    fn format_text(
        &self,
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
                // Simple timestamp formatting
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

        // Caller (simplified - actual caller info would require backtrace)
        if inner.report_caller {
            let caller = (inner.caller_formatter)("unknown", 0, "unknown");
            let styled = styles.caller.render(&format!("<{caller}>"));
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
            self.format_text_keyval(styles, key, value, &mut first, output);
        }

        // Additional keyvals
        for (key, value) in keyvals {
            self.format_text_keyval(styles, key, value, &mut first, output);
        }

        output.push('\n');
    }

    fn format_text_keyval(
        &self,
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

    fn format_json(
        &self,
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

    fn format_logfmt(
        &self,
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
        DEFAULT_TIME_FORMAT, Formatter, Level, Logger, Options, ParseLevelError, ParseResult,
        Styles, keys, long_caller_formatter, now_utc, short_caller_formatter,
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
}
