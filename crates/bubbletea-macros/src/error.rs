//! Error handling utilities for the proc-macro.
//!
//! This module provides error types and utilities for generating
//! helpful compile-time error messages.

use proc_macro2::TokenStream;
use quote::quote_spanned;

/// Error type for macro processing failures.
///
/// This type wraps various error conditions that can occur during
/// macro expansion and provides methods for converting them to
/// compile-time error messages.
#[derive(Debug)]
pub enum MacroError {
    /// Failed to parse the input syntax.
    Parse(syn::Error),

    /// Missing a required attribute (e.g., `#[init]`).
    #[allow(dead_code)] // Used in tests and for future macro expansion
    MissingAttribute {
        name: &'static str,
        span: proc_macro2::Span,
    },

    /// Invalid attribute usage.
    #[allow(dead_code)] // Used in tests and for future macro expansion
    InvalidAttribute {
        message: String,
        span: proc_macro2::Span,
    },

    /// The macro was applied to an unsupported item type.
    #[allow(dead_code)] // Used in tests and for future macro expansion
    UnsupportedItem {
        expected: &'static str,
        span: proc_macro2::Span,
    },
}

impl MacroError {
    /// Creates a "missing attribute" error.
    #[allow(dead_code)] // Used in tests and for future macro expansion
    pub fn missing_attribute(name: &'static str, span: proc_macro2::Span) -> Self {
        Self::MissingAttribute { name, span }
    }

    /// Creates an "invalid attribute" error.
    #[allow(dead_code)] // Used in tests and for future macro expansion
    pub fn invalid_attribute(message: impl Into<String>, span: proc_macro2::Span) -> Self {
        Self::InvalidAttribute {
            message: message.into(),
            span,
        }
    }

    /// Creates an "unsupported item" error.
    #[allow(dead_code)] // Used in tests and for future macro expansion
    pub fn unsupported_item(expected: &'static str, span: proc_macro2::Span) -> Self {
        Self::UnsupportedItem { expected, span }
    }

    /// Converts this error into a compile-time error token stream.
    pub fn to_compile_error(&self) -> TokenStream {
        match self {
            Self::Parse(err) => err.to_compile_error(),

            Self::MissingAttribute { name, span } => {
                let message = format!("Missing #[{name}] method on Model struct");
                quote_spanned! {*span=>
                    compile_error!(#message);
                }
            }

            Self::InvalidAttribute { message, span } => {
                quote_spanned! {*span=>
                    compile_error!(#message);
                }
            }

            Self::UnsupportedItem { expected, span } => {
                let message = format!("#[derive(Model)] can only be applied to {expected}");
                quote_spanned! {*span=>
                    compile_error!(#message);
                }
            }
        }
    }
}

impl From<syn::Error> for MacroError {
    fn from(err: syn::Error) -> Self {
        Self::Parse(err)
    }
}

impl std::fmt::Display for MacroError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(err) => write!(f, "Parse error: {err}"),
            Self::MissingAttribute { name, .. } => {
                write!(f, "Missing #{name} method on Model struct")
            }
            Self::InvalidAttribute { message, .. } => write!(f, "Invalid attribute: {message}"),
            Self::UnsupportedItem { expected, .. } => {
                write!(f, "#[derive(Model)] can only be applied to {expected}")
            }
        }
    }
}

impl std::error::Error for MacroError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(err) => Some(err),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;

    #[test]
    fn test_missing_attribute_error() {
        let err = MacroError::missing_attribute("init", Span::call_site());
        let error_str = err.to_string();
        assert!(error_str.contains("init"));
        assert!(error_str.contains("Missing"));
    }

    #[test]
    fn test_invalid_attribute_error() {
        let err = MacroError::invalid_attribute("bad syntax", Span::call_site());
        let error_str = err.to_string();
        assert!(error_str.contains("bad syntax"));
    }

    #[test]
    fn test_unsupported_item_error() {
        let err = MacroError::unsupported_item("named structs", Span::call_site());
        let error_str = err.to_string();
        assert!(error_str.contains("named structs"));
    }
}
