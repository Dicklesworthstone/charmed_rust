//! State tracking code generation.
//!
//! This module generates code for tracking state changes in Model structs.
//! Fields marked with `#[state]` are included in change detection, which
//! enables optimized rendering where only state changes trigger view updates.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::attributes::{ModelField, StateFieldArgs};

/// Information about a state-tracked field for code generation.
#[derive(Debug)]
pub struct StateField<'a> {
    /// The field identifier.
    pub ident: &'a Ident,
    /// The field type.
    pub ty: &'a syn::Type,
    /// Parsed state attribute arguments.
    pub args: StateFieldArgs,
}

impl<'a> StateField<'a> {
    /// Create a StateField from a ModelField if it's tracked.
    pub fn from_model_field(field: &'a ModelField) -> Option<Self> {
        let ident = field.ident.as_ref()?;
        let args = field.state_args()?;

        if args.skip {
            return None;
        }

        Some(Self {
            ident,
            ty: &field.ty,
            args,
        })
    }
}

/// Generate the state snapshot struct and related methods.
///
/// The snapshot struct stores clones of all tracked fields, enabling
/// efficient change detection between update cycles.
#[allow(dead_code)] // Non-generic version for simple cases
pub fn generate_state_snapshot(struct_name: &Ident, fields: &[StateField<'_>]) -> TokenStream {
    if fields.is_empty() {
        // No tracked fields, return empty implementation
        return quote! {
            impl #struct_name {
                /// Returns whether any tracked state has changed.
                /// (No fields are tracked, always returns false.)
                #[doc(hidden)]
                #[inline]
                pub fn __state_changed(&self, _prev: &()) -> bool {
                    false
                }

                /// Create a snapshot of tracked state fields.
                /// (No fields are tracked, returns unit.)
                #[doc(hidden)]
                #[inline]
                pub fn __snapshot_state(&self) -> () {
                    ()
                }
            }
        };
    }

    let snapshot_name = format_ident!("__{}StateSnapshot", struct_name);

    // Generate field definitions for snapshot struct
    let field_defs: Vec<_> = fields
        .iter()
        .map(|f| {
            let ident = f.ident;
            let ty = f.ty;
            quote! { #ident: #ty }
        })
        .collect();

    // Generate field clones for snapshot creation
    let field_clones: Vec<_> = fields
        .iter()
        .map(|f| {
            let ident = f.ident;
            quote! { #ident: self.#ident.clone() }
        })
        .collect();

    // Generate field comparisons for change detection
    let comparisons: Vec<_> = fields
        .iter()
        .map(|f| {
            let ident = f.ident;

            // Use custom equality function if provided, otherwise use PartialEq
            // Debug logging is handled via a separate function to avoid attribute issues
            if let Some(eq_fn) = &f.args.eq {
                if f.args.debug {
                    quote! {
                        {
                            let __changed = !#eq_fn(&self.#ident, &__prev.#ident);
                            if __changed {
                                eprintln!(
                                    "[bubbletea::state] {}.{} changed",
                                    stringify!(#struct_name),
                                    stringify!(#ident)
                                );
                            }
                            __changed
                        }
                    }
                } else {
                    quote! {
                        !#eq_fn(&self.#ident, &__prev.#ident)
                    }
                }
            } else if f.args.debug {
                quote! {
                    {
                        let __changed = self.#ident != __prev.#ident;
                        if __changed {
                            eprintln!(
                                "[bubbletea::state] {}.{} changed",
                                stringify!(#struct_name),
                                stringify!(#ident)
                            );
                        }
                        __changed
                    }
                }
            } else {
                quote! {
                    self.#ident != __prev.#ident
                }
            }
        })
        .collect();

    quote! {
        /// Auto-generated state snapshot for change detection.
        #[doc(hidden)]
        #[derive(Clone)]
        pub struct #snapshot_name {
            #(#field_defs),*
        }

        impl #struct_name {
            /// Create a snapshot of tracked state fields.
            #[doc(hidden)]
            #[inline]
            pub fn __snapshot_state(&self) -> #snapshot_name {
                #snapshot_name {
                    #(#field_clones),*
                }
            }

            /// Returns whether any tracked state has changed since the given snapshot.
            #[doc(hidden)]
            #[inline]
            pub fn __state_changed(&self, __prev: &#snapshot_name) -> bool {
                #(#comparisons)||*
            }
        }
    }
}

/// Generate state snapshot code with generic parameters.
///
/// This version handles structs with type parameters.
pub fn generate_state_snapshot_with_generics(
    struct_name: &Ident,
    fields: &[StateField<'_>],
    generics: &syn::Generics,
) -> TokenStream {
    if fields.is_empty() {
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        return quote! {
            impl #impl_generics #struct_name #ty_generics #where_clause {
                /// Returns whether any tracked state has changed.
                /// (No fields are tracked, always returns false.)
                #[doc(hidden)]
                #[inline]
                pub fn __state_changed(&self, _prev: &()) -> bool {
                    false
                }

                /// Create a snapshot of tracked state fields.
                /// (No fields are tracked, returns unit.)
                #[doc(hidden)]
                #[inline]
                pub fn __snapshot_state(&self) -> () {
                    ()
                }
            }
        };
    }

    let snapshot_name = format_ident!("__{}StateSnapshot", struct_name);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate field definitions for snapshot struct
    let field_defs: Vec<_> = fields
        .iter()
        .map(|f| {
            let ident = f.ident;
            let ty = f.ty;
            quote! { #ident: #ty }
        })
        .collect();

    // Generate field clones for snapshot creation
    let field_clones: Vec<_> = fields
        .iter()
        .map(|f| {
            let ident = f.ident;
            quote! { #ident: self.#ident.clone() }
        })
        .collect();

    // Generate field comparisons for change detection
    let comparisons: Vec<_> = fields
        .iter()
        .map(|f| {
            let ident = f.ident;

            if let Some(eq_fn) = &f.args.eq {
                if f.args.debug {
                    quote! {
                        {
                            let __changed = !#eq_fn(&self.#ident, &__prev.#ident);
                            if __changed {
                                eprintln!(
                                    "[bubbletea::state] {}.{} changed",
                                    stringify!(#struct_name),
                                    stringify!(#ident)
                                );
                            }
                            __changed
                        }
                    }
                } else {
                    quote! {
                        !#eq_fn(&self.#ident, &__prev.#ident)
                    }
                }
            } else if f.args.debug {
                quote! {
                    {
                        let __changed = self.#ident != __prev.#ident;
                        if __changed {
                            eprintln!(
                                "[bubbletea::state] {}.{} changed",
                                stringify!(#struct_name),
                                stringify!(#ident)
                            );
                        }
                        __changed
                    }
                }
            } else {
                quote! {
                    self.#ident != __prev.#ident
                }
            }
        })
        .collect();

    quote! {
        /// Auto-generated state snapshot for change detection.
        #[doc(hidden)]
        #[derive(Clone)]
        pub struct #snapshot_name #ty_generics #where_clause {
            #(#field_defs),*
        }

        impl #impl_generics #struct_name #ty_generics #where_clause {
            /// Create a snapshot of tracked state fields.
            #[doc(hidden)]
            #[inline]
            pub fn __snapshot_state(&self) -> #snapshot_name #ty_generics {
                #snapshot_name {
                    #(#field_clones),*
                }
            }

            /// Returns whether any tracked state has changed since the given snapshot.
            #[doc(hidden)]
            #[inline]
            pub fn __state_changed(&self, __prev: &#snapshot_name #ty_generics) -> bool {
                #(#comparisons)||*
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attributes::StateFieldArgs;

    fn make_field<'a>(
        ident: &'a Ident,
        ty: &'a syn::Type,
        args: StateFieldArgs,
    ) -> StateField<'a> {
        StateField { ident, ty, args }
    }

    #[test]
    fn test_empty_state_fields() {
        let struct_name = Ident::new("Counter", proc_macro2::Span::call_site());
        let output = generate_state_snapshot(&struct_name, &[]);
        let output_str = output.to_string();

        assert!(output_str.contains("__state_changed"));
        assert!(output_str.contains("__snapshot_state"));
        assert!(output_str.contains("false")); // Always returns false
    }

    #[test]
    fn test_single_state_field() {
        let struct_name = Ident::new("Counter", proc_macro2::Span::call_site());
        let field_ident = Ident::new("count", proc_macro2::Span::call_site());
        let field_ty: syn::Type = syn::parse_quote!(i32);

        let fields = vec![make_field(&field_ident, &field_ty, StateFieldArgs::default())];
        let output = generate_state_snapshot(&struct_name, &fields);
        let output_str = output.to_string();

        assert!(output_str.contains("__CounterStateSnapshot"));
        assert!(output_str.contains("count : i32"));
        assert!(output_str.contains("self . count . clone ()"));
        assert!(output_str.contains("self . count != __prev . count"));
    }

    #[test]
    fn test_custom_equality() {
        let struct_name = Ident::new("App", proc_macro2::Span::call_site());
        let field_ident = Ident::new("progress", proc_macro2::Span::call_site());
        let field_ty: syn::Type = syn::parse_quote!(f64);

        let mut args = StateFieldArgs::default();
        args.eq = Some(syn::parse_quote!(float_approx_eq));

        let fields = vec![make_field(&field_ident, &field_ty, args)];
        let output = generate_state_snapshot(&struct_name, &fields);
        let output_str = output.to_string();

        assert!(output_str.contains("float_approx_eq"));
        assert!(!output_str.contains("!=")); // Should use custom fn, not !=
    }

    #[test]
    fn test_debug_logging() {
        let struct_name = Ident::new("App", proc_macro2::Span::call_site());
        let field_ident = Ident::new("selected", proc_macro2::Span::call_site());
        let field_ty: syn::Type = syn::parse_quote!(usize);

        let mut args = StateFieldArgs::default();
        args.debug = true;

        let fields = vec![make_field(&field_ident, &field_ty, args)];
        let output = generate_state_snapshot(&struct_name, &fields);
        let output_str = output.to_string();

        assert!(output_str.contains("eprintln !"));
        assert!(output_str.contains("bubbletea::state"));
    }
}
