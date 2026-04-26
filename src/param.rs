use syn::{Ident, Lifetime};

/// A type or lifetime param, potentially used as a generic.
/// E.g. the 'a in `'a: 'b + 'c` or the T in `T: U + V`.
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
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
