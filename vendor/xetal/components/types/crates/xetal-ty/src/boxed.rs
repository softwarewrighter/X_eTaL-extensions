//! Boxes and classes (A7, B14): only Eq admits a box, and a box is in
//! Eq when what it holds is, so the class passes on to the item type.

use xetal_base::{Diagnostic, Span};

use crate::Classes;
use crate::ty::Type;
use crate::unify::Unifier;

impl Unifier {
    /// A box is in a class (only Eq admits boxes) when what it holds
    /// is: pass the classes on to the item type.
    pub(crate) fn constrain(
        &mut self,
        t: &Type,
        classes: Classes,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if classes == Classes::default() {
            return Ok(());
        }
        let fresh = self.fresh_in(classes);
        self.unify(&fresh, t, span)
    }
}
