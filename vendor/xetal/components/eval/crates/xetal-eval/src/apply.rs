//! Application: the function is evaluated first, then its arguments
//! right to left (E4); an argument for a lazy (`~`) parameter is passed
//! unevaluated (E1).

use std::rc::Rc;

use xetal_base::{Diagnostic, Span};
use xetal_core::{Expr, Kind, Param};

use crate::machine::Machine;
use crate::prim;
use crate::run::err;
use xetal_value::{Env, Prim, Slot, Value, extend};

impl<'a> Machine<'a, '_> {
    pub(crate) fn app(
        &mut self,
        f: &'a Expr,
        x: &'a Expr,
        env: &Env<'a>,
        span: Span,
    ) -> Result<Value<'a>, Diagnostic> {
        let fv = self.eval(f, env)?;
        let arg = self.argument(x, env, lazy_first(&fv))?;
        self.apply(fv, arg, span)
    }

    /// `x f y`: f, then y, then x; laziness is known for a lambda written
    /// with both parameters.
    pub(crate) fn app2(
        &mut self,
        f: &'a Expr,
        l: &'a Expr,
        r: &'a Expr,
        env: &Env<'a>,
        span: Span,
    ) -> Result<Value<'a>, Diagnostic> {
        let fv = self.eval(f, env)?;
        let right = self.argument(r, env, lazy_second(&fv))?;
        let left = self.argument(l, env, lazy_first(&fv))?;
        let partial = self.apply(fv, left, span)?;
        self.apply(partial, right, span)
    }

    fn argument(&mut self, x: &'a Expr, env: &Env<'a>, lazy: bool) -> Result<Slot<'a>, Diagnostic> {
        if lazy {
            return Ok(Slot::Thunk(x, env.clone()));
        }
        Ok(Slot::Value(self.eval(x, env)?))
    }

    pub(crate) fn apply(
        &mut self,
        fv: Value<'a>,
        arg: Slot<'a>,
        span: Span,
    ) -> Result<Value<'a>, Diagnostic> {
        match fv {
            Value::Closure(c) => match c.param {
                Param::Name(name) => {
                    let env = extend(&c.env, name, arg);
                    self.eval(c.body, &env)
                }
                Param::Unit => match self.force(arg)? {
                    Value::Unit => self.eval(c.body, &c.env),
                    other => Err(err(
                        "not-unit",
                        span,
                        format!("this function takes @, got {other}"),
                    )),
                },
            },
            Value::Prim(p) => {
                let mut args = p.args.clone();
                args.push(self.force(arg)?);
                if args.len() < p.arity {
                    return Ok(Value::Prim(Rc::new(Prim {
                        name: p.name,
                        arity: p.arity,
                        args,
                    })));
                }
                match xetal_hof::call(p.name, &args, span, self) {
                    Some(result) => result,
                    None => prim::call(p.name, &args, span, self.out, &mut self.rng),
                }
            }
            other => Err(err(
                "not-a-function",
                span,
                format!("{other} is not a function"),
            )),
        }
    }

    fn force(&mut self, slot: Slot<'a>) -> Result<Value<'a>, Diagnostic> {
        match slot {
            Slot::Value(v) => Ok(v),
            Slot::Thunk(e, env) => self.eval(e, &env),
            Slot::Forcing | Slot::Empty => unreachable!("arguments are values or thunks"),
        }
    }
}

fn lazy_first(v: &Value) -> bool {
    matches!(v, Value::Closure(c) if c.lazy)
}

fn lazy_second(v: &Value) -> bool {
    matches!(v, Value::Closure(c) if matches!(c.body.kind, Kind::Lam { lazy: true, .. }))
}
