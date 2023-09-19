use std::borrow::ToOwned;
use std::boxed::Box;
use std::vec::Vec;
use syn::{Ident, Lifetime, Type, TypeParamBound};

use crate::{iter::BoxedIter, param::Param};

pub trait Extractor {
    fn extract_lifetimes(&self) -> Vec<Lifetime>;
    fn extract_types(&self) -> Vec<Ident>;
    fn extract_consts(&self) -> Vec<Ident>;
    fn extract_params(&self) -> Box<dyn Iterator<Item = Param>> {
        self.extract_lifetimes()
            .into_iter()
            .map(Param::Lifetime)
            .chain(self.extract_types().into_iter().map(Param::Type))
            .chain(self.extract_consts().into_iter().map(Param::Const))
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
            Type::Path(_) => Vec::new(),
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

    fn extract_types(&self) -> Vec<Ident> {
        match self {
            Type::Array(a) => a.elem.extract_types(),
            Type::BareFn(_) => Vec::new(),
            Type::Group(g) => g.elem.extract_types(),
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
            Type::Paren(p) => p.elem.extract_types(),
            Type::Path(p) => p
                .path
                .get_ident()
                .map(ToOwned::to_owned)
                .into_iter()
                .collect(),
            Type::Ptr(p) => p.elem.extract_types(),
            Type::Reference(r) => r.elem.extract_types(),
            Type::Slice(s) => s.elem.extract_types(),
            Type::TraitObject(_) => Vec::new(),
            Type::Tuple(t) => t.elems.iter().flat_map(Self::extract_types).collect(),
            Type::Verbatim(_) => Vec::new(),
            #[allow(unknown_lints)]
            #[cfg_attr(test, deny(non_exhaustive_omitted_patterns))]
            _ => Vec::new(),
        }
    }

    fn extract_consts(&self) -> Vec<Ident> {
        match self {
            Type::Array(a) => a.elem.extract_consts(),
            Type::BareFn(_) => Vec::new(),
            Type::Group(g) => g.elem.extract_consts(),
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
            Type::Paren(p) => p.elem.extract_consts(),
            Type::Path(p) => p
                .path
                .get_ident()
                .map(ToOwned::to_owned)
                .into_iter()
                .collect(),
            Type::Ptr(p) => p.elem.extract_consts(),
            Type::Reference(r) => r.elem.extract_consts(),
            Type::Slice(s) => s.elem.extract_consts(),
            Type::TraitObject(_) => Vec::new(),
            Type::Tuple(t) => t.elems.iter().flat_map(Self::extract_consts).collect(),
            Type::Verbatim(_) => Vec::new(),
            #[allow(unknown_lints)]
            #[cfg_attr(test, deny(non_exhaustive_omitted_patterns))]
            _ => Vec::new(),
        }
    }
}
