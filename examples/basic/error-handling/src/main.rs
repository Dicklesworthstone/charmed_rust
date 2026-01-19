//! Error Handling Pattern Example
//!
//! This example demonstrates the unified error handling pattern
//! used across all charmed_rust crates, based on the `wish::Error`
//! reference implementation.
//!
//! See `docs/error-handling-guide.md` for the full specification.

use std::io;
use thiserror::Error;

// =============================================================================
// Pattern 1: Basic Error Enum with thiserror
// =============================================================================

/// Errors that can occur in the example application.
///
/// This follows the charmed_rust error pattern:
/// - `thiserror::Error` derive macro for Display and Error traits
/// - `#[from]` for automatic From implementations
/// - Doc comments on every variant
/// - Lowercase error messages
#[derive(Error, Debug)]
pub enum Error {
    /// I/O error from file operations.
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    /// Configuration error with context message.
    #[error("configuration error: {0}")]
    Configuration(String),

    /// Parse error at a specific location.
    #[error("parse error at line {line}: {message}")]
    Parse { line: usize, message: String },

    /// Operation was cancelled by the user.
    #[error("operation cancelled")]
    Cancelled,
}

/// Result type alias for ergonomic use.
pub type Result<T> = std::result::Result<T, Error>;

// =============================================================================
// Pattern 2: Clone + PartialEq Compatible Errors
// =============================================================================

/// Form errors that can be cloned and compared.
///
/// When Clone and PartialEq are required (e.g., for testing),
/// use String instead of io::Error which doesn't implement these.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum FormError {
    /// User aborted the form.
    #[error("user aborted")]
    UserAborted,

    /// Form timed out waiting for input.
    #[error("timeout")]
    Timeout,

    /// Validation failed with message.
    #[error("validation error: {0}")]
    Validation(String),

    /// I/O error (stored as String for Clone + PartialEq).
    #[error("io error: {0}")]
    Io(String),
}

// =============================================================================
// Pattern 3: Structured Errors with Complex Formatting
// =============================================================================

/// API-related errors with structured data.
#[derive(Error, Debug)]
pub enum ApiError {
    /// HTTP request failed.
    #[error("request failed: {0}")]
    Request(String),

    /// API returned an error response.
    #[error("API error ({status}): {message}")]
    Response { status: u16, message: String },

    /// Rate limited with optional reset time.
    #[error("rate limited{}", .reset_at.map(|ts| format!(", resets at {ts}")).unwrap_or_default())]
    RateLimited { reset_at: Option<u64> },
}

// =============================================================================
// Usage Examples
// =============================================================================

/// Demonstrates using the ? operator with #[from] conversion.
fn read_config(path: &str) -> Result<String> {
    // io::Error automatically converts to Error::Io via #[from]
    let content = std::fs::read_to_string(path)?;
    Ok(content)
}

/// Demonstrates creating domain-specific errors.
fn parse_config(content: &str) -> Result<()> {
    if content.is_empty() {
        return Err(Error::Configuration("empty configuration".to_string()));
    }

    if content.starts_with('#') {
        return Err(Error::Parse {
            line: 1,
            message: "unexpected comment at start".to_string(),
        });
    }

    Ok(())
}

/// Demonstrates integration with anyhow for application code.
fn run_app() -> anyhow::Result<()> {
    // thiserror errors work seamlessly with anyhow
    use anyhow::Context;

    let config = read_config("app.toml").context("failed to load configuration")?;

    parse_config(&config).context("invalid configuration format")?;

    Ok(())
}

/// Demonstrates error display formatting.
fn demonstrate_error_display() {
    // Basic error with wrapped value
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let err = Error::Io(io_err);
    println!("Display: {err}");
    // Output: "Display: io error: file not found"

    // Structured error with named fields
    let parse_err = Error::Parse {
        line: 42,
        message: "unexpected token".to_string(),
    };
    println!("Display: {parse_err}");
    // Output: "Display: parse error at line 42: unexpected token"

    // Sentinel error with no data
    let cancel_err = Error::Cancelled;
    println!("Display: {cancel_err}");
    // Output: "Display: operation cancelled"

    // API error with optional field
    let rate_err = ApiError::RateLimited {
        reset_at: Some(1705678900),
    };
    println!("Display: {rate_err}");
    // Output: "Display: rate limited, resets at 1705678900"

    let rate_err_no_time = ApiError::RateLimited { reset_at: None };
    println!("Display: {rate_err_no_time}");
    // Output: "Display: rate limited"
}

/// Demonstrates error source chaining.
fn demonstrate_error_chain() {
    use std::error::Error as StdError;

    let inner = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    let err = Error::Io(inner);

    // The source() method is automatically implemented by thiserror
    if let Some(source) = err.source() {
        println!("Error source: {source}");
    }
}

fn main() {
    println!("=== Error Display Examples ===");
    demonstrate_error_display();

    println!("\n=== Error Chain Example ===");
    demonstrate_error_chain();

    println!("\n=== Application Error Handling ===");
    match run_app() {
        Ok(()) => println!("Application ran successfully"),
        Err(e) => {
            // anyhow provides rich error formatting with chains
            println!("Error: {e:?}");
        }
    }

    println!("\n=== Clone + PartialEq Example ===");
    let err1 = FormError::Validation("required field missing".to_string());
    let err2 = err1.clone();
    assert_eq!(err1, err2);
    println!("FormError supports Clone + PartialEq");
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_format() {
        let err = Error::Configuration("test".to_string());
        assert_eq!(err.to_string(), "configuration error: test");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::Other, "test");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
        assert!(err.to_string().starts_with("io error:"));
    }

    #[test]
    fn test_structured_error_display() {
        let err = Error::Parse {
            line: 10,
            message: "syntax error".to_string(),
        };
        assert_eq!(err.to_string(), "parse error at line 10: syntax error");
    }

    #[test]
    fn test_sentinel_error_display() {
        let err = Error::Cancelled;
        assert_eq!(err.to_string(), "operation cancelled");
    }

    #[test]
    fn test_form_error_clone_eq() {
        let err1 = FormError::UserAborted;
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_api_error_optional_field() {
        let err_with = ApiError::RateLimited {
            reset_at: Some(12345),
        };
        assert!(err_with.to_string().contains("12345"));

        let err_without = ApiError::RateLimited { reset_at: None };
        assert!(!err_without.to_string().contains("resets at"));
    }

    #[test]
    fn test_error_source_chain() {
        use std::error::Error as StdError;

        let inner = io::Error::new(io::ErrorKind::Other, "inner error");
        let err = Error::Io(inner);

        let source = err.source().expect("should have source");
        assert!(source.to_string().contains("inner error"));
    }
}
