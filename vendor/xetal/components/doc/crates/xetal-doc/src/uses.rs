//! The names a definition's value uses: qualified names (`u:f_`, a
//! library's `LA:x`) and top-level variables not shadowed by a lambda
//! parameter or a local binding.

use std::collections::{BTreeSet, HashSet};

use xetal_core::{Expr, Kind, Param};

/// The top-level names `value` refers to, in sorted order.
pub(crate) fn used(value: &Expr, tops: &HashSet<String>) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    walk(value, &mut Vec::new(), tops, &mut out);
    out
}

fn walk(e: &Expr, bound: &mut Vec<String>, tops: &HashSet<String>, out: &mut BTreeSet<String>) {
    match &e.kind {
        Kind::Global(name) => {
            out.insert(name.clone());
        }
        Kind::Var(name) if tops.contains(name) && !bound.contains(name) => {
            out.insert(name.clone());
        }
        Kind::Lam { param, body, .. } => {
            let name = match param {
                Param::Name(n) => n.clone(),
                Param::Unit => String::new(),
            };
            scoped(name, body, bound, tops, out);
        }
        Kind::Let {
            name, value, body, ..
        } => {
            walk(value, bound, tops, out);
            scoped(name.clone(), body, bound, tops, out);
        }
        _ => children(e)
            .into_iter()
            .for_each(|c| walk(c, bound, tops, out)),
    }
}

/// Walk `body` with `name` bound.
fn scoped(
    name: String,
    body: &Expr,
    bound: &mut Vec<String>,
    tops: &HashSet<String>,
    out: &mut BTreeSet<String>,
) {
    bound.push(name);
    walk(body, bound, tops, out);
    bound.pop();
}

/// The sub-expressions of a node that binds nothing.
fn children(e: &Expr) -> Vec<&Expr> {
    match &e.kind {
        Kind::Array(items) => items.iter().collect(),
        Kind::Axes { f, .. } => vec![f],
        Kind::App(f, x) => vec![f, x],
        Kind::App2 { f, left, right } => vec![f, left, right],
        Kind::Set { value, body, .. } => vec![value, body],
        Kind::If { cond, then, other } => vec![cond, then, other],
        _ => Vec::new(),
    }
}
