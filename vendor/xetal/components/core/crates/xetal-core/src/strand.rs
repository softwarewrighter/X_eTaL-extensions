//! Strand items: a number stays as it is; a string in a strand is
//! enclosed, so a strand of strings is a nested vector (B14).

use xetal_base::Diagnostic;
use xetal_syntax::{Expr as Surface, ExprKind};

use crate::lower::Lower;
use xetal_ir::{Expr, Kind};

impl Lower {
    pub(crate) fn strand_item(&mut self, e: &Surface) -> Result<Expr, Diagnostic> {
        let item = self.expr(e)?;
        if !matches!(e.kind, ExprKind::Str(_)) {
            return Ok(item);
        }
        let f = self.node(e.span, Kind::Prim("e_nclose".into()));
        Ok(self.node(e.span, Kind::App(Box::new(f), Box::new(item))))
    }
}
