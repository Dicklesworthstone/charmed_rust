//! Unit tests for wish error types.
//!
//! Tests verify:
//! - Error variant creation
//! - Display formatting
//! - Error chaining (source)
//! - From implementations
//! - Result type alias

use std::error::Error as StdError;
use std::io;
use std::net::AddrParseError;
use wish::{Error, Result};

mod creation_tests {
    use super::*;

    #[test]
    fn test_io_error_variant() {
        let io_err = io::Error::new(io::ErrorKind::AddrInUse, "port in use");
        let e = Error::Io(io_err);
        assert!(matches!(e, Error::Io(_)));
    }

    #[test]
    fn test_ssh_error_variant() {
        let e = Error::Ssh("protocol mismatch".into());
        assert!(matches!(e, Error::Ssh(_)));
    }

    #[test]
    fn test_key_error_variant() {
        let e = Error::Key("invalid key format".into());
        assert!(matches!(e, Error::Key(_)));
    }

    #[test]
    fn test_authentication_failed_variant() {
        let e = Error::AuthenticationFailed;
        assert!(matches!(e, Error::AuthenticationFailed));
    }

    #[test]
    fn test_configuration_error_variant() {
        let e = Error::Configuration("invalid port".into());
        assert!(matches!(e, Error::Configuration(_)));
    }

    #[test]
    fn test_session_error_variant() {
        let e = Error::Session("connection dropped".into());
        assert!(matches!(e, Error::Session(_)));
    }

    #[test]
    fn test_addr_parse_error_variant() {
        let addr_err: AddrParseError = "invalid".parse::<std::net::SocketAddr>().unwrap_err();
        let e = Error::AddrParse(addr_err);
        assert!(matches!(e, Error::AddrParse(_)));
    }
}

mod display_tests {
    use super::*;

    #[test]
    fn test_io_error_display() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let e = Error::Io(io_err);
        let msg = format!("{}", e);
        assert!(msg.contains("io error"));
        assert!(msg.contains("access denied"));
    }

    #[test]
    fn test_ssh_error_display() {
        let e = Error::Ssh("handshake failed".into());
        let msg = format!("{}", e);
        assert!(msg.contains("ssh error"));
        assert!(msg.contains("handshake failed"));
    }

    #[test]
    fn test_key_error_display() {
        let e = Error::Key("invalid pem format".into());
        let msg = format!("{}", e);
        assert!(msg.contains("key error"));
        assert!(msg.contains("invalid pem format"));
    }

    #[test]
    fn test_authentication_failed_display() {
        let e = Error::AuthenticationFailed;
        let msg = format!("{}", e);
        assert!(msg.contains("authentication failed"));
    }

    #[test]
    fn test_configuration_error_display() {
        let e = Error::Configuration("missing host key".into());
        let msg = format!("{}", e);
        assert!(msg.contains("configuration error"));
        assert!(msg.contains("missing host key"));
    }

    #[test]
    fn test_session_error_display() {
        let e = Error::Session("channel closed".into());
        let msg = format!("{}", e);
        assert!(msg.contains("session error"));
        assert!(msg.contains("channel closed"));
    }

    #[test]
    fn test_addr_parse_error_display() {
        let addr_err: AddrParseError = "not-an-addr".parse::<std::net::SocketAddr>().unwrap_err();
        let e = Error::AddrParse(addr_err);
        let msg = format!("{}", e);
        assert!(msg.contains("address parse error"));
    }

    #[test]
    fn test_debug_impl() {
        let e = Error::Ssh("test".into());
        let debug = format!("{:?}", e);
        assert!(debug.contains("Ssh"));
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
    fn test_addr_parse_has_source() {
        let addr_err: AddrParseError = "bad".parse::<std::net::SocketAddr>().unwrap_err();
        let e = Error::AddrParse(addr_err);

        assert!(e.source().is_some());
    }

    #[test]
    fn test_string_variants_no_source() {
        // Variants with String payloads don't have source
        let errors = [
            Error::Ssh("test".into()),
            Error::Key("test".into()),
            Error::Configuration("test".into()),
            Error::Session("test".into()),
        ];

        for e in errors {
            assert!(e.source().is_none());
        }
    }

    #[test]
    fn test_authentication_failed_no_source() {
        let e = Error::AuthenticationFailed;
        assert!(e.source().is_none());
    }
}

mod from_tests {
    use super::*;

    #[test]
    fn test_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::Other, "test");
        let e: Error = io_err.into();
        assert!(matches!(e, Error::Io(_)));
    }

    #[test]
    fn test_from_addr_parse_error() {
        let addr_err: AddrParseError = "x".parse::<std::net::SocketAddr>().unwrap_err();
        let e: Error = addr_err.into();
        assert!(matches!(e, Error::AddrParse(_)));
    }

    #[test]
    fn test_question_mark_io() {
        fn may_fail() -> Result<()> {
            let _file = std::fs::File::open("/nonexistent/wish/path")?;
            Ok(())
        }

        let result = may_fail();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Io(_)));
    }

    #[test]
    fn test_question_mark_addr_parse() {
        fn parse_addr() -> Result<std::net::SocketAddr> {
            Ok("invalid-addr".parse()?)
        }

        let result = parse_addr();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::AddrParse(_)));
    }
}

mod result_tests {
    use super::*;

    #[test]
    fn test_result_alias_ok() {
        fn do_something() -> Result<u16> {
            Ok(22)
        }

        assert_eq!(do_something().unwrap(), 22);
    }

    #[test]
    fn test_result_alias_err() {
        fn do_something() -> Result<()> {
            Err(Error::AuthenticationFailed)
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
            Err(Error::Configuration("bad config".into()))
        }

        let result = outer();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Configuration(_)));
    }
}
