//! Values a running program left for its host, read without
//! formatting them.
//!
//! The evaluator gives each top-level value to an output hook in place
//! of printing it. A hook made here keeps the last such value, and,
//! when that value is a function, the bindings it was made in, so the
//! host can read a named variable (a mutable `frame!`, say) each time
//! the program waits. The idea is snowflow_demo-fork's.
//!
//! This is a workaround for the missing native hook (ask E1), not an
//! interface X_eTaL promises: it reads the pinned runtime's own
//! environment, and a newer X_eTaL may change it.

use std::cell::RefCell;
use std::rc::Rc;

use xetal_value::{Env, Slot, Value, lookup};

use crate::Error;

#[derive(Default)]
struct Kept<'a> {
    last: Option<Value<'a>>,
    env: Env<'a>,
}

/// What a program published: its last top-level value and the bindings
/// of the last top-level function.
#[derive(Clone, Default)]
pub struct Published<'a> {
    kept: Rc<RefCell<Kept<'a>>>,
}

impl<'a> Published<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The hook to give the evaluator (`Machine::hooks`' `keep`).
    pub fn hook(&self) -> impl FnMut(&Value<'a>) + use<'a> {
        let kept = self.kept.clone();
        move |value: &Value<'a>| {
            let mut kept = kept.borrow_mut();
            if let Value::Closure(closure) = value {
                kept.env = closure.env.clone();
            }
            kept.last = Some(value.clone());
        }
    }

    /// The last top-level value the program gave, if any yet.
    #[must_use]
    pub fn last(&self) -> Option<Value<'a>> {
        self.kept.borrow().last.clone()
    }

    /// The value of the variable `name` where the program's last
    /// top-level function was made. The value is shared, not copied.
    ///
    /// # Errors
    ///
    /// When no function has been given yet, the name is not bound
    /// there, or it has no value yet.
    pub fn named(&self, caller: &str, name: &str) -> Result<Value<'a>, Error> {
        let kept = self.kept.borrow();
        if kept.env.is_none() {
            return Err(Error::new(
                caller,
                "the program has published no function yet",
            ));
        }
        let slot = lookup(&kept.env, name).ok_or_else(|| {
            Error::new(
                caller,
                format!("{name} is not a variable the program published"),
            )
        })?;
        match &*slot.borrow() {
            Slot::Value(value) => Ok(value.clone()),
            _ => Err(Error::new(caller, format!("{name} has no value yet"))),
        }
    }
}
