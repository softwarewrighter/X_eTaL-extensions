//! Every name in a file's source linked to what it names: an item of
//! the same file, an export of a library imported under its alias, a
//! system macro, a built-in; parameters and locals left alone.

use std::path::PathBuf;

use xetal_doc::{DocFile, model};
use xetal_doclink::{Resolver, Target, links, uses};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn app() -> Vec<DocFile> {
    let path = fixtures().join("app.xtl");
    let text = std::fs::read_to_string(&path).expect("fixture");
    model(&path.to_string_lossy(), &text).expect("model")
}

/// Each link in file `file` as (the text linked, what it names).
fn linked(files: &[DocFile], file: usize, text: &str) -> Vec<(String, String)> {
    let r = Resolver::new(files);
    let shown = |t: Target| match t {
        Target::Item { file, item } => {
            format!("{}#{}", files[file].kind, files[file].items[item].name)
        }
        Target::File(f) => files[f].kind.to_string(),
        Target::Builtin(b) => format!("builtin {b}"),
    };
    links(&r, file, text)
        .into_iter()
        .map(|l| (text[l.span.start..l.span.end].to_string(), shown(l.target)))
        .collect()
}

fn pairs(want: &[(&str, &str)]) -> Vec<(String, String)> {
    want.iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
}

#[test]
fn names_link_to_their_items_through_the_alias() {
    let files = app();
    let got = linked(
        &files,
        0,
        "u:g_reetAll := { n -> g:s_hout n }\nu:g_reetAll names\n",
    );
    let want = [
        ("g:s_hout", "library#l:s_hout"),
        ("u:g_reetAll", "program#u:g_reetAll"),
        ("names", "program#names"),
    ];
    assert_eq!(got, pairs(&want));
}

#[test]
fn macros_imports_and_builtins_are_linked_too() {
    let files = app();
    let got = linked(
        &files,
        0,
        "\"g:\" u_se< \"Greet\"\n\"1 = 1\" g:w_hen< \"p_rint! 1\"\n",
    );
    let want = [
        ("\"Greet\"", "library"),
        ("g:w_hen<", "macro library#m:w_hen<"),
    ];
    assert_eq!(got, pairs(&want));
    let got = linked(&files, 1, "l:s_hout := { n -> (l:h_ello n) c_at mark }");
    let want = [
        ("l:h_ello", "library#l:h_ello"),
        ("c_at", "builtin c_at"),
        ("mark", "library#mark"),
    ];
    assert_eq!(got, pairs(&want));
}

#[test]
fn a_parameter_or_local_hides_an_item_of_the_same_name() {
    let files = app();
    let got = linked(
        &files,
        0,
        "f_ := { names -> names }\ng_ := { x -> names := 1; names }\n",
    );
    assert!(got.is_empty(), "{got:?}");
}

#[test]
fn a_system_macro_links_into_system_xtlm() {
    let files = model("-e", "x := \"1 > 0\" i_f< \"1; 2\"").expect("model");
    let got = linked(&files, 0, "\"1 > 0\" i_f< \"1; 2\"");
    assert_eq!(got, pairs(&[("i_f<", "system macros#s:i_f<")]));
}

#[test]
fn uses_list_each_items_places_across_files() {
    let files = app();
    let r = Resolver::new(&files);
    let all = uses(&r);
    let hello = files[1]
        .items
        .iter()
        .position(|i| i.name == "l:h_ello")
        .expect("hello");
    let at = &all[&Target::Item {
        file: 1,
        item: hello,
    }];
    let shout_line = files[1]
        .items
        .iter()
        .find(|i| i.name == "l:s_hout")
        .expect("shout")
        .line;
    assert_eq!(at, &[(1, shout_line)]);
    let greet_all = Target::Item { file: 0, item: 0 };
    let lines: Vec<usize> = all[&greet_all].iter().map(|(_, l)| *l).collect();
    assert_eq!(lines, [12]);
}
