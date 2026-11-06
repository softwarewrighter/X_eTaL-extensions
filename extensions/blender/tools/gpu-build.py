#!/usr/bin/env python3
"""Compiler evidence for the GPU build movie; no OpenCL device is run."""
import hashlib
import json
import math
import os
from pathlib import Path
import re
import shutil
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[3]
EXT = ROOT / "extensions/blender"
GPU = ROOT / "work/gpu"
OUT = EXT / "work/gpubuild"
BIN = ROOT / "work/gpu-video-target/release"
ARTIFACTS = set()


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def checked(text):
    nodes = []
    edges = []
    ids = {}
    for name, scalar, shape, expression in re.findall(r"^%(\w+) : (i64|f64) \[([^]]*)\] = (.+)$", text, re.M):
        parts = expression.split()
        kind = 0 if parts[0] == "const" else 5 if parts[:2] == ["reduce", "add"] else {
            "mul": 1, "add": 2, "max": 3, "div": 4}.get(parts[1], -1)
        if kind < 0 or parts[0] not in ("const", "map", "reduce"):
            raise ValueError("unsupported checked operation")
        index = len(nodes) + 1
        for dep in re.findall(r"%(\w+)", expression):
            if dep not in ids:
                raise ValueError("missing or forward XIR dependency")
            edges.append([ids[dep], index])
        if name in ids:
            raise ValueError("duplicate XIR value")
        ids[name] = index
        nodes.append({"name": name, "id": index, "kind": kind, "type": int(scalar == "f64"),
                      "items": math.prod(map(int, shape.split()))})
    if not nodes or len(nodes) != text.count(" = "):
        raise ValueError("incomplete checked XIR")
    return nodes, edges


def plan(text, nodes):
    lines = text.splitlines()
    sections = {}
    key = None
    for line in lines:
        if line.endswith(":") and not line.startswith(" "):
            key = line[:-1]
            sections[key] = []
        elif key and line.startswith("  "):
            sections[key].append(line)
    buffers = re.findall(r"^  (\w+):", "\n".join(sections["buffers"]), re.M)
    by_name = {n["name"]: n for n in nodes}
    kernels = []
    seen = set()
    for line in sections["kernels"]:
        match = re.match(r"  (\w+): .*; computes (.+?)(?:; writes (.+))?$", line)
        if not match:
            raise ValueError("unknown kernel description")
        name, computes, writes = match.groups()
        names = re.findall(r"%(\w+)", computes)
        if any(n not in by_name or n in seen for n in names):
            raise ValueError("unknown or repeated kernel value")
        seen.update(names)
        maps = [by_name[n] for n in names if by_name[n]["kind"] != 5]
        if maps:
            if not writes or re.findall(r"%(\w+)", writes) != [names[-1]]:
                raise ValueError("fused kernel must write only its final value")
            if any("b_" + n in buffers for n in names[:-1]):
                raise ValueError("fused intermediate is materialized")
        kernels.append({"name": name, "values": names, "map": bool(maps)})
    if seen != {n["name"] for n in nodes if n["kind"] != 0}:
        raise ValueError("kernel coverage differs from checked XIR")
    launches = []
    for line in sections["launches"]:
        match = re.match(r"  (\d+): (\w+)\((.*)\) global (\d+) local (\d+): (.*)", line)
        if not match:
            raise ValueError("unknown launch description")
        number, kernel, args, glob, local, note = match.groups()
        if kernel not in {k["name"] for k in kernels} or int(glob) <= 0 or int(local) <= 0 or int(glob) % int(local):
            raise ValueError("invalid launch")
        launches.append({"kernel": kernel, "global": int(glob), "local": int(local), "note": note,
                         "local_bytes": sum(map(int, re.findall(r"local (\d+) bytes", args)))})
    if {l["kernel"] for l in launches} != {k["name"] for k in kernels}:
        raise ValueError("kernel launch coverage differs")
    return {"buffers": buffers, "kernels": kernels, "launches": launches}


def run(command, name, env):
    result = subprocess.run(list(map(str, command)), cwd=ROOT, env=env, capture_output=True, text=True, timeout=120)
    (OUT / name).write_text(result.stdout)
    (OUT / (name + ".stderr")).write_text(result.stderr)
    ARTIFACTS.update((name, name + ".stderr"))
    with (OUT / "commands.jsonl").open("a") as log:
        log.write(json.dumps({"argv": list(map(str, command)), "stdout": name, "exit": result.returncode}) + "\n")
    if result.returncode:
        raise ValueError(f"{name} failed: {result.stderr}")
    return result.stdout


def prepare():
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "evidence.json").write_text('{"status":"in-progress"}\n')
    (OUT / "commands.jsonl").write_text("")
    ARTIFACTS.clear()
    ARTIFACTS.add("commands.jsonl")
    for clone, pin in ((GPU, ROOT / "GPU_COMMIT"), (ROOT / "work/xetal", ROOT / "XETAL_COMMIT"),
                       (EXT / "work/gpu-tool/work/xetal", GPU / "XETAL_COMMIT")):
        head = subprocess.check_output(["git", "-C", str(clone), "rev-parse", "HEAD"], text=True).strip()
        dirty = subprocess.check_output(["git", "-C", str(clone), "status", "--porcelain"], text=True)
        if head != pin.read_text().strip() or dirty:
            raise ValueError("compiler source pin differs or clone is dirty")
    env = dict(os.environ, XETAL_PATH=str(GPU / "libs/Accel/src"), XETAL_BIN=str(BIN / "xetal"))
    demo = GPU / "libs/Accel/demos/pipeline.xtl"
    core = run([BIN / "gpu-video-core", demo], "resolved.core", env)
    definitions = dict(re.findall(r"^\(def (\S+) (.*)\)$", core, re.M))
    root = next(n for n in definitions if n.endswith(":p_ipeline"))
    used = []

    def visit(name):
        if name in used:
            return
        used.append(name)
        for ref in re.findall(r"\b\w+:\w+", definitions[name]):
            if ref in definitions:
                visit(ref)

    visit(root)
    if {n.split(":")[1] for n in used} != {"p_ipeline", "s_um", "r_elu"}:
        raise ValueError("review the changed pipeline Core closures")
    if not any("#*" in definitions[n] for n in used) or not any("#r_/" in definitions[n] for n in used):
        raise ValueError("resolved Core lacks the pipeline operations")
    (OUT / "core-labels.txt").write_text("|".join(n.split(":")[1].replace("_", "") for n in used))
    core_edges = [[used.index(n) + 1, used.index(ref) + 1] for n in used
                  for ref in re.findall(r"\b\w+:\w+", definitions[n]) if ref in used]
    (OUT / "core-edges.txt").write_text("".join(f"{a} {b}\n" for a, b in core_edges))
    run([BIN / "xetal", "core", demo], "surface.core", env)
    run([BIN / "xetal-gpu", "lower", demo], "pipeline.xir", env)
    if (OUT / "pipeline.xir.stderr").read_text().strip():
        raise ValueError("pipeline unexpectedly fell back to host evaluation")
    xir = (OUT / "pipeline.xir").read_text()
    inputs = [re.search(r"^%" + n + r" = const i64 \[8\] (.+)$", xir, re.M)[1] for n in ("x", "w")]
    (OUT / "inputs.txt").write_text("\n".join(inputs) + "\n")
    nodes, edges = checked(run([BIN / "xetal-gpu", "check", demo], "checked.txt", env))
    oracles = []
    for label, tool in (("animation", ROOT / "bin/xetal"), ("compiler", BIN / "xetal"), ("xir", BIN / "xetal-gpu")):
        output = run([tool, "run", demo], label + "-oracle.txt", env)
        oracles.append(list(map(float, output.split())))
    if oracles != [[32.0, 16.625]] * 3:
        raise ValueError("evaluator/XIR pipeline results differ")
    matrix_oracles = []
    matrix_demo = GPU / "libs/Accel/demos/matmul.xtl"
    for label, tool in (("animation", ROOT / "bin/xetal"), ("compiler", BIN / "xetal"), ("xir", BIN / "xetal-gpu")):
        output = run([tool, "run", matrix_demo], "matrix-" + label + "-oracle.txt", env)
        matrix_oracles.append(list(map(float, output.split())))
    if not matrix_oracles[0] or any(v != matrix_oracles[0] for v in matrix_oracles[1:]):
        raise ValueError("evaluator/XIR matrix results differ")
    profiles = []
    files = [demo, GPU / "libs/Accel/src/Accel.xtl", GPU / "libs/Accel/demos/matmul.xtl", ROOT / "GPU_COMMIT",
             ROOT / "XETAL_COMMIT", GPU / "XETAL_COMMIT", EXT / "tools/gpu-core/src/main.rs",
             EXT / "tools/gpu-core/Cargo.toml", EXT / "tools/gpu-core/Cargo.lock", ROOT / "scripts/gpu-video-tool.sh",
             EXT / "tools/gpu-build.py"]
    for label, file in (("m1", "apple-m1-max.toml"), ("rtx", "nvidia-rtx5060-ti.toml")):
        schedule = GPU / "schedules" / file
        files.append(schedule)
        config = tomllib.loads(schedule.read_text())
        if config["float"] != "f32" or config["int"] != "i64" or config["work_group"] != 256:
            raise ValueError("review the changed device profile")
        result = {"name": label, "config": config}
        for program in ("pipeline", "matmul"):
            source = GPU / "libs/Accel/demos" / (program + ".xtl")
            text = run([BIN / "xetal-gpu", "explain", source, "--schedule", schedule], f"{label}-{program}.plan", env)
            kernel = run([BIN / "xetal-gpu", "kernel", source, "--schedule", schedule], f"{label}-{program}.cl", env)
            run(["clang", "-x", "cl", "-cl-std=CL1.2", "-fsyntax-only", OUT / f"{label}-{program}.cl"], f"{label}-{program}-syntax.txt", env)
            if program == "pipeline":
                result["plan"] = plan(text, nodes)
                if set(re.findall(r"__kernel void (\w+)", kernel)) != {k["name"] for k in result["plan"]["kernels"]}:
                    raise ValueError("kernel source and plan disagree")
            else:
                launch = re.search(r"1: (\w+)\(.*\) global (\d+) local (\d+): %mm: 3 by 4 times 4 by 2", text)
                if not launch:
                    raise ValueError("matrix schedule example changed")
                result["matrix"] = {"kernel": launch[1], "global": int(launch[2]), "local": int(launch[3]), "outputs": 6}
        profiles.append(result)
    first = profiles[0]["plan"]
    if first != profiles[1]["plan"]:
        raise ValueError("pipeline schedules differ; update the explanation")
    kernel_ids = {v: i + 1 for i, k in enumerate(first["kernels"]) for v in k["values"]}
    by_name = {n["name"]: n for n in nodes}
    map_outputs = {k["values"][-1] for k in first["kernels"] if k["map"]}
    buffer_rows = []
    for name in first["buffers"]:
        node = by_name[name.removeprefix("b_")]
        role = 0 if node["kind"] == 0 else 1 if node["name"] in map_outputs else 2
        buffer_rows.append([role, node["type"], node["items"]])
    (OUT / "buffers.txt").write_text("".join(" ".join(map(str, row)) + "\n" for row in buffer_rows))
    rows = [[n["id"], n["kind"], n["type"], n["items"], kernel_ids.get(n["name"], 0)] for n in nodes]
    for name, data in (("nodes", rows), ("edges", edges)):
        (OUT / (name + ".txt")).write_text("".join(" ".join(map(str, row)) + "\n" for row in data))
    labels = []
    for n in nodes:
        if n["kind"]:
            labels.append({1: "mul", 2: "add", 3: "max", 4: "div", 5: "sum"}[n["kind"]])
        elif n["items"] == 1:
            labels.append(re.search(r"^%" + n["name"] + r" = const \w+ \[\] (.+)$", xir, re.M)[1])
        else:
            labels.append(n["name"])
    (OUT / "labels.txt").write_text("|".join(labels))
    stats = [len(nodes), sum(n["kind"] in (1, 2, 3, 4) for n in nodes), sum(n["kind"] == 5 for n in nodes),
             len(first["kernels"]), len(first["buffers"]), sum(len(k["values"]) - 1 for k in first["kernels"] if k["map"]),
             profiles[0]["config"]["work_group"], profiles[1]["config"]["work_group"],
             profiles[0]["config"]["tile"], profiles[1]["config"]["tile"],
             profiles[0]["matrix"]["global"], profiles[0]["matrix"]["local"],
             profiles[1]["matrix"]["global"], profiles[1]["matrix"]["local"], 6, 32, 16625, len(used),
             first["launches"][0]["global"], first["launches"][1]["global"],
             first["launches"][1]["local_bytes"], first["launches"][3]["local_bytes"]]
    (OUT / "stats.txt").write_text(" ".join(map(str, stats)) + "\n")
    ARTIFACTS.update(("nodes.txt", "edges.txt", "labels.txt", "stats.txt", "inputs.txt", "core-labels.txt", "core-edges.txt", "buffers.txt"))
    record = {"status": "checked", "gpu_commit": (ROOT / "GPU_COMMIT").read_text().strip(),
              "animation_evaluator_commit": (ROOT / "XETAL_COMMIT").read_text().strip(),
              "compiler_evaluator_commit": (GPU / "XETAL_COMMIT").read_text().strip(),
              "inputs_sha256": {str(p.relative_to(ROOT)): sha(p) for p in files},
              "core_closures": {n: definitions[n] for n in used}, "nodes": nodes, "edges": edges,
              "profiles": profiles, "stats": stats, "oracle_results": oracles, "matrix_oracles": matrix_oracles,
              "device_execution": False, "device_timings": None,
              "tool_sha256": {name: sha(BIN / name) for name in ("xetal", "xetal-gpu", "gpu-video-core")},
              "outputs_sha256": {name: sha(OUT / name) for name in sorted(ARTIFACTS)}}
    (OUT / "evidence.json").write_text(json.dumps(record, indent=2) + "\n")
    print(f"PASS GPU compiler evidence: {len(nodes)} XIR values, {len(first['kernels'])} kernels, {len(first['buffers'])} buffers; both evaluators and XIR agree")


def verify():
    record = json.loads((OUT / "evidence.json").read_text())
    if record.get("status") != "checked" or record.get("device_execution") is not False:
        raise ValueError("missing build-only compiler evidence")
    for name, digest in record["inputs_sha256"].items():
        if sha(ROOT / name) != digest:
            raise ValueError(f"compiler input changed: {name}")
    for name, digest in record["outputs_sha256"].items():
        if sha(OUT / name) != digest:
            raise ValueError(f"compiler output changed: {name}")
    for name, digest in record["tool_sha256"].items():
        if sha(BIN / name) != digest:
            raise ValueError(f"compiler binary changed: {name}")
    return record


def publish():
    record = verify()
    dest = EXT / "demos/gpu-build-data"
    dest.mkdir(parents=True, exist_ok=True)
    names = ["resolved.core", "surface.core", "pipeline.xir", "checked.txt", "nodes.txt", "edges.txt",
             "labels.txt", "inputs.txt", "stats.txt", "buffers.txt", "core-labels.txt", "core-edges.txt"]
    names += [f"{profile}-{program}.{suffix}" for profile in ("m1", "rtx")
              for program in ("pipeline", "matmul") for suffix in ("cl", "plan")]
    for name in names:
        shutil.copy2(OUT / name, dest / name)
    for label, name in (("m1", "apple-m1-max.toml"), ("rtx", "nvidia-rtx5060-ti.toml")):
        shutil.copy2(GPU / "schedules" / name, dest / (label + "-profile.toml"))
        names.append(label + "-profile.toml")
    public = {key: record[key] for key in ("gpu_commit", "animation_evaluator_commit", "compiler_evaluator_commit",
                                         "core_closures", "oracle_results", "matrix_oracles", "device_execution")}
    public["artifacts_sha256"] = {name: sha(dest / name) for name in names}
    (dest / "manifest.json").write_text(json.dumps(public, indent=2) + "\n")
    print(f"PASS published {len(names)} compiler artifacts from the verified data")


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("prepare", "verify", "publish"), nargs="?", default="prepare")
    action = parser.parse_args().action
    if action == "prepare":
        prepare()
    elif action == "publish":
        publish()
    else:
        verify()
        print("PASS GPU compiler evidence hashes")
