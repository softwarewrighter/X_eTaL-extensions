#!/usr/bin/env python3
"""Simulate and build the complete pinned 1K calculator outside its clone.

The build and constraint checks follow hardwarewrighter/hex1000's
scripts/board-calculator.py at HEX1000_COMMIT. Only simulation, synthesis,
place/route and packing are available here; no programming tool is called.
"""
import argparse
from collections import Counter
import hashlib
import json
import math
import os
from pathlib import Path
import re
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[3]
CLONE = ROOT / "work/hex1000"
BOARD = "boards/tangnano1k/"
SOURCES = [BOARD + name for name in ("calculator.v", "calculator.cst", "calculator.sdc", "calculator.tb.v")]
SOURCES += ["libs/Front/rtl/front.v", "libs/Front/rtl/front.tb.v", "libs/Hex1000/rtl/hex1000.v", "scripts/front-trace.xtl"]
EXPECTED = {"clk": 47, "max_din": 17, "max_load": 18, "max_clk": 19, "max_oe": 23}
for group, pins in (("pad_row_n", (15, 16, 27, 28)), ("pad_col_n", (31, 20, 35, 34)),
                    ("op_n", (38, 39, 40, 41)), ("led_n", (9, 10, 11))):
    EXPECTED.update({f"{group}[{i}]": pin for i, pin in enumerate(pins)})


def constraints(text):
    locations = re.findall(r'IO_LOC "([\w\[\]]+)" (\d+);', text)
    if len(locations) != len(EXPECTED) or {k: int(v) for k, v in locations} != EXPECTED:
        raise ValueError("pin map differs from the reviewed physical calculator")
    if 'CLOCK_LOC "machine_clk" BUFG;' not in text:
        raise ValueError("missing global machine clock")
    for i in range(4):
        if f'IO_PORT "pad_row_n[{i}]" IO_TYPE=LVCMOS33 DRIVE=4 TO=SIG;' not in text:
            raise ValueError("row output-enable polarity must be explicit TO=SIG")
    allowed = r'(IO_LOC ".+" \d+;|IO_PORT ".+" IO_TYPE=LVCMOS33(?: PULL_MODE=UP| DRIVE=4(?: TO=SIG)?)?;|CLOCK_LOC "machine_clk" BUFG;)'
    if any(line.strip() and not re.fullmatch(allowed, line) for line in text.splitlines()):
        raise ValueError("unexpected constraints statement")


def timing(report):
    clocks = report.get("fmax", {})
    if "machine_clk" not in clocks or len(clocks) < 2:
        raise ValueError("missing input or machine clock timing")
    expected = {"input": 27.0, "machine": 27.0 / 28.0}
    seen = set()
    for name, data in clocks.items():
        kind = "machine" if name == "machine_clk" else "input"
        achieved, required = data["achieved"], data["constraint"]
        if not all(math.isfinite(v) and v > 0 for v in (achieved, required)):
            raise ValueError("invalid timing measurement")
        if abs(required - expected[kind]) > 0.001 or achieved < required:
            raise ValueError(f"timing failed or wrong clock constraint: {name}")
        seen.add(kind)
    if seen != {"input", "machine"}:
        raise ValueError("incomplete timing report")
    return clocks


def routed_checks(module):
    rows = [(name, cell) for name, cell in module["cells"].items()
            if cell["type"] == "TBUF" and name.startswith("pad_row_n")]
    if len(rows) != 4 or any(int(str(cell.get("attributes", {}).get("&TO=SIG", "0")), 2) != 1 for _, cell in rows):
        raise ValueError("routed rows lost explicit TO=SIG")
    if not any(cell["type"] == "BUFG" and "machine_clk" in name for name, cell in module["cells"].items()):
        raise ValueError("missing global machine clock buffer")
    for name, cell in module["cells"].items():
        if "NEXTPNR_BEL" not in cell.get("attributes", {}):
            raise ValueError(f"unplaced cell: {name}")


def run(command, directory, label, cwd=None, env=None):
    with (directory / "commands.jsonl").open("a") as log:
        log.write(json.dumps({"label": label, "argv": list(map(str, command)),
                              "cwd": str(cwd or directory), "xetal_path": (env or {}).get("XETAL_PATH")}) + "\n")
    result = subprocess.run(list(map(str, command)), cwd=cwd or directory, env=env,
                            capture_output=True, text=True, timeout=300)
    (directory / f"{label}.log").write_text(result.stdout + result.stderr)
    if result.returncode:
        raise RuntimeError(f"{label} failed; see {directory / (label + '.log')}")
    return result.stdout


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify(directory):
    manifest = json.loads((directory / "evidence.json").read_text())
    if manifest.get("status") != "passed" or manifest.get("hardware_actions") != []:
        raise ValueError("evidence is incomplete or is not build/simulation-only")
    if manifest["project_commit"] != (ROOT / "HEX1000_COMMIT").read_text().strip():
        raise ValueError("evidence source pin changed")
    if manifest["libraries_commit"] != (CLONE / "XETAL_LIBRARIES_COMMIT").read_text().strip():
        raise ValueError("evidence library pin changed")
    if manifest["evaluator_commit"] != (ROOT / "XETAL_COMMIT").read_text().strip():
        raise ValueError("evidence evaluator pin changed")
    for name, digest in manifest["inputs_sha256"].items():
        if sha(CLONE / name) != digest:
            raise ValueError(f"source hash changed: {name}")
        if name in SOURCES and sha(directory / "inputs" / name) != digest:
            raise ValueError(f"build input copy changed: {name}")
    if sha(ROOT / "work/hex1000-libraries/libs/Strings/src/Strings.xtl") != manifest["strings_sha256"]:
        raise ValueError("Strings source hash changed")
    for name, digest in manifest["outputs_sha256"].items():
        if sha(directory / name) != digest:
            raise ValueError(f"artifact hash changed: {name}")
    return manifest


def simulate(directory):
    paths = [CLONE / f"libs/{name}/src" for name in ("Front", "Panel", "Hex1000", "Radix")]
    paths.append(ROOT / "work/hex1000-libraries/libs/Strings/src")
    env = dict(os.environ, XETAL_PATH=os.pathsep.join(map(str, paths)))
    trace = run([ROOT / "bin/xetal", "run", CLONE / "scripts/front-trace.xtl"], directory, "front-trace", env=env)
    rows = [line.split() for line in trace.splitlines() if line.strip()]
    if len(rows) != 1100 or any(len(row) != 22 for row in rows):
        raise ValueError("incomplete X_eTaL front-panel trace")
    # The upstream testbench opens this relative path. Give it an output-
    # owned working tree rather than changing the clone or testbench.
    trace_file = directory / "work/board/calculator/front-trace.txt"
    trace_file.parent.mkdir(parents=True, exist_ok=True)
    trace_file.write_text(trace)
    inputs = directory / "inputs"
    run(["iverilog", "-g2005-sv", "-s", "tb", "-o", "front.vvp",
         inputs / "libs/Front/rtl/front.v", inputs / "libs/Front/rtl/front.tb.v"], directory, "front-compile")
    result = run(["vvp", "front.vvp"], directory, "front-model")
    if "PASS: 1100 panel clocks match X_eTaL" not in result:
        raise ValueError("front-panel model comparison did not pass")
    logs = {"front_model_clocks": 1100}
    for clocks in (4, 256):
        run(["iverilog", "-g2005-sv", "-s", "tb", f"-Ptb.TEST_ROWS={clocks}", "-o", f"calculator-{clocks}.vvp",
             inputs / (BOARD + "calculator.v"), inputs / "libs/Front/rtl/front.v",
             inputs / "libs/Hex1000/rtl/hex1000.v", inputs / (BOARD + "calculator.tb.v")], directory, f"compile-{clocks}")
        result = run(["vvp", f"calculator-{clocks}.vvp"], directory, f"simulation-{clocks}")
        if "PASS physical panel, debounce, held-key handshake, arithmetic, == clear and MAX7219 framing" not in result:
            raise ValueError("complete calculator simulation did not pass")
        logs[str(clocks)] = [line for line in result.splitlines() if line.startswith("PASS")]
    print("PASS: 1100 Front model clocks and complete calculator tests at ROW_CLOCKS=4 and 256", flush=True)
    return logs


def build(directory):
    design = " ".join("inputs/" + name for name in (BOARD + "calculator.v", "libs/Front/rtl/front.v", "libs/Hex1000/rtl/hex1000.v"))
    run(["yosys", "-p", f"read_verilog {design}; synth_gowin -family gw1n -top hex1000_calculator -json synth.json"], directory, "synth")
    synth = json.loads((directory / "synth.json").read_text())["modules"]["hex1000_calculator"]
    ports = {}
    for name, data in synth["ports"].items():
        for i in range(len(data["bits"])):
            ports[name if len(data["bits"]) == 1 else f"{name}[{i}]"] = data["direction"]
    if set(ports) != set(EXPECTED):
        raise ValueError("synthesis top ports differ from reviewed pin map")
    for name, direction in ports.items():
        wanted = "input" if name == "clk" or name.startswith(("pad_col_n", "op_n")) else "output"
        if direction != wanted:
            raise ValueError(f"wrong port direction: {name}")
    run(["nextpnr-himbaechel", "--device", "GW1NZ-LV1QN48C6/I5", "--vopt", "family=GW1NZ-1",
         "--vopt", "sspi_as_gpio", "--vopt", f"cst={directory / 'inputs' / (BOARD + 'calculator.cst')}",
         "--json", "synth.json", "--write", "pnr.json", "--freq", "27", "--sdc",
         directory / "inputs" / (BOARD + "calculator.sdc"), "--report", "timing.json"], directory, "pnr")
    placed = json.loads((directory / "pnr.json").read_text())["modules"]["top"]
    routed_checks(placed)
    report = json.loads((directory / "timing.json").read_text())
    clocks = timing(report)
    utilization = report.get("utilization", {})
    # TBUF shares the I/O pads; nextpnr reports zero standalone TBUF
    # capacity. Check the independent resources, including IOB, instead.
    for kind in ("LUT4", "DFF", "BSRAM", "ALU", "IOB"):
        data = utilization.get(kind, {})
        if not 0 < data.get("used", 0) <= data.get("available", 0):
            raise ValueError(f"missing or over-capacity resource: {kind}")
    run(["gowin_pack", "-d", "GW1NZ-1", "--sspi_as_gpio", "--mspi_as_gpio", "--cpu_as_gpio",
         "-o", "calculator.fs", "pnr.json"], directory, "pack")
    if not (directory / "calculator.fs").stat().st_size:
        raise ValueError("empty bitstream")
    cells = [{"name": name, "type": cell["type"], "bel": cell["attributes"]["NEXTPNR_BEL"]}
             for name, cell in sorted(placed["cells"].items())]
    routing = {name: net["attributes"]["ROUTING"] for name, net in placed["netnames"].items()
               if net.get("attributes", {}).get("ROUTING", "").strip()}
    if not routing:
        raise ValueError("missing routing evidence")
    (directory / "cells.json").write_text(json.dumps(cells, indent=2) + "\n")
    (directory / "routes.json").write_text(json.dumps(routing, indent=2) + "\n")
    return {"synthesis_counts": dict(Counter(c["type"] for c in synth["cells"].values())),
            "placed_counts": dict(Counter(c["type"] for c in placed["cells"].values())), "fmax_mhz": clocks,
            "utilization": utilization, "routed_nets": len(routing),
            "pnr_warnings": [line for line in (directory / "pnr.log").read_text().splitlines() if line.startswith("Warning:")],
            "bitstream_bytes": (directory / "calculator.fs").stat().st_size}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--simulate-only", action="store_true")
    mode.add_argument("--verify-only", action="store_true", help="verify existing pins and input/artifact hashes")
    args = parser.parse_args()
    directory = args.directory.resolve()
    if directory == CLONE or CLONE in directory.parents or directory == ROOT / "work/hex1000-libraries" or ROOT / "work/hex1000-libraries" in directory.parents:
        parser.error("outputs must be outside the pinned clones")
    if Path(os.environ.get("VIRTUAL_ENV", "")).resolve() != ROOT / ".venv":
        parser.error("use the repository uv environment (just ext blender hex-keys-leds-evidence)")
    if args.verify_only:
        subprocess.run(["bash", ROOT / "scripts/hex1000.sh", "--check"], check=True, stdout=subprocess.DEVNULL)
        verify(directory)
        print(f"Verified pinned inputs and artifact hashes: {directory / 'evidence.json'}")
        return
    directory.mkdir(parents=True, exist_ok=True)
    # Invalidate the previous success record before any new work starts.
    manifest = {"status": "in-progress", "hardware_actions": [], "project_commit": (ROOT / "HEX1000_COMMIT").read_text().strip(),
                "scope": "simulation-only" if args.simulate_only else "simulation-and-build"}
    record = directory / "evidence.json"
    record.write_text(json.dumps(manifest, indent=2) + "\n")
    subprocess.run(["bash", ROOT / "scripts/hex1000.sh"], check=True, stdout=subprocess.DEVNULL)
    (directory / "commands.jsonl").write_text("")
    for name in SOURCES:
        target = directory / "inputs" / name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(CLONE / name, target)
    constraints((CLONE / (BOARD + "calculator.cst")).read_text())
    manifest.update({"variant": "full calculator with TXS0108E OE on FPGA23", "device": "GW1NZ-LV1QN48C6/I5",
                     "family": "GW1NZ-1", "clock_hz": 27000000, "divider": 28, "row_clocks": 256,
                     "pins": EXPECTED, "evaluator_commit": (ROOT / "XETAL_COMMIT").read_text().strip(),
                     "libraries_commit": (CLONE / "XETAL_LIBRARIES_COMMIT").read_text().strip(),
                     "inputs_sha256": {name: sha(CLONE / name) for name in SOURCES}})
    for name in ("Front", "Panel", "Hex1000", "Radix"):
        for path in sorted((CLONE / f"libs/{name}/src").glob("*.xtl")):
            manifest["inputs_sha256"][str(path.relative_to(CLONE))] = sha(path)
    for path in sorted((CLONE / "libs/Hex1000/rom").glob("*")):
        if path.is_file():
            manifest["inputs_sha256"][str(path.relative_to(CLONE))] = sha(path)
    strings = ROOT / "work/hex1000-libraries/libs/Strings/src/Strings.xtl"
    manifest["strings_sha256"] = sha(strings)
    historical = CLONE / (BOARD + "evidence/calculator.json")
    manifest["historical_evidence"] = {"path": str(historical.relative_to(CLONE)), "sha256": sha(historical),
                                        "date": json.loads(historical.read_text())["date"], "fresh_measurement": False}
    tools = ("iverilog", "vvp") if args.simulate_only else ("iverilog", "vvp", "yosys", "nextpnr-himbaechel", "gowin_pack")
    for tool in tools:
        if not shutil.which(tool):
            parser.error(f"{tool} missing; put the OSS CAD Suite on PATH")
    manifest["tools"] = {}
    for tool in tools:
        flag = "-V" if tool in ("iverilog", "vvp", "yosys") else "--version" if tool == "nextpnr-himbaechel" else "--help"
        run([tool, flag], directory, tool + "-version")
        executable = Path(shutil.which(tool)).resolve()
        manifest["tools"][tool] = {"path": str(executable), "launcher_sha256": sha(executable), "version_log": tool + "-version.log"}
        # OSS CAD Suite launchers are wrappers. Identify the actual programs
        # and the Gowin device databases, as well as human-readable versions.
        suite = executable.parent.parent
        assets = [suite / "libexec" / tool]
        if tool == "nextpnr-himbaechel":
            assets += [suite / "share/nextpnr/himbaechel/gowin/chipdb-GW1NZ-1.bin"]
        if tool == "gowin_pack":
            for packages in suite.glob("lib/python*/site-packages/apycula"):
                assets += [packages / "gowin_pack.py", packages / "GW1NZ-1.msgpack.xz"]
        manifest["tools"][tool]["implementation_sha256"] = {str(p): sha(p) for p in assets if p.is_file()}
    manifest["simulation"] = simulate(directory)
    if not args.simulate_only:
        manifest["build"] = build(directory)
    subprocess.run(["bash", ROOT / "scripts/hex1000.sh", "--check"], check=True, stdout=subprocess.DEVNULL)
    outputs = ("commands.jsonl", "front-trace.log", "front-model.log", "simulation-4.log", "simulation-256.log")
    if not args.simulate_only:
        outputs += ("synth.json", "pnr.json", "timing.json", "calculator.fs", "cells.json", "routes.json")
    manifest["outputs_sha256"] = {name: sha(directory / name) for name in outputs}
    manifest["limitations"] = ["Fresh simulations/build only; no device detection or programming.",
                               "Upstream testbench clock is 100 MHz; production divider, row and startup counts are retained.",
                               "Physical arithmetic/CRC observations in the source repository are historical, not new measurements.",
                               "The dedicated-route warning is the divider data-feedback inverter; machine clock sinks use BUFG routing.",
                               "Overflow wraps; OuErFLO, ROM-side RAM clearing, full physical key coverage and a separate cold boot remain upstream follow-ups."]
    manifest["status"] = "passed"
    record.write_text(json.dumps(manifest, indent=2) + "\n")
    verify(directory)
    print(f"Evidence saved: {record}")
    if "build" in manifest:
        measured = manifest["build"]
        print("resources:", {kind: measured["utilization"][kind]["used"] for kind in ("LUT4", "DFF", "ALU", "BSRAM", "IOB")})
        print("timing (MHz):", measured["fmax_mhz"])
        print("bitstream bytes:", measured["bitstream_bytes"])


if __name__ == "__main__":
    main()
