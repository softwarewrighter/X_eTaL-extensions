#!/usr/bin/env python3
"""The pictures on the city's screens, made small for the web: reads
extensions/three/media.toml and writes extensions/three/web/static/media
(slides as STEM-N.jpg and STEM.webm, videos as NAME.webm), with
SOURCES.md saying where each came from. Files are written over, never
removed. Needs ffmpeg and ImageMagick (magick).
    scripts/city-media.py
"""
import pathlib
import shutil
import subprocess
import sys
import tempfile
import tomllib

ROOT = pathlib.Path(__file__).resolve().parent.parent
EXT = ROOT / "extensions"
BLOG = ROOT.parent.parent / "software-wrighter-lab/blog/assets/images/posts"
OUT = EXT / "three/web/static/media"
REEL = (480, 270)


def find(source):
    kind, _, rest = source.partition(":")
    path = {"blog": BLOG, "ext": EXT}[kind] / rest
    if not path.exists():
        sys.exit(f"city-media: {source}: no file {path}")
    return path


def run(*args):
    subprocess.run([str(a) for a in args], check=True, capture_output=True)


def cover(w, h):
    """ffmpeg: fill w by h, cropping what does not fit."""
    return f"scale={w}:{h}:force_original_aspect_ratio=increase,crop={w}:{h}"


def webm(frames_in, out, fps, vf):
    run("ffmpeg", "-loglevel", "error", "-y", *frames_in, "-vf", vf, "-an", "-r", fps,
        "-c:v", "libvpx-vp9", "-crf", "40", "-b:v", "0", "-pix_fmt", "yuv420p", out)


def main():
    for tool in ("ffmpeg", "magick"):
        if not shutil.which(tool):
            sys.exit(f"city-media: {tool} not found")
    spec = tomllib.loads((EXT / "three/media.toml").read_text())
    OUT.mkdir(parents=True, exist_ok=True)
    notes = ["# The pictures on the screens", "",
             "Small copies made by `scripts/city-media.py` from `extensions/three/media.toml`.",
             "The originals are the author's own: `blog:` is the blog's",
             "`assets/images/posts`, `ext:` this repository's extensions.", ""]
    with tempfile.TemporaryDirectory() as tmp:
        tmp = pathlib.Path(tmp)
        for show in spec.get("slides", []):
            name, (w, h), hold = show["name"], show["size"], show["hold"]
            notes += [f"## {name} (slides, {w} by {h}, {hold} s each)", ""]
            for i, source in enumerate(show["sources"], 1):
                # the first frame, on white where it is see-through (line art)
                run("magick", f"{find(source)}[0]", "-background", "white", "-alpha", "remove", "-alpha", "off",
                    "-resize", f"{w}x{h}^", "-gravity", "center", "-extent", f"{w}x{h}",
                    "-quality", "82", OUT / f"{name}-{i}.jpg")
                notes.append(f"- `{name}-{i}.jpg`: `{source}`")
            webm(["-framerate", f"1/{hold}", "-i", OUT / f"{name}-%d.jpg"], OUT / f"{name}.webm", 6, f"scale={w}:{h}")
            notes += [f"- `{name}.webm`: the same slides as a video (for Blender)", ""]
            print(f"city-media: {name}: {len(show['sources'])} slides")
        notes += [f"## Videos ({REEL[0]} by {REEL[1]}, looping)", ""]
        for reel in spec.get("reels", []):
            name, source = reel["name"], find(reel["source"])
            out = OUT / f"{name}.webm"
            if source.suffix == ".webp":
                frames = tmp / name
                frames.mkdir()
                run("magick", source, "-coalesce", "-background", "white", "-alpha", "remove", frames / "f-%03d.png")
                webm(["-framerate", reel.get("fps", 8), "-i", frames / "f-%03d.png"], out, 12, cover(*REEL))
            else:
                webm(["-ss", reel.get("start", 0), "-t", reel.get("length", 10), "-i", source], out, 12, cover(*REEL))
            notes.append(f"- `{name}.webm`: `{reel['source']}`")
            print(f"city-media: {name}: {out.stat().st_size // 1024} KB")
    (OUT / "SOURCES.md").write_text("\n".join(notes) + "\n")
    total = sum(f.stat().st_size for f in OUT.iterdir())
    print(f"city-media: {len(list(OUT.iterdir()))} files, {total // 1024} KB in {OUT.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
