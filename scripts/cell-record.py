#!/usr/bin/env python3
"""A recording of the assembly cell's live page running a program: the
page (extensions/cell/web/dist, built by scripts/publish-cell.sh) served
on port 8470 and loaded once by one headless Chrome (no window, a
throwaway profile), which runs programs/NAME.cell (?program=NAME). When
the page is ready -- the simulator run, every model loaded and drawn --
each moment is sought through the page's own window.cell and captured
after two drawn frames (the DevTools protocol, spoken here with the
standard library), then the frames are joined by ffmpeg into
extensions/cell/work/record/NAME.mp4. A frame as dark as the bare
background (nothing drawn) fails the recording. `just videos cell NAME`
encodes it for the site (videos/NAME.film).
    scripts/cell-record.py NAME [--fps 8] [--speed 2]
"""
import argparse
import base64
import hashlib
import http.server
import json
import os
import pathlib
import shutil
import socket
import struct
import subprocess
import sys
import tempfile
import threading
import time
import urllib.request

root = pathlib.Path(__file__).resolve().parent.parent
CHROMES = ["/Applications/Google Chrome.app/Contents/MacOS/Google Chrome", "google-chrome", "chromium"]


def chrome():
    for c in [os.environ.get("CHROME", "")] + CHROMES:
        if c and (os.path.exists(c) or shutil.which(c)):
            return c
    sys.exit("cell-record: no Chrome (set CHROME)")


def serve(directory):
    http.server.SimpleHTTPRequestHandler.log_message = lambda *a: None
    handler = lambda *a: http.server.SimpleHTTPRequestHandler(*a, directory=str(directory))
    try:
        httpd = http.server.ThreadingHTTPServer(("127.0.0.1", 8470), handler)
    except OSError:
        sys.exit("cell-record: port 8470 is in use: stop what serves it")
    threading.Thread(target=httpd.serve_forever, daemon=True).start()
    return httpd


class Socket:
    """A WebSocket client, just enough for the DevTools protocol."""

    def __init__(self, url):
        host, rest = url[len("ws://"):].split("/", 1)
        h, p = host.split(":")
        self.s = socket.create_connection((h, int(p)))
        key = base64.b64encode(os.urandom(16)).decode()
        self.s.sendall((f"GET /{rest} HTTP/1.1\r\nHost: {host}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n"
                        f"Sec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n").encode())
        head = b""
        while b"\r\n\r\n" not in head:
            head += self.s.recv(1)
        want = base64.b64encode(hashlib.sha1((key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").encode()).digest()).decode()
        if want not in head.decode():
            sys.exit("cell-record: Chrome refused the connection")
        self.n = 0

    def exact(self, n):
        out = b""
        while len(out) < n:
            chunk = self.s.recv(n - len(out))
            if not chunk:
                raise ConnectionError("Chrome closed the connection")
            out += chunk
        return out

    def send(self, method, **params):
        self.n += 1
        data = json.dumps({"id": self.n, "method": method, "params": params}).encode()
        mask = os.urandom(4)
        n = len(data)
        head = bytes([0x81]) + (bytes([0x80 | n]) if n < 126 else bytes([0x80 | 126]) + struct.pack(">H", n) if n < 65536 else bytes([0x80 | 127]) + struct.pack(">Q", n))
        self.s.sendall(head + mask + bytes(b ^ mask[i % 4] for i, b in enumerate(data)))
        while True:
            m = self.receive()
            if m.get("id") == self.n:
                if "error" in m:
                    raise RuntimeError(f"{method}: {m['error']}")
                return m.get("result", {})

    def receive(self):
        data = b""
        while True:
            b0, b1 = self.exact(2)
            n = b1 & 0x7F
            if n == 126:
                n = struct.unpack(">H", self.exact(2))[0]
            elif n == 127:
                n = struct.unpack(">Q", self.exact(8))[0]
            data += self.exact(n)
            if b0 & 0x80:
                return json.loads(data)


def evaluate(ws, expression):
    r = ws.send("Runtime.evaluate", expression=expression, awaitPromise=True, returnByValue=True)
    if "exceptionDetails" in r:
        raise RuntimeError(f"{expression}: {r['exceptionDetails'].get('text')} {r.get('result', {}).get('description', '')}")
    return r.get("result", {}).get("value")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("name")
    ap.add_argument("--fps", type=int, default=8)
    ap.add_argument("--speed", type=float, default=2.0)
    a = ap.parse_args()
    site = root / "extensions/cell/web/dist"
    if not (site / "index.html").exists():
        sys.exit("cell-record: no page yet: scripts/publish-cell.sh")
    out = root / "extensions/cell/work/record"
    out.mkdir(parents=True, exist_ok=True)
    httpd = serve(site)
    tmp = tempfile.mkdtemp()
    browser = None
    try:
        # one headless Chrome, its own profile (never the user's), no window
        browser = subprocess.Popen([chrome(), "--headless=new", "--use-angle=metal", "--enable-gpu", "--ignore-gpu-blocklist",
                                    "--no-first-run", "--no-default-browser-check", f"--user-data-dir={tmp}/profile",
                                    "--hide-scrollbars", "--window-size=1280,720", "--remote-debugging-port=0", "about:blank"],
                                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        port_file = pathlib.Path(tmp, "profile", "DevToolsActivePort")
        for _ in range(200):
            if port_file.exists() and port_file.read_text().strip():
                break
            time.sleep(0.1)
        port = port_file.read_text().split()[0]
        pages = json.loads(urllib.request.urlopen(f"http://127.0.0.1:{port}/json/list").read())
        ws = Socket(next(p for p in pages if p["type"] == "page")["webSocketDebuggerUrl"])
        ws.send("Emulation.setDeviceMetricsOverride", width=1280, height=720, deviceScaleFactor=1, mobile=False)
        ws.send("Page.navigate", url=f"http://127.0.0.1:8470/?program={a.name}")
        # ready: the simulator ran the program and every model is drawn
        for _ in range(600):
            try:
                if evaluate(ws, "!!(window.cell && document.title.endsWith(': ready'))"):
                    break
            except RuntimeError:
                pass
            time.sleep(0.1)
        else:
            sys.exit("cell-record: the page did not get ready")
        duration = evaluate(ws, "cell.duration")
        frames = int(duration / a.speed * a.fps) + 1
        print(f"cell-record: {a.name}: {duration:.1f} s at {a.speed}x, {frames} frames from one page")
        for i in range(frames):
            t = min(duration, i * a.speed / a.fps)
            evaluate(ws, f"cell.seek({t}); new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(() => r(1))))")
            shot = ws.send("Page.captureScreenshot", format="png")
            pathlib.Path(tmp, f"frame-{i:05d}.png").write_bytes(base64.b64decode(shot["data"]))
        mp4 = out / f"{a.name}.mp4"
        subprocess.run(["ffmpeg", "-loglevel", "error", "-y", "-framerate", str(a.fps), "-i", f"{tmp}/frame-%05d.png",
                        "-pix_fmt", "yuv420p", str(mp4)], check=True)
    finally:
        if browser:
            browser.terminate()
            browser.wait()
        httpd.shutdown()
        httpd.server_close()
        shutil.rmtree(tmp, ignore_errors=True)
    # every frame has the scene: brighter than the bare background
    stats = subprocess.run(["ffmpeg", "-loglevel", "error", "-i", str(mp4), "-vf",
                            "crop=900:600:360:40,signalstats,metadata=print:key=lavfi.signalstats.YAVG:file=-", "-f", "null", "-"],
                           capture_output=True, text=True).stdout
    levels = [float(l.split("=")[1]) for l in stats.split() if l.startswith("lavfi.signalstats.YAVG=")]
    bare = [i + 1 for i, v in enumerate(levels) if v < 55]
    if bare:
        sys.exit(f"cell-record: {len(bare)} of {len(levels)} frames show nothing drawn (first: {bare[:5]})")
    print(f"cell-record: {mp4.relative_to(root)} ({len(levels)} frames, each drawn)")


if __name__ == "__main__":
    main()
