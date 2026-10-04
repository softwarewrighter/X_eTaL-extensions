#!/usr/bin/env python3
"""pages/index.html: the extensions and their demos, each demo shown by
its recording (extensions/EXT/videos/NAME.webm: a terminal session by
vhs, or a window demo's frames) with the commands to run it yourself.
Extensions are native: they run with xetal-x on your machine, so the
site shows them running rather than running them.
    scripts/build-catalog.py
"""
import html
import pathlib
import shutil
import subprocess

root = pathlib.Path(__file__).resolve().parent.parent
REPO = "https://github.com/softwarewrighter/X_eTaL-extensions"
pages = root / "pages"


def first_para(name):
    text = (root / "extensions" / name / "docs" / "README.md").read_text()
    paras = [p.strip() for p in text.split("\n\n")]
    para = next(p for p in paras if p and not p.startswith("#"))
    return " ".join(para.replace("`", "").split())


def what(demo):
    lines = []
    for line in demo.read_text().splitlines():
        if not line.startswith("#"):
            break
        t = line.lstrip("# ").rstrip()
        if t.startswith("Run:") or not t:
            break
        lines.append(t)
    return " ".join(lines)


def vendored():
    for line in (root / "vendor/xetal/VENDORED").read_text().splitlines():
        if line.startswith('commit = "'):
            return line.split('"')[1][:7]
    return "unknown"


sections = []
for ext_dir in sorted(p.parent for p in root.glob("extensions/*/extension.toml")):
    ext = ext_dir.name
    demos = []
    for demo in sorted(d for d in (ext_dir / "demos").glob("*.xtl") if d.stem[0].islower()):
        name = demo.stem
        video = ext_dir / "videos" / f"{name}.webm"
        if video.exists():
            (pages / ext).mkdir(parents=True, exist_ok=True)
            shutil.copy(video, pages / ext / f"{name}.webm")
            player = (f'<video src="{ext}/{name}.webm" autoplay loop muted playsinline '
                      f'controls preload="metadata"></video>')
        else:
            player = '<p class="note">(no recording yet)</p>'
        demos.append(f"""    <section class="panel demo">
      <h3>{html.escape(ext)} / {html.escape(name)}</h3>
      <p class="note">{html.escape(what(demo))}</p>
      {player}
      <pre class="source">just demo {ext} {name}</pre>
    </section>""")
    sections.append(f"""  <h2 id="{ext}"><a href="{REPO}/blob/main/extensions/{ext}/docs/README.md">{html.escape(ext)}</a></h2>
  <p class="lede">{html.escape(first_para(ext))}</p>
  <div class="demos">
{chr(10).join(demos) if demos else '    <p class="note">No demo yet.</p>'}
  </div>""")

sha = subprocess.run(["git", "rev-parse", "--short", "HEAD"], cwd=root,
                     capture_output=True, text=True).stdout.strip() or "unknown"
page = f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>X_eTaL extensions -- demos</title>
  <link rel="icon" href="favicon.ico" />
  <link rel="stylesheet" href="shell.css" />
</head>
<body>
<header>
  <div class="brand">
    <img class="logo" src="modern-xetal-logo.jpg" alt="X_eTaL" />
    <h1>X_eTaL extensions</h1>
  </div>
  <p class="lede"><b>Libraries extend the vocabulary; macros extend the language; native
  extensions extend the machine.</b> These are X_eTaL's native extensions: Rust shared
  libraries that X_eTaL programs load at run time through a small C ABI, behind ordinary
  typed X_eTaL libraries -- a database, a clock, a window. They run on your machine, so
  this page shows them running; the commands under each recording run them yourself.</p>
  <pre class="source">git clone {REPO}
cd X_eTaL-extensions
cargo build --workspace     # Rust is all it needs: X_eTaL is vendored
just demos                  # the demos; then: just demo EXT NAME</pre>
</header>
<main>
{chr(10).join(sections)}
</main>
<footer>
  <span>Copyright (c) 2026 Michael A Wright</span><span class="sep">&middot;</span>
  <span>MIT License</span><span class="sep">&middot;</span>
  <a href="{REPO}">Repository</a><span class="sep">&middot;</span>
  <a href="https://softwarewrighter.github.io/X_eTaL/">X_eTaL live</a><span class="sep">&middot;</span>
  <a href="https://softwarewrighter.github.io/X_eTaL-demos/">X_eTaL demos</a><span class="sep">&middot;</span>
  <span>X_eTaL {vendored()}</span><span class="sep">&middot;</span>
  <span>built at {sha}</span>
</footer>
</body>
</html>
"""
(pages / "index.html").write_text(page)
print(f"pages/index.html: {len(sections)} extensions")
