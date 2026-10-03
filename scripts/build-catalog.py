#!/usr/bin/env python3
"""pages/index.html: the catalog of every extension, one card each: its
name, the first paragraph of extensions/NAME/docs/README.md, and a link
to its live page (extensions/NAME/web/) or, for an extension that runs
only on the command line, to its documentation.
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


REPO = "https://github.com/softwarewrighter/X_eTaL-extensions"
names = sorted(p.parent.name for p in root.glob("extensions/*/extension.toml"))


def card(n):
    live = (root / "extensions" / n / "web" / "Cargo.toml").exists()
    href = f"{n}/" if live else f"{REPO}/blob/main/extensions/{n}/docs/README.md"
    how = "live in your browser" if live else "command line (xetal-x): documentation"
    return (f'    <a class="card" href="{href}"><h2>{html.escape(n)}</h2>'
            f'<p>{html.escape(lede(n))}</p><p class="how">{how}</p></a>')


cards = "\n".join(card(n) for n in names)
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
  <p class="lede"><b>Libraries extend the vocabulary; macros extend the language; native
  extensions extend the machine.</b> These are X_eTaL's native extensions: Rust libraries
  that give programs in X_eTaL, the eXperimental Extensible Typed Array Language, what
  the language should not reinvent -- a database, a clock -- behind ordinary typed X_eTaL
  libraries. The live pages run with the Rust compiled to WebAssembly: edit a program and
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
