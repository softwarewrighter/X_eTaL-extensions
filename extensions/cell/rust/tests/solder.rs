//! Soldering: a joint wets, melts and freezes into what the way it was
//! made deserves -- good, cold, starved, blob, disturbed -- a good one
//! holds its two parts together as one subassembly, carried as one;
//! a joint heated again melts again; heat damages a body over its limit.

use xetal_ext_cell::command::{parse_line, parse_program};
use xetal_ext_cell::kin;
use xetal_ext_cell::world::{Outcome, World};

/// At the bench: the DIP in the vise legs up, a resistor's lead bent
/// and laid alongside pin 3 (a lap joint, decision C22) and gripped
/// there by pincer P1, the arm gone, the joint presented to the iron
/// station S1 (which feeds its own solder).
const READY: &str = "reset circuit=lap
tool A tweezers
pick A U1 grasp=ends
place A U1 at=0,0,-24 rpy=180,0,90
vise close
release A U1
pick A R1 grasp=body
lift A mm=60
insert A on=R1.1 at=12
form angle=90
lift A mm=30
shear A on=R1.1 at=17.2
lift A mm=60
bring A R1.1 to=U1.3.tip along=-0.934,0.356,0 lap=3.2
pinch P1 on=R1.1 at=3
release A R1
lift A mm=60
present U1.3+R1.1 at=S1";

fn run(w: &mut World, program: &str) -> Vec<Outcome> {
    let out = w.apply_all(&parse_program(program).unwrap());
    for o in &out {
        assert!(o.ok, "{o:?}");
    }
    out
}

fn ready() -> World {
    let mut w = World::default();
    run(&mut w, READY);
    w
}

fn result(w: &World) -> Option<String> {
    w.joints
        .iter()
        .find(|j| j.name == "U1.3+R1.1")
        .and_then(|j| j.result.clone())
}

#[test]
fn a_good_joint_holds_two_parts_as_one() {
    let mut w = ready();
    run(
        &mut w,
        "touch S1 at=U1.3+R1.1 dwell=2\nfeed S1 mm=2 rate=1\nback S1\nwait until=U1.3+R1.1 below=100",
    );
    assert_eq!(result(&w).as_deref(), Some("good"));
    let j = &w.joints[0];
    assert!(j.wet_a >= 1.0 && j.wet_b >= 1.0 && j.solder > 0.3, "{j:?}");
    // the pincer lets go, the resistor stays: soldered to the DIP the vise holds
    let o = w.apply(&parse_line("unpinch P1").unwrap());
    assert!(o.events.iter().any(|e| e.contains("R1 stays")), "{o:?}");
    assert_eq!(w.part("R1").unwrap().holder.as_deref(), Some("vise"));
    // picked out of the vise by the resistor, the pair goes up together
    run(&mut w, "lift A mm=40\npick A R1 grasp=body\nvise open");
    let gap = |w: &World| {
        kin::sub(
            w.part("R1").unwrap().pose().p,
            w.part("U1").unwrap().pose().p,
        )
    };
    let before = gap(&w);
    run(&mut w, "lift A mm=80");
    assert!(kin::norm(kin::sub(gap(&w), before)) < 1e-6);
    assert_eq!(w.part("U1").unwrap().holder.as_deref(), Some("A"));
    assert!(!w.part("R1").unwrap().lost && !w.part("U1").unwrap().lost);
}

#[test]
fn too_little_heat_makes_a_cold_joint() {
    // solder fed fast and the iron taken away at once: not wet a second
    let mut w = ready();
    run(
        &mut w,
        "touch S1 at=U1.3+R1.1 dwell=2\nfeed S1 mm=0.5 rate=5\nback S1\nwait until=U1.3+R1.1 below=100",
    );
    assert_eq!(result(&w).as_deref(), Some("cold"));
}

#[test]
fn too_little_or_too_much_solder() {
    let mut w = ready();
    run(
        &mut w,
        "touch S1 at=U1.3+R1.1 dwell=2\nfeed S1 mm=0.8 rate=0.4\nback S1\nwait until=U1.3+R1.1 below=100",
    );
    assert_eq!(result(&w).as_deref(), Some("starved"));
    let mut w = ready();
    run(
        &mut w,
        "touch S1 at=U1.3+R1.1 dwell=2\nfeed S1 mm=10 rate=2\nback S1\nwait until=U1.3+R1.1 below=100",
    );
    assert_eq!(result(&w).as_deref(), Some("blob"));
}

#[test]
fn moving_a_side_while_molten_disturbs_it() {
    let mut w = ready();
    let out = run(
        &mut w,
        "touch S1 at=U1.3+R1.1 dwell=2\nfeed S1 mm=2 rate=1\npick A R1 grasp=body\nback S1\nwait until=U1.3+R1.1 below=100",
    );
    assert!(
        out.iter().any(|o| o
            .warnings
            .iter()
            .any(|x| x.contains("molten while arm A moved"))),
        "{out:?}"
    );
    assert_eq!(result(&w).as_deref(), Some("disturbed"));
}

#[test]
fn heated_again_it_melts_again() {
    let mut w = ready();
    run(
        &mut w,
        "touch S1 at=U1.3+R1.1 dwell=2\nfeed S1 mm=2 rate=1\nback S1\nwait until=U1.3+R1.1 below=100",
    );
    assert_eq!(result(&w).as_deref(), Some("good"));
    let out = run(&mut w, "touch S1 at=U1.3+R1.1 dwell=2");
    assert!(
        out[0]
            .warnings
            .iter()
            .any(|x| x == "U1.3+R1.1 melted again"),
        "{out:?}"
    );
    assert!(w.joints[0].molten && result(&w).is_none());
    run(&mut w, "back S1\nwait until=U1.3+R1.1 below=100");
    assert_eq!(result(&w).as_deref(), Some("good"));
}

#[test]
fn solder_on_a_cold_joint_does_not_melt() {
    let mut w = ready();
    // no iron on any joint: refused
    let o = w.apply(&parse_line("feed S1 mm=1").unwrap());
    assert!(
        o.refused
            .unwrap_or_default()
            .contains("no iron is on a joint")
    );
    // leads too far apart to be a joint
    let o = w.apply(&parse_line("touch S1 at=U1.1+R1.2").unwrap());
    assert!(o.refused.unwrap_or_default().contains("align them first"));
}

#[test]
fn a_body_over_its_limit_too_long_is_damaged() {
    // the DIP's body at 300 C: 260 C is its limit for 10 s
    let mut w = World::default();
    run(&mut w, "reset circuit=demo");
    let k = w.parts.iter().position(|p| p.id == "U1").unwrap();
    let n = 1 + 8 * 5;
    w.parts[k].temps = vec![300.0; n];
    let out = run(&mut w, "wait s=12");
    assert!(
        out[0]
            .warnings
            .iter()
            .any(|x| x.starts_with("U1 is damaged")),
        "{out:?}"
    );
    assert!(w.part("U1").unwrap().damaged);
    let o = w.apply(&parse_line("inspect U1.3").unwrap());
    assert!(o.events[0].contains("the part is damaged"), "{o:?}");
}

#[test]
fn heat_is_recorded_for_players() {
    let mut w = ready();
    run(
        &mut w,
        "touch S1 at=U1.3+R1.1 dwell=2\nfeed S1 mm=2 rate=1\nback S1\nwait until=U1.3+R1.1 below=100",
    );
    let u1 = w.heat.iter().find(|h| h.id == "U1").unwrap();
    // rows: the time, the body, eight pins
    assert!(u1.rows.iter().all(|r| r.len() == 10));
    // pin 3 the hottest, near the iron's 350 C
    // the time, the body, then pins 1 to 8: pin 3 is column 4
    let hottest = u1.rows.iter().map(|r| r[4]).fold(0.0, f64::max);
    assert!(hottest > 280.0 && hottest < 350.0, "{hottest}");
    assert!(!w.joints[0].track.is_empty());
}

/// The lap joint program, with `extra` after the feed.
fn solder_with(extra: &str) -> xetal_ext_cell::world::World {
    let prog = include_str!("../../programs/lap.cell").replace(
        "feed S1 mm=2 rate=1\n",
        &format!("feed S1 mm=2 rate=1\n{extra}"),
    );
    let mut w = xetal_ext_cell::world::World::default();
    let out = w.apply_all(&xetal_ext_cell::command::parse_program(&prog).unwrap());
    assert!(out.iter().all(|o| o.ok), "{:?}", out.iter().find(|o| !o.ok));
    w
}

#[test]
fn the_flux_comes_with_the_solder_and_burns_off() {
    // fed and let go: good, the flux still active as it froze
    let w = solder_with("");
    let j = w.joints.iter().find(|j| j.name == "U1.3+R1.1").unwrap();
    assert_eq!(j.result.as_deref(), Some("good"), "{j:?}");
    // the iron held on 10 s past the feed: the flux spent, the joint dull
    let w = solder_with("wait s=10\n");
    let j = w.joints.iter().find(|j| j.name == "U1.3+R1.1").unwrap();
    assert_eq!(j.result.as_deref(), Some("dull"), "{j:?}");
    assert!(j.dry > 4.0, "{}", j.dry);
}
