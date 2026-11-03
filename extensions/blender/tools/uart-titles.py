#!/usr/bin/env python3
"""Replace static opening cards; retain checked footage and original inputs.

The X_eTaL driver renders cards and rewrites the cue sheets. Published
recordings are always available; refresh full masters too when present.
An absent master is not reconstructed from a lower-resolution recording.
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

EXT = Path(__file__).resolve().parents[1]
ROOT = EXT.parents[1]
OUT = EXT / "work/uart-titles"


def run(args, **kwargs):
    return subprocess.run(list(map(str, args)), check=True, **kwargs)


def probe(path):
    result = run(["ffprobe", "-v", "error", "-count_frames", "-select_streams", "v:0",
                  "-show_entries", "stream=width,height,r_frame_rate,nb_read_frames",
                  "-of", "json", path], capture_output=True, text=True)
    return json.loads(result.stdout)["streams"][0]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    os.chdir(EXT)
    OUT.mkdir(parents=True, exist_ok=True)
    if not (EXT / "work/hexbuild/stats.txt").is_file():
        raise SystemExit("run just ext blender hex-build-data first (the checked build counts)")
    libs = run([ROOT / "scripts/libraries.sh"], capture_output=True, text=True).stdout.strip()
    env = dict(os.environ, XETAL_PATH=f"{EXT}/demos:{libs}")
    run([ROOT / "target/release/xetal-x", "--ext", ROOT / "extensions", "run", "tools/uart-titles.xtl"], env=env)
    evidence = []
    for kind, cut, total in (("run", 97, 1261), ("build", 91, 721)):
        sources = [("site", EXT / f"videos/hex-{kind}.webm", False)]
        work = "hexrun" if kind == "run" else "hexbuild"
        sources += [("wide", EXT / f"work/{work}/out/hex-{kind}.mp4", True),
                    ("short", EXT / f"work/{work}/out-short/hex-{kind}-short.mp4", True)]
        for shape, source, master in sources:
            if not source.is_file():
                print(f"no {kind} {shape} master here; full demo renders it with the new title")
                continue
            backup = OUT / "originals" / f"{kind}-{shape}{source.suffix}"
            backup.parent.mkdir(parents=True, exist_ok=True)
            if not backup.exists():
                shutil.copy2(source, backup)
            before = probe(backup)
            if before["r_frame_rate"] != "30/1" or int(before["nb_read_frames"]) != total:
                raise ValueError(f"unexpected source timing: {backup}: {before}")
            output = OUT / (f"hex-{kind}.mp4" if shape == "site" else f"{kind}-{shape}.mp4")
            card = OUT / "cards" / f"{kind}-{shape}-1.png"
            filters = (f"[0:v]trim=end_frame={cut},setpts=PTS-STARTPTS,setsar=1,format=yuv420p[a];"
                       f"[1:v]trim=start_frame={cut},setpts=PTS-STARTPTS,setsar=1,format=yuv420p[b];"
                       "[a][b]concat=n=2:v=1:a=0[v]")
            run(["ffmpeg", "-loglevel", "error", "-y", "-loop", "1", "-framerate", "30", "-i", card,
                 "-i", backup, "-filter_complex", filters, "-map", "[v]", "-frames:v", total,
                 "-an", "-c:v", "libx264", "-preset", "fast", "-crf", "16", "-threads", "4", output])
            after = probe(output)
            if before != after:
                raise ValueError(f"retitle changed dimensions/timing: {before} -> {after}")
            evidence.append({"source": str(source.relative_to(EXT)), "original_sha256": sha(backup),
                             "output_sha256": sha(output), "replaced_frames": cut, "stream": after})
            if master:
                shutil.copy2(output, source)
            print(f"retitled {kind} {shape}: {cut}/{total} frames, original retained at {backup}")
    (OUT / "evidence.json").write_text(json.dumps(evidence, indent=2) + "\n")


if __name__ == "__main__":
    main()
