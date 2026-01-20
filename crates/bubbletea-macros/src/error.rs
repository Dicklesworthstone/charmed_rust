//! Error handling utilities for the proc-macro.
//!
//! This module provides error types and utilities for generating
//! helpful compile-time error messages with suggestions and examples.

use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;

/// Error type for macro processing failures.
///
/// This type wraps various error conditions that can occur during
/// macro expansion and provides methods for converting them to
/// compile-time error messages with helpful suggestions.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Variants are designed for future macro validation
pub enum MacroError {
    /// Failed to parse the input syntax.
    Parse(String, Span),

    /// Missing a required method (init, update, or view).
    MissingMethod {
        method: &'static str,
        struct_name: String,
        span: Span,
    },

    /// Duplicate method attribute found.
    DuplicateMethod { method: &'static str, span: Span },

    /// Method has wrong signature.
    WrongSignature {
        method: &'static str,
        expected: String,
        found: String,
        span: Span,
    },

    /// The macro was applied to an unsupported item type.
    UnsupportedItem {
        expected: &'static str,
        found: String,
        span: Span,
    },

    /// Invalid attribute syntax or arguments.
    InvalidAttribute {
        attr_name: &'static str,
        message: String,
        span: Span,
    },

    /// Field marked with #[state] doesn't meet requirements.
    InvalidStateField {
        field_name: String,
        reason: String,
        span: Span,
    },

    /// Generic type parameter is missing required bounds.
    MissingBounds {
        param: String,
        bounds: Vec<String>,
        span: Span,
    },
}

#[allow(dead_code)] // Constructor methods for future macro validation
impl MacroError {
    /// Creates a parse error.
    pub fn parse(message: impl Into<String>, span: Span) -> Self {
        Self::Parse(message.into(), span)
    }

    /// Creates a missing method error with helpful suggestion.
    pub fn missing_method(
        method: &'static str,
        struct_name: impl Into<String>,
        span: Span,
    ) -> Self {
        Self::MissingMethod {
            method,
            struct_name: struct_name.into(),
            span,
        }
    }

    /// Creates a duplicate method error.
    pub fn duplicate_method(method: &'static str, span: Span) -> Self {
        Self::DuplicateMethod { method, span }
    }

    /// Creates a wrong signature error.
    pub fn wrong_signature(
        method: &'static str,
        expected: impl Into<String>,
        found: impl Into<String>,
        span: Span,
    ) -> Self {
        Self::WrongSignature {
            method,
            expected: expected.into(),
            found: found.into(),
            span,
        }
    }

    /// Creates an unsupported item error.
    pub fn unsupported_item(expected: &'static str, found: impl Into<String>, span: Span) -> Self {
        Self::UnsupportedItem {
            expected,
            found: found.into(),
            span,
        }
    }

    /// Creates an invalid attribute error.
    pub fn invalid_attribute(
        attr_name: &'static str,
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        Self::InvalidAttribute {
            attr_name,
            message: message.into(),
            span,
        }
    }

    /// Creates an invalid state field error.
    pub fn invalid_state_field(
        field_name: impl Into<String>,
        reason: impl Into<String>,
        span: Span,
    ) -> Self {
        Self::InvalidStateField {
            field_name: field_name.into(),
            reason: reason.into(),
            span,
        }
    }

    /// Creates a missing bounds error.
    pub fn missing_bounds(param: impl Into<String>, bounds: Vec<String>, span: Span) -> Self {
        Self::MissingBounds {
            param: param.into(),
            bounds,
            span,
        }
    }

    /// Returns the span where this error occurred.
    pub fn span(&self) -> Span {
        match self {
            Self::Parse(_, span)
            | Self::MissingMethod { span, .. }
            | Self::DuplicateMethod { span, .. }
            | Self::WrongSignature { span, .. }
            | Self::UnsupportedItem { span, .. }
            | Self::InvalidAttribute { span, .. }
            | Self::InvalidStateField { span, .. }
            | Self::MissingBounds { span, .. } => *span,
        }
    }

    /// Generates the primary error message.
    fn primary_message(&self) -> String {
        match self {
            Self::Parse(msg, _) => format!("parse error: {msg}"),

            Self::MissingMethod {
                method,
                struct_name,
                ..
            } => {
                format!("struct `{struct_name}` is missing an `{method}` method")
            }

            Self::DuplicateMethod { method, .. } => {
                format!("duplicate `{method}` method found; only one is allowed")
            }

            Self::WrongSignature {
                method,
                expected,
                found,
                ..
            } => {
                format!(
                    "incorrect signature for `{method}` method\n  expected: {expected}\n  found: {found}"
                )
            }

            Self::UnsupportedItem {
                expected, found, ..
            } => {
                format!("#[derive(Model)] can only be applied to {expected}, found {found}")
            }

            Self::InvalidAttribute {
                attr_name, message, ..
            } => {
                format!("invalid #{attr_name} attribute: {message}")
            }

            Self::InvalidStateField {
                field_name, reason, ..
            } => {
                format!("field `{field_name}` cannot be used with #[state]: {reason}")
            }

            Self::MissingBounds { param, bounds, .. } => {
                let bounds_str = bounds.join(" + ");
                format!("type parameter `{param}` is missing required bounds: {bounds_str}")
            }
        }
    }

    /// Generates a help message with suggestions.
    fn help_message(&self) -> Option<String> {
        match self {
            Self::MissingMethod { method, .. } => {
                let example = match *method {
                    "init" => {
                        r#"Add an `init` method that returns an initial command:

    fn init(&self) -> Option<Cmd> {
        None // or Some(Cmd::batch([...]))
    }"#
                    }
                    "update" => {
                        r#"Add an `update` method that handles messages:

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle your messages here
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            // handle key press
        }
        None
    }"#
                    }
                    "view" => {
                        r#"Add a `view` method that returns a String:

    fn view(&self) -> String {
        format!("Your view here")
    }"#
                    }
                    _ => return None,
                };
                Some(example.to_string())
            }

            Self::DuplicateMethod { method, .. } => {
                Some(format!(
                    "Remove the duplicate `{method}` method. Each Model struct should have exactly one `{method}` method."
                ))
            }

            Self::WrongSignature { expected, .. } => {
                Some(format!(
                    "Change the method signature to match: {expected}"
                ))
            }

            Self::UnsupportedItem { expected, .. } => {
                Some(format!(
                    "Apply #[derive(Model)] to {expected} instead."
                ))
            }

            Self::InvalidAttribute { attr_name, .. } => {
                Some(format!(
                    "Check the documentation for valid #{attr_name} attribute syntax."
                ))
            }

            Self::InvalidStateField { .. } => {
                Some(
                    "Options:\n  1. Remove #[state] from this field\n  2. Make the field type implement Clone + PartialEq\n  3. Use #[state(skip)] to exclude from change detection"
                        .to_string(),
                )
            }

            Self::MissingBounds { param, bounds, .. } => {
                let bounds_str = bounds.join(" + ");
                Some(format!(
                    "Add the bounds to your type parameter: `{param}: {bounds_str}`"
                ))
            }

            Self::Parse(_, _) => None,
        }
    }

    /// Converts this error into a compile-time error token stream.
    pub fn to_compile_error(&self) -> TokenStream {
        let span = self.span();
        let message = self.primary_message();

        // Build the full error message with help text if available
        let full_message = if let Some(help) = self.help_message() {
            format!("{message}\n\nhelp: {help}")
        } else {
            message
        };

        quote_spanned! {span=>
            compile_error!(#full_message);
        }
    }
}

impl From<syn::Error> for MacroError {
    fn from(err: syn::Error) -> Self {
        Self::Parse(err.to_string(), err.span())
    }
}

impl From<darling::Error> for MacroError {
    fn from(err: darling::Error) -> Self {
        Self::Parse(err.to_string(), err.span())
    }
}

impl std::fmt::Display for MacroError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.primary_message())
    }
}

impl std::error::Error for MacroError {}

/// Accumulator for collecting multiple errors before aborting.
///
/// This allows the macro to report all validation errors at once,
/// rather than stopping at the first error.
#[derive(Debug, Default)]
#[allow(dead_code)] // Designed for future macro validation
pub struct ErrorAccumulator {
    errors: Vec<MacroError>,
}

#[allow(dead_code)] // Methods designed for future macro validation
impl ErrorAccumulator {
    /// Creates a new empty error accumulator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an error to the accumulator.
    pub fn push(&mut self, error: MacroError) {
        self.errors.push(error);
    }

    /// Returns true if no errors have been collected.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns the number of collected errors.
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Converts all errors to a single compile error token stream.
    ///
    /// If there are multiple errors, they are combined with newlines.
    pub fn to_compile_error(&self) -> TokenStream {
        if self.errors.is_empty() {
            return TokenStream::new();
        }

        // Combine all error messages
        let combined: Vec<TokenStream> = self.errors.iter().map(|e| e.to_compile_error()).collect();

        quote_spanned! {self.errors[0].span()=>
            #(#combined)*
        }
    }

    /// Returns the first error if any exist, consuming the accumulator.
    pub fn into_result<T>(self, ok_value: T) -> Result<T, MacroError> {
        if let Some(first) = self.errors.into_iter().next() {
            Err(first)
        } else {
            Ok(ok_value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_method_error() {
        let err = MacroError::missing_method("init", "Counter", Span::call_site());
        let error_str = err.to_string();
        assert!(error_str.contains("init"));
        assert!(error_str.contains("Counter"));
        assert!(error_str.contains("missing"));
    }

    #[test]
    fn test_missing_method_has_help() {
        let err = MacroError::missing_method("init", "Counter", Span::call_site());
        assert!(err.help_message().is_some());
        let help = err.help_message().unwrap();
        assert!(help.contains("fn init"));
    }

    #[test]
    fn test_duplicate_method_error() {
        let err = MacroError::duplicate_method("update", Span::call_site());
        let error_str = err.to_string();
        assert!(error_str.contains("duplicate"));
        assert!(error_str.contains("update"));
    }

    #[test]
    fn test_wrong_signature_error() {
        let err = MacroError::wrong_signature(
            "view",
            "fn view(&self) -> String",
            "fn view(&mut self) -> &str",
            Span::call_site(),
        );
        let error_str = err.to_string();
        assert!(error_str.contains("signature"));
        assert!(error_str.contains("view"));
        assert!(error_str.contains("expected"));
        assert!(error_str.contains("found"));
    }

    #[test]
    fn test_unsupported_item_error() {
        let err = MacroError::unsupported_item("a named struct", "enum", Span::call_site());
        let error_str = err.to_string();
        assert!(error_str.contains("named struct"));
        assert!(error_str.contains("enum"));
    }

    #[test]
    fn test_invalid_attribute_error() {
        let err = MacroError::invalid_attribute("state", "unrecognized option", Span::call_site());
        let error_str = err.to_string();
        assert!(error_str.contains("state"));
        assert!(error_str.contains("unrecognized"));
    }

    #[test]
    fn test_invalid_state_field_error() {
        let err = MacroError::invalid_state_field(
            "cache",
            "type does not implement Clone",
            Span::call_site(),
        );
        let error_str = err.to_string();
        assert!(error_str.contains("cache"));
        assert!(error_str.contains("Clone"));
    }

    #[test]
    fn test_missing_bounds_error() {
        let err = MacroError::missing_bounds(
            "T",
            vec!["Clone".to_string(), "Send".to_string()],
            Span::call_site(),
        );
        let error_str = err.to_string();
        assert!(error_str.contains("T"));
        assert!(error_str.contains("Clone"));
        assert!(error_str.contains("Send"));
    }

    #[test]
    fn test_error_accumulator() {
        let mut acc = ErrorAccumulator::new();
        assert!(acc.is_empty());

        acc.push(MacroError::missing_method("init", "App", Span::call_site()));
        acc.push(MacroError::missing_method(
            "update",
            "App",
            Span::call_site(),
        ));

        assert!(!acc.is_empty());
        assert_eq!(acc.len(), 2);
    }

    #[test]
    fn test_error_accumulator_into_result() {
        let acc = ErrorAccumulator::new();
        let result: Result<i32, MacroError> = acc.into_result(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        let mut acc = ErrorAccumulator::new();
        acc.push(MacroError::missing_method("init", "App", Span::call_site()));
        let result: Result<i32, MacroError> = acc.into_result(42);
        assert!(result.is_err());
    }
}
