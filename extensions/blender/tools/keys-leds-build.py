#!/usr/bin/env python3
"""Export verified full Nano 1K build artifacts to scene arrays."""
import argparse
from collections import Counter
import importlib.util
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[3]
EXT = ROOT / "extensions/blender"
spec = importlib.util.spec_from_file_location("calculator_evidence", Path(__file__).with_name("keys-leds-evidence.py"))
evidence = importlib.util.module_from_spec(spec)
spec.loader.exec_module(evidence)


def category(kind):
    if re.fullmatch(r"LUT[1-4]", kind):
        return 0
    if kind.startswith("DFF"):
        return 1
    if kind == "ALU":
        return 2
    if kind in ("SP", "DP"):
        return 3
    if kind in ("IBUF", "OBUF", "TBUF"):
        return 4
    return 5


def arrays(cells, routing, build):
    if not cells or Counter(c["type"] for c in cells) != Counter(build["placed_counts"]):
        raise ValueError("placed cell counts differ")
    if len({c["name"] for c in cells}) != len(cells):
        raise ValueError("duplicate cell identity")
    rows = []
    counts = Counter()
    for cell in cells:
        match = re.fullmatch(r"X(\d+)Y(\d+)/([A-Za-z_]+)(\d*)", cell["bel"])
        if not match:
            raise ValueError("invalid or missing BEL")
        x, y, suffix, slot = match.groups()
        kind = category(cell["type"])
        counts[kind] += 1
        # Slot offsets are schematic; the tile and BEL identity are retained.
        rows.append([int(x), int(y), int(slot or (1 if suffix.endswith("B") else 0)), kind])
    u = build["utilization"]
    resources = [u[k]["used"] for k in ("LUT4", "DFF", "ALU", "BSRAM", "IOB")]
    tri = sum(c["type"] == "TBUF" for c in cells)
    if tri != u.get("TBUF", {}).get("used", 0):
        raise ValueError("tri-state resource totals disagree")
    resources[4] += tri
    if [counts[i] for i in range(5)] != resources:
        raise ValueError("resource totals disagree with placed cells")
    if len({c["bel"] for c in cells if category(c["type"]) == 4}) != counts[4]:
        raise ValueError("I/O pad BELs overlap")
    if not routing or len(routing) != build["routed_nets"]:
        raise ValueError("missing routed nets")
    tiles = set()
    for route in routing.values():
        footprint = re.findall(r"X(\d+)Y(\d+)/", route)
        if not footprint:
            raise ValueError("route has no wire tile coordinates")
        tiles.update((int(x), int(y)) for x, y in footprint)
    clocks = evidence.timing({"fmax": build["fmax_mhz"]})
    machine = clocks["machine_clk"]
    primary = next(v for k, v in clocks.items() if k != "machine_clk")
    stats = [resources[0], resources[1], len(cells), len(tiles), len(routing), build["bitstream_bytes"],
             resources[2], resources[3], resources[4], round(primary["achieved"] * 1000),
             round(machine["achieved"] * 1000), round(primary["constraint"] * 1000000),
             round(machine["constraint"] * 1000000), tri, u["IOB"]["used"]]
    return rows, sorted(tiles), stats


def export(source, out):
    subprocess.run(["bash", ROOT / "scripts/hex1000.sh", "--check"], check=True, stdout=subprocess.DEVNULL)
    manifest = evidence.verify(source)
    report = json.loads((source / "timing.json").read_text())
    build = manifest["build"]
    if build["fmax_mhz"] != report["fmax"] or build["utilization"] != report["utilization"]:
        raise ValueError("manifest timing/resources differ from router report")
    if build["bitstream_bytes"] != (source / "calculator.fs").stat().st_size:
        raise ValueError("manifest bitstream size differs from packed file")
    cells = json.loads((source / "cells.json").read_text())
    routes = json.loads((source / "routes.json").read_text())
    rows, tiles, stats = arrays(cells, routes, build)
    out.mkdir(parents=True, exist_ok=True)
    for name, values in (("cells", rows), ("routes", tiles), ("stats", [stats])):
        (out / (name + ".txt")).write_text("".join(" ".join(map(str, row)) + "\n" for row in values))
    record = {"source_evidence_sha256": evidence.sha(source / "evidence.json"),
              "project_commit": manifest["project_commit"], "cells": cells,
              "route_kind": "unique occupied wire tiles; not physical wire geometry",
              "array_sha256": {n: evidence.sha(out / n) for n in ("cells.txt", "routes.txt", "stats.txt")},
              "stats": stats, "hardware_actions": []}
    (out / "scene-evidence.json").write_text(json.dumps(record, indent=2) + "\n")
    print(f"PASS build arrays: {len(cells)} BEL identities, {len(tiles)} wire tiles, {len(routes)} routed nets")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=EXT / "work/keys-leds-evidence")
    parser.add_argument("--out", type=Path, default=EXT / "work/keysbuild")
    args = parser.parse_args()
    export(args.source.resolve(), args.out.resolve())
