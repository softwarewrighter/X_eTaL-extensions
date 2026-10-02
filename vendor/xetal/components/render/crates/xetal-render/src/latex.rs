//! Raw ASCII -> LaTeX math, one way, for post-processing (KaTeX,
//! MathJax, pdflatex). Every token is braced so TeX adds no operator
//! spacing; source spacing is explicit (`\ `, `\\` per newline). An
//! exponent is left unbraced so it is attached to the token it touches,
//! and axes subscript a function's whole name.

use xetal_base::Diagnostic;
use xetal_lex::{FuncName, Side, Symbol, TokenKind, lex};

/// Render lexable raw source as the body of a LaTeX math environment.
pub fn latex(src: &str) -> Result<String, Diagnostic> {
    let tokens = lex(src)?;
    let mut out = String::new();
    let mut pos = 0;
    for token in &tokens {
        let raw = &src[token.span.start..token.span.end];
        // Space before a line end (or a comment, which is dropped) sets
        // nothing, so it is not written.
        if token.kind != TokenKind::Newline {
            out.push_str(&spacing(&src[pos..token.span.start]));
        }
        if token.kind == TokenKind::Newline {
            out.push_str("\\\\\n");
        } else if let TokenKind::Exp(_) = token.kind {
            // Unbraced, so it is the exponent of the token it touches.
            out.push_str(&token_tex(&token.kind, raw));
        } else {
            out.push('{');
            out.push_str(&token_tex(&token.kind, raw));
            out.push('}');
        }
        pos = token.span.end;
    }
    Ok(out)
}

/// Whitespace becomes explicit spaces; comment text is dropped.
fn spacing(gap: &str) -> String {
    gap.split('#')
        .next()
        .unwrap_or("")
        .chars()
        .filter(|c| *c != '\r')
        .map(|_| "\\ ")
        .collect()
}

fn token_tex(kind: &TokenKind, raw: &str) -> String {
    match kind {
        TokenKind::Var(v) => {
            let ns = ns_tex(&v.ns);
            let bang = if v.mutable { "!" } else { "" };
            format!(r"{ns}\mathrm{{{}}}{bang}", v.name)
        }
        TokenKind::Func(name) => func_tex(name),
        TokenKind::LamArg { side, applied } => {
            let s = if *side == Side::Left { "l" } else { "r" };
            let a = if *applied { r"\_" } else { "" };
            format!(r"\_\mathrm{{{s}}}{a}")
        }
        TokenKind::Exp(_) => format!("^{{{}}}", &raw[1..]),
        TokenKind::Sym(sym) => symbol_tex(*sym).into(),
        TokenKind::Str(_) => format!(r"\text{{{}}}", text_tex(raw)),
        TokenKind::Assign => r"\leftarrow".into(),
        TokenKind::Semi => r"\diamond".into(),
        TokenKind::Arrow => r"\to".into(),
        TokenKind::Lazy => r"\sim".into(),
        TokenKind::Quote => r"\text{'}".into(),
        TokenKind::Apply => r"\_".into(),
        TokenKind::LBrace => r"\{".into(),
        TokenKind::RBrace => r"\}".into(),
        _ => raw.into(),
    }
}

fn func_tex(name: &FuncName) -> String {
    let mut out = ns_tex(&name.ns);
    let (before, rest) = name.stem.split_at(name.underline);
    let (letter, after) = rest.split_at(1);
    out.push_str(&format!(
        r"\mathrm{{{before}\underline{{{letter}}}{after}}}"
    ));
    if let Some(mark) = name.mark {
        let tex = MARKS.iter().find(|(m, _)| *m == mark).map(|(_, t)| *t);
        out.push_str(&format!("{{{}}}", tex.unwrap_or(&mark.to_string())));
    }
    if name.axes.is_empty() {
        return out;
    }
    // The whole name carries the axes, never a bare mark.
    let digits: String = name.axes.iter().map(u8::to_string).collect();
    format!("{{{out}}}_{{{digits}}}")
}

/// A function name's trailing mark, braced by the caller so TeX sets it
/// as an ordinary symbol; a mark not listed is written as it is.
const MARKS: &[(char, &str)] = &[
    ('\\', r"\backslash"),
    ('|', r"\mid"),
    ('~', r"\sim"),
    ('%', r"\%"),
    ('$', r"\$"),
    ('&', r"\&"),
    ('*', r"\ast"),
];

/// A string as text, spelled as in the source: TeX's specials escaped,
/// and a letter with a combining underline as `\underline`.
fn text_tex(raw: &str) -> String {
    let mut out = String::new();
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        let tex = match c {
            '\\' => r"\textbackslash{}".to_string(),
            '~' => r"\textasciitilde{}".to_string(),
            '^' => r"\textasciicircum{}".to_string(),
            '#' | '_' | '$' | '%' | '&' | '{' | '}' => format!("\\{c}"),
            c if chars.peek() == Some(&'\u{332}') => {
                chars.next();
                format!(r"\underline{{{c}}}")
            }
            c => c.to_string(),
        };
        out.push_str(&tex);
    }
    out
}

/// A namespace as a leading superscript.
fn ns_tex(ns: &Option<String>) -> String {
    ns.as_ref().map_or(String::new(), |ns| match ns.as_str() {
        xetal_lex::SYSTEM => r"\square ".to_string(),
        ns => format!(r"{{}}^{{\mathrm{{{ns}}}}}"),
    })
}

fn symbol_tex(sym: Symbol) -> &'static str {
    match sym {
        Symbol::Times => r"\times",
        Symbol::Divide => r"\div",
        Symbol::Or => r"\vee",
        Symbol::And => r"\wedge",
        Symbol::Ne => r"\neq",
        Symbol::Le => r"\leq",
        Symbol::Ge => r"\geq",
        Symbol::Power => r"\mathbin{\hat{}}",
        other => other.text(),
    }
}
