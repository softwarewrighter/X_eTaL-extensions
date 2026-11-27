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
    let cut = PLAN.find("pick A R4").unwrap();
    let mut w = World::default();
    run(&mut w, &PLAN[..cut]);
    let r = w.check_work();
    assert!(!r.ok && r.made == (3, 4), "{r:?}");
    assert!(
        r.lines
            .iter()
            .any(|l| l == "net N7: R4.1 not joined to U1.7"),
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
        "reset circuit=blinker\ntool A tweezers\npick A U1 grasp=ends\nplace A U1 at=0,0,-24 rpy=0,0,90\nvise close\nrelease A U1\nlift A mm=60\nwire S5 awg=30 span=U1.4.tip+U1.8.tip enamel=yes",
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
    run(&mut w, PLAN.split("present U1.2+R1.1").next().unwrap());
    // P1 gripping R1.1, away from its rest
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
