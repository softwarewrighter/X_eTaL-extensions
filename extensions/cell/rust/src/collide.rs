//! Collision: the cell as solids -- an arm's links and joint housings as
//! capsules, its tool as two capsules, a part's body as a box or a
//! cylinder and its leads as thin capsules, the fixtures as boxes and
//! cylinders, and the floor -- tested with parry3d (behind a bounding
//! sphere each). Solids are gathered in groups (an arm with its tool and
//! what it holds; a free part; a fixture; the stage's turning top with
//! what the vise holds); only a group that moves is tested, against the
//! others, and pairs of groups a motion is allowed to touch are passed
//! by name. An arm does not collide with itself (not simulated).

use crate::kin::{self, Pose, V3};
use crate::layout::FLOOR;
use crate::parts::PartKind;
use crate::rig::Rig;
use parry3d_f64::math::{Mat3, Pose3, Rot3, Vec3};
use parry3d_f64::query;
use parry3d_f64::shape::{Capsule, Cuboid, Cylinder};

/// A link's radius, joints 1 to 6, mm: as CellModels.xtl draws them.
pub const LINK_RADII: [f64; 6] = [48.0, 42.0, 34.0, 30.0, 26.0, 22.0];
/// A joint housing's radius and length along its axis, joints 1 to 6:
/// as CellModels.xtl draws them.
pub const HOUSINGS: [(f64, f64); 6] = [
    (70.0, 120.0),
    (60.0, 110.0),
    (52.0, 100.0),
    (40.0, 70.0),
    (34.0, 60.0),
    (30.0, 40.0),
];
/// A tool's block at the flange: a capsule this radius from 6 to 30 mm.
pub const TOOL_BLOCK: f64 = 19.0;
/// A tool's working end: a capsule this radius from 34 mm to its tip.
pub const TOOL_END: f64 = 5.0;

/// A shape in the cell's frame.
#[derive(Clone, Debug)]
pub enum Shape {
    /// A segment and a radius.
    Capsule(V3, V3, f64),
    /// A box: its pose (its middle) and half its sides.
    Box(Pose, V3),
    /// A cylinder along its pose's z: half its length and its radius.
    Cylinder(Pose, f64, f64),
}

/// A shape with a name for messages.
#[derive(Clone, Debug)]
pub struct Solid {
    /// What it is: `arm B link 5`, `R1 body`.
    pub label: String,
    /// Its shape.
    pub shape: Shape,
    /// It must stay above the floor.
    pub floored: bool,
}

/// Solids that move together.
#[derive(Clone, Debug, Default)]
pub struct Group {
    /// `arm:A`, `part:U1`, `fixture:stage`, `stage`.
    pub name: String,
    /// Its solids.
    pub solids: Vec<Solid>,
}

fn v(p: V3) -> Vec3 {
    Vec3::new(p[0], p[1], p[2])
}

fn pose3(p: &Pose) -> Pose3 {
    let r = p.r;
    let m = Mat3::from_cols(
        Vec3::new(r[0][0], r[1][0], r[2][0]),
        Vec3::new(r[0][1], r[1][1], r[2][1]),
        Vec3::new(r[0][2], r[1][2], r[2][2]),
    );
    Pose3::from_parts(v(p.p), Rot3::from_mat3(&m))
}

impl Shape {
    /// A sphere round it: its middle and radius.
    #[must_use]
    pub fn sphere(&self) -> (V3, f64) {
        match self {
            Shape::Capsule(a, b, r) => (
                kin::scale(kin::add(*a, *b), 0.5),
                kin::norm(kin::sub(*a, *b)) / 2.0 + r,
            ),
            Shape::Box(p, h) => (p.p, kin::norm(*h)),
            Shape::Cylinder(p, h, r) => (p.p, h.hypot(*r)),
        }
    }

    /// Its lowest point's height.
    #[must_use]
    pub fn lowest(&self) -> f64 {
        match self {
            Shape::Capsule(a, b, r) => a[2].min(b[2]) - r,
            Shape::Box(p, h) => p.p[2] - (0..3).map(|j| p.r[2][j].abs() * h[j]).sum::<f64>(),
            Shape::Cylinder(p, h, r) => {
                let z = p.r[2][2].abs();
                p.p[2] - z * h - (1.0 - z * z).max(0.0).sqrt() * r
            }
        }
    }

    fn hits(&self, other: &Shape) -> bool {
        // parry's cylinder runs along its y: turned to the pose's z
        let up = Pose::from_xyz_rpy([0.0; 3], [90.0, 0.0, 0.0]);
        let place = |s: &Shape| -> (Pose3, Box<dyn parry3d_f64::shape::Shape>) {
            match s {
                Shape::Capsule(a, b, r) => {
                    (Pose3::IDENTITY, Box::new(Capsule::new(v(*a), v(*b), *r)))
                }
                Shape::Box(p, h) => (pose3(p), Box::new(Cuboid::new(v(*h)))),
                Shape::Cylinder(p, h, r) => (pose3(&p.then(&up)), Box::new(Cylinder::new(*h, *r))),
            }
        };
        let (p1, s1) = place(self);
        let (p2, s2) = place(other);
        query::intersection_test(&p1, s1.as_ref(), &p2, s2.as_ref()).is_ok_and(|i| i.intersecting)
    }
}

fn capsule(label: String, a: V3, b: V3, r: f64, floored: bool) -> Solid {
    Solid {
        label,
        shape: Shape::Capsule(a, b, r),
        floored,
    }
}

/// An arm's solids at joint angles: its links and housings, its tool
/// (its length along the flange's z) if it holds one.
#[must_use]
pub fn arm(name: &str, base: &Pose, rig: &Rig, q: &[f64; 6], tool: Option<f64>) -> Vec<Solid> {
    let f = kin::frames(rig, q).map(|p| base.then(&p));
    let mut out = Vec::new();
    for i in 0..6 {
        let (from, to) = (f[i], f[i + 1]);
        let z = from.z();
        let mid = kin::add(from.p, kin::scale(z, rig.joints[i].d));
        // the base's column and housing stand on the floor
        let floored = i > 0;
        out.push(capsule(
            format!("arm {name} link {}", i + 1),
            from.p,
            mid,
            LINK_RADII[i],
            floored,
        ));
        out.push(capsule(
            format!("arm {name} link {}", i + 1),
            mid,
            to.p,
            LINK_RADII[i],
            floored,
        ));
        let (r, len) = HOUSINGS[i];
        let h = kin::scale(z, len / 2.0);
        out.push(capsule(
            format!("arm {name} joint {}", i + 1),
            kin::sub(from.p, h),
            kin::add(from.p, h),
            r,
            i > 1,
        ));
    }
    if let Some(len) = tool {
        let flange = f[6];
        let at = |d: f64| flange.then(&kin::tz(d)).p;
        out.push(capsule(
            format!("arm {name}'s tool"),
            at(6.0),
            at(30.0),
            TOOL_BLOCK,
            true,
        ));
        out.push(capsule(
            format!("arm {name}'s tool"),
            at(34.0),
            at(len - TOOL_END),
            TOOL_END,
            true,
        ));
    }
    out
}

/// A part's solids at a pose: its body and each lead.
#[must_use]
pub fn part(id: &str, kind: &PartKind, pose: &Pose) -> Vec<Solid> {
    let b = &kind.body;
    let body = match b.shape.as_str() {
        "box" => {
            let s = b.size.unwrap_or([1.0; 3]);
            Shape::Box(*pose, [s[0] / 2.0, s[1] / 2.0, s[2] / 2.0])
        }
        _ => {
            let turn = match b.axis.as_deref() {
                Some("x") => [0.0, 90.0, 0.0],
                Some("y") => [-90.0, 0.0, 0.0],
                _ => [0.0; 3],
            };
            let p = pose.then(&Pose::from_xyz_rpy([0.0; 3], turn));
            Shape::Cylinder(
                p,
                b.length.unwrap_or(1.0) / 2.0,
                b.diameter.unwrap_or(1.0) / 2.0,
            )
        }
    };
    let mut out = vec![Solid {
        label: format!("{id} body"),
        shape: body,
        floored: true,
    }];
    for pin in &kind.pins {
        for w in pin.lead.windows(2) {
            let a = pose.then(&Pose::from_xyz_rpy(w[0], [0.0; 3])).p;
            let c = pose.then(&Pose::from_xyz_rpy(w[1], [0.0; 3])).p;
            out.push(capsule(
                format!("{id}.{} lead", pin.name),
                a,
                c,
                pin.diameter / 2.0,
                true,
            ));
        }
    }
    out
}

/// A box solid of a fixture: its middle and sides in the fixture's
/// frame.
#[must_use]
pub fn fixture_box(label: &str, frame: &Pose, at: V3, size: V3) -> Solid {
    Solid {
        label: label.to_string(),
        shape: Shape::Box(
            frame.then(&Pose::from_xyz_rpy(at, [0.0; 3])),
            size.map(|x| x / 2.0),
        ),
        floored: false,
    }
}

/// A cylinder solid of a fixture, along its z: its middle, radius and
/// length in the fixture's frame.
#[must_use]
pub fn fixture_cylinder(label: &str, frame: &Pose, at: V3, r: f64, len: f64) -> Solid {
    Solid {
        label: label.to_string(),
        shape: Shape::Cylinder(frame.then(&Pose::from_xyz_rpy(at, [0.0; 3])), len / 2.0, r),
        floored: false,
    }
}

impl Group {
    /// A sphere round all its solids: its middle and radius.
    #[must_use]
    pub fn sphere(&self) -> (V3, f64) {
        let spheres: Vec<(V3, f64)> = self.solids.iter().map(|s| s.shape.sphere()).collect();
        if spheres.is_empty() {
            return ([0.0; 3], -1.0);
        }
        #[allow(clippy::cast_precision_loss)]
        let c = kin::scale(
            spheres.iter().fold([0.0; 3], |a, (c, _)| kin::add(a, *c)),
            1.0 / spheres.len() as f64,
        );
        let r = spheres
            .iter()
            .map(|(p, r)| kin::norm(kin::sub(*p, c)) + r)
            .fold(0.0, f64::max);
        (c, r)
    }
}

/// The first solid of `moving` touching one of `others` (or the floor),
/// skipping the groups in `allowed`: what touched what. Groups whose
/// spheres are apart are skipped whole.
#[must_use]
pub fn first_hit(moving: &Group, others: &[Group], allowed: &[String]) -> Option<String> {
    for s in &moving.solids {
        if s.floored && s.shape.lowest() < FLOOR - 1e-6 {
            return Some(format!("{} with the floor", s.label));
        }
    }
    let (mc, mr) = moving.sphere();
    for g in others {
        if g.name == moving.name || allowed.contains(&g.name) {
            continue;
        }
        let (gc, gr) = g.sphere();
        if gr < 0.0 || kin::norm(kin::sub(mc, gc)) > mr + gr {
            continue;
        }
        for s in &moving.solids {
            let (c1, r1) = s.shape.sphere();
            for o in &g.solids {
                let (c2, r2) = o.shape.sphere();
                if kin::norm(kin::sub(c1, c2)) > r1 + r2 {
                    continue;
                }
                if s.shape.hits(&o.shape) {
                    return Some(format!("{} with {}", s.label, o.label));
                }
            }
        }
    }
    None
}
