#!/usr/bin/env python3
"""Every export documented: in the binding macro (lib/*.xtlm), every
facade (extensions/*/lib/*.xtl) and every library the demos share
(extensions/*/demos/ with a capital letter), each `l:` definition and
each facade signature line has a `##` block directly above it, and the
file starts with one (X_eTaL's convention, lang-choices S9: `##`
documents, `###` is a section heading, `#` is ignored by `xetal doc`).
Then every `## >>` example is run (`xetal doc --test`, from the file's
directory), as rustdoc runs doc tests.
    scripts/check-docs.py            # list what is missing; fail if any
"""
import os
import pathlib
import re
import subprocess
import sys

root = pathlib.Path(__file__).resolve().parent.parent
EXPORT = re.compile(r'^(l:[A-Za-z][A-Za-z0-9_!]*\s*:=|"[A-Za-z][A-Za-z0-9_!]* : )')
files = sorted(root.glob("lib/*.xtlm")) + sorted(root.glob("extensions/*/lib/*.xtl")) + sorted(
    p for p in root.glob("extensions/*/demos/*.xtl") if p.stem[0].isupper())
missing = []
for f in files:
    lines = f.read_text().split("\n")
    rel = f.relative_to(root)
    if not lines[0].startswith("##"):
        missing.append(f"{rel}:1: no ## block at the top (the file's description)")
    for i, line in enumerate(lines):
        if EXPORT.match(line) and not (i > 0 and lines[i - 1].startswith("##")):
            name = line.split(":=")[0].split(" : ")[0].strip().strip('"')
            missing.append(f"{rel}:{i + 1}: {name} has no ## block directly above it")
for m in missing:
    print(m)
if missing:
    sys.exit(f"check-docs: {len(missing)} undocumented (## above each export)")
xetal = subprocess.run([str(root / "scripts/build-xetal.sh")], capture_output=True,
                       text=True, check=True).stdout.strip()
tested = 0
for f in files:
    if "## >>" not in f.read_text():
        continue
    env = dict(os.environ, XETAL_PATH=str(root / "lib"))
    r = subprocess.run([xetal, "doc", "--test", f.name], cwd=f.parent, env=env,
                       capture_output=True, text=True)
    if r.returncode != 0:
        print(r.stdout + r.stderr)
        sys.exit(f"check-docs: examples failed in {f.relative_to(root)}")
    tested += 1
print(f"check-docs: ok ({len(files)} files, every export documented; examples run in {tested})")
