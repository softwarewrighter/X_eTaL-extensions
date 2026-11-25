//! Tool stations and the positioner (decision C17): the positioner
//! turns, tilts and moves the stage with what the vise holds; a pin or
//! joint presented at a station's point; a station's iron, feeder and
//! welder working there; the layout, chosen by trying candidates on
//! every program, runs them all.

use xetal_ext_cell::command::{parse_line, parse_program};
use xetal_ext_cell::kin;
use xetal_ext_cell::stations;
use xetal_ext_cell::world::{Outcome, World};

fn run_all(program: &str) -> (World, Vec<Outcome>) {
    let mut w = World::default();
    let out = w.apply_all(&parse_program(program).unwrap());
    (w, out)
}

fn ok(program: &str) -> (World, Vec<Outcome>) {
    let (w, out) = run_all(program);
    for o in &out {
        assert!(o.ok, "{o:?}");
    }
    (w, out)
}

fn result(w: &World, name: &str) -> Option<String> {
    w.joints
        .iter()
        .find(|j| j.name == name)
        .and_then(|j| j.result.clone())
}

#[test]
fn the_stations_read_and_point_at_the_work() {
    let all = stations::data().unwrap();
    assert_eq!(
        all.iter().map(|s| s.tool.as_str()).collect::<Vec<_>>(),
        ["iron", "solder", "welder"]
    );
    for s in &all {
        assert!((kin::norm(s.from) - 1.0).abs() < 1e-12);
        // extended, the tip is at the point; retracted, a stroke back
        assert!(kin::norm(kin::sub(s.tip(s.stroke), s.point)) < 1e-9);
        assert!((kin::norm(kin::sub(s.tip(0.0), s.point)) - s.stroke).abs() < 1e-9);
    }
    // a new cell: every station retracted, the arms and stage clear of them
    let w = World::default();
    assert!(w.stations.iter().all(|s| s.ext == 0.0));
}

#[test]
fn the_positioner_carries_what_the_vise_holds() {
    let (mut w, _) = ok("reset circuit=demo
tool A tweezers
pick A U1 grasp=ends
place A U1 at=0,0,-32 rpy=0,0,90
vise close
release A U1
lift A mm=80");
    let before = w.part("U1").unwrap().pose().p;
    let local = w
        .stage_pose()
        .inverse()
        .then(&w.part("U1").unwrap().pose())
        .p;
    let o = w.apply(&parse_line("stage turn=30 tilt=-20 at=10,0,30").unwrap());
    assert!(o.ok, "{o:?}");
    assert_eq!(
        (w.stage, w.tilt, w.offset),
        (30.0, -20.0, [10.0, 0.0, 30.0])
    );
    // U1 stays where it was on the stage
    let now = w
        .stage_pose()
        .inverse()
        .then(&w.part("U1").unwrap().pose())
        .p;
    assert!(kin::norm(kin::sub(now, local)) < 1e-6);
    assert!(kin::norm(kin::sub(w.part("U1").unwrap().pose().p, before)) > 20.0);
    assert_eq!(
        w.stage_track.last().unwrap()[1..],
        [30.0, -20.0, 10.0, 0.0, 30.0]
    );
    // beyond its reach: refused, nothing changed
    let o = w.apply(&parse_line("stage at=0,0,90").unwrap());
    assert!(
        o.refused.as_deref().unwrap().contains("-30 to 70 mm in z"),
        "{o:?}"
    );
    let o = w.apply(&parse_line("stage").unwrap());
    assert!(
        o.refused
            .as_deref()
            .unwrap()
            .contains("give turn=, tilt= or at="),
        "{o:?}"
    );
}

#[test]
fn present_brings_a_pin_to_a_stations_point() {
    let (mut w, out) = ok("reset circuit=demo
tool A tweezers
pick A U1 grasp=ends
place A U1 at=0,0,-32 rpy=0,0,90
vise close
release A U1
pick A R1 grasp=body
present U1.3.tip at=S1 and=S2");
    assert!(
        out.last().unwrap().events[0].starts_with("U1.3.tip at S1's point"),
        "{:?}",
        out.last()
    );
    let s1 = stations::data()
        .unwrap()
        .into_iter()
        .find(|s| s.name == "S1")
        .unwrap();
    let tip = w.frame("U1.3.tip").unwrap().p;
    assert!(kin::norm(kin::sub(tip, s1.point)) < 1e-6, "{tip:?}");
    // only what the vise holds is presented
    let o = w.apply(&parse_line("present R1.1.tip at=S1").unwrap());
    assert!(
        o.refused
            .as_deref()
            .unwrap()
            .contains("R1 is not on the stage (held by A)"),
        "{o:?}"
    );
}

#[test]
fn a_station_works_only_on_what_is_presented() {
    let (mut w, _) = ok("reset circuit=demo
tool A tweezers
pick A U1 grasp=ends
place A U1 at=0,0,-32 rpy=0,0,90
vise close
release A U1
pick A R1 grasp=body
move A to=40,60,0
align A R1.1.tip to=U1.3.tip gap=0.1");
    let o = w.apply(&parse_line("touch S1 at=U1.3+R1.1").unwrap());
    assert!(
        o.refused
            .as_deref()
            .unwrap()
            .contains("present it there first (present U1.3+R1.1 at=S1)"),
        "{o:?}"
    );
    let o = w.apply(&parse_line("weld S1 at=U1.3+R1.1 energy=8").unwrap());
    assert!(
        o.refused
            .as_deref()
            .unwrap()
            .contains("S1 holds the iron, not the welder"),
        "{o:?}"
    );
    // a station is no arm: it does not pick
    let e = parse_line("pick S1 R1 grasp=body").unwrap_err();
    assert!(e.contains("no arm S1"), "{e}");
}

#[test]
fn soldering_at_stations_makes_a_good_joint() {
    let (w, out) = ok(&program("solder-stations"));
    assert_eq!(result(&w, "U1.3+R1.1").as_deref(), Some("good"));
    assert!(out.iter().any(|o| {
        o.events
            .iter()
            .any(|e| e == "R1 stays: it is soldered to a part the vise holds")
    }));
    assert!(
        w.stations.iter().all(|s| s.ext == 0.0),
        "every station backed away"
    );
}

#[test]
fn welding_at_a_station() {
    let (w, _) = ok(&program("weld-stations"));
    assert_eq!(result(&w, "C1.2+C2.1").as_deref(), Some("welded"));
    assert_eq!(w.part("C2").unwrap().holder.as_deref(), Some("vise"));
}

#[test]
fn the_chosen_layout_runs_every_program() {
    for name in PROGRAMS {
        let (_, out) = run_all(&program(name));
        let bad = out.iter().find(|o| !o.ok);
        assert!(bad.is_none(), "{name}: {bad:?}");
    }
}

/// The candidate layouts tried (the chosen one is `stations.toml`).
pub const LAYOUTS: [(&str, &str); 6] = [
    ("above", "S1 -1,-1,0.8|S2 1,-1,0.4|S3 -1,1,-1.2"),
    ("tray side", "S1 -1,-1,0.8|S2 1,1,-0.8|S3 -1,1,-1.2"),
    ("level", "S1 -1,-1,0|S2 1,-1,0|S3 -1,1,-1.2"),
    (
        "slightly below (chosen)",
        "S1 -1,-1,-0.4|S2 1,-1,-0.4|S3 -1,1,-1.2",
    ),
    ("below", "S1 -1,-1,-1|S2 1,-1,-0.8|S3 -1,1,-1.2"),
    ("under the arms", "S1 0,-1,-1|S2 1,0,-1|S3 -1,1,-1.2"),
];

fn layout_text(spec: &str) -> String {
    let tools = [
        ("S1", "iron", "temp = 350.0\n"),
        ("S2", "solder", ""),
        ("S3", "welder", ""),
    ];
    let mut out = String::from("point = [0.0, 0.0, 0.0]\n");
    for part in spec.split('|') {
        let (name, from) = part.split_once(' ').unwrap();
        let (_, tool, extra) = tools.iter().find(|t| t.0 == name).unwrap();
        out += &format!(
            "[[station]]\nname = \"{name}\"\ntool = \"{tool}\"\nfrom = [{from}]\nstroke = 120.0\n{extra}"
        );
    }
    out
}

const PROGRAMS: [&str; 7] = [
    "arms",
    "pick",
    "solder",
    "weld",
    "claude",
    "solder-stations",
    "weld-stations",
];

fn program(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/../programs/{name}.cell",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

/// Every program with a layout: the first command not ok in each, or
/// none.
fn trial(spec: &str) -> Vec<(String, Option<String>)> {
    stations::try_layout(Some(&layout_text(spec)));
    let out = PROGRAMS
        .iter()
        .map(|name| {
            let (_, out) = run_all(&program(name));
            let bad = out.iter().find(|o| !o.ok).map(|o| {
                format!(
                    "{}: {}",
                    o.command,
                    o.refused
                        .clone()
                        .or_else(|| o.stopped.clone())
                        .unwrap_or_default()
                )
            });
            ((*name).to_string(), bad)
        })
        .collect();
    stations::try_layout(None);
    out
}

/// Every candidate layout on every program (slow: `cargo test --release
/// -p xetal-ext-cell --test stations layouts -- --ignored --nocapture`);
/// its results are on the cell's page.
#[test]
#[ignore]
fn layouts() {
    for (name, spec) in LAYOUTS {
        for (prog, bad) in trial(spec) {
            println!("{name} | {prog} | {}", bad.unwrap_or_else(|| "ok".into()));
        }
    }
}
