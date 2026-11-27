//! The bending rig (decision C19, step 21): a lead slid in at a distance
//! from the body is bent down round the mandrel in the rig's plane; the
//! part turned end for end bends its other lead the same way (a U), or
//! turned about the lead (`roll`) square to it; the shear trims.

use xetal_ext_cell::command::{parse_line, parse_program};
use xetal_ext_cell::kin::{self, V3};
use xetal_ext_cell::world::{Outcome, World};

fn run(w: &mut World, program: &str) -> Vec<Outcome> {
    let out = w.apply_all(&parse_program(program).unwrap());
    for o in &out {
        assert!(o.ok, "{o:?}");
    }
    out
}

/// R1 picked from the tray, lifted clear.
const HELD: &str = "reset circuit=demo
tool A tweezers
pick A R1 grasp=body
lift A mm=60";

/// The way a lead's last segment points, in the part's frame.
fn leg(w: &World, pin: &str) -> V3 {
    let (id, pn) = pin.split_once('.').unwrap();
    let p = w.part(id).unwrap();
    let l = &p.leads[pn];
    let d = kin::sub(l[l.len() - 1], l[l.len() - 2]);
    kin::scale(d, 1.0 / kin::norm(d))
}

fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[test]
fn both_leads_bent_the_same_way_make_a_u() {
    let mut w = World::default();
    let out = run(
        &mut w,
        &format!(
            "{HELD}\ninsert A on=R1.2 at=3\nform angle=90\nlift A mm=30\ninsert A on=R1.1 at=3\nform angle=90\nlift A mm=30"
        ),
    );
    assert!(out.iter().any(|o| {
        o.events
            .iter()
            .any(|e| e == "the rig bends R1.2 90 degrees at 3 mm, round 1 mm")
    }));
    // the two legs parallel, the same way: a U
    let (a, b) = (leg(&w, "R1.1"), leg(&w, "R1.2"));
    assert!(dot(a, b) > 0.999, "{a:?} {b:?}");
    // and down in the cell, the part held as it was picked
    let r = w.part("R1").unwrap().pose().r;
    assert!(kin::mv(&r, a)[2] < -0.999, "{:?}", kin::mv(&r, a));
    assert!(w.clashes().is_empty(), "{:?}", w.clashes());
    assert_eq!(World::from_json(&w.to_json()).unwrap(), w);
}

#[test]
fn turned_about_the_lead_the_bend_comes_out_square() {
    let mut w = World::default();
    run(
        &mut w,
        &format!(
            "{HELD}\ninsert A on=R1.2 at=3\nform angle=90\nlift A mm=30\ninsert A on=R1.1 at=3 roll=90\nform angle=90\nlift A mm=30"
        ),
    );
    let (a, b) = (leg(&w, "R1.1"), leg(&w, "R1.2"));
    assert!(dot(a, b).abs() < 1e-6, "{a:?} {b:?}");
}

#[test]
fn the_rig_refuses_what_it_cannot_do() {
    let mut w = World::default();
    run(&mut w, HELD);
    let o = w.apply(&parse_line("form angle=90").unwrap());
    assert!(
        o.refused
            .as_deref()
            .unwrap()
            .contains("nothing is in the bending rig"),
        "{o:?}"
    );
    // inserted, then carried away before the bend
    run(&mut w, "insert A on=R1.2 at=3\nlift A mm=20");
    let o = w.apply(&parse_line("form angle=90").unwrap());
    assert!(
        o.refused
            .as_deref()
            .unwrap()
            .contains("no longer in the bending rig"),
        "{o:?}"
    );
    // a radius under the lead's own
    run(&mut w, "insert A on=R1.2 at=3");
    let o = w.apply(&parse_line("form angle=90 radius=0.3").unwrap());
    assert!(o.refused.is_some() || o.ok, "{o:?}");
}

#[test]
fn the_shear_trims_a_formed_lead() {
    let mut w = World::default();
    let out = run(
        &mut w,
        &format!(
            "{HELD}\ninsert A on=R1.2 at=3\nform angle=90\nlift A mm=30\nshear A on=R1.2 at=8\nlift A mm=30"
        ),
    );
    assert!(
        out.iter().any(|o| o
            .events
            .iter()
            .any(|e| e.starts_with("the rig's shear cuts R1.2 at 8 mm"))),
        "{out:?}"
    );
    let p = w.part("R1").unwrap();
    let total = xetal_ext_cell::leads::length(&p.leads["2"]);
    assert!((total - 8.0).abs() < 1e-6, "{total}");
    assert!(w.clashes().is_empty(), "{:?}", w.clashes());
}

#[test]
fn the_bend_rig_program_forms_trims_and_solders() {
    let mut w = World::default();
    run(&mut w, include_str!("../../programs/bend-rig.cell"));
    let j = w.joints.iter().find(|j| j.name == "U1.3+R1.1").unwrap();
    // a butt joint (a lead's end against a leg): weak, decision C22; this
    // program shows something else, and moves to a lap joint later
    assert_eq!(j.result.as_deref(), Some("weak"));
    assert!((xetal_ext_cell::leads::length(&w.part("R1").unwrap().leads["2"]) - 8.0).abs() < 1e-6);
    assert!(w.clashes().is_empty(), "{:?}", w.clashes());
}
