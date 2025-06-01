#![allow(clippy::needless_doctest_main)]
#![doc = include_str!("../README.md")]
#![no_std]
extern crate alloc;

mod build;
mod derive;
mod r#enum;
mod extractor;
mod iter;
mod param;

use alloc::{borrow::ToOwned, collections::BTreeMap, string::ToString, vec::Vec};

use derive::Derive;
use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use r#enum::Enum;
use syn::{
    parse_macro_input, Attribute, AttributeArgs, DeriveInput, Field, Meta, MetaList, MetaNameValue,
    NestedMeta, Type,
};

const SUBENUM: &str = "subenum";
const ERR: &str =
    "subenum must be called with a list of identifiers, like `#[subenum(EnumA, EnumB(derive(Clone)))]`";

fn snake_case(field: &Field) -> Ident {
    let ident = field.ident.as_ref().unwrap_or_else(|| {
        // No ident; the Type must be Path. Use that.
        match &field.ty {
            Type::Path(path) => path.path.get_ident().unwrap(),
            _ => unimplemented!("a"),
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

fn attribute_paths(attr: &Attribute) -> impl Iterator<Item = Meta> {
    let meta = attr.parse_meta().unwrap();
    let nested = match meta {
        Meta::List(list) => list.nested,
        _ => unimplemented!("b"),
    };
    nested.into_iter().map(|nested| match nested {
        NestedMeta::Meta(meta) => meta,
        _ => unimplemented!("c"),
    })
}

fn build_enum_map(args: AttributeArgs, derives: &[Derive]) -> BTreeMap<Ident, Enum> {
    args.into_iter()
        .map(|nested| match nested {
            NestedMeta::Meta(meta) => meta,
            NestedMeta::Lit(_) => panic!("{}", ERR),
        })
        .map(|meta| match meta {
            Meta::Path(path) => (path.get_ident().expect(ERR).to_owned(), Vec::new()),
            Meta::List(MetaList { path, nested, .. }) => (
                path.get_ident().expect(ERR).to_owned(),
                nested
                    .into_iter()
                    .map(|nested| match nested {
                        NestedMeta::Meta(meta) => meta,
                        NestedMeta::Lit(_) => panic!("{}", ERR),
                    })
                    .map(|meta| match meta {
                        Meta::Path(path) => quote! { #path },
                        Meta::List(MetaList { path, nested, .. }) => quote! { #path(#nested) },
                        Meta::NameValue(MetaNameValue { path, lit, .. }) => quote! { #path = #lit },
                    })
                    .collect::<Vec<proc_macro2::TokenStream>>(),
            ),
            _ => panic!("{}", ERR),
        })
        .map(|(ident, attrs)| {
            (
                ident.clone(),
                Enum::new(ident.clone(), attrs, derives.to_owned()),
            )
        })
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
            for meta in attribute_paths(attr) {
                match meta {
                    Meta::Path(path) => {
                        if path.is_ident("PartialEq") {
                            derives.push(Derive::PartialEq);
                        }
                    }
                    _ => unimplemented!("{:?}", meta),
                }
            }
        }
    }
    let mut enums = build_enum_map(args, &derives);

    for variant in &data.variants {
        for attribute in &variant.attrs {
            // Check for "subenum", iterate through the idents.
            if attribute.path.is_ident(SUBENUM) {
                for meta in attribute_paths(attribute) {
                    let mut var = variant.clone();

                    let (ident, attrs) = match meta {
                        Meta::Path(ref path) => (path.get_ident().unwrap(), Vec::new()),
                        Meta::List(MetaList {
                            ref path, nested, ..
                        }) => (
                            path.get_ident().unwrap(),
                            nested
                                .into_iter()
                                .map(|nested| match nested {
                                    NestedMeta::Meta(meta) => meta,
                                    NestedMeta::Lit(_) => panic!("{}", ERR),
                                })
                                .map(|meta| match meta {
                                    Meta::Path(path) => quote! { #[ #path ] },
                                    Meta::List(MetaList { path, nested, .. }) => {
                                        quote! { #[ #path(#nested) ] }
                                    }
                                    Meta::NameValue(MetaNameValue { path, lit, .. }) => {
                                        quote! { #[ #path = #lit ] }
                                    }
                                })
                                .collect::<Vec<proc_macro2::TokenStream>>(),
                        ),
                        _ => unimplemented!("e"),
                    };

                    // We want all attributes except the "subenum" one.
                    var.attrs.retain(|attr| attribute != attr);

                    let e = enums
                        .get_mut(ident)
                        .expect("All enums to be created must be declared at the top-level subenum attribute");
                    e.variants.push(var);
                    e.variants_attributes.push(attrs);
                }
            }
        }
    }

    for e in enums.values_mut() {
        e.compute_generics(&input.generics);
    }

    let enums: Vec<_> = enums.into_values().map(|e| e.build(&input)).collect();

    sanitize_input(&mut input);

    quote!(
        #input

        #(#enums)*
    )
    .into()
}
