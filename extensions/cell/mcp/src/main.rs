//! xetal-cell-mcp: the assembly cell over MCP (stdio). An agent drives
//! the simulator command by command: `cell_do` takes any commands (the
//! command language, or JSON lines), a typed tool for each command
//! (`cell_move`, `cell_pick`, ...) takes its arguments by name, its
//! schema made from the command table; `cell_reset`, `cell_run`,
//! `cell_state`, `cell_rewind`, `cell_inspect` and `cell_program` round
//! it out. Every command that runs is saved as a program
//! (`work/mcp/session.cell`), to preview and run again.
//!
//! Beside it, the cell's live page is served on port 8470
//! (XETAL_CELL_PORT; 0 for none; XETAL_CELL_BIND for the network), and
//! with `?mcp` the page follows the session: each command an agent sends
//! is played as it comes, and the page's own controls drive the same
//! cell.

mod page;

use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation,
    ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerConfig, Tool,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler, ServiceExt};
use serde_json::{Map, Value as Json, json};
use std::sync::{Arc, Mutex};
use xetal_ext_cell::command::{self, COMMANDS, parse_json, parse_program};
use xetal_ext_cell::world::{World, outcome_json};

/// The cell an agent drives, shared with the page.
#[derive(Default)]
pub struct Session {
    /// The cell.
    pub world: World,
    /// Every command that ran, as text: the session as a program.
    pub program: Vec<String>,
    /// Bumped each time the cell changes, for the page to notice.
    pub version: u64,
    /// The last commands' times and outcomes: what the page plays.
    pub last: Json,
}

impl Session {
    /// Runs commands on the cell: `{"outcomes": [...], "t0", "t1"}`.
    pub fn run(&mut self, commands: &[command::Command]) -> Json {
        let t0 = self.world.t;
        let outs = self.world.apply_all(commands);
        for (c, o) in commands.iter().zip(&outs) {
            if !o.dry && o.refused.is_none() {
                self.program.push(c.to_text());
            }
        }
        let outcomes: Vec<Json> = outs
            .iter()
            .map(|o| serde_json::from_str(&outcome_json(o)).unwrap_or(Json::Null))
            .collect();
        let r = json!({ "outcomes": outcomes, "t0": t0, "t1": self.world.t });
        if outs.iter().any(|o| !o.dry && o.refused.is_none()) {
            self.version += 1;
            self.last = r.clone();
            save(&self.program);
        }
        r
    }

    /// Commands tried on a copy of the cell: kept only if every one is
    /// ok (`{"kept": true, "outcomes", "t0", "t1"}`), else the cell as it
    /// was and `{"kept": false, "failed": COMMAND, "why", "outcomes"}`.
    pub fn attempt(&mut self, commands: &[command::Command]) -> Json {
        let mut w = self.world.clone();
        let t0 = w.t;
        let outs = w.apply_all(commands);
        let outcomes: Vec<Json> = outs
            .iter()
            .map(|o| serde_json::from_str(&outcome_json(o)).unwrap_or(Json::Null))
            .collect();
        if let Some(o) = outs.iter().find(|o| !o.ok) {
            let why = o
                .refused
                .clone()
                .or_else(|| o.stopped.clone())
                .unwrap_or_default();
            return json!({ "kept": false, "failed": o.command, "why": why, "outcomes": outcomes });
        }
        self.world = w;
        for c in commands {
            self.program.push(c.to_text());
        }
        let r = json!({ "kept": true, "outcomes": outcomes, "t0": t0, "t1": self.world.t });
        self.version += 1;
        self.last = r.clone();
        save(&self.program);
        r
    }

    /// A new cell, a circuit's tray laid out.
    pub fn reset(&mut self, circuit: &str) -> Json {
        self.world = World::default();
        self.program.clear();
        let c = parse_program(&format!("reset circuit={circuit}")).unwrap_or_default();
        self.run(&c)
    }
}

/// Where the session is saved: XETAL_CELL_SESSION, else
/// extensions/cell/work/mcp/session.cell.
fn session_path() -> std::path::PathBuf {
    std::env::var_os("XETAL_CELL_SESSION")
        .map(Into::into)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../work/mcp/session.cell")
        })
}

fn save(program: &[String]) {
    let path = session_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let text: String =
        std::iter::once("# a session over MCP: every command that ran, in order\n".to_string())
            .chain(program.iter().map(|c| format!("{c}\n")))
            .collect();
    let _ = std::fs::write(path, text);
}

/// A circuit's netlist as the planner reads it (circuit, core, a net a
/// line): the core is the part with the most pins in the nets.
pub fn netlist(circuit: &str) -> Result<String, String> {
    let nets = xetal_ext_cell::layout::nets(circuit)?;
    if nets.is_empty() {
        return Err(format!("{circuit} has no netlist"));
    }
    let laid = xetal_ext_cell::layout::circuit(circuit)?;
    let count = |id: &str| {
        nets.iter()
            .flat_map(|n| &n.pins)
            .filter(|p| p.split('.').next() == Some(id))
            .count()
    };
    let core = laid
        .iter()
        .max_by_key(|l| count(&l.id))
        .ok_or_else(|| format!("{circuit} has no parts"))?;
    let mut text = format!("circuit {circuit}\ncore {} {}\n", core.id, core.kind);
    for n in &nets {
        text.push_str(&format!("net {} {}\n", n.name, n.pins.join(" ")));
    }
    Ok(text)
}

/// A netlist planned by CellPlan.xtl (the search for one of nets, the
/// neighbor planner for one of neighbor lines): the program, not run.
/// It runs X_eTaL (target/debug/xetal-x, built with the workspace).
pub fn plan(netlist: &str) -> Result<String, String> {
    let cell = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let xetal = cell.join("../../target/debug/xetal-x");
    if !xetal.is_file() {
        return Err("no xetal-x: build it (cargo build -p xetal-x)".into());
    }
    let search = netlist.lines().any(|l| l.starts_with("net "));
    let lit = netlist
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    let script = format!(
        "\"cp:\" u_se< \"CellPlan\"\ncp:{} \"{lit}\"\n",
        if search { "b_uild" } else { "p_lan" }
    );
    let dir = std::env::temp_dir().join(format!("cell-plan-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let file = dir.join("plan.xtl");
    std::fs::write(&file, script).map_err(|e| e.to_string())?;
    let out = std::process::Command::new(&xetal)
        .args(["--ext", ".", "run"])
        .arg(&file)
        .current_dir(&cell)
        .env("XETAL_PATH", cell.join("lib"))
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr)
            .chars()
            .take(2000)
            .collect());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

#[derive(Clone)]
struct Cell {
    session: Arc<Mutex<Session>>,
}

fn text(v: &Json) -> CallToolResponse {
    CallToolResult::success(vec![ContentBlock::text(
        serde_json::to_string_pretty(v).unwrap_or_default(),
    )])
    .into()
}

fn failed(why: impl Into<String>) -> CallToolResponse {
    CallToolResult::error(vec![ContentBlock::text(why.into())]).into()
}

fn object(v: Json) -> Arc<Map<String, Json>> {
    Arc::new(v.as_object().cloned().unwrap_or_default())
}

/// The tools: the generic ones, then one a command from the table.
fn tools() -> Vec<Tool> {
    let s = |props: Json, required: &[&str]| {
        object(json!({ "type": "object", "properties": props, "required": required }))
    };
    let mut out = vec![
        Tool::new(
            "cell_netlist",
            "A circuit's netlist: what the build must connect, as the planner reads it -- circuit NAME, core ID KIND (the part with the most pins), then a net a line: net NAME PIN PIN ... (U1.8 is the core's pin 8; R1.1 a resistor's lead 1). cell_check compares the cell's joints with it.",
            s(
                json!({ "circuit": { "type": "string", "enum": xetal_ext_cell::layout::circuits(), "description": "the circuit" } }),
                &["circuit"],
            ),
        ),
        Tool::new(
            "cell_attempt",
            "Try commands: they run on a copy of the cell, kept only if every one is ok; otherwise the cell is as it was and the answer says which failed and why (kept: false). A step to try without needing cell_rewind after.",
            s(
                json!({ "commands": { "type": "string", "description": "commands, as cell_do takes them" } }),
                &["commands"],
            ),
        ),
        Tool::new(
            "cell_plan",
            "Plan a netlist (cell_netlist's text, or a circuit's own): CellPlan.xtl's search tries each step on a simulator of its own and keeps the first it takes (a comment for each it refused), and returns the whole program -- not run: cell_run runs it, or take what helps. A circuit of nets takes some minutes.",
            s(
                json!({ "netlist": { "type": "string", "description": "a netlist's text" }, "circuit": { "type": "string", "description": "a circuit, its netlist planned" } }),
                &[],
            ),
        ),
        Tool::new(
            "cell_reset",
            "A new cell: arms home with no tool, the racks full, the stage at 0, the vise open; the tray laid out for a circuit (demo: a DIP-8, a resistor, an LED, a ceramic disc; weld: two ceramic discs, an LED, a resistor, to try welding with; none: empty). The session's program starts again.",
            s(
                json!({ "circuit": { "type": "string", "enum": xetal_ext_cell::layout::circuits(), "description": "the tray's circuit (demo if not given)" } }),
                &[],
            ),
        ),
        Tool::new(
            "cell_do",
            "Run commands in the cell's command language -- one a line, ';' between, '#' a comment, or a JSON object a line -- stopping after the first not ok. Each outcome: ok, its start and end times, the motions it was made of, why it was refused (nothing changed) or where it stopped (a collision: what touched what), events and warnings. cell_state, and the command list in this tool's companions (cell_move, cell_pick, ...), say what there is.",
            s(
                json!({ "commands": { "type": "string", "description": "e.g. 'tool A tweezers\\npick A U1 grasp=ends'" } }),
                &["commands"],
            ),
        ),
        Tool::new(
            "cell_run",
            "Run a whole program (as cell_do), from a new cell when it begins with reset.",
            s(json!({ "program": { "type": "string" } }), &["program"]),
        ),
        Tool::new(
            "cell_state",
            "The cell now: the time; each arm's joints, tool, what it holds and its tool tip (x y z mm, roll pitch yaw degrees); each part's kind, pose, what carries it, whether lost or damaged, its grasp frames; the stage, the vise; each joint's state; the command list with its synopses.",
            s(json!({}), &[]),
        ),
        Tool::new(
            "cell_rewind",
            "Put the cell back as it was at a time; the commands after it stay on the timeline, marked rewound.",
            s(
                json!({ "t": { "type": "number", "minimum": 0, "description": "seconds" } }),
                &["t"],
            ),
        ),
        Tool::new(
            "cell_inspect",
            "A joint's state (solder, how long each side was wet, what it came to, its temperature) or a pin's temperature.",
            s(
                json!({ "target": { "type": "string", "description": "a joint (U1.3+R1.1) or a pin (U1.3)" } }),
                &["target"],
            ),
        ),
        Tool::new(
            "cell_program",
            "The session so far as a program: every command that ran, in order, rewinds included (saved to work/mcp/session.cell, to preview with just ext cell preview or run again, the same); and the effective program, the commands that stand (none rewound).",
            s(json!({}), &[]),
        ),
    ];
    // a typed tool for each command, its schema the table's (op implied);
    // reset, rewind and inspect are the generic tools above already
    for spec in COMMANDS
        .iter()
        .filter(|s| !["reset", "rewind", "inspect"].contains(&s.op))
    {
        let mut schema = command::schema(spec);
        if let Some(props) = schema["properties"].as_object_mut() {
            props.remove("op");
        }
        if let Some(req) = schema["required"].as_array_mut() {
            req.retain(|r| r != "op");
        }
        let name = format!("cell_{}", spec.op);
        let what = format!(
            "{} -- as the command `{}`.",
            spec.summary,
            command::synopsis(spec)
        );
        out.push(Tool::new(name, what, object(schema)));
    }
    out
}

/// The cell's state for an agent.
fn state(w: &World) -> Json {
    let mut s = w.summary();
    s["commands"] = json!(command::list().lines().collect::<Vec<_>>());
    s
}

impl ServerHandler for Cell {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("xetal-cell-mcp", env!("CARGO_PKG_VERSION")))
            .with_instructions(
            "A simulated assembly bench: one six-axis arm (A) with a tool rack, a parts tray, a turntable with a vise that also turns, tilts and rises (stage turn=, tilt=, at=), pincers on the turntable (P1, P2) that grip a lead from the side, tool stations on fixed lines to one point (S1 an iron that feeds its own solder, S3 a welder from below), a bending rig beside the turntable (insert A on=R1.2 at=3, form angle=90, shear A on=R1.2 at=8), a microscope; mm, Z up, the work zone at 0 0 0. To solder: the arm brings one lead to another (align A R1.1.tip to=U1.3.tip), a pincer grips it (pinch P1 on=R1.1 at=6), the arm lets go, the turntable presents the joint (present U1.3+R1.1 at=S1), then touch S1, feed S1, back S1, unpinch P1. cell_do takes the command language (cell_state lists the commands and says where things are), or use the typed tools. A command that cannot start is refused and changes nothing; one that collides stops there. The page follows the session live at http://127.0.0.1:8470/?mcp.",
        )
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult {
            tools: tools(),
            ..Default::default()
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let args = request.arguments.unwrap_or_default();
        let arg = |k: &str| args.get(k).and_then(Json::as_str).map(str::to_string);
        let name = request.name.as_ref();
        // the planner and the netlist need no cell: no lock held while a
        // search runs for minutes
        if name == "cell_netlist" {
            return Ok(match netlist(&arg("circuit").unwrap_or_default()) {
                Ok(t) => text(&json!({ "netlist": t })),
                Err(e) => failed(e),
            });
        }
        if name == "cell_plan" {
            let net = match (arg("netlist"), arg("circuit")) {
                (Some(n), _) => Ok(n),
                (None, Some(c)) => netlist(&c),
                (None, None) => Err("give netlist= or circuit=".to_string()),
            };
            return Ok(match net.and_then(|n| plan(&n)) {
                Ok(p) => {
                    text(&json!({ "program": p, "note": "planned, not run: cell_run runs it" }))
                }
                Err(e) => failed(e),
            });
        }
        let mut s = self
            .session
            .lock()
            .map_err(|_| ErrorData::internal_error("the session is poisoned", None))?;
        Ok(match name {
            "cell_attempt" => match parse_program(&arg("commands").unwrap_or_default()) {
                Ok(c) => text(&s.attempt(&c)),
                Err(e) => failed(e),
            },
            "cell_reset" => text(&s.reset(&arg("circuit").unwrap_or_else(|| "demo".into()))),
            "cell_do" | "cell_run" => {
                let key = if name == "cell_do" {
                    "commands"
                } else {
                    "program"
                };
                match parse_program(&arg(key).unwrap_or_default()) {
                    Ok(c) => {
                        if name == "cell_run" && c.first().is_some_and(|c| c.op == "reset") {
                            s.world = World::default();
                            s.program.clear();
                        }
                        text(&s.run(&c))
                    }
                    Err(e) => failed(e),
                }
            }
            "cell_state" => text(&state(&s.world)),
            "cell_rewind" => {
                let t = args.get("t").and_then(Json::as_f64).unwrap_or(0.0);
                match parse_program(&format!("rewind t={t}")) {
                    Ok(c) => text(&s.run(&c)),
                    Err(e) => failed(e),
                }
            }
            "cell_inspect" => {
                match parse_program(&format!("inspect {}", arg("target").unwrap_or_default())) {
                    Ok(c) => text(&s.run(&c)),
                    Err(e) => failed(e),
                }
            }
            "cell_program" => {
                // what stands: the timeline's commands, bar the rewound and the rewinds
                let effective: Vec<&str> = s
                    .world
                    .timeline
                    .iter()
                    .filter(|e| !e.rewound && !e.command.starts_with("rewind"))
                    .map(|e| e.command.as_str())
                    .collect();
                text(
                    &json!({ "program": s.program.join("\n"), "effective": effective.join("\n"), "saved": session_path().display().to_string() }),
                )
            }
            _ => match name.strip_prefix("cell_").and_then(command::spec) {
                Some(spec) => {
                    let mut j = Json::Object(args.clone());
                    j["op"] = json!(spec.op);
                    match parse_json(&j) {
                        Ok(c) => text(&s.run(&[c])),
                        Err(e) => failed(e),
                    }
                }
                None => return Err(ErrorData::invalid_params(format!("no tool {name}"), None)),
            },
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session = Arc::new(Mutex::new(Session::default()));
    session.lock().map_err(|_| "poisoned")?.reset("demo");
    page::serve(session.clone());
    let server = Cell { session }.serve(rmcp::transport::stdio()).await?;
    server.waiting().await?;
    Ok(())
}
