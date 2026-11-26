//! The microscope (step 20): aimed at a joint or pin, it turns its head
//! so its eye's axis passes through it; it follows its aim when the stage
//! moves the work; zoom sets the field; `snap` records a picture.

use xetal_ext_cell::command::parse_program;
use xetal_ext_cell::kin;
use xetal_ext_cell::scope;
use xetal_ext_cell::world::World;

/// The distance from `p` to the eye's axis at a pan and tilt.
fn off_axis(p: kin::V3, pan: f64, tilt: f64) -> f64 {
    let e = scope::eye(pan, tilt);
    let d = kin::sub(p, e.p);
    let z = e.z();
    let along = d[0] * z[0] + d[1] * z[1] + d[2] * z[2];
    kin::norm(kin::sub(d, kin::scale(e.z(), along)))
}

#[test]
fn aim_points_the_eye_through_the_target() {
    for at in [[0.0, 0.0, 0.0], [40.0, -25.0, -10.0], [-60.0, 30.0, 20.0]] {
        let (pan, tilt) = scope::aim_at(at, 0.0);
        assert!(
            off_axis(at, pan, tilt) < 1e-6,
            "{at:?}: pan {pan} tilt {tilt}"
        );
    }
}

#[test]
fn the_solder_program_looks_at_its_joint_and_follows_it() {
    let mut w = World::default();
    let out = w.apply_all(&parse_program(include_str!("../../programs/solder.cell")).unwrap());
    assert!(out.iter().all(|o| o.ok), "{:?}", out.iter().find(|o| !o.ok));
    assert!(out.iter().any(|o| {
        o.events
            .iter()
            .any(|e| e == "the scope looks at U1.3+R1.1, 15 mm across")
    }));
    assert_eq!(w.scope.aim.as_deref(), Some("U1.3+R1.1"));
    // where the joint is now, after the stage presented it: on the axis
    let j = w.joints.iter().find(|j| j.name == "U1.3+R1.1").unwrap();
    let p = w.joint_point(j).unwrap();
    assert!(
        off_axis(p, w.scope.pan, w.scope.tilt) < 0.01,
        "{}",
        off_axis(p, w.scope.pan, w.scope.tilt)
    );
    // the track's times only move forward
    assert!(w.scope.track.windows(2).all(|r| r[1][0] >= r[0][0]));
    assert_eq!(World::from_json(&w.to_json()).unwrap(), w);
}

#[test]
fn snap_records_a_picture() {
    let mut w = World::default();
    let out = w.apply_all(&parse_program("reset circuit=demo\nscope zoom=4\nsnap").unwrap());
    assert!(out.iter().all(|o| o.ok), "{out:?}");
    assert_eq!(out[2].events, ["a picture of the work zone, 30 mm across"]);
    assert_eq!(w.scope.snaps.len(), 1);
}
