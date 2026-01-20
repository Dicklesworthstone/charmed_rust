//! Attribute parsing using darling.
//!
//! This module provides types and utilities for parsing the custom attributes
//! used by the derive macro (`#[state]`, `#[init]`, `#[update]`, `#[view]`).

use darling::{FromDeriveInput, FromField, FromMeta, ast};
use proc_macro2::Span;
use syn::{Attribute, FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, ReturnType, Signature, Type};

// =============================================================================
// Struct-level Input Parsing
// =============================================================================

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

    /// Optional custom message type path (defaults to using bubbletea::Message).
    /// Use `message_type()` to get this as a `syn::Type`.
    #[darling(default)]
    pub message: Option<syn::Path>,
}

/// Configuration for `#[state]` field attribute.
///
/// This attribute marks a field for change tracking to optimize re-renders.
///
/// # Examples
///
/// ```rust,ignore
/// #[derive(Model)]
/// struct App {
///     #[state]
///     counter: i32,                    // Basic tracking
///
///     #[state(eq = "float_approx_eq")] // Custom equality
///     progress: f64,
///
///     #[state(skip)]                   // Excluded from tracking
///     last_tick: Instant,
///
///     #[state(debug)]                  // Log changes
///     selected: usize,
/// }
/// ```
#[allow(dead_code)] // Used by macro expansion
#[derive(Debug, Default, Clone, FromMeta)]
pub struct StateFieldArgs {
    /// Custom equality function for change detection.
    /// If not specified, PartialEq is used.
    #[darling(default)]
    pub eq: Option<syn::Path>,

    /// Skip this field in state change detection.
    #[darling(default)]
    pub skip: bool,

    /// Enable debug logging for changes to this field.
    #[darling(default)]
    pub debug: bool,
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

    /// Returns true if this field should be tracked for state changes.
    /// A field is tracked if it has `#[state]` but not `#[state(skip)]`.
    #[allow(dead_code)] // Used for future macro expansion
    pub fn is_tracked(&self) -> bool {
        if let Some(args) = self.state_args() {
            !args.skip
        } else {
            false
        }
    }

    /// Parse the `#[state(...)]` attribute arguments if present.
    pub fn state_args(&self) -> Option<StateFieldArgs> {
        for attr in &self.attrs {
            if attr.path().is_ident("state") {
                return Some(parse_attribute_args::<StateFieldArgs>(attr).unwrap_or_default());
            }
        }
        None
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
        self.fields().into_iter().filter(|f| f.is_state()).collect()
    }

    /// Returns the custom message type as a `syn::Type`, if specified.
    ///
    /// This converts the stored `syn::Path` to a `syn::Type::Path`.
    #[allow(dead_code)] // Used in macro expansion
    pub fn message_type(&self) -> Option<Type> {
        self.message.as_ref().map(|path| {
            Type::Path(syn::TypePath {
                qself: None,
                path: path.clone(),
            })
        })
    }
}

// =============================================================================
// Method Attribute Argument Parsing
// =============================================================================

/// Configuration for `#[init]` attribute.
///
/// # Example
///
/// ```rust,ignore
/// #[init]
/// fn init(&self) -> Option<Cmd> { ... }
///
/// #[init(command = "startup_cmd")]
/// fn init(&self) -> Option<Cmd> { ... }
/// ```
#[allow(dead_code)] // Used by macro expansion in future tasks
#[derive(Debug, Default, Clone, FromMeta)]
pub struct InitArgs {
    /// Custom command function to call (defaults to returning None).
    #[darling(default)]
    pub command: Option<syn::Path>,
}

/// Configuration for `#[update]` attribute.
///
/// # Example
///
/// ```rust,ignore
/// #[update]
/// fn update(&mut self, msg: Message) -> Option<Cmd> { ... }
/// ```
#[allow(dead_code)] // Used by macro expansion in future tasks
#[derive(Debug, Default, Clone, FromMeta)]
pub struct UpdateArgs {
    // Reserved for future use (e.g., message pattern hints)
}

/// Configuration for `#[view]` attribute.
///
/// # Example
///
/// ```rust,ignore
/// #[view]
/// fn view(&self) -> String { ... }
/// ```
#[allow(dead_code)] // Used by macro expansion in future tasks
#[derive(Debug, Default, Clone, FromMeta)]
pub struct ViewArgs {
    // Reserved for future use (e.g., viewport dimensions)
}

// =============================================================================
// Method Detection and Extraction
// =============================================================================

/// Parsed method information from an impl block.
///
/// Extracts methods marked with `#[init]`, `#[update]`, and `#[view]` attributes.
#[allow(dead_code)] // Used by macro expansion in future tasks
#[derive(Debug, Default)]
pub struct ParsedMethods {
    /// The method marked with `#[init]`, if any.
    pub init: Option<ImplItemFn>,
    /// Arguments from the `#[init]` attribute.
    pub init_args: Option<InitArgs>,

    /// The method marked with `#[update]`, if any.
    pub update: Option<ImplItemFn>,
    /// Arguments from the `#[update]` attribute.
    pub update_args: Option<UpdateArgs>,

    /// The method marked with `#[view]`, if any.
    pub view: Option<ImplItemFn>,
    /// Arguments from the `#[view]` attribute.
    pub view_args: Option<ViewArgs>,
}

/// Error that can occur when parsing method attributes.
#[allow(dead_code)] // Used by macro expansion in future tasks
#[derive(Debug)]
pub struct AttributeError {
    /// The error message.
    pub message: String,
    /// The span where the error occurred.
    pub span: Span,
}

impl AttributeError {
    /// Creates a new attribute error.
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}

impl ParsedMethods {
    /// Extract attributed methods from an impl block.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Multiple methods have the same attribute
    /// - Attribute arguments fail to parse
    #[allow(dead_code)] // Used by macro expansion in future tasks
    pub fn from_impl(impl_block: &ItemImpl) -> Result<Self, AttributeError> {
        let mut result = Self::default();

        for item in &impl_block.items {
            let ImplItem::Fn(method) = item else {
                continue;
            };

            for attr in &method.attrs {
                if attr.path().is_ident("init") {
                    if result.init.is_some() {
                        return Err(AttributeError::new(
                            "Multiple #[init] methods found; only one is allowed",
                            method.sig.ident.span(),
                        ));
                    }
                    let args = parse_attribute_args::<InitArgs>(attr)?;
                    result.init = Some(method.clone());
                    result.init_args = Some(args);
                } else if attr.path().is_ident("update") {
                    if result.update.is_some() {
                        return Err(AttributeError::new(
                            "Multiple #[update] methods found; only one is allowed",
                            method.sig.ident.span(),
                        ));
                    }
                    let args = parse_attribute_args::<UpdateArgs>(attr)?;
                    result.update = Some(method.clone());
                    result.update_args = Some(args);
                } else if attr.path().is_ident("view") {
                    if result.view.is_some() {
                        return Err(AttributeError::new(
                            "Multiple #[view] methods found; only one is allowed",
                            method.sig.ident.span(),
                        ));
                    }
                    let args = parse_attribute_args::<ViewArgs>(attr)?;
                    result.view = Some(method.clone());
                    result.view_args = Some(args);
                }
            }
        }

        Ok(result)
    }

    /// Returns true if all required methods are present.
    #[allow(dead_code)]
    pub fn is_complete(&self) -> bool {
        self.init.is_some() && self.update.is_some() && self.view.is_some()
    }
}

/// Parse attribute arguments into the specified type.
#[allow(dead_code)] // Used by from_impl and in tests
fn parse_attribute_args<T: FromMeta + Default>(attr: &Attribute) -> Result<T, AttributeError> {
    match &attr.meta {
        syn::Meta::Path(_) => {
            // No arguments, use default
            Ok(T::default())
        }
        syn::Meta::List(list) => {
            // Parse nested meta
            let nested = darling::ast::NestedMeta::parse_meta_list(list.tokens.clone())
                .map_err(|e| AttributeError::new(e.to_string(), list.delimiter.span().open()))?;
            T::from_list(&nested)
                .map_err(|e| AttributeError::new(e.to_string(), list.delimiter.span().open()))
        }
        syn::Meta::NameValue(nv) => Err(AttributeError::new(
            "Expected #[attr] or #[attr(...)] syntax",
            nv.eq_token.span,
        )),
    }
}

// =============================================================================
// Method Signature Validation
// =============================================================================

/// Validates the signature of an `#[init]` method.
///
/// Requirements:
/// - Must take `&self` as the only parameter
/// - Must return `Option<Cmd>` or a compatible type
///
/// # Errors
///
/// Returns an error if the signature is invalid.
pub fn validate_init_signature(sig: &Signature) -> Result<(), AttributeError> {
    // Check for &self parameter
    let mut has_self = false;
    let mut extra_params = 0;

    for input in &sig.inputs {
        match input {
            FnArg::Receiver(recv) => {
                if recv.mutability.is_some() {
                    return Err(AttributeError::new(
                        "#[init] method should take &self, not &mut self",
                        recv.self_token.span,
                    ));
                }
                has_self = true;
            }
            FnArg::Typed(_) => {
                extra_params += 1;
            }
        }
    }

    if !has_self {
        return Err(AttributeError::new(
            "#[init] method must take &self as the first argument",
            sig.fn_token.span,
        ));
    }

    if extra_params > 0 {
        return Err(AttributeError::new(
            "#[init] method should only take &self, no additional parameters",
            sig.paren_token.span.open(),
        ));
    }

    // Check return type exists
    if matches!(sig.output, ReturnType::Default) {
        return Err(AttributeError::new(
            "#[init] method must return Option<Cmd>",
            sig.fn_token.span,
        ));
    }

    Ok(())
}

/// Validates the signature of an `#[update]` method.
///
/// Requirements:
/// - Must take `&mut self` as the first parameter
/// - Must take a message parameter as the second parameter
/// - Must return `Option<Cmd>` or a compatible type
///
/// # Errors
///
/// Returns an error if the signature is invalid.
pub fn validate_update_signature(sig: &Signature) -> Result<(), AttributeError> {
    let mut has_self_mut = false;
    let mut has_msg = false;

    for input in &sig.inputs {
        match input {
            FnArg::Receiver(recv) => {
                if recv.mutability.is_none() {
                    return Err(AttributeError::new(
                        "#[update] method must take &mut self, not &self",
                        recv.self_token.span,
                    ));
                }
                has_self_mut = true;
            }
            FnArg::Typed(_) => {
                has_msg = true;
            }
        }
    }

    if !has_self_mut {
        return Err(AttributeError::new(
            "#[update] method must take &mut self as the first argument",
            sig.fn_token.span,
        ));
    }

    if !has_msg {
        return Err(AttributeError::new(
            "#[update] method must take a message argument",
            sig.fn_token.span,
        ));
    }

    // Check return type exists
    if matches!(sig.output, ReturnType::Default) {
        return Err(AttributeError::new(
            "#[update] method must return Option<Cmd>",
            sig.fn_token.span,
        ));
    }

    Ok(())
}

/// Validates the signature of a `#[view]` method.
///
/// Requirements:
/// - Must take `&self` as the only parameter
/// - Must return `String` or a compatible type
///
/// # Errors
///
/// Returns an error if the signature is invalid.
pub fn validate_view_signature(sig: &Signature) -> Result<(), AttributeError> {
    let mut has_self = false;
    let mut extra_params = 0;

    for input in &sig.inputs {
        match input {
            FnArg::Receiver(recv) => {
                if recv.mutability.is_some() {
                    return Err(AttributeError::new(
                        "#[view] method should take &self, not &mut self (view should be pure)",
                        recv.self_token.span,
                    ));
                }
                has_self = true;
            }
            FnArg::Typed(_) => {
                extra_params += 1;
            }
        }
    }

    if !has_self {
        return Err(AttributeError::new(
            "#[view] method must take &self as the first argument",
            sig.fn_token.span,
        ));
    }

    if extra_params > 0 {
        return Err(AttributeError::new(
            "#[view] method should only take &self, no additional parameters",
            sig.paren_token.span.open(),
        ));
    }

    // Check return type exists
    if matches!(sig.output, ReturnType::Default) {
        return Err(AttributeError::new(
            "#[view] method must return String",
            sig.fn_token.span,
        ));
    }

    Ok(())
}

/// Validates all methods in a `ParsedMethods` struct.
///
/// # Errors
///
/// Returns an error if any method signature is invalid.
#[allow(dead_code)]
pub fn validate_all_signatures(methods: &ParsedMethods) -> Result<(), AttributeError> {
    if let Some(ref init) = methods.init {
        validate_init_signature(&init.sig)?;
    }
    if let Some(ref update) = methods.update {
        validate_update_signature(&update.sig)?;
    }
    if let Some(ref view) = methods.view {
        validate_view_signature(&view.sig)?;
    }
    Ok(())
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

    #[test]
    fn test_parse_struct_with_custom_message() {
        let input: syn::DeriveInput = parse_quote! {
            #[model(message = MyMsg)]
            struct MyApp {
                count: i32,
            }
        };

        let parsed = ModelInput::from_derive_input(&input).unwrap();
        assert!(parsed.message.is_some());
    }

    // =========================================================================
    // Signature Validation Tests
    // =========================================================================

    #[test]
    fn test_validate_init_valid() {
        let sig: Signature = parse_quote! {
            fn init(&self) -> Option<Cmd>
        };
        assert!(validate_init_signature(&sig).is_ok());
    }

    #[test]
    fn test_validate_init_missing_self() {
        let sig: Signature = parse_quote! {
            fn init() -> Option<Cmd>
        };
        let result = validate_init_signature(&sig);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("&self"));
    }

    #[test]
    fn test_validate_init_mut_self_rejected() {
        let sig: Signature = parse_quote! {
            fn init(&mut self) -> Option<Cmd>
        };
        let result = validate_init_signature(&sig);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("&self, not &mut self"));
    }

    #[test]
    fn test_validate_init_extra_params_rejected() {
        let sig: Signature = parse_quote! {
            fn init(&self, extra: i32) -> Option<Cmd>
        };
        let result = validate_init_signature(&sig);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("no additional parameters")
        );
    }

    #[test]
    fn test_validate_update_valid() {
        let sig: Signature = parse_quote! {
            fn update(&mut self, msg: Message) -> Option<Cmd>
        };
        assert!(validate_update_signature(&sig).is_ok());
    }

    #[test]
    fn test_validate_update_immutable_self_rejected() {
        let sig: Signature = parse_quote! {
            fn update(&self, msg: Message) -> Option<Cmd>
        };
        let result = validate_update_signature(&sig);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("&mut self"));
    }

    #[test]
    fn test_validate_update_missing_msg_rejected() {
        let sig: Signature = parse_quote! {
            fn update(&mut self) -> Option<Cmd>
        };
        let result = validate_update_signature(&sig);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("message argument"));
    }

    #[test]
    fn test_validate_view_valid() {
        let sig: Signature = parse_quote! {
            fn view(&self) -> String
        };
        assert!(validate_view_signature(&sig).is_ok());
    }

    #[test]
    fn test_validate_view_mut_self_rejected() {
        let sig: Signature = parse_quote! {
            fn view(&mut self) -> String
        };
        let result = validate_view_signature(&sig);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("pure"));
    }

    #[test]
    fn test_validate_view_missing_return_rejected() {
        let sig: Signature = parse_quote! {
            fn view(&self)
        };
        let result = validate_view_signature(&sig);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("String"));
    }

    // =========================================================================
    // ParsedMethods Tests
    // =========================================================================

    #[test]
    fn test_parsed_methods_from_impl() {
        let impl_block: ItemImpl = parse_quote! {
            impl Counter {
                #[init]
                fn init(&self) -> Option<Cmd> { None }

                #[update]
                fn update(&mut self, msg: Message) -> Option<Cmd> { None }

                #[view]
                fn view(&self) -> String { String::new() }
            }
        };

        let methods = ParsedMethods::from_impl(&impl_block).unwrap();
        assert!(methods.init.is_some());
        assert!(methods.update.is_some());
        assert!(methods.view.is_some());
        assert!(methods.is_complete());
    }

    #[test]
    fn test_parsed_methods_partial() {
        let impl_block: ItemImpl = parse_quote! {
            impl Counter {
                #[init]
                fn init(&self) -> Option<Cmd> { None }

                fn other_method(&self) {}
            }
        };

        let methods = ParsedMethods::from_impl(&impl_block).unwrap();
        assert!(methods.init.is_some());
        assert!(methods.update.is_none());
        assert!(methods.view.is_none());
        assert!(!methods.is_complete());
    }

    #[test]
    fn test_parsed_methods_duplicate_rejected() {
        let impl_block: ItemImpl = parse_quote! {
            impl Counter {
                #[init]
                fn init1(&self) -> Option<Cmd> { None }

                #[init]
                fn init2(&self) -> Option<Cmd> { None }
            }
        };

        let result = ParsedMethods::from_impl(&impl_block);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Multiple #[init]"));
    }

    // =========================================================================
    // Attribute Args Tests
    // =========================================================================

    #[test]
    fn test_init_args_default() {
        let attr: Attribute = parse_quote!(#[init]);
        let args = parse_attribute_args::<InitArgs>(&attr).unwrap();
        assert!(args.command.is_none());
    }

    #[test]
    fn test_init_args_with_command() {
        let attr: Attribute = parse_quote!(#[init(command = my_startup)]);
        let args = parse_attribute_args::<InitArgs>(&attr).unwrap();
        assert!(args.command.is_some());
    }
}
