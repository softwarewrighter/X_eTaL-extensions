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
