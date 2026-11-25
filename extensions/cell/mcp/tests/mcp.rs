//! xetal-cell-mcp driven by a scripted MCP client over stdio: the tools
//! (generic and one a command, their schemas the command table's), a
//! session that puts the DIP in the vise and brings a resistor's lead to
//! its pin, refusals as tool errors, a program run, the session saved;
//! and the page's endpoints, which follow the same session.

use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

struct Client {
    child: Child,
    to: ChildStdin,
    from: BufReader<ChildStdout>,
    id: u64,
}

impl Client {
    fn start(port: u16, session: &std::path::Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_xetal-cell-mcp"))
            .env("XETAL_CELL_PORT", port.to_string())
            .env("XETAL_CELL_SESSION", session)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let to = child.stdin.take().unwrap();
        let from = BufReader::new(child.stdout.take().unwrap());
        let mut c = Self {
            child,
            to,
            from,
            id: 0,
        };
        let init = c.request("initialize", json!({"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "test", "version": "0"}}));
        assert_eq!(init["result"]["serverInfo"]["name"], "xetal-cell-mcp");
        c.send(json!({"jsonrpc": "2.0", "method": "notifications/initialized"}));
        c
    }

    fn send(&mut self, v: Value) {
        writeln!(self.to, "{v}").unwrap();
        self.to.flush().unwrap();
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        self.id += 1;
        let id = self.id;
        self.send(json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}));
        loop {
            let mut line = String::new();
            assert!(
                self.from.read_line(&mut line).unwrap() > 0,
                "the server stopped"
            );
            let v: Value = serde_json::from_str(&line).unwrap();
            if v["id"] == id {
                return v;
            }
        }
    }

    /// A tool's result: whether it is an error, and its text as JSON (or
    /// as a string when it is not JSON).
    fn tool(&mut self, name: &str, args: Value) -> (bool, Value) {
        let r = self.request("tools/call", json!({"name": name, "arguments": args}));
        let result = &r["result"];
        let text = result["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();
        (
            result["isError"] == true,
            serde_json::from_str(&text).unwrap_or(Value::String(text)),
        )
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn session_file(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("xetal-cell-mcp-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("session.cell")
}

fn all_ok(r: &Value) -> bool {
    r["outcomes"]
        .as_array()
        .is_some_and(|o| o.iter().all(|x| x["ok"] == true))
}

#[test]
fn the_tools_come_from_the_command_table() {
    let mut c = Client::start(0, &session_file("tools"));
    let tools = c.request("tools/list", json!({}))["result"]["tools"]
        .as_array()
        .unwrap()
        .clone();
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    for generic in [
        "cell_reset",
        "cell_do",
        "cell_run",
        "cell_state",
        "cell_rewind",
        "cell_inspect",
        "cell_program",
    ] {
        assert!(names.contains(&generic), "{generic}");
    }
    // one a command (bar the three the generic tools are), each once
    let mut sorted = names.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), names.len(), "a tool named twice");
    assert_eq!(names.len(), 7 + xetal_ext_cell::command::COMMANDS.len() - 3);
    // a typed tool's schema is the command's, with op implied
    let mv = tools.iter().find(|t| t["name"] == "cell_move").unwrap();
    let schema = &mv["inputSchema"];
    assert_eq!(schema["required"], json!(["arm", "to"]));
    assert!(schema["properties"].get("op").is_none());
    assert_eq!(schema["properties"]["speed"]["maximum"], 500.0);
    // the stations: an arm or a station touches; present names a station
    let touch = tools.iter().find(|t| t["name"] == "cell_touch").unwrap();
    assert_eq!(
        touch["inputSchema"]["properties"]["arm"]["enum"],
        json!(["A", "S1", "S3"])
    );
    let present = tools.iter().find(|t| t["name"] == "cell_present").unwrap();
    assert_eq!(
        present["inputSchema"]["properties"]["at"]["enum"],
        json!(["S1", "S3"])
    );
}

#[test]
fn an_agent_puts_the_dip_in_the_vise_and_a_lead_to_its_pin() {
    let file = session_file("agent");
    let mut c = Client::start(0, &file);
    let (err, r) = c.tool("cell_reset", json!({"circuit": "demo"}));
    assert!(!err && all_ok(&r));
    // the command language, then typed tools
    let (err, r) = c.tool(
        "cell_do",
        json!({"commands": "tool A tweezers\npick A U1 grasp=ends"}),
    );
    assert!(!err && all_ok(&r), "{r}");
    let (_, r) = c.tool(
        "cell_place",
        json!({"arm": "A", "part": "U1", "at": [0, 0, -24], "rpy": [0, 0, 90]}),
    );
    assert!(all_ok(&r), "{r}");
    let (_, r) = c.tool("cell_vise", json!({"action": "close"}));
    assert!(
        all_ok(&r) && r["outcomes"][0]["events"][0] == "the vise holds U1",
        "{r}"
    );
    let (_, r) = c.tool("cell_release", json!({"arm": "A", "part": "U1"}));
    assert!(all_ok(&r), "{r}");
    let (_, r) = c.tool(
        "cell_pick",
        json!({"arm": "A", "part": "R1", "grasp": "body"}),
    );
    assert!(all_ok(&r), "{r}");
    let (_, r) = c.tool(
        "cell_do",
        json!({"commands": "lift A mm=80\nmove A to=0,60,60 rpy=180,0,90\nmove A to=34.81,1.27,-14.05\nalign A R1.1.tip to=U1.3.tip gap=0.1"}),
    );
    assert!(all_ok(&r), "{r}");
    // the state says where it all is
    let (_, s) = c.tool("cell_state", json!({}));
    assert_eq!(s["arms"][0]["holding"], "R1");
    let u1 = s["parts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == "U1")
        .unwrap();
    assert_eq!(u1["holder"], "vise");
    assert!(
        s["commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|l| l.as_str().unwrap().starts_with("align ARM FRAME"))
    );
    // where things are: the vise's middle is where U1 was placed
    assert_eq!(s["places"]["vise_center"], json!([0.0, 0.0, -32.0]));
    assert_eq!(s["places"]["tray_pockets"].as_array().unwrap().len(), 8);
    // the session, saved as a program
    let (_, p) = c.tool("cell_program", json!({}));
    let program = p["program"].as_str().unwrap();
    assert!(
        program
            .starts_with("reset circuit=demo\ntool A tweezers\npick A U1 grasp=ends\nplace A U1"),
        "{program}"
    );
    assert!(program.ends_with("align A R1.1.tip to=U1.3.tip gap=0.1"));
    let saved = std::fs::read_to_string(&file).unwrap();
    assert!(saved.contains("align A R1.1.tip to=U1.3.tip gap=0.1\n"));
    // and it runs again, the same
    let mut again = xetal_ext_cell::world::World::default();
    for o in again.apply_all(&xetal_ext_cell::command::parse_program(&saved).unwrap()) {
        assert!(o.ok, "{o:?}");
    }
    assert_eq!(again.arm("A").unwrap().holding.as_deref(), Some("R1"));
}

#[test]
fn refusals_are_tool_errors_or_outcomes() {
    let mut c = Client::start(0, &session_file("refused"));
    // arguments the table refuses: a tool error naming why
    let (err, r) = c.tool(
        "cell_pick",
        json!({"arm": "Q", "part": "U1", "grasp": "ends"}),
    );
    assert!(err && r.as_str().unwrap().contains("no arm Q"), "{r}");
    let (err, r) = c.tool("cell_do", json!({"commands": "fly A"}));
    assert!(err && r.as_str().unwrap().contains("no command fly"), "{r}");
    // a command the cell refuses: an outcome saying why, nothing changed
    let (err, r) = c.tool(
        "cell_pick",
        json!({"arm": "A", "part": "U1", "grasp": "ends"}),
    );
    assert!(
        !err && r["outcomes"][0]["refused"]
            .as_str()
            .unwrap()
            .contains("not tweezers"),
        "{r}"
    );
    let (_, p) = c.tool("cell_program", json!({}));
    assert_eq!(
        p["program"], "reset circuit=demo",
        "a refused command is not part of the session"
    );
}

#[test]
fn a_program_runs_whole() {
    let mut c = Client::start(0, &session_file("run"));
    let (err, r) = c.tool(
        "cell_run",
        json!({"program": include_str!("../../programs/solder.cell")}),
    );
    assert!(!err && all_ok(&r), "{r}");
    let (_, j) = c.tool("cell_inspect", json!({"target": "U1.3+R1.1"}));
    assert!(
        j["outcomes"][0]["events"][0]
            .as_str()
            .unwrap()
            .contains(": good;"),
        "{j}"
    );
    let (_, r) = c.tool("cell_rewind", json!({"t": 0}));
    assert!(all_ok(&r), "{r}");
}

#[test]
fn the_effective_program_leaves_out_what_was_rewound() {
    let mut c = Client::start(0, &session_file("rewound"));
    let (_, r) = c.tool("cell_do", json!({"commands": "tool A tweezers"}));
    let t = r["outcomes"][0]["t1"].as_f64().unwrap();
    c.tool("cell_do", json!({"commands": "move A to=0,100,0"}));
    c.tool("cell_rewind", json!({"t": t}));
    c.tool("cell_do", json!({"commands": "move A to=0,80,0"}));
    let (_, p) = c.tool("cell_program", json!({}));
    assert_eq!(
        p["effective"], "reset circuit=demo\ntool A tweezers\nmove A to=0,80,0",
        "{p}"
    );
    // both run to the same cell
    let run = |text: &str| {
        let mut w = xetal_ext_cell::world::World::default();
        w.apply_all(&xetal_ext_cell::command::parse_program(text).unwrap());
        w.arm("A").unwrap().joints
    };
    assert_eq!(
        run(p["program"].as_str().unwrap()),
        run(p["effective"].as_str().unwrap())
    );
}

fn http(port: u16, method: &str, path: &str, body: &str) -> String {
    let mut s = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
    write!(
        s,
        "{method} {path} HTTP/1.1\r\nHost: x\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    )
    .unwrap();
    let mut out = String::new();
    s.read_to_string(&mut out).unwrap();
    out.split_once("\r\n\r\n")
        .map(|(_, b)| b.to_string())
        .unwrap_or_default()
}

#[test]
fn the_page_follows_the_session() {
    // a free port, the page's server on it
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let mut c = Client::start(port, &session_file("page"));
    let v0: Value = serde_json::from_str(&http(port, "GET", "/mcp/version", "")).unwrap();
    let (_, r) = c.tool("cell_do", json!({"commands": "tool A tweezers"}));
    assert!(all_ok(&r));
    let v1: Value = serde_json::from_str(&http(port, "GET", "/mcp/version", "")).unwrap();
    assert_eq!(
        v1["version"].as_u64().unwrap(),
        v0["version"].as_u64().unwrap() + 1
    );
    let last: Value = serde_json::from_str(&http(port, "GET", "/mcp/last", "")).unwrap();
    assert_eq!(last["outcomes"][0]["command"], "tool A tweezers");
    assert!(http(port, "GET", "/mcp/scene", "").starts_with("cell('mcp', 1280, 720)"));
    // the page's own command drives the same cell
    let r: Value =
        serde_json::from_str(&http(port, "POST", "/mcp/command", "tool A iron temp=330")).unwrap();
    assert!(all_ok(&r), "{r}");
    let (_, s) = c.tool("cell_state", json!({}));
    assert_eq!(s["arms"][0]["tool"], "iron");
    // nothing outside the page's directory
    assert!(http(port, "GET", "/../Cargo.toml", "").starts_with("no"));
}

#[test]
fn claude_codes_session_stops_where_the_checker_now_sees_a_collision() {
    // programs/claude.cell: Claude Code's session over MCP, recorded
    // while a carried part could pass through the vise unchecked; its
    // resistor lead went through a jaw to pin 3, which is now caught
    let mut w = xetal_ext_cell::world::World::default();
    let out = w.apply_all(
        &xetal_ext_cell::command::parse_program(include_str!("../../programs/claude.cell"))
            .unwrap(),
    );
    let last = out.last().unwrap();
    assert!(last.command.starts_with("align A R1.1.tip"), "{last:?}");
    assert!(
        last.stopped
            .as_deref()
            .unwrap()
            .contains("with the vise's jaw"),
        "{last:?}"
    );
}
