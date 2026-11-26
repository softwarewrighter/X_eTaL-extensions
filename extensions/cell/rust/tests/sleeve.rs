//! Heat-shrink sleeving (decision C20, step 24): a sleeve slid over a
//! lead's free end and shrunk insulates it, so crossing leads of other
//! parts do not short.

use xetal_ext_cell::command::{parse_line, parse_program};
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

const PROGRAM: &str = include_str!("../../programs/insulate.cell");

#[test]
fn the_bare_pieces_of_a_lead() {
    let line = vec![[0.0, 0.0, 0.0], [30.0, 0.0, 0.0]];
    let p = xetal_ext_cell::sleeve::bare(&line, &[(15.0, 27.0)]);
    assert_eq!(p.len(), 2);
    assert!((xetal_ext_cell::leads::length(&p[0]) - 15.0).abs() < 1e-9);
    assert!((xetal_ext_cell::leads::length(&p[1]) - 3.0).abs() < 1e-9);
    // sleeved end to end: no bare metal
    assert!(xetal_ext_cell::sleeve::bare(&line, &[(0.0, 30.0)]).is_empty());
}

#[test]
fn a_sleeve_insulates_a_crossing_that_would_short() {
    let mut w = World::default();
    let out = run(&mut w, PROGRAM);
    let last = said(out.last().unwrap());
    assert_eq!(
        last,
        "shorts: none; insulated: R1.1 crosses U1.1, R1.1 crosses U1.2"
    );
    let j = w.joints.iter().find(|j| j.name == "U1.3+R1.1").unwrap();
    assert_eq!(j.result.as_deref(), Some("good"), "{j:?}");
    assert!(w.clashes().is_empty(), "{:?}", w.clashes());
    let s = &w.part("R1").unwrap().sleeves[0];
    assert!(s.shrunk.is_some() && s.color == "red" && (s.from, s.to) == (15.0, 27.0));
    assert_eq!(World::from_json(&w.to_json()).unwrap(), w);
    // the same without the sleeve: two shorts
    let bare: String = PROGRAM
        .lines()
        .filter(|l| !l.starts_with("sleeve ") && !l.starts_with("shrink "))
        .collect::<Vec<_>>()
        .join("\n");
    let mut w = World::default();
    let out = run(&mut w, &bare);
    assert!(
        said(out.last().unwrap()).starts_with("shorts: U1.1 touches R1.1, U1.2 touches R1.1"),
        "{}",
        said(out.last().unwrap())
    );
    assert!(
        w.clashes()
            .iter()
            .any(|c| c == "a short: U1.1 touches R1.1"),
        "{:?}",
        w.clashes()
    );
}

/// R1 in the vise, its lead 1's free end at S6.
const AT_S6: &str = "reset circuit=demo
tool A tweezers
pick A R1 grasp=body
place A R1 at=0,0,-20
vise close
release A R1
lift A mm=60
present R1.1.tip at=S6";

fn refused(w: &mut World, line: &str, says: &str) {
    let o = w.apply(&parse_line(line).unwrap());
    let why = o.refused.clone().or(o.stopped.clone()).unwrap_or_default();
    assert!(!o.ok && why.contains(says), "{line}: {o:?}");
}

#[test]
fn what_a_sleeve_cannot_do() {
    let mut w = World::default();
    run(&mut w, AT_S6);
    // too small to pass over the lead
    refused(
        &mut w,
        "sleeve S6 on=R1.1 from=15 to=27 size=0.6",
        "does not pass over",
    );
    // past the lead's end
    refused(&mut w, "sleeve S6 on=R1.1 from=15 to=40", "28 mm long");
    // the other lead's end is not at the station
    refused(
        &mut w,
        "sleeve S6 on=R1.2 from=15 to=27",
        "present R1.2.tip there first",
    );
    // nothing to shrink yet
    refused(&mut w, "shrink S6 on=R1.1", "no sleeve to shrink");
    // a sleeve too big for the lead: on, but loose -- a warning
    let o = &run(&mut w, "sleeve S6 on=R1.1 from=20 to=27 size=2.0")[0];
    assert!(o.warnings.iter().any(|x| x.contains("loose")), "{o:?}");
    // a second sleeve cannot pass the first
    refused(&mut w, "sleeve S6 on=R1.1 from=10 to=22", "in the way");
    // a short blast: partly shrunk
    run(&mut w, "present R1.1.23 at=S6");
    let o = &run(&mut w, "shrink S6 on=R1.1 dwell=1")[0];
    assert!(
        o.warnings.iter().any(|x| x.contains("shrank only partly")),
        "{o:?}"
    );
    // only the sleeve station sleeves; it does not feed wire
    refused(&mut w, "sleeve S1 on=R1.1 from=1 to=5", "");
    refused(&mut w, "wire S6 awg=26 mm=40", "");
}

#[test]
fn a_sleeve_goes_on_before_the_lead_is_bent_or_joined() {
    // bent first: the bend is in the way
    let mut w = World::default();
    run(
        &mut w,
        "reset circuit=demo\ntool A tweezers\npick A R1 grasp=body\nlift A mm=60\ninsert A on=R1.1 at=3\nform angle=90\nlift A mm=30\nmove A to=0,40,60 rpy=180,0,90\nplace A R1 at=0,0,-20\nvise close\nrelease A R1\nlift A mm=60",
    );
    let _ = w.apply(&parse_line("present R1.1.tip at=S6").unwrap());
    refused(&mut w, "sleeve S6 on=R1.1 from=1 to=10", "bent at 3 mm");
}
