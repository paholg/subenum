#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(not(feature = "std"))]
extern crate alloc as std;

mod build;
mod derive;
mod r#enum;
mod extractor;
mod iter;
mod param;

#[cfg(not(feature = "std"))]
use std::{borrow::ToOwned, string::ToString, vec::Vec};
use std::collections::BTreeMap;

use derive::Derive;
use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use r#enum::Enum;
use syn::{
    parse_macro_input, Attribute, AttributeArgs, DeriveInput, Field, Meta, NestedMeta, Path, Type,
};

const SUBENUM: &str = "subenum";

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

fn build_enum_map(args: AttributeArgs, derives: &[Derive]) -> BTreeMap<Ident, Enum> {
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

    for e in enums.values_mut() {
        e.compute_generics(&input.generics);
    }

    let enums: Vec<_> = enums.into_values().map(|e| e.build(&input, data)).collect();

    sanitize_input(&mut input);

    quote!(
        #input

        #(#enums)*
    )
    .into()
}
