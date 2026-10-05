//! The live demo's keys and programs (the language is xetal-play's).

use std::sync::{Arc, OnceLock};

use xetal_store::{Memory, Store};
use xetal_web::{Action, DEMOS, action, choices, open, seed};

/// One store for every test (the store is global, and tests run at
/// once), with the demos' own libraries seeded, as the page does.
fn store() -> Arc<Memory> {
    static STORE: OnceLock<Arc<Memory>> = OnceLock::new();
    STORE
        .get_or_init(|| {
            let store = Arc::new(Memory::default());
            xetal_store::install(store.clone());
            seed();
            store
        })
        .clone()
}

#[test]
fn run_and_zoom_keys() {
    assert_eq!(action("Enter", true), Some(Action::Run));
    assert_eq!(action("r", true), Some(Action::Run));
    assert_eq!(action(".", true), Some(Action::Zoom));
    assert_eq!(action("Enter", false), None);
    assert_eq!(action("t", true), None, "Ctrl-T is the browser's");
}

#[test]
fn every_demo_checks() {
    store();
    for demo in DEMOS {
        let lines = xetal_play::check(demo.text);
        assert!(
            !lines.iter().any(|l| l.starts_with("error[")),
            "{}: {lines:?}",
            demo.name
        );
    }
    assert_eq!(DEMOS[0].name, "tour.xtl");
    assert_eq!((DEMOS[1].name, DEMOS[1].text), ("(empty)", ""));
}

#[test]
fn open_offers_the_demos_the_libraries_and_the_saved_files() {
    store().put("MyLib.xtl", "l:t_wo := { 2 }").unwrap();
    let list = choices(&["MyLib.xtl".to_string()]);
    let label = |v: &str| {
        list.iter()
            .find(|(_, value, _)| value == v)
            .map(|(g, _, l)| (*g, l.clone()))
    };
    assert_eq!(label("demo:0"), Some(("Demos", "tour.xtl".into())));
    assert_eq!(label("lib:Stats"), Some(("Libraries", "Stats.xtl".into())));
    assert_eq!(
        label("file:MyLib.xtl"),
        Some(("Your files", "MyLib.xtl".into()))
    );
    assert_eq!(open("lib:Stats").unwrap().0, "Stats.xtl");
    assert!(open("lib:Stats").unwrap().1.contains("l:m_ean"));
    assert_eq!(
        open("file:MyLib.xtl").unwrap(),
        ("MyLib.xtl".into(), "l:t_wo := { 2 }".into())
    );
    assert!(open("file:nothing.xtl").is_none());
}

#[test]
fn the_demos_own_libraries_are_among_your_files_and_edits_are_kept() {
    let store = store();
    assert!(store.get("Hello.xtl").unwrap().contains("l:h_ello"));
    assert!(store.get("Greetings.xtl").unwrap().contains("l:g_reet"));
    store
        .put("Greetings.xtl", "l:g_reet := { n -> n }")
        .unwrap();
    seed();
    assert_eq!(
        store.get("Greetings.xtl").unwrap(),
        "l:g_reet := { n -> n }"
    );
    let hello = DEMOS
        .iter()
        .find(|d| d.name == "hello-library.xtl")
        .unwrap();
    assert!(
        xetal_play::run(hello.text, 1)
            .out
            .contains("hello X\u{332}\u{1d49}T\u{1d43}L")
    );
}

/// After the tour and the empty editor, the demos are listed in
/// alphabetical order in the source, so a new one goes in its place.
#[test]
fn the_demo_list_is_kept_in_alphabetical_order() {
    let names: Vec<&str> = DEMOS[2..].iter().map(|d| d.name).collect();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    assert_eq!(
        names, sorted,
        "keep DEMOS (after the first two) alphabetical"
    );
}

/// And Open sorts each group whatever order the lists are in: the
/// demos after the first two, the libraries, your files.
#[test]
fn open_lists_each_group_alphabetically() {
    let files = ["zeta.xtl".to_string(), "Alpha.xtl".to_string()];
    let list = choices(&files);
    let labels = |group: &str| -> Vec<String> {
        list.iter()
            .filter(|(g, ..)| *g == group)
            .map(|(_, _, l)| l.clone())
            .collect()
    };
    let demos = labels("Demos");
    assert_eq!(demos[..2], ["tour.xtl".to_string(), "(empty)".to_string()]);
    for group in [
        demos[2..].to_vec(),
        labels("Libraries"),
        labels("Your files"),
    ] {
        let mut sorted = group.clone();
        sorted.sort_by_key(|l| l.to_lowercase());
        assert_eq!(group, sorted);
    }
}

/// The live demo installs as an app: a manifest with its name, start
/// page and icons (maskable too), linked from the page, and a service
/// worker that the page registers and that keeps the app for offline.
#[test]
fn the_live_demo_is_an_installable_app() {
    let manifest = include_str!("../manifest.webmanifest");
    for field in [
        r#""name": "X_eTaL live""#,
        r#""short_name": "XeTaL""#,
        r#""start_url": "./""#,
        r#""scope": "./""#,
        r#""display": "standalone""#,
        r#""sizes": "192x192""#,
        r#""sizes": "512x512""#,
        r#""purpose": "maskable""#,
    ] {
        assert!(manifest.contains(field), "manifest lacks {field}");
    }
    let page = include_str!("../index.html");
    for part in [
        r#"rel="manifest""#,
        "apple-touch-icon",
        r#"name="theme-color""#,
        "serviceWorker.register(\"./sw.js\")",
    ] {
        assert!(page.contains(part), "index.html lacks {part}");
    }
    let worker = include_str!("../sw.js");
    for part in [
        "addEventListener(\"fetch\"",
        "caches.open",
        "fetch(event.request)",
    ] {
        assert!(worker.contains(part), "sw.js lacks {part}");
    }
}

/// Open's groups, in order: Demos (the top folder), Classics, Libraries,
/// Misc (any other folder), Your files; a classic is listed without its
/// folder.
#[test]
fn open_groups_the_demos_by_folder() {
    let list = choices(&["Mine.xtl".to_string()]);
    let mut groups: Vec<&str> = list.iter().map(|(g, ..)| *g).collect();
    groups.dedup();
    assert_eq!(
        groups,
        ["Demos", "Classics", "Libraries", "Misc", "Your files"]
    );
    let label = |v: &str| {
        list.iter()
            .find(|(_, value, _)| value == v)
            .map(|(g, _, l)| (*g, l.as_str()))
    };
    let duck = DEMOS
        .iter()
        .position(|d| d.name == "classics/duck.xtl")
        .unwrap();
    assert_eq!(
        label(&format!("demo:{duck}")),
        Some(("Classics", "duck.xtl"))
    );
    let leet = DEMOS
        .iter()
        .position(|d| d.name.starts_with("leetcode/"))
        .unwrap();
    assert_eq!(label(&format!("demo:{leet}")).map(|(g, _)| g), Some("Misc"));
}
