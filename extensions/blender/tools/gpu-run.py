#!/usr/bin/env python3
"""Check device observations and export arrays for an X_eTaL scene."""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

EXT = Path(__file__).resolve().parents[1]
ROOT = EXT.parents[1]
OUT = EXT / "work/gpurun"
DATA = EXT / "demos/gpu-build-data"
spec = importlib.util.spec_from_file_location("gpu_profile", EXT / "tools/gpu-profile.py")
observer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(observer)


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def command(args):
    p = subprocess.run(list(map(str, args)), cwd=ROOT, text=True, capture_output=True, check=True)
    return p.stdout, p.stderr


def collect():
    OUT.mkdir(parents=True, exist_ok=True)
    command([ROOT / ".venv/bin/python3", EXT / "tools/gpu-build.py", "verify"])
    command([ROOT / ".venv/bin/python3", EXT / "tools/gpu-profile.py", "--output", OUT / "rtx.json"])
    binary = ROOT / "work/gpu-video-target/release/xetal-gpu"
    for kind, args in (("device", ["--device", "opencl:0", "--schedule", ROOT / "work/gpu/schedules/nvidia-rtx5060-ti.toml", "--time"]),
                       ("cpu", ["--device", "cpu"])):
        stdout, stderr = command([binary, "run", DATA / "pipeline.xir", *args])
        if stdout.split() != ["32", "16.625"]:
            raise ValueError("pinned runtime disagrees with evaluator")
        (OUT / (kind + ".stdout")).write_text(stdout)
        (OUT / (kind + ".stderr")).write_text(stderr)
    (OUT / "tool-sha.txt").write_text(sha(binary) + "\n")


def prepare():
    trace = json.loads((OUT / "rtx.json").read_text())
    if trace["observer_sha256"] != sha(EXT / "tools/gpu-profile.py"):
        raise ValueError("observer changed; collect a fresh trace")
    for name, digest in trace["artifacts_sha256"].items():
        if sha(DATA / name) != digest:
            raise ValueError("compiler artifact changed")
    if "RTX 5060 Ti" not in trace["device"]["name"] or trace["device"]["host_unified_memory"]:
        raise ValueError("review changed device architecture")
    for r in trace["records"]:
        observer.validate(r)
    for k in ("cpu", "device"):
        if (OUT / (k + ".stdout")).read_text().split() != ["32", "16.625"]:
            raise ValueError("missing independent runtime result")
    if "(device)" not in (OUT / "device.stderr").read_text():
        raise ValueError("missing pinned runtime profiling")
    r = trace["records"][2]
    events = r["events"]
    kernels = [e for e in events if e["kind"] == "kernel"]
    origin = kernels[0]["start_ns"]
    span = (kernels[-1]["end_ns"] - origin) / 1000
    timings = [[(e["start_ns"] - origin) / 1000, (e["end_ns"] - origin) / 1000] for e in kernels]
    h = r["host_ms"]
    stats = [trace["device"]["compute_units"], trace["device"]["host_unified_memory"],
             sum(e["bytes"] for e in events if e["kind"] == "upload"),
             sum(e["bytes"] for e in events if e["kind"] == "download"), trace["setup_ms"], h["total"],
             h["bind"], h["upload"], h["launch"], h["download"],
             sum(b-a for a,b in timings), span, *r["outputs"]]
    (OUT / "stats.txt").write_text(" ".join(map(str, stats)) + "\n")
    (OUT / "timings.txt").write_text("\n".join(" ".join(map(str, row)) for row in timings) + "\n")
    (OUT / "timing-labels.txt").write_text("|".join(f'{e["name"]}: {(e["end_ns"]-e["start_ns"])/1000:.3f} us' for e in kernels))
    # Emulate the emitted local-memory tree, including 248 padded zeros.
    state = [max(x + x*w, 0) for x,w in zip(range(1,9), [1,-1]*4)] + [0]*248
    states = [state.copy()]
    for stride in (128,64,32,16,8,4,2,1):
        for i in range(stride):
            state[i] += state[i+stride]
        states.append(state.copy())
    if state[0] != r["outputs"][0]:
        raise ValueError("illustrated reduction disagrees with device")
    (OUT / "reduction.txt").write_text("\n".join(" ".join(map(str,row)) for row in states) + "\n")
    (OUT / "reduction-labels.txt").write_text("|".join(" + ".join(map(str,row[:min(8,active)])) for row,active in zip(states, (256,128,64,32,16,8,4,2,1))) + "|")
    (OUT / "host-labels.txt").write_text("|".join(f"{k}: {h[k]*1000:.3f} us" for k in ("bind","upload","launch","download")))
    names = ["rtx.json", "device.stdout", "device.stderr", "cpu.stdout", "cpu.stderr", "tool-sha.txt", "stats.txt", "timings.txt", "timing-labels.txt", "reduction.txt", "reduction-labels.txt", "host-labels.txt"]
    manifest = {"schema": 1, "gpu_commit": (ROOT / "GPU_COMMIT").read_text().strip(),
                "selected_trial": 2, "all_trials_retained": True,
                "limits": "RTX observation only; M1 unified-memory panel is architectural, with no M1 timing. Work-group placement and reduction animation are algorithm diagrams, not hardware traces. Host launch time contains device execution; never add them. Separate host/device clocks. No benchmark or speedup claim.",
                "files_sha256": {n: sha(OUT/n) for n in names},
                "inputs_sha256": {str(p.relative_to(ROOT)): sha(p) for p in [EXT / "tools/gpu-run.py", EXT / "tools/gpu-profile.py", ROOT / "GPU_COMMIT", DATA / "manifest.json"]}}
    (OUT / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    return manifest


def verify(directory=OUT):
    m = json.loads((directory / "manifest.json").read_text())
    for n, digest in m["files_sha256"].items():
        if sha(directory/n) != digest:
            raise ValueError("recorded evidence changed: " + n)
    for n, digest in m["inputs_sha256"].items():
        if sha(ROOT/n) != digest:
            raise ValueError("evidence input changed: " + n)
    trace = json.loads((directory/"rtx.json").read_text())
    for record in trace["records"]:
        observer.validate(record)
    print("PASS GPU execution evidence and event traces")


if __name__ == "__main__":
    action = sys.argv[1] if len(sys.argv) > 1 else "prepare"
    if action == "collect":
        collect()
        prepare()
    elif action == "prepare":
        prepare()
    elif action == "verify":
        verify()
    elif action == "publish":
        import shutil
        verify()
        dest = EXT / "demos/gpu-run-data"
        dest.mkdir(parents=True, exist_ok=True)
        m = json.loads((OUT/"manifest.json").read_text())
        for name in [*m["files_sha256"], "manifest.json"]:
            shutil.copy2(OUT/name, dest/name)
    else:
        raise ValueError("unknown action")
