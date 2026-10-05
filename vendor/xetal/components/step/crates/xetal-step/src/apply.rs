//! Application as frames: the function first, then its arguments right
//! to left (E4); an argument for a lazy (`~`) parameter passed
//! unevaluated (E1); a built-in called once it has all its arguments.

use xetal_base::{Diagnostic, Span};
use xetal_core::{Expr, Kind, Param};
use xetal_value::{Env, Slot, Value, extend};

use crate::kont::err;
use crate::kont::{Control, Kont};
use crate::machine::Machine;

impl<'a> Machine<'a, '_> {
    /// The application frames take their value.
    pub(crate) fn resume_app(
        &mut self,
        k: Kont<'a>,
        v: Value<'a>,
    ) -> Result<Control<'a>, Diagnostic> {
        match k {
            Kont::Arg { x, env, span } if lazy_first(&v) => {
                self.apply(v, Slot::Thunk(x, env), span)
            }
            Kont::Arg { x, env, span } => Ok(self.push(Kont::Call { f: v, span }, x, env)),
            Kont::Call { f, span } => self.apply(f, Slot::Value(v), span),
            Kont::Pair { l, r, env, span } if lazy_second(&v) => {
                Ok(self.left(v, Slot::Thunk(r, env.clone()), l, env, span)?)
            }
            Kont::Pair { l, r, env, span } => Ok(self.push(
                Kont::PairRight {
                    f: v,
                    l,
                    env: env.clone(),
                    span,
                },
                r,
                env,
            )),
            Kont::PairRight { f, l, env, span } => self.left(f, Slot::Value(v), l, env, span),
            Kont::PairLeft { f, right, span } => {
                self.stack.push(Kont::ApplyTo { arg: right, span });
                self.apply(f, Slot::Value(v), span)
            }
            Kont::ApplyTo { arg, span } => self.apply(v, arg, span),
            Kont::Force { slot, .. } => {
                *slot.borrow_mut() = Slot::Value(v.clone());
                Ok(Control::Return(v))
            }
            Kont::PrimArg { p, span } => self.prim(&p, v, span),
            Kont::UnitBody { body, env, span } => self.unit_body(v, body, env, span),
            Kont::Called => Ok(Control::Return(v)),
            Kont::Kernel { kernel, span } => self.drive(kernel, Some(v), span),
            _ => Err(Diagnostic::new("internal", "a frame out of place")),
        }
    }

    fn push(&mut self, k: Kont<'a>, next: &'a Expr, env: Env<'a>) -> Control<'a> {
        self.stack.push(k);
        Control::Eval(next, env)
    }

    /// `x f y` once y is in hand: x next (unevaluated if lazy).
    fn left(
        &mut self,
        f: Value<'a>,
        right: Slot<'a>,
        l: &'a Expr,
        env: Env<'a>,
        span: Span,
    ) -> Result<Control<'a>, Diagnostic> {
        if lazy_first(&f) {
            self.stack.push(Kont::ApplyTo { arg: right, span });
            return self.apply(f, Slot::Thunk(l, env), span);
        }
        Ok(self.push(Kont::PairLeft { f, right, span }, l, env))
    }

    /// `f` applied to one argument.
    pub(crate) fn apply(
        &mut self,
        f: Value<'a>,
        arg: Slot<'a>,
        span: Span,
    ) -> Result<Control<'a>, Diagnostic> {
        match f {
            Value::Closure(c) => match (c.param, arg) {
                (Param::Name(name), arg) => {
                    self.stack.push(Kont::Called);
                    Ok(Control::Eval(c.body, extend(&c.env, name, arg)))
                }
                (Param::Unit, Slot::Value(v)) => self.unit_body(v, c.body, c.env.clone(), span),
                (Param::Unit, Slot::Thunk(e, env)) => {
                    let k = Kont::UnitBody {
                        body: c.body,
                        env: c.env.clone(),
                        span,
                    };
                    Ok(self.push(k, e, env))
                }
                _ => Err(err(
                    "internal",
                    span,
                    "an argument is neither a value nor a thunk",
                )),
            },
            Value::Prim(p) => match arg {
                Slot::Value(v) => self.prim(&p, v, span),
                Slot::Thunk(e, env) => Ok(self.push(Kont::PrimArg { p, span }, e, env)),
                _ => Err(err(
                    "internal",
                    span,
                    "an argument is neither a value nor a thunk",
                )),
            },
            other => Err(err(
                "not-a-function",
                span,
                format!("{other} is not a function"),
            )),
        }
    }

    /// A niladic function's body, once its argument is known to be `@`.
    fn unit_body(
        &mut self,
        v: Value<'a>,
        body: &'a Expr,
        env: Env<'a>,
        span: Span,
    ) -> Result<Control<'a>, Diagnostic> {
        match v {
            Value::Unit => Ok(self.push(Kont::Called, body, env)),
            other => Err(err(
                "not-unit",
                span,
                format!("this function takes @, got {other}"),
            )),
        }
    }
}

fn lazy_first(v: &Value) -> bool {
    matches!(v, Value::Closure(c) if c.lazy)
}

fn lazy_second(v: &Value) -> bool {
    matches!(v, Value::Closure(c) if matches!(c.body.kind, Kind::Lam { lazy: true, .. }))
}
