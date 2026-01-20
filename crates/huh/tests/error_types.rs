//! Unit tests for huh error types.
//!
//! Tests verify:
//! - Error variant creation
//! - Display formatting
//! - Clone and `PartialEq` derives
//! - Helper methods
//! - Result type alias

use huh::{FormError, Result};
use std::error::Error as StdError;

mod creation_tests {
    use super::*;

    #[test]
    fn test_user_aborted_variant() {
        let e = FormError::UserAborted;
        assert!(matches!(e, FormError::UserAborted));
    }

    #[test]
    fn test_timeout_variant() {
        let e = FormError::Timeout;
        assert!(matches!(e, FormError::Timeout));
    }

    #[test]
    fn test_validation_variant() {
        let e = FormError::Validation("email must contain @".into());
        assert!(matches!(e, FormError::Validation(_)));
    }

    #[test]
    fn test_io_variant() {
        let e = FormError::Io("terminal not available".into());
        assert!(matches!(e, FormError::Io(_)));
    }

    #[test]
    fn test_all_variants_creatable() {
        let errors = [
            FormError::UserAborted,
            FormError::Timeout,
            FormError::Validation("test".into()),
            FormError::Io("test".into()),
        ];

        assert_eq!(errors.len(), 4);
    }
}

mod display_tests {
    use super::*;

    #[test]
    fn test_user_aborted_display() {
        let e = FormError::UserAborted;
        let msg = format!("{e}");
        assert_eq!(msg, "user aborted");
    }

    #[test]
    fn test_timeout_display() {
        let e = FormError::Timeout;
        let msg = format!("{e}");
        assert_eq!(msg, "timeout");
    }

    #[test]
    fn test_validation_display() {
        let e = FormError::Validation("must be at least 8 characters".into());
        let msg = format!("{e}");
        assert!(msg.contains("validation error"));
        assert!(msg.contains("must be at least 8 characters"));
    }

    #[test]
    fn test_io_display() {
        let e = FormError::Io("stdin not a tty".into());
        let msg = format!("{e}");
        assert!(msg.contains("io error"));
        assert!(msg.contains("stdin not a tty"));
    }

    #[test]
    fn test_debug_impl() {
        let e = FormError::Validation("test".into());
        let debug = format!("{e:?}");
        assert!(debug.contains("Validation"));
    }
}

mod derives_tests {
    use super::*;

    #[test]
    fn test_clone() {
        let e1 = FormError::Validation("test".into());
        let e2 = e1.clone();
        assert_eq!(e1, e2);
    }

    #[test]
    fn test_partial_eq() {
        assert_eq!(FormError::UserAborted, FormError::UserAborted);
        assert_eq!(FormError::Timeout, FormError::Timeout);
        assert_eq!(
            FormError::Validation("test".into()),
            FormError::Validation("test".into())
        );
        assert_ne!(
            FormError::Validation("a".into()),
            FormError::Validation("b".into())
        );
        assert_ne!(FormError::UserAborted, FormError::Timeout);
    }
}

mod helper_methods_tests {
    use super::*;

    #[test]
    fn test_validation_helper() {
        let e = FormError::validation("invalid email");
        assert!(matches!(e, FormError::Validation(_)));
        assert!(e.to_string().contains("invalid email"));
    }

    #[test]
    fn test_io_helper() {
        let e = FormError::io("terminal error");
        assert!(matches!(e, FormError::Io(_)));
        assert!(e.to_string().contains("terminal error"));
    }

    #[test]
    fn test_is_user_abort() {
        assert!(FormError::UserAborted.is_user_abort());
        assert!(!FormError::Timeout.is_user_abort());
        assert!(!FormError::Validation("x".into()).is_user_abort());
        assert!(!FormError::Io("x".into()).is_user_abort());
    }

    #[test]
    fn test_is_timeout() {
        assert!(FormError::Timeout.is_timeout());
        assert!(!FormError::UserAborted.is_timeout());
        assert!(!FormError::Validation("x".into()).is_timeout());
        assert!(!FormError::Io("x".into()).is_timeout());
    }
}

mod chaining_tests {
    use super::*;

    #[test]
    fn test_no_source_for_simple_variants() {
        // FormError uses String for Clone/PartialEq, so no source chaining
        let e = FormError::Io("test".into());
        assert!(e.source().is_none());

        let e = FormError::Validation("test".into());
        assert!(e.source().is_none());
    }
}

mod result_tests {
    use super::*;

    #[test]
    #[allow(clippy::unnecessary_wraps)]
    fn test_result_alias_ok() {
        fn do_something() -> Result<String> {
            Ok("success".into())
        }

        assert_eq!(do_something().unwrap(), "success");
    }

    #[test]
    fn test_result_alias_err() {
        fn do_something() -> Result<()> {
            Err(FormError::UserAborted)
        }

        assert!(do_something().is_err());
    }

    #[test]
    fn test_result_error_propagation() {
        fn outer() -> Result<()> {
            inner()?;
            Ok(())
        }

        fn inner() -> Result<()> {
            Err(FormError::Timeout)
        }

        let result = outer();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FormError::Timeout));
    }
}
