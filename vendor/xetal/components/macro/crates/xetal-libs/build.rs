//! Embeds the repo's `lib/*.xtl` as `LIBRARIES` and `lib/*.xtlm` as
//! `MACROS` (name, text), so an installed xetal has the standard
//! libraries without the repo.

use std::fmt::Write;
use std::path::Path;

fn main() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../lib");
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut out = table(&dir, "xtl", "LIBRARIES", "The standard libraries");
    out.push_str(&table(
        &dir,
        "xtlm",
        "MACROS",
        "The standard macro libraries",
    ));
    let target = Path::new(&std::env::var("OUT_DIR").unwrap_or_default()).join("libraries.rs");
    std::fs::write(target, out).unwrap_or_else(|e| panic!("cannot write libraries.rs: {e}"));
}

/// The files `dir/*.ext` as a constant `name` of (name, text).
fn table(dir: &Path, ext: &str, name: &str, doc: &str) -> String {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|x| x == ext))
                .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    let mut out = format!("/// {doc}: (name, text).\npub const {name}: &[(&str, &str)] = &[\n");
    for stem in &names {
        let path = dir.join(format!("{stem}.{ext}"));
        println!("cargo:rerun-if-changed={}", path.display());
        let _ = writeln!(
            out,
            "    ({stem:?}, include_str!({:?})),",
            path.canonicalize().unwrap_or(path).display().to_string()
        );
    }
    out.push_str("];\n");
    out
}
