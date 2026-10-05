//! Giving a value to the frame on top of the stack: the top level's
//! items, bindings, guards and arrays here; application in `apply`.

use std::rc::Rc;

use xetal_arith::truth;
use xetal_array::Array;
use xetal_base::Diagnostic;
use xetal_core::Item;
use xetal_value::{Slot, Value, extend};

use crate::kont::err;
use crate::kont::{Control, Kont};
use crate::machine::Machine;

impl<'a> Machine<'a, '_> {
    /// The frame on top takes `v`; `None` when the program has finished.
    pub(crate) fn resume(&mut self, v: Value<'a>) -> Result<Option<Control<'a>>, Diagnostic> {
        let Some(k) = self.stack.pop() else {
            return Err(Diagnostic::new(
                "internal",
                "a value with nothing waiting for it",
            ));
        };
        Ok(Some(match k {
            Kont::Items { next } => return self.item(next),
            Kont::Def { name } => {
                self.globals.insert(name.to_string(), v);
                Control::Return(Value::Unit)
            }
            Kont::Show { span } => self.show(&v, span)?,
            k @ (Kont::Bind { .. }
            | Kont::Assign { .. }
            | Kont::Choose { .. }
            | Kont::Array { .. }) => self.resume_local(k, v)?,
            Kont::Axes { axes, arity, span } => Control::Return(self.axes(axes, arity, v, span)?),
            other => self.resume_app(other, v)?,
        }))
    }

    /// The frames of bindings, guards and arrays take their value.
    fn resume_local(&mut self, k: Kont<'a>, v: Value<'a>) -> Result<Control<'a>, Diagnostic> {
        Ok(match k {
            Kont::Bind {
                name,
                env,
                inner,
                body,
            } => {
                let bound = match inner {
                    Some(inner) => {
                        if let Some(slot) = xetal_value::lookup(&inner, name) {
                            *slot.borrow_mut() = Slot::Value(v);
                        }
                        inner
                    }
                    None => extend(&env, name, Slot::Value(v)),
                };
                self.then(body, bound)
            }
            Kont::Assign { slot, env, body } => {
                *slot.borrow_mut() = Slot::Value(v);
                self.then(body, env)
            }
            Kont::Choose {
                then,
                other,
                env,
                span,
            } => Control::Eval(if truth(&v, span)? { then } else { other }, env),
            Kont::Array {
                items,
                left,
                done,
                env,
            } => self.array(items, left, done, env, v),
            _ => return Err(Diagnostic::new("internal", "a frame out of place")),
        })
    }

    /// One more array item in hand (right to left): the next, or the array.
    fn array(
        &mut self,
        items: &'a [xetal_core::Expr],
        left: usize,
        mut done: Vec<Value<'a>>,
        env: xetal_value::Env<'a>,
        v: Value<'a>,
    ) -> Control<'a> {
        done.push(v);
        if left == 0 {
            done.reverse();
            return Control::Return(Value::Array(Rc::new(Array::vector(done))));
        }
        let next = &items[left - 1];
        let k = Kont::Array {
            items,
            left: left - 1,
            done,
            env: env.clone(),
        };
        self.stack.push(k);
        Control::Eval(next, env)
    }

    /// After a binding: its body, or at the top level the next item
    /// with the new bindings.
    fn then(
        &mut self,
        body: Option<&'a xetal_core::Expr>,
        env: xetal_value::Env<'a>,
    ) -> Control<'a> {
        match body {
            Some(body) => Control::Eval(body, env),
            None => {
                self.env = env;
                Control::Return(Value::Unit)
            }
        }
    }

    /// Start the item `next`, or finish when there is none.
    fn item(&mut self, next: usize) -> Result<Option<Control<'a>>, Diagnostic> {
        let program = self.program;
        let Some(item) = program.items.get(next) else {
            return Ok(None);
        };
        self.stack.push(Kont::Items { next: next + 1 });
        let env = self.env.clone();
        if let Some(hook) = self.before.as_mut() {
            hook(match item {
                Item::Def { value, .. } | Item::Let { value, .. } | Item::Set { value, .. } => {
                    value.span
                }
                Item::Eval(e) => e.span,
            });
        }
        Ok(Some(match item {
            Item::Def { name, value } => {
                if self.globals.contains_key(name) {
                    return Err(err(
                        "duplicate-definition",
                        value.span,
                        format!("{name} is already defined in this file"),
                    ));
                }
                self.stack.push(Kont::Def { name });
                Control::Eval(value, env)
            }
            Item::Let { name, rec, value } => self.bind(name, *rec, value, &env, None),
            Item::Set { name, value } => self.assign(name, value, &env, None)?,
            Item::Eval(e) => {
                self.stack.push(Kont::Show { span: e.span });
                Control::Eval(e, env)
            }
        }))
    }

    /// A top-level value: printed, or given to the keeper.
    fn show(&mut self, v: &Value<'a>, span: xetal_base::Span) -> Result<Control<'a>, Diagnostic> {
        match self.keep.as_mut() {
            Some(keep) => keep(v),
            None => writeln!(self.out, "{}", xetal_value::printed(v))
                .map_err(|x| err("io", span, x.to_string()))?,
        }
        Ok(Control::Return(Value::Unit))
    }
}
