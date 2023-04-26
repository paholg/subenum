use std::vec::Vec;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    punctuated::Punctuated, Data, DataEnum, DeriveInput, Generics, Ident, Token, TypeParamBound,
    Variant,
};

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

// Map a variant from an enum definition to how it would be used in a match
// E.g.
// * Foo -> Foo
// * Foo(Bar, Baz) -> Foo(var1, var2)
// * Foo { x: i32, y: i32 } -> Foo { x, y }
fn variant_to_unary_pat(variant: &Variant) -> TokenStream2 {
    let ident = &variant.ident;

    match &variant.fields {
        syn::Fields::Named(named) => {
            let vars: Punctuated<Ident, Token![,]> = named.named.iter().map(snake_case).collect();
            quote!(#ident{#vars})
        }
        syn::Fields::Unnamed(unnamed) => {
            let vars: Punctuated<Ident, Token![,]> = unnamed
                .unnamed
                .iter()
                .enumerate()
                .map(|(idx, _)| format_ident!("var{idx}"))
                .collect();
            quote!(#ident(#vars))
        }
        syn::Fields::Unit => quote!(#ident),
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

    pub fn build(&self, parent: &DeriveInput, parent_data: &DataEnum) -> TokenStream2 {
        let mut child_data = parent_data.clone();
        child_data.variants = self.variants.clone();

        let mut child = parent.clone();
        child.ident = self.ident.clone();
        child.data = Data::Enum(child_data);
        child.generics = self.generics.clone();

        let child_ident = &self.ident;
        let parent_ident = &parent.ident;

        let error = format_ident!("{child_ident}ConvertError");

        let pats: Vec<TokenStream2> = self.variants.iter().map(variant_to_unary_pat).collect();

        let from_child_arms = pats
            .iter()
            .map(|pat| quote!(#child_ident::#pat => #parent_ident::#pat));

        let try_from_parent_arms = pats
            .iter()
            .map(|pat| quote!(#parent_ident::#pat => Ok(#child_ident::#pat)));

        let inherited_derives = self
            .derives
            .iter()
            .map(|&derive| self.build_inherited_derive(parent, derive, &self.variants));

        let vis = &parent.vis;

        let (_child_impl, child_ty, _child_where) = child.generics.split_for_impl();

        let (parent_impl, parent_ty, parent_where) = parent.generics.split_for_impl();

        quote!(
            #child

            #(#inherited_derives)*

            #[derive(Copy, Clone, Debug)]
            #vis struct #error;

            impl core::fmt::Display for #error {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    core::fmt::Debug::fmt(self, f)
                }
            }

            impl core::error::Error for #error {}

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

                fn try_from(parent: #parent_ident #parent_ty) -> Result<Self, Self::Error> {
                    match parent {
                        #(#try_from_parent_arms),*,
                        _ => Err(#error)
                    }
                }
            }
        )
    }
}
