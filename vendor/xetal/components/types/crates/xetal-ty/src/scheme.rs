//! Class constraints on type variables, and generalization,
//! instantiation and defaulting of types.

use std::collections::HashMap;

use crate::Classes;
use crate::ty::{Scheme, Type};

use crate::unify::Unifier;

impl Unifier {
    /// Quantify the variables of `ty` not free in `env` (the types of
    /// the enclosing bindings).
    pub fn generalize(&self, ty: &Type, env: &[Type]) -> Scheme {
        let ty = self.resolve(ty);
        let mut fixed = Vec::new();
        for t in env {
            self.resolve(t).vars(&mut fixed);
        }
        let mut vars = Vec::new();
        ty.vars(&mut vars);
        vars.retain(|v| !fixed.contains(v));
        let classes = vars
            .iter()
            .filter_map(|v| self.classes.get(v).map(|c| (*v, *c)))
            .collect();
        Scheme { vars, classes, ty }
    }

    /// A fresh copy of a scheme's type, and the fresh types standing for
    /// its `Num` variables, in the order of `scheme.num()` (the number-type
    /// arguments a use of the scheme passes, T6).
    pub fn instantiate(&mut self, scheme: &Scheme) -> (Type, Vec<Type>) {
        let mut map = HashMap::new();
        for v in &scheme.vars {
            map.insert(*v, self.fresh_in(scheme.class_of(*v)));
        }
        let nums = scheme.num().iter().map(|v| map[v].clone()).collect();
        (scheme.ty.rename(&map), nums)
    }
}

/// A scheme with no quantified variables.
pub fn mono(ty: Type) -> Scheme {
    Scheme {
        vars: Vec::new(),
        classes: Vec::new(),
        ty,
    }
}

impl Scheme {
    /// The classes of quantified variable `v`.
    pub fn class_of(&self, v: crate::TypeVar) -> Classes {
        self.classes
            .iter()
            .find(|(w, _)| *w == v)
            .map_or_else(Classes::default, |(_, c)| *c)
    }

    /// The quantified `Num` variables, in order (dictionary passing, T6).
    pub fn num(&self) -> Vec<crate::TypeVar> {
        let is_num = |v: &crate::TypeVar| self.class_of(*v).names().any(|n| n == "Num");
        self.vars.iter().copied().filter(is_num).collect()
    }
}
