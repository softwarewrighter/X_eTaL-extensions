//! A page's whole host code, natively: a `Page` and `game_page!`, whose
//! exports are called as `static/game.js` calls them.

use xetal_ext_shell::page::Page;

const PAGE: Page = Page {
    program: "count.xtl",
    files: &[
        ("count.xtl", include_str!("game/count.xtl")),
        ("Picture.xtl", include_str!("game/Picture.xtl")),
        ("notes.txt", "a < b & c"),
    ],
    linked: &[],
};

xetal_ext_shell::game_page!(PAGE);

// One test: the exports share the page's one game.
#[test]
fn the_exports_play_the_game() {
    assert!(over(), "no game yet");
    assert!(event("tick 0.05"), "an event before a start does nothing");
    assert_eq!(picture(), "");

    start(1);
    assert!(!over());
    assert_eq!(printed(), "ready\n");
    assert_eq!(picture(), "");
    assert!(!event("tick 0.05"));
    assert!(
        picture().contains("<metadata id=\"scene\">view(4, 3, '#101010')\ncount(0)</metadata>")
    );
    assert_eq!(picture(), "", "taken once");
    assert_eq!(printed(), "tick 0\n");

    // the sources may be read while the game waits
    assert_eq!(files(), "count.xtl\nPicture.xtl\nnotes.txt");
    let html = rendered("count.xtl");
    assert!(html.contains("<span class=\"t-builtin\">"), "{html}");
    assert_eq!(rendered("notes.txt"), "a &lt; b &amp; c");
    assert_eq!(rendered("nowhere.xtl"), "");

    assert!(!event("key space"));
    assert_eq!(printed(), "key 1\n");
    assert!(event("up 1 2"), "over on its error");
    assert!(over());
    assert!(error().starts_with("error[domain]"), "{}", error());

    // again from the beginning
    start(1);
    assert!(!over());
    assert_eq!(error(), "");
    assert_eq!(printed(), "ready\n");
    stop();
    assert!(over());
    assert_eq!(error(), "");
}
