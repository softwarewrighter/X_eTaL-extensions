//! The site of the fixtures: which pages there are, and what links
//! where.

use std::path::PathBuf;

use xetal_doc::{DocFile, model};
use xetal_docsite::{site, write};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn app() -> Vec<DocFile> {
    let path = fixtures().join("app.xtl");
    let text = std::fs::read_to_string(&path).expect("fixture");
    model(&path.to_string_lossy(), &text).expect("model")
}

fn page<'a>(pages: &'a [(String, String)], name: &str) -> &'a str {
    let found = pages.iter().find(|(p, _)| p == name);
    found
        .map(|(_, html)| html.as_str())
        .unwrap_or_else(|| panic!("{name}"))
}

#[test]
fn an_index_a_page_and_a_source_page_per_file_and_the_builtins() {
    let files = app();
    let mut names: Vec<String> = site(&files).into_iter().map(|(p, _)| p).collect();
    names.sort();
    let (main, lib) = (
        xetal_dochtml::page(&files[0].name),
        xetal_dochtml::page(&files[1].name),
    );
    let mut want = vec![
        format!("{main}.html"),
        format!("{main}.src.html"),
        "builtins.html".to_string(),
        format!("{lib}.html"),
        format!("{lib}.src.html"),
        format!("{lib}m.html"),
        format!("{lib}m.src.html"),
        "index.html".to_string(),
        "style.css".to_string(),
        "theme.js".to_string(),
    ];
    want.sort();
    assert_eq!(names, want);
}

#[test]
fn items_link_to_what_they_use_and_back() {
    let files = app();
    let pages = site(&files);
    let (main, lib) = (
        xetal_dochtml::page(&files[0].name),
        xetal_dochtml::page(&files[1].name),
    );
    let app = page(&pages, &format!("{main}.html"));
    assert!(app.contains("id=\"u.g_reetAll\""), "{app}");
    assert!(
        app.contains(&format!("href=\"{lib}.html#l.s_hout\"")),
        "{app}"
    );
    let greet = page(&pages, &format!("{lib}.html"));
    assert!(
        greet.contains(&format!("href=\"{main}.html#u.g_reetAll\"")),
        "used by: {greet}"
    );
    assert!(greet.contains("href=\"builtins.html#c_at\""), "{greet}");
}

#[test]
fn sections_are_headings_with_a_table_of_contents() {
    let files = app();
    let pages = site(&files);
    let greet = page(
        &pages,
        &format!("{}.html", xetal_dochtml::page(&files[1].name)),
    );
    assert!(
        greet.contains("<h2 class=\"section\" id=\"section-1\">Saying hello</h2>"),
        "{greet}"
    );
    assert!(
        greet.contains("<a href=\"#section-2\">Values</a>"),
        "{greet}"
    );
}

#[test]
fn docs_examples_and_source_lines_are_drawn() {
    let files = app();
    let pages = site(&files);
    let greet = page(
        &pages,
        &format!("{}.html", xetal_dochtml::page(&files[1].name)),
    );
    assert!(greet.contains("<pre class=\"example\">"), "{greet}");
    assert!(greet.contains("<p>The greeting for a name.</p>"), "{greet}");
    let main = xetal_dochtml::page(&files[0].name);
    let source = page(&pages, &format!("{main}.src.html"));
    assert!(source.contains("id=\"L10\""), "{source}");
}

#[test]
fn the_site_is_written_into_a_directory() {
    let dir = std::env::temp_dir().join(format!("xetal-docsite-{}-write", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let written = write(&dir, &app()).expect("written");
    assert_eq!(written.len(), 10);
    assert!(dir.join("index.html").is_file());
    std::fs::remove_dir_all(&dir).expect("cleaned");
}
