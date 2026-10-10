#!/usr/bin/env python3
"""pages/index.html: the extensions and their demos, each demo shown by
its recording (extensions/EXT/videos/NAME.webm: a terminal session by
vhs, or a window demo's frames) with the commands to run it yourself.
Extensions are native: they run with xetal-x on your machine, so the
site shows them running rather than running them.
    scripts/build-catalog.py   # PAGES_DIR=DIR builds elsewhere (the gate)
"""
import html
import os
import pathlib
import shutil
import subprocess

root = pathlib.Path(__file__).resolve().parent.parent
REPO = "https://github.com/softwarewrighter/X_eTaL-extensions"
pages = pathlib.Path(os.environ.get("PAGES_DIR", root / "pages"))


def literate():
    """The literate documents (docs/literate/*.org) by the demo each
    explains (its `#+DEMO: EXT NAME` line): {(ext, name): (page, title)}."""
    docs = {}
    for org in sorted((root / "docs/literate").glob("*.org")):
        text = org.read_text()
        demo = next((l.split()[1:3] for l in text.splitlines() if l.startswith("#+DEMO: ")), None)
        title = next((l[len("#+TITLE: "):] for l in text.splitlines() if l.startswith("#+TITLE: ")), org.stem)
        if demo and len(demo) == 2:
            docs[tuple(demo)] = (f"literate/{org.stem}.html", title)
    return docs


LITERATE = literate()


def lit_link(ext, name):
    """A demo's link to the literate document that explains it, if any."""
    if (ext, name) not in LITERATE:
        return ""
    page, title = LITERATE[(ext, name)]
    return f' <a class="docs" href="{page}" title="{html.escape(title)}">literate document</a>'


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
    return (root / "XETAL_COMMIT").read_text().strip()[:7] or "unknown"


sections = []
toc = []
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
        demos.append(f"""    <section class="panel demo" id="{ext}-{name}">
      <h3>{html.escape(ext)} / {html.escape(name)} <a class="docs" href="doc/extensions-{ext}-demos-{name}.xtl.html">source and docs</a>{lit_link(ext, name)}</h3>
      <p class="note">{html.escape(what(demo))}</p>
      {player}
      <pre class="source">just demo {ext} {name}</pre>
    </section>""")
    links = "".join(f'<li><a href="#{ext}-{d}">{html.escape(d)}</a></li>'
                    for d in sorted(d.stem for d in (ext_dir / "demos").glob("*.xtl") if d.stem[0].islower()))
    toc.append(f'    <li><a href="#{ext}">{html.escape(ext)}</a>'
               + (f"<ul>{links}</ul>" if links else "") + "</li>")
    facades = sorted(p.name for p in (ext_dir / "lib").glob("*.xtl"))
    fdoc = (f' <a class="docs" href="doc/extensions-{ext}-lib-{facades[0]}.html">facade docs</a>'
            if facades else "")
    sections.append(f"""  <h2 id="{ext}"><a href="{REPO}/blob/main/extensions/{ext}/docs/README.md">{html.escape(ext)}</a>{fdoc}</h2>
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
<nav class="toc" aria-label="Demos">
  <p><a href="#top">X_eTaL extensions</a></p>
  <p class="docs"><a href="doc/index.html">Docs: every facade, library and demo, cross-referenced</a></p>
  <ul>
{chr(10).join(toc)}
  </ul>
</nav>
<header id="top">
  <div class="brand">
    <img class="logo" src="xetal-logo-red.png" alt="X_eTaL: X underlined, a raised e, T, a raised a, L" />
    <h1>X_eTaL Extensions</h1>
  </div>
  <p class="lede"><b>Libraries extend the vocabulary; macros extend the language; native
  extensions extend the machine.</b> These are X_eTaL's native extensions: Rust shared
  libraries that X_eTaL programs load at run time through a small C ABI, behind ordinary
  typed X_eTaL libraries -- a database, a clock, a window. They run on your machine, so
  this page shows them running; the commands under each recording run them yourself.</p>
  <pre class="source">git clone {REPO}
cd X_eTaL-extensions
just xetal                  # X_eTaL at its known-good commit (work/xetal)
cargo build --workspace
just demos                  # the demos; then: just demo EXT NAME</pre>
</header>
<main>
{chr(10).join(sections)}
</main>
<footer>
  <span>Copyright (c) 2026 Michael A Wright</span><span class="sep">&middot;</span>
  <span>MIT License</span><span class="sep">&middot;</span>
  <a href="{REPO}">Repository</a><span class="sep">&middot;</span>
  <a href="literate/">Literate documents</a><span class="sep">&middot;</span>
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
