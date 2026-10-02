//! The S-expression dump printed by `xetal core`. Built-ins are marked
//! with `#`; node ids and spans are not printed.

use std::fmt;

use xetal_lex::Number;

use crate::ir::{Expr, Item, Kind, Param, Program};

fn number(n: &Number) -> String {
    match n {
        Number::Int(i) => i.to_string(),
        Number::Float(x) => format!("{x:?}"),
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lines: Vec<String> = self.items.iter().map(ToString::to_string).collect();
        f.write_str(&lines.join("\n"))
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Item::Def { name, value } => write!(f, "(def {name} {value})"),
            Item::Let { name, rec, value } => {
                write!(
                    f,
                    "({} {name} {value})",
                    if *rec { "letrec" } else { "let" }
                )
            }
            Item::Set { name, value } => write!(f, "(set {name} {value})"),
            Item::Eval(e) => write!(f, "(eval {e})"),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            Kind::Lit(n) => f.write_str(&number(n)),
            Kind::Str(s) => write!(f, "{s:?}"),
            Kind::Unit => f.write_str("@"),
            Kind::Array(items) => {
                let parts: Vec<String> = items.iter().map(ToString::to_string).collect();
                write!(f, "(array {})", parts.join(" "))
            }
            Kind::Var(name) | Kind::Global(name) => f.write_str(name),
            Kind::Prim(name) => write!(f, "#{name}"),
            Kind::Axes { axes, f: fun, .. } => {
                let digits: String = axes.iter().map(u8::to_string).collect();
                write!(f, "(axes {digits} {fun})")
            }
            Kind::Lam { param, lazy, body } => {
                let p = match param {
                    Param::Name(n) => n.as_str(),
                    Param::Unit => "@",
                };
                write!(f, "(lam {}{p} {body})", if *lazy { "~" } else { "" })
            }
            Kind::App(fun, arg) => write!(f, "(app {fun} {arg})"),
            Kind::App2 {
                f: fun,
                left,
                right,
            } => write!(f, "(app2 {fun} {left} {right})"),
            Kind::Let {
                name,
                rec,
                value,
                body,
            } => {
                write!(
                    f,
                    "({} {name} {value} {body})",
                    if *rec { "letrec" } else { "let" }
                )
            }
            Kind::Set { name, value, body } => write!(f, "(set {name} {value} {body})"),
            Kind::If { cond, then, other } => write!(f, "(if {cond} {then} {other})"),
            Kind::NoMatch => f.write_str("nomatch"),
        }
    }
}
