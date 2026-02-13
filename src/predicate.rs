use crate::param::Param;
use crate::visitor::ParamVisitor;
use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use syn::visit::Visit;
use syn::{Generics, PredicateLifetime, PredicateType, Type, WherePredicate};

/// Represents a single constraint and the specific generics it relies on.
#[derive(Debug, Clone)]
pub struct PredicateDependency {
    /// The normalized predicate (e.g., "T: Debug" or "Option<T>: Clone")
    pub predicate: WherePredicate,
    /// The set of generics used inside this specific predicate.
    /// e.g. for "Option<T>: Iterator<Item=U>", this contains {T, U}.
    pub used_params: BTreeSet<Param>,
}

pub fn analyze_generics(generics: &Generics) -> Vec<PredicateDependency> {
    let visitor = ParamVisitor::new(generics);
    let mut results = Vec::new();
    let mut predicates = Vec::new();

    // A. Convert Inline Bounds (<T: Debug>) to WherePredicates
    for param in &generics.params {
        match param {
            syn::GenericParam::Type(t) => {
                if !t.bounds.is_empty() {
                    let pred = WherePredicate::Type(PredicateType {
                        lifetimes: None,
                        bounded_ty: Type::Path(syn::TypePath {
                            qself: None,
                            path: t.ident.clone().into(),
                        }),
                        colon_token: Default::default(),
                        bounds: t.bounds.clone(),
                    });
                    predicates.push(pred);
                }
            }
            syn::GenericParam::Lifetime(l) => {
                if !l.bounds.is_empty() {
                    let pred = WherePredicate::Lifetime(PredicateLifetime {
                        lifetime: l.lifetime.clone(),
                        colon_token: Default::default(),
                        bounds: l.bounds.clone(),
                    });
                    predicates.push(pred);
                }
            }
            _ => {}
        }
    }

    // B. Collect Explicit Where Clause Predicates
    if let Some(where_clause) = &generics.where_clause {
        predicates.extend(where_clause.predicates.iter().cloned());
    }

    // C. Analyze dependencies for each predicate
    for predicate in predicates {
        // Reset the visitor for this specific predicate
        let mut visitor = visitor.clone();

        visitor.visit_where_predicate(&predicate);

        results.push(PredicateDependency {
            predicate,
            used_params: visitor.found,
        });
    }

    results
}
