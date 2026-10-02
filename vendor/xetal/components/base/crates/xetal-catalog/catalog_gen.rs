//! Read `builtins.toml`, check it, and write `builtins.rs` into
//! `OUT_DIR`, where `src/table.rs` `include!`s it.

use std::collections::HashSet;
use std::path::Path;

use crate::catalog_emit::render;
use crate::catalog_parse::Catalog;

/// Arrows outside parentheses: a signature's arity.
fn arrows(sig: &str) -> usize {
    let ty = sig.rsplit("=>").next().unwrap_or(sig);
    let (mut depth, mut count) = (0i32, 0);
    for (i, c) in ty.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            '-' if depth == 0 && ty[i..].starts_with("->") => count += 1,
            _ => {}
        }
    }
    count
}

fn check(catalog: &Catalog) {
    let mut seen = HashSet::new();
    let rows = catalog.group.iter().flat_map(|g| &g.entries);
    for (name, arity, sig) in rows {
        assert_eq!(arrows(sig), *arity, "builtins.toml: {name} arity vs {sig}");
        assert!(seen.insert(name.clone()), "builtins.toml: {name} twice");
    }
    for name in catalog.later.iter().flat_map(|l| &l.names) {
        assert!(seen.insert(name.clone()), "builtins.toml: {name} twice");
    }
}

pub fn run() {
    println!("cargo:rerun-if-changed=builtins.toml");
    let text = std::fs::read_to_string("builtins.toml").expect("read builtins.toml");
    let catalog: Catalog = toml::from_str(&text).expect("parse builtins.toml");
    check(&catalog);
    let out = Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR")).join("builtins.rs");
    std::fs::write(out, render(&catalog)).expect("write builtins.rs");
}
