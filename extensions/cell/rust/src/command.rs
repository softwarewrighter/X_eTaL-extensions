//! The command language: one table of commands, their arguments, units
//! and limits, from which the text parser, the JSON parser, the MCP
//! tools' schemas and the command list in the docs are all made -- a
//! command added here appears everywhere at once.
//!
//! As text, one command a line (or several on a line, split by `;`), a
//! `#` starting a comment: `move A to=10,0,5 speed=50`. As JSON, one
//! object: `{"op": "move", "arm": "A", "to": [10, 0, 5], "speed": 50}`.
//! Positional arguments in text are named arguments in JSON. Any command
//! takes `dry=1` (`"dry": true`): run it on a copy, change nothing.

use serde_json::{Map, Value as Json, json};
use std::fmt::Write as _;

/// The arms, by name.
pub const ARMS: [&str; 1] = ["A"];
/// The tools in the rack, by name.
pub const TOOLS: [&str; 7] = [
    "tweezers", "iron", "solder", "clip", "welder", "cutter", "wire",
];

/// What an argument's value may be.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    /// An arm: `A` to `D`.
    Arm,
    /// An arm or a tool station (`S1`).
    Actor,
    /// A tool station: `S1`.
    Station,
    /// A pincer on the turntable: `P1`.
    Pincer,
    /// A frame or a joint: what the positioner presents.
    Spot,
    /// A tool in the rack.
    Tool,
    /// A part's id: a capital letter, then letters, digits or `_`: `U1`.
    Part,
    /// A pin: `U1.3`.
    Pin,
    /// A frame: a part (`U1`), a pin (`U1.3`) or a pin's tip (`U1.3.tip`).
    Frame,
    /// A joint, two pins: `U1.3+R1.1`.
    Joint,
    /// A pin or a joint.
    Target,
    /// A free name: a circuit's, a grasp frame's.
    Name,
    /// One of the words given.
    Choice(&'static [&'static str]),
    /// A number in a unit, between limits (inclusive).
    Number(&'static str, f64, f64),
    /// A whole number between limits.
    Int(i64, i64),
    /// Three numbers in a unit: `x,y,z` in text, `[x, y, z]` in JSON.
    Vec3(&'static str),
    /// Six joint angles, degrees: six numbers in text, an array in JSON.
    Angles,
}

/// One argument of a command.
#[derive(Clone, Copy, Debug)]
pub struct Arg {
    /// Its name: in text, `name=value` (named) or a place (positional).
    pub name: &'static str,
    /// What it may be.
    pub kind: Kind,
    /// Must be given.
    pub required: bool,
    /// What it is.
    pub doc: &'static str,
}

/// One command.
#[derive(Clone, Copy, Debug)]
pub struct Spec {
    /// Its name, the line's first word.
    pub op: &'static str,
    /// Its family, for typed tools and the docs' grouping.
    pub family: &'static str,
    /// What it does.
    pub summary: &'static str,
    /// Its positional arguments, in order.
    pub positional: &'static [Arg],
    /// Its named arguments.
    pub named: &'static [Arg],
}

const fn req(name: &'static str, kind: Kind, doc: &'static str) -> Arg {
    Arg {
        name,
        kind,
        required: true,
        doc,
    }
}

const fn opt(name: &'static str, kind: Kind, doc: &'static str) -> Arg {
    Arg {
        name,
        kind,
        required: false,
        doc,
    }
}

const ARM: Arg = req("arm", Kind::Arm, "the arm");
const ACTOR: Arg = req("arm", Kind::Actor, "the arm, or a tool station");
const OPEN_CLOSE: &[&str] = &["open", "close"];

/// The command table: the one source of the language.
pub const COMMANDS: &[Spec] = &[
    Spec {
        op: "reset",
        family: "cell",
        summary: "the cell as it starts: arms home, tools in the rack, the trays filled for a circuit",
        positional: &[],
        named: &[opt(
            "circuit",
            Kind::Name,
            "the circuit whose parts fill the trays",
        )],
    },
    Spec {
        op: "tool",
        family: "arm",
        summary: "change the arm's tool at the rack",
        positional: &[ARM, req("tool", Kind::Tool, "the tool to take")],
        named: &[opt(
            "temp",
            Kind::Number("C", 150.0, 450.0),
            "the iron's set point",
        )],
    },
    Spec {
        op: "joints",
        family: "arm",
        summary: "move the arm to absolute joint angles, all joints arriving together",
        positional: &[ARM, req("angles", Kind::Angles, "j1 to j6, degrees")],
        named: &[opt(
            "speed",
            Kind::Number("fraction", 0.01, 1.0),
            "of each joint's top speed",
        )],
    },
    Spec {
        op: "move",
        family: "arm",
        summary: "move the tool to a pose, by inverse kinematics, in a straight line",
        positional: &[ARM],
        named: &[
            req(
                "to",
                Kind::Vec3("mm"),
                "the tool point, in the cell's frame",
            ),
            opt("rpy", Kind::Vec3("deg"), "the tool's roll, pitch and yaw"),
            opt("speed", Kind::Number("mm/s", 1.0, 500.0), "along the line"),
        ],
    },
    Spec {
        op: "lift",
        family: "arm",
        summary: "move the tool straight up",
        positional: &[ARM],
        named: &[opt(
            "mm",
            Kind::Number("mm", 0.0, 100.0),
            "how far (10 if not given)",
        )],
    },
    Spec {
        op: "back",
        family: "arm",
        summary: "move the tool straight back along its own axis, away from what it touches",
        positional: &[ACTOR],
        named: &[opt(
            "mm",
            Kind::Number("mm", 0.0, 100.0),
            "how far (30 if not given)",
        )],
    },
    Spec {
        op: "grip",
        family: "tweezers",
        summary: "open or close the tweezers",
        positional: &[
            ARM,
            req("action", Kind::Choice(OPEN_CLOSE), "open or close"),
        ],
        named: &[opt(
            "force",
            Kind::Number("N", 0.0, 5.0),
            "the closing force's limit",
        )],
    },
    Spec {
        op: "pick",
        family: "tweezers",
        summary: "approach a part, close on a grasp frame, lift it",
        positional: &[ARM, req("part", Kind::Part, "the part")],
        named: &[req("grasp", Kind::Name, "the part kind's grasp frame")],
    },
    Spec {
        op: "place",
        family: "tweezers",
        summary: "carry the held part to a pose",
        positional: &[ARM, req("part", Kind::Part, "the held part")],
        named: &[
            req(
                "at",
                Kind::Vec3("mm"),
                "the part's origin, in the cell's frame",
            ),
            opt("rpy", Kind::Vec3("deg"), "the part's roll, pitch and yaw"),
        ],
    },
    Spec {
        op: "align",
        family: "tweezers",
        summary: "bring a frame of the held part to another frame, a gap apart",
        positional: &[ARM, req("frame", Kind::Frame, "the held part's frame")],
        named: &[
            req("to", Kind::Frame, "the frame it goes to"),
            opt(
                "gap",
                Kind::Number("mm", 0.0, 5.0),
                "left between them (0.1 if not given)",
            ),
        ],
    },
    Spec {
        op: "release",
        family: "tweezers",
        summary: "open the tweezers and let the part go",
        positional: &[ARM, req("part", Kind::Part, "the held part")],
        named: &[],
    },
    Spec {
        op: "clip",
        family: "clip",
        summary: "clip a heatsink on a lead, between the body and the joint",
        positional: &[ARM],
        named: &[
            req("on", Kind::Pin, "the pin whose lead it clips"),
            req(
                "at",
                Kind::Number("mm", 0.0, 30.0),
                "from the body, along the lead",
            ),
        ],
    },
    Spec {
        op: "touch",
        family: "iron",
        summary: "put the iron's tip on a joint and hold it there",
        positional: &[ACTOR],
        named: &[
            req("at", Kind::Joint, "the joint"),
            opt(
                "dwell",
                Kind::Number("s", 0.0, 30.0),
                "how long (1 if not given)",
            ),
            opt(
                "temp",
                Kind::Number("C", 150.0, 450.0),
                "a station's iron set to this first (an arm's iron is set by tool ... temp=)",
            ),
        ],
    },
    Spec {
        op: "feed",
        family: "solder",
        summary: "feed solder wire into the joint the iron is on",
        positional: &[ACTOR],
        named: &[
            req("mm", Kind::Number("mm", 0.0, 20.0), "of solder wire"),
            opt(
                "rate",
                Kind::Number("mm/s", 0.1, 20.0),
                "how fast (2 if not given)",
            ),
        ],
    },
    Spec {
        op: "weld",
        family: "welder",
        summary: "spot weld a lap of two leads by a capacitor discharge",
        positional: &[ACTOR],
        named: &[
            req("at", Kind::Joint, "the lap"),
            req(
                "energy",
                Kind::Number("J", 0.1, 100.0),
                "the discharge's energy",
            ),
            opt(
                "risky",
                Kind::Choice(&["refuse", "allow"]),
                "a semiconductor in the path: refused unless allow",
            ),
        ],
    },
    Spec {
        op: "wire",
        family: "wire",
        summary: "the wire feeder station feeds bare or enameled wire from its spool out of its nozzle",
        positional: &[req("arm", Kind::Station, "the wire feeder station")],
        named: &[
            req("awg", Kind::Int(22, 34), "the gauge"),
            req("mm", Kind::Number("mm", 1.0, 120.0), "how much"),
            opt(
                "enamel",
                Kind::Choice(&["no", "yes"]),
                "enameled (magnet) wire: no if not given",
            ),
            opt(
                "metal",
                Kind::Choice(&["copper", "steel"]),
                "copper if not given; steel welds well, for supports",
            ),
            opt(
                "support",
                Kind::Choice(&["no", "yes"]),
                "a sacrificial support, cut away later: no if not given",
            ),
        ],
    },
    Spec {
        op: "cut",
        family: "wire",
        summary: "the wire feeder cuts its wire at the nozzle: what holds the free end keeps it",
        positional: &[req("arm", Kind::Station, "the wire feeder station")],
        named: &[],
    },
    Spec {
        op: "stage",
        family: "stage",
        summary: "move the positioner: turn the stage by an angle, tilt it, move it",
        positional: &[],
        named: &[
            opt(
                "turn",
                Kind::Number("deg", -360.0, 360.0),
                "by, counterclockwise seen from above",
            ),
            opt("tilt", Kind::Number("deg", -90.0, 90.0), "to, about x"),
            opt(
                "at",
                Kind::Vec3("mm"),
                "to, the stage's top from its place at rest",
            ),
        ],
    },
    Spec {
        op: "present",
        family: "stage",
        summary: "move the positioner so a pin's frame or a joint is at a tool station's point",
        positional: &[req(
            "target",
            Kind::Spot,
            "a frame (U1.3.tip) or a joint (U1.3+R1.1) on the stage",
        )],
        named: &[
            req("at", Kind::Station, "the station"),
            opt(
                "and",
                Kind::Station,
                "a second station that works there too (its line kept clear)",
            ),
            opt(
                "turn",
                Kind::Number("deg", -360.0, 360.0),
                "the stage's turn (searched if not given)",
            ),
            opt(
                "tilt",
                Kind::Number("deg", -90.0, 90.0),
                "the stage's tilt (searched if not given)",
            ),
        ],
    },
    Spec {
        op: "pinch",
        family: "pincer",
        summary: "a pincer on the turntable grips a lead from the side: it holds the part there, and sinks heat",
        positional: &[req("pincer", Kind::Pincer, "the pincer")],
        named: &[
            req("on", Kind::Pin, "the pin whose lead it grips"),
            opt(
                "at",
                Kind::Number("mm", 0.0, 30.0),
                "from the body, along the lead (5 if not given)",
            ),
        ],
    },
    Spec {
        op: "unpinch",
        family: "pincer",
        summary: "a pincer lets go and draws back",
        positional: &[req("pincer", Kind::Pincer, "the pincer")],
        named: &[],
    },
    Spec {
        op: "bend",
        family: "pincer",
        summary: "a pincer bends a lead round a mandrel, the rest of it toward a direction",
        positional: &[req("pincer", Kind::Pincer, "the pincer")],
        named: &[
            req("on", Kind::Pin, "the pin whose lead it bends"),
            req(
                "at",
                Kind::Number("mm", 0.0, 40.0),
                "from the body, along the lead",
            ),
            req("angle", Kind::Number("deg", -180.0, 180.0), "how far"),
            req(
                "toward",
                Kind::Vec3("dir"),
                "the direction the rest turns toward (the cell's frame)",
            ),
            opt(
                "radius",
                Kind::Number("mm", 0.1, 10.0),
                "the mandrel's (the lead's diameter if not given)",
            ),
        ],
    },
    Spec {
        op: "trim",
        family: "cutter",
        summary: "a cutter station cuts a lead, what is past the cut falling into the bin",
        positional: &[req("arm", Kind::Station, "the cutter station")],
        named: &[
            req("on", Kind::Pin, "the pin whose lead it cuts"),
            req(
                "at",
                Kind::Number("mm", 0.0, 40.0),
                "from the body, along the lead",
            ),
        ],
    },
    Spec {
        op: "insert",
        family: "bender",
        summary: "the arm slides a held part's lead into the bending rig, at a distance from the body at its mandrel",
        positional: &[ARM],
        named: &[
            req("on", Kind::Pin, "the pin whose lead goes in"),
            opt(
                "at",
                Kind::Number("mm", 0.0, 30.0),
                "from the body, along the lead (3 if not given)",
            ),
            opt(
                "roll",
                Kind::Number("deg", -180.0, 180.0),
                "the part turned about the lead: the bend square to one made at 0",
            ),
        ],
    },
    Spec {
        op: "form",
        family: "bender",
        summary: "the bending rig clamps the lead in it and bends it down round its mandrel, in the rig's plane",
        positional: &[],
        named: &[
            opt(
                "angle",
                Kind::Number("deg", 1.0, 135.0),
                "how far (90 if not given)",
            ),
            opt(
                "radius",
                Kind::Number("mm", 0.2, 5.0),
                "the bend's inner radius (the lead's diameter, at least 1 mm, if not given)",
            ),
        ],
    },
    Spec {
        op: "shear",
        family: "bender",
        summary: "the arm puts a held part's lead through the rig's shear, which cuts it; what is past the cut falls into the bin",
        positional: &[ARM],
        named: &[
            req("on", Kind::Pin, "the pin whose lead it cuts"),
            req(
                "at",
                Kind::Number("mm", 0.0, 40.0),
                "from the body, along the lead",
            ),
        ],
    },
    Spec {
        op: "vise",
        family: "stage",
        summary: "open or close the vise on the stage",
        positional: &[req("action", Kind::Choice(OPEN_CLOSE), "open or close")],
        named: &[],
    },
    Spec {
        op: "wait",
        family: "cell",
        summary: "let time pass, or wait until a pin or joint cools",
        positional: &[],
        named: &[
            opt("s", Kind::Number("s", 0.0, 3600.0), "how long"),
            opt("until", Kind::Target, "the pin or joint to watch"),
            opt(
                "below",
                Kind::Number("C", 0.0, 500.0),
                "the temperature to wait for",
            ),
        ],
    },
    Spec {
        op: "scope",
        family: "scope",
        summary: "move the microscope: pan, tilt, zoom, focus, rotate, or aim it at a target",
        positional: &[],
        named: &[
            opt(
                "pan",
                Kind::Number("deg", -180.0, 180.0),
                "about the vertical",
            ),
            opt(
                "tilt",
                Kind::Number("deg", -90.0, 90.0),
                "from the horizontal",
            ),
            opt("zoom", Kind::Number("x", 1.0, 200.0), "the magnification"),
            opt(
                "focus",
                Kind::Number("mm", -30.0, 30.0),
                "the focal plane's offset",
            ),
            opt(
                "rotate",
                Kind::Number("deg", -180.0, 180.0),
                "the picture about its axis",
            ),
            opt("aim", Kind::Target, "center the view on a pin or joint"),
        ],
    },
    Spec {
        op: "snap",
        family: "scope",
        summary: "take the microscope's picture",
        positional: &[],
        named: &[opt("stereo", Kind::Int(0, 1), "1: both lenses' pictures")],
    },
    Spec {
        op: "inspect",
        family: "scope",
        summary: "report a joint's gap, fillet and result, or a pin's temperature",
        positional: &[req("target", Kind::Target, "the pin or joint")],
        named: &[],
    },
    Spec {
        op: "probe",
        family: "scope",
        summary: "measure continuity and resistance between two pins",
        positional: &[
            req("from", Kind::Pin, "one pin"),
            req("to", Kind::Pin, "the other"),
        ],
        named: &[],
    },
    Spec {
        op: "rewind",
        family: "cell",
        summary: "put the cell back as it was at a time (the timeline keeps both branches)",
        positional: &[],
        named: &[req("t", Kind::Number("s", 0.0, 1.0e9), "the time")],
    },
];

/// An argument's value.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// A name or a word.
    Word(String),
    /// A number (an Int's too).
    Number(f64),
    /// Three numbers, or six angles.
    Numbers(Vec<f64>),
}

/// A command, checked against its spec: its arguments in the spec's
/// order (positional, then named), only those given.
#[derive(Clone, Debug, PartialEq)]
pub struct Command {
    /// The command's name.
    pub op: String,
    /// Its arguments.
    pub args: Vec<(String, Value)>,
    /// Run on a copy, changing nothing.
    pub dry: bool,
}

impl Command {
    /// An argument by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.args.iter().find(|(n, _)| n == name).map(|(_, v)| v)
    }

    /// A word argument.
    #[must_use]
    pub fn word(&self, name: &str) -> Option<&str> {
        match self.get(name) {
            Some(Value::Word(w)) => Some(w),
            _ => None,
        }
    }

    /// A number argument.
    #[must_use]
    pub fn number(&self, name: &str) -> Option<f64> {
        match self.get(name) {
            Some(Value::Number(x)) => Some(*x),
            _ => None,
        }
    }

    /// The command's spec.
    ///
    /// # Panics
    ///
    /// Never: a Command is only made from a spec.
    #[must_use]
    pub fn spec(&self) -> &'static Spec {
        spec(&self.op).expect("a command's op is in the table")
    }

    /// The command as canonical text: positional arguments, then named
    /// ones, numbers in their shortest form.
    #[must_use]
    pub fn to_text(&self) -> String {
        let s = self.spec();
        let mut out = self.op.clone();
        for (name, v) in &self.args {
            let positional = s.positional.iter().any(|a| a.name == name);
            out.push(' ');
            if !positional {
                out.push_str(name);
                out.push('=');
            }
            match v {
                Value::Word(w) => out.push_str(w),
                Value::Number(x) => out.push_str(&num(*x)),
                Value::Numbers(xs) => {
                    let sep = if positional { " " } else { "," };
                    out.push_str(&xs.iter().map(|x| num(*x)).collect::<Vec<_>>().join(sep));
                }
            }
        }
        if self.dry {
            out.push_str(" dry=1");
        }
        out
    }

    /// The command as JSON: `{"op": ..., each argument by name}`.
    #[must_use]
    pub fn to_json(&self) -> Json {
        let mut m = Map::new();
        m.insert("op".into(), json!(self.op));
        let s = self.spec();
        for (name, v) in &self.args {
            let int = matches!(arg(s, name).map(|a| a.kind), Some(Kind::Int(..)));
            let j = match v {
                Value::Word(w) => json!(w),
                #[allow(clippy::cast_possible_truncation)] // an Int's value is whole
                Value::Number(x) if int => json!(*x as i64),
                Value::Number(x) => json!(x),
                Value::Numbers(xs) => json!(xs),
            };
            m.insert(name.clone(), j);
        }
        if self.dry {
            m.insert("dry".into(), json!(true));
        }
        Json::Object(m)
    }
}

/// A number in its shortest form: `350`, `0.1`, `-2.5`.
#[must_use]
pub fn num(x: f64) -> String {
    let x = if x == 0.0 { 0.0 } else { x }; // no -0
    format!("{x}")
}

/// A command's spec by name.
#[must_use]
pub fn spec(op: &str) -> Option<&'static Spec> {
    COMMANDS.iter().find(|s| s.op == op)
}

fn arg<'a>(s: &'a Spec, name: &str) -> Option<&'a Arg> {
    s.positional.iter().chain(s.named).find(|a| a.name == name)
}

fn is_id(t: &str) -> bool {
    let mut c = t.chars();
    c.next().is_some_and(|f| f.is_ascii_uppercase())
        && c.all(|x| x.is_ascii_alphanumeric() || x == '_')
}

fn is_pin(t: &str) -> bool {
    t.split_once('.').is_some_and(|(p, n)| {
        is_id(p) && !n.is_empty() && n.chars().all(|x| x.is_ascii_alphanumeric())
    })
}

fn check_word(kind: Kind, t: &str) -> Result<(), String> {
    let ok = match kind {
        Kind::Arm => ARMS.contains(&t),
        Kind::Actor => ARMS.contains(&t) || crate::stations::names().iter().any(|n| n == t),
        Kind::Station => crate::stations::names().iter().any(|n| n == t),
        Kind::Pincer => crate::pincers::NAMES.contains(&t),
        Kind::Spot => check_word(Kind::Frame, t).is_ok() || check_word(Kind::Joint, t).is_ok(),
        Kind::Tool => TOOLS.contains(&t),
        Kind::Part => is_id(t),
        Kind::Pin => is_pin(t),
        Kind::Frame => {
            is_id(t) || is_pin(t) || t.strip_suffix(".tip").is_some_and(is_pin) || {
                // a point along a lead, the mm after the pin (U1.3.2.5)
                let mut it = t.splitn(3, '.');
                match (it.next(), it.next(), it.next()) {
                    (Some(id), Some(pn), Some(mm)) => {
                        is_pin(&format!("{id}.{pn}"))
                            && mm.parse::<f64>().is_ok_and(|x| x.is_finite() && x >= 0.0)
                    }
                    _ => false,
                }
            }
        }
        Kind::Joint => t
            .split_once('+')
            .is_some_and(|(a, b)| is_pin(a) && is_pin(b) && a != b),
        Kind::Target => is_pin(t) || check_word(Kind::Joint, t).is_ok() || t == "supports",
        Kind::Name => {
            !t.is_empty()
                && t.chars()
                    .all(|x| x.is_ascii_alphanumeric() || x == '_' || x == '-')
        }
        Kind::Choice(words) => words.contains(&t),
        Kind::Number(..) | Kind::Int(..) | Kind::Vec3(_) | Kind::Angles => false,
    };
    if ok {
        return Ok(());
    }
    Err(match kind {
        Kind::Arm => format!("no arm {t} (arms: {})", ARMS.join(" ")),
        Kind::Actor => format!(
            "no arm or station {t} (arms: {}; stations: {})",
            ARMS.join(" "),
            crate::stations::names().join(" ")
        ),
        Kind::Station => format!(
            "no station {t} (stations: {})",
            crate::stations::names().join(" ")
        ),
        Kind::Pincer => format!(
            "no pincer {t} (pincers: {})",
            crate::pincers::NAMES.join(" ")
        ),
        Kind::Spot => format!("{t} is not a frame or a joint (U1.3.tip, U1.3+R1.1)"),
        Kind::Tool => format!("no tool {t} (tools: {})", TOOLS.join(" ")),
        Kind::Part => format!("{t} is not a part id (U1, R2, ...)"),
        Kind::Pin => format!("{t} is not a pin (U1.3)"),
        Kind::Frame => format!("{t} is not a frame (U1, U1.3, U1.3.tip, U1.3.5)"),
        Kind::Joint => format!("{t} is not a joint (U1.3+R1.1)"),
        Kind::Target => format!("{t} is not a pin or a joint (U1.3, U1.3+R1.1)"),
        Kind::Choice(words) => format!("{t} is not one of {}", words.join(" ")),
        _ => format!("{t} is not a name"),
    })
}

fn check_number(kind: Kind, x: f64) -> Result<f64, String> {
    if !x.is_finite() {
        return Err("not a finite number".into());
    }
    match kind {
        Kind::Number(unit, lo, hi) if x < lo || x > hi => Err(format!(
            "{} is outside {} to {} {unit}",
            num(x),
            num(lo),
            num(hi)
        )),
        #[allow(clippy::cast_precision_loss)]
        Kind::Int(lo, hi) if x.fract() != 0.0 || x < lo as f64 || x > hi as f64 => Err(format!(
            "{} is not a whole number from {lo} to {hi}",
            num(x)
        )),
        _ => Ok(x),
    }
}

fn parse_number(t: &str) -> Result<f64, String> {
    t.parse::<f64>().map_err(|_| format!("{t} is not a number"))
}

/// A value from a text token (or six, for angles).
fn from_text(a: &Arg, tokens: &[&str]) -> Result<Value, String> {
    let r = match a.kind {
        Kind::Number(..) | Kind::Int(..) => parse_number(tokens[0])
            .and_then(|x| check_number(a.kind, x))
            .map(Value::Number),
        Kind::Vec3(_) => {
            let xs = tokens[0]
                .split(',')
                .map(parse_number)
                .collect::<Result<Vec<_>, _>>()?;
            if xs.len() != 3 || xs.iter().any(|x| !x.is_finite()) {
                Err(format!("{} is not three numbers x,y,z", tokens[0]))
            } else {
                Ok(Value::Numbers(xs))
            }
        }
        Kind::Angles => tokens
            .iter()
            .map(|t| parse_number(t).and_then(|x| check_number(a.kind, x)))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Numbers),
        _ => check_word(a.kind, tokens[0]).map(|()| Value::Word(tokens[0].to_string())),
    };
    r.map_err(|e| format!("{}: {e}", a.name))
}

/// A value from JSON.
fn from_json(a: &Arg, j: &Json) -> Result<Value, String> {
    let numbers = |n: usize| -> Result<Vec<f64>, String> {
        let xs: Option<Vec<f64>> = j
            .as_array()
            .and_then(|v| v.iter().map(Json::as_f64).collect());
        match xs {
            Some(xs) if xs.len() == n => Ok(xs),
            _ => Err(format!("{}: expected an array of {n} numbers", a.name)),
        }
    };
    match a.kind {
        Kind::Number(..) | Kind::Int(..) => {
            let x = j
                .as_f64()
                .ok_or_else(|| format!("{}: expected a number", a.name))?;
            check_number(a.kind, x)
                .map(Value::Number)
                .map_err(|e| format!("{}: {e}", a.name))
        }
        Kind::Vec3(_) => numbers(3).map(Value::Numbers),
        Kind::Angles => numbers(6).map(Value::Numbers),
        _ => {
            let t = j
                .as_str()
                .ok_or_else(|| format!("{}: expected text", a.name))?;
            check_word(a.kind, t).map_err(|e| format!("{}: {e}", a.name))?;
            Ok(Value::Word(t.to_string()))
        }
    }
}

/// Puts the arguments given in the spec's order and checks the required
/// ones are there.
fn finish(s: &Spec, mut given: Vec<(String, Value)>, dry: bool) -> Result<Command, String> {
    for a in s.positional.iter().chain(s.named) {
        if a.required && !given.iter().any(|(n, _)| n == a.name) {
            return Err(format!("{}: {} is required", s.op, a.name));
        }
    }
    let order = |n: &str| s.positional.iter().chain(s.named).position(|a| a.name == n);
    given.sort_by_key(|(n, _)| order(n));
    Ok(Command {
        op: s.op.to_string(),
        args: given,
        dry,
    })
}

fn dry_flag(t: &str) -> Result<bool, String> {
    match t {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(format!("dry: {t} is not 0 or 1")),
    }
}

/// One command from a line of text (no `;`, no comment).
///
/// # Errors
///
/// An unknown command or argument, a missing or wrong value, all named.
pub fn parse_line(line: &str) -> Result<Command, String> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    let Some((&op, rest)) = tokens.split_first() else {
        return Err("an empty command".into());
    };
    let s =
        spec(op).ok_or_else(|| format!("no command {op} (just ext cell commands lists them)"))?;
    let (named, positional): (Vec<&str>, Vec<&str>) = rest.iter().partition(|t| t.contains('='));
    let mut given = Vec::new();
    let mut dry = false;
    let mut i = 0;
    for a in s.positional {
        let n = if a.kind == Kind::Angles { 6 } else { 1 };
        if i + n > positional.len() {
            if i < positional.len() && a.kind == Kind::Angles {
                return Err(format!(
                    "{op}: {}: six angles needed, {} given",
                    a.name,
                    positional.len() - i
                ));
            }
            break; // the required check below names it
        }
        given.push((
            a.name.to_string(),
            from_text(a, &positional[i..i + n]).map_err(|e| format!("{op}: {e}"))?,
        ));
        i += n;
    }
    if i < positional.len() {
        return Err(format!(
            "{op}: too many words: {}",
            positional[i..].join(" ")
        ));
    }
    for t in named {
        let (name, value) = t.split_once('=').unwrap_or((t, ""));
        if name == "dry" {
            dry = dry_flag(value).map_err(|e| format!("{op}: {e}"))?;
            continue;
        }
        let a = s
            .named
            .iter()
            .find(|a| a.name == name)
            .ok_or_else(|| format!("{op}: no argument {name}= ({})", names(s)))?;
        if given.iter().any(|(n, _)| n == name) {
            return Err(format!("{op}: {name}= given twice"));
        }
        given.push((
            name.to_string(),
            from_text(a, &[value]).map_err(|e| format!("{op}: {e}"))?,
        ));
    }
    finish(s, given, dry)
}

fn names(s: &Spec) -> String {
    let n: Vec<String> = s.named.iter().map(|a| format!("{}=", a.name)).collect();
    if n.is_empty() {
        "it takes none".into()
    } else {
        n.join(" ")
    }
}

/// One command from a JSON object.
///
/// # Errors
///
/// Not an object, no or an unknown `op`, an unknown key, a wrong value.
pub fn parse_json(j: &Json) -> Result<Command, String> {
    let m = j.as_object().ok_or("a command in JSON is an object")?;
    let op = m
        .get("op")
        .and_then(Json::as_str)
        .ok_or("a command needs \"op\"")?;
    let s =
        spec(op).ok_or_else(|| format!("no command {op} (just ext cell commands lists them)"))?;
    let mut given = Vec::new();
    let mut dry = false;
    for (k, v) in m {
        match k.as_str() {
            "op" => {}
            "dry" => {
                dry = match v {
                    Json::Bool(b) => *b,
                    Json::Number(n) if n.as_f64() == Some(0.0) => false,
                    Json::Number(n) if n.as_f64() == Some(1.0) => true,
                    _ => return Err(format!("{op}: dry is true or false")),
                }
            }
            _ => {
                let a = arg(s, k).ok_or_else(|| format!("{op}: no argument {k}"))?;
                given.push((
                    k.clone(),
                    from_json(a, v).map_err(|e| format!("{op}: {e}"))?,
                ));
            }
        }
    }
    finish(s, given, dry)
}

/// Every command in a program: text lines, `;` between commands on a
/// line, `#` comments; a line starting with `{` is one JSON command.
/// Errors name the line.
///
/// # Errors
///
/// The first line with a wrong command.
pub fn parse_program(text: &str) -> Result<Vec<Command>, String> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('{') {
            let j: Json =
                serde_json::from_str(trimmed).map_err(|e| format!("line {}: {e}", n + 1))?;
            out.push(parse_json(&j).map_err(|e| format!("line {}: {e}", n + 1))?);
            continue;
        }
        let code = line.split('#').next().unwrap_or("");
        for part in code.split(';') {
            if part.trim().is_empty() {
                continue;
            }
            out.push(parse_line(part).map_err(|e| format!("line {}: {e}", n + 1))?);
        }
    }
    Ok(out)
}

fn placeholder(a: &Arg) -> String {
    match a.kind {
        Kind::Arm => "ARM".into(),
        Kind::Actor => "ARM|STATION".into(),
        Kind::Station => "STATION".into(),
        Kind::Pincer => "PINCER".into(),
        Kind::Spot => "FRAME|JOINT".into(),
        Kind::Tool => "TOOL".into(),
        Kind::Part => "PART".into(),
        Kind::Pin => "PIN".into(),
        Kind::Frame => "FRAME".into(),
        Kind::Joint => "JOINT".into(),
        Kind::Target => "TARGET".into(),
        Kind::Name => "NAME".into(),
        Kind::Choice(w) => w.join("|"),
        Kind::Number(unit, ..) => match unit {
            "fraction" | "x" => "N".into(),
            u => u.to_uppercase(),
        },
        Kind::Int(..) => "N".into(),
        Kind::Vec3("deg") => "R,P,Y".into(),
        Kind::Vec3(_) => "X,Y,Z".into(),
        Kind::Angles => "J1 J2 J3 J4 J5 J6".into(),
    }
}

/// A command's synopsis: `move ARM to=X,Y,Z [rpy=R,P,Y] [speed=MM/S]`.
#[must_use]
pub fn synopsis(s: &Spec) -> String {
    let mut out = s.op.to_string();
    for a in s.positional {
        out.push(' ');
        out.push_str(&placeholder(a));
    }
    for a in s.named {
        let t = format!("{}={}", a.name, placeholder(a));
        out.push(' ');
        if a.required {
            out.push_str(&t);
        } else {
            let _ = write!(out, "[{t}]");
        }
    }
    out
}

/// The command list, one line a command: its synopsis and what it does.
#[must_use]
pub fn list() -> String {
    COMMANDS
        .iter()
        .map(|s| format!("{} -- {}\n", synopsis(s), s.summary))
        .collect()
}

/// The command list as a markdown table, for the docs.
#[must_use]
pub fn markdown() -> String {
    let mut out = String::from("| Command | Family | Does |\n| ------- | ------ | ---- |\n");
    for s in COMMANDS {
        let _ = writeln!(
            out,
            "| `{}` | {} | {} |",
            synopsis(s).replace('|', "\\|"),
            s.family,
            s.summary
        );
    }
    out
}

fn arg_schema(a: &Arg) -> Json {
    let mut j = match a.kind {
        Kind::Arm => json!({"type": "string", "enum": ARMS}),
        Kind::Actor => {
            let mut all: Vec<String> = ARMS.iter().map(|a| (*a).to_string()).collect();
            all.extend(crate::stations::names());
            json!({"type": "string", "enum": all})
        }
        Kind::Station => json!({"type": "string", "enum": crate::stations::names()}),
        Kind::Pincer => json!({"type": "string", "enum": crate::pincers::NAMES}),
        Kind::Spot => {
            json!({"type": "string", "pattern": "^[A-Z][A-Za-z0-9_]*\\.[A-Za-z0-9_]+(\\.(tip|[0-9.]+))?(\\+[A-Z][A-Za-z0-9_]*\\.[A-Za-z0-9_]+)?$"})
        }
        Kind::Tool => json!({"type": "string", "enum": TOOLS}),
        Kind::Choice(w) => json!({"type": "string", "enum": w}),
        Kind::Part => json!({"type": "string", "pattern": "^[A-Z][A-Za-z0-9_]*$"}),
        Kind::Pin => json!({"type": "string", "pattern": "^[A-Z][A-Za-z0-9_]*\\.[A-Za-z0-9]+$"}),
        Kind::Frame => json!({"type": "string",
            "pattern": "^[A-Z][A-Za-z0-9_]*(\\.[A-Za-z0-9]+(\\.tip)?)?$"}),
        Kind::Joint => json!({"type": "string",
            "pattern": "^[A-Z][A-Za-z0-9_]*\\.[A-Za-z0-9]+\\+[A-Z][A-Za-z0-9_]*\\.[A-Za-z0-9]+$"}),
        Kind::Target => json!({"type": "string",
            "pattern": "^[A-Z][A-Za-z0-9_]*\\.[A-Za-z0-9]+(\\+[A-Z][A-Za-z0-9_]*\\.[A-Za-z0-9]+)?$"}),
        Kind::Name => json!({"type": "string", "pattern": "^[A-Za-z0-9_-]+$"}),
        Kind::Number(_, lo, hi) => json!({"type": "number", "minimum": lo, "maximum": hi}),
        Kind::Int(lo, hi) => json!({"type": "integer", "minimum": lo, "maximum": hi}),
        Kind::Vec3(_) => {
            json!({"type": "array", "items": {"type": "number"}, "minItems": 3, "maxItems": 3})
        }
        Kind::Angles => {
            json!({"type": "array", "items": {"type": "number"}, "minItems": 6, "maxItems": 6})
        }
    };
    let unit = match a.kind {
        Kind::Number(u, ..) | Kind::Vec3(u) => format!(" ({u})"),
        Kind::Angles => " (deg)".into(),
        _ => String::new(),
    };
    j["description"] = json!(format!("{}{unit}", a.doc));
    j
}

/// A command's JSON Schema: the input schema of its typed MCP tool.
#[must_use]
pub fn schema(s: &Spec) -> Json {
    let mut props = Map::new();
    props.insert("op".into(), json!({"const": s.op}));
    let mut required = vec![json!("op")];
    for a in s.positional.iter().chain(s.named) {
        props.insert(a.name.into(), arg_schema(a));
        if a.required {
            required.push(json!(a.name));
        }
    }
    props.insert(
        "dry".into(),
        json!({"type": "boolean", "description": "run on a copy, change nothing"}),
    );
    json!({
        "title": s.op,
        "description": s.summary,
        "type": "object",
        "properties": props,
        "required": required,
        "additionalProperties": false,
    })
}

/// Every command's schema, by name, in the table's order.
#[must_use]
pub fn schemas() -> Json {
    Json::Array(COMMANDS.iter().map(schema).collect())
}
