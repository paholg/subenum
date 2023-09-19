use std::collections::HashMap;

use syn::{GenericParam, Generics, Ident, Lifetime, TypeParamBound, WherePredicate};

use crate::extractor::Extractor;

/// A type or lifetime param, potentially used as a generic.
/// E.g. the 'a in `'a: 'b + 'c` or the T in `T: U + V`.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Param {
    Lifetime(Lifetime),
    Ident(Ident),
}

impl From<Ident> for Param {
    fn from(value: Ident) -> Self {
        Param::Ident(value)
    }
}

impl From<Lifetime> for Param {
    fn from(value: Lifetime) -> Self {
        Param::Lifetime(value)
    }
}

impl Param {
    /// Given a set of `Generics`, finds the first `GenericParam` and
    /// `WherePredicate` where it appears.
    pub fn find<'b>(
        &self,
        generics: &'b Generics,
    ) -> (Option<&'b GenericParam>, Option<&'b WherePredicate>) {
        match self {
            Param::Lifetime(lt) => find_lt(lt, generics),
            Param::Ident(ty) => find_ident(ty, generics),
        }
    }

    /// Given a param and a map of bounds, will find all params that we may
    /// need.
    /// Example:
    /// Given `T` and bounds `T: U, U: V, V: W + X, W, X, Y: Z, Z`
    /// Will return `T, U, V, W, X`.
    pub fn find_relevant(&self, bound_map: &HashMap<Param, Vec<TypeParamBound>>) -> Vec<Param> {
        match bound_map.get(self) {
            Some(bounds) => bounds
                .iter()
                .flat_map(|bound| match bound {
                    TypeParamBound::Trait(tr) => {
                        // TODO: Handle BoundLifetimes (`for<'a, 'b, 'c>`)
                        tr.path
                            .get_ident()
                            .into_iter()
                            .cloned()
                            .flat_map(|ident| Param::from(ident).find_relevant(bound_map))
                            .collect()
                    }
                    TypeParamBound::Lifetime(lifetime) => {
                        Param::from(lifetime.clone()).find_relevant(bound_map)
                    }
                })
                .chain([self.clone()])
                .collect(),
            None => Vec::new(),
        }
    }
}
fn find_lt<'a>(
    lt: &Lifetime,
    generics: &'a Generics,
) -> (Option<&'a GenericParam>, Option<&'a WherePredicate>) {
    let generic_param = generics.params.iter().find(|p| match p {
        GenericParam::Lifetime(lt_def) => &lt_def.lifetime == lt,
        _ => false,
    });

    let predicate = generics
        .where_clause
        .iter()
        .flat_map(|wh| wh.predicates.iter())
        .find(|pred| match pred {
            WherePredicate::Lifetime(pred_lt) => &pred_lt.lifetime == lt,
            _ => false,
        });

    (generic_param, predicate)
}

fn find_ident<'a>(
    ident: &Ident,
    generics: &'a Generics,
) -> (Option<&'a GenericParam>, Option<&'a WherePredicate>) {
    let generic_param = generics.params.iter().find(|p| match p {
        GenericParam::Type(ty_param) => &ty_param.ident == ident,
        _ => false,
    });

    let predicate = generics
        .where_clause
        .iter()
        .flat_map(|wh| wh.predicates.iter())
        .find(|pred| match pred {
            WherePredicate::Type(pred_ty) => pred_ty
                .bounded_ty
                .extract_idents()
                .iter()
                .any(|id| id == ident),
            _ => false,
        });

    (generic_param, predicate)
}
