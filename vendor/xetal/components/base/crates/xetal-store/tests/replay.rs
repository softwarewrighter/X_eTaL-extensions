//! A session replays its accepted source for each new line: pictures
//! shown before are skipped on the replay, and muted ones (a context
//! run silently) are counted but never shown. One test, since the
//! counts are process-wide.

use std::sync::Arc;

use xetal_store::{Memory, install, muted, replay, show, shown};

#[test]
fn replays_skip_pictures_already_shown_and_muted_ones_are_counted_only() {
    let store = Arc::new(Memory::default());
    install(store.clone());
    replay(0);
    show("<svg>1</svg>").unwrap();
    assert_eq!(shown(), 1);
    // The next line replays the first, then shows its own picture.
    replay(1);
    show("<svg>1</svg>").unwrap();
    show("<svg>2</svg>").unwrap();
    assert_eq!(shown(), 2);
    assert_eq!(store.pictures(), ["<svg>1</svg>", "<svg>2</svg>"]);
    // Muted: counted, so a later replay skips it, but never shown.
    replay(2);
    muted(true);
    show("<svg>context</svg>").unwrap();
    muted(false);
    assert_eq!(shown(), 1);
    assert_eq!(store.pictures().len(), 2);
}
