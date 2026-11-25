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
            "A simulated robot assembly cell: four six-axis arms (A to D) round a work zone at 0 0 0 (mm, Z up), a tool rack beside each, a parts tray, a rotary stage with a vise, a microscope. Drive it with commands: cell_do takes the command language (cell_state lists the commands), or use the typed tools. cell_state's places say where things are: the vise's middle (place a part with its origin there, then close the vise), the stage top, the floor, the tray's pockets. Tools also stand in stations (S1 iron, S2 solder feeder, S3 welder) on fixed lines to one point; the positioner (stage turn=, tilt=, at=) brings work to them: present PIN at=S1 and=S2, bring the other lead with an arm, then touch S1, feed S2, weld S3. A command that cannot start is refused and changes nothing; one that collides stops there. The cell's page follows the session live at http://127.0.0.1:8470/?mcp.",
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
        let mut s = self
            .session
            .lock()
            .map_err(|_| ErrorData::internal_error("the session is poisoned", None))?;
        let name = request.name.as_ref();
        Ok(match name {
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
