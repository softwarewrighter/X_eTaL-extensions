//! Forming and trimming leads (decision C15): a bend round a mandrel
//! keeps the lead's length and turns the rest by its angle; a cut keeps
//! what is before it; a pincer bends a held part's lead, a cutter
//! station trims one presented to it; what may not be done is refused.

use xetal_ext_cell::command::{parse_line, parse_program};
use xetal_ext_cell::kin;
use xetal_ext_cell::leads;
use xetal_ext_cell::world::{Outcome, World};

/// A straight lead 20 mm down from the body.
const STRAIGHT: [[f64; 3]; 2] = [[0.0, 0.0, 0.0], [0.0, 0.0, -20.0]];

fn dir(line: &[[f64; 3]]) -> [f64; 3] {
    let n = line.len();
    let d = kin::sub(line[n - 1], line[n - 2]);
    kin::scale(d, 1.0 / kin::norm(d))
}

#[test]
fn a_bend_keeps_the_length_and_turns_the_rest() {
    let bent = leads::bend(&STRAIGHT, 5.0, 90.0, [1.0, 0.0, 0.0], 1.0, 0.6).unwrap();
    // the arc's chords are a hair shorter than the arc
    assert!(
        (leads::length(&bent) - 20.0).abs() < 0.05,
        "{}",
        leads::length(&bent)
    );
    // the rest points along +x now
    let d = dir(&bent);
    assert!(kin::norm(kin::sub(d, [1.0, 0.0, 0.0])) < 1e-6, "{d:?}");
    // up to the bend, the lead is as it was
    assert!(kin::norm(kin::sub(bent[1], [0.0, 0.0, -5.0])) < 1e-9);
    // the tip: 5 mm down, round a 1 mm radius, then the rest along x
    let tip = *bent.last().unwrap();
    let rest = 20.0 - 5.0 - std::f64::consts::FRAC_PI_2;
    assert!(
        kin::norm(kin::sub(tip, [1.0 + rest, 0.0, -6.0])) < 1e-6,
        "{tip:?}"
    );
    // 45 degrees: halfway round
    let d = dir(&leads::bend(&STRAIGHT, 5.0, 45.0, [0.0, 1.0, 0.0], 0.6, 0.6).unwrap());
    let h = std::f64::consts::FRAC_1_SQRT_2;
    assert!(kin::norm(kin::sub(d, [0.0, h, -h])) < 1e-6, "{d:?}");
}

#[test]
fn what_cannot_be_bent_is_refused() {
    let says = |r: Result<Vec<[f64; 3]>, String>, what: &str| {
        let e = r.unwrap_err();
        assert!(e.contains(what), "{e}");
    };
    says(
        leads::bend(&STRAIGHT, 25.0, 90.0, [1.0, 0.0, 0.0], 1.0, 0.6),
        "the lead is 20 mm long",
    );
    says(
        leads::bend(&STRAIGHT, 5.0, 90.0, [1.0, 0.0, 0.0], 0.3, 0.6),
        "under the 0.6 mm",
    );
    says(
        leads::bend(&STRAIGHT, 5.0, 90.0, [0.0, 0.0, -1.0], 1.0, 0.6),
        "toward is along the lead",
    );
    says(
        leads::bend(&STRAIGHT, 19.5, 90.0, [1.0, 0.0, 0.0], 1.0, 0.6),
        "the bend needs",
    );
}

#[test]
fn a_cut_keeps_what_is_before_it() {
    let (kept, cut) = leads::split(&STRAIGHT, 8.0);
    assert!((leads::length(&kept) - 8.0).abs() < 1e-9);
    assert!((leads::length(&cut) - 12.0).abs() < 1e-9);
}

fn run(w: &mut World, program: &str) -> Vec<Outcome> {
    let out = w.apply_all(&parse_program(program).unwrap());
    for o in &out {
        assert!(o.ok, "{o:?}");
    }
    out
}

/// The DIP in the vise, the resistor's lead 1 at pin 3, P1 holding it.
const HELD: &str = "reset circuit=demo
tool A tweezers
pick A U1 grasp=ends
place A U1 at=0,0,-24 rpy=0,0,90
vise close
release A U1
pick A R1 grasp=body
lift A mm=80
move A to=0,60,60 rpy=180,0,90
move A to=34.81,1.27,-14.05
align A R1.1.tip to=U1.3.tip gap=0.1
pinch P1 on=R1.1 at=6
release A R1
lift A mm=60";

#[test]
fn a_pincer_bends_a_held_parts_lead() {
    let mut w = World::default();
    run(&mut w, HELD);
    let before = w.frame("R1.2.tip").unwrap().p;
    let o = run(&mut w, "bend P2 on=R1.2 at=3 angle=90 toward=0,0,-1");
    assert!(
        o[0].events[0].starts_with("P2 bends R1.2 90 degrees at 3 mm"),
        "{o:?}"
    );
    let r1 = w.part("R1").unwrap();
    assert!(r1.leads.contains_key("2") && r1.reformed.len() == 1);
    // the lead's tip went somewhere else, and points down now
    let after = w.frame("R1.2.tip").unwrap().p;
    assert!(kin::norm(kin::sub(after, before)) > 5.0);
    let k = r1.kind_of().unwrap();
    let lead: Vec<[f64; 3]> =
        xetal_ext_cell::solder::lead_in(&r1.pose(), &k.pin("2").unwrap().lead);
    assert!(dir(&lead)[2] < -0.99, "{:?}", dir(&lead));
    assert!(w.clashes().is_empty(), "{:?}", w.clashes());
    // bent where it was bent twice more: it may crack
    run(&mut w, "bend P2 on=R1.2 at=3 angle=-30 toward=1,0,0");
    let o = run(&mut w, "bend P2 on=R1.2 at=3 angle=20 toward=1,0,0");
    assert!(
        o[0].warnings
            .iter()
            .any(|x| x.contains("bent 3 times at 3 mm: it may crack")),
        "{o:?}"
    );
}

#[test]
fn what_may_not_be_formed_is_refused() {
    let mut w = World::default();
    run(&mut w, "reset circuit=demo");
    // nothing holds it
    let o = w.apply(&parse_line("bend P2 on=R1.2 at=3 angle=90 toward=0,0,-1").unwrap());
    assert!(
        o.refused.as_deref().unwrap().contains("nothing holds R1"),
        "{o:?}"
    );
    let mut w = World::default();
    run(&mut w, HELD);
    // a joined lead
    let o = w.apply(&parse_line("bend P2 on=R1.1 at=3 angle=90 toward=0,0,-1").unwrap());
    assert!(
        o.refused.as_deref().unwrap().contains("is in U1.3+R1.1"),
        "{o:?}"
    );
    // a cut not presented to the cutter
    let o = w.apply(&parse_line("trim S4 on=R1.2 at=8").unwrap());
    assert!(
        o.refused
            .as_deref()
            .unwrap()
            .contains("present it there first (present R1.2.8 at=S4)"),
        "{o:?}"
    );
    // a cut at or before a joint cuts it off
    run(&mut w, "present R1.1.4 at=S4");
    let o = w.apply(&parse_line("trim S4 on=R1.1 at=4").unwrap());
    assert!(
        o.refused.as_deref().unwrap().contains("would cut it off"),
        "{o:?}"
    );
    // the iron cannot cut
    let o = w.apply(&parse_line("trim S1 on=R1.2 at=8").unwrap());
    assert!(
        o.refused.as_deref().unwrap().contains("not the cutter"),
        "{o:?}"
    );
}

#[test]
fn the_leads_program_forms_solders_and_trims() {
    let mut w = World::default();
    let out = run(&mut w, include_str!("../../programs/leads.cell"));
    let joint = w.joints.iter().find(|j| j.name == "U1.3+R1.1").unwrap();
    assert_eq!(joint.result.as_deref(), Some("good"));
    assert!(out.iter().any(|o| {
        o.events
            .iter()
            .any(|e| e.starts_with("S4 cuts R1.2 at 8 mm"))
    }));
    let r1 = w.part("R1").unwrap();
    let k = r1.kind_of().unwrap();
    assert!((leads::length(&k.pin("2").unwrap().lead) - 8.0).abs() < 1e-6);
    assert_eq!(r1.holder.as_deref(), Some("vise"));
    assert!(w.clashes().is_empty(), "{:?}", w.clashes());
    // the state round-trips, re-formed leads and all
    assert_eq!(World::from_json(&w.to_json()).unwrap(), w);
}
