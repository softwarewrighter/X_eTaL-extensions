//! Grasping and collision: tools changed at each arm's rack; parts
//! picked, carried, placed, aligned and let go; the vise holding a part
//! as the stage turns; unsupported parts falling to the floor, lost; a
//! protective stop where arms meet; refused commands changing nothing;
//! dry runs and rewind.

use xetal_ext_cell::command::{parse_line, parse_program};
use xetal_ext_cell::kin;
use xetal_ext_cell::layout::{self, FLOOR};
use xetal_ext_cell::world::{Outcome, World};

fn run(w: &mut World, program: &str) -> Vec<Outcome> {
    w.apply_all(&parse_program(program).unwrap())
}

fn ok(w: &mut World, program: &str) {
    for o in run(w, program) {
        assert!(o.ok, "{o:?}");
    }
}

fn refused(w: &mut World, line: &str, says: &str) {
    let before = w.clone();
    let o = w.apply(&parse_line(line).unwrap());
    let why = o.refused.clone().unwrap_or_default();
    assert!(!o.ok && why.contains(says), "{line}: {o:?}");
    assert_eq!(*w, before, "{line} changed the cell");
}

fn near(a: [f64; 3], b: [f64; 3], eps: f64) -> bool {
    kin::norm(kin::sub(a, b)) < eps
}

const IN_VISE: &str = "reset circuit=demo
tool A tweezers
pick A U1 grasp=ends
place A U1 at=0,0,-24 rpy=0,0,90
vise close
release A U1
lift A mm=60
move A to=0,220,120
joints A 0 0 0 0 90 0";

#[test]
fn a_reset_fills_the_tray_and_the_racks() {
    let mut w = World::default();
    ok(&mut w, "reset circuit=demo");
    let ids: Vec<&str> = w.parts.iter().map(|p| p.id.as_str()).collect();
    assert_eq!(ids, ["U1", "R1", "D1", "C1"]);
    // every part rests on the tray's plate, 140 mm below the work zone
    for p in &w.parts {
        let k = xetal_ext_cell::parts::kind(&p.kind).unwrap();
        let low = xetal_ext_cell::collide::part(&p.id, &k, &p.pose())
            .iter()
            .map(|s| s.shape.lowest())
            .fold(f64::INFINITY, f64::min);
        assert!((low + 140.0).abs() < 1e-6, "{}: {low}", p.id);
    }
    assert_eq!(w.tools.len(), 7, "one rack of seven");
    assert!(w.tools.iter().all(|t| t.holder.is_none()));
    refused(
        &mut w,
        "reset circuit=blinker",
        "no circuit blinker (circuits: none demo weld tester tester-b plan)",
    );
}

#[test]
fn tools_are_changed_at_the_rack() {
    let mut w = World::default();
    ok(&mut w, "tool A tweezers");
    assert_eq!(w.arm("A").unwrap().tool.as_deref(), Some("tweezers"));
    // the tip is the tool's length out along the flange's z
    let len = layout::tool_length("tweezers");
    assert!((len - 100.0).abs() < 1e-3);
    let f = w.flange("A").unwrap();
    assert!(near(w.tip("A").unwrap().p, f.then(&kin::tz(len)).p, 1e-9));
    ok(&mut w, "tool A iron");
    let tw = w.tools.iter().find(|t| t.id == "A.tweezers").unwrap();
    assert!(tw.holder.is_none() && tw.at == tw.start, "{tw:?}");
    let iron = w.tools.iter().find(|t| t.id == "A.iron").unwrap();
    assert_eq!(iron.holder.as_deref(), Some("A"));
    // the iron's mount rides on the flange
    assert!(near(
        [iron.at[0], iron.at[1], iron.at[2]],
        w.flange("A").unwrap().p,
        1e-3
    ));
    let o = w.apply(&parse_line("tool A iron").unwrap());
    assert!(o.ok && o.events == ["A already holds the iron"] && o.did.is_empty());
}

#[test]
fn a_part_is_picked_carried_and_let_fall() {
    let mut w = World::default();
    ok(
        &mut w,
        "reset circuit=demo\ntool A tweezers\npick A U1 grasp=ends",
    );
    assert_eq!(w.arm("A").unwrap().holding.as_deref(), Some("U1"));
    // it rides with the tip: the same offset after a move
    let off = |w: &World| kin::sub(w.part("U1").unwrap().pose().p, w.tip("A").unwrap().p);
    let before = off(&w);
    ok(&mut w, "move A to=60,120,60");
    assert!(near(off(&w), before, 1e-6));
    let o = w.apply(&parse_line("release A U1").unwrap());
    assert!(
        o.ok && o.warnings.len() == 1 && o.warnings[0].starts_with("U1 fell"),
        "{o:?}"
    );
    let p = w.part("U1").unwrap();
    assert!(p.lost && p.holder.is_none());
    let k = xetal_ext_cell::parts::kind("dip8").unwrap();
    let low = xetal_ext_cell::collide::part("U1", &k, &p.pose())
        .iter()
        .map(|s| s.shape.lowest())
        .fold(f64::INFINITY, f64::min);
    assert!((low - FLOOR).abs() < 1e-6, "{low}");
    // it landed after falling, by gravity
    // the command lasts until it lands
    let m = &p.moves[0];
    assert!((m.t - w.t).abs() < 1e-6 && w.t > 0.0, "{} {}", m.t, w.t);
    refused(&mut w, "pick A U1 grasp=ends", "U1 is lost");
}

#[test]
fn the_vise_holds_a_part_as_the_stage_turns() {
    let mut w = World::default();
    ok(&mut w, IN_VISE);
    assert_eq!(w.part("U1").unwrap().holder.as_deref(), Some("vise"));
    let at = w.part("U1").unwrap().pose();
    ok(&mut w, "stage turn=90");
    let now = w.part("U1").unwrap().pose();
    // turned a quarter about the stage's axis, the vertical through 0 0
    let want = kin::Pose::from_xyz_rpy([0.0; 3], [0.0, 0.0, 90.0]).then(&at);
    assert!(near(now.p, want.p, 1e-6) && kin::angle_between(&now.r, &want.r) < 1e-6);
    let o = w.apply(&parse_line("vise open").unwrap());
    assert!(
        o.events.contains(&"the vise lets U1 go".to_string()),
        "{o:?}"
    );
    // it lies on the vise's bed, riding the turntable still
    let u1 = w.part("U1").unwrap();
    assert!(
        !u1.lost && u1.on_stage && u1.holder.as_deref() == Some("vise"),
        "{o:?}"
    );
}

#[test]
fn an_arm_that_meets_the_vise_stops_where_it_touches() {
    let mut w = World::default();
    ok(&mut w, "move A to=0,0,40 rpy=180,0,0");
    let o = w.apply(&parse_line("move A to=0,0,-60 rpy=180,0,0").unwrap());
    let why = o.stopped.clone().unwrap_or_default();
    assert!(
        !o.ok && why.starts_with("collision: arm A") && why.contains("the vise"),
        "{o:?}"
    );
    // stopped short, the motion up to there kept and on the timeline
    assert!(o.t1 > o.t0);
    assert_eq!(w.t, o.t1);
    assert!(w.flange("A").unwrap().p[2] > -60.0 + 10.0);
    assert_eq!(
        w.timeline.last().unwrap().command,
        "move A to=0,0,-60 rpy=180,0,0"
    );
    // and the arm can go on from there: straight back up first
    ok(&mut w, "lift A mm=40\nmove A to=0,60,100");
}

#[test]
fn a_dry_run_that_collides_changes_nothing() {
    let mut w = World::default();
    ok(&mut w, "move A to=0,0,40 rpy=180,0,0");
    let before = w.clone();
    let o = w.apply(&parse_line("move A to=0,0,-60 rpy=180,0,0 dry=1").unwrap());
    assert!(o.dry && !o.ok && o.stopped.is_some());
    assert_eq!(w, before);
}

#[test]
fn what_cannot_start_is_refused() {
    let mut w = World::default();
    ok(&mut w, "reset circuit=demo");
    refused(
        &mut w,
        "pick A U1 grasp=ends",
        "A holds no tool, not tweezers (tool A tweezers)",
    );
    ok(&mut w, "tool A tweezers");
    refused(
        &mut w,
        "pick A U1 grasp=legs",
        "dip8 has no grasp frame legs (grasps: ends sides)",
    );
    refused(&mut w, "pick A Q9 grasp=body", "no part Q9");
    refused(&mut w, "place A U1 at=0,0,0", "A holds nothing");
    refused(&mut w, "release A U1", "A holds nothing");
    ok(&mut w, "pick A U1 grasp=ends");
    refused(&mut w, "tool A iron", "A holds U1: release it first");
    refused(&mut w, "pick A D1 grasp=body", "A already holds U1");
    refused(
        &mut w,
        "align A U1.1.tip to=U1.3.tip",
        "are on the same part",
    );
    refused(&mut w, "place A U1 at=0,700,0", "cannot go there");
}

#[test]
fn a_lead_is_aligned_to_a_lead() {
    let mut w = World::default();
    ok(&mut w, IN_VISE);
    ok(
        &mut w,
        "pick A R1 grasp=body\nlift A mm=80\nmove A to=0,60,60 rpy=180,0,90\nmove A to=34.81,1.27,-14.05",
    );
    let o = w.apply(&parse_line("align A R1.1.tip to=U1.3.tip gap=0.1").unwrap());
    assert!(o.ok, "{o:?}");
    let d = kin::norm(kin::sub(
        w.point("R1.1.tip").unwrap(),
        w.point("U1.3.tip").unwrap(),
    ));
    assert!((d - 0.1).abs() < 1e-6, "{d}");
}

#[test]
fn rewind_goes_back_and_keeps_both_branches() {
    let mut w = World::default();
    ok(
        &mut w,
        "reset circuit=demo\ntool A tweezers\npick A U1 grasp=ends\nmove A to=60,120,60",
    );
    let at_pick = w.timeline[2].t1;
    let mut fresh = World::default();
    ok(
        &mut fresh,
        "reset circuit=demo\ntool A tweezers\npick A U1 grasp=ends",
    );
    let o = w.apply(&parse_line(&format!("rewind t={at_pick}")).unwrap());
    assert!(o.ok && o.t1 == at_pick, "{o:?}");
    assert_eq!(w.arms, fresh.arms);
    assert_eq!(w.parts, fresh.parts);
    let marks: Vec<(String, bool)> = w
        .timeline
        .iter()
        .map(|e| (e.command.clone(), e.rewound))
        .collect();
    assert_eq!(
        marks,
        [
            ("reset circuit=demo".to_string(), false),
            ("tool A tweezers".to_string(), false),
            ("pick A U1 grasp=ends".to_string(), false),
            ("move A to=60,120,60".to_string(), true),
            (format!("rewind t={at_pick}"), false),
        ]
    );
    // a new branch from there
    ok(&mut w, "move A to=-60,120,60");
}

#[test]
fn a_dropped_part_lands_on_what_is_below_it() {
    // over the turntable: onto the vise, and it rides the turntable
    let mut w = World::default();
    ok(
        &mut w,
        "reset circuit=demo\ntool A tweezers\npick A U1 grasp=ends\nplace A U1 at=0,0,40\nrelease A U1\nlift A mm=40",
    );
    let u1 = w.part("U1").unwrap();
    assert!(!u1.lost && u1.on_stage && u1.holder.as_deref() == Some("vise"));
    let before = u1.pose().p;
    ok(&mut w, "stage turn=90");
    let after = w.part("U1").unwrap().pose().p;
    let want = kin::Pose::from_xyz_rpy([0.0; 3], [0.0, 0.0, 90.0])
        .then(&kin::Pose::from_xyz_rpy(before, [0.0; 3]))
        .p;
    assert!(near(after, want, 1e-6), "{after:?} {want:?}");
    // over the tray: onto it, resting there, nothing holding it
    let o = w.apply(&parse_line("pick A R1 grasp=body").unwrap());
    assert!(o.ok, "{o:?}");
    ok(&mut w, "place A R1 at=150,150,-60");
    let o = w.apply(&parse_line("release A R1").unwrap());
    assert!(
        o.warnings
            .iter()
            .any(|x| x.contains("onto the tray and rests there")),
        "{o:?}"
    );
    let r1 = w.part("R1").unwrap();
    assert!(!r1.lost && r1.holder.is_none() && !r1.on_stage);
    // and it can be picked up again
    ok(&mut w, "pick A R1 grasp=body");
    assert!(w.clashes().is_empty(), "{:?}", w.clashes());
}
