//! The cell's state: deterministic, round-trips through JSON, refuses
//! what it cannot do and changes nothing then; dry runs change nothing.

use xetal_ext_cell::command::{parse_line, parse_program};
use xetal_ext_cell::world::{World, outcome_json};

#[test]
fn a_new_cell() {
    let w = World::default();
    assert_eq!(w.t, 0.0);
    let names: Vec<&str> = w.arms.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(names, ["A", "B", "C", "D"]);
    // on a ring of 450 mm, each base facing the center
    for a in &w.arms {
        assert!(((a.base[0].powi(2) + a.base[1].powi(2)).sqrt() - 450.0).abs() < 1e-9);
        let yaw = a.base[5].to_radians();
        let facing = [yaw.cos(), yaw.sin()];
        let inward = [-a.base[0] / 450.0, -a.base[1] / 450.0];
        assert!(
            (facing[0] - inward[0]).abs() < 1e-9 && (facing[1] - inward[1]).abs() < 1e-9,
            "{}",
            a.name
        );
    }
}

#[test]
fn state_round_trips_through_json() {
    let mut w = World::default();
    w.apply_all(&parse_program("wait s=1.25\nwait s=2").unwrap());
    let j = w.to_json();
    assert_eq!(World::from_json(&j).unwrap(), w);
    assert_eq!(World::from_json(&j).unwrap().to_json(), j);
    assert!(
        World::from_json("{}")
            .unwrap_err()
            .starts_with("not a cell's state")
    );
    let three = j.replacen("\"name\": \"D\"", "\"name\": \"E\"", 1);
    assert!(
        World::from_json(&three)
            .unwrap_err()
            .contains("the arms are A, B, C, D")
    );
}

#[test]
fn time_passes_and_the_timeline_records() {
    let mut w = World::default();
    let out = w.apply_all(&parse_program("wait s=1.5 ; wait s=0.5").unwrap());
    assert!(out.iter().all(|o| o.ok));
    assert_eq!(w.t, 2.0);
    assert_eq!(w.timeline.len(), 2);
    assert_eq!((w.timeline[1].t0, w.timeline[1].t1), (1.5, 2.0));
    assert_eq!(w.timeline[1].command, "wait s=0.5");
    assert_eq!(
        outcome_json(&out[1]),
        r#"{"ok":true,"command":"wait s=0.5","t0":1.5,"t1":2.0,"did":["wait 0.5 s"],"events":[],"warnings":[]}"#
    );
}

#[test]
fn a_refused_command_changes_nothing() {
    let mut w = World::default();
    w.apply(&parse_line("wait s=3").unwrap());
    let before = w.clone();
    for line in [
        "move A to=2000,0,10",
        "grip A close",
        "reset circuit=blinker",
        "wait until=U1.3 below=100",
        "wait",
    ] {
        let o = w.apply(&parse_line(line).unwrap());
        assert!(!o.ok && o.refused.is_some(), "{line}");
        assert_eq!((o.t0, o.t1), (3.0, 3.0));
        assert_eq!(w, before, "{line}");
    }
    let o = w.apply(&parse_line("reset circuit=blinker").unwrap());
    assert_eq!(
        o.refused.as_deref(),
        Some("no circuit blinker (circuits: none)")
    );
    // a program stops at the first command that is not ok
    let out = w.apply_all(&parse_program("wait s=1\ngrip A close\nwait s=1").unwrap());
    assert_eq!(out.len(), 2);
    assert_eq!(w.t, 4.0);
}

#[test]
fn a_dry_run_changes_nothing() {
    let mut w = World::default();
    let o = w.apply(&parse_line("wait s=5 dry=1").unwrap());
    assert!(o.ok && o.dry);
    assert_eq!(o.t1, 5.0);
    assert_eq!(o.command, "wait s=5");
    assert_eq!(w, World::default());
}

#[test]
fn reset_starts_over() {
    let mut w = World::default();
    w.apply_all(&parse_program("wait s=9\nreset").unwrap());
    assert_eq!(w.t, 0.0);
    assert_eq!(w.timeline.len(), 1);
    assert_eq!(w.timeline[0].command, "reset");
}

#[test]
fn the_same_program_gives_the_same_state() {
    let p = parse_program("wait s=0.1\nwait s=0.2\nlift B\nwait s=3").unwrap();
    let run = || {
        let mut w = World::default();
        w.apply_all(&p);
        w.to_json()
    };
    assert_eq!(run(), run());
}
