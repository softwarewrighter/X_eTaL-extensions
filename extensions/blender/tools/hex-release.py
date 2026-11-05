#!/usr/bin/env python3
"""Audit all HEX1000 masters, Shorts, recordings and narration artifacts."""
import hashlib
import importlib.util
import json
from datetime import datetime, timezone
from pathlib import Path
import re
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[3]
EXT = ROOT / "extensions/blender"
OUT = EXT / "work/hex-release"
VIDEOS = (
    ("hex-run", "hexrun", "HEX1000-UART", 42, (5, 13, 29, 40)),
    ("hex-build", "hexbuild", "HEX1000-UART", 24, (5, 9, 14, 18, 23)),
    ("hex-keys-leds-run", "keysrun", "HEX1000-keys-leds", 36, (4.5, 7, 14.8, 25.8, 28, 35)),
    ("hex-keys-leds-build", "keysbuild", "HEX1000-keys-leds", 32, (5, 10, 16.8, 19, 23, 31)),
)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(name):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def evidence_checks():
    old = EXT / "work/hexbuild"
    manifest = json.loads((old / "evidence.json").read_text())
    if manifest["fpga_commit"] != (ROOT / "FPGA_COMMIT").read_text().strip():
        raise ValueError("UART build pin changed")
    for name, digest in manifest["sha256"].items():
        if sha(old / name) != digest:
            raise ValueError(f"UART build artifact changed: {name}")
    physical = EXT / "work/keys-leds-evidence"
    load("keys-leds-evidence").verify(physical)
    load("keys-leds-run").checked_record()
    scene = EXT / "work/keysbuild"
    record = json.loads((scene / "scene-evidence.json").read_text())
    if record["source_evidence_sha256"] != sha(physical / "evidence.json"):
        raise ValueError("physical build evidence changed after scene export")
    for name, digest in record["array_sha256"].items():
        if sha(scene / name) != digest:
            raise ValueError(f"physical scene array changed: {name}")


def stream(path, size, frames):
    data = json.loads(subprocess.check_output([
        "ffprobe", "-v", "error", "-count_frames", "-select_streams", "v",
        "-show_entries", "stream=width,height,avg_frame_rate,nb_read_frames", "-of", "json", str(path)]))["streams"]
    if len(data) != 1:
        raise ValueError(f"expected one video stream: {path}")
    info = data[0]
    if (info["width"], info["height"]) != size or info["avg_frame_rate"] != "30/1" or int(info["nb_read_frames"]) != frames:
        raise ValueError(f"unexpected dimensions/rate/frame count: {path}: {info}")
    return {"path": str(path.relative_to(ROOT)), "sha256": sha(path), **info}


def seconds(text):
    h, m, s, ms = map(int, re.split(r"[:,]", text))
    return h * 3600 + m * 60 + s + ms / 1000


def captions(directory, name, family, length):
    cue_file = directory / (name + ".cues.toml")
    srt_file = directory / (name + ".srt")
    cues = tomllib.loads(cue_file.read_text())["cue"]
    subtitles = re.findall(r"(\d+)\n([\d:,]+) --> ([\d:,]+)\n(.*?)(?=\n\n|\Z)", srt_file.read_text(), re.S)
    if len(cues) != len(subtitles) or not cues or family not in cues[0]["caption"]:
        raise ValueError(f"missing cues/subtitles/family: {name}")
    previous = 0
    for i, (cue, sub) in enumerate(zip(cues, subtitles), 1):
        n, start, end, caption = sub
        if cue["n"] != i or int(n) != i or not previous <= cue["start"] < cue["end"] <= length:
            raise ValueError(f"cue order/range: {name} {i}")
        if abs(seconds(start) - cue["start"]) > 0.001 or abs(seconds(end) - cue["end"]) > 0.001:
            raise ValueError(f"subtitle timing differs: {name} {i}")
        if " ".join(caption.split()) != " ".join(cue["caption"].split()):
            raise ValueError(f"subtitle text differs: {name} {i}")
        previous = cue["end"]
    return {"cues": len(cues), "cue_sha256": sha(cue_file), "srt_sha256": sha(srt_file)}


def audit():
    OUT.mkdir(parents=True, exist_ok=True)
    record = {"status": "in-progress", "checked_at_utc": datetime.now(timezone.utc).isoformat(),
              "hardware_actions": [], "videos": []}
    (OUT / "audit.json").write_text(json.dumps(record) + "\n")
    evidence_checks()
    page = (ROOT / "pages/index.html").read_text()
    for name, folder, family, length, stages in VIDEOS:
        directory = EXT / "work" / folder
        frames = length * 30 + 1
        entry = {"demo": name, "family": family, "frames": frames,
                 **captions(directory, name, family, length)}
        entry["landscape"] = stream(directory / "out" / (name + ".mp4"), (1920, 1080), frames)
        entry["short"] = stream(directory / "out-short" / (name + "-short.mp4"), (1080, 1920), frames)
        recording = EXT / "videos" / (name + ".webm")
        entry["recording"] = stream(recording, (1280, 720), frames)
        if sha(recording) != sha(ROOT / "pages/blender" / (name + ".webm")):
            raise ValueError(f"site recording copy differs: {name}")
        anchor = f'id="blender-{name}"'
        section = page.split(anchor, 1)[1].split("</section>", 1)[0] if anchor in page else ""
        if f'{family}: ' not in section or f"just ext blender {name}" not in section:
            raise ValueError(f"missing catalog family/anchor/preparation command: {name}")
        entry["stages"] = []
        for shape, path in (("wide", directory / "out" / (name + ".mp4")),
                            ("short", directory / "out-short" / (name + "-short.mp4"))):
            for i, at in enumerate((1, *stages), 1):
                picture = OUT / f"{name}-{shape}-{i}.png"
                subprocess.run(["ffmpeg", "-loglevel", "error", "-y", "-ss", str(at), "-i", str(path),
                                "-frames:v", "1", "-vf", "scale=640:-1" if shape == "wide" else "scale=360:-1", str(picture)], check=True)
                entry["stages"].append({"path": picture.name, "seconds": at, "sha256": sha(picture)})
        record["videos"].append(entry)
        print(f"PASS {name}: {frames} frames; landscape, Short, recording, cues/SRT and stage samples", flush=True)
    record["pins"] = {name: (ROOT / name).read_text().strip() for name in ("XETAL_COMMIT", "FPGA_COMMIT", "HEX1000_COMMIT", "LIBRARIES_COMMIT")}
    record["media_tools"] = {name: subprocess.check_output([name, option], text=True).splitlines()[0]
                             for name, option in (("ffmpeg", "-version"), ("ffprobe", "-version"), ("blender", "--version"))}
    record["evidence_sha256"] = {name: sha(EXT / "work" / name) for name in (
        "hexbuild/evidence.json", "hexrun/hex-run.vcd", "hexrun/browser.txt", "hexrun/browser.png",
        "keys-leds-evidence/evidence.json", "keysrun/run-evidence.json", "keysbuild/scene-evidence.json")}
    record["status"] = "passed"
    (OUT / "audit.json").write_text(json.dumps(record, indent=2) + "\n")
    print("PASS all four HEX1000 video exports; no hardware access")


if __name__ == "__main__":
    audit()
