//! Loading programs with standard and local libraries.

use xetal_program::{load, located};

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("xetal-program-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn a_program_without_imports_loads_as_before() {
    let loaded = load("-e", "1 + 2").unwrap();
    assert_eq!(loaded.sources.file_count(), 1);
    assert_eq!(
        loaded.program.to_string(),
        xetal_core::lower("1 + 2").unwrap().to_string()
    );
}

#[test]
fn a_library_beside_the_program_is_found() {
    let dir = scratch("beside");
    std::fs::write(dir.join("Two.xtl"), "l:t_wo := { _r * 2 }\n").unwrap();
    let main = dir.join("main.xtl");
    let loaded = load(
        &main.display().to_string(),
        "\"t:\" u_se< \"Two\"\nt:t_wo 21\n",
    )
    .unwrap();
    assert_eq!(loaded.sources.file_count(), 2);
}

#[test]
fn an_error_in_a_library_is_located_there() {
    let dir = scratch("err");
    std::fs::write(dir.join("Bad.xtl"), "l:f_ := { _r }\n1 + 2\n").unwrap();
    let main = dir.join("main.xtl").display().to_string();
    let err = load(&main, "\"b:\" u_se< \"Bad\"\n").unwrap_err();
    assert_eq!(err.code, "expression-in-library");
    assert!(err.to_string().ends_with("Bad.xtl:2:1"), "{err}");
}

#[test]
fn located_leaves_one_file_alone() {
    let loaded = load("-e", "1").unwrap();
    let d = xetal_base::Diagnostic::new("x", "y").with_span(xetal_base::Span::new(0, 1));
    assert_eq!(
        located(&loaded.sources, d.clone()).to_string(),
        d.to_string()
    );
}

#[test]
fn an_error_in_the_program_keeps_its_place_in_the_program() {
    let src = "\"s:\" u_se< \"Stats\"\ns:m_ean 1 2 3\n1 / y\n";
    let loaded = load("-e", src).unwrap();
    let err = xetal_types::check_program(&mut loaded.program.clone()).unwrap_err();
    let local = xetal_program::in_program(&loaded.sources, err);
    let span = local.span.expect("a span in the program");
    assert_eq!(&src[span.start..span.end], "y");
}

#[test]
fn a_library_may_reuse_a_name_from_a_library_it_imports() {
    let dir = scratch("nested");
    let outer = "\"s:\" u_se< \"Stats\"\nl:m_ean := { 2 * s:m_ean _r }\n";
    std::fs::write(dir.join("Outer.xtl"), outer).unwrap();
    let main = dir.join("main.xtl").display().to_string();
    let mut loaded = load(&main, "\"o:\" u_se< \"Outer\"\no:m_ean 1 2 3\n").unwrap();
    assert_eq!(loaded.sources.file_count(), 3);
    let types = xetal_types::check_program(&mut loaded.program).unwrap();
    assert_eq!(xetal_program::program_types(types), ["Float"]);
}
