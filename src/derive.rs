use proc_macro2::Span;
use syn::{Ident, Path, TraitBound, TraitBoundModifier, TypeParamBound};

pub mod partial_eq;

#[derive(Clone, Copy, Debug, Ord, PartialOrd, PartialEq, Eq)]
pub enum Derive {
    PartialEq,
}

impl Derive {
    pub fn as_bound(&self) -> TypeParamBound {
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
