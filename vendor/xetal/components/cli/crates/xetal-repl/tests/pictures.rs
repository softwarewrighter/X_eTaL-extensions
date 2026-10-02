//! Pictures in a session: a line's picture is shown once, not again
//! when later lines replay it; a context run (org-babel sessions) shows
//! none of its own. One test, and one store installed for the binary,
//! since the store and the picture counts are process-wide.

use std::sync::{Arc, OnceLock};

use xetal_repl::{Session, continued};
use xetal_store::{Memory, install};

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
fn a_picture_is_shown_once_and_a_context_shows_none() {
    let store = installed();
    let before = store.pictures().len();
    let mut session = Session::new("-e", 1);
    session.feed("p := []S_HOW []G_RID 1 0");
    session.feed("q := 2");
    session.feed("r := []S_HOW []G_RID 0 1");
    session.feed("3");
    let pictures = store.pictures()[before..].to_vec();
    assert_eq!(pictures.len(), 2, "each picture once");
    assert!(pictures[0].contains("x=\"0\" y=\"0\" width=\"24\""));
    assert!(pictures[1].contains("x=\"24\" y=\"0\" width=\"24\""));

    let before = store.pictures().len();
    let context = "a := []S_HOW []G_RID 1 1\nb := 5\n";
    let cell = continued("-e", context, "c := []S_HOW []G_RID b r_eshape 0", 1);
    assert_eq!(cell.err, "");
    let pictures = store.pictures()[before..].to_vec();
    assert_eq!(pictures.len(), 1, "only the block's own picture");
    assert!(pictures[0].contains("width=\"120\""));
}
