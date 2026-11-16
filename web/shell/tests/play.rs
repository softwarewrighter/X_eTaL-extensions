//! A game kept running, natively: tests/game/count.xtl (it counts its
//! events and shows a picture an event) started, given events, run to
//! its waits, its output and pictures taken, stopped, and failing.

use std::sync::{Mutex, MutexGuard, PoisonError};

use xetal_ext_shell::play::{Files, Game, State, scene};
use xetal_ext_shell::run::{Linked, install, run};

/// The store is one for the process: tests that install one take turns.
static TURN: Mutex<()> = Mutex::new(());

fn turn() -> MutexGuard<'static, ()> {
    TURN.lock().unwrap_or_else(PoisonError::into_inner)
}

fn files() -> Files {
    Files::new([
        ("count.xtl", include_str!("game/count.xtl")),
        ("Picture.xtl", include_str!("game/Picture.xtl")),
        ("bad.xtl", "1 +"),
        (
            "forever.xtl",
            "u:s_pin! := { n ->\n  u:s_pin! n + 1\n}\nu:s_pin! 0\n",
        ),
        (
            "notes.xtl",
            "e := []E_VENT @\n_ := \"kept\" []N_PUT \"save.txt\"\n_ := p_rint! ([]N_GET \"save.txt\") c_at []N_GET \"data/word.txt\"\n",
        ),
        ("data/word.txt", " word"),
        (
            "shout.xtl",
            "\"hx:\" u_se< \"Hello\"\ne := []E_VENT @\nhx:s_hout []E_KIND e\n",
        ),
    ])
}

fn hello() -> Linked {
    Linked {
        descriptor: xetal_ext_hello::__xetal_extension::descriptor,
        facade: (
            "Hello.xtl",
            include_str!("../../../extensions/hello/lib/Hello.xtl"),
        ),
    }
}

#[test]
fn a_game_starts_and_waits_for_its_first_event() {
    let _turn = turn();
    let mut game = Game::start(&files(), &[], "count.xtl", 1);
    assert_eq!(game.state(), State::Event);
    assert!(!game.over());
    assert_eq!(game.error(), "");
    assert_eq!(game.take_printed(), "ready\n");
    assert_eq!(game.take_printed(), "", "taken once");
    assert_eq!(game.take_picture(), None);
    assert_eq!(game.pictures(), 0);
}

#[test]
fn an_event_runs_the_game_to_its_next_wait_and_shows_a_picture() {
    let _turn = turn();
    let mut game = Game::start(&files(), &[], "count.xtl", 1);
    assert_eq!(game.take_printed(), "ready\n");

    assert_eq!(game.event("tick 0.05"), State::Event);
    assert_eq!(game.take_printed(), "tick 0\n");
    let picture = game.take_picture().expect("a picture an event");
    assert!(picture.starts_with("<svg "), "{picture}");
    assert_eq!(scene(&picture), Some("view(4, 3, '#101010')\ncount(0)"));
    assert_eq!(game.take_picture(), None, "taken once");

    // two events before the page asks: the latest picture, all the text
    assert_eq!(game.event("key space"), State::Event);
    assert_eq!(game.event("click 364 221"), State::Event);
    assert_eq!(game.take_printed(), "key 1\nclick 2\n");
    let picture = game.take_picture().expect("the latest picture");
    assert_eq!(scene(&picture), Some("view(4, 3, '#101010')\ncount(2)"));
    assert_eq!(game.pictures(), 3);
}

#[test]
fn the_end_event_ends_the_game_cleanly() {
    let _turn = turn();
    let mut game = Game::start(&files(), &[], "count.xtl", 1);
    game.event("tick 0.05");
    assert_eq!(game.event("end"), State::Over);
    assert!(game.over());
    assert_eq!(game.error(), "");
    assert_eq!(game.take_printed(), "ready\ntick 0\ncounted 1\n");
    // nothing more happens
    assert_eq!(game.event("tick 0.05"), State::Over);
    assert_eq!(game.take_printed(), "");
    assert_eq!(game.pictures(), 1);
}

#[test]
fn a_game_is_stopped_where_it_waits() {
    let _turn = turn();
    let mut game = Game::start(&files(), &[], "count.xtl", 1);
    game.event("tick 0.05");
    game.stop();
    assert_eq!(game.state(), State::Over);
    assert_eq!(game.error(), "");
    assert_eq!(game.event("tick 0.05"), State::Over);
    assert_eq!(game.take_printed(), "ready\ntick 0\n");
    assert!(game.take_picture().is_some(), "its last picture is kept");
}

#[test]
fn an_error_ends_the_game_and_is_kept() {
    let _turn = turn();
    let mut game = Game::start(&files(), &[], "count.xtl", 1);
    assert_eq!(game.event("up 1 2"), State::Over);
    assert!(
        game.error()
            .starts_with("error[domain]: []E_KEY: a up event has no key"),
        "{}",
        game.error()
    );
    assert_eq!(game.take_printed(), "ready\nup 0\n");

    // a line that is no event is the program's error too
    let mut game = Game::start(&files(), &[], "count.xtl", 1);
    assert_eq!(game.event("down 120"), State::Over);
    assert!(
        game.error().starts_with("error[bad-event]"),
        "{}",
        game.error()
    );
}

#[test]
fn a_program_that_cannot_start_is_a_game_over_with_its_error() {
    let _turn = turn();
    let game = Game::start(&files(), &[], "bad.xtl", 1);
    assert!(game.over());
    assert!(game.error().starts_with("error["), "{}", game.error());

    let game = Game::start(&files(), &[], "nowhere.xtl", 1);
    assert!(game.over());
    assert_eq!(game.error(), "nowhere.xtl: no such file");

    // a library it imports is not among the files
    let alone = Files::new([("count.xtl", include_str!("game/count.xtl"))]);
    let game = Game::start(&alone, &[], "count.xtl", 1);
    assert!(game.over());
    assert!(game.error().contains("Picture"), "{}", game.error());
}

#[test]
fn a_program_that_never_waits_is_stopped_at_the_limit() {
    let _turn = turn();
    let game = Game::start_within(&files(), &[], "forever.xtl", 1, 100_000);
    assert!(game.over());
    assert_eq!(game.error(), "stopped: 100000 steps without waiting");
}

#[test]
fn a_game_reads_its_files_and_what_it_wrote() {
    let _turn = turn();
    let mut game = Game::start(&files(), &[], "notes.xtl", 1);
    assert_eq!(game.state(), State::Event);
    assert_eq!(game.event("tick 0.05"), State::Over);
    assert_eq!(game.error(), "");
    assert_eq!(game.take_printed(), "kept word\n");
}

#[test]
fn a_game_calls_a_linked_extension_and_keeps_its_store() {
    let _turn = turn();
    let mut game = Game::start(&files(), &[hello()], "shout.xtl", 1);
    assert_eq!(game.error(), "");
    assert_eq!(game.state(), State::Event);
    // the page runs something else on another store meanwhile
    install(&[]);
    assert_eq!(run("1 + 1").out, "2\n");
    assert_eq!(game.event("tick 0.05"), State::Over);
    assert_eq!(game.error(), "");
    assert_eq!(game.take_printed(), "TICK\n");
}

#[test]
fn a_picture_without_a_scene_has_none() {
    assert_eq!(scene("<svg><rect/></svg>"), None);
    assert_eq!(scene("<svg><metadata id=\"scene\">view(1, 1, #000)"), None);
    assert_eq!(
        scene("<svg><metadata id=\"scene\"></metadata></svg>"),
        Some("")
    );
}
