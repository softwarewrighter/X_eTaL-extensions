//! One step of evaluating an expression: a leaf gives its value; any
//! other form pushes the work waiting for its parts and evaluates the
//! first part (right to left, E4).

use std::rc::Rc;

use xetal_array::Array;
use xetal_base::{Diagnostic, Span};
use xetal_core::{Expr, Kind};
use xetal_lex::Number;
use xetal_value::{Closure, Env, Prim, Slot, Value, extend, lookup};

use crate::kont::err;
use crate::kont::{Control, Kont};
use crate::machine::Machine;

impl<'a> Machine<'a, '_> {
    pub(crate) fn eval(&mut self, e: &'a Expr, env: &Env<'a>) -> Result<Control<'a>, Diagnostic> {
        Ok(match &e.kind {
            Kind::Let {
                name,
                rec,
                value,
                body,
            } => self.bind(name, *rec, value, env, Some(body)),
            Kind::Set { name, value, body } => self.assign(name, value, env, Some(body))?,
            Kind::Var(name) => self.var(name, e.span, env)?,
            Kind::App(..) | Kind::App2 { .. } | Kind::If { .. } | Kind::Axes { .. } => {
                self.compound(e, env)
            }
            Kind::Array(items) if !items.is_empty() => self.array_start(items, env),
            _ => Control::Return(self.leaf(e, env)?),
        })
    }

    /// Forms with parts: the work waiting for the first part's value
    /// pushed, and that part next.
    fn compound(&mut self, e: &'a Expr, env: &Env<'a>) -> Control<'a> {
        let (env, span) = (env.clone(), e.span);
        let (k, next): (Kont<'a>, &'a Expr) = match &e.kind {
            Kind::App(f, x) => (
                Kont::Arg {
                    x,
                    env: env.clone(),
                    span,
                },
                f,
            ),
            Kind::App2 { f, left, right } => (
                Kont::Pair {
                    l: left,
                    r: right,
                    env: env.clone(),
                    span,
                },
                f,
            ),
            Kind::If { cond, then, other } => (
                Kont::Choose {
                    then,
                    other,
                    env: env.clone(),
                    span: cond.span,
                },
                cond,
            ),
            Kind::Axes { axes, arity, f } => (
                Kont::Axes {
                    axes,
                    arity: *arity,
                    span,
                },
                f,
            ),
            _ => return Control::Eval(e, env),
        };
        self.stack.push(k);
        Control::Eval(next, env)
    }

    /// An array's items, right to left: the last one first.
    fn array_start(&mut self, items: &'a [Expr], env: &Env<'a>) -> Control<'a> {
        let left = items.len() - 1;
        let done = Vec::new();
        self.stack.push(Kont::Array {
            items,
            left,
            done,
            env: env.clone(),
        });
        Control::Eval(&items[left], env.clone())
    }

    /// Literals, globals, built-ins, lambdas, the empty array.
    fn leaf(&mut self, e: &'a Expr, env: &Env<'a>) -> Result<Value<'a>, Diagnostic> {
        Ok(match &e.kind {
            Kind::Lit(Number::Int(i)) => Value::Int(*i),
            Kind::Lit(Number::Float(x)) => Value::Float(*x),
            Kind::Unit => Value::Unit,
            Kind::Str(text) => Value::Array(Rc::new(Array::vector(
                text.chars().map(Value::Char).collect(),
            ))),
            Kind::Array(_) => Value::Array(Rc::new(Array::vector(Vec::new()))),
            Kind::Global(name) => {
                let v = self.globals.get(name).cloned();
                v.ok_or_else(|| err("undefined-name", e.span, format!("{name} is not defined")))?
            }
            Kind::Prim(name) => {
                let (name, arity) = xetal_prim::arity(name, e.span)?;
                Value::Prim(Rc::new(Prim {
                    name,
                    arity,
                    args: Vec::new(),
                }))
            }
            Kind::Lam { param, lazy, body } => {
                let closure = Closure {
                    param,
                    lazy: *lazy,
                    body,
                    env: env.clone(),
                };
                Value::Closure(Rc::new(closure))
            }
            Kind::NoMatch => {
                return Err(err(
                    "no-guard-matched",
                    e.span,
                    "no guard matched and no statement follows",
                ));
            }
            _ => return Err(err("internal", e.span, "not a leaf")),
        })
    }

    /// A variable's value, forcing a lazy argument on first use (E1).
    fn var(&mut self, name: &str, span: Span, env: &Env<'a>) -> Result<Control<'a>, Diagnostic> {
        let slot = lookup(env, name)
            .ok_or_else(|| err("undefined-name", span, format!("{name} is not defined")))?;
        let (expr, thunk_env) = match &*slot.borrow() {
            Slot::Value(v) => return Ok(Control::Return(v.clone())),
            Slot::Thunk(expr, thunk_env) => (*expr, thunk_env.clone()),
            Slot::Forcing => {
                return Err(err(
                    "infinite-recursion",
                    span,
                    format!("{name} depends on itself"),
                ));
            }
            Slot::Empty => {
                return Err(err(
                    "undefined-name",
                    span,
                    format!("{name} is used before it has a value"),
                ));
            }
        };
        *slot.borrow_mut() = Slot::Forcing;
        self.stack.push(Kont::Force {
            slot,
            expr,
            env: thunk_env.clone(),
        });
        Ok(Control::Eval(expr, thunk_env))
    }

    /// `name := value` (recursive when `rec`), then `body` or the next item.
    pub(crate) fn bind(
        &mut self,
        name: &'a str,
        rec: bool,
        value: &'a Expr,
        env: &Env<'a>,
        body: Option<&'a Expr>,
    ) -> Control<'a> {
        let inner = rec.then(|| extend(env, name, Slot::Empty));
        let at = inner.clone().unwrap_or_else(|| env.clone());
        self.stack.push(Kont::Bind {
            name,
            env: env.clone(),
            inner,
            body,
        });
        Control::Eval(value, at)
    }

    /// `name := value` for a `!` variable already bound (M2).
    pub(crate) fn assign(
        &mut self,
        name: &str,
        value: &'a Expr,
        env: &Env<'a>,
        body: Option<&'a Expr>,
    ) -> Result<Control<'a>, Diagnostic> {
        let slot = lookup(env, name).ok_or_else(|| {
            err(
                "undefined-name",
                value.span,
                format!("{name} is not defined"),
            )
        })?;
        self.stack.push(Kont::Assign {
            slot,
            env: env.clone(),
            body,
        });
        Ok(Control::Eval(value, env.clone()))
    }
}
