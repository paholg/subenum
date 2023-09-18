#![doc = include_str!("../README.md")]

mod build;
mod derive;
mod r#enum;
mod extractor;
mod iter;
mod param;

use std::collections::{HashMap, HashSet};

use derive::Derive;
use heck::ToSnakeCase;
use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use r#enum::Enum;
use syn::{
    parse_macro_input, Attribute, AttributeArgs, DeriveInput, Field, Meta, MetaList, MetaNameValue,
    NestedMeta, Type, Variant, Path, punctuated::Punctuated,
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

    for e in enums.values_mut() {
        e.compute_generics(&input.generics);
    }

    let mut sibling_conversions = Vec::new();
    for (sibling1, sibling2) in enums.values().cloned().tuple_combinations() {
        let sibling1_variants_hash_set: HashSet<Variant> = sibling1.variants.into_iter().collect();
        let sibling2_variants_hash_set: HashSet<Variant> = sibling2.variants.into_iter().collect();

        let intersection = sibling1_variants_hash_set.intersection(&sibling2_variants_hash_set).collect::<Vec<&Variant>>();
        if intersection.is_empty() {
            continue;
        }

        let sibling1_ident = sibling1.ident;
        let (_, sibling1_ty, _) = sibling1.generics.split_for_impl();

        let sibling2_ident = sibling2.ident;
        let (_, sibling2_ty, _) = sibling2.generics.split_for_impl();

        let mut combined_generics = sibling1.generics.params.clone().into_iter().collect::<HashSet<syn::GenericParam>>();
        combined_generics.extend(sibling2.generics.params.clone().into_iter().collect::<HashSet<syn::GenericParam>>());

        let combined_generics = syn::Generics {
            lt_token: Some(syn::token::Lt::default()),
            params: Punctuated::from_iter(combined_generics.into_iter()),
            gt_token: Some(syn::token::Gt::default()),
            where_clause: None,
        };

        let mut combined_where = sibling1.generics.where_clause.clone()
        .map(|where_clause| where_clause.predicates.into_iter().collect::<HashSet<syn::WherePredicate>>()).unwrap_or_default();
        combined_where.extend(sibling2.generics.where_clause.clone()
        .map(|where_clause| where_clause.predicates.into_iter().collect::<HashSet<syn::WherePredicate>>()).unwrap_or_default());

        let combined_where = Some(syn::WhereClause {
            where_token: syn::token::Where::default(),
            predicates: Punctuated::from_iter(combined_where.into_iter())
        });

        let pats: Vec<proc_macro2::TokenStream> = intersection.iter().map(|variant| build::variant_to_unary_pat(*variant)).collect();

        let sibling1_to_sibling2 = if sibling1_variants_hash_set.len() == intersection.len() {
            let from_sibling1_arms = pats
                .iter()
                .map(|pat| quote!(#sibling1_ident::#pat => #sibling2_ident::#pat));

            quote! {
                #[automatically_derived]
                impl #combined_generics std::convert::From<#sibling1_ident #sibling1_ty> for #sibling2_ident #sibling2_ty #combined_where {
                    fn from(sibling: #sibling1_ident #sibling1_ty) -> Self {
                        match sibling {
                            #(#from_sibling1_arms),*
                        }
                    }
                }
            }
        } else {
            let try_from_sibling1_arms = pats
                .iter()
                .map(|pat| quote!(#sibling1_ident::#pat => Ok(#sibling2_ident::#pat)));

            let error = quote::format_ident!("{sibling2_ident}ConvertError");

            quote! {
                #[automatically_derived]
                impl #combined_generics std::convert::TryFrom<#sibling1_ident #sibling1_ty> for #sibling2_ident #sibling2_ty #combined_where {
                    type Error = #error;

                    fn try_from(sibling: #sibling1_ident #sibling1_ty) -> Result<Self, Self::Error> {
                        match sibling {
                            #(#try_from_sibling1_arms),*,
                            _ => Err(#error)
                        }
                    }
                }
            }
        };

        let sibling2_to_sibling1 = if sibling2_variants_hash_set.len() == intersection.len() {
            let from_sibling2_arms = pats
                .iter()
                .map(|pat| quote!(#sibling2_ident::#pat => #sibling1_ident::#pat));

            quote! {
                #[automatically_derived]
                impl #combined_generics std::convert::From<#sibling2_ident #sibling2_ty> for #sibling1_ident #sibling1_ty #combined_where {
                    fn from(sibling: #sibling2_ident #sibling2_ty) -> Self {
                        match sibling {
                            #(#from_sibling2_arms),*
                        }
                    }
                }
            }
        } else {
            let try_from_sibling2_arms = pats
                .iter()
                .map(|pat| quote!(#sibling2_ident::#pat => Ok(#sibling1_ident::#pat)));

            let error = quote::format_ident!("{sibling1_ident}ConvertError");

            quote! {
                #[automatically_derived]
                impl #combined_generics std::convert::TryFrom<#sibling2_ident #sibling2_ty> for #sibling1_ident #sibling1_ty #combined_where {
                    type Error = #error;

                    fn try_from(sibling: #sibling2_ident #sibling2_ty) -> Result<Self, Self::Error> {
                        match sibling {
                            #(#try_from_sibling2_arms),*,
                            _ => Err(#error)
                        }
                    }
                }
            }
        };

        sibling_conversions.push(
            quote!{
                #sibling1_to_sibling2

                #sibling2_to_sibling1
            }
        );
    }


    let enums: Vec<_> = enums.into_values().map(|e| e.build(&input, data)).collect();

    sanitize_input(&mut input);

    quote!(
        #input

        #(#enums)*

        #(#sibling_conversions)*
    )
    .into()
}
