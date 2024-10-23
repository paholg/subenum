use alloc::{borrow::ToOwned, boxed::Box, vec::Vec};
use syn::{Ident, Lifetime, Type, TypeParamBound};

use crate::{iter::BoxedIter, param::Param};

pub trait Extractor {
    fn extract_lifetimes(&self) -> Vec<Lifetime>;
    fn extract_idents(&self) -> Vec<Ident>;
    fn extract_params(&self) -> Box<dyn Iterator<Item = Param>> {
        self.extract_lifetimes()
            .into_iter()
            .map(Param::Lifetime)
            .chain(self.extract_idents().into_iter().map(Param::Ident))
            .boxed()
    }
}

impl Extractor for Type {
    fn extract_lifetimes(&self) -> Vec<Lifetime> {
        match self {
            Type::Array(a) => a.elem.extract_lifetimes(),
            Type::BareFn(_) => Vec::new(),
            Type::Group(g) => g.elem.extract_lifetimes(),
            Type::ImplTrait(it) => it
                .bounds
                .iter()
                .cloned()
                .filter_map(|b| match b {
                    TypeParamBound::Trait(_) => None,
                    TypeParamBound::Lifetime(lt) => Some(lt),
                })
                .collect(),
            Type::Infer(_) => Vec::new(),
            Type::Macro(_) => Vec::new(),
            Type::Never(_) => Vec::new(),
            Type::Paren(p) => p.elem.extract_lifetimes(),
            Type::Path(p) => p
                .path
                .segments
                .iter()
                .flat_map(|x| match x.arguments {
                    syn::PathArguments::AngleBracketed(ref ab) => {
                        ab.args.iter().flat_map(|arg| match arg {
                            syn::GenericArgument::Lifetime(lt) => Vec::from([lt.clone()]),
                            syn::GenericArgument::Type(ty) => ty
                                .extract_lifetimes(),
                            syn::GenericArgument::Binding(b) => b
                                .ty
                                .extract_lifetimes(),
                            _=> Vec::new(),
                        })
                        .collect()
                    }
                    syn::PathArguments::Parenthesized(ref p) => p
                        .inputs
                        .iter()
                        .flat_map(|x| x.extract_lifetimes())
                        .collect(),
                    syn::PathArguments::None => Vec::new()
                })
                .collect(),
            Type::Ptr(p) => p.elem.extract_lifetimes(),
            Type::Reference(r) => r
                .lifetime
                .iter()
                .cloned()
                .chain(r.elem.extract_lifetimes())
                .collect(),
            Type::Slice(s) => s.elem.extract_lifetimes(),
            Type::TraitObject(to) => to
                .bounds
                .iter()
                .cloned()
                .filter_map(|b| match b {
                    TypeParamBound::Trait(_) => None,
                    TypeParamBound::Lifetime(lt) => Some(lt),
                })
                .collect(),

            Type::Tuple(t) => t.elems.iter().flat_map(Self::extract_lifetimes).collect(),
            Type::Verbatim(_) => Vec::new(),
            #[allow(unknown_lints)]
            #[cfg_attr(test, deny(non_exhaustive_omitted_patterns))]
            _ => Vec::new(),
        }
    }

    fn extract_idents(&self) -> Vec<Ident> {
        match self {
            Type::Array(a) => a.elem.extract_idents(),
            Type::BareFn(_) => Vec::new(),
            Type::Group(g) => g.elem.extract_idents(),
            Type::ImplTrait(it) => it
                .bounds
                .iter()
                .cloned()
                .filter_map(|b| match b {
                    TypeParamBound::Trait(t) => t.path.get_ident().map(ToOwned::to_owned),
                    TypeParamBound::Lifetime(_) => None,
                })
                .collect(),
            Type::Infer(_) => Vec::new(),
            Type::Macro(_) => Vec::new(),
            Type::Never(_) => Vec::new(),
            Type::Paren(p) => p.elem.extract_idents(),
            Type::Path(p) => p
                .path
                .get_ident()
                .map(ToOwned::to_owned)
                .into_iter()
                .collect(),
            Type::Ptr(p) => p.elem.extract_idents(),
            Type::Reference(r) => r.elem.extract_idents(),
            Type::Slice(s) => s.elem.extract_idents(),
            Type::TraitObject(_) => Vec::new(),
            Type::Tuple(t) => t.elems.iter().flat_map(Self::extract_idents).collect(),
            Type::Verbatim(_) => Vec::new(),
            #[allow(unknown_lints)]
            #[cfg_attr(test, deny(non_exhaustive_omitted_patterns))]
            _ => Vec::new(),
        }
    }
}
