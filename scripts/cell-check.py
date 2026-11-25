#!/usr/bin/env python3
"""The cell player checked headlessly: a preview written by
scripts/cell-preview.sh NAME --write is served on port 8470 and drawn
by headless Chrome (no window, its own throwaway profile) with ?check;
the page seeks every expect line (an arm's flange at a time, from the
simulator) and reports the drawn flange's distance from it. Fails when
any is off by more than 0.01 mm, or the page does not finish. A
picture of the page at --at seconds is kept: work/preview/NAME/check.png.
    scripts/cell-check.py NAME [--at SECONDS]
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


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("name")
    ap.add_argument("--at", type=float, default=None)
    a = ap.parse_args()
    site = root / "extensions/cell/work/preview" / a.name
    if not (site / "scene.calls.js").exists():
        sys.exit(f"cell-check: nothing written: scripts/cell-preview.sh {a.name} --write")
    c = chrome()
    httpd = serve(site)
    try:
        with tempfile.TemporaryDirectory() as tmp:
            dom = run(c, f"{tmp}/a", ["--dump-dom"], "http://127.0.0.1:8470/?check&hud=0").stdout
            m = re.search(r'<pre id="check">(.*?)</pre>', dom, re.S)
            if not m:
                err = re.search(r'<div id="err">(.*?)</div>', dom, re.S)
                sys.exit("cell-check: the page did not report" + (f": {err.group(1)}" if err and err.group(1) else ""))
            report = json.loads(m.group(1).replace("&quot;", '"').replace("&amp;", "&"))
            at = a.at if a.at is not None else report["duration"] / 2
            run(c, f"{tmp}/b", [f"--screenshot={site / 'check.png'}"], f"http://127.0.0.1:8470/?t={at}")
    finally:
        httpd.shutdown()
    print(f"cell-check: {a.name}: {report['steps']} steps, {report['duration']:.3f} s; "
          f"arms {' '.join(report['arms'])}; fixtures {' '.join(report['fixtures'])}")
    print(f"cell-check: {report['expects']} positions (arms' flanges, parts, tools), the worst {report['worst']} mm from the simulator's")
    for r in report.get("off", []):
        print(f"cell-check:   {r['arm']} at {r['t']} s: drawn at {r['got']}, the simulator says {r['want']} ({r['off']} mm)")
    if report["expects"] == 0 or report["worst"] > 0.01:
        sys.exit("cell-check: the drawing is not the simulator's")
    print(f"cell-check: ok (picture at {at:.2f} s: extensions/cell/work/preview/{a.name}/check.png)")


if __name__ == "__main__":
    main()
