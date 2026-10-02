//! The S-expression dump printed by `xetal parse`.

use std::fmt;

use xetal_lex::{FuncName, Number, Side};

use crate::ast::{Expr, ExprKind, Fun, FunKind, Params, Program, Stmt, Target};

fn number(n: &Number) -> String {
    match n {
        Number::Int(i) => i.to_string(),
        Number::Float(x) => format!("{x:?}"),
    }
}

fn func_name(n: &FuncName) -> String {
    let ns = xetal_lex::ns_text(&n.ns);
    let axes: String = n.axes.iter().map(u8::to_string).collect();
    let sub = if axes.is_empty() {
        String::new()
    } else {
        format!("_{axes}")
    };
    format!("{ns}{}{sub}", n.spelled())
}

fn target(t: &Target) -> String {
    match t {
        Target::Var(v) => {
            let ns = v.ns.as_ref().map_or(String::new(), |ns| format!("{ns}:"));
            format!("{ns}{}{}", v.name, if v.mutable { "!" } else { "" })
        }
        Target::Func(f) => func_name(f),
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lines: Vec<String> = self.stmts.iter().map(ToString::to_string).collect();
        f.write_str(&lines.join("\n"))
    }
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Bind {
                target: t, value, ..
            } => write!(f, "(:= {} {value})", target(t)),
            Stmt::Guard { cond, result, .. } => write!(f, "(? {cond} {result})"),
            Stmt::Expr(e) => write!(f, "{e}"),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ExprKind::Num(n) => f.write_str(&number(n)),
            ExprKind::Strand(items) => {
                let parts: Vec<String> = items.iter().map(ToString::to_string).collect();
                write!(f, "(strand {})", parts.join(" "))
            }
            ExprKind::Var(v) => f.write_str(&target(&Target::Var(v.clone()))),
            ExprKind::Arg(side) => f.write_str(if *side == Side::Left { "_l" } else { "_r" }),
            ExprKind::Str(s) => write!(f, "{s:?}"),
            ExprKind::Unit => f.write_str("@"),
            ExprKind::Pow { base, exp } => write!(f, "(pow {base} {})", number(exp)),
            ExprKind::Quote(fun) => write!(f, "(quote {fun})"),
            ExprKind::Monadic { f: fun, arg } => write!(f, "({fun} {arg})"),
            ExprKind::Dyadic {
                left,
                f: fun,
                right,
            } => write!(f, "({fun} {left} {right})"),
            ExprKind::Fn(fun) => write!(f, "{fun}"),
        }
    }
}

impl fmt::Display for Fun {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            FunKind::Name(n) => f.write_str(&func_name(n)),
            FunKind::Sym(s) => f.write_str(s.text()),
            FunKind::Arg(side) => f.write_str(if *side == Side::Left { "_l_" } else { "_r_" }),
            FunKind::Apply(e) => write!(f, "(apply {e})"),
            FunKind::Operand { operand, f: fun } => write!(f, "(operand {operand} {fun})"),
            FunKind::Power { f: fun, count } => write!(f, "(power {fun} {count})"),
            FunKind::Train(fs) => {
                let parts: Vec<String> = fs.iter().map(ToString::to_string).collect();
                write!(f, "(train {})", parts.join(" "))
            }
            FunKind::Lambda(lam) => {
                let params: Vec<String> = match &lam.params {
                    Params::Right => vec!["_r".into()],
                    Params::LeftRight => vec!["_l".into(), "_r".into()],
                    Params::Niladic => vec!["@".into()],
                    Params::Named(ps) => ps
                        .iter()
                        .map(|p| format!("{}{}", if p.lazy { "~" } else { "" }, target(&p.name)))
                        .collect(),
                };
                let body: Vec<String> = lam.body.iter().map(ToString::to_string).collect();
                write!(f, "(lambda ({}) {})", params.join(" "), body.join(" "))
            }
        }
    }
}
