//! `identify` (decision C12, step 22): the scope turns onto a part and
//! reads its marking; oracle gives it as known, sensed as the picture
//! would, with a confidence; a hidden side, a poor picture or a misread
//! is a warning, never silently right.

use xetal_ext_cell::command::parse_program;
use xetal_ext_cell::identify::{GOOD, POOR, seen};
use xetal_ext_cell::world::{Outcome, World};

fn run(w: &mut World, program: &str) -> Vec<Outcome> {
    let out = w.apply_all(&parse_program(program).unwrap());
    for o in &out {
        assert!(o.ok, "{o:?}");
    }
    out
}

fn said(o: &Outcome) -> String {
    o.events
        .iter()
        .chain(&o.warnings)
        .cloned()
        .collect::<Vec<_>>()
        .join(" | ")
}

#[test]
fn oracle_and_sensed_read_the_same_when_the_picture_is_good() {
    let mut w = World::default();
    let out = run(
        &mut w,
        "reset circuit=demo\nidentify R1\nidentify R1 mode=sensed\nidentify D1",
    );
    assert_eq!(
        out[1].events[1],
        "R1: yellow violet red gold: 4.7 kilohm, 5% (oracle)"
    );
    assert_eq!(
        out[2].events[1],
        "R1: yellow violet red gold: 4.7 kilohm, 5% (sensed, confidence 1.00)"
    );
    assert_eq!(out[3].events[1], "D1: red body: a red LED (oracle)");
    // the scope ends on the part
    assert_eq!(w.scope.aim.as_deref(), Some("D1"));
}

#[test]
fn a_poor_picture_is_a_warning_never_silently_right() {
    let mut w = World::default();
    // the whole cell in view: the bands get a pixel each
    let out = run(&mut w, "reset circuit=demo\nidentify R1 mode=sensed zoom=1");
    assert!(
        out[1].warnings[0].starts_with("R1: unreadable, confidence 0.00"),
        "{:?}",
        out[1]
    );
    // halfway: read, but maybe wrongly -- and it says so
    let out = run(&mut w, "identify R1 mode=sensed zoom=2.6");
    let w0 = &out[0].warnings[0];
    assert!(
        w0.starts_with("R1: read as ") && w0.contains("it may be wrong"),
        "{w0}"
    );
    assert!(
        !w0.contains("yellow violet red gold"),
        "the misread differs: {w0}"
    );
    assert!(
        out[0].events.iter().all(|e| !e.contains("kilohm")),
        "no sure reading: {:?}",
        out[0]
    );
}

#[test]
fn a_hidden_side_must_be_turned_to_the_scope() {
    let mut w = World::default();
    let out = run(&mut w, include_str!("../../programs/identify.cell"));
    let c1: Vec<String> = out
        .iter()
        .filter(|o| o.command == "identify C1 mode=sensed")
        .map(said)
        .collect();
    assert_eq!(c1.len(), 3);
    assert!(
        c1[0].contains("C1: unreadable -- its marking is on its marked face"),
        "{}",
        c1[0]
    );
    assert!(
        c1[1].contains("C1: read as") && c1[1].contains("it may be wrong"),
        "{}",
        c1[1]
    );
    assert!(
        c1[2].ends_with("C1: \"104\": 100 nF (sensed, confidence 1.00)"),
        "{}",
        c1[2]
    );
    assert!(w.clashes().is_empty(), "{:?}", w.clashes());
}

#[test]
fn glare_off_a_glossy_top_seen_straight_down() {
    let mut w = World::default();
    let out = run(
        &mut w,
        "reset circuit=demo\ntool A tweezers\npick A U1 grasp=ends\nplace A U1 at=0,0,-24 rpy=0,0,90\nvise close\nrelease A U1\nlift A mm=60\nidentify U1 mode=sensed\nidentify U1",
    );
    let o = &out[7];
    assert!(
        o.warnings
            .iter()
            .any(|x| x.contains("glare off its glossy top")),
        "{o:?}"
    );
    // the oracle is not fooled
    assert!(
        out[8].events[1].ends_with("the part number NE555P (oracle)"),
        "{:?}",
        out[8]
    );
}

#[test]
fn the_confidence_model() {
    assert_eq!(seen(8.0, 0.0, 0.0, false).confidence, 1.0);
    assert!(seen(3.0, 0.0, 0.0, false).confidence < GOOD);
    assert!(seen(1.5, 0.0, 0.0, false).confidence < POOR);
    assert!(seen(8.0, 70.0, 0.0, false).confidence < GOOD);
    assert_eq!(seen(8.0, 0.0, 0.0, true).confidence, 0.5);
}

/// `PROG=name cargo test --test identify say_program -- --ignored --nocapture`:
/// a program's events and warnings, a line a command.
#[test]
#[ignore]
fn say_program() {
    let name = std::env::var("PROG").unwrap();
    let text = std::fs::read_to_string(format!(
        "{}/../programs/{name}.cell",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let mut w = World::default();
    for c in parse_program(&text).unwrap() {
        let o = w.apply(&c);
        if std::env::var("CLASHES").is_ok() {
            println!("CLASH {:?}", w.clashes());
        }
        println!(
            "SAY {} ok={} {:?} {}",
            o.command,
            o.ok,
            o.refused.as_ref().or(o.stopped.as_ref()),
            said(&o)
        );
    }
}
