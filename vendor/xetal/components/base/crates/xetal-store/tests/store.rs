//! The file store: the disk by default, another one when installed.

use std::sync::{Arc, OnceLock};

use xetal_store::{Memory, Store, install, read, write};

/// One installed store for every test that needs one: the store is
/// global and tests run at once, so two tests installing their own
/// would race (each would see the other's).
fn installed() -> Arc<Memory> {
    static STORE: OnceLock<Arc<Memory>> = OnceLock::new();
    STORE
        .get_or_init(|| {
            let store = Arc::new(Memory::default());
            install(store.clone());
            store
        })
        .clone()
}

#[test]
fn a_memory_store_keeps_what_is_written() {
    let store = Memory::default();
    store.put("work/m.txt", "1 2 3").unwrap();
    assert_eq!(store.get("work/m.txt").unwrap(), "1 2 3");
    assert!(store.get("nothing.txt").is_err());
    assert_eq!(store.paths(), ["work/m.txt"]);
}

#[test]
fn the_installed_store_serves_reads_and_writes() {
    let store = installed();
    write("work/a.txt", "hello").unwrap();
    assert_eq!(read("work/a.txt").unwrap(), "hello");
    assert_eq!(store.get("work/a.txt").unwrap(), "hello");
    assert!(read("missing.txt").unwrap_err().contains("missing.txt"));
}

#[test]
fn a_memory_store_answers_reads_from_the_keyboard_in_order() {
    let store = Memory::default();
    store.push_line("5");
    store.push_line("x");
    assert_eq!(store.line().unwrap(), "5");
    assert_eq!(store.line().unwrap(), "x");
    assert!(store.line().unwrap_err().contains("no more input"));
}

#[test]
fn a_memory_store_keeps_the_pictures_shown_in_order() {
    let store = Memory::default();
    store.show("<svg>1</svg>").unwrap();
    store.show("<svg>2</svg>").unwrap();
    assert_eq!(store.pictures(), ["<svg>1</svg>", "<svg>2</svg>"]);
}

#[test]
fn the_installed_store_is_where_pictures_are_shown() {
    let store = installed();
    xetal_store::show("<svg>installed</svg>").unwrap();
    assert!(
        store
            .pictures()
            .contains(&"<svg>installed</svg>".to_string())
    );
}

#[test]
fn a_store_that_cannot_show_pictures_says_so() {
    let err = xetal_store::Disk.show("<svg/>").unwrap_err();
    assert!(err.contains("show pictures"), "{err}");
}

#[test]
fn a_drawing_store_writes_each_picture_as_a_numbered_file_and_reports_it() {
    use std::sync::Mutex;
    static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());
    let dir = std::env::temp_dir().join(format!("xetal-drawing-{}", std::process::id()));
    let notify = |p: &std::path::Path| SEEN.lock().unwrap().push(p.display().to_string());
    let store = xetal_store::Drawing::new(&dir, "life", notify);
    store.show("<svg>1</svg>").unwrap();
    store.show("<svg>2</svg>").unwrap();
    let (one, two) = (dir.join("life-1.svg"), dir.join("life-2.svg"));
    assert_eq!(std::fs::read_to_string(&one).unwrap(), "<svg>1</svg>");
    assert_eq!(std::fs::read_to_string(&two).unwrap(), "<svg>2</svg>");
    let seen = SEEN.lock().unwrap().clone();
    assert_eq!(seen, [one.display().to_string(), two.display().to_string()]);
    store
        .put(&dir.join("t.txt").display().to_string(), "x")
        .unwrap();
    assert_eq!(
        store.get(&dir.join("t.txt").display().to_string()).unwrap(),
        "x"
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

/// A memory store keeps the pictures shown until they are taken (the
/// live demo's engine takes a run's pictures when it ends); a store
/// that keeps none gives none.
#[test]
fn the_pictures_shown_are_kept_until_taken() {
    let store = Memory::default();
    store.show("<svg>a</svg>").unwrap();
    store.show("<svg>b</svg>").unwrap();
    assert_eq!(store.take_shown(), ["<svg>a</svg>", "<svg>b</svg>"]);
    assert!(store.take_shown().is_empty());
    assert!(xetal_store::Disk.take_shown().is_empty());
}
