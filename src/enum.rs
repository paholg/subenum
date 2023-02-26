use std::collections::{HashMap, HashSet};

use syn::{punctuated::Punctuated, Generics, Ident, Token, TypeParamBound, Variant};

use crate::{extractor::Extractor, iter::BoxedIter, param::Param, Derive};

pub struct Enum {
    pub ident: Ident,
    pub variants: Punctuated<Variant, Token![,]>,
    pub derives: Vec<Derive>,
    pub generics: Generics,
}

impl Enum {
    pub fn new(ident: Ident, derives: Vec<Derive>) -> Self {
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

    pub fn compute_generics(&mut self, parent_generics: &Generics) {
        let generic_bounds: HashMap<Param, Vec<TypeParamBound>> = parent_generics
            .type_params()
            .map(|param| {
                (
                    Param::Ident(param.ident.clone()),
                    param.bounds.iter().cloned().collect(),
                )
            })
            .chain(parent_generics.lifetimes().map(|lifetime_def| {
                (
                    Param::Lifetime(lifetime_def.lifetime.clone()),
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
                    .flat_map(|pred| match pred {
                        syn::WherePredicate::Type(ty) => {
                            // We have to be a bit careful here. Imagine the bound
                            // <T as Add<U>>:: Foo
                            // We need to treat this as a bound on both `T` and on `U`.
                            let bounds: Vec<TypeParamBound> = ty.bounds.iter().cloned().collect();
                            ty.bounded_ty
                                .extract_idents()
                                .into_iter()
                                .map(move |ident| (Param::Ident(ident), bounds.clone()))
                                .boxed()
                        }
                        syn::WherePredicate::Lifetime(lt) => [(
                            Param::Lifetime(lt.lifetime.clone()),
                            lt.bounds
                                .iter()
                                .cloned()
                                .map(TypeParamBound::Lifetime)
                                .collect(),
                        )]
                        .into_iter()
                        .boxed(),
                        syn::WherePredicate::Eq(_) => {
                            panic!("Equality predicates in where clauses are unsupported")
                        }
                    }),
            )
            .collect();

        // panic!("{generic_bounds:#?}");

        let types = self
            .variants
            .iter()
            .flat_map(|variant| match &variant.fields {
                syn::Fields::Named(named) => named.named.iter().map(|field| &field.ty).collect(),
                syn::Fields::Unnamed(unnamed) => {
                    unnamed.unnamed.iter().map(|field| &field.ty).collect()
                }
                syn::Fields::Unit => Vec::new(),
            });
        // Extract all of the lifetimes and idents we care about from the types.
        let params = types.into_iter().flat_map(|ty| ty.extract_params());

        // The same generic may appear in multiple bounds, so we use a HashSet to dedup.
        let relevant_params: HashSet<Param> = params
            .flat_map(|param| param.find_relevant(&generic_bounds))
            .collect();

        self.generics = generics_subset(parent_generics, relevant_params.into_iter());
    }
}

/// Given a set of `Generics`, return the subset that we're interested in.
/// Expects `params` already includes all possible types/lifetimes we care
// about.
/// E.g. with generics `T: U, U, V`, this function should never be called with
/// just params of `T`; it would instead expect `T, U`.
/// In short: call `find_all_generics` first.
fn generics_subset(generics: &Generics, params: impl Iterator<Item = Param>) -> Generics {
    let mut new = Generics::default();

    for param in params {
        let (generic_param, predicate) = param.find(generics);
        if let Some(gp) = generic_param {
            new.params.push(gp.clone());
        }
        if let Some(pred) = predicate {
            new.make_where_clause().predicates.push(pred.clone());
        }
    }

    new
}
