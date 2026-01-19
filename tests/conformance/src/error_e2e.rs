//! E2E tests for error propagation across charmed_rust crates.
//!
//! These tests verify that errors:
//! - Propagate correctly across crate boundaries
//! - Maintain source chains when converted
//! - Can be pattern matched for recovery
//! - Have helpful display messages
//!
//! Note: Tests cover actual error types in the codebase:
//! - bubbletea::Error (Io)
//! - huh::FormError (UserAborted, Timeout, Validation, Io)
//! - wish::Error (multiple variants) - requires `wish` feature
//! - charmed_log::ParseLevelError

use std::error::Error as StdError;
use std::io;

/// Tests for cross-crate io::Error propagation
mod io_error_propagation {
    use super::*;

    #[test]
    fn test_io_error_to_bubbletea() {
        use bubbletea::Error;

        let io_err = io::Error::new(io::ErrorKind::NotFound, "config file not found");
        let bt_err: Error = io_err.into();

        // Should match as Io variant
        assert!(matches!(bt_err, Error::Io(_)));

        // Source chain should be intact
        assert!(bt_err.source().is_some());

        // Display should be informative
        let msg = bt_err.to_string();
        assert!(msg.contains("terminal io error"));
        assert!(msg.contains("config file not found"));
    }

    #[cfg(feature = "wish")]
    #[test]
    fn test_io_error_to_wish() {
        use wish::Error;

        let io_err = io::Error::new(io::ErrorKind::AddrInUse, "port 22 in use");
        let wish_err: Error = io_err.into();

        assert!(matches!(wish_err, Error::Io(_)));
        assert!(wish_err.source().is_some());
        assert!(wish_err.to_string().contains("port 22 in use"));
    }

    #[test]
    fn test_io_error_chain_walkable() {
        use bubbletea::Error;

        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "cannot open /dev/tty");
        let bt_err: Error = io_err.into();

        // Walk the error chain
        let mut depth = 0;
        let mut current: &dyn StdError = &bt_err;
        while let Some(source) = current.source() {
            depth += 1;
            current = source;
        }

        assert_eq!(depth, 1, "Should have exactly one source level");
    }
}

/// Tests for huh::FormError in application contexts
mod form_error_handling {
    #![allow(unused_imports)]
    use super::*;

    #[test]
    fn test_user_abort_is_not_error() {
        use huh::FormError;

        fn process_form_result(result: Result<(), FormError>) -> &'static str {
            match result {
                Ok(()) => "completed",
                Err(FormError::UserAborted) => "cancelled", // Normal path
                Err(_) => "error",
            }
        }

        assert_eq!(
            process_form_result(Err(FormError::UserAborted)),
            "cancelled"
        );
        assert_eq!(process_form_result(Ok(())), "completed");
    }

    #[test]
    fn test_validation_error_recovery() {
        use huh::FormError;

        fn validate_email(input: &str) -> Result<(), FormError> {
            if input.contains('@') {
                Ok(())
            } else {
                Err(FormError::validation("must contain @"))
            }
        }

        let result = validate_email("invalid");
        assert!(result.is_err());

        if let Err(FormError::Validation(msg)) = result {
            assert!(msg.contains("@"));
        } else {
            panic!("Expected Validation error");
        }
    }

    #[test]
    fn test_form_error_equality() {
        use huh::FormError;

        // FormError implements PartialEq for testing
        let e1 = FormError::Timeout;
        let e2 = FormError::Timeout;
        assert_eq!(e1, e2);

        let e3 = FormError::UserAborted;
        assert_ne!(e1, e3);
    }
}

/// Tests for wish::Error categorization
#[cfg(feature = "wish")]
mod wish_error_categorization {
    use super::*;

    #[test]
    fn test_expected_auth_failure() {
        use wish::Error;

        fn handle_connection_error(err: Error) -> &'static str {
            match err {
                Error::AuthenticationFailed => "auth_failed", // Expected, not a bug
                Error::Io(_) => "network_error",
                Error::Ssh(_) => "protocol_error",
                _ => "other",
            }
        }

        assert_eq!(
            handle_connection_error(Error::AuthenticationFailed),
            "auth_failed"
        );
    }

    #[test]
    fn test_config_vs_runtime_errors() {
        use wish::Error;

        fn is_config_error(err: &Error) -> bool {
            matches!(
                err,
                Error::Configuration(_) | Error::Key(_) | Error::KeyLoad(_)
            )
        }

        fn is_runtime_error(err: &Error) -> bool {
            matches!(err, Error::Io(_) | Error::Session(_) | Error::Ssh(_))
        }

        let config_err = Error::Configuration("invalid port".into());
        assert!(is_config_error(&config_err));
        assert!(!is_runtime_error(&config_err));

        let runtime_err = Error::Session("connection dropped".into());
        assert!(is_runtime_error(&runtime_err));
        assert!(!is_config_error(&runtime_err));
    }

    #[test]
    fn test_from_implementations() {
        use wish::Error;

        // From<io::Error>
        let io_err = io::Error::new(io::ErrorKind::Other, "test");
        let _: Error = io_err.into();

        // From<AddrParseError>
        let addr_err: std::net::AddrParseError =
            "bad".parse::<std::net::SocketAddr>().unwrap_err();
        let _: Error = addr_err.into();
    }
}

/// Tests for charmed_log::ParseLevelError
mod log_level_error {
    #![allow(unused_imports)]
    use super::*;

    #[test]
    fn test_parse_level_error_display() {
        use charmed_log::Level;
        use std::str::FromStr;

        let result = Level::from_str("INVALID_LEVEL");
        assert!(result.is_err());

        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("invalid level"));
        assert!(msg.contains("INVALID_LEVEL"));
    }

    #[test]
    fn test_parse_level_in_config_context() {
        use charmed_log::{Level, ParseResult};
        use std::str::FromStr;

        fn parse_config_level(s: &str) -> ParseResult<Level> {
            Ok(Level::from_str(s)?)
        }

        // Valid levels work
        assert!(parse_config_level("info").is_ok());
        assert!(parse_config_level("DEBUG").is_ok());

        // Invalid levels propagate error
        let result = parse_config_level("verbose");
        assert!(result.is_err());
    }
}

/// Tests for error source chain preservation
mod source_chain_preservation {
    use super::*;

    #[test]
    fn test_bubbletea_io_source_preserved() {
        use bubbletea::Error;

        let original_msg = "underlying cause";
        let io_err = io::Error::new(io::ErrorKind::Other, original_msg);
        let bt_err: Error = io_err.into();

        // Get source
        let source = bt_err.source().expect("should have source");

        // Source should contain original message
        assert!(source.to_string().contains(original_msg));
    }

    #[cfg(feature = "wish")]
    #[test]
    fn test_wish_io_source_preserved() {
        use wish::Error;

        let original_msg = "socket reset by peer";
        let io_err = io::Error::new(io::ErrorKind::ConnectionReset, original_msg);
        let wish_err: Error = io_err.into();

        let source = wish_err.source().expect("should have source");
        assert!(source.to_string().contains(original_msg));
    }

    #[test]
    fn test_error_chain_depth() {
        use bubbletea::Error;

        // Create a chain: bt_err -> io_err
        let io_err = io::Error::new(io::ErrorKind::Other, "root");
        let bt_err: Error = io_err.into();

        // Measure depth
        let mut depth = 0;
        let mut current: Option<&(dyn StdError + 'static)> = Some(&bt_err);

        while let Some(err) = current {
            depth += 1;
            current = err.source();
        }

        assert_eq!(depth, 2, "Chain should be: bt_err -> io_err");
    }
}

/// Tests for error recovery patterns
mod error_recovery {
    use super::*;

    #[test]
    fn test_retry_on_io_error() {
        use bubbletea::Error;

        fn with_retry<F>(mut f: F, max_retries: u32) -> Result<&'static str, Error>
        where
            F: FnMut() -> Result<&'static str, Error>,
        {
            let mut last_err = None;
            for _ in 0..max_retries {
                match f() {
                    Ok(v) => return Ok(v),
                    Err(e) => last_err = Some(e),
                }
            }
            Err(last_err
                .unwrap_or_else(|| Error::Io(io::Error::new(io::ErrorKind::Other, "no attempts"))))
        }

        // Simulate a flaky operation that succeeds on 3rd try
        let mut attempts = 0;
        let result = with_retry(
            || {
                attempts += 1;
                if attempts < 3 {
                    Err(Error::Io(io::Error::new(io::ErrorKind::Other, "retry")))
                } else {
                    Ok("success")
                }
            },
            5,
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[test]
    fn test_fallback_on_error() {
        use huh::FormError;

        fn get_input_with_fallback(result: Result<String, FormError>, default: &str) -> String {
            match result {
                Ok(s) => s,
                Err(FormError::UserAborted) | Err(FormError::Timeout) => default.to_string(),
                Err(e) => panic!("unexpected error: {e}"),
            }
        }

        assert_eq!(
            get_input_with_fallback(Err(FormError::UserAborted), "default"),
            "default"
        );
        assert_eq!(
            get_input_with_fallback(Ok("user_input".into()), "default"),
            "user_input"
        );
    }
}

/// Tests for debug output quality
mod debug_output {
    use super::*;

    #[test]
    fn test_bubbletea_error_debug() {
        use bubbletea::Error;

        let io_err = io::Error::new(io::ErrorKind::NotFound, "file.txt");
        let bt_err: Error = io_err.into();

        let debug = format!("{:?}", bt_err);
        assert!(debug.contains("Io"));
        assert!(debug.contains("NotFound"));
    }

    #[cfg(feature = "wish")]
    #[test]
    fn test_wish_error_debug() {
        use wish::Error;

        let err = Error::Configuration("invalid setting".into());
        let debug = format!("{:?}", err);

        assert!(debug.contains("Configuration"));
        assert!(debug.contains("invalid setting"));
    }

    #[test]
    fn test_huh_error_debug() {
        use huh::FormError;

        let err = FormError::Validation("field required".into());
        let debug = format!("{:?}", err);

        assert!(debug.contains("Validation"));
        assert!(debug.contains("field required"));
    }
}
