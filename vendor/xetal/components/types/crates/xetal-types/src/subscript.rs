//! A function under an axis subscript (A6) has the function's type;
//! the type is recorded so elaboration can tell the evaluator how many
//! arguments the function takes.

use xetal_base::Diagnostic;
use xetal_core::Expr;
use xetal_ty::Type;

use crate::infer::Infer;

impl Infer {
    pub(crate) fn subscripted(&mut self, e: &Expr, f: &Expr) -> Result<Type, Diagnostic> {
        let t = self.expr(f)?;
        self.rec.axes.push((e.id, t.clone()));
        Ok(t)
    }
}
