//! What a model of the arm must be, from the rig: the asset contract of
//! docs/cell-design.md. The arm is a hierarchy of nodes, one a joint's
//! pivot -- arm_base, arm_j1 to arm_j6, arm_flange -- each placed in its
//! parent (mm; turn as Euler XYZ degrees, Blender's), its local z the
//! joint's axis; the links between them; and poses (joint angles with
//! where the flange must then be) that a check of the exported model
//! sets and compares. Blender builds the arm from this and writes it
//! back unchanged as `arm.rig.json`, the file the simulator reads
//! (`Rig::arm`).

use crate::kin::{self, Pose};
use crate::rig::Rig;
use serde_json::{Value as Json, json};

/// The joint angles a check poses the model at, degrees.
pub const CHECK_POSES: [[f64; 6]; 3] = [
    [0.0, 0.0, 0.0, 0.0, 90.0, 0.0],
    [30.0, 20.0, -40.0, 60.0, 45.0, -90.0],
    [-120.0, -60.0, 100.0, -150.0, -100.0, 200.0],
];

/// Frame `n` (0 the base) in the base frame, every joint at 0.
fn prefix(rig: &Rig, n: usize) -> Pose {
    (0..n).fold(Pose::IDENTITY, |pose, i| pose.then(&single(rig, i, 0.0)))
}

/// Link i alone at joint angle `angle` (degrees).
fn single(rig: &Rig, i: usize, angle: f64) -> Pose {
    let j = &rig.joints[i];
    let theta = (angle + j.offset).to_radians();
    let alpha = j.alpha.to_radians();
    let r = kin::mm(&rz(theta), &rx(alpha));
    let (s, c) = theta.sin_cos();
    Pose {
        p: [j.a * c, j.a * s, j.d],
        r,
    }
}

fn rx(a: f64) -> kin::M3 {
    let (s, c) = a.sin_cos();
    [[1.0, 0.0, 0.0], [0.0, c, -s], [0.0, s, c]]
}

fn rz(a: f64) -> kin::M3 {
    let (s, c) = a.sin_cos();
    [[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]]
}

fn r3(v: kin::V3) -> Json {
    json!(v.map(|x| kin::clean((x * 1e6).round() / 1e6)))
}

/// The arm's model: its nodes, links and check poses, with the rig.
#[must_use]
pub fn arm(rig: &Rig) -> Json {
    let names = [
        "arm_j1",
        "arm_j2",
        "arm_j3",
        "arm_j4",
        "arm_j5",
        "arm_j6",
        "arm_flange",
    ];
    let mut nodes = vec![json!({
        "name": "arm_base", "parent": null, "at": [0.0, 0.0, 0.0], "turn": [0.0, 0.0, 0.0],
        "world": [0.0, 0.0, 0.0], "axis": [0.0, 0.0, 1.0], "does": "the base, on the floor",
    })];
    // arm_j1 sits at the base frame; arm_j(i+1) at frame i; the flange at frame 6
    for (k, name) in names.iter().enumerate() {
        let (parent, rel) = if k == 0 {
            ("arm_base", Pose::IDENTITY)
        } else {
            (names[k - 1], single(rig, k - 1, 0.0))
        };
        let world = prefix(rig, k);
        let does = if k < 6 {
            format!(
                "{} ({}), about its local z",
                rig.joints[k].name, rig.joints[k].does
            )
        } else {
            "tools attach here; z is the approach".to_string()
        };
        nodes.push(json!({
            "name": name, "parent": parent, "at": r3(rel.p), "turn": r3(rel.rpy()),
            "world": r3(world.p), "axis": r3(world.z()), "does": does,
        }));
    }
    // each joint group's link to the next pivot: along its z by d, then
    // along its rotated x by a (in the group's own frame)
    let mut links = Vec::new();
    for (i, j) in rig.joints.iter().enumerate() {
        let theta = j.offset.to_radians();
        let mid = [0.0, 0.0, j.d];
        let end = [j.a * theta.cos(), j.a * theta.sin(), j.d];
        links.push(json!({"group": names[i], "from": [0.0, 0.0, 0.0], "to": r3(mid)}));
        links.push(json!({"group": names[i], "from": r3(mid), "to": r3(end)}));
    }
    let poses: Vec<Json> = CHECK_POSES
        .iter()
        .map(|q| {
            let f = kin::forward(rig, q);
            json!({"q": q, "flange": r3(f.p), "z": r3(f.z())})
        })
        .collect();
    json!({
        "name": "arm",
        "units": "mm, degrees",
        "contract": "docs/cell-design.md, the asset contract: one node a joint's pivot, its local z the joint's axis",
        "nodes": nodes,
        "links": links,
        "poses": poses,
        "rig": serde_json::to_value(rig).unwrap_or(Json::Null),
    })
}
