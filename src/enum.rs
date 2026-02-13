use crate::predicate::analyze_generics;
use crate::visitor::ParamVisitor;
use crate::{param::Param, Derive};
use alloc::{collections::BTreeSet, vec::Vec};
use proc_macro2::TokenStream;
use syn::visit::Visit;
use syn::{punctuated::Punctuated, Generics, Ident, Token, Variant, WherePredicate};

pub struct Enum {
    pub ident: Ident,
    pub variants: Punctuated<Variant, Token![,]>,
    pub variants_attributes: Vec<Vec<TokenStream>>,
    pub attributes: Vec<TokenStream>,
    pub derives: Vec<Derive>,
    pub generics: Generics,
}

impl Enum {
    pub fn new(ident: Ident, attributes: Vec<TokenStream>, derives: Vec<Derive>) -> Self {
        Enum {
            ident,
            variants: Punctuated::new(),
            variants_attributes: Vec::new(),
            attributes,
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
        // 1. Analyze constraints: Convert all inline bounds and where clauses
        //    into a list of PredicateDependency
        let mut deps = analyze_generics(parent_generics);

        // 2. Identify "Root" params: The generics explicitly used in the variants.
        let mut visitor = ParamVisitor::new(parent_generics);
        for variant in &self.variants {
            visitor.visit_variant(variant);
        }

        let mut active_params: BTreeSet<Param> = visitor.found;
        let mut active_predicates: Vec<WherePredicate> = Vec::new();

        // 3. Repeatedly iterate through dependencies. If a predicate mentions
        //    ANY active param, we must keep that predicate AND activate
        //    any other params it mentions.
        let mut changed = true;
        while changed {
            changed = false;

            // We retain only the predicates we haven't matched yet.
            deps.retain(|dep| {
                // Check if this dependency touches any currently active param
                let is_relevant = dep.used_params.iter().any(|p| active_params.contains(p));

                if is_relevant {
                    // It is relevant: Keep the predicate
                    active_predicates.push(dep.predicate.clone());

                    // Activate all params used by this predicate
                    for p in &dep.used_params {
                        if active_params.insert(p.clone()) {
                            // If we added a NEW param, we must loop again
                            // to check for bounds dependent on this new param.
                            changed = true;
                        }
                    }
                    // Remove from `deps` so we don't process it again
                    return false;
                }
                true // Keep in `deps` for next pass
            });
        }
        // 4. Construct the final Generics struct in-place
        self.generics = Generics::default();

        // A. Filter params and strip inline bounds
        for param in &parent_generics.params {
            let keep = match param {
                syn::GenericParam::Type(t) => {
                    active_params.contains(&Param::Ident(t.ident.clone()))
                }
                syn::GenericParam::Lifetime(l) => {
                    active_params.contains(&Param::Lifetime(l.lifetime.clone()))
                }
                syn::GenericParam::Const(c) => {
                    active_params.contains(&Param::Ident(c.ident.clone()))
                }
            };

            if keep {
                let mut p = param.clone();
                // CRITICAL: We clear inline bounds here because `analyze_generics`
                // has already converted them into predicates. If we don't clear them,
                // we will have duplicates (once in <> and once in where clause).
                match &mut p {
                    syn::GenericParam::Type(t) => t.bounds.clear(),
                    syn::GenericParam::Lifetime(l) => l.bounds.clear(),
                    _ => {}
                }
                self.generics.params.push(p);
            }
        }

        // B. Append the collected predicates to the where clause
        if !active_predicates.is_empty() {
            let where_clause = self.generics.make_where_clause();
            where_clause.predicates.extend(active_predicates);
        }
    }
}
