//! `check` (step 27): the work's connections against its circuit's
//! netlist; and `stage to=`, an absolute heading.

use xetal_ext_cell::command::parse_program;
use xetal_ext_cell::world::{Outcome, World};

fn run(w: &mut World, program: &str) -> Vec<Outcome> {
    w.apply_all(&parse_program(program).unwrap())
}

const PLAN: &str = include_str!("../../programs/plan.cell");

#[test]
fn the_planned_work_matches_its_netlist() {
    let mut w = World::default();
    let out = run(&mut w, PLAN);
    assert!(out.iter().all(|o| o.ok), "{:?}", out.iter().find(|o| !o.ok));
    let r = w.check_work();
    assert!(r.ok && r.made == (4, 4), "{r:?}");
}

#[test]
fn a_missing_joint_or_a_bad_one_is_found() {
    // the last resistor never soldered: its net is not made
    let cut = PLAN.find("pick A R2").unwrap();
    let mut w = World::default();
    run(&mut w, &PLAN[..cut]);
    let r = w.check_work();
    assert!(!r.ok && r.made == (3, 4), "{r:?}");
    assert!(
        r.lines
            .iter()
            .any(|l| l == "net N3: R2.1 not joined to U1.3"),
        "{:?}",
        r.lines
    );
    // too little heat on the first joint: a cold joint, its net not made
    let cold = PLAN.replacen(
        "touch S1 at=U1.2+R1.1 dwell=2",
        "touch S1 at=U1.2+R1.1 dwell=2 temp=230",
        1,
    );
    let mut w = World::default();
    run(&mut w, &cold);
    let r = w.check_work();
    assert!(!r.ok, "{r:?}");
    assert!(
        r.lines
            .iter()
            .any(|l| l == "net N2: R1.1 not joined to U1.2"),
        "{:?}",
        r.lines
    );
}

#[test]
fn the_stage_turns_to_a_heading() {
    let mut w = World::default();
    run(&mut w, "reset circuit=demo\nstage turn=30\nstage turn=15");
    assert!((w.stage - 45.0).abs() < 1e-9);
    run(&mut w, "stage to=0");
    assert!(w.stage.abs() < 1e-9, "{}", w.stage);
    run(&mut w, "stage to=-90");
    assert!((w.stage + 90.0).abs() < 1e-9);
    let o = &run(&mut w, "stage turn=10 to=20")[0];
    assert!(o.refused.as_deref().unwrap().contains("not both"), "{o:?}");
}

#[test]
fn a_jumper_is_cut_to_its_span() {
    let mut w = World::default();
    let out = run(
        &mut w,
        "reset circuit=blinker\ntool A tweezers\npick A U1 grasp=ends\nplace A U1 at=0,0,-24 rpy=180,0,90\nvise close\nrelease A U1\nlift A mm=60\nwire S5 awg=30 span=U1.4.tip+U1.8.tip enamel=yes",
    );
    assert!(out.iter().all(|o| o.ok), "{:?}", out.iter().find(|o| !o.ok));
    // 7.62 mm square: 10.776 across, less 0.1 mm at each end
    let w1 = w.part("W1").unwrap();
    let len = xetal_ext_cell::leads::length(&w1.kind_of().unwrap().pin("1").unwrap().lead) * 2.0;
    assert!((len - 10.576).abs() < 0.01, "{len}");
    // neither mm= nor span=: refused
    let o = &run(&mut w, "cut S5\nwire S5 awg=30")[1];
    assert!(
        o.refused.as_deref().unwrap().contains("give mm= or span="),
        "{o:?}"
    );
}

#[test]
fn a_pincer_parks_at_its_rest() {
    let mut w = World::default();
    run(&mut w, PLAN.split("present U1.6+R3.1").next().unwrap());
    // P1 gripping R3.1, away from its rest
    assert!(w.pincers[0].on.is_some());
    let out = run(
        &mut w,
        "unpinch P1
park P1",
    );
    assert!(out.iter().all(|o| o.ok), "{out:?}");
    let p = &w.pincers[0];
    assert!(
        (xetal_ext_cell::kin::wrap(p.phi + 90.0)).abs() < 1e-6 && p.reach == 0.0,
        "{p:?}"
    );
}

#[test]
fn a_jumper_is_formed_to_its_route_and_laid_on_it() {
    let mut w = World::default();
    let out = run(
        &mut w,
        "reset circuit=demo\ntool A tweezers\npick A U1 grasp=ends\nplace A U1 at=0,0,-24 rpy=0,0,90\nvise close\nrelease A U1\nlift A mm=60\n\
         wire S5 awg=30 span=U1.1.tip+U1.8.tip via=6,-8,-19 via2=-6,-8,-19 enamel=yes\npick A W1 grasp=middle\ncut S5\nlift A mm=60\nlay A W1",
    );
    assert!(out.iter().all(|o| o.ok), "{:?}", out.iter().find(|o| !o.ok));
    assert!(
        out[7]
            .events
            .iter()
            .any(|e| e.starts_with("formed to its route: 2 bends")),
        "{:?}",
        out[7]
    );
    let last = out.last().unwrap();
    assert_eq!(
        last.events,
        [
            "W1.1.tip is 0.1 mm from U1.1.tip",
            "W1.2.tip is 0.1 mm from U1.8.tip"
        ]
    );
    // both joints made, the wire up and over the chip's end, not under it
    assert!(
        w.joints.iter().any(|j| j.name == "U1.1+W1.1")
            && w.joints.iter().any(|j| j.name == "U1.8+W1.2")
    );
    assert!(w.clashes().is_empty(), "{:?}", w.clashes());
    // a wire not formed to a route cannot be laid
    let out = run(
        &mut w,
        "pinch P1 on=W1.2 at=2\nrelease A W1\nlift A mm=60\nwire S5 awg=30 mm=20\npick A W2 grasp=middle\ncut S5\nlift A mm=60\nlay A W2",
    );
    let o = out.last().unwrap();
    assert!(
        o.refused
            .as_deref()
            .unwrap_or("")
            .contains("is not formed to a route"),
        "{out:?}"
    );
}
