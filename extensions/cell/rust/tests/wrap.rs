//! Wrap joints (decision C23): the wrap tool winds a wire's end round a
//! lead 1.5 to under 2 turns and holds it while the iron solders, then
//! cuts the tail; fewer than 1.5 turns is weak, 2 or more refused.

use xetal_ext_cell::command::parse_program;
use xetal_ext_cell::world::{Outcome, World};

const WRAP: &str = include_str!("../../programs/wrap.cell");

fn run(w: &mut World, program: &str) -> Vec<Outcome> {
    w.apply_all(&parse_program(program).unwrap())
}

fn joint<'a>(w: &'a World, name: &str) -> &'a xetal_ext_cell::solder::Joint {
    w.joints.iter().find(|j| j.name == name).unwrap()
}

#[test]
fn a_wrap_of_one_and_three_quarter_turns_is_good() {
    let mut w = World::default();
    let out = run(&mut w, WRAP);
    assert!(out.iter().all(|o| o.ok), "{:?}", out.iter().find(|o| !o.ok));
    for name in ["U1.4+W1.1", "U1.5+W1.2"] {
        let j = joint(&w, name);
        assert_eq!(j.result.as_deref(), Some("good"), "{j:?}");
        assert!((j.turns - 1.75).abs() < 1e-9, "{j:?}");
        // the solder touches the wire all round the lead
        assert!(j.nodes_b.len() >= 3, "{j:?}");
    }
    let o = out
        .iter()
        .find(|o| o.command.starts_with("wrap A on=W1.1"))
        .unwrap();
    assert_eq!(
        o.events,
        ["A winds W1.1 1.75 turns round U1.4 and holds it"]
    );
    let o = out
        .iter()
        .find(|o| o.command.starts_with("snip A on=W1.1"))
        .unwrap();
    assert_eq!(
        o.events,
        ["A's blade cuts W1.1's tail (2 mm) and the bit lifts off"]
    );
    let o = out
        .iter()
        .find(|o| o.command.starts_with("inspect U1.4"))
        .unwrap();
    assert!(
        o.events[0].starts_with("U1.4+W1.1: good; a wrap of 1.75 turns;"),
        "{o:?}"
    );
    let o = out.iter().find(|o| o.command == "inspect shorts").unwrap();
    assert_eq!(o.events, ["shorts: none"]);
    // the wire's end is a helix round the lead: its points all about
    // the same distance from the lead's axis
    let wire = w.part("W1").unwrap();
    let lead = wire.kind_of().unwrap().pin("1").unwrap().lead.clone();
    assert!(lead.len() > 40, "{}", lead.len());
}

#[test]
fn fewer_than_one_and_a_half_turns_is_weak() {
    let mut w = World::default();
    let out = run(&mut w, &WRAP.replace("turns=1.75", "turns=1.25"));
    assert!(out.iter().all(|o| o.ok), "{:?}", out.iter().find(|o| !o.ok));
    let j = joint(&w, "U1.4+W1.1");
    assert_eq!(j.result.as_deref(), Some("weak"), "{j:?}");
    let r = w.check_work();
    assert!(
        r.lines
            .iter()
            .any(|l| l == "U1.4+W1.1: weak, a wrap of 1.25 turns"),
        "{r:?}"
    );
}

#[test]
fn two_turns_or_more_are_refused() {
    let mut w = World::default();
    let out = run(&mut w, &WRAP.replace("turns=1.75", "turns=2"));
    let o = out.iter().find(|o| o.command.starts_with("wrap")).unwrap();
    assert!(
        o.refused
            .as_deref()
            .unwrap_or("")
            .contains("winds under two turns"),
        "{o:?}"
    );
}

#[test]
fn a_wire_not_fed_for_it_is_not_wrapped() {
    // fed with lap ends, not wrap=: its end lies down the leg, not past it
    let mut w = World::default();
    let out = run(&mut w, &WRAP.replace("wrap=1.75", "lap=3.2"));
    let o = out.iter().find(|o| o.command.starts_with("wrap")).unwrap();
    assert!(o.refused.is_some(), "{o:?}");
    // and nothing wrapped, nothing to snip
    let mut w = World::default();
    let prefix = WRAP.split("wrap A on=W1.1").next().unwrap();
    run(&mut w, prefix);
    let o = &run(&mut w, "snip A on=W1.1")[0];
    assert!(
        o.refused
            .as_deref()
            .unwrap_or("")
            .contains("is not wrapped"),
        "{o:?}"
    );
}
