//! Which lambda arguments (`_l`, `_r`) a lambda body uses.

use xetal_lex::Side;

use crate::ast::{Expr, ExprKind, Fun, FunKind, Stmt};

/// Uses of `_l` / `_r` belonging to this lambda (not nested ones).
pub fn stmt_args(stmt: &Stmt, acc: (bool, bool)) -> (bool, bool) {
    match stmt {
        Stmt::Bind { value, .. } => expr_args(value, acc),
        Stmt::Guard { cond, result, .. } => expr_args(result, expr_args(cond, acc)),
        Stmt::Expr(e) => expr_args(e, acc),
    }
}

fn expr_args(e: &Expr, (l, r): (bool, bool)) -> (bool, bool) {
    let side = |s: &Side| (l || *s == Side::Left, r || *s == Side::Right);
    match &e.kind {
        ExprKind::Arg(s) => side(s),
        ExprKind::Strand(items) => items.iter().fold((l, r), |acc, x| expr_args(x, acc)),
        ExprKind::Pow { base, .. } => expr_args(base, (l, r)),
        ExprKind::Quote(f) | ExprKind::Fn(f) => fun_args(f, (l, r)),
        ExprKind::Monadic { f, arg } => expr_args(arg, fun_args(f, (l, r))),
        ExprKind::Dyadic { left, f, right } => {
            expr_args(right, expr_args(left, fun_args(f, (l, r))))
        }
        _ => (l, r),
    }
}

fn fun_args(f: &Fun, (l, r): (bool, bool)) -> (bool, bool) {
    match &f.kind {
        FunKind::Arg(s) => (l || *s == Side::Left, r || *s == Side::Right),
        FunKind::Apply(e) => expr_args(e, (l, r)),
        FunKind::Operand { operand, f } => fun_args(f, fun_args(operand, (l, r))),
        FunKind::Power { f, .. } => fun_args(f, (l, r)),
        FunKind::Train(fs) => fs.iter().fold((l, r), |acc, x| fun_args(x, acc)),
        _ => (l, r),
    }
}
