//! The doc model of the fixtures: files, items, docs, sections and uses.

use std::path::PathBuf;
use xetal_doc::{DocFile, Item, Use, model};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn app() -> Vec<DocFile> {
    let path = fixtures().join("app.xtl");
    let text = std::fs::read_to_string(&path).expect("fixture");
    model(&path.to_string_lossy(), &text).expect("model")
}

fn file<'a>(files: &'a [DocFile], end: &str) -> &'a DocFile {
    files.iter().find(|f| f.name.ends_with(end)).expect(end)
}

fn item<'a>(file: &'a DocFile, name: &str) -> &'a Item {
    file.items.iter().find(|i| i.name == name).expect(name)
}

#[test]
fn the_program_and_its_imports_are_documented() {
    let kinds: Vec<&str> = app().iter().map(|f| f.kind).collect();
    assert_eq!(kinds, ["program", "library", "macro library"]);
}

#[test]
fn the_system_macros_are_documented_when_one_is_called() {
    let files = model("-e", "x := \"1 > 0\" i_f< \"1; 2\"").expect("model");
    assert_eq!(files[1].name, "std/System.xtlm");
    assert!(files[1].items.iter().any(|i| i.name == "s:i_f<"));
    let plain = model("-e", "x := 1").expect("model");
    assert_eq!(plain.len(), 1);
}

#[test]
fn library_items_keep_their_written_names_and_privacy() {
    let files = app();
    let greet = file(&files, "Greet.xtl");
    assert!(item(greet, "l:h_ello").public);
    assert!(!item(greet, "mark").public);
    assert_eq!(item(greet, "l:count").ty, "Int");
    assert_eq!(item(greet, "l:h_ello").kind, "function");
    assert_eq!(item(file(&files, "Greet.xtlm"), "m:w_hen<").kind, "macro");
}

#[test]
fn docs_sections_and_examples_come_from_the_comments() {
    let files = app();
    let greet = file(&files, "Greet.xtl");
    let hello = item(greet, "l:h_ello");
    assert_eq!(hello.section.as_deref(), Some("Saying hello"));
    assert_eq!(hello.examples()[1].output, "Hello, Ann");
    assert_eq!(item(greet, "l:count").section.as_deref(), Some("Values"));
    assert!(
        greet
            .doc
            .as_ref()
            .expect("file doc")
            .text
            .starts_with("Greetings")
    );
}

#[test]
fn a_use_names_the_item_and_file_it_resolves_to() {
    let files = app();
    let greet_all = item(&files[0], "u:g_reetAll");
    let greet = file(&files, "Greet.xtl").name.clone();
    let want = Use {
        written: "g:s_hout".into(),
        file: greet,
        item: "l:s_hout".into(),
    };
    assert_eq!(greet_all.uses, [want]);
}

#[test]
fn a_files_imports_name_the_files_found() {
    let files = app();
    let imports = &files[0].imports;
    assert_eq!(imports.len(), 1);
    let named = (imports[0].alias.as_str(), imports[0].spec.as_str());
    assert_eq!(named, ("g", "Greet"));
    let found: Vec<&str> = imports[0].files.iter().map(String::as_str).collect();
    let want = [
        &*file(&files, "Greet.xtl").name,
        &*file(&files, "Greet.xtlm").name,
    ];
    assert_eq!(found, want);
}

#[test]
fn a_bad_program_is_a_diagnostic_not_a_panic() {
    assert!(model("bad.xtl", "x := ( 1").is_err());
}

#[test]
fn documenting_the_system_macros_lists_them_once() {
    let text = xetal_libs_text();
    let files = model("lib/System.xtlm", &text).expect("model");
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].kind, "system macros");
}

fn xetal_libs_text() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../lib/System.xtlm");
    std::fs::read_to_string(path).expect("System.xtlm")
}
