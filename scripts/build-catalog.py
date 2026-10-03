#!/usr/bin/env python3
"""pages/index.html: the catalog of live extension pages, one card per
extensions/NAME/web/, its title the extension's name and its text the
first paragraph of extensions/NAME/docs/README.md.
    scripts/build-catalog.py
"""
import html
import pathlib
import subprocess

root = pathlib.Path(__file__).resolve().parent.parent


def lede(name):
    text = (root / "extensions" / name / "docs" / "README.md").read_text()
    paras = [p.strip() for p in text.split("\n\n")]
    para = next(p for p in paras if p and not p.startswith("#"))
    return " ".join(para.replace("`", "").split())


def vendored():
    for line in (root / "vendor/xetal/VENDORED").read_text().splitlines():
        if line.startswith('commit = "'):
            return line.split('"')[1][:7]
    return "unknown"


names = sorted(p.parent.parent.name for p in root.glob("extensions/*/web/Cargo.toml"))
cards = "\n".join(
    f'    <a class="card" href="{n}/"><h2>{html.escape(n)}</h2><p>{html.escape(lede(n))}</p></a>'
    for n in names
)
sha = subprocess.run(["git", "rev-parse", "--short", "HEAD"], cwd=root,
                     capture_output=True, text=True).stdout.strip() or "unknown"
page = f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>X_eTaL extensions -- live</title>
  <link rel="icon" href="favicon.ico" />
  <link rel="stylesheet" href="shell.css" />
</head>
<body>
<header>
  <div class="brand">
    <img class="logo" src="modern-xetal-logo.jpg" alt="X_eTaL" />
    <h1>X_eTaL extensions</h1>
  </div>
  <p class="lede">Native Rust extensions for X_eTaL, the eXperimental Extensible Typed
  Array Language, compiled to WebAssembly and running in your browser: each page runs an
  X_eTaL program that calls Rust through an ordinary X_eTaL library. Edit the program and
  run it again.</p>
</header>
<main>
  <div class="cards">
{cards}
  </div>
</main>
<footer>
  <span>Copyright (c) 2026 Michael A Wright</span><span class="sep">&middot;</span>
  <span>MIT License</span><span class="sep">&middot;</span>
  <a href="https://github.com/softwarewrighter/X_eTaL-extensions">Repository</a><span class="sep">&middot;</span>
  <a href="https://softwarewrighter.github.io/X_eTaL/">X_eTaL live</a><span class="sep">&middot;</span>
  <a href="https://softwarewrighter.github.io/X_eTaL-demos/">X_eTaL demos</a><span class="sep">&middot;</span>
  <span>X_eTaL {vendored()}</span><span class="sep">&middot;</span>
  <span>built at {sha}</span>
</footer>
</body>
</html>
"""
(root / "pages" / "index.html").write_text(page)
print(f"pages/index.html: {len(names)} extension(s)")
