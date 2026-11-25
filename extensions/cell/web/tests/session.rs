//! The page's session, run natively: a reset lays out the tray, the
//! offered programs run clean, commands come back as outcomes with
//! their times, refusals and stops say why, the scene is the cell's,
//! and X_eTaL in the page reaches the cell.

use serde_json::Value;

fn json(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

#[test]
fn a_session() {
    let r = json(&cell_web::reset("demo"));
    assert_eq!(r["outcomes"][0]["ok"], true);
    let s = json(&cell_web::state());
    assert_eq!(s["parts"].as_array().unwrap().len(), 4);
    assert_eq!(
        s["parts"][0]["grasps"],
        serde_json::json!(["ends", "sides"])
    );
    let o = json(&cell_web::command("tool A tweezers\npick A U1 grasp=ends"));
    let outs = o["outcomes"].as_array().unwrap();
    assert!(outs.iter().all(|x| x["ok"] == true), "{o}");
    assert!(o["t1"].as_f64().unwrap() > o["t0"].as_f64().unwrap());
    assert_eq!(json(&cell_web::state())["arms"][0]["holding"], "U1");
    // a refusal says why and changes nothing
    let r = json(&cell_web::command("pick B R1 grasp=body"));
    assert!(
        r["outcomes"][0]["refused"]
            .as_str()
            .unwrap()
            .contains("not tweezers")
    );
    // not commands at all
    assert!(
        json(&cell_web::command("fly A"))["error"]
            .as_str()
            .unwrap()
            .contains("no command fly")
    );
    // the scene is the cell's
    let calls = cell_web::scene("live");
    assert!(calls.starts_with("cell('live', 1280, 720)"));
    assert!(calls.contains("step(3, 'pick A U1 grasp=ends'"));
}

#[test]
fn the_offered_programs_run_clean() {
    for name in ["solder", "pick", "arms"] {
        let _ = cell_web::reset("none");
        let o = json(&cell_web::command(&cell_web::program(name)));
        for x in o["outcomes"].as_array().unwrap() {
            assert_eq!(x["ok"], true, "{name}: {x}");
        }
    }
    assert_eq!(cell_web::program("nothing"), "");
}

#[test]
fn xetal_in_the_page_reaches_the_cell() {
    assert!(cell_web::install() >= 10);
    let o = json(&cell_web::xetal(cell_web::EXAMPLE));
    assert_eq!(o["err"], "", "{o}");
    let out = o["out"].as_str().unwrap();
    assert!(out.starts_with("cell('from X_eTaL', 1280, 720)"), "{out}");
    assert!(out.contains("pick D C1 grasp=body"));
    assert!(cell_web::rendered(cell_web::EXAMPLE).contains("<span"));
}
