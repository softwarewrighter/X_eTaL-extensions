//! The command table: text and JSON parse to the same commands, the
//! canonical text round-trips, every limit is checked, and the docs and
//! schemas are made from the table.

use xetal_ext_cell::command::{
    self, COMMANDS, Command, Value, parse_json, parse_line, parse_program,
};

fn round_trip(line: &str) -> Command {
    let c = parse_line(line).unwrap();
    assert_eq!(parse_line(&c.to_text()).unwrap(), c, "text: {line}");
    assert_eq!(parse_json(&c.to_json()).unwrap(), c, "json: {line}");
    c
}

#[test]
fn the_design_examples_parse() {
    let program = "
reset circuit=blinker                      # trays filled, arms home
tool A tweezers
pick A U1 grasp=ends
place A U1 at=0,0,0 rpy=180,0,0
pick B R1 grasp=body
align B R1.1.tip to=U1.3.tip gap=0.1
tool D clip
clip D on=U1.3 at=2
tool C iron temp=350
touch C at=U1.3+R1.1 dwell=1.5
feed D mm=1.2 rate=2
lift C
wait until=U1.3+R1.1 below=150
release B R1
scope zoom=40 aim=U1.3+R1.1 ; snap
inspect U1.3+R1.1
joints A 0 -90 90 0 45 0
move A to=10,20,30 rpy=0,90,0 speed=50
grip A close force=0.5
stage turn=90
vise close
weld C at=Q1.1+R2.2 energy=2 risky=allow
wire D awg=26 mm=40
cut D
probe U1.3 R1.2
rewind t=12.5
back C mm=20
";
    let p = parse_program(program).unwrap();
    assert_eq!(p.len(), 28);
    for c in &p {
        round_trip(&c.to_text());
    }
    assert_eq!(p[3].to_text(), "place A U1 at=0,0,0 rpy=180,0,0");
    assert_eq!(
        p[17].get("angles"),
        Some(&Value::Numbers(vec![0.0, -90.0, 90.0, 0.0, 45.0, 0.0]))
    );
    assert_eq!(p[17].to_text(), "joints A 0 -90 90 0 45 0");
}

#[test]
fn every_command_has_an_example() {
    // the examples above cover the whole table
    let covered = [
        "reset", "tool", "joints", "move", "lift", "back", "grip", "pick", "place", "align",
        "release", "clip", "touch", "feed", "weld", "wire", "cut", "stage", "vise", "wait",
        "scope", "snap", "inspect", "probe", "rewind",
    ];
    let ops: Vec<&str> = COMMANDS.iter().map(|s| s.op).collect();
    assert_eq!(ops, covered);
}

#[test]
fn canonical_text_orders_and_shortens() {
    let c = round_trip("move  A speed=50.0 to=10,0.50,-0 dry=1");
    assert_eq!(c.to_text(), "move A to=10,0.5,0 speed=50 dry=1");
    assert!(c.dry);
    assert_eq!(
        c.to_json(),
        serde_json::json!({"op": "move", "arm": "A", "to": [10.0, 0.5, -0.0], "speed": 50.0, "dry": true})
    );
    let w = round_trip("wire D mm=40 awg=26");
    assert_eq!(w.to_json()["awg"], serde_json::json!(26));
}

#[test]
fn json_is_the_same_command() {
    let j = serde_json::json!({"op": "align", "arm": "B", "frame": "R1.1.tip", "to": "U1.3.tip", "gap": 0.1});
    assert_eq!(
        parse_json(&j).unwrap(),
        parse_line("align B R1.1.tip to=U1.3.tip gap=0.1").unwrap()
    );
    let p = parse_program("{\"op\": \"vise\", \"action\": \"close\"}\nvise close").unwrap();
    assert_eq!(p[0], p[1]);
}

#[test]
fn wrong_commands_say_what_is_wrong() {
    let says = |line: &str, msg: &str| {
        let e = parse_line(line).unwrap_err();
        assert!(e.contains(msg), "{line}: {e}");
    };
    says("fly A", "no command fly");
    says("tool E iron", "no arm E (arms: A B C D)");
    says("tool A hammer", "no tool hammer");
    says("tool A iron temp=900", "temp: 900 is outside 150 to 450 C");
    says("tool A", "tool: tool is required");
    says("move A", "move: to is required");
    says("move A to=1,2", "is not three numbers");
    says(
        "move A to=1,2,3 color=red",
        "no argument color= (to= rpy= speed=)",
    );
    says("move A to=1,2,3 to=1,2,3", "to= given twice");
    says("joints A 1 2 3", "six angles needed, 3 given");
    says("lift A B", "too many words: B");
    says("pick A u1 grasp=ends", "u1 is not a part id");
    says("clip D on=U1 at=2", "U1 is not a pin");
    says("touch C at=U1.3", "is not a joint");
    says("touch C at=U1.3+U1.3", "is not a joint");
    says("wire D awg=27.5 mm=3", "not a whole number from 22 to 34");
    says("vise close dry=yes", "dry: yes is not 0 or 1");
    says("stage turn=nan", "not a finite number");
    let j = parse_json(&serde_json::json!({"op": "lift", "arm": "A", "height": 3})).unwrap_err();
    assert!(j.contains("no argument height"), "{j}");
    let e = parse_program("vise close\n\nlift Z").unwrap_err();
    assert!(e.starts_with("line 3: lift: arm: no arm Z"), "{e}");
}

#[test]
fn docs_and_schemas_come_from_the_table() {
    let md = command::markdown();
    assert_eq!(md.lines().count(), COMMANDS.len() + 2);
    assert!(md.contains("| `move ARM to=X,Y,Z [rpy=R,P,Y] [speed=MM/S]` | arm |"));
    assert!(md.contains("| `grip ARM open\\|close [force=N]` | tweezers |"));
    let page = include_str!("../../docs/README.md");
    assert!(
        page.contains(&md),
        "docs/README.md's command table is not the table's: paste command::markdown()"
    );
    let list = command::list();
    assert_eq!(list.lines().count(), COMMANDS.len());
    let schemas = command::schemas();
    let move_ = &schemas[3];
    assert_eq!(move_["properties"]["op"]["const"], "move");
    assert_eq!(move_["required"], serde_json::json!(["op", "arm", "to"]));
    assert_eq!(move_["properties"]["speed"]["maximum"], 500.0);
    assert_eq!(
        move_["properties"]["to"]["description"],
        "the tool point, in the cell's frame (mm)"
    );
    assert_eq!(move_["additionalProperties"], false);
}
