//! The arms, stage and vise in the cell: moves arrive where they were
//! sent, straight moves stay straight, joint moves respect every speed
//! and arrive together, and every limit is refused with nothing changed.

use xetal_ext_cell::command::{parse_line, parse_program};
use xetal_ext_cell::kin;
use xetal_ext_cell::motion::Profile;
use xetal_ext_cell::rig::Rig;
use xetal_ext_cell::world::World;

fn run(w: &mut World, program: &str) {
    for o in w.apply_all(&parse_program(program).unwrap()) {
        assert!(o.ok, "{o:?}");
    }
}

#[test]
fn arms_start_home_pointing_down() {
    let w = World::default();
    let rig = Rig::arm().unwrap();
    for a in &w.arms {
        assert_eq!(a.joints, rig.home);
        let f = w.flange(&a.name).unwrap();
        assert!((f.z()[2] + 1.0).abs() < 1e-12, "{}", a.name);
        // 100 mm in from the ring, 390 mm above the work zone
        assert!((f.p[0].hypot(f.p[1]) - 100.0).abs() < 1e-9, "{}", a.name);
        assert!((f.p[2] - 390.0).abs() < 1e-9, "{}", a.name);
    }
}

#[test]
fn a_move_arrives_where_it_was_sent() {
    let mut w = World::default();
    // the arm to each corner of the work zone in turn
    for (arm, at) in [
        ("A", [70.0, 70.0, 20.0]),
        ("A", [-70.0, 70.0, 20.0]),
        ("A", [-70.0, -70.0, 20.0]),
        ("A", [70.0, -70.0, 20.0]),
    ] {
        run(
            &mut w,
            &format!("move {arm} to={},{},{} rpy=180,0,30", at[0], at[1], at[2]),
        );
        let f = w.flange(arm).unwrap();
        assert!(kin::norm(kin::sub(f.p, at)) < 1e-6, "{arm}: {:?}", f.p);
        let want = kin::rpy_to_r([180.0, 0.0, 30.0]);
        assert!(kin::angle_between(&f.r, &want) < 1e-4, "{arm}");
    }
    run(&mut w, "lift A mm=25");
    assert!((w.flange("A").unwrap().p[2] - 45.0).abs() < 1e-6);
}

#[test]
fn a_straight_move_stays_straight() {
    let mut w = World::default();
    run(&mut w, "move A to=0,0,20 rpy=180,0,0");
    let start = w.flange("A").unwrap().p;
    let t0 = w.t;
    run(&mut w, "move A to=40,30,-10 speed=50");
    let end = w.flange("A").unwrap().p;
    let rig = Rig::arm().unwrap();
    let arm = w.arm("A").unwrap();
    let base = arm.base_pose();
    let dir = kin::scale(kin::sub(end, start), 1.0 / kin::norm(kin::sub(end, start)));
    let rows: Vec<_> = arm.track.iter().filter(|r| r[0] >= t0).collect();
    assert!(rows.len() > 10);
    let mut worst: f64 = 0.0;
    for r in rows {
        let q = [r[1], r[2], r[3], r[4], r[5], r[6]];
        let p = base.then(&kin::forward(&rig, &q)).p;
        let v = kin::sub(p, start);
        let along = v[0] * dir[0] + v[1] * dir[1] + v[2] * dir[2];
        worst = worst.max(kin::norm(kin::sub(v, kin::scale(dir, along))));
    }
    assert!(worst < 0.05, "{worst} mm off the line");
    // 58.3 mm at 50 mm/s, accelerating at 1000: a little over 1.2 s
    let dt = w.t - t0;
    assert!(dt > 58.3 / 50.0 && dt < 58.3 / 50.0 + 0.1, "{dt}");
}

#[test]
fn a_joint_move_respects_speeds_and_arrives_together() {
    let rig = Rig::arm().unwrap();
    let mut w = World::default();
    let from = w.arm("A").unwrap().joints;
    let to = [-30.0, -20.0, 10.0, 90.0, 45.0, -120.0];
    run(&mut w, "joints A -30 -20 10 90 45 -120 speed=0.8");
    let arm = w.arm("A").unwrap();
    assert_eq!(arm.joints, to);
    let rows = &arm.track;
    for pair in rows.windows(2) {
        let dt = pair[1][0] - pair[0][0];
        assert!(dt > 0.0);
        for (i, j) in rig.joints.iter().enumerate() {
            let v = (pair[1][i + 1] - pair[0][i + 1]).abs() / dt;
            assert!(v <= j.speed * 0.8 * 1.0001, "{}: {v} deg/s", j.name);
        }
    }
    // together: every joint that moves is still short of its target one
    // sample before the end, each the same fraction of its way
    let before = rows[rows.len() - 2];
    let fractions: Vec<f64> = (0..6)
        .filter(|&i| (to[i] - from[i]).abs() > 1e-9)
        .map(|i| (before[i + 1] - from[i]) / (to[i] - from[i]))
        .collect();
    assert!(
        fractions
            .iter()
            .all(|f| *f < 1.0 && (f - fractions[0]).abs() < 1e-6),
        "{fractions:?}"
    );
}

#[test]
fn trapezoids_and_triangles() {
    // long: accelerate, cruise, decelerate
    let p = Profile::new(0.5, 2.0);
    assert!((p.ta - 0.25).abs() < 1e-12 && (p.t - 2.25).abs() < 1e-12);
    // short: a triangle, never reaching the top speed
    let q = Profile::new(10.0, 4.0);
    assert!((q.ta - 0.5).abs() < 1e-12 && (q.t - 1.0).abs() < 1e-12 && q.v < 10.0);
    for pr in [p, q] {
        let mut last = 0.0;
        for i in 0..=100 {
            let s = pr.at(pr.t * f64::from(i) / 100.0);
            assert!(s >= last - 1e-12);
            last = s;
        }
        assert!((pr.at(pr.t) - 1.0).abs() < 1e-12 && pr.at(0.0) == 0.0);
    }
    let s = p.stretched(4.5);
    assert!((s.t - 4.5).abs() < 1e-12 && (s.at(2.25) - 0.5).abs() < 1e-9);
}

#[test]
fn every_limit_is_refused_with_nothing_changed() {
    let rig = Rig::arm().unwrap();
    let mut w = World::default();
    run(&mut w, "move A to=0,0,20 rpy=180,0,0");
    let before = w.clone();
    for (i, j) in rig.joints.iter().enumerate() {
        for beyond in [j.min - 1.0, j.max + 1.0] {
            let mut q = rig.home;
            q[i] = beyond;
            let line = format!(
                "joints A {} {} {} {} {} {}",
                q[0], q[1], q[2], q[3], q[4], q[5]
            );
            let o = w.apply(&parse_line(&line).unwrap());
            let why = o.refused.unwrap_or_default();
            assert!(
                !o.ok && why.starts_with(&format!("{} at ", j.name)),
                "{line}: {why}"
            );
            assert!(why.contains("beyond its limits"), "{why}");
            assert_eq!(w, before, "{line}");
        }
    }
    for (line, says) in [
        ("move A to=900,0,0", "out of reach"),
        ("move A to=0,450,100 rpy=180,0,0", "(near a singularity)"),
        ("lift A mm=100", ""),
    ] {
        let mut copy = w.clone();
        let o = copy.apply(&parse_line(line).unwrap());
        if says.is_empty() {
            assert!(o.ok, "{line}: {o:?}");
            continue;
        }
        let why = o.refused.unwrap_or_default();
        assert!(
            !o.ok && why.contains("cannot go there in a straight line") && why.contains(says),
            "{line}: {why}"
        );
        assert_eq!(copy, w, "{line}");
    }
}

#[test]
fn a_dry_move_changes_nothing() {
    let mut w = World::default();
    let o = w.apply(&parse_line("move A to=0,0,0 rpy=180,0,0 dry=1").unwrap());
    assert!(o.ok && o.dry && o.t1 > 0.0, "{o:?}");
    assert_eq!(w, World::default());
}

#[test]
fn stage_and_vise() {
    let mut w = World::default();
    run(&mut w, "stage turn=90\nstage turn=-30\nvise close");
    assert_eq!(w.stage, 60.0);
    assert_eq!(w.vise, "closed");
    // 90 degrees at 90 deg/s, accelerating at 360: 1.25 s; 30: 0.583 s
    assert!((w.timeline[0].t1 - 1.25).abs() < 1e-6);
    assert!((w.timeline[1].t1 - w.timeline[1].t0 - (30.0 / 90.0 + 0.25)).abs() < 1e-5);
    assert_eq!(w.stage_track.last().unwrap()[1], 60.0);
    let o = w.apply(&parse_line("vise close").unwrap());
    assert!(
        o.ok && o.warnings == ["the vise was already closed"],
        "{o:?}"
    );
}

#[test]
fn the_timeline_records_every_motion() {
    let mut w = World::default();
    run(
        &mut w,
        "joints A 10 0 0 0 90 0\nmove A to=0,0,30 rpy=180,0,0\nstage turn=45\nwait s=1",
    );
    let ops: Vec<&str> = w
        .timeline
        .iter()
        .map(|e| e.command.split(' ').next().unwrap())
        .collect();
    assert_eq!(ops, ["joints", "move", "stage", "wait"]);
    for pair in w.timeline.windows(2) {
        assert_eq!(pair[0].t1, pair[1].t0);
    }
    // a track a player can draw: rows in time order, starting at 0
    for a in &w.arms {
        assert_eq!(a.track[0][0], 0.0);
        assert!(a.track.windows(2).all(|p| p[1][0] > p[0][0]));
    }
    assert_eq!(World::from_json(&w.to_json()).unwrap(), w);
}
