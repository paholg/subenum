use alloc::{format, vec::Vec};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{punctuated::Punctuated, DeriveInput, Generics, Ident, Token, TypeParamBound, Variant};

use crate::{
    derive::{partial_eq::partial_eq_arm, Derive},
    r#enum::Enum,
    snake_case,
};

// Add a bound to generics
fn add_bound(generics: &mut Generics, bound: TypeParamBound) {
    for param in generics.type_params_mut() {
        if param.bounds.iter().all(|b| b != &bound) {
            param.bounds.push(bound.clone());
        }
    }
}

/// Generates the pattern and the corresponding expression with `.into()` calls.
/// Returns (pattern: TokenStream2, expression: TokenStream2)
fn variant_to_pat_and_into_expr(variant: &Variant) -> (TokenStream2, TokenStream2) {
    let ident = &variant.ident;

    match &variant.fields {
        // --- 1. Named Fields (e.g., Variant { a, b }) ---
        syn::Fields::Named(named) => {
            // Pattern: Variant { var1, var2 }
            let vars: Punctuated<Ident, Token![,]> = named
                .named
                .iter()
                .flat_map(|it| it.ident.as_ref())
                .cloned()
                .collect();
            let pattern = quote!(#ident { #vars });

            let vars = vars.iter();
            // Expression: Variant { var1: var1.into(), var2: var2.into() }
            let expression = quote! {
                #ident {
                    #(#vars: #vars.into()),*
                }
            };
            (pattern, expression)
        }

        // --- 2. Unnamed Fields (e.g., Variant(var1, var2)) ---
        syn::Fields::Unnamed(unnamed) => {
            // Create identifiers for the variables (var0, var1, ...)
            let vars: Punctuated<Ident, Token![,]> = unnamed
                .unnamed
                .iter()
                .enumerate()
                .map(|(idx, _)| format_ident!("var{idx}"))
                .collect();

            // Pattern: Variant(var0, var1, ...)
            let pattern = quote!(#ident(#vars));

            let vars = vars.iter();
            // Expression: Variant(var0.into(), var1.into(), ...)
            let expression = quote! {
                #ident(#(#vars.into()),*)
            };
            (pattern, expression)
        }

        // --- 3. Unit Field (e.g., Variant) ---
        syn::Fields::Unit => {
            let pattern = quote!(#ident);
            let expression = quote!(#ident);
            (pattern, expression)
        }
    }
}

fn variant_to_pat_and_try_into_expr(
    variant: &Variant,
    error_type: &Ident,
) -> (TokenStream2, TokenStream2) {
    let ident = &variant.ident;
    let error_ident = error_type;

    match &variant.fields {
        // --- 1. Named Fields ---
        syn::Fields::Named(named) => {
            // Pattern: ParentEnum::Variant { var1, var2 }
            let vars: Punctuated<Ident, Token![,]> = named.named.iter().map(snake_case).collect();
            let pattern = quote!(#ident { #vars });

            // Expression: ParentEnum::Variant { var1: var1.try_into().map_err(|_| E)? }
            let conversion_exprs = vars
                .iter()
                .map(|v| quote!(#v: #v.try_into().map_err(|_| #error_ident)?));

            let expression = quote! {
                #ident {
                    #(#conversion_exprs),*
                }
            };
            (pattern, expression)
        }

        // --- 2. Unnamed Fields ---
        syn::Fields::Unnamed(unnamed) => {
            // Create identifiers for the variables (var0, var1, ...)
            let vars: Punctuated<Ident, Token![,]> = unnamed
                .unnamed
                .iter()
                .enumerate()
                .map(|(idx, _)| format_ident!("var{idx}"))
                .collect();

            // Pattern: ParentEnum::Variant(var0, var1, ...)
            let pattern = quote!(#ident(#vars));

            // Expression: ParentEnum::Variant(var0.try_into().map_err(|_| E)?, ...)
            let conversion_exprs = vars
                .iter()
                .map(|v| quote!(#v.try_into().map_err(|_| #error_ident)?));

            let expression = quote! {
                #ident(#(#conversion_exprs),*)
            };
            (pattern, expression)
        }

        // --- 3. Unit Field ---
        syn::Fields::Unit => {
            let pattern = quote!(#ident);
            let expression = quote!(#ident);
            (pattern, expression)
        }
    }
}

impl Enum {
    fn build_inherited_derive<'a>(
        &self,
        parent: &DeriveInput,
        derive: Derive,
        variants: impl IntoIterator<Item = &'a Variant>,
    ) -> TokenStream2 {
        let child_ident = &self.ident;
        let parent_ident = &parent.ident;

        let (_child_impl, child_ty, _child_where) = self.generics.split_for_impl();

        match derive {
            Derive::PartialEq => {
                let mut generics = parent.generics.clone();
                add_bound(&mut generics, derive.as_bound());
                let (parent_impl, parent_ty, parent_where) = generics.split_for_impl();

                let arms: Punctuated<TokenStream2, Token![,]> = variants
                    .into_iter()
                    .map(|variant| partial_eq_arm(variant, child_ident, parent_ident))
                    .collect();

                quote!(
                    #[automatically_derived]
                    impl #parent_impl PartialEq<#parent_ident #parent_ty> for #child_ident #child_ty #parent_where {
                        fn eq(&self, other: &#parent_ident #parent_ty) -> bool {
                            match (self, other) {
                                #arms,
                                _ => false,
                            }
                        }

                    }

                    #[automatically_derived]
                    impl #parent_impl PartialEq<#child_ident #child_ty> for #parent_ident #parent_ty #parent_where {
                        fn eq(&self, other: &#child_ident #child_ty) -> bool {
                            match (other, self) {
                                #arms,
                                _ => false,
                            }
                        }

                    }
                )
            }
        }
    }

    pub fn build(&self, parent: &DeriveInput) -> TokenStream2 {
        let attributes = self.attributes.clone();
        let child_attrs = parent.attrs.clone();
        let variants = self
            .variants
            .iter()
            .zip(self.variants_attributes.clone())
            .map(|(variant, attribute)| quote! { #(#attribute)* #variant })
            .collect::<Vec<TokenStream2>>();
        let child_generics = self.generics.clone();

        let child_ident = &self.ident;
        let parent_ident = &parent.ident;

        let error = format_ident!("{child_ident}ConvertError");

        #[cfg(not(feature = "error_trait"))]
        let error_trait_impl = quote!();
        #[cfg(all(feature = "error_trait", feature = "std"))]
        let error_trait_impl = quote!(
            impl std::error::Error for #error {}
        );
        #[cfg(all(feature = "error_trait", not(feature = "std")))]
        let error_trait_impl = quote!(
            impl core::error::Error for #error {}
        );

        let into_pats = self.variants.iter().map(variant_to_pat_and_into_expr);
        let try_into_pats = self
            .variants
            .iter()
            .map(|it| variant_to_pat_and_try_into_expr(it, &error));

        let from_child_arms = into_pats.map(|(a, b)| quote!(#child_ident::#a => #parent_ident::#b));

        let try_from_parent_arms =
            try_into_pats.map(|(a, b)| quote!(#parent_ident::#a => Ok(#child_ident::#b)));

        let inherited_derives = self
            .derives
            .iter()
            .map(|&derive| self.build_inherited_derive(parent, derive, &self.variants));

        let vis = &parent.vis;

        let (_child_impl, child_ty, child_where) = child_generics.split_for_impl();

        let (parent_impl, parent_ty, parent_where) = parent.generics.split_for_impl();

        let error_doc = format!(
            "An error type used for converting from [`{parent_ident}`] to [`{child_ident}`]."
        );

        quote!(
            #(#[ #attributes ])*
            #(#child_attrs)*
            #vis enum #child_ident #child_generics #child_where {
                #(#variants),*
            }

            #(#inherited_derives)*

            #[doc = #error_doc]
            #[derive(Copy, Clone, Debug)]
            #vis struct #error;

            impl core::fmt::Display for #error {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    core::fmt::Debug::fmt(self, f)
                }
            }

            #error_trait_impl

            #[automatically_derived]
            impl #parent_impl core::convert::From<#child_ident #child_ty> for #parent_ident #parent_ty #parent_where {
                fn from(child: #child_ident #child_ty) -> Self {
                    match child {
                        #(#from_child_arms),*
                    }
                }
            }

            #[automatically_derived]
            impl #parent_impl core::convert::TryFrom<#parent_ident #parent_ty> for #child_ident #child_ty #parent_where {
                type Error = #error;

                fn try_from(parent: #parent_ident #parent_ty) -> std::result::Result<Self, <Self as core::convert::TryFrom<#parent_ident #parent_ty>>::Error> {
                    match parent {
                        #(#try_from_parent_arms),*,
                        _ => Err(#error)
                    }
                }
            }
        )
    }
}
