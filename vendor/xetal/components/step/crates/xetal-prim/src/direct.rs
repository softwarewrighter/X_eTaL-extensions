//! The evaluator's direct calls (Saga 30): a first-order built-in
//! operand of a higher-order built-in is called at once, without the
//! machine, since it runs no user code and cannot pause. A built-in
//! that waits for input (`[]R_EAD`, `[]K_EY`) or calls functions itself
//! (a higher-order one) still goes through the machine.

use std::io::Write;

use xetal_arith::Rng;
use xetal_base::{Diagnostic, Span};
use xetal_kernel::Direct;
use xetal_value::Value;

/// Calls made at once, printing to `out` and rolling from `rng` as the
/// machine would, errors placed at `span`.
pub struct Now<'r> {
    pub out: &'r mut dyn Write,
    pub rng: &'r mut Rng,
    pub span: Span,
}

impl<'a> Direct<'a> for Now<'_> {
    fn takes(&self, f: &Value<'a>, n: usize) -> bool {
        matches!(f, Value::Prim(p) if p.args.len() + n == p.arity
            && !xetal_hof::higher(p.name)
            && !matches!(p.name, "[]R_EAD" | "[]K_EY"))
    }

    fn call(&mut self, f: &Value<'a>, args: &[Value<'a>]) -> Result<Value<'a>, Diagnostic> {
        let Value::Prim(p) = f else {
            return Err(
                Diagnostic::new("internal", "a direct call of a non-built-in").with_span(self.span),
            );
        };
        match p.args.is_empty() {
            true => crate::call(p.name, args, self.span, self.out, self.rng),
            false => {
                let all = [p.args.as_slice(), args].concat();
                crate::call(p.name, &all, self.span, self.out, self.rng)
            }
        }
    }
}
