use crate::param::Param;
use alloc::collections::BTreeSet;
use syn::{
    visit::{self, Visit},
    Generics, Lifetime, Path,
};

#[derive(Debug, Clone)]
pub struct ParamVisitor {
    // The list of all generics declared on the struct
    declared: BTreeSet<Param>,
    // The generics we actually found being used
    pub found: BTreeSet<Param>,
}

impl ParamVisitor {
    pub fn new(generics: &Generics) -> Self {
        let mut declared = BTreeSet::new();
        for param in &generics.params {
            if let syn::GenericParam::Lifetime(lifetime_def) = param {
                declared.insert(Param::Lifetime(lifetime_def.lifetime.clone()));
            }
            if let syn::GenericParam::Type(type_param) = param {
                declared.insert(Param::Ident(type_param.ident.clone()));
            }
            if let syn::GenericParam::Const(type_param) = param {
                declared.insert(Param::Ident(type_param.ident.clone()));
            }
        }
        Self {
            declared,
            found: Default::default(),
        }
    }

    fn register(&mut self, param: &Param) {
        if let Some(param) = self.declared.get(param) {
            self.found.insert(param.clone());
        }
    }
}

impl<'ast> Visit<'ast> for ParamVisitor {
    fn visit_lifetime(&mut self, i: &'ast Lifetime) {
        self.register(&Param::Lifetime(i.clone()));
        visit::visit_lifetime(self, i);
    }

    fn visit_path(&mut self, path: &'ast Path) {
        if let Some(segment) = path.segments.first() {
            self.register(&Param::Ident(segment.ident.clone()));
        }
        visit::visit_path(self, path);
    }
}
