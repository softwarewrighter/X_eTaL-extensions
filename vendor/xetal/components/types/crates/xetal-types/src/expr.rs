//! Algorithm W for Core expressions.

use xetal_base::Diagnostic;
use xetal_core::{Expr, Kind, Param};
use xetal_lex::Number;

use crate::infer::{Global, Infer};
use crate::record::{Args, Binder};
use xetal_ty::Type;
use xetal_ty::mono;

fn fun(a: Type, b: Type) -> Type {
    Type::Fn(Box::new(a), Box::new(b))
}

impl Infer {
    pub(crate) fn expr(&mut self, e: &Expr) -> Result<Type, Diagnostic> {
        Ok(match &e.kind {
            Kind::Lit(Number::Int(_)) => self.int_literal(e.id),
            Kind::Lit(Number::Float(_)) => Type::Float,
            Kind::Str(_) => Type::Char,
            Kind::Unit => Type::Unit,
            Kind::Array(items) => {
                let elem = self.u.fresh();
                for item in items {
                    let t = self.expr(item)?;
                    self.u.unify(&elem, &t, item.span)?;
                }
                elem
            }
            Kind::Var(name) => self.variable(name, e)?,
            Kind::Global(name) => self.global(name, e),
            Kind::Prim(name) => self.builtin(name, e)?,
            Kind::Axes { f, .. } => self.subscripted(e, f)?,
            Kind::Lam { param, body, .. } => self.lambda(param, body)?,
            Kind::App(f, x) => {
                let (tf, tx, r) = (self.expr(f)?, self.expr(x)?, self.u.fresh());
                self.u.unify(&tf, &fun(tx, r.clone()), e.span)?;
                r
            }
            Kind::App2 { f, left, right } => {
                let (tf, tl, tr, r) = (
                    self.expr(f)?,
                    self.expr(left)?,
                    self.expr(right)?,
                    self.u.fresh(),
                );
                self.u.unify(&tf, &fun(tl, fun(tr, r.clone())), e.span)?;
                r
            }
            Kind::Let {
                name,
                rec,
                value,
                body,
            } => self.let_in(name, *rec, value, body)?,
            Kind::Set { name, value, body } => {
                self.set(name, value)?;
                self.expr(body)?
            }
            Kind::If { cond, then, other } => self.guard(cond, then, other)?,
            Kind::NoMatch => self.u.fresh(),
        })
    }

    fn lambda(&mut self, param: &Param, body: &Expr) -> Result<Type, Diagnostic> {
        let arg = match param {
            Param::Name(name) => {
                let a = self.u.fresh();
                self.env.push((name.clone(), mono(a.clone()), None));
                a
            }
            Param::Unit => Type::Unit,
        };
        let body = self.expr(body);
        if matches!(param, Param::Name(_)) {
            self.env.pop();
        }
        Ok(fun(arg, body?))
    }

    fn let_in(
        &mut self,
        name: &str,
        rec: bool,
        value: &Expr,
        body: &Expr,
    ) -> Result<Type, Diagnostic> {
        let t = self.binding(name, rec, value)?;
        let scheme = self.close(&t, value, false)?;
        self.env.push((name.to_string(), scheme, Some(value.id)));
        let body = self.expr(body);
        self.env.pop();
        body
    }

    /// A guard: the condition is Truthy (T1, T5); both branches agree.
    fn guard(&mut self, cond: &Expr, then: &Expr, other: &Expr) -> Result<Type, Diagnostic> {
        let (tc, want) = (self.expr(cond)?, self.u.fresh_truthy());
        self.u.unify(&want, &tc, cond.span)?;
        let (tt, to) = (self.expr(then)?, self.expr(other)?);
        self.u.unify(&tt, &to, other.span)?;
        Ok(tt)
    }

    fn variable(&mut self, name: &str, e: &Expr) -> Result<Type, Diagnostic> {
        match self.env.iter().rev().find(|(n, ..)| n == name) {
            Some((_, scheme, binder)) => {
                let (scheme, binder) = (scheme.clone(), *binder);
                let (t, nums) = self.u.instantiate(&scheme);
                let args = match binder {
                    Some(id) if scheme.vars.is_empty() => Args::Mono(Binder::Node(id)),
                    _ => Args::Inst(nums),
                };
                self.rec.use_of(e.id, args);
                Ok(t)
            }
            None => Err(
                Diagnostic::new("undefined-name", format!("{name} is not defined"))
                    .with_span(e.span),
            ),
        }
    }

    /// A module definition: its scheme, or one monomorphic type until it
    /// is defined (so definitions may refer to later ones).
    fn global(&mut self, name: &str, e: &Expr) -> Type {
        match self.globals.get(name) {
            Some(Global::Defined(scheme)) => {
                let scheme = scheme.clone();
                let (t, nums) = self.u.instantiate(&scheme);
                self.rec.use_of(e.id, Args::Inst(nums));
                t
            }
            Some(Global::Pending(t, _) | Global::Open { ty: t, .. }) => {
                let t = t.clone();
                self.rec
                    .use_of(e.id, Args::Mono(Binder::Global(name.into())));
                t
            }
            None => {
                let t = self.u.fresh();
                self.globals
                    .insert(name.to_string(), Global::Pending(t.clone(), e.span));
                self.rec
                    .use_of(e.id, Args::Mono(Binder::Global(name.into())));
                t
            }
        }
    }
}
