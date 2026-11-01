#!/usr/bin/env python3
"""What the gate must check for the files changed since origin/main (the
commits not pushed yet and the working tree), as X_eTaL's own planner
does (../X_eTaL/scripts/affected.py, decisions D87 and D93 there).

    scripts/affected.py              # the plan, one line a part
    scripts/affected.py --self-test  # the rules on made-up changes

The plan's lines:

    exts: NAME ...      the extensions whose goldens run ("all", or none)
    rust: PACKAGE ...   the Rust packages linted and tested ("all", or none)
    xetal-x: yes|no     the bridge host checked against the X_eTaL CLI
    web: yes|no         the live pages' crates built

Everything is checked when shared code changed: the crates (the ABI,
SDK, loader, bridge, xetal-x), the workspace's Cargo files, the pinned
X_eTaL or X_eTaL-libraries commit, the gate's own scripts. An extension
is checked when a file in it changed, and so is every extension whose
programs use its facade (blender's tests hash with digest, say).
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

root = Path(__file__).resolve().parent.parent

# changes that check everything
SHARED = re.compile(
    r"^(crates/|Cargo\.(toml|lock)$|XETAL_COMMIT$|LIBRARIES_COMMIT$|GPU_COMMIT$|FPGA_COMMIT$|templates/|"
    r"scripts/(gate\.sh|affected\.py|test-exts\.sh|reg-ext\.sh|xetal\.sh|libraries\.sh|check-rust\.sh)$)"
)
# changes the bridge host's check depends on
XETAL_X = re.compile(r"^(crates/|Cargo\.(toml|lock)$|XETAL_COMMIT$|scripts/check-xetal-x\.sh$)")
# changes the live pages' crates depend on
WEB = re.compile(r"^(web/|extensions/[^/]+/web/|crates/|scripts/check-web\.sh$|XETAL_COMMIT$)")


def changed() -> list[str]:
    """The files changed since origin/main: committed, staged, unstaged
    and untracked."""
    def git(*args: str) -> list[str]:
        out = subprocess.run(["git", *args], cwd=root, capture_output=True, text=True, check=True).stdout
        return [l for l in out.splitlines() if l]
    base = git("merge-base", "HEAD", "origin/main")[0]
    files = git("diff", "--name-only", base) + git("ls-files", "--others", "--exclude-standard")
    return sorted(set(files))


def extensions() -> dict[str, dict]:
    """Each extension: its Rust package, its facades and shared demo
    libraries, and the libraries its programs use."""
    found = {}
    for toml in sorted(root.glob("extensions/*/extension.toml")):
        d = toml.parent
        cargo = (d / "rust/Cargo.toml").read_text()
        package = re.search(r'^name\s*=\s*"([^"]+)"', cargo, re.M).group(1)
        # its facades and its shared demo libraries (capitalized: Td, Stats)
        facades = {p.stem for p in (d / "lib").glob("*.xtl")} | {p.stem for p in (d / "demos").glob("[A-Z]*.xtl")}
        uses = set()
        for xtl in list(d.glob("demos/*.xtl")) + list(d.glob("tests/*.xtl")) + list(d.glob("lib/*.xtl")):
            uses |= set(re.findall(r'u_se<\s*"([^"]+)"', xtl.read_text()))
        found[d.name] = {"package": package, "facades": facades, "uses": uses}
    return found


def plan(files: list[str], exts: dict[str, dict]) -> dict[str, object]:
    """The checks the changed files call for."""
    if any(SHARED.match(f) for f in files):
        return {"exts": "all", "rust": "all", "xetal-x": True, "web": True}
    touched = {f.split("/")[1] for f in files if f.startswith("extensions/") and f.split("/")[1] in exts}
    rust = sorted(exts[e]["package"] for e in touched
                  if any(f.startswith(f"extensions/{e}/rust/") for f in files))
    # the extensions whose programs use a touched extension's facades, and so on
    goldens = set(touched)
    grew = True
    while grew:
        grew = False
        for e, info in exts.items():
            if e not in goldens and any(info["uses"] & exts[t]["facades"] for t in goldens):
                goldens.add(e)
                grew = True
    return {
        "exts": sorted(goldens),
        "rust": rust,
        "xetal-x": any(XETAL_X.match(f) for f in files),
        "web": any(WEB.match(f) for f in files),
    }


def show(p: dict[str, object]) -> str:
    def words(v):
        return v if isinstance(v, str) else " ".join(v)
    yes = {True: "yes", False: "no"}
    return (f"exts: {words(p['exts'])}\nrust: {words(p['rust'])}\n"
            f"xetal-x: {yes[p['xetal-x']]}\nweb: {yes[p['web']]}")


def self_test() -> None:
    exts = {
        "digest": {"package": "xetal-ext-digest", "facades": {"Digest"}, "uses": {"Ffi"}},
        "blender": {"package": "xetal-ext-blender", "facades": {"Blender", "Bpy"}, "uses": {"Blender", "Bpy", "Digest", "Td"}},
        "clock": {"package": "xetal-ext-clock", "facades": {"Clock"}, "uses": {"Clock"}},
    }
    cases = [
        ([], {"exts": [], "rust": [], "xetal-x": False, "web": False}),
        (["CHANGES.md", "docs/plan.md"], {"exts": [], "rust": [], "xetal-x": False, "web": False}),
        (["extensions/blender/demos/Td.xtl"], {"exts": ["blender"], "rust": [], "xetal-x": False, "web": False}),
        (["extensions/digest/rust/src/lib.rs"], {"exts": ["blender", "digest"], "rust": ["xetal-ext-digest"], "xetal-x": False, "web": False}),
        (["crates/xetal-ext-sdk/src/lib.rs"], {"exts": "all", "rust": "all", "xetal-x": True, "web": True}),
        (["XETAL_COMMIT"], {"exts": "all", "rust": "all", "xetal-x": True, "web": True}),
        (["scripts/gate.sh"], {"exts": "all", "rust": "all", "xetal-x": True, "web": True}),
        (["web/shell/src/main.rs"], {"exts": [], "rust": [], "xetal-x": False, "web": True}),
        (["scripts/check-xetal-x.sh"], {"exts": [], "rust": [], "xetal-x": True, "web": False}),
    ]
    for files, want in cases:
        got = plan(files, exts)
        assert got == want, f"{files}: {got} != {want}"
    print(f"affected: self-test ok ({len(cases)} cases)")


if __name__ == "__main__":
    if sys.argv[1:] == ["--self-test"]:
        self_test()
    else:
        print(show(plan(changed(), extensions())))
