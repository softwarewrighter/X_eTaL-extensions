//! Core nodes the elaborator adds. They reuse the id and span of the
//! node they elaborate.

use xetal_core::{Expr, Kind, Param};
use xetal_lex::Number;
use xetal_ty::{Type, TypeVar};

/// Hidden parameters in scope: a quantified variable and its name.
pub(crate) type Scope = Vec<(TypeVar, String)>;

fn node(like: &Expr, kind: Kind) -> Expr {
    Expr {
        id: like.id,
        span: like.span,
        kind,
    }
}

/// The zero of number type `t`: a literal, or the hidden parameter that
/// carries it. A variable no binding quantifies defaults to Int (T5).
pub(crate) fn zero(like: &Expr, t: &Type, scope: &Scope) -> Expr {
    let kind = match t {
        Type::Float => Kind::Lit(Number::Float(0.0)),
        Type::Var(v) => match scope.iter().rev().find(|(w, _)| w == v) {
            Some((_, name)) => Kind::Var(name.clone()),
            None => Kind::Lit(Number::Int(0)),
        },
        _ => Kind::Lit(Number::Int(0)),
    };
    node(like, kind)
}

/// `{ name -> body }`, a hidden parameter.
pub(crate) fn lam(name: String, body: Expr) -> Expr {
    let kind = Kind::Lam {
        param: Param::Name(name),
        lazy: false,
        body: Box::new(body.clone()),
    };
    node(&body, kind)
}

/// `f x`.
pub(crate) fn app(f: Expr, x: Expr) -> Expr {
    node(&f.clone(), Kind::App(Box::new(f), Box::new(x)))
}

/// A built-in of `arity` arguments whose result must have number type
/// `t`: `{ #a1 ... #an -> f_loat (prim #a1 ... #an) }` at Float,
/// `... + zero` at a quantified number type, else the built-in itself.
pub(crate) fn typed_result(prim: &Expr, t: &Type, arity: usize, scope: &Scope) -> Expr {
    let var = |name: &str| node(prim, Kind::Var(name.into()));
    let names: Vec<String> = (1..=arity).map(|i| format!("#a{i}")).collect();
    let call = names.iter().fold(prim.clone(), |f, n| app(f, var(n)));
    let body = match zero(prim, t, scope).kind {
        Kind::Lit(Number::Float(_)) => app(node(prim, Kind::Prim("f_loat".into())), call),
        Kind::Var(name) => node(
            prim,
            app2(node(prim, Kind::Prim("+".into())), call, var(&name)),
        ),
        _ => return prim.clone(),
    };
    names.into_iter().rev().fold(body, |body, n| lam(n, body))
}

fn app2(f: Expr, left: Expr, right: Expr) -> Kind {
    Kind::App2 {
        f: Box::new(f),
        left: Box::new(left),
        right: Box::new(right),
    }
}

/// The integer literal `n` at number type `t`.
pub(crate) fn literal(like: &Expr, n: i64, t: &Type, scope: &Scope) -> Expr {
    match zero(like, t, scope).kind {
        Kind::Lit(Number::Float(_)) => node(like, Kind::Lit(Number::Float(n as f64))),
        Kind::Var(name) => {
            let plus = node(like, Kind::Prim("+".into()));
            let n = node(like, Kind::Lit(Number::Int(n)));
            node(like, app2(plus, n, node(like, Kind::Var(name))))
        }
        _ => node(like, Kind::Lit(Number::Int(n))),
    }
}
