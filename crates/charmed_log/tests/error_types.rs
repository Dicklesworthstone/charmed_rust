//! Unit tests for charmed_log error types.
//!
//! Tests verify:
//! - Error creation
//! - Display formatting
//! - Clone derive
//! - FromStr integration
//! - Result type alias

use charmed_log::{Level, ParseLevelError, ParseResult};
use std::error::Error as StdError;
use std::str::FromStr;

mod creation_tests {
    use super::*;

    #[test]
    fn test_parse_level_error_from_invalid_input() {
        let result = Level::from_str("invalid");
        assert!(result.is_err());
        let e = result.unwrap_err();
        assert!(matches!(e, ParseLevelError { .. }));
    }

    #[test]
    fn test_various_invalid_inputs() {
        let invalid_inputs = ["", "foobar", "123", "VERBOSE", "warning"];

        for input in invalid_inputs {
            let result = Level::from_str(input);
            assert!(result.is_err(), "Expected error for input: {}", input);
        }
    }
}

mod display_tests {
    use super::*;

    #[test]
    fn test_display_contains_invalid_value() {
        let result = Level::from_str("badlevel");
        let e = result.unwrap_err();
        let msg = format!("{}", e);
        assert!(msg.contains("invalid level"));
        assert!(msg.contains("badlevel"));
    }

    #[test]
    fn test_display_with_empty_string() {
        let result = Level::from_str("");
        let e = result.unwrap_err();
        let msg = format!("{}", e);
        assert!(msg.contains("invalid level"));
    }

    #[test]
    fn test_debug_impl() {
        let result = Level::from_str("xyz");
        let e = result.unwrap_err();
        let debug = format!("{:?}", e);
        assert!(debug.contains("ParseLevelError"));
    }
}

mod derive_tests {
    use super::*;

    #[test]
    fn test_clone() {
        let result = Level::from_str("bad");
        let e1 = result.unwrap_err();
        let e2 = e1.clone();
        assert_eq!(e1.to_string(), e2.to_string());
    }
}

mod chaining_tests {
    use super::*;

    #[test]
    fn test_no_source() {
        // ParseLevelError is a simple tuple struct, no source
        let result = Level::from_str("invalid");
        let e = result.unwrap_err();
        assert!(e.source().is_none());
    }
}

mod valid_levels_tests {
    use super::*;

    #[test]
    fn test_valid_levels_lowercase() {
        let valid = ["debug", "info", "warn", "error", "fatal"];

        for level in valid {
            let result = Level::from_str(level);
            assert!(result.is_ok(), "Expected OK for level: {}", level);
        }
    }

    #[test]
    fn test_valid_levels_uppercase() {
        let valid = ["DEBUG", "INFO", "WARN", "ERROR", "FATAL"];

        for level in valid {
            let result = Level::from_str(level);
            assert!(result.is_ok(), "Expected OK for level: {}", level);
        }
    }

    #[test]
    fn test_valid_levels_mixed_case() {
        let valid = ["Debug", "Info", "Warn", "Error", "Fatal"];

        for level in valid {
            let result = Level::from_str(level);
            assert!(result.is_ok(), "Expected OK for level: {}", level);
        }
    }
}

mod result_tests {
    use super::*;

    #[test]
    fn test_parse_result_ok() {
        fn parse_level(s: &str) -> ParseResult<Level> {
            Ok(Level::from_str(s)?)
        }

        let result = parse_level("info");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_result_err() {
        fn parse_level(s: &str) -> ParseResult<Level> {
            Ok(Level::from_str(s)?)
        }

        let result = parse_level("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_result_error_propagation() {
        fn outer() -> ParseResult<Level> {
            inner()
        }

        fn inner() -> ParseResult<Level> {
            Ok(Level::from_str("bad")?)
        }

        let result = outer();
        assert!(result.is_err());
    }
}
