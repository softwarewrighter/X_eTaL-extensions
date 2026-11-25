#!/usr/bin/env python3
"""The cell player checked headlessly: a preview written by
scripts/cell-preview.sh NAME --write is served on port 8470 and drawn
by headless Chrome (no window, its own throwaway profile) with ?check;
the page seeks every expect line (an arm's flange at a time, from the
simulator) and reports the drawn flange's distance from it. Fails when
any is off by more than 0.01 mm, or the page does not finish. Then
stepping: in one headless Chrome driven through the DevTools protocol
(scripts/cell-record.py's client), Next step from the start must play
the steps in turn, each ending at its end, the list highlighting it,
and Previous step come back a start at a time. A picture of the page at
--at seconds is kept: work/check-NAME.png.
    scripts/cell-check.py NAME [--at SECONDS]
    scripts/cell-check.py NAME --site DIR   # the live page (built in DIR)
                                            # running programs/NAME.cell
"""
import argparse
import http.server
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile
import threading

root = pathlib.Path(__file__).resolve().parent.parent
CHROMES = ["/Applications/Google Chrome.app/Contents/MacOS/Google Chrome", "google-chrome", "chromium"]


def chrome():
    for c in [os.environ.get("CHROME", "")] + CHROMES:
        if c and (os.path.exists(c) or shutil.which(c)):
            return c
    sys.exit("cell-check: no Chrome (set CHROME)")


def serve(directory):
    handler = lambda *a: http.server.SimpleHTTPRequestHandler(*a, directory=str(directory))
    http.server.SimpleHTTPRequestHandler.log_message = lambda *a: None
    try:
        httpd = http.server.ThreadingHTTPServer(("127.0.0.1", 8470), handler)
    except OSError:
        sys.exit("cell-check: port 8470 is in use: stop what serves it")
    threading.Thread(target=httpd.serve_forever, daemon=True).start()
    return httpd


def run(c, profile, args, url):
    # headless, its own profile (never the user's), no window
    cmd = [c, "--headless=new", "--use-angle=metal", "--enable-gpu", "--ignore-gpu-blocklist", "--no-first-run",
           "--no-default-browser-check", f"--user-data-dir={profile}", "--hide-scrollbars",
           "--window-size=1280,720", "--virtual-time-budget=15000"] + args + [url]
    return subprocess.run(cmd, capture_output=True, text=True, timeout=120)


def stepping(site, query, steps):
    """Next step through the first steps, then Previous step back: what
    went wrong, or nothing."""
    import importlib.util
    import json as js
    import time
    import urllib.request
    spec = importlib.util.spec_from_file_location("cell_record", pathlib.Path(__file__).with_name("cell-record.py"))
    rec = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(rec)
    httpd = serve(site)
    tmp = tempfile.mkdtemp()
    b = subprocess.Popen([chrome(), "--headless=new", "--use-angle=metal", "--enable-gpu", "--ignore-gpu-blocklist", "--no-first-run",
                          "--no-default-browser-check", f"--user-data-dir={tmp}/p", "--remote-debugging-port=0", "about:blank"],
                         stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        pf = pathlib.Path(tmp, "p", "DevToolsActivePort")
        for _ in range(200):
            if pf.exists() and pf.read_text().strip():
                break
            time.sleep(0.1)
        port = pf.read_text().split()[0]
        page = next(p for p in js.loads(urllib.request.urlopen(f"http://127.0.0.1:{port}/json/list").read()) if p["type"] == "page")
        ws = rec.Socket(page["webSocketDebuggerUrl"])
        ws.send("Page.navigate", url=f"http://127.0.0.1:8470/?{query}hud=1")
        for _ in range(600):
            try:
                if rec.evaluate(ws, "!!(window.cell && document.title.endsWith(': ready'))"):
                    break
            except RuntimeError:
                pass
            time.sleep(0.1)
        else:
            return "the page did not get ready for stepping"
        rec.evaluate(ws, "cell.speed = 64; cell.seek(0)")
        lit = "[...document.querySelectorAll('#cellsteps li')].findIndex((li) => li.style.color) + 1"
        start = rec.evaluate(ws, "cell.state.step")
        n = min(steps - start, 6)
        for k in range(start + 1, start + 1 + n):
            rec.evaluate(ws, "cell.next()")
            step, shown = rec.evaluate(ws, f"[cell.state.step, {lit}]")
            if (step, shown) != (k, k):
                return f"Next step went to step {step} (the list shows {shown}), not {k}"
        for k in range(start + n, start, -1):
            rec.evaluate(ws, "cell.previous()")
            step = rec.evaluate(ws, "cell.state.step")
            if step != k:
                return f"Previous step went to step {step}, not {k}"
        return None
    finally:
        b.terminate()
        b.wait()
        httpd.shutdown()
        httpd.server_close()
        shutil.rmtree(tmp, ignore_errors=True)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("name")
    ap.add_argument("--at", type=float, default=None)
    ap.add_argument("--site", default=None)
    a = ap.parse_args()
    if a.site:
        site = pathlib.Path(a.site).resolve()
        if not (site / "index.html").exists():
            sys.exit(f"cell-check: no page in {a.site}")
        query = f"program={a.name}&"
    else:
        site = root / "extensions/cell/work/preview" / a.name
        query = ""
        if not (site / "scene.calls.js").exists():
            sys.exit(f"cell-check: nothing written: scripts/cell-preview.sh {a.name} --write")
    c = chrome()
    httpd = serve(site)
    try:
        with tempfile.TemporaryDirectory() as tmp:
            dom = run(c, f"{tmp}/a", ["--dump-dom"], f"http://127.0.0.1:8470/?{query}check&hud=0").stdout
            m = re.search(r'<pre id="check">(.*?)</pre>', dom, re.S)
            if not m:
                err = re.search(r'<div id="err">(.*?)</div>', dom, re.S)
                sys.exit("cell-check: the page did not report" + (f": {err.group(1)}" if err and err.group(1) else ""))
            report = json.loads(m.group(1).replace("&quot;", '"').replace("&amp;", "&"))
            at = a.at if a.at is not None else report["duration"] / 2
            shot = root / "extensions/cell/work" / f"check-{a.name}{'-live' if a.site else ''}.png"
            shot.parent.mkdir(parents=True, exist_ok=True)
            run(c, f"{tmp}/b", [f"--screenshot={shot}"], f"http://127.0.0.1:8470/?{query}t={at}")
    finally:
        httpd.shutdown()
        httpd.server_close()
    print(f"cell-check: {a.name}: {report['steps']} steps, {report['duration']:.3f} s; "
          f"arms {' '.join(report['arms'])}; fixtures {' '.join(report['fixtures'])}")
    print(f"cell-check: {report['expects']} positions (arms' flanges, parts, tools), the worst {report['worst']} mm from the simulator's")
    for r in report.get("off", []):
        print(f"cell-check:   {r['arm']} at {r['t']} s: drawn at {r['got']}, the simulator says {r['want']} ({r['off']} mm)")
    if report["expects"] == 0 or report["worst"] > 0.01:
        sys.exit("cell-check: the drawing is not the simulator's")
    wrong = stepping(site, query, report["steps"])
    if wrong:
        sys.exit(f"cell-check: stepping: {wrong}")
    print("cell-check: stepping: Next step plays each step in turn, Previous step comes back")
    print(f"cell-check: ok (picture at {at:.2f} s: {shot.relative_to(root)})")


if __name__ == "__main__":
    main()
