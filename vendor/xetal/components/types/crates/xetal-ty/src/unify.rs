//! Unification over a substitution, with an occurs check and the class
//! constraints `Num` (Int, Float) and `Truthy` (Bool, Int) on type
//! variables (T5).

use std::collections::HashMap;

use xetal_base::{Diagnostic, Span};

use crate::Classes;
use crate::ty::{Type, TypeVar};

#[derive(Debug, Default)]
pub struct Unifier {
    pub(crate) subst: HashMap<TypeVar, Type>,
    pub(crate) classes: HashMap<TypeVar, Classes>,
    pub(crate) next: u32,
}

impl Unifier {
    /// A fresh variable in `classes`.
    pub fn fresh_in(&mut self, classes: Classes) -> Type {
        self.next += 1;
        let v = TypeVar(self.next);
        if classes != Classes::default() {
            self.classes.insert(v, classes);
        }
        Type::Var(v)
    }

    pub fn fresh(&mut self) -> Type {
        self.fresh_in(Classes::default())
    }

    /// A fresh variable that must be a number (Int or Float).
    pub fn fresh_num(&mut self) -> Type {
        self.fresh_in(Classes::named("Num").unwrap_or_default())
    }

    /// A fresh variable that must be usable as a condition (Bool or Int).
    pub fn fresh_truthy(&mut self) -> Type {
        self.fresh_in(Classes::named("Truthy").unwrap_or_default())
    }

    /// Apply the substitution everywhere in `ty`.
    pub fn resolve(&self, ty: &Type) -> Type {
        match ty {
            Type::Var(v) => match self.subst.get(v) {
                Some(t) => self.resolve(t),
                None => ty.clone(),
            },
            Type::Fn(a, b) => Type::Fn(Box::new(self.resolve(a)), Box::new(self.resolve(b))),
            Type::Box(a) => Type::Box(Box::new(self.resolve(a))),
            other => other.clone(),
        }
    }

    pub fn unify(&mut self, expected: &Type, found: &Type, span: Span) -> Result<(), Diagnostic> {
        let (a, b) = (self.resolve(expected), self.resolve(found));
        match (&a, &b) {
            _ if a == b => Ok(()),
            (Type::Var(v), t) => self.bind(*v, t, span, true),
            (t, Type::Var(v)) => self.bind(*v, t, span, false),
            (Type::Fn(a1, a2), Type::Fn(b1, b2)) => {
                self.unify(a1, b1, span)?;
                self.unify(a2, b2, span)
            }
            (Type::Box(a1), Type::Box(b1)) => self.unify(a1, b1, span),
            _ => Err(
                Diagnostic::new("type-mismatch", format!("expected {a}, found {b}"))
                    .with_span(span),
            ),
        }
    }

    /// Bind `v` to `t`; `expected` says which side of the mismatch `v`
    /// was on, for the message.
    fn bind(&mut self, v: TypeVar, t: &Type, span: Span, expected: bool) -> Result<(), Diagnostic> {
        let mut vars = Vec::new();
        t.vars(&mut vars);
        if vars.contains(&v) {
            return Err(Diagnostic::new(
                "infinite-type",
                format!("a type would contain itself: {t}"),
            )
            .with_span(span));
        }
        let classes = self.classes.get(&v).copied().unwrap_or_default();
        match t {
            Type::Var(w) => {
                let other = self.classes.get(w).copied().unwrap_or_default();
                let merged = classes.union(other);
                if merged != Classes::default() {
                    self.classes.insert(*w, merged);
                }
            }
            t if !classes.admits(t) => {
                let (want, got) = (classes.describe(), t.to_string());
                let (e, f) = if expected { (want, got) } else { (got, want) };
                return Err(
                    Diagnostic::new("type-mismatch", format!("expected {e}, found {f}"))
                        .with_span(span),
                );
            }
            Type::Box(inner) => self.constrain(inner, classes, span)?,
            _ => {}
        }
        self.subst.insert(v, t.clone());
        Ok(())
    }
}
