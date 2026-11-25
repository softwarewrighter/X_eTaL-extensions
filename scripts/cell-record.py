#!/usr/bin/env python3
"""A recording of the assembly cell's live page running a program: the
page (extensions/cell/web/dist, built by scripts/publish-cell.sh) served
on port 8470 and drawn by headless Chrome (no window, a throwaway
profile each) at moments of programs/NAME.cell -- ?program=NAME&t=T --
a few at a time, then joined by ffmpeg into
extensions/cell/work/record/NAME.mp4. `just videos cell NAME` encodes
it for the site (videos/NAME.film).
    scripts/cell-record.py NAME [--fps 8] [--speed 2] [--until SECONDS]
"""
import argparse
import concurrent.futures
import pathlib
import subprocess
import sys
import tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import importlib.util

spec = importlib.util.spec_from_file_location("cell_check", pathlib.Path(__file__).with_name("cell-check.py"))
check = importlib.util.module_from_spec(spec)
spec.loader.exec_module(check)

root = pathlib.Path(__file__).resolve().parent.parent


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("name")
    ap.add_argument("--fps", type=int, default=8)
    ap.add_argument("--speed", type=float, default=2.0)
    ap.add_argument("--until", type=float, default=None)
    a = ap.parse_args()
    site = root / "extensions/cell/web/dist"
    if not (site / "index.html").exists():
        sys.exit("cell-record: no page yet: scripts/publish-cell.sh")
    c = check.chrome()
    httpd = check.serve(site)
    out = root / "extensions/cell/work/record"
    out.mkdir(parents=True, exist_ok=True)
    try:
        with tempfile.TemporaryDirectory() as tmp:
            # the program's length, from the page itself
            import json
            import re
            dom = check.run(c, f"{tmp}/len", ["--dump-dom"], f"http://127.0.0.1:8470/?program={a.name}&check&hud=0").stdout
            m = re.search(r'<pre id="check">(.*?)</pre>', dom, re.S)
            if not m:
                sys.exit("cell-record: the page did not load the program")
            duration = json.loads(m.group(1).replace("&quot;", '"').replace("&amp;", "&"))["duration"]
            end = min(duration, a.until) if a.until else duration
            frames = int(end / a.speed * a.fps) + 1
            print(f"cell-record: {a.name}: {end:.1f} s at {a.speed}x, {frames} frames")

            def shot(i):
                t = min(end, i * a.speed / a.fps)
                png = pathlib.Path(tmp) / f"frame-{i:05d}.png"
                check.run(c, f"{tmp}/p{i}", [f"--screenshot={png}"], f"http://127.0.0.1:8470/?program={a.name}&t={t:.3f}")
                return png.exists()

            with concurrent.futures.ThreadPoolExecutor(4) as pool:
                done = list(pool.map(shot, range(frames)))
            if not all(done):
                sys.exit(f"cell-record: {done.count(False)} frames missing")
            mp4 = out / f"{a.name}.mp4"
            subprocess.run(["ffmpeg", "-loglevel", "error", "-y", "-framerate", str(a.fps), "-i", f"{tmp}/frame-%05d.png",
                            "-pix_fmt", "yuv420p", str(mp4)], check=True)
    finally:
        httpd.shutdown()
    print(f"cell-record: {mp4.relative_to(root)}")


if __name__ == "__main__":
    main()
