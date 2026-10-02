//! Warnings: a parameter or local binding that shadows a built-in (L7).

use xetal_base::Diagnostic;
use xetal_catalog::find;
use xetal_ir::{Expr, Item, Kind, Param, Program};

/// The warnings for a lowered program.
pub fn warnings(program: &Program) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for item in &program.items {
        match item {
            Item::Def { value, .. }
            | Item::Let { value, .. }
            | Item::Set { value, .. }
            | Item::Eval(value) => {
                walk(value, &mut out);
            }
        }
    }
    out
}

fn walk(e: &Expr, out: &mut Vec<Diagnostic>) {
    let shadow = |name: &str, out: &mut Vec<Diagnostic>| {
        if find(name).is_some() {
            let message = format!("`{name}` here shadows the built-in {name}");
            out.push(Diagnostic::warning("shadows-builtin", message).with_span(e.span));
        }
    };
    match &e.kind {
        Kind::Lam { param, body, .. } => {
            if let Param::Name(name) = param {
                shadow(name, out);
            }
            walk(body, out);
        }
        Kind::Let {
            name, value, body, ..
        } => {
            shadow(name, out);
            walk(value, out);
            walk(body, out);
        }
        Kind::Array(items) => items.iter().for_each(|x| walk(x, out)),
        Kind::Axes { f, .. } => walk(f, out),
        Kind::App(f, x) => {
            walk(f, out);
            walk(x, out);
        }
        Kind::App2 { f, left, right } => [f, left, right].iter().for_each(|x| walk(x, out)),
        Kind::Set { value, body, .. } => [value, body].iter().for_each(|x| walk(x, out)),
        Kind::If { cond, then, other } => [cond, then, other].iter().for_each(|x| walk(x, out)),
        _ => {}
    }
}
