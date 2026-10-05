//! A library and its macro library found together (MC11): the first
//! directory holding either gives both it holds; explicit paths give
//! one file; the store and the standard libraries likewise.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use xetal_lookup::{FsLibraries, Libraries, StoreLibraries};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("xetal-pairs-{}-{name}", std::process::id()));
    std::fs::create_dir_all(dir.join("path")).unwrap();
    dir
}

fn texts(pair: &xetal_lookup::Pair) -> (Option<&str>, Option<&str>) {
    (
        pair.0.as_ref().map(|f| f.text.as_str()),
        pair.1.as_ref().map(|f| f.text.as_str()),
    )
}

#[test]
fn both_files_of_one_directory_are_found_together() {
    let dir = scratch("both");
    std::fs::write(dir.join("Control.xtl"), "l:a := 1\n").unwrap();
    std::fs::write(dir.join("Control.xtlm"), "m:x_< := 1\n").unwrap();
    let libs = FsLibraries::new(vec![dir.join("path")]);
    let pair = libs.find_both("Control", &dir.join("main.xtl").display().to_string());
    assert_eq!(texts(&pair), (Some("l:a := 1\n"), Some("m:x_< := 1\n")));
}

#[test]
fn the_first_directory_holding_either_wins() {
    let dir = scratch("first");
    std::fs::write(dir.join("Control.xtlm"), "m:x_< := 1\n").unwrap();
    std::fs::write(dir.join("path/Control.xtl"), "l:a := 2\n").unwrap();
    let libs = FsLibraries::new(vec![dir.join("path")]);
    let from = dir.join("main.xtl").display().to_string();
    assert_eq!(
        texts(&libs.find_both("Control", &from)),
        (None, Some("m:x_< := 1\n"))
    );
    assert!(libs.find("Control", &from).is_none());
}

#[test]
fn a_macro_library_alone_is_found_on_the_search_path() {
    let dir = scratch("path");
    std::fs::write(dir.join("path/Only.xtlm"), "m:x_< := 1\n").unwrap();
    let libs = FsLibraries::new(vec![dir.join("path")]);
    let pair = libs.find_both("Only", &dir.join("main.xtl").display().to_string());
    assert_eq!(texts(&pair), (None, Some("m:x_< := 1\n")));
}

#[test]
fn an_explicit_path_gives_that_file_only() {
    let dir = scratch("explicit");
    std::fs::write(dir.join("C.xtl"), "l:a := 1\n").unwrap();
    std::fs::write(dir.join("C.xtlm"), "m:x_< := 1\n").unwrap();
    let libs = FsLibraries::new(Vec::new());
    let from = dir.join("main.xtl").display().to_string();
    assert_eq!(
        texts(&libs.find_both("C.xtl", &from)),
        (Some("l:a := 1\n"), None)
    );
    assert_eq!(
        texts(&libs.find_both("C.xtlm", &from)),
        (None, Some("m:x_< := 1\n"))
    );
}

#[test]
fn the_standard_libraries_come_last() {
    let libs = FsLibraries::new(Vec::new());
    let pair = libs.find_both("Stats", "/nowhere/main.xtl");
    assert!(pair.0.is_some());
    let none = libs.find_both("Nope", "/nowhere/main.xtl");
    assert!(none.0.is_none() && none.1.is_none());
}

fn store() -> &'static xetal_store::Memory {
    static STORE: OnceLock<Arc<xetal_store::Memory>> = OnceLock::new();
    STORE.get_or_init(|| {
        let memory = Arc::new(xetal_store::Memory::default());
        xetal_store::install(memory.clone());
        memory
    })
}

#[test]
fn the_store_holds_both_kinds() {
    store();
    xetal_store::write("Store1.xtlm", "m:x_< := 1\n").unwrap();
    let pair = StoreLibraries.find_both("Store1", "main.xtl");
    assert_eq!(texts(&pair), (None, Some("m:x_< := 1\n")));
    let pair = StoreLibraries.find_both("Store1.xtlm", "main.xtl");
    assert_eq!(texts(&pair), (None, Some("m:x_< := 1\n")));
}
