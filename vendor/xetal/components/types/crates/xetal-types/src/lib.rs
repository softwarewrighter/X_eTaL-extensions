//! Type checking: Algorithm W over Core with Haskell-style numbers
//! (T5), and elaboration of the checked program (T6). The type
//! representation and unifier live in `xetal-ty`.

mod check;
mod closing;
mod expr;
mod infer;
mod record;
mod subscript;

pub use check::{check_program, check_source};
pub use infer::infer_program;
pub use xetal_ty::{Scheme, Type, TypeVar, Unifier};
