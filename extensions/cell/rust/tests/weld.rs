//! Spot welding: a capacitor discharge welds a lap when its energy is in
//! the window of the leads' metals (steel and copper-clad steel well,
//! copper poorly), does not fuse below it and blows out above it; a weld
//! holds two parts as one, as a good solder joint does. A semiconductor
//! the current reaches -- on the lap with no grounding clip, or joined
//! to both sides by other joints -- is refused unless risky=allow, then
//! damaged.

use xetal_ext_cell::command::parse_program;
use xetal_ext_cell::weld::{self, Fused};
use xetal_ext_cell::world::{Outcome, World};

/// At the bench: a ceramic disc in the vise, its leads down; a second
/// disc's lead 1 brought to the first's lead 2, 6 mm from its body, and
/// gripped there by pincer P1; the arm gone; the lap presented to the
/// welder station S3.
const DISCS: &str = "reset circuit=weld
tool A tweezers
pick A C1 grasp=body
place A C1 at=0,0,-20
vise close
release A C1
pick A C2 grasp=body
lift A mm=80
move A to=20.08,0,-5.746
align A C2.1.tip to=C1.2.6 gap=0.1
pinch P1 on=C2.1 at=6
release A C2
lift A mm=60
present C1.2+C2.1 at=S3";

fn run(w: &mut World, program: &str) -> Vec<Outcome> {
    let out = w.apply_all(&parse_program(program).unwrap());
    for o in &out {
        assert!(o.ok, "{o:?}");
    }
    out
}

fn ready(program: &str) -> World {
    let mut w = World::default();
    run(&mut w, program);
    w
}

fn result(w: &World, name: &str) -> Option<String> {
    w.joints
        .iter()
        .find(|j| j.name == name)
        .and_then(|j| j.result.clone())
}

#[test]
fn the_data_reads_and_the_windows_judge() {
    let w = weld::data().unwrap();
    assert_eq!(
        weld::judge(&w, "steel", "steel", 6.0).unwrap().0,
        Fused::Welded("welded")
    );
    assert_eq!(
        weld::judge(&w, "steel", "steel", 1.0).unwrap().0,
        Fused::Cold
    );
    assert_eq!(
        weld::judge(&w, "steel", "steel", 20.0).unwrap().0,
        Fused::Blown
    );
    // a lap of two metals: the overlap of their windows
    assert_eq!(
        weld::window(&w, "steel", "copper-clad-steel").unwrap(),
        Some((5.0, 12.0, "welded"))
    );
    // copper welds weakly; copper to steel not at all
    assert_eq!(
        weld::judge(&w, "copper", "copper", 16.0).unwrap().0,
        Fused::Welded("weak")
    );
    assert_eq!(weld::window(&w, "copper", "steel").unwrap(), None);
    assert_eq!(
        weld::judge(&w, "copper", "steel", 10.0).unwrap().0,
        Fused::Cold
    );
    assert_eq!(
        weld::judge(&w, "copper", "steel", 13.0).unwrap().0,
        Fused::Blown
    );
    // E = C V^2 / 2
    assert!((weld::volts(&w, 5.0) - 10.0).abs() < 1e-9);
}

#[test]
fn a_weld_in_the_window_holds_two_parts_as_one() {
    let mut w = ready(DISCS);
    let o = run(&mut w, "weld S3 at=C1.2+C2.1 energy=8");
    assert_eq!(result(&w, "C1.2+C2.1").as_deref(), Some("welded"), "{o:?}");
    assert!(
        o[0].events
            .iter()
            .any(|e| e.contains("in the window of 5 to 12 J")),
        "{o:?}"
    );
    // the pincer lets go, the disc stays: welded to the one the vise holds
    let o = run(&mut w, "unpinch P1");
    assert!(o[0].events.iter().any(|e| e.contains("C2 stays")), "{o:?}");
    assert_eq!(w.part("C2").unwrap().holder.as_deref(), Some("vise"));
    assert!(!w.part("C1").unwrap().damaged && !w.part("C2").unwrap().damaged);
}

#[test]
fn too_little_energy_does_not_fuse_and_too_much_blows_out() {
    let mut w = ready(DISCS);
    let o = run(&mut w, "weld S3 at=C1.2+C2.1 energy=2");
    assert_eq!(result(&w, "C1.2+C2.1"), None);
    assert!(
        o[0].warnings.iter().any(|x| x.contains("did not weld")),
        "{o:?}"
    );
    // it can be welded again, and too hard it blows out, both parts damaged
    run(&mut w, "weld S3 at=C1.2+C2.1 energy=30");
    assert_eq!(result(&w, "C1.2+C2.1").as_deref(), Some("blown"));
    assert!(w.part("C1").unwrap().damaged && w.part("C2").unwrap().damaged);
    // a blown lap holds nothing: the pincer lets go, C2 falls
    let o = run(&mut w, "unpinch P1");
    assert!(!o[0].events.iter().any(|e| e.contains("C2 stays")), "{o:?}");
}

#[test]
fn the_welder_refuses_what_it_cannot_give_or_a_soldered_lap() {
    let mut w = ready(DISCS);
    let o = w.apply_all(&parse_program("weld S3 at=C1.2+C2.1 energy=90").unwrap());
    assert!(
        o[0].refused.as_deref().unwrap().contains("at most 80 J"),
        "{o:?}"
    );
    let o = w.apply_all(&parse_program("weld A at=C1.2+C2.1 energy=8").unwrap());
    assert!(
        o[0].refused.as_deref().unwrap().contains("not the welder"),
        "{o:?}"
    );
}

/// The LED's lead 1 (steel) laid to the disc's lead, as `DISCS` lays C2's;
/// P1 grips its other lead, so nothing grounds the lap's.
fn led() -> String {
    DISCS
        .replace("pick A C2 grasp=body", "pick A D1 grasp=body")
        .replace("move A to=20.08,0,-5.746", "move A to=18.81,0,3.554")
        .replace("align A C2.1.tip", "align A D1.1.tip")
        .replace("pinch P1 on=C2.1 at=6", "pinch P1 on=D1.2 at=6")
        .replace("release A C2", "release A D1")
        .replace("present C1.2+C2.1", "present C1.2+D1.1")
}

#[test]
fn a_semiconductor_on_the_lap_is_refused_unless_allowed() {
    let mut w = ready(&led());
    let o = w.apply_all(&parse_program("weld S3 at=C1.2+D1.1 energy=8").unwrap());
    let why = o[0].refused.as_deref().unwrap_or("");
    assert!(
        why.contains("D1 is a semiconductor on the lap with no grounding clip on D1.1"),
        "{o:?}"
    );
    assert!(why.contains("risky=allow"), "{why}");
    assert_eq!(result(&w, "C1.2+D1.1"), None, "refused: nothing changed");
    // allowed: welded, and the LED damaged by the transient
    let o = run(&mut w, "weld S3 at=C1.2+D1.1 energy=8 risky=allow");
    assert_eq!(result(&w, "C1.2+D1.1").as_deref(), Some("welded"));
    assert!(w.part("D1").unwrap().damaged);
    assert!(
        !w.part("C1").unwrap().damaged,
        "a disc is not a semiconductor"
    );
    assert!(
        o[0].warnings
            .iter()
            .any(|x| x.contains("D1 is damaged by the transient")),
        "{o:?}"
    );
}

#[test]
fn a_grounding_clip_between_the_body_and_the_lap_makes_it_safe() {
    let mut w = ready(&led());
    let o = run(
        &mut w,
        "pinch P2 on=D1.1 at=8\nweld S3 at=C1.2+D1.1 energy=8",
    );
    assert_eq!(result(&w, "C1.2+D1.1").as_deref(), Some("welded"), "{o:?}");
    assert!(!w.part("D1").unwrap().damaged);
}

#[test]
fn a_semiconductor_joined_to_both_sides_is_in_the_path() {
    // C1 and C2 already joined through D1 (C1.2 to D1.1, D1.2 to C2.2):
    // the discharge across C1.2+C2.1 has a second way, through the LED
    let mut w = ready(DISCS);
    for (a, b) in [("C1.1", "D1.1"), ("D1.2", "C2.2")] {
        w.joints.push(xetal_ext_cell::solder::Joint {
            name: format!("{a}+{b}"),
            a: a.into(),
            b: b.into(),
            node_a: 0,
            node_b: 0,
            at: [0.0; 3],
            solder: 0.0,
            alloy: String::new(),
            temp: 25.0,
            molten: false,
            wet_a: 0.0,
            wet_b: 0.0,
            flux: 0.0,
            dry: 0.0,
            moved: false,
            result: Some("welded".into()),
            set_at: Some(0.0),
            track: Vec::new(),
            weld: Some(8.0),
            lap: 0.0,
            nodes_a: Vec::new(),
            nodes_b: Vec::new(),
            turns: 0.0,
        });
    }
    let o = w.apply_all(&parse_program("weld S3 at=C1.2+C2.1 energy=8").unwrap());
    let why = o[0].refused.as_deref().unwrap_or("");
    assert!(
        why.contains("D1 is a semiconductor in a path from electrode to electrode"),
        "{o:?}"
    );
}

#[test]
fn copper_to_steel_does_not_weld_and_a_weld_inspects() {
    // the resistor's copper lead to the disc's copper-clad steel: no
    // overlap of windows (5 to 12 J and 15 to 20 J), so it does not weld
    let prog = DISCS
        .replace("pick A C2 grasp=body", "pick A R1 grasp=body")
        .replace("align A C2.1.tip", "align A R1.1.tip")
        .replace("pinch P1 on=C2.1", "pinch P1 on=R1.1")
        .replace("release A C2", "release A R1")
        .replace("present C1.2+C2.1", "present C1.2+R1.1");
    let mut w = ready(&prog);
    let o = run(&mut w, "weld S3 at=C1.2+R1.1 energy=10");
    assert!(o[0].events.iter().any(|e| e.contains("no window")), "{o:?}");
    let o = run(&mut w, "inspect C1.2+R1.1");
    assert!(o[0].events[0].contains("not welded"), "{o:?}");
    // a good weld says so
    let mut w = ready(DISCS);
    run(&mut w, "weld S3 at=C1.2+C2.1 energy=8");
    let o = run(&mut w, "inspect C1.2+C2.1");
    assert_eq!(o[0].events[0], "C1.2+C2.1: welded by a 8 J discharge");
}

#[test]
fn the_weld_program_joins_three_parts_held_as_one() {
    let mut w = World::default();
    let out = run(&mut w, include_str!("../../programs/weld.cell"));
    assert_eq!(result(&w, "C1.2+C2.1").as_deref(), Some("welded"));
    assert_eq!(result(&w, "C1.1+D1.1").as_deref(), Some("welded"));
    assert!(out.iter().any(|o| {
        o.events
            .iter()
            .any(|e| e == "D1 stays: it is welded to a part the vise holds")
    }));
    for id in ["C1", "C2", "D1"] {
        let p = w.part(id).unwrap();
        assert!(!p.damaged && !p.lost, "{id}");
        assert_eq!(
            p.holder.as_deref(),
            Some("vise"),
            "{id} is held with the rest"
        );
    }
    assert!(w.clashes().is_empty(), "{:?}", w.clashes());
    for (a, b, _) in w.leads_near(0.0) {
        assert!(
            w.joints
                .iter()
                .any(|j| (j.a == a && j.b == b) || (j.a == b && j.b == a)),
            "a short: {a} {b}"
        );
    }
}
