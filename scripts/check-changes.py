#!/usr/bin/env python3
"""CHANGES.md kept as X_eTaL keeps its own: every entry under a day
(`## YYYY-MM-DD`, newest first) begins with its commit's time and a
category -- `- HH:MM \\`category\\` text` -- the times of a day newest
first. Fails, naming the line, when one does not.
    scripts/check-changes.py [FILE]      (CHANGES.md if not given)
    scripts/check-changes.py --self-test
"""
import pathlib
import re
import sys

CATEGORIES = {"feat", "fix", "refactor", "test", "build", "ext", "abi", "docs", "plan", "release", "chore", "vendor"}
ENTRY = re.compile(r"^- (\d\d):(\d\d) `([a-z]+)` \S")


def problems(text: str) -> list[str]:
    out = []
    day = None
    last_time = None
    days = []
    for n, line in enumerate(text.splitlines(), 1):
        m = re.match(r"^## (\d{4}-\d\d-\d\d)$", line)
        if m:
            day = m.group(1)
            if days and day >= days[-1]:
                out.append(f"line {n}: day {day} is not older than {days[-1]} above it")
            days.append(day)
            last_time = None
            continue
        if line.startswith("## "):
            day = None  # another section (Versions)
            continue
        if day is None or not line.startswith("- "):
            continue
        e = ENTRY.match(line)
        if not e:
            out.append(f"line {n}: not '- HH:MM `category` text': {line[:60]}")
            continue
        hh, mm, cat = int(e.group(1)), int(e.group(2)), e.group(3)
        if hh > 23 or mm > 59:
            out.append(f"line {n}: no time {e.group(1)}:{e.group(2)}")
        if cat not in CATEGORIES:
            out.append(f"line {n}: no category {cat} ({' '.join(sorted(CATEGORIES))})")
        t = hh * 60 + mm
        if last_time is not None and t > last_time:
            out.append(f"line {n}: {e.group(1)}:{e.group(2)} is later than the entry above it")
        last_time = t
    return out


def self_test() -> None:
    good = "# Changes\n\n## Versions\n\n- a version note\n\n## 2026-11-25\n\n- 09:10 `fix` b\n- 09:10 `chore` c\n\n## 2026-11-24\n\n- 23:59 `feat` a\n"
    assert problems(good) == [], problems(good)
    cases = [
        ("## 2026-11-25\n\n- `fix` untimed\n", "not '- HH:MM"),
        ("## 2026-11-25\n\n- 09:10 `nope` x\n", "no category nope"),
        ("## 2026-11-25\n\n- 09:10 `fix` a\n- 10:00 `fix` b\n", "later than the entry above"),
        ("## 2026-11-24\n\n- 09:10 `fix` a\n\n## 2026-11-25\n\n- 09:10 `fix` b\n", "is not older"),
    ]
    for text, want in cases:
        got = problems(text)
        assert any(want in g for g in got), (text, got)
    print(f"check-changes: self-test ok ({len(cases) + 1} cases)")


if __name__ == "__main__":
    if sys.argv[1:] == ["--self-test"]:
        self_test()
        sys.exit(0)
    path = pathlib.Path(sys.argv[1] if sys.argv[1:] else pathlib.Path(__file__).resolve().parent.parent / "CHANGES.md")
    found = problems(path.read_text())
    for p in found:
        print(f"check-changes: {path.name}: {p}")
    if found:
        sys.exit(1)
    print("check-changes: ok (every entry timed and categorized, newest first)")
