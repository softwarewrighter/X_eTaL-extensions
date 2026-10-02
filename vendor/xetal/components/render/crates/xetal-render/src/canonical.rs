//! The canonical form (`xetal fmt`): raw ASCII, one statement per line,
//! every application in parentheses and canonical spacing. It reparses
//! to the same tree: `parse(fmt(parse x)) == parse x`.

use xetal_base::Diagnostic;
use xetal_lex::{FuncName, Number, Side, Var};
use xetal_syntax::{Expr, ExprKind, Fun, FunKind, Lambda, Params, Stmt, Target, parse};

/// Format lexable, parsable source canonically.
pub fn canonical(src: &str) -> Result<String, Diagnostic> {
    let program = parse(src)?;
    let lines: Vec<String> = program.stmts.iter().map(stmt).collect();
    Ok(lines.join("\n"))
}

fn stmt(s: &Stmt) -> String {
    match s {
        Stmt::Bind { target, value, .. } => {
            let name = match target {
                Target::Var(v) => var_text(v),
                Target::Func(f) => func_text(f),
            };
            format!("{name} := {}", expr(value, true))
        }
        Stmt::Guard { cond, result, .. } => {
            format!("{} ? {}", expr(cond, true), expr(result, true))
        }
        Stmt::Expr(e) => expr(e, true),
    }
}

/// `wrap` puts an application in parentheses (off only where the caller
/// supplies them, as in `(expr)_`).
fn expr(e: &Expr, wrap: bool) -> String {
    let group = |text: String| if wrap { format!("({text})") } else { text };
    match &e.kind {
        ExprKind::Num(Number::Int(i)) => i.to_string(),
        ExprKind::Num(Number::Float(x)) => format!("{x:?}"),
        ExprKind::Strand(items) => items
            .iter()
            .map(|x| expr(x, true))
            .collect::<Vec<_>>()
            .join(" "),
        ExprKind::Var(v) => var_text(v),
        ExprKind::Arg(side) => if *side == Side::Left { "_l" } else { "_r" }.into(),
        ExprKind::Str(s) => {
            let escaped = s
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
                .replace('\t', "\\t");
            format!("\"{escaped}\"")
        }
        ExprKind::Unit => "@".into(),
        ExprKind::Pow { base, exp } => {
            let base_text = match base.kind {
                ExprKind::Num(_)
                | ExprKind::Var(_)
                | ExprKind::Arg(_)
                | ExprKind::Monadic { .. }
                | ExprKind::Dyadic { .. } => expr(base, true),
                _ => format!("({})", expr(base, true)),
            };
            let exp_text = expr(&Expr::new(ExprKind::Num(*exp), base.span), true);
            format!("{base_text}^{exp_text}")
        }
        ExprKind::Quote(f) => format!("'{}", fun(f)),
        ExprKind::Monadic { f, arg } => group(format!("{} {}", fun(f), expr(arg, true))),
        ExprKind::Dyadic { left, f, right } => group(format!(
            "{} {} {}",
            expr(left, true),
            fun(f),
            expr(right, true)
        )),
        ExprKind::Fn(f) => fun(f),
    }
}

fn fun(f: &Fun) -> String {
    match &f.kind {
        FunKind::Name(n) => func_text(n),
        FunKind::Sym(s) => s.text().into(),
        FunKind::Arg(side) => if *side == Side::Left { "_l_" } else { "_r_" }.into(),
        FunKind::Apply(e) => format!("({})_", expr(e, false)),
        FunKind::Operand { operand, f } => format!("'{} {}", fun(operand), fun(f)),
        FunKind::Power { f, count } => format!("{}^{count}", fun(f)),
        FunKind::Lambda(l) => lambda(l),
        FunKind::Train(fs) => {
            format!("[{}]", fs.iter().map(fun).collect::<Vec<_>>().join(" "))
        }
    }
}

fn lambda(l: &Lambda) -> String {
    let params = match &l.params {
        Params::Right | Params::LeftRight => String::new(),
        Params::Niladic => "@ -> ".into(),
        Params::Named(ps) => {
            let names: Vec<String> = ps
                .iter()
                .map(|p| {
                    let name = match &p.name {
                        Target::Var(v) => var_text(v),
                        Target::Func(f) => func_text(f),
                    };
                    format!("{}{name}", if p.lazy { "~" } else { "" })
                })
                .collect();
            format!("{} -> ", names.join(" "))
        }
    };
    let body: Vec<String> = l.body.iter().map(stmt).collect();
    format!("{{ {params}{} }}", body.join("; "))
}

fn func_text(n: &FuncName) -> String {
    let ns = xetal_lex::ns_text(&n.ns);
    let axes: String = n.axes.iter().map(u8::to_string).collect();
    let sub = if axes.is_empty() {
        String::new()
    } else {
        format!("_{axes}")
    };
    format!("{ns}{}{sub}", n.spelled())
}

fn var_text(v: &Var) -> String {
    let ns = xetal_lex::ns_text(&v.ns);
    format!("{ns}{}{}", v.name, if v.mutable { "!" } else { "" })
}
