//! The parts library: every file parses and checks; pins, grasps and
//! thermal data are as the files say; a wrong file is refused.

use xetal_ext_cell::parts::{self, PartKind, distance};

#[test]
fn every_kind_parses_in_order() {
    let lib = parts::library().unwrap();
    let kinds: Vec<&str> = lib.iter().map(|k| k.kind.as_str()).collect();
    assert_eq!(kinds, parts::kinds());
    assert_eq!(
        kinds,
        [
            "dip8",
            "dip14",
            "dip16",
            "to92",
            "axial-resistor",
            "axial-diode",
            "radial-electrolytic",
            "ceramic-disc",
            "led-5mm",
            "battery-snap"
        ]
    );
    let pins: Vec<usize> = lib.iter().map(|k| k.pins.len()).collect();
    assert_eq!(pins, [8, 14, 16, 3, 2, 2, 2, 2, 2, 2]);
}

#[test]
fn every_file_is_named_for_its_kind() {
    for k in parts::kinds() {
        assert_eq!(parts::kind(k).unwrap().kind, k);
    }
    assert!(
        parts::kind("sot23")
            .unwrap_err()
            .contains("no part kind sot23")
    );
}

#[test]
fn every_number_is_sourced_or_illustrative() {
    for k in parts::library().unwrap() {
        let t = &k.thermal;
        let mut all: Vec<&str> = t
            .sourced
            .iter()
            .chain(&t.illustrative)
            .map(String::as_str)
            .collect();
        all.sort_unstable();
        assert_eq!(
            all,
            [
                "body_limit_c",
                "body_limit_s",
                "heat_capacity_j_per_k",
                "mass_g"
            ],
            "{}",
            k.kind
        );
        assert_eq!(t.sourced.is_empty(), t.source.is_none(), "{}", k.kind);
    }
}

#[test]
fn dip_pins_count_counterclockwise() {
    let k = parts::kind("dip8").unwrap();
    let p = |n: &str| k.pin(n).unwrap().tip();
    // pin 1 at -x on the -y row; 4 at +x on it; 5 across from 4; 8 across from 1
    assert!(p("1")[0] < 0.0 && p("1")[1] < 0.0);
    assert!((p("4")[0] - 3.81).abs() < 1e-9 && p("4")[1] < 0.0);
    assert_eq!([p("5")[0], -p("5")[1]], [p("4")[0], p("4")[1]]);
    assert_eq!([p("8")[0], -p("8")[1]], [p("1")[0], p("1")[1]]);
    // 2.54 mm pitch, rows 7.62 mm apart, every lead below the body
    assert!((distance(p("1"), p("2")) - 2.54).abs() < 1e-9);
    assert!((distance(p("1"), p("8")) - 7.62).abs() < 1e-9);
    assert!(k.pins.iter().all(|pin| pin.tip()[2] < -1.65));
    let d16 = parts::kind("dip16").unwrap();
    assert!(
        (distance(d16.pin("1").unwrap().tip(), d16.pin("8").unwrap().tip()) - 7.0 * 2.54).abs()
            < 1e-9
    );
}

#[test]
fn leads_and_grasps() {
    let r = parts::kind("axial-resistor").unwrap();
    assert!((r.pin("1").unwrap().length() - 28.0).abs() < 1e-9);
    assert_eq!(r.grasp("body").unwrap().width, 2.3);
    let led = parts::kind("led-5mm").unwrap();
    // the anode is the longer lead
    assert!(led.pin("1").unwrap().length() > led.pin("2").unwrap().length());
    assert_eq!(led.pin("1").unwrap().role.as_deref(), Some("anode"));
    let t = parts::kind("to92").unwrap();
    assert!(t.heat_sensitive);
    let roles: Vec<_> = t.pins.iter().map(|p| p.role.clone().unwrap()).collect();
    assert_eq!(roles, ["E", "B", "C"]);
}

#[test]
fn json_has_the_pins_worked_out() {
    let j: serde_json::Value =
        serde_json::from_str(&parts::kind("dip14").unwrap().to_json()).unwrap();
    assert_eq!(j["pins"].as_array().unwrap().len(), 14);
    assert_eq!(j["pins"][13]["name"], "14");
    assert_eq!(j["grasps"][0]["name"], "ends");
}

const GOOD: &str = r#"
kind = "x"
title = "X"
example = "x"
heat_sensitive = false
[body]
shape = "box"
size = [1, 1, 1]
material = "epoxy"
[[pin]]
name = "1"
lead = [[0, 0, 0], [0, 0, -5]]
diameter = 0.5
material = "copper"
[[grasp]]
name = "body"
at = [0, 0, 0]
rpy = [0, 0, 0]
width = 1
[thermal]
mass_g = 1
heat_capacity_j_per_k = 1
body_limit_c = 260
body_limit_s = 10
illustrative = ["mass_g", "heat_capacity_j_per_k", "body_limit_c", "body_limit_s"]
"#;

#[test]
fn wrong_files_are_refused() {
    PartKind::from_toml(GOOD).unwrap();
    let bad = |from: &str, to: &str, says: &str| {
        let e = PartKind::from_toml(&GOOD.replace(from, to)).unwrap_err();
        assert!(e.contains(says), "{from} -> {to}: {e}");
    };
    bad("\"box\"", "\"sphere\"", "no shape sphere");
    bad(
        "[[0, 0, 0], [0, 0, -5]]",
        "[[0, 0, 0]]",
        "two points or more",
    );
    bad("\"copper\"", "\"gold\"", "no lead material gold");
    bad("width = 1", "width = 0", "width must be positive");
    bad(
        "illustrative = [\"mass_g\", ",
        "illustrative = [",
        "mass_g must be in exactly one",
    );
    bad(
        "illustrative = [\"mass_g\", ",
        "sourced = [\"mass_g\"]\nillustrative = [",
        "sourced numbers need a source",
    );
    bad(
        "heat_sensitive = false",
        "heat_sensitive = false\ncolor = 1",
        "unknown field",
    );
    let twice = "[[pin]]\nname = \"1\"\nlead = [[0, 0, 0], [0, 0, -5]]\ndiameter = 0.5\nmaterial = \"copper\"\n[[grasp]]";
    bad("[[grasp]]", twice, "pin 1 twice");
}
