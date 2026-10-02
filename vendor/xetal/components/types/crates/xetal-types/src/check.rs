//! The checker's entry points: check a source or a lowered program.

use xetal_base::Diagnostic;
use xetal_core::Program;

use crate::infer::infer_program;

/// Lex, parse, desugar and infer a whole program; one line per item.
pub fn check_source(src: &str) -> Result<Vec<String>, Diagnostic> {
    check_program(&mut xetal_core::lower(src)?)
}

/// Infer `program`, then elaborate it (`xetal-elab`): polymorphic
/// number code takes the number type it is used at, so evaluation
/// agrees with the types (T6).
pub fn check_program(program: &mut Program) -> Result<Vec<String>, Diagnostic> {
    let (lines, dicts) = infer_program(program)?;
    xetal_elab::elaborate(program, &dicts);
    Ok(lines)
}
