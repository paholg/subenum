#![doc = include_str!("../README.md")]
use std::collections::HashMap;

use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, punctuated::Punctuated, Attribute, AttributeArgs, Data, DataEnum,
    DeriveInput, Field, Meta, NestedMeta, Path, Token, Type, Variant,
};

const SUBENUM: &str = "subenum";

#[derive(Clone, Copy, Debug)]
enum Derive {
    PartialEq,
}

struct Enum {
    ident: Ident,
    variants: Punctuated<Variant, Token![,]>,
    derives: Vec<Derive>,
}

impl Enum {
    fn new(ident: Ident, derives: Vec<Derive>) -> Self {
        Enum {
            ident,
            variants: Punctuated::new(),
            derives,
        }
    }
}

fn snake_case(field: &Field) -> Ident {
    let ident = field.ident.as_ref().unwrap_or_else(|| {
        // No ident; the Type must be Path. Use that.
        match &field.ty {
            Type::Path(path) => path.path.get_ident().unwrap(),
            _ => unimplemented!(),
        }
    });
    Ident::new(&ident.to_string().to_snake_case(), ident.span())
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

// Map a variant from an enum definition to how it would be used in a match (a, b)
// E.g.
// * Foo -> (Child::Foo, Parent::Foo) => true,
// * Foo(Bar, Baz) -> (Child::Foo(bar, baz), Parent::Foo(bar2, baz2)) => bar == bar2 && baz == baz2,
// * Foo { x: i32, y: i32 } -> (Child::Foo { x, y }, Parent::Foo { x2, y2 }) => x == x2 && y == y2,
fn partial_eq_arm(variant: &Variant, child_ident: &Ident, parent_ident: &Ident) -> TokenStream2 {
    let ident = &variant.ident;

    match &variant.fields {
        syn::Fields::Named(named) => {
            let vars1: Punctuated<Ident, Token![,]> = named.named.iter().map(snake_case).collect();
            let vars2: Punctuated<Ident, Token![,]> =
                vars1.iter().map(|v| format_ident!("{}_b", v)).collect();
            let vars_rhs: Punctuated<TokenStream2, Token![&&]> = vars1
                .iter()
                .zip(vars2.iter())
                .map(|(var1, var2)| quote!(#var1 == #var2))
                .collect();
            let vars2: Punctuated<TokenStream2, Token![,]> = vars1
                .iter()
                .map(|v| {
                    let v2 = format_ident!("{}_b", v);
                    quote!(#v: #v2)
                })
                .collect();
            quote!((#child_ident::#ident{#vars1}, #parent_ident::#ident{#vars2}) => #vars_rhs)
        }
        syn::Fields::Unnamed(unnamed) => {
            let vars1: Punctuated<Ident, Token![,]> = unnamed
                .unnamed
                .iter()
                .enumerate()
                .map(|(idx, _)| format_ident!("var{idx}"))
                .collect();
            let vars2: Punctuated<Ident, Token![,]> =
                vars1.iter().map(|v| format_ident!("{}_b", v)).collect();
            let vars_rhs: Punctuated<TokenStream2, Token![&&]> = vars1
                .iter()
                .zip(vars2.iter())
                .map(|(var1, var2)| quote!(#var1 == #var2))
                .collect();
            quote!((#child_ident::#ident(#vars1), #parent_ident::#ident(#vars2)) => #vars_rhs)
        }
        syn::Fields::Unit => quote!((#child_ident::#ident, #parent_ident::#ident) => true),
    }
}

impl Enum {
    fn build_inherited_derive<'a>(
        &self,
        parent_ident: &Ident,
        derive: Derive,
        variants: impl IntoIterator<Item = &'a Variant>,
    ) -> TokenStream2 {
        let child_ident = &self.ident;

        match derive {
            Derive::PartialEq => {
                let arms: Punctuated<TokenStream2, Token![,]> = variants
                    .into_iter()
                    .map(|variant| partial_eq_arm(variant, child_ident, parent_ident))
                    .collect();

                quote!(
                    #[automatically_derived]
                    impl PartialEq<#parent_ident> for #child_ident {
                        fn eq(&self, other: &#parent_ident) -> bool {
                            match (self, other) {
                                #arms,
                                _ => false,
                            }
                        }

                    }

                    #[automatically_derived]
                    impl PartialEq<#child_ident> for #parent_ident {
                        fn eq(&self, other: &#child_ident) -> bool {
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

    fn build(&self, parent: &DeriveInput, parent_data: &DataEnum) -> TokenStream2 {
        let mut child_data = parent_data.clone();
        child_data.variants = self.variants.clone();

        let mut child = parent.clone();
        child.ident = self.ident.clone();
        child.data = Data::Enum(child_data);

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
            .map(|&derive| self.build_inherited_derive(parent_ident, derive, &self.variants));

        let vis = &parent.vis;

        quote!(
            #child

            #(#inherited_derives)*

            #[derive(Copy, Clone, Debug)]
            #vis struct #error;

            impl std::fmt::Display for #error {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    std::fmt::Debug::fmt(self, f)
                }
            }

            impl std::error::Error for #error {}

            #[automatically_derived]
            impl std::convert::From<#child_ident> for #parent_ident {
                fn from(child: #child_ident) -> Self {
                    match child {
                        #(#from_child_arms),*
                    }
                }
            }

            #[automatically_derived]
            impl std::convert::TryFrom<#parent_ident> for #child_ident {
                type Error = #error;

                fn try_from(parent: #parent_ident) -> Result<Self, Self::Error> {
                    match parent {
                        #(#try_from_parent_arms),*,
                        _ => Err(#error)
                    }
                }
            }
        )
    }
}

/// Remove our #[subenum(...)] attributes from the input.
fn sanitize_input(input: &mut DeriveInput) {
    let data = match input.data {
        syn::Data::Enum(ref mut data) => data,
        _ => panic!("SubEnum may only be used on enums."),
    };

    for variant in data.variants.iter_mut() {
        // TODO: Switch to Vec::drain_filter once stabilized.
        let mut i = 0;
        while i < variant.attrs.len() {
            if variant.attrs[i].path.is_ident(SUBENUM) {
                variant.attrs.remove(i);
            } else {
                i += 1;
            }
        }
    }
}

fn attribute_paths(attr: &Attribute) -> impl Iterator<Item = Path> {
    let meta = attr.parse_meta().unwrap();
    let nested = match meta {
        Meta::List(list) => list.nested,
        _ => unimplemented!(),
    };
    nested.into_iter().map(|nested| match nested {
        NestedMeta::Meta(Meta::Path(path)) => path,
        _ => unimplemented!(),
    })
}

fn build_enum_map(args: AttributeArgs, derives: &[Derive]) -> HashMap<Ident, Enum> {
    let err = "subenum must be called with a list of identifiers, like `#[subenum(EnumA, EnumB)]`";
    args.into_iter()
        .map(|nested| match nested {
            NestedMeta::Meta(meta) => meta,
            NestedMeta::Lit(_) => panic!("{err}"),
        })
        .map(|meta| match meta {
            Meta::Path(path) => path,
            _ => panic!("{err}"),
        })
        .map(|path| path.get_ident().expect(err).to_owned())
        .map(|ident| (ident.clone(), Enum::new(ident, derives.to_owned())))
        .collect()
}

#[proc_macro_attribute]
pub fn subenum(args: TokenStream, tokens: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let mut input = parse_macro_input!(tokens as DeriveInput);
    let data = match input.data {
        syn::Data::Enum(ref data) => data,
        _ => panic!("subenum may only be used on enums."),
    };

    let mut derives = Vec::new();
    for attr in &input.attrs {
        if attr.path.is_ident("derive") {
            for path in attribute_paths(attr) {
                if path.is_ident("PartialEq") {
                    derives.push(Derive::PartialEq);
                }
            }
        }
    }
    let mut enums = build_enum_map(args, &derives);

    for variant in &data.variants {
        for attribute in &variant.attrs {
            // Check for "subenum", iterate through the idents.
            if attribute.path.is_ident(SUBENUM) {
                for path in attribute_paths(attribute) {
                    let ident = path.get_ident().unwrap();
                    let mut var = variant.clone();

                    // We want all attributes except the "subenum" one.
                    var.attrs = var
                        .attrs
                        .iter()
                        .cloned()
                        .filter(|attr| attribute != attr)
                        .collect();

                    let e = enums
                        .get_mut(ident)
                        .expect("All enums to be created must be declared at the top-level subenum attribute");
                    e.variants.push(var);
                }
            }
        }
    }

    let enums: Vec<_> = enums.into_values().map(|e| e.build(&input, data)).collect();

    sanitize_input(&mut input);

    quote!(
        #input

        #(#enums)*
    )
    .into()
}
