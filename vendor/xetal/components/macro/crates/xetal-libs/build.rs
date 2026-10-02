//! Embeds the repo's `lib/*.xtl` as `LIBRARIES` (name, text), so an
//! installed xetal has the standard libraries without the repo.

use std::fmt::Write;
use std::path::Path;

fn main() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../lib");
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|x| x == "xtl"))
                .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    let mut out = String::from(
        "/// The standard libraries: (name, text).\npub const LIBRARIES: &[(&str, &str)] = &[\n",
    );
    for name in &names {
        let path = dir.join(format!("{name}.xtl"));
        println!("cargo:rerun-if-changed={}", path.display());
        let _ = writeln!(
            out,
            "    ({name:?}, include_str!({:?})),",
            path.canonicalize().unwrap_or(path).display().to_string()
        );
    }
    out.push_str("];\n");
    let target = Path::new(&std::env::var("OUT_DIR").unwrap_or_default()).join("libraries.rs");
    std::fs::write(target, out).unwrap_or_else(|e| panic!("cannot write libraries.rs: {e}"));
}
