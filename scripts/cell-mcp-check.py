#!/usr/bin/env python3
"""The cell's page following an MCP session, checked headlessly: the
server (target/debug/xetal-cell-mcp, built) is started on port 8470 and
driven over stdio as an agent would -- the DIP into the vise -- then
the page is opened with ?mcp in headless Chrome (no window, its own
throwaway profile; scripts/cell-record.py's DevTools client) and must
show that session; more commands are sent while it is open -- a
resistor picked and its lead brought to pin 3 -- and the page must
play them as they come. A picture of the page at the end is kept:
extensions/cell/work/check-mcp.png.
    scripts/cell-mcp-check.py
"""
import base64
import importlib.util
import json
import os
import pathlib
import subprocess
import sys
import tempfile
import time
import urllib.request

root = pathlib.Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location("cell_record", pathlib.Path(__file__).with_name("cell-record.py"))
rec = importlib.util.module_from_spec(spec)
spec.loader.exec_module(rec)


class Agent:
    """A scripted MCP client over the server's stdio."""

    def __init__(self, env):
        self.p = subprocess.Popen([str(root / "target/debug/xetal-cell-mcp")], stdin=subprocess.PIPE,
                                  stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True, env=env)
        self.n = 0
        self.request("initialize", {"protocolVersion": "2025-06-18", "capabilities": {},
                                    "clientInfo": {"name": "cell-mcp-check", "version": "0"}})
        self.send({"jsonrpc": "2.0", "method": "notifications/initialized"})

    def send(self, msg):
        self.p.stdin.write(json.dumps(msg) + "\n")
        self.p.stdin.flush()

    def request(self, method, params):
        self.n += 1
        self.send({"jsonrpc": "2.0", "id": self.n, "method": method, "params": params})
        while True:
            line = self.p.stdout.readline()
            if not line:
                sys.exit("cell-mcp-check: the server stopped")
            r = json.loads(line)
            if r.get("id") == self.n:
                return r

    def tool(self, name, **args):
        r = self.request("tools/call", {"name": name, "arguments": args})["result"]
        out = json.loads(r["content"][0]["text"])
        if r.get("isError") or not all(o["ok"] for o in out.get("outcomes", [])):
            sys.exit(f"cell-mcp-check: {name} {args}: {out}")
        return out

    def close(self):
        self.p.stdin.close()
        self.p.wait(timeout=10)


def main():
    if not (root / "target/debug/xetal-cell-mcp").exists():
        sys.exit("cell-mcp-check: build the server first (cargo build -p xetal-cell-mcp)")
    if not (root / "extensions/cell/web/dist/index.html").exists():
        sys.exit("cell-mcp-check: build the page first (just ext cell live)")
    tmp = tempfile.mkdtemp()
    env = dict(os.environ, XETAL_CELL_PORT="8470", XETAL_CELL_SESSION=f"{tmp}/session.cell")
    agent = Agent(env)
    b = None
    try:
        # before the page: the DIP into the vise
        agent.tool("cell_reset", circuit="demo")
        agent.tool("cell_do", commands="tool A tweezers\npick A U1 grasp=ends")
        agent.tool("cell_place", arm="A", part="U1", at=[0, 0, -24], rpy=[0, 0, 90])
        agent.tool("cell_vise", action="close")
        agent.tool("cell_release", arm="A", part="U1")
        try:
            urllib.request.urlopen("http://127.0.0.1:8470/mcp/version", timeout=5).read()
        except OSError:
            sys.exit("cell-mcp-check: no page on port 8470 (is something else serving it?)")
        b = subprocess.Popen([rec.chrome(), "--headless=new", "--use-angle=metal", "--enable-gpu", "--ignore-gpu-blocklist",
                              "--no-first-run", "--no-default-browser-check", f"--user-data-dir={tmp}/p",
                              "--window-size=1280,720", "--remote-debugging-port=0", "about:blank"],
                             stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        pf = pathlib.Path(tmp, "p", "DevToolsActivePort")
        for _ in range(200):
            if pf.exists() and pf.read_text().strip():
                break
            time.sleep(0.1)
        port = pf.read_text().split()[0]
        page = next(p for p in json.loads(urllib.request.urlopen(f"http://127.0.0.1:{port}/json/list").read()) if p["type"] == "page")
        ws = rec.Socket(page["webSocketDebuggerUrl"])
        ws.send("Page.navigate", url="http://127.0.0.1:8470/?mcp&hud=0")

        def wait(expr, what, tries=300):
            for _ in range(tries):
                try:
                    v = rec.evaluate(ws, expr)
                    if v:
                        return v
                except RuntimeError:
                    pass
                time.sleep(0.1)
            try:
                seen = rec.evaluate(ws, "[document.title, (document.getElementById('err') || {}).textContent]")
            except RuntimeError as e:
                seen = str(e)
            sys.exit(f"cell-mcp-check: {what} (the page: {seen})")

        wait("!!(window.cell && document.title.endsWith(': ready'))", "the page did not get ready")
        holds = wait("document.getElementById('holds').textContent", "no controls")
        parts = rec.evaluate(ws, "[...document.getElementById('part').options].map((o) => o.textContent).join('; ')")
        if "U1 (dip8), held by vise" not in parts:
            sys.exit(f"cell-mcp-check: the page does not show the session (parts: {parts})")
        print("cell-mcp-check: the page shows the session (U1 in the vise)")
        # while the page is open: a resistor lead to pin 3
        agent.tool("cell_pick", arm="A", part="R1", grasp="body")
        agent.tool("cell_do", commands="lift A mm=80\nmove A to=0,60,60 rpy=180,0,90\nmove A to=34.81,1.27,-14.05\nalign A R1.1.tip to=U1.3.tip gap=0.1")
        log = wait("(() => { const t = document.getElementById('log').textContent; return t.includes('align A R1.1.tip') && t; })()",
                   "the page did not follow the agent's commands")
        holds = wait("(() => { const t = document.getElementById('holds').textContent; return t.includes('holding R1') && t; })()",
                     "the page's controls did not follow")
        print("cell-mcp-check: the page followed (pick, move, align played as they came)")
        # it plays to the end; then a picture
        end = json.loads(urllib.request.urlopen("http://127.0.0.1:8470/mcp/last").read())["t1"]
        wait(f"cell.state.t >= {end} - 0.01", "the page did not play to the end", tries=600)
        shot = ws.send("Page.captureScreenshot", format="png")
        out = root / "extensions/cell/work/check-mcp.png"
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_bytes(base64.b64decode(shot["data"]))
        print(f"cell-mcp-check: ok ({holds}; picture: {out.relative_to(root)})")
    finally:
        if b:
            b.terminate()
            b.wait()
        agent.close()


if __name__ == "__main__":
    main()
