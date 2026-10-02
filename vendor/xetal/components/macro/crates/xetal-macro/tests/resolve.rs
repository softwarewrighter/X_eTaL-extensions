//! Finding libraries on disk: paths, the importing file's directory,
//! then the search path.

use std::path::PathBuf;

use xetal_macro::{FsLibraries, Libraries};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("xetal-resolve-{}-{name}", std::process::id()));
    std::fs::create_dir_all(dir.join("sub")).unwrap();
    std::fs::create_dir_all(dir.join("path")).unwrap();
    dir
}

#[test]
fn a_name_is_found_beside_the_importer_first() {
    let dir = scratch("beside");
    std::fs::write(dir.join("Stats.xtl"), "l:a := 1\n").unwrap();
    std::fs::write(dir.join("path/Stats.xtl"), "l:a := 2\n").unwrap();
    let libs = FsLibraries::new(vec![dir.join("path")]);
    let found = libs
        .find("Stats", &dir.join("main.xtl").display().to_string())
        .unwrap();
    assert_eq!(found.text, "l:a := 1\n");
}

#[test]
fn then_on_the_search_path() {
    let dir = scratch("path");
    std::fs::write(dir.join("path/Geo.xtl"), "l:a := 2\n").unwrap();
    let libs = FsLibraries::new(vec![dir.join("path")]);
    let found = libs
        .find("Geo", &dir.join("main.xtl").display().to_string())
        .unwrap();
    assert_eq!(found.text, "l:a := 2\n");
    assert!(
        libs.find("Nope", &dir.join("main.xtl").display().to_string())
            .is_none()
    );
}

#[test]
fn a_path_is_relative_to_the_importing_file() {
    let dir = scratch("rel");
    std::fs::write(dir.join("sub/helpers.xtl"), "l:h := 3\n").unwrap();
    let libs = FsLibraries::new(Vec::new());
    let found = libs
        .find(
            "sub/helpers.xtl",
            &dir.join("main.xtl").display().to_string(),
        )
        .unwrap();
    assert_eq!(found.text, "l:h := 3\n");
    let again = libs
        .find(
            "./sub/../sub/helpers.xtl",
            &dir.join("main.xtl").display().to_string(),
        )
        .unwrap();
    assert_eq!(found.key, again.key, "one library, one key");
}

#[test]
fn a_name_matches_exactly_even_on_a_case_insensitive_disk() {
    let dir = scratch("case");
    std::fs::write(dir.join("stats.xtl"), "\"s:\" u_se< \"Stats\"\n").unwrap();
    let libs = FsLibraries::new(Vec::new());
    let found = libs.find("Stats", &dir.join("stats.xtl").display().to_string());
    let me = dir
        .join("stats.xtl")
        .canonicalize()
        .unwrap()
        .display()
        .to_string();
    assert_ne!(found.map(|f| f.key), Some(me), "found the importer itself");
}

/// Libraries of your own live in userlibs/ (in the current directory),
/// searched after the importing file's directory and before the
/// directories of XETAL_PATH.
#[test]
fn the_search_path_is_userlibs_then_xetal_path() {
    let dirs = |var| -> Vec<String> {
        FsLibraries::from_path_var(var)
            .search()
            .iter()
            .map(|d| d.display().to_string())
            .collect()
    };
    assert_eq!(dirs(None), ["userlibs"]);
    assert_eq!(dirs(Some("a:b")), ["userlibs", "a", "b"]);
    assert_eq!(dirs(Some("")), ["userlibs"]);
}
