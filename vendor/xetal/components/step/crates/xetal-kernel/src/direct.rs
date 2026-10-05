//! Operands called at once: a first-order built-in operand (`'*`,
//! `'+`, `'r_ight`, partly applied or not) runs no user code, so nothing
//! can pause inside it, and a higher-order built-in may compute with it
//! directly instead of asking the evaluator for each call (Saga 30).

use xetal_base::Diagnostic;
use xetal_value::Value;

/// The runner's way to call a built-in operand directly.
pub trait Direct<'a> {
    /// Whether `f`, given `n` more arguments, is a first-order built-in
    /// call this runner makes at once.
    fn takes(&self, f: &Value<'a>, n: usize) -> bool;

    /// `f` applied to `args` (only where [`Direct::takes`] said yes).
    fn call(&mut self, f: &Value<'a>, args: &[Value<'a>]) -> Result<Value<'a>, Diagnostic>;
}

/// A runner that calls nothing directly: every call goes through the
/// kernel (for tests, and runners without built-ins).
pub struct Never;

impl<'a> Direct<'a> for Never {
    fn takes(&self, _: &Value<'a>, _: usize) -> bool {
        false
    }

    fn call(&mut self, _: &Value<'a>, _: &[Value<'a>]) -> Result<Value<'a>, Diagnostic> {
        Err(Diagnostic::new("internal", "no operand is called directly"))
    }
}
