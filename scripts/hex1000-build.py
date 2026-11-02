#!/usr/bin/env python3
"""Build the pinned HEX1000 board and export placement/routing evidence.

Run with the repository's uv Python and the OSS CAD Suite on PATH.
Outputs stay outside the pinned clone; no hardware is accessed.
"""
import argparse
from collections import Counter
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess

ROOT = Path(__file__).resolve().parent.parent
XY = re.compile(r"X(\d+)Y(\d+)/")


def run(command, directory, label, cwd=ROOT):
    result = subprocess.run(list(map(str, command)), cwd=cwd, capture_output=True, text=True)
    (directory / f"{label}.log").write_text(result.stdout + result.stderr)
    if result.returncode:
        raise ValueError(f"{label} failed; see {directory / (label + '.log')}")
    return result.stdout


def export(directory):
    synth = json.loads((directory / "synth.json").read_text())["modules"]["hex1000_top"]
    placed = json.loads((directory / "pnr.json").read_text())["modules"]["top"]
    counts = Counter(c["type"] for c in synth["cells"].values())
    cells = []
    names = []
    for name, cell in sorted(placed["cells"].items()):
        kind = cell["type"]
        if not (kind.startswith("LUT") or kind.startswith("DFF")):
            continue
        bel = cell.get("attributes", {}).get("NEXTPNR_BEL", "")
        match = XY.match(bel)
        if not match:
            raise ValueError(f"unplaced logic cell: {name}")
        x, y = map(int, match.groups())
        slot = re.search(r"(\d+)$", bel)
        cells.append((x, y, int(slot[1]) if slot else 0, int(kind.startswith("DFF"))))
        names.append({"name": name, "type": kind, "bel": bel})
    if not cells:
        raise ValueError("no placed LUTs or flip-flops")
    # ROUTING is triples (wire, incoming pip, strength). Show occupied tile
    # coordinates, not invented straight connections between endpoints.
    tiles = set()
    routed = 0
    for net in placed["netnames"].values():
        routing = net.get("attributes", {}).get("ROUTING", "").strip()
        if not routing:
            continue
        fields = routing.split(";")
        if len(fields) % 3:
            raise ValueError("unsupported nextpnr ROUTING format")
        routed += 1
        for wire in fields[::3]:
            match = XY.match(wire)
            if match:
                tiles.add(tuple(map(int, match.groups())))
    if not tiles or not routed:
        raise ValueError("no routed wire coordinates")
    for file, rows in (("cells.txt", cells), ("routes.txt", sorted(tiles))):
        (directory / file).write_text("\n".join(" ".join(map(str, row)) for row in rows) + "\n")
    lut = sum(v for k, v in counts.items() if re.fullmatch(r"LUT[1-4]", k))
    ff = sum(v for k, v in counts.items() if k.startswith("DFF"))
    bitstream = directory / "hex1000.fs"
    if bitstream.stat().st_size == 0:
        raise ValueError("empty bitstream")
    (directory / "stats.txt").write_text(f"{lut} {ff} {len(cells)} {len(tiles)} {routed} {bitstream.stat().st_size}\n")
    files = ("design.v", "design.cst", "synth.json", "pnr.json", "hex1000.fs")
    manifest = {
        "fpga_commit": (ROOT / "FPGA_COMMIT").read_text().strip(),
        "device": "GW1NR-LV9QN88PC6/I5", "family": "GW1N-9C",
        "synthesis_counts": dict(sorted(counts.items())), "cells": names,
        "routed_nets": routed, "route_tiles": len(tiles),
        "sha256": {f: hashlib.sha256((directory / f).read_bytes()).hexdigest() for f in files},
        "source_note": "hx:s_tep is hand-lowered to XIR upstream; the pinned Verilog is emitted from that XIR. Board top and UART are included.",
        "route_note": "Occupied nextpnr wire tiles; no physical wire lengths or die dimensions are inferred.",
    }
    (directory / "evidence.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"HEX1000: {lut} LUTs, {ff} flip-flops; {len(tiles)} route tiles; {bitstream.stat().st_size} bitstream bytes")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--export-only", action="store_true", help="read existing build artifacts")
    args = parser.parse_args()
    if Path(os.environ.get("VIRTUAL_ENV", "")).resolve() != ROOT / ".venv":
        parser.error("run through just ext blender hex-build-data (the repository uv environment)")
    directory = args.directory.resolve()
    directory.mkdir(parents=True, exist_ok=True)
    if not args.export_only:
        for tool in ("yosys", "nextpnr-himbaechel", "gowin_pack", "cargo"):
            if not shutil.which(tool):
                parser.error(f"{tool} missing; put the OSS CAD Suite on PATH")
        subprocess.run([ROOT / "scripts/libraries.sh"], check=True, stdout=subprocess.DEVNULL)
        fpga = ROOT / "work/fpga"
        env = dict(os.environ, CARGO_TARGET_DIR=str(ROOT / "work/fpga-target"))
        subprocess.run(["cargo", "build", "-q", "--release", "-p", "xetal-fpga-cli"], cwd=fpga / "components", env=env, check=True)
        uart = (fpga / "boards/examples/uart.xir").read_text()
        clocks = round(27000000 / 115200)
        uart = uart.replace("%dm1 = const i16 2", f"%dm1 = const i16 {clocks - 1}")
        uart = uart.replace("%rxk = const i16 3", f"%rxk = const i16 {clocks + clocks // 2 - 1}")
        (directory / "uart.xir").write_text(uart)
        serial = run([ROOT / "work/fpga-target/release/xetal-fpga", "verilog", directory / "uart.xir"], directory, "uart")
        design = (fpga / "boards/tangnano9k/hex1000-top.v").read_text() + "\n" + serial + "\n" + (fpga / "libs/Hex1000/rtl/hex1000.v").read_text()
        (directory / "design.v").write_text(design)
        cst = (fpga / "boards/tangnano9k/uart.cst").read_text() + 'IO_LOC "led" 10;\nIO_PORT "led" IO_TYPE=LVCMOS33 PULL_MODE=UP DRIVE=8;\n'
        (directory / "design.cst").write_text(cst)
        for stage, source, output, extra in (
            ("synth", "design.v", "synth.json", ["--top", "hex1000_top", "--cst", directory / "design.cst"]),
            ("pnr", "synth.json", "pnr.json", ["--cst", directory / "design.cst", "--sdc", fpga / "boards/tangnano9k/hex1000.sdc"]),
            ("pack", "pnr.json", "hex1000.fs", []),
        ):
            run([ROOT / ".venv/bin/python3", fpga / "scripts/board-flow.py", stage, directory / source, "--out", directory / output, *extra], directory, stage, cwd=fpga)
    export(directory)


if __name__ == "__main__":
    main()
