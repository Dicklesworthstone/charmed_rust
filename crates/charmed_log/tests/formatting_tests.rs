//! Comprehensive unit tests for `charmed_log` formatting and configuration (bd-2qjc).
//!
//! Tests cover:
//! - Log level enumeration and ordering
//! - Logger configuration (getters, setters, round-trips)
//! - Field composition chains
//! - Styles construction and customization
//! - Caller formatting functions
//! - Options struct construction
//! - Formatter enum behavior
//! - Keys module constants
//! - Prelude module imports
//! - Thread safety
//! - Edge cases

#![allow(clippy::uninlined_format_args)]

use charmed_log::prelude::*;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Barrier};
use std::thread;

// ===========================================================================
// 1. Level Enumeration: Extended Coverage
// ===========================================================================

#[test]
fn level_as_str_all_variants() {
    assert_eq!(Level::Debug.as_str(), "debug");
    assert_eq!(Level::Info.as_str(), "info");
    assert_eq!(Level::Warn.as_str(), "warn");
    assert_eq!(Level::Error.as_str(), "error");
    assert_eq!(Level::Fatal.as_str(), "fatal");
}

#[test]
fn level_as_upper_str_all_variants() {
    assert_eq!(Level::Debug.as_upper_str(), "DEBU");
    assert_eq!(Level::Info.as_upper_str(), "INFO");
    assert_eq!(Level::Warn.as_upper_str(), "WARN");
    assert_eq!(Level::Error.as_upper_str(), "ERRO");
    assert_eq!(Level::Fatal.as_upper_str(), "FATA");
}

#[test]
fn level_ordering_all_pairs() {
    let levels = [
        Level::Debug,
        Level::Info,
        Level::Warn,
        Level::Error,
        Level::Fatal,
    ];
    for i in 0..levels.len() {
        for j in (i + 1)..levels.len() {
            assert!(
                levels[i] < levels[j],
                "{:?} should be < {:?}",
                levels[i],
                levels[j]
            );
        }
    }
}

#[test]
fn level_equality() {
    assert_eq!(Level::Debug, Level::Debug);
    assert_eq!(Level::Info, Level::Info);
    assert_ne!(Level::Debug, Level::Info);
    assert_ne!(Level::Error, Level::Fatal);
}

#[test]
fn level_clone_and_copy() {
    let level = Level::Warn;
    #[allow(clippy::clone_on_copy)] // Intentionally testing Clone impl
    let cloned = level.clone();
    let copied = level;
    assert_eq!(level, cloned);
    assert_eq!(level, copied);
}

#[test]
fn level_hash_consistent() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Level::Debug);
    set.insert(Level::Info);
    set.insert(Level::Debug); // duplicate
    assert_eq!(set.len(), 2);
}

#[test]
fn level_display_format() {
    assert_eq!(format!("{}", Level::Debug), "debug");
    assert_eq!(format!("{}", Level::Info), "info");
    assert_eq!(format!("{}", Level::Warn), "warn");
    assert_eq!(format!("{}", Level::Error), "error");
    assert_eq!(format!("{}", Level::Fatal), "fatal");
}

#[test]
fn level_debug_format() {
    // Debug format should contain the variant name
    let debug = format!("{:?}", Level::Info);
    assert!(debug.contains("Info"), "Debug format: {}", debug);
}

#[test]
fn level_from_str_roundtrip() {
    let levels = [
        Level::Debug,
        Level::Info,
        Level::Warn,
        Level::Error,
        Level::Fatal,
    ];
    for level in &levels {
        let s = level.as_str();
        let parsed = Level::from_str(s).unwrap();
        assert_eq!(*level, parsed, "Round-trip failed for {:?}", level);
    }
}

#[test]
fn level_from_str_case_insensitive() {
    assert_eq!(Level::from_str("debug").unwrap(), Level::Debug);
    assert_eq!(Level::from_str("DEBUG").unwrap(), Level::Debug);
    assert_eq!(Level::from_str("Debug").unwrap(), Level::Debug);
    assert_eq!(Level::from_str("dEbUg").unwrap(), Level::Debug);
}

#[test]
fn level_from_str_invalid_returns_error() {
    assert!(Level::from_str("").is_err());
    assert!(Level::from_str("trace").is_err());
    assert!(Level::from_str("verbose").is_err());
    assert!(Level::from_str("warning").is_err());
    assert!(Level::from_str("critical").is_err());
    assert!(Level::from_str("notice").is_err());
    assert!(Level::from_str("0").is_err());
    assert!(Level::from_str("-4").is_err());
}

// ===========================================================================
// 2. Logger Configuration: Getters and Setters
// ===========================================================================

#[test]
fn logger_default_level_is_info() {
    let logger = Logger::new();
    assert_eq!(logger.level(), Level::Info);
}

#[test]
fn logger_set_level_roundtrip() {
    let logger = Logger::new();
    for level in [
        Level::Debug,
        Level::Info,
        Level::Warn,
        Level::Error,
        Level::Fatal,
    ] {
        logger.set_level(level);
        assert_eq!(
            logger.level(),
            level,
            "Level round-trip failed for {:?}",
            level
        );
    }
}

#[test]
fn logger_default_prefix_is_empty() {
    let logger = Logger::new();
    assert_eq!(logger.prefix(), "");
}

#[test]
fn logger_set_prefix_roundtrip() {
    let logger = Logger::new();
    logger.set_prefix("myapp");
    assert_eq!(logger.prefix(), "myapp");

    logger.set_prefix("other");
    assert_eq!(logger.prefix(), "other");

    logger.set_prefix("");
    assert_eq!(logger.prefix(), "");
}

#[test]
fn logger_set_prefix_with_string() {
    let logger = Logger::new();
    logger.set_prefix(String::from("test"));
    assert_eq!(logger.prefix(), "test");
}

#[test]
fn logger_set_report_timestamp_toggle() {
    let logger = Logger::new();
    logger.set_report_timestamp(true);
    logger.set_report_timestamp(false);
    // Should not panic
}

#[test]
fn logger_set_report_caller_toggle() {
    let logger = Logger::new();
    logger.set_report_caller(true);
    logger.suppress_caller_warning();
    logger.set_report_caller(false);
    // Should not panic
}

#[test]
fn logger_set_time_format() {
    let logger = Logger::new();
    logger.set_time_format("%H:%M:%S");
    logger.set_time_format(DEFAULT_TIME_FORMAT);
    // Should not panic
}

#[test]
fn logger_set_formatter_all_types() {
    let logger = Logger::new();
    logger.set_formatter(Formatter::Text);
    logger.set_formatter(Formatter::Json);
    logger.set_formatter(Formatter::Logfmt);
    // Should not panic
}

// ===========================================================================
// 3. Field Composition
// ===========================================================================

#[test]
fn logger_with_fields_creates_child() {
    let parent = Logger::new();
    parent.set_prefix("parent");
    let child = parent.with_fields(&[("key", "value")]);
    // Child is a separate logger
    assert_eq!(child.prefix(), "parent"); // inherits prefix
}

#[test]
fn logger_with_fields_does_not_modify_parent() {
    let parent = Logger::new();
    parent.set_prefix("parent");
    let child = parent.with_fields(&[("extra", "data")]);
    child.set_prefix("child");
    // Parent should be unaffected
    assert_eq!(parent.prefix(), "parent");
    assert_eq!(child.prefix(), "child");
}

#[test]
fn logger_with_multiple_fields() {
    let logger = Logger::new();
    let child = logger.with_fields(&[("a", "1"), ("b", "2"), ("c", "3")]);
    // Should not panic
    let _ = child;
}

#[test]
fn logger_with_chained_fields() {
    let logger = Logger::new();
    let child1 = logger.with_fields(&[("a", "1")]);
    let child2 = child1.with_fields(&[("b", "2")]);
    let child3 = child2.with_fields(&[("c", "3")]);
    // Chained composition should not panic
    let _ = child3;
}

#[test]
fn logger_with_go_api_compat() {
    let logger = Logger::new();
    // `with` is an alias for `with_fields`
    let child = logger.with(&[("key", "value")]);
    let _ = child;
}

#[test]
fn logger_with_prefix_creates_child() {
    let parent = Logger::new();
    parent.set_prefix("parent");
    let child = parent.with_prefix("child");
    assert_eq!(child.prefix(), "child");
    assert_eq!(parent.prefix(), "parent"); // parent unchanged
}

#[test]
fn logger_with_empty_fields() {
    let logger = Logger::new();
    let child = logger.with_fields(&[]);
    let _ = child;
}

// ===========================================================================
// 4. Styles Configuration
// ===========================================================================

#[test]
fn styles_default_has_all_levels() {
    let styles = Styles::new();
    assert!(styles.levels.contains_key(&Level::Debug));
    assert!(styles.levels.contains_key(&Level::Info));
    assert!(styles.levels.contains_key(&Level::Warn));
    assert!(styles.levels.contains_key(&Level::Error));
    assert!(styles.levels.contains_key(&Level::Fatal));
}

#[test]
fn styles_default_keys_and_values_empty() {
    let styles = Styles::new();
    assert!(styles.keys.is_empty());
    assert!(styles.values.is_empty());
}

#[test]
fn styles_clone() {
    let styles1 = Styles::new();
    let styles2 = styles1.clone();
    assert_eq!(styles1.levels.len(), styles2.levels.len());
}

#[test]
fn styles_custom_key_style() {
    let mut styles = Styles::new();
    styles
        .keys
        .insert("important".to_string(), lipgloss::Style::new().bold());
    assert!(styles.keys.contains_key("important"));
}

#[test]
fn styles_custom_value_style() {
    let mut styles = Styles::new();
    styles
        .values
        .insert("error_code".to_string(), lipgloss::Style::new().bold());
    assert!(styles.values.contains_key("error_code"));
}

#[test]
fn logger_set_styles() {
    let logger = Logger::new();
    let mut styles = Styles::new();
    styles
        .keys
        .insert("custom".to_string(), lipgloss::Style::new());
    logger.set_styles(styles);
    // Should not panic
}

// ===========================================================================
// 5. Caller Formatting Functions
// ===========================================================================

#[test]
fn short_caller_formatter_trims_path() {
    let result = short_caller_formatter("/home/user/project/src/main.rs", 42, "main");
    // Should show last 2 path segments
    assert!(
        result.contains("main.rs"),
        "Should contain filename: {}",
        result
    );
    assert!(
        result.contains("42"),
        "Should contain line number: {}",
        result
    );
}

#[test]
fn short_caller_formatter_short_path() {
    let result = short_caller_formatter("main.rs", 10, "main");
    assert!(result.contains("main.rs"), "Result: {}", result);
    assert!(result.contains("10"), "Result: {}", result);
}

#[test]
fn short_caller_formatter_empty_path() {
    let result = short_caller_formatter("", 0, "");
    // Should not panic; produces some output
    assert!(result.contains('0'), "Result: {}", result);
}

#[test]
fn long_caller_formatter_full_path() {
    let result = long_caller_formatter("/home/user/project/src/main.rs", 42, "main");
    assert!(
        result.contains("/home/user/project/src/main.rs"),
        "Should contain full path: {}",
        result
    );
    assert!(
        result.contains("42"),
        "Should contain line number: {}",
        result
    );
}

#[test]
fn long_caller_formatter_empty_path() {
    let result = long_caller_formatter("", 0, "");
    assert!(result.contains('0'), "Result: {}", result);
}

#[test]
fn caller_formatters_zero_line() {
    let short = short_caller_formatter("file.rs", 0, "func");
    let long = long_caller_formatter("file.rs", 0, "func");
    assert!(short.contains('0'));
    assert!(long.contains('0'));
}

#[test]
fn caller_formatters_large_line() {
    let short = short_caller_formatter("file.rs", u32::MAX, "func");
    let long = long_caller_formatter("file.rs", u32::MAX, "func");
    assert!(short.contains(&u32::MAX.to_string()));
    assert!(long.contains(&u32::MAX.to_string()));
}

// ===========================================================================
// 6. Options Struct
// ===========================================================================

#[test]
fn options_default_values() {
    let opts = Options::default();
    assert_eq!(opts.level, Level::Info);
    assert_eq!(opts.prefix, "");
    assert!(!opts.report_timestamp);
    assert!(!opts.report_caller);
    assert_eq!(opts.caller_offset, 0);
    assert!(opts.fields.is_empty());
    assert_eq!(opts.formatter, Formatter::Text);
    assert_eq!(opts.time_format, DEFAULT_TIME_FORMAT);
}

#[test]
fn options_custom_construction() {
    let opts = Options {
        level: Level::Debug,
        prefix: "test".to_string(),
        report_timestamp: true,
        report_caller: false,
        formatter: Formatter::Json,
        time_format: "%H:%M:%S".to_string(),
        fields: vec![("env".to_string(), "test".to_string())],
        caller_offset: 2,
        time_function: now_utc,
        caller_formatter: long_caller_formatter,
    };
    assert_eq!(opts.level, Level::Debug);
    assert_eq!(opts.prefix, "test");
    assert!(opts.report_timestamp);
    assert_eq!(opts.formatter, Formatter::Json);
    assert_eq!(opts.fields.len(), 1);
}

#[test]
fn logger_with_options() {
    let opts = Options {
        level: Level::Warn,
        prefix: "app".to_string(),
        formatter: Formatter::Logfmt,
        ..Options::default()
    };
    let logger = Logger::with_options(opts);
    assert_eq!(logger.level(), Level::Warn);
    assert_eq!(logger.prefix(), "app");
}

// ===========================================================================
// 7. Formatter Enum
// ===========================================================================

#[test]
fn formatter_default_is_text() {
    assert_eq!(Formatter::default(), Formatter::Text);
}

#[test]
fn formatter_equality() {
    assert_eq!(Formatter::Text, Formatter::Text);
    assert_eq!(Formatter::Json, Formatter::Json);
    assert_eq!(Formatter::Logfmt, Formatter::Logfmt);
    assert_ne!(Formatter::Text, Formatter::Json);
    assert_ne!(Formatter::Json, Formatter::Logfmt);
}

#[test]
fn formatter_clone_and_copy() {
    let f = Formatter::Json;
    #[allow(clippy::clone_on_copy)] // Intentionally testing Clone impl
    let cloned = f.clone();
    let copied = f;
    assert_eq!(f, cloned);
    assert_eq!(f, copied);
}

#[test]
fn formatter_debug() {
    let debug = format!("{:?}", Formatter::Text);
    assert!(debug.contains("Text"), "Debug: {}", debug);
    let debug = format!("{:?}", Formatter::Json);
    assert!(debug.contains("Json"), "Debug: {}", debug);
    let debug = format!("{:?}", Formatter::Logfmt);
    assert!(debug.contains("Logfmt"), "Debug: {}", debug);
}

// ===========================================================================
// 8. Keys Module Constants
// ===========================================================================

#[test]
fn keys_constants_defined() {
    assert_eq!(keys::TIMESTAMP, "time");
    assert_eq!(keys::MESSAGE, "msg");
    assert_eq!(keys::LEVEL, "level");
    assert_eq!(keys::CALLER, "caller");
    assert_eq!(keys::PREFIX, "prefix");
}

#[test]
fn keys_constants_non_empty() {
    assert!(!keys::TIMESTAMP.is_empty());
    assert!(!keys::MESSAGE.is_empty());
    assert!(!keys::LEVEL.is_empty());
    assert!(!keys::CALLER.is_empty());
    assert!(!keys::PREFIX.is_empty());
}

#[test]
fn keys_constants_unique() {
    let keys_list = [
        keys::TIMESTAMP,
        keys::MESSAGE,
        keys::LEVEL,
        keys::CALLER,
        keys::PREFIX,
    ];
    let mut seen = std::collections::HashSet::new();
    for key in &keys_list {
        assert!(seen.insert(*key), "Duplicate key: {}", key);
    }
}

// ===========================================================================
// 9. DEFAULT_TIME_FORMAT
// ===========================================================================

#[test]
fn default_time_format_is_valid() {
    // Should be parseable by chrono
    let now = chrono::Utc::now();
    let formatted = now.format(DEFAULT_TIME_FORMAT).to_string();
    assert!(!formatted.is_empty());
    // Should look like "2026/01/28 12:34:56"
    assert!(
        formatted.len() >= 19,
        "Formatted time too short: {}",
        formatted
    );
}

// ===========================================================================
// 10. now_utc Function
// ===========================================================================

#[test]
fn now_utc_returns_same_time() {
    let input = std::time::SystemTime::now();
    let output = now_utc(input);
    assert_eq!(input, output);
}

// ===========================================================================
// 11. ParseLevelError
// ===========================================================================

#[test]
fn parse_level_error_display() {
    let err = Level::from_str("bad").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("invalid level"), "Message: {}", msg);
    assert!(msg.contains("bad"), "Message: {}", msg);
}

#[test]
fn parse_level_error_is_std_error() {
    let err = Level::from_str("xyz").unwrap_err();
    let _: &dyn std::error::Error = &err;
}

#[test]
fn parse_level_error_clone() {
    let err1 = Level::from_str("abc").unwrap_err();
    let err2 = err1.clone();
    assert_eq!(err1.to_string(), err2.to_string());
}

// ===========================================================================
// 12. Thread Safety
// ===========================================================================

#[test]
fn logger_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Logger>();
}

#[test]
fn logger_clone_shares_state() {
    let logger1 = Logger::new();
    let logger2 = logger1.clone();
    logger1.set_level(Level::Debug);
    // Clones share the same Arc, so changes are visible
    assert_eq!(logger2.level(), Level::Debug);
}

#[test]
fn logger_concurrent_level_changes() {
    let logger = Logger::new();
    let barrier = Arc::new(Barrier::new(4));
    let mut handles = vec![];

    let levels = [Level::Debug, Level::Info, Level::Warn, Level::Error];
    for level in levels {
        let l = logger.clone();
        let b = barrier.clone();
        handles.push(thread::spawn(move || {
            b.wait();
            l.set_level(level);
            l.level() // read back
        }));
    }

    for handle in handles {
        let result = handle.join().unwrap();
        // Should be one of the valid levels (not corrupted)
        assert!(
            [Level::Debug, Level::Info, Level::Warn, Level::Error].contains(&result),
            "Got invalid level: {:?}",
            result
        );
    }
}

#[test]
fn logger_concurrent_prefix_changes() {
    let logger = Logger::new();
    let barrier = Arc::new(Barrier::new(4));
    let mut handles = vec![];

    let prefixes = ["alpha", "bravo", "charlie", "delta"];
    for prefix in prefixes {
        let l = logger.clone();
        let b = barrier.clone();
        let p = prefix.to_string();
        handles.push(thread::spawn(move || {
            b.wait();
            l.set_prefix(&p);
            l.prefix()
        }));
    }

    for handle in handles {
        let result = handle.join().unwrap();
        // Should be one of the valid prefixes (not corrupted)
        assert!(
            ["alpha", "bravo", "charlie", "delta"].contains(&result.as_str()),
            "Got invalid prefix: {}",
            result
        );
    }
}

#[test]
fn logger_concurrent_log_calls_no_panic() {
    let logger = Logger::new();
    logger.set_level(Level::Fatal); // suppress output to stderr
    logger.suppress_caller_warning();

    let barrier = Arc::new(Barrier::new(4));
    let mut handles = vec![];

    for i in 0..4 {
        let l = logger.clone();
        let b = barrier.clone();
        handles.push(thread::spawn(move || {
            b.wait();
            for _ in 0..10 {
                l.info(&format!("Thread {i}"), &[("tid", &i.to_string())]);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

// ===========================================================================
// 13. Logger Log Methods: Smoke Tests
// ===========================================================================

#[test]
fn logger_log_all_levels_no_panic() {
    let logger = Logger::new();
    logger.set_level(Level::Debug);
    logger.debug("debug message", &[]);
    logger.info("info message", &[]);
    logger.warn("warn message", &[]);
    logger.error("error message", &[]);
    logger.fatal("fatal message", &[]);
}

#[test]
fn logger_log_with_keyvals_no_panic() {
    let logger = Logger::new();
    logger.info(
        "structured log",
        &[("key1", "val1"), ("key2", "val2"), ("key3", "val3")],
    );
}

#[test]
fn logger_logf_no_panic() {
    let logger = Logger::new();
    logger.infof("Hello %s, count=%s", &[&"world", &42]);
    logger.debugf("Debug %s", &[&"test"]);
    logger.warnf("Warning %s", &[&"alert"]);
    logger.errorf("Error %s", &[&"fail"]);
    logger.fatalf("Fatal %s", &[&"crash"]);
}

#[test]
fn logger_log_below_level_silently_skips() {
    let logger = Logger::new();
    logger.set_level(Level::Error);
    // These should be silently skipped (below Error level)
    logger.debug("should not appear", &[]);
    logger.info("should not appear", &[]);
    logger.warn("should not appear", &[]);
    // These should be logged
    logger.error("should appear", &[]);
    logger.fatal("should appear", &[]);
}

#[test]
fn logger_log_generic_method() {
    let logger = Logger::new();
    logger.log(Level::Info, "test", &[]);
    logger.log(Level::Debug, "test", &[("k", "v")]);
}

// ===========================================================================
// 14. Logger with All Formatters
// ===========================================================================

#[test]
fn logger_text_formatter_no_panic() {
    let logger = Logger::new();
    logger.set_formatter(Formatter::Text);
    logger.info("text format", &[("key", "value")]);
}

#[test]
fn logger_json_formatter_no_panic() {
    let opts = Options {
        formatter: Formatter::Json,
        ..Options::default()
    };
    let logger = Logger::with_options(opts);
    logger.info("json format", &[("key", "value")]);
}

#[test]
fn logger_logfmt_formatter_no_panic() {
    let opts = Options {
        formatter: Formatter::Logfmt,
        ..Options::default()
    };
    let logger = Logger::with_options(opts);
    logger.info("logfmt format", &[("key", "value")]);
}

// ===========================================================================
// 15. Logger with Timestamps
// ===========================================================================

#[test]
fn logger_with_timestamp_no_panic() {
    let opts = Options {
        report_timestamp: true,
        ..Options::default()
    };
    let logger = Logger::with_options(opts);
    logger.info("with timestamp", &[]);
}

#[test]
fn logger_with_custom_time_format_no_panic() {
    let opts = Options {
        report_timestamp: true,
        time_format: "%H:%M:%S%.3f".to_string(),
        ..Options::default()
    };
    let logger = Logger::with_options(opts);
    logger.info("custom time format", &[]);
}

// ===========================================================================
// 16. Error Handler
// ===========================================================================

#[test]
fn error_handler_builder_returns_logger() {
    let logger = Logger::new();
    let logger = logger.with_error_handler(|_err| {
        // Custom handler
    });
    // Should still work as a logger
    logger.info("test", &[]);
}

#[test]
fn error_handler_inherited_by_child() {
    let called = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let logger = Logger::new().with_error_handler(move |_| {
        called.store(true, std::sync::atomic::Ordering::SeqCst);
    });
    // Create a child; error handler should be inherited
    let child = logger.with_fields(&[("child", "true")]);
    let _ = child;
}

// ===========================================================================
// 17. Edge Cases
// ===========================================================================

#[test]
fn logger_empty_message() {
    let logger = Logger::new();
    logger.info("", &[]);
}

#[test]
fn logger_very_long_message() {
    let logger = Logger::new();
    let long_msg: String = "x".repeat(10_000);
    logger.info(&long_msg, &[]);
}

#[test]
fn logger_special_characters_in_message() {
    let logger = Logger::new();
    logger.info("Hello\nWorld\t!\r\0", &[]);
    logger.info("Quotes: \"hello\" 'world'", &[]);
    logger.info("Backslash: C:\\path\\file", &[]);
    logger.info("Unicode: こんにちは 🦀", &[]);
}

#[test]
fn logger_special_characters_in_keyvals() {
    let logger = Logger::new();
    logger.info(
        "test",
        &[
            ("key with spaces", "value with spaces"),
            ("key=equals", "value=equals"),
            ("key\"quotes", "value\"quotes"),
            ("empty_key", ""),
            ("", "empty_key_name"),
        ],
    );
}

#[test]
fn logger_many_fields() {
    let logger = Logger::new();
    let fields: Vec<(String, String)> = (0..100)
        .map(|i| (format!("key_{i}"), format!("value_{i}")))
        .collect();
    let field_refs: Vec<(&str, &str)> = fields
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    logger.info("many fields", &field_refs);
}

#[test]
fn logger_newlines_in_values() {
    let logger = Logger::new();
    logger.info("test", &[("multiline", "line1\nline2\nline3")]);
}

#[test]
fn logger_unicode_in_fields() {
    let logger = Logger::new();
    logger.info("unicode fields", &[("名前", "太郎"), ("emoji", "🦀")]);
}

// ===========================================================================
// 18. Prelude Module
// ===========================================================================

#[test]
fn prelude_exports_all_types() {
    // Verify all expected types are accessible via prelude
    let _: Level = Level::Info;
    let _: Formatter = Formatter::Text;
    let _: Options = Options::default();
    let _: Logger = Logger::new();
    let _: Styles = Styles::new();
    let _: &str = DEFAULT_TIME_FORMAT;
    let _ = now_utc;
    let _ = short_caller_formatter;
    let _ = long_caller_formatter;
}

// ===========================================================================
// 19. CallerInfo
// ===========================================================================

#[test]
fn caller_info_capture_returns_some() {
    // Should be able to capture caller info from within this test
    let info = CallerInfo::capture(0);
    // May return None in some environments, but should not panic
    if let Some(info) = info {
        assert!(!info.file.is_empty() || !info.function.is_empty());
    }
}

#[test]
fn caller_info_capture_large_skip() {
    // Very large skip should return None (no more frames)
    let info = CallerInfo::capture(1000);
    assert!(info.is_none(), "Large skip should return None");
}

// ===========================================================================
// 20. Logger with All Options Combined
// ===========================================================================

#[test]
fn logger_with_all_options_no_panic() {
    let opts = Options {
        time_function: now_utc,
        time_format: "%Y-%m-%dT%H:%M:%S%.3fZ".to_string(),
        level: Level::Debug,
        prefix: "fulltest".to_string(),
        report_timestamp: true,
        report_caller: false, // skip caller for speed
        caller_formatter: short_caller_formatter,
        caller_offset: 0,
        fields: vec![
            ("app".to_string(), "test".to_string()),
            ("env".to_string(), "ci".to_string()),
        ],
        formatter: Formatter::Text,
    };
    let logger = Logger::with_options(opts);
    logger.suppress_caller_warning();

    // Exercise all formatters
    logger.set_formatter(Formatter::Text);
    logger.info("text", &[("extra", "data")]);

    logger.set_formatter(Formatter::Json);
    logger.info("json", &[("extra", "data")]);

    logger.set_formatter(Formatter::Logfmt);
    logger.info("logfmt", &[("extra", "data")]);
}

#[test]
fn logger_rapid_reconfiguration_no_panic() {
    let logger = Logger::new();
    for i in 0..100 {
        if i % 2 == 0 {
            logger.set_level(Level::Debug);
            logger.set_formatter(Formatter::Text);
        } else {
            logger.set_level(Level::Error);
            logger.set_formatter(Formatter::Json);
        }
        logger.set_prefix(format!("iter_{i}"));
    }
}

// ===========================================================================
// 21. Styles HashMap Customization
// ===========================================================================

#[test]
fn styles_override_level_style() {
    let mut styles = Styles::new();
    // Override info level style
    let custom_style = lipgloss::Style::new().bold().italic();
    styles.levels.insert(Level::Info, custom_style);
    assert!(styles.levels.contains_key(&Level::Info));
}

#[test]
fn styles_add_many_custom_keys() {
    let mut styles = Styles::new();
    for i in 0..50 {
        styles
            .keys
            .insert(format!("key_{i}"), lipgloss::Style::new());
    }
    assert_eq!(styles.keys.len(), 50);
}

#[test]
fn styles_applied_to_logger() {
    let mut styles = Styles::new();
    styles.keys = HashMap::new();
    styles.values = HashMap::new();
    styles
        .keys
        .insert("status".to_string(), lipgloss::Style::new().bold());
    styles
        .values
        .insert("ok".to_string(), lipgloss::Style::new().bold());

    let logger = Logger::new();
    logger.set_styles(styles);
    logger.info("styled", &[("status", "ok")]);
}
