//! The evaluation of Core items and expressions.

use std::collections::HashMap;
use std::io::Write;
use std::rc::Rc;

use xetal_array::Array;
use xetal_base::{Diagnostic, Span};
use xetal_core::{Expr, Item, Kind, Program};
use xetal_lex::Number;

use crate::events::Shown;
use crate::prim;
use crate::run::err;
use xetal_arith::{Rng, truth};
use xetal_value::{Closure, Env, Prim, Slot, Value, extend, lookup};

/// Nested evaluations allowed before reporting `stack-overflow`.
const MAX_DEPTH: usize = 100_000;

pub(crate) struct Machine<'a, 'o> {
    pub globals: HashMap<String, Value<'a>>,
    pub out: &'o mut dyn Write,
    pub depth: usize,
    pub rng: Rng,
    /// When set, top-level values are kept for display, not printed.
    pub shown: Option<Shown>,
    /// Called with each top-level item's span just before it runs.
    pub before: Option<&'o mut (dyn FnMut(Span) + Send)>,
}

impl<'a> Machine<'a, '_> {
    pub fn run(&mut self, program: &'a Program) -> Result<(), Diagnostic> {
        let mut env: Env<'a> = None;
        for item in &program.items {
            if let Some(hook) = self.before.as_mut() {
                hook(match item {
                    Item::Def { value, .. } | Item::Let { value, .. } | Item::Set { value, .. } => {
                        value.span
                    }
                    Item::Eval(e) => e.span,
                });
            }
            match item {
                Item::Def { name, value } => {
                    if self.globals.contains_key(name) {
                        return Err(err(
                            "duplicate-definition",
                            value.span,
                            format!("{name} is already defined in this file"),
                        ));
                    }
                    let v = self.eval(value, &env)?;
                    self.globals.insert(name.clone(), v);
                }
                Item::Let { name, rec, value } => env = self.bind(name, *rec, value, &env)?,
                Item::Set { name, value } => self.set(name, value, &env)?,
                Item::Eval(e) => {
                    let v = self.eval(e, &env)?;
                    match &mut self.shown {
                        Some(s) => s.values.push((s.text.len(), xetal_value::grid(&v))),
                        None => writeln!(self.out, "{}", xetal_value::printed(&v))
                            .map_err(|x| err("io", e.span, x.to_string()))?,
                    }
                }
            }
        }
        Ok(())
    }

    fn bind(
        &mut self,
        name: &'a str,
        rec: bool,
        value: &'a Expr,
        env: &Env<'a>,
    ) -> Result<Env<'a>, Diagnostic> {
        if !rec {
            let v = self.eval(value, env)?;
            return Ok(extend(env, name, Slot::Value(v)));
        }
        let inner = extend(env, name, Slot::Empty);
        let v = self.eval(value, &inner)?;
        if let Some(slot) = lookup(&inner, name) {
            *slot.borrow_mut() = Slot::Value(v);
        }
        Ok(inner)
    }

    fn set(&mut self, name: &str, value: &'a Expr, env: &Env<'a>) -> Result<(), Diagnostic> {
        let slot = lookup(env, name).ok_or_else(|| {
            err(
                "undefined-name",
                value.span,
                format!("{name} is not defined"),
            )
        })?;
        let v = self.eval(value, env)?;
        *slot.borrow_mut() = Slot::Value(v);
        Ok(())
    }

    pub fn eval(&mut self, e: &'a Expr, env: &Env<'a>) -> Result<Value<'a>, Diagnostic> {
        if self.depth >= MAX_DEPTH {
            return Err(err("stack-overflow", e.span, "recursion is too deep"));
        }
        self.depth += 1;
        let result = self.eval_kind(e, env);
        self.depth -= 1;
        result
    }

    fn eval_kind(&mut self, e: &'a Expr, env: &Env<'a>) -> Result<Value<'a>, Diagnostic> {
        Ok(match &e.kind {
            Kind::Lit(_)
            | Kind::Unit
            | Kind::Str(_)
            | Kind::Array(_)
            | Kind::Axes { .. }
            | Kind::Var(_)
            | Kind::Global(_)
            | Kind::Prim(_) => return self.atom(e, env),
            Kind::Lam { param, lazy, body } => Value::Closure(Rc::new(Closure {
                param,
                lazy: *lazy,
                body,
                env: env.clone(),
            })),
            Kind::App(f, x) => return self.app(f, x, env, e.span),
            Kind::App2 { f, left, right } => return self.app2(f, left, right, env, e.span),
            Kind::Let {
                name,
                rec,
                value,
                body,
            } => {
                let inner = self.bind(name, *rec, value, env)?;
                return self.eval(body, &inner);
            }
            Kind::Set { name, value, body } => {
                self.set(name, value, env)?;
                return self.eval(body, env);
            }
            Kind::If { cond, then, other } => {
                let c = self.eval(cond, env)?;
                return self.eval(if truth(&c, cond.span)? { then } else { other }, env);
            }
            Kind::NoMatch => {
                return Err(err(
                    "no-guard-matched",
                    e.span,
                    "no guard matched and no statement follows",
                ));
            }
        })
    }

    /// Leaves: literals, names and built-ins.
    fn atom(&mut self, e: &'a Expr, env: &Env<'a>) -> Result<Value<'a>, Diagnostic> {
        Ok(match &e.kind {
            Kind::Lit(Number::Int(i)) => Value::Int(*i),
            Kind::Lit(Number::Float(x)) => Value::Float(*x),
            Kind::Unit => Value::Unit,
            Kind::Var(name) => return self.var(name, e.span, env),
            Kind::Global(name) => {
                return self.globals.get(name).cloned().ok_or_else(|| {
                    err("undefined-name", e.span, format!("{name} is not defined"))
                });
            }
            Kind::Prim(name) => {
                let (name, arity) = prim::arity(name, e.span)?;
                Value::Prim(Rc::new(Prim {
                    name,
                    arity,
                    args: Vec::new(),
                }))
            }
            Kind::Str(text) => Value::Array(Rc::new(Array::vector(
                text.chars().map(Value::Char).collect(),
            ))),
            Kind::Array(items) => {
                let mut values = items
                    .iter()
                    .rev()
                    .map(|x| self.eval(x, env))
                    .collect::<Result<Vec<_>, _>>()?;
                values.reverse();
                Value::Array(Rc::new(Array::vector(values)))
            }
            Kind::Axes { axes, arity, f } => {
                let fv = self.eval(f, env)?;
                return self.axes(axes, *arity, fv, e.span);
            }
            _ => return Err(err("internal", e.span, "not a leaf")),
        })
    }

    /// A variable's value, forcing a lazy argument on first use (E1).
    fn var(&mut self, name: &str, span: Span, env: &Env<'a>) -> Result<Value<'a>, Diagnostic> {
        let slot = lookup(env, name)
            .ok_or_else(|| err("undefined-name", span, format!("{name} is not defined")))?;
        let (expr, thunk_env) = match &*slot.borrow() {
            Slot::Value(v) => return Ok(v.clone()),
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
        let v = self.eval(expr, &thunk_env);
        *slot.borrow_mut() = match &v {
            Ok(v) => Slot::Value(v.clone()),
            Err(_) => Slot::Thunk(expr, thunk_env),
        };
        v
    }
}
