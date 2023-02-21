#![doc = include_str!("../README.md")]
use std::collections::{HashMap, HashSet};

use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, punctuated::Punctuated, Attribute, AttributeArgs, Data, DataEnum,
    DeriveInput, Field, GenericParam, Generics, Lifetime, Meta, NestedMeta, Path, Token,
    TraitBound, TraitBoundModifier, Type, TypeParamBound, TypePath, Variant, WherePredicate,
};

const SUBENUM: &str = "subenum";

#[derive(Clone, Copy, Debug)]
enum Derive {
    PartialEq,
}

impl Derive {
    fn as_bound(&self) -> TypeParamBound {
        match self {
            Derive::PartialEq => TypeParamBound::Trait(TraitBound {
                paren_token: None,
                modifier: TraitBoundModifier::None,
                lifetimes: None,
                path: Path::from(Ident::new("PartialEq", Span::call_site())),
            }),
        }
    }
}

struct Enum {
    ident: Ident,
    variants: Punctuated<Variant, Token![,]>,
    derives: Vec<Derive>,
    generics: Generics,
}

/// A type or lifetime param, potentially used as a generic.
/// E.g. the 'a in `'a: 'b + 'c` or the T in `T: U + V`.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
enum Param {
    Lifetime(Lifetime),
    Type(Type),
}

impl From<Ident> for Param {
    fn from(value: Ident) -> Self {
        Param::Type(Type::Path(TypePath {
            qself: None,
            path: value.into(),
        }))
    }
}

impl From<Type> for Param {
    fn from(value: Type) -> Self {
        Param::Type(value)
    }
}

impl From<Lifetime> for Param {
    fn from(value: Lifetime) -> Self {
        Param::Lifetime(value)
    }
}

fn ty_as_ident(ty: &Type) -> Option<&Ident> {
    if let Type::Path(p) = ty {
        if p.qself.is_some() {
            None
        } else {
            p.path.get_ident()
        }
    } else {
        None
    }
}

impl Param {
    fn find(&self, generics: &Generics) -> (Option<GenericParam>, Option<WherePredicate>) {
        match self {
            Param::Lifetime(lt) => find_lt(lt, generics),
            Param::Type(ty) => find_ty(ty, generics),
        }
    }
}
fn find_lt(lt: &Lifetime, generics: &Generics) -> (Option<GenericParam>, Option<WherePredicate>) {
    let generic_param = generics
        .params
        .iter()
        .find(|p| match p {
            GenericParam::Lifetime(lt_def) => &lt_def.lifetime == lt,
            _ => false,
        })
        .map(ToOwned::to_owned);

    let predicate = generics
        .where_clause
        .iter()
        .flat_map(|wh| wh.predicates.to_owned())
        .find(|pred| match pred {
            WherePredicate::Lifetime(pred_lt) => &pred_lt.lifetime == lt,
            _ => false,
        });

    (generic_param, predicate)
}
fn find_ty(ty: &Type, generics: &Generics) -> (Option<GenericParam>, Option<WherePredicate>) {
    let generic_param = generics
        .params
        .iter()
        .find(|p| match p {
            GenericParam::Type(ty_param) => {
                matches!(ty_as_ident(ty), Some(t) if t == &ty_param.ident)
            }
            _ => false,
        })
        .map(ToOwned::to_owned);

    let predicate = generics
        .where_clause
        .iter()
        .flat_map(|wh| wh.predicates.to_owned())
        .find(|pred| match pred {
            WherePredicate::Type(pred_ty) => &pred_ty.bounded_ty == ty,
            _ => false,
        });

    (generic_param, predicate)
}

impl Enum {
    fn new(ident: Ident, derives: Vec<Derive>) -> Self {
        Enum {
            ident,
            variants: Punctuated::new(),
            derives,
            generics: Generics {
                lt_token: Some(syn::token::Lt::default()),
                params: Punctuated::new(),
                gt_token: Some(syn::token::Gt::default()),
                where_clause: None,
            },
        }
    }

    fn compute_generics(&mut self, parent_generics: &Generics) {
        let generic_bounds: HashMap<Param, Vec<TypeParamBound>> = parent_generics
            .type_params()
            .map(|param| {
                (
                    param.ident.to_owned().into(),
                    param.bounds.iter().cloned().collect(),
                )
            })
            .chain(parent_generics.lifetimes().map(|lifetime_def| {
                (
                    lifetime_def.lifetime.to_owned().into(),
                    lifetime_def
                        .bounds
                        .iter()
                        .cloned()
                        .map(TypeParamBound::Lifetime)
                        .collect(),
                )
            }))
            .chain(
                parent_generics
                    .where_clause
                    .iter()
                    .flat_map(|clause| &clause.predicates)
                    .map(|pred| match pred {
                        syn::WherePredicate::Type(ty) => (
                            ty.bounded_ty.to_owned().into(),
                            ty.bounds.iter().cloned().collect(),
                        ),

                        syn::WherePredicate::Lifetime(lt) => (
                            lt.lifetime.to_owned().into(),
                            lt.bounds
                                .iter()
                                .cloned()
                                .map(TypeParamBound::Lifetime)
                                .collect(),
                        ),
                        syn::WherePredicate::Eq(_) => {
                            panic!("Equality predicates in where clauses are unsupported")
                        }
                    }),
            )
            // TODO: Incorporate where clause
            .collect();

        // panic!("{generic_bounds:#?}");

        let types: Vec<Type> = self
            .variants
            .iter()
            .flat_map(|variant| match &variant.fields {
                syn::Fields::Named(named) => named.named.iter().map(|field| &field.ty).collect(),
                syn::Fields::Unnamed(unnamed) => {
                    unnamed.unnamed.iter().map(|field| &field.ty).collect()
                }
                syn::Fields::Unit => Vec::new(),
            })
            .cloned()
            .collect();
        // panic!("types: {types:#?}");
        // We have all the types we care about, but we still need to extract
        // relevant lifetimes.
        let lifetimes: Vec<Lifetime> = types.iter().flat_map(extract_lifetimes).collect();
        let params = types
            .into_iter()
            .map(Param::Type)
            .chain(lifetimes.into_iter().map(Param::Lifetime));

        let relevant_params: HashSet<Param> = params
            .flat_map(|generic| find_all_generics(&generic, &generic_bounds))
            .collect();

        self.generics = generics_subset(parent_generics, relevant_params.into_iter());
    }
}

// Pull out any lifetimes from a type.
fn extract_lifetimes(ty: &Type) -> Vec<Lifetime> {
    match ty {
        Type::Array(a) => extract_lifetimes(&a.elem),
        Type::BareFn(_) => Vec::new(),
        Type::Group(g) => extract_lifetimes(&g.elem),
        Type::ImplTrait(it) => it
            .bounds
            .iter()
            .filter_map(|b| match b {
                TypeParamBound::Trait(_) => None,
                TypeParamBound::Lifetime(lt) => Some(lt.to_owned()),
            })
            .collect(),
        Type::Infer(_) => Vec::new(),
        Type::Macro(_) => Vec::new(),
        Type::Never(_) => Vec::new(),
        Type::Paren(p) => extract_lifetimes(&p.elem),
        Type::Path(_) => Vec::new(),
        Type::Ptr(p) => extract_lifetimes(&p.elem),
        Type::Reference(r) => r
            .lifetime
            .iter()
            .cloned()
            .chain(extract_lifetimes(&r.elem))
            .collect(),
        Type::Slice(s) => extract_lifetimes(&s.elem),
        Type::TraitObject(to) => to
            .bounds
            .iter()
            .filter_map(|b| match b {
                TypeParamBound::Trait(_) => None,
                TypeParamBound::Lifetime(lt) => Some(lt.to_owned()),
            })
            .collect(),

        Type::Tuple(t) => t.elems.iter().flat_map(extract_lifetimes).collect(),
        Type::Verbatim(_) => Vec::new(),
        #[allow(unknown_lints)]
        #[cfg_attr(test, deny(non_exhaustive_omitted_patterns))]
        _ => Vec::new(),
    }
}

/// Given a generic and a map of bounds, will find all generics that we need.
/// Example:
/// Given `T` and bounds `T: U, U: V, V: W + X, W, X, Y: Z, Z`
/// Will return `T, U, V, W, X`.
fn find_all_generics(param: &Param, bound_map: &HashMap<Param, Vec<TypeParamBound>>) -> Vec<Param> {
    match bound_map.get(param) {
        Some(bounds) => bounds
            .iter()
            .flat_map(|bound| match bound {
                TypeParamBound::Trait(tr) => {
                    // TODO: Handle BoundLifetimes (`for<'a, 'b, 'c>`)
                    tr.path
                        .get_ident()
                        .into_iter()
                        .flat_map(|ident| {
                            let param = ident.to_owned().into();
                            find_all_generics(&param, bound_map)
                        })
                        .collect()
                }
                TypeParamBound::Lifetime(lifetime) => {
                    let param = lifetime.to_owned().into();
                    find_all_generics(&param, bound_map)
                }
            })
            .chain([param.to_owned()].into_iter())
            .collect(),
        None => Vec::new(),
    }
}

/// Given a set of `Generics`, return the subset that we're interested in.
/// Expects `params` already includes all possible types/lifetimes we care
// about.
/// E.g. with generics `T: U, U, V`, this function should never be called with
/// just params of `T`; it would instead expect `T, U`.
/// That is, call `find_all_generics` first.
fn generics_subset(generics: &Generics, params: impl Iterator<Item = Param>) -> Generics {
    let mut new = Generics::default();

    for param in params {
        let (generic_param, predicate) = param.find(generics);
        if let Some(gp) = generic_param {
            new.params.push(gp);
        }
        if let Some(pred) = predicate {
            new.make_where_clause().predicates.push(pred);
        }
    }

    new
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

// Add a bound to generics
fn add_bound(generics: &mut Generics, bound: TypeParamBound) {
    for param in generics.type_params_mut() {
        if param.bounds.iter().all(|b| b != &bound) {
            param.bounds.push(bound.clone());
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

    fn build(&self, parent: &DeriveInput, parent_data: &DataEnum) -> TokenStream2 {
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

            impl std::fmt::Display for #error {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    std::fmt::Debug::fmt(self, f)
                }
            }

            impl std::error::Error for #error {}

            #[automatically_derived]
            impl #parent_impl std::convert::From<#child_ident #child_ty> for #parent_ident #parent_ty #parent_where {
                fn from(child: #child_ident #child_ty) -> Self {
                    match child {
                        #(#from_child_arms),*
                    }
                }
            }

            #[automatically_derived]
            impl #parent_impl std::convert::TryFrom<#parent_ident #parent_ty> for #child_ident #child_ty #parent_where {
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

    let enums: Vec<_> = enums.into_values().map(|e| e.build(&input, data)).collect();

    sanitize_input(&mut input);

    quote!(
        #input

        #(#enums)*
    )
    .into()
}
