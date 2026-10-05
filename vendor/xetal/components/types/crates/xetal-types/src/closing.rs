//! Closing a binding's type (T5): a lambda is generalized, a value
//! stays monomorphic and, at the top level, has its numbers defaulted;
//! a top-level condition stays a condition, so it is an Int in
//! arithmetic and a Bool in a guard, as written inline (T1).

use xetal_base::Diagnostic;
use xetal_core::{Expr, Kind};
use xetal_ty::{Scheme, Type, mono};

use crate::infer::{Global, Infer};

impl Infer {
    /// A lambda value is generalized; anything else stays monomorphic,
    /// and at the top level its numbers default now (Int, Bool; T5).
    pub(crate) fn close(
        &mut self,
        t: &Type,
        value: &Expr,
        top: bool,
    ) -> Result<Scheme, Diagnostic> {
        if matches!(value.kind, Kind::Lam { .. }) {
            let scheme = self.u.generalize(t, &self.fixed());
            self.rec.generalized(value.id, &scheme);
            return Ok(scheme);
        }
        if top {
            self.u
                .default_since(self.mark, value.span, &self.rec.quantified)?;
            let defaulted = self.u.defaulted(t);
            self.u.unify(&defaulted, t, value.span)?;
        }
        Ok(mono(self.u.resolve(t)))
    }

    /// A top-level value whose type is a condition (`Truthy`, not yet a
    /// Bool or an Int) is generalized: each use is a Bool or an Int as it
    /// needs (T1); its value is the same at run time.
    pub(crate) fn condition(&mut self, t: &Type) -> Option<Scheme> {
        let Type::Var(v) = self.u.resolve(t) else {
            return None;
        };
        let scheme = self.u.generalize(&Type::Var(v), &self.fixed());
        let truthy = scheme.vars == [v] && scheme.class_of(v).names().eq(["Truthy"]);
        truthy.then_some(scheme)
    }

    /// The types the environment fixes (not generalized over).
    fn fixed(&self) -> Vec<Type> {
        let mut fixed: Vec<Type> = self.env.iter().map(|(_, s, _)| s.ty.clone()).collect();
        fixed.extend(self.globals.values().filter_map(|g| match g {
            Global::Pending(t, _) | Global::Open { ty: t, .. } => Some(t.clone()),
            Global::Defined(_) => None,
        }));
        fixed
    }
}
