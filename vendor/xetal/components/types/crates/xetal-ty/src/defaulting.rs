//! Defaulting (T5): constrained variables left open at the top level
//! become Int (a number) or Bool (a condition).

use std::collections::HashSet;

use xetal_base::{Diagnostic, Span};

use crate::ty::{Type, TypeVar};
use crate::unify::Unifier;

impl Unifier {
    /// The last variable made so far; pass it to [`Unifier::default_since`].
    pub fn mark(&self) -> u32 {
        self.next
    }

    /// Default every constrained variable created after `mark` (the
    /// variables of one top-level item, including those it shares with
    /// a monomorphic definition): the item is evaluated now (T5).
    /// Variables quantified by a scheme (`skip`) stay open.
    pub fn default_since(
        &mut self,
        mark: u32,
        span: Span,
        skip: &HashSet<TypeVar>,
    ) -> Result<(), Diagnostic> {
        let mut open = Vec::new();
        for v in mark + 1..=self.next {
            self.resolve(&Type::Var(TypeVar(v))).vars(&mut open);
        }
        open.retain(|v| !skip.contains(v));
        for v in open {
            let t = Type::Var(v);
            let defaulted = self.defaulted(&t);
            self.unify(&defaulted, &t, span)?;
        }
        Ok(())
    }

    /// Resolve `ty` and default its constrained variables: a number (or
    /// a number used as a condition) is Int, a condition is Bool (T5).
    pub fn defaulted(&self, ty: &Type) -> Type {
        let ty = self.resolve(ty);
        let mut vars = Vec::new();
        ty.vars(&mut vars);
        let map = vars
            .into_iter()
            .filter_map(|v| {
                let c = self.classes.get(&v).copied().unwrap_or_default();
                let names: Vec<&str> = c.names().collect();
                let default = match (names.contains(&"Num"), names.contains(&"Truthy")) {
                    (true, _) => Type::Int,
                    (false, true) => Type::Bool,
                    _ => return None,
                };
                Some((v, default))
            })
            .collect();
        ty.rename(&map)
    }
}
