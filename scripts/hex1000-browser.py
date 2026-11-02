#!/usr/bin/env python3
"""HEX1000's browser calculator typed at in headless Chrome, for the
blender extension's hex-run video: X_eTaL-fpga's live page (the X_eTaL
engine in WebAssembly running Hex1000.xtl, the machine running its ROM)
is opened, the text typed a key at a time, each once the page is ready
for it, and when it is ready again its calculator (the screen and the
keys) is saved as DIR/browser.png and the screen's text as
DIR/browser.txt. Chrome is driven over its DevTools pipe (standard
library only); no window opens.
  .venv/bin/python3 scripts/hex1000-browser.py TEXT DIR [URL]
"""
import base64
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time

URL = "https://hardwarewrighter.github.io/X_eTaL-fpga/calc/"
CHROMES = ["/Applications/Google Chrome.app/Contents/MacOS/Google Chrome", "google-chrome", "chromium"]


class Chrome:
    """Headless Chrome and its DevTools protocol on fds 3 and 4."""

    def __init__(self):
        exe = os.environ.get("CHROME") or next((c for c in CHROMES if shutil.which(c) or os.access(c, os.X_OK)), None)
        if not exe:
            sys.exit("hex1000-browser: no Chrome found (set CHROME)")
        self.profile = tempfile.mkdtemp()
        to_chrome_r, self.to_chrome = os.pipe()
        self.from_chrome, from_chrome_w = os.pipe()

        def fds():
            # the child reads commands on 3 and writes replies on 4
            a, b = os.dup(to_chrome_r), os.dup(from_chrome_w)
            os.dup2(a, 3)
            os.dup2(b, 4)

        self.proc = subprocess.Popen(
            [exe, "--headless=new", "--remote-debugging-pipe", "--no-first-run", "--hide-scrollbars",
             "--user-data-dir=" + self.profile, "--window-size=900,1000", "--force-dark-mode", "about:blank"],
            close_fds=False, preexec_fn=fds, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        os.close(to_chrome_r)
        os.close(from_chrome_w)
        self.buf = b""
        self.n = 0
        self.session = None

    def call(self, method, **params):
        self.n += 1
        msg = {"id": self.n, "method": method, "params": params}
        if self.session:
            msg["sessionId"] = self.session
        os.write(self.to_chrome, json.dumps(msg).encode() + b"\0")
        while True:
            while b"\0" not in self.buf:
                chunk = os.read(self.from_chrome, 1 << 20)
                if not chunk:
                    sys.exit("hex1000-browser: Chrome went away")
                self.buf += chunk
            line, self.buf = self.buf.split(b"\0", 1)
            reply = json.loads(line)
            if reply.get("id") == self.n:
                if "error" in reply:
                    sys.exit("hex1000-browser: %s: %s" % (method, reply["error"]))
                return reply["result"]

    def open(self, url):
        target = self.call("Target.createTarget", url="about:blank")["targetId"]
        self.session = self.call("Target.attachToTarget", targetId=target, flatten=True)["sessionId"]
        self.call("Emulation.setDeviceMetricsOverride", width=900, height=1000, deviceScaleFactor=2, mobile=False)
        self.call("Page.enable")
        self.call("Page.navigate", url=url)

    def js(self, expression):
        return self.call("Runtime.evaluate", expression=expression, returnByValue=True)["result"].get("value")

    def wait(self, expression, seconds, what):
        end = time.time() + seconds
        while time.time() < end:
            if self.js(expression):
                return
            time.sleep(0.2)
        sys.exit("hex1000-browser: timed out waiting for " + what)

    def close(self):
        self.proc.terminate()
        self.proc.wait()
        shutil.rmtree(self.profile, ignore_errors=True)


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    text, out = sys.argv[1], sys.argv[2]
    url = sys.argv[3] if len(sys.argv) > 3 else URL
    os.makedirs(out, exist_ok=True)
    ready = "(document.getElementById('status') || {}).textContent === 'ready for a key'"
    c = Chrome()
    try:
        c.open(url)
        c.wait(ready, 120, "the calculator to start")
        for ch in text:
            c.js("document.dispatchEvent(new KeyboardEvent('keydown', {key: %s}))" % json.dumps(ch))
            time.sleep(0.3)
            c.wait(ready, 300, "the key " + ch)
        screen = c.js("document.getElementById('screen').textContent")
        box = c.js("(() => { const a = document.getElementById('screen').getBoundingClientRect(),"
                   " b = document.getElementById('keys').getBoundingClientRect();"
                   " return [a.left, a.top, Math.max(a.right, b.right) - a.left, b.bottom - a.top]; })()")
        clip = dict(x=box[0] - 12, y=box[1] - 12, width=box[2] + 24, height=box[3] + 24, scale=1)
        shot = c.call("Page.captureScreenshot", format="png", clip=clip)["data"]
    finally:
        c.close()
    with open(os.path.join(out, "browser.png"), "wb") as f:
        f.write(base64.b64decode(shot))
    with open(os.path.join(out, "browser.txt"), "w") as f:
        f.write(screen + "\n")
    print("hex1000-browser: %s typed at %s; the screen:\n%s" % (text, url, screen))


if __name__ == "__main__":
    main()
