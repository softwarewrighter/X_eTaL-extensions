//! The arm's kinematics: the home pose, forward of inverse round trips
//! over a sample of poses, every solution found, the nearest chosen,
//! out of reach and beyond a limit told apart.

use xetal_ext_cell::kin::{self, Pose};
use xetal_ext_cell::rig::Rig;

fn close(a: [f64; 3], b: [f64; 3], eps: f64) -> bool {
    kin::norm(kin::sub(a, b)) < eps
}

/// The same rotation, element by element (acos near 1 is too coarse).
fn same_r(a: &kin::M3, b: &kin::M3) -> bool {
    (0..3).all(|i| (0..3).all(|j| (a[i][j] - b[i][j]).abs() < 1e-9))
}

/// A small deterministic generator: the same poses every run.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        #[allow(clippy::cast_precision_loss)]
        let x = (self.0 >> 11) as f64 / (1u64 << 53) as f64;
        x
    }

    fn joints(&mut self, rig: &Rig) -> [f64; 6] {
        std::array::from_fn(|i| {
            let j = &rig.joints[i];
            // inside the limits, a margin from each end
            let (lo, hi) = (j.min.max(-175.0) + 5.0, j.max.min(175.0) - 5.0);
            lo + (hi - lo) * self.next()
        })
    }
}

#[test]
fn the_rig_reads_and_checks() {
    let rig = Rig::arm().unwrap();
    assert_eq!(rig.joints.len(), 6);
    assert!(rig.illustrative);
    let text = include_str!("../../rigs/arm.toml");
    let e = Rig::from_toml(&text.replacen("alpha = 0", "alpha = 30", 1)).unwrap_err();
    assert!(e.contains("spherical wrist"), "{e}");
    let e = Rig::from_toml(&text.replacen(
        "home = [0, 0, 0, 0, 90, 0]",
        "home = [0, 0, 0, 0, 130, 0]",
        1,
    ))
    .unwrap_err();
    assert!(e.contains("home: j5 at 130 is beyond its limits"), "{e}");
    let e = Rig::from_toml(&text.replacen("speed = 120", "speed = 0", 1)).unwrap_err();
    assert!(e.contains("speed and accel must be positive"), "{e}");
}

#[test]
fn home_pose() {
    // upper arm up, elbow offset up, forearm out, flange facing forward
    let rig = Rig::arm().unwrap();
    let (flange, joints) = kin::forward_all(&rig, &[0.0; 6]);
    assert!(
        close(joints[1], [50.0, 0.0, 300.0], 1e-9),
        "{:?}",
        joints[1]
    );
    assert!(
        close(joints[2], [50.0, 0.0, 620.0], 1e-9),
        "{:?}",
        joints[2]
    );
    assert!(close(flange.p, [420.0, 0.0, 660.0], 1e-9), "{:?}", flange.p);
    assert!(
        close(flange.z(), [1.0, 0.0, 0.0], 1e-12),
        "{:?}",
        flange.z()
    );
}

#[test]
fn forward_of_inverse_round_trips() {
    let rig = Rig::arm().unwrap();
    let mut g = Lcg(17);
    let mut tried = 0;
    let mut found_original = 0;
    while tried < 500 {
        let q = g.joints(&rig);
        if q[4].abs() < 3.0 {
            continue; // a straight wrist has a family of solutions
        }
        tried += 1;
        let pose = kin::forward(&rig, &q);
        let all = kin::inverse_all(&rig, &pose, &q);
        assert!(!all.is_empty(), "no solution for {q:?}");
        for s in &all {
            let back = kin::forward(&rig, s);
            assert!(
                close(back.p, pose.p, 1e-6),
                "{q:?} -> {s:?}: {:?} vs {:?}",
                back.p,
                pose.p
            );
            assert!(same_r(&back.r, &pose.r), "{q:?} -> {s:?}");
            assert!(rig.beyond_limit(s).is_none(), "{s:?}");
        }
        if all
            .iter()
            .any(|s| s.iter().zip(&q).all(|(a, b)| (a - b).abs() < 1e-6))
        {
            found_original += 1;
        }
        // nearest the pose it came from: the pose itself
        let nearest = kin::inverse(&rig, &pose, &q).unwrap();
        assert!(
            nearest.iter().zip(&q).all(|(a, b)| (a - b).abs() < 1e-6),
            "{q:?} -> {nearest:?}"
        );
    }
    assert_eq!(found_original, tried);
}

#[test]
fn a_straight_wrist_keeps_its_roll() {
    let rig = Rig::arm().unwrap();
    let q = [10.0, 20.0, -30.0, 40.0, 0.0, 15.0];
    let pose = kin::forward(&rig, &q);
    let s = kin::inverse(&rig, &pose, &q).unwrap();
    let back = kin::forward(&rig, &s);
    assert!(close(back.p, pose.p, 1e-6));
    assert!(same_r(&back.r, &pose.r));
    assert!((s[3] - 40.0).abs() < 1e-6, "{s:?}");
}

#[test]
fn up_to_eight_solutions() {
    let rig = Rig::arm().unwrap().unlimited();
    let q = [20.0, 10.0, 20.0, 30.0, 40.0, 50.0];
    let pose = kin::forward(&rig, &q);
    let n = kin::inverse_all(&rig, &pose, &q).len();
    assert!((4..=8).contains(&n), "{n}");
}

#[test]
fn out_of_reach_and_beyond_a_limit() {
    let rig = Rig::arm().unwrap();
    let far = Pose::from_xyz_rpy([2000.0, 0.0, 300.0], [0.0, 90.0, 0.0]);
    assert_eq!(
        kin::inverse(&rig, &far, &[0.0; 6]).unwrap_err(),
        "out of reach"
    );
    // straight down at the base's own foot: reachable without limits only
    let foot = Pose::from_xyz_rpy([60.0, 0.0, -200.0], [180.0, 0.0, 0.0]);
    let e = kin::inverse(&rig, &foot, &[0.0; 6]).unwrap_err();
    assert_eq!(e, "every way to reach it is beyond a joint limit");
}

#[test]
fn rpy_round_trips() {
    for rpy in [
        [0.0, 0.0, 0.0],
        [180.0, 0.0, 0.0],
        [10.0, 20.0, 30.0],
        [-45.0, 60.0, 170.0],
    ] {
        let back = kin::r_to_rpy(&kin::rpy_to_r(rpy));
        for i in 0..3 {
            assert!((back[i] - rpy[i]).abs() < 1e-9, "{rpy:?} -> {back:?}");
        }
    }
    assert!((kin::wrap(190.0) + 170.0).abs() < 1e-12);
    assert!((kin::wrap(-180.0) - 180.0).abs() < 1e-12);
}
