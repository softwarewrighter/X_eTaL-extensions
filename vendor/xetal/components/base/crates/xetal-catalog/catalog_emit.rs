//! A parsed catalog as Rust source.

use crate::catalog_parse::Catalog;

fn row(name: &str, arity: usize, sig: &str, rule: &str, implemented: bool) -> String {
    format!(
        "    Builtin {{ name: {name:?}, arity: {arity}, sig: {sig:?}, rule: {rule:?}, implemented: {implemented} }},\n"
    )
}

/// `pub const BUILTINS: &[Builtin] = &[ ... ];`
pub fn render(catalog: &Catalog) -> String {
    let mut out = String::from("pub const BUILTINS: &[Builtin] = &[\n");
    for g in &catalog.group {
        for (name, arity, sig) in &g.entries {
            out += &row(name, *arity, sig, &g.rule, true);
        }
    }
    for l in &catalog.later {
        for name in &l.names {
            out += &row(name, 0, "", &l.rule, false);
        }
    }
    out + "];\n"
}
