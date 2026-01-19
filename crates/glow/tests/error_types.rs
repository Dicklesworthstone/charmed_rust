//! Unit tests for glow error types.
//!
//! Tests verify:
//! - Error variant creation (ParseError and FetchError)
//! - Display formatting
//! - Error chaining (source)
//! - From implementations
//! - Result type aliases
//!
//! Requires the `github` feature to be enabled.

#![cfg(feature = "github")]

use glow::github::{FetchError, FetchResult, ParseError, ParseResult};
use std::error::Error as StdError;
use std::io;

mod parse_error_creation_tests {
    use super::*;

    #[test]
    fn test_invalid_format_variant() {
        let e = ParseError::InvalidFormat;
        assert!(matches!(e, ParseError::InvalidFormat));
    }

    #[test]
    fn test_missing_owner_or_repo_variant() {
        let e = ParseError::MissingOwnerOrRepo;
        assert!(matches!(e, ParseError::MissingOwnerOrRepo));
    }

    #[test]
    fn test_all_parse_variants() {
        let errors = [ParseError::InvalidFormat, ParseError::MissingOwnerOrRepo];
        assert_eq!(errors.len(), 2);
    }
}

mod parse_error_display_tests {
    use super::*;

    #[test]
    fn test_invalid_format_display() {
        let e = ParseError::InvalidFormat;
        let msg = format!("{}", e);
        assert!(msg.contains("invalid repository format"));
    }

    #[test]
    fn test_missing_owner_or_repo_display() {
        let e = ParseError::MissingOwnerOrRepo;
        let msg = format!("{}", e);
        assert!(msg.contains("missing owner or repository name"));
    }

    #[test]
    fn test_debug_impl() {
        let e = ParseError::InvalidFormat;
        let debug = format!("{:?}", e);
        assert!(debug.contains("InvalidFormat"));
    }
}

mod parse_error_derives_tests {
    use super::*;

    #[test]
    fn test_clone() {
        let e1 = ParseError::InvalidFormat;
        let e2 = e1.clone();
        assert_eq!(e1, e2);
    }

    #[test]
    fn test_partial_eq() {
        assert_eq!(ParseError::InvalidFormat, ParseError::InvalidFormat);
        assert_eq!(
            ParseError::MissingOwnerOrRepo,
            ParseError::MissingOwnerOrRepo
        );
        assert_ne!(ParseError::InvalidFormat, ParseError::MissingOwnerOrRepo);
    }
}

mod fetch_error_creation_tests {
    use super::*;

    #[test]
    fn test_api_error_variant() {
        let e = FetchError::ApiError {
            status: 404,
            message: "Not Found".into(),
        };
        assert!(matches!(e, FetchError::ApiError { .. }));
    }

    #[test]
    fn test_decode_error_variant() {
        let e = FetchError::DecodeError("invalid base64".into());
        assert!(matches!(e, FetchError::DecodeError(_)));
    }

    #[test]
    fn test_rate_limited_with_reset() {
        let e = FetchError::RateLimited {
            reset_at: Some(1700000000),
        };
        assert!(matches!(e, FetchError::RateLimited { .. }));
    }

    #[test]
    fn test_rate_limited_without_reset() {
        let e = FetchError::RateLimited { reset_at: None };
        assert!(matches!(e, FetchError::RateLimited { reset_at: None }));
    }

    #[test]
    fn test_cache_error_variant() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "cannot write cache");
        let e = FetchError::CacheError(io_err);
        assert!(matches!(e, FetchError::CacheError(_)));
    }
}

mod fetch_error_display_tests {
    use super::*;

    #[test]
    fn test_api_error_display() {
        let e = FetchError::ApiError {
            status: 404,
            message: "Not Found".into(),
        };
        let msg = format!("{}", e);
        assert!(msg.contains("API error"));
        assert!(msg.contains("404"));
        assert!(msg.contains("Not Found"));
    }

    #[test]
    fn test_api_error_display_various_statuses() {
        let statuses = [(403, "Forbidden"), (500, "Internal Server Error")];

        for (status, message) in statuses {
            let e = FetchError::ApiError {
                status,
                message: message.into(),
            };
            let msg = format!("{}", e);
            assert!(msg.contains(&status.to_string()));
            assert!(msg.contains(message));
        }
    }

    #[test]
    fn test_decode_error_display() {
        let e = FetchError::DecodeError("not valid base64".into());
        let msg = format!("{}", e);
        assert!(msg.contains("decode error"));
        assert!(msg.contains("not valid base64"));
    }

    #[test]
    fn test_rate_limited_with_reset_display() {
        let e = FetchError::RateLimited {
            reset_at: Some(1700000000),
        };
        let msg = format!("{}", e);
        assert!(msg.contains("rate limited"));
        assert!(msg.contains("1700000000"));
    }

    #[test]
    fn test_rate_limited_without_reset_display() {
        let e = FetchError::RateLimited { reset_at: None };
        let msg = format!("{}", e);
        assert!(msg.contains("rate limited"));
        assert!(!msg.contains("resets at"));
    }

    #[test]
    fn test_cache_error_display() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "cache directory missing");
        let e = FetchError::CacheError(io_err);
        let msg = format!("{}", e);
        assert!(msg.contains("cache error"));
        assert!(msg.contains("cache directory missing"));
    }

    #[test]
    fn test_debug_impl() {
        let e = FetchError::DecodeError("test".into());
        let debug = format!("{:?}", e);
        assert!(debug.contains("DecodeError"));
    }
}

mod fetch_error_chaining_tests {
    use super::*;

    #[test]
    fn test_cache_error_has_source() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "not found");
        let e = FetchError::CacheError(io_err);

        assert!(e.source().is_some());
        let source = e.source().unwrap();
        assert!(source.to_string().contains("not found"));
    }

    #[test]
    fn test_simple_variants_no_source() {
        let errors = [
            FetchError::ApiError {
                status: 500,
                message: "error".into(),
            },
            FetchError::DecodeError("error".into()),
            FetchError::RateLimited { reset_at: None },
        ];

        for e in errors {
            assert!(e.source().is_none());
        }
    }
}

mod fetch_error_from_tests {
    use super::*;

    #[test]
    fn test_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::Other, "test");
        let e: FetchError = io_err.into();
        assert!(matches!(e, FetchError::CacheError(_)));
    }

    #[test]
    fn test_question_mark_io() {
        fn may_fail() -> FetchResult<()> {
            let _file = std::fs::File::open("/nonexistent/glow/path")?;
            Ok(())
        }

        let result = may_fail();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FetchError::CacheError(_)));
    }
}

mod result_tests {
    use super::*;

    #[test]
    fn test_parse_result_ok() {
        fn parse_something() -> ParseResult<String> {
            Ok("owner/repo".into())
        }

        assert_eq!(parse_something().unwrap(), "owner/repo");
    }

    #[test]
    fn test_parse_result_err() {
        fn parse_something() -> ParseResult<String> {
            Err(ParseError::InvalidFormat)
        }

        assert!(parse_something().is_err());
    }

    #[test]
    fn test_fetch_result_ok() {
        fn fetch_something() -> FetchResult<String> {
            Ok("# README".into())
        }

        assert_eq!(fetch_something().unwrap(), "# README");
    }

    #[test]
    fn test_fetch_result_err() {
        fn fetch_something() -> FetchResult<String> {
            Err(FetchError::RateLimited { reset_at: None })
        }

        assert!(fetch_something().is_err());
    }

    #[test]
    fn test_result_error_propagation() {
        fn outer() -> ParseResult<()> {
            inner()?;
            Ok(())
        }

        fn inner() -> ParseResult<()> {
            Err(ParseError::MissingOwnerOrRepo)
        }

        let result = outer();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ParseError::MissingOwnerOrRepo
        ));
    }
}
