#!/usr/bin/env python3
"""Copy a freshly built site into pages/ only where something changed.

    scripts/sync-pages.py FRESH DEST

pages/ is committed, so a rebuild that rewrites files whose content did
not really change would add churn to the repository on every build. A
file is copied from FRESH into DEST only when it is new or differs in a
line that matters: lines that differ only in the build stamp ("built at
COMMIT") or in an inferred type (xetal doc types some functions
differently from run to run, docs/xetal-asks.md E9) are not reasons to
rewrite it. A file in DEST that FRESH no longer has is listed, never
deleted (CLAUDE.md rule 18: ask before deleting anything); the pages
check fails until it is dealt with.
"""
import re
import shutil
import sys
from pathlib import Path

IGNORED = re.compile(r'built at [0-9a-f]*|class="type"|<td>function</td>|","function","')


def lines_that_matter(path: Path) -> list[str] | bytes:
    data = path.read_bytes()
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError:
        return data
    return [line for line in text.splitlines() if not IGNORED.search(line)]


def main() -> int:
    fresh, dest = Path(sys.argv[1]), Path(sys.argv[2])
    copied, obsolete = 0, []
    for f in sorted(p for p in fresh.rglob("*") if p.is_file()):
        rel = f.relative_to(fresh)
        target = dest / rel
        if target.is_file() and lines_that_matter(target) == lines_that_matter(f):
            continue
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(f, target)
        copied += 1
    for f in sorted(p for p in dest.rglob("*") if p.is_file()):
        if not (fresh / f.relative_to(dest)).is_file():
            obsolete.append(f.relative_to(dest))
    print(f"pages: {copied} file(s) written to {dest}")
    for rel in obsolete:
        print(f"pages: obsolete, not removed (ask before deleting): {dest / rel}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
