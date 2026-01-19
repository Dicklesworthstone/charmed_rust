//! Attribute parsing using darling.
//!
//! This module provides types and utilities for parsing the custom attributes
//! used by the derive macro (`#[state]`, `#[init]`, `#[update]`, `#[view]`).

use darling::{ast, FromDeriveInput, FromField};
use syn::{Attribute, Ident};

/// Parsed input for the Model derive macro.
///
/// This struct is populated by darling from the annotated struct definition.
#[allow(dead_code)] // Constructed via darling derive
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(model), supports(struct_named))]
pub struct ModelInput {
    /// The struct identifier (name).
    pub ident: Ident,

    /// Generics from the struct definition.
    pub generics: syn::Generics,

    /// The parsed fields of the struct.
    pub data: ast::Data<(), ModelField>,
}

/// A single field in the Model struct.
#[allow(dead_code)] // Constructed via darling derive
#[derive(Debug, FromField)]
#[darling(forward_attrs(state))]
pub struct ModelField {
    /// The field identifier (name).
    pub ident: Option<Ident>,

    /// The field type.
    pub ty: syn::Type,

    /// Forwarded attributes (captures `#[state]` and similar).
    pub attrs: Vec<Attribute>,
}

impl ModelField {
    /// Returns true if this field is marked with `#[state]`.
    pub fn is_state(&self) -> bool {
        self.attrs.iter().any(|attr| attr.path().is_ident("state"))
    }
}

impl ModelInput {
    /// Returns all fields in the struct.
    #[allow(dead_code)] // Used in tests, intended for macro expansion
    pub fn fields(&self) -> Vec<&ModelField> {
        match &self.data {
            ast::Data::Struct(fields) => fields.iter().collect(),
            _ => Vec::new(),
        }
    }

    /// Returns only fields marked with `#[state]`.
    #[allow(dead_code)] // Used in tests, intended for macro expansion
    pub fn state_fields(&self) -> Vec<&ModelField> {
        self.fields()
            .into_iter()
            .filter(|f| f.is_state())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use darling::FromDeriveInput;
    use syn::parse_quote;

    #[test]
    fn test_parse_simple_struct() {
        let input: syn::DeriveInput = parse_quote! {
            struct Counter {
                count: i32,
            }
        };

        let parsed = ModelInput::from_derive_input(&input).unwrap();
        assert_eq!(parsed.ident.to_string(), "Counter");
        assert_eq!(parsed.fields().len(), 1);
    }

    #[test]
    fn test_parse_struct_with_state_fields() {
        let input: syn::DeriveInput = parse_quote! {
            struct MyApp {
                #[state]
                text: String,
                #[state]
                count: i32,
                not_state: bool,
            }
        };

        let parsed = ModelInput::from_derive_input(&input).unwrap();
        assert_eq!(parsed.fields().len(), 3);
        assert_eq!(parsed.state_fields().len(), 2);
    }
}
