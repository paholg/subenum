use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{punctuated::Punctuated, Ident, Token, Variant};

use crate::snake_case;

// Map a variant from an enum definition to how it would be used in a match (a, b)
// E.g.
// * Foo -> (Child::Foo, Parent::Foo) => true,
// * Foo(Bar, Baz) -> (Child::Foo(bar, baz), Parent::Foo(bar2, baz2)) => bar == bar2 && baz == baz2,
// * Foo { x: i32, y: i32 } -> (Child::Foo { x, y }, Parent::Foo { x2, y2 }) => x == x2 && y == y2,
pub fn partial_eq_arm(
    variant: &Variant,
    child_ident: &Ident,
    parent_ident: &Ident,
) -> TokenStream2 {
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
