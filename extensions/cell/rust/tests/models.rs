//! The models' rig files: the arm the simulator runs is the arm Blender
//! exported, and that is what rigs/arm.toml makes; every rig file
//! copied in from the models reads.

use xetal_ext_cell::model;
use xetal_ext_cell::rig::{ARM_RIG_JSON, ARM_TOML, Rig};

#[test]
fn the_exported_arm_is_the_authored_arm() {
    let authored = Rig::from_toml(ARM_TOML).unwrap();
    assert_eq!(Rig::arm().unwrap(), authored);
    // the whole model description, not only the rig: nodes, links, poses
    let exported: serde_json::Value = serde_json::from_str(ARM_RIG_JSON).unwrap();
    assert_eq!(
        exported,
        model::arm(&authored),
        "rigs/arm.rig.json is not what rigs/arm.toml makes: run just ext cell models, then just ext cell rigs"
    );
}

#[test]
fn the_asset_contract() {
    let m = model::arm(&Rig::arm().unwrap());
    let names: Vec<&str> = m["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "arm_base",
            "arm_j1",
            "arm_j2",
            "arm_j3",
            "arm_j4",
            "arm_j5",
            "arm_j6",
            "arm_flange"
        ]
    );
    // each node hangs from the one before it
    for (k, n) in m["nodes"].as_array().unwrap().iter().enumerate().skip(1) {
        assert_eq!(n["parent"], names[k - 1]);
    }
    // at rest the flange is where forward kinematics puts it at all 0
    let flange = &m["nodes"][7]["world"];
    assert_eq!(flange, &serde_json::json!([420.0, 0.0, 660.0]));
}

#[test]
fn every_rig_file_reads() {
    let files = [
        ("tweezers", include_str!("../../rigs/tweezers.rig.json")),
        ("iron", include_str!("../../rigs/iron.rig.json")),
        ("solder", include_str!("../../rigs/solder.rig.json")),
        ("clip", include_str!("../../rigs/clip.rig.json")),
        ("welder", include_str!("../../rigs/welder.rig.json")),
        ("cutter", include_str!("../../rigs/cutter.rig.json")),
        ("wire", include_str!("../../rigs/wire.rig.json")),
        ("rack", include_str!("../../rigs/rack.rig.json")),
        ("tray", include_str!("../../rigs/tray.rig.json")),
        ("stage", include_str!("../../rigs/stage.rig.json")),
        ("scope", include_str!("../../rigs/scope.rig.json")),
    ];
    for (name, text) in files {
        let v: serde_json::Value = serde_json::from_str(text).unwrap();
        assert_eq!(v["name"], name);
        let nodes = v["nodes"].as_array().unwrap();
        assert!(!nodes.is_empty(), "{name}");
        // a tool has its mount at the flange and a tip out along z
        if !["rack", "tray", "stage", "scope"].contains(&name) {
            let tip = nodes.iter().find(|n| n["name"] == "tool_tip").unwrap();
            let w = tip["world"].as_array().unwrap();
            assert!(
                w[0].as_f64() == Some(0.0)
                    && w[1].as_f64() == Some(0.0)
                    && w[2].as_f64().unwrap() > 50.0,
                "{name}"
            );
        }
    }
}
