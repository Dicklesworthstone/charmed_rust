//! Unit tests for bubbletea error types.
//!
//! Tests verify:
//! - Error variant creation
//! - Display formatting
//! - Error chaining (source)
//! - From implementations
//! - Result type alias

use bubbletea::{Error, Result};
use std::error::Error as StdError;
use std::io;

mod creation_tests {
    use super::*;

    #[test]
    fn test_io_error_variant() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let e = Error::Io(io_err);
        assert!(matches!(e, Error::Io(_)));
    }

    #[test]
    fn test_all_io_error_kinds() {
        // Test various io::ErrorKind values can be wrapped
        let kinds = [
            io::ErrorKind::NotFound,
            io::ErrorKind::PermissionDenied,
            io::ErrorKind::ConnectionRefused,
            io::ErrorKind::BrokenPipe,
            io::ErrorKind::Other,
        ];

        for kind in kinds {
            let io_err = io::Error::new(kind, "test");
            let e = Error::Io(io_err);
            assert!(matches!(e, Error::Io(_)));
        }
    }
}

mod display_tests {
    use super::*;

    #[test]
    fn test_io_error_display() {
        let io_err = io::Error::other("terminal disconnected");
        let e = Error::Io(io_err);
        let msg = format!("{e}");

        assert!(msg.contains("terminal io error"));
        assert!(msg.contains("terminal disconnected"));
    }

    #[test]
    fn test_debug_impl() {
        let io_err = io::Error::other("test error");
        let e = Error::Io(io_err);
        let debug = format!("{e:?}");

        assert!(debug.contains("Io"));
    }
}

mod chaining_tests {
    use super::*;

    #[test]
    fn test_io_error_has_source() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let e = Error::Io(io_err);

        assert!(e.source().is_some());
        let source = e.source().unwrap();
        assert!(source.to_string().contains("not found"));
    }

    #[test]
    fn test_source_chain_walkable() {
        let io_err = io::Error::other("root cause");
        let e = Error::Io(io_err);

        // Walk the error chain
        let mut current: &dyn StdError = &e;
        let mut depth = 0;
        while let Some(source) = current.source() {
            current = source;
            depth += 1;
        }

        assert_eq!(depth, 1); // Error -> io::Error
    }
}

mod from_tests {
    use super::*;

    #[test]
    fn test_from_io_error() {
        let io_err = io::Error::other("test");
        let e: Error = io_err.into();

        assert!(matches!(e, Error::Io(_)));
    }

    #[test]
    fn test_question_mark_propagation() {
        fn may_fail() -> Result<()> {
            let _file = std::fs::File::open("/nonexistent/path/that/does/not/exist")?;
            Ok(())
        }

        let result = may_fail();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Io(_)));
    }

    #[test]
    fn test_into_io_error() {
        // Verify we can convert io::Error to Error using Into
        let io_err = io::Error::new(io::ErrorKind::BrokenPipe, "pipe broken");
        let e: Error = io_err.into();
        assert!(e.to_string().contains("pipe broken"));
    }
}

mod result_tests {
    use super::*;

    #[test]
    #[allow(clippy::unnecessary_wraps)]
    fn test_result_alias_ok() {
        fn do_something() -> Result<i32> {
            Ok(42)
        }

        assert_eq!(do_something().unwrap(), 42);
    }

    #[test]
    fn test_result_alias_err() {
        fn do_something() -> Result<i32> {
            let io_err = io::Error::other("failed");
            Err(Error::Io(io_err))
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
            let io_err = io::Error::other("inner failed");
            Err(Error::Io(io_err))
        }

        assert!(outer().is_err());
    }
}
