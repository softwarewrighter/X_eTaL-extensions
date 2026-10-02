//! Build-script facade: generates `BUILTINS` from `builtins.toml` into
//! `OUT_DIR` (data lists live in data, not Rust source). The work is in
//! named sibling files:
//!
//! - `catalog_parse` -- the serde types of the TOML file.
//! - `catalog_emit`  -- a parsed catalog -> the `const BUILTINS` literal.
//! - `catalog_gen`   -- read the TOML, check it, write the generated `.rs`.

mod catalog_emit;
mod catalog_gen;
mod catalog_parse;

fn main() {
    catalog_gen::run();
}
