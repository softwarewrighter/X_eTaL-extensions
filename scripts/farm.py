#!/usr/bin/env python3
"""The render farm: Blender work divided between machines by their names.

Every machine has this repository and runs the same command; each reads
the same two tracked tables -- farm/hosts.toml (the machines, how fast
each is, how much video memory) and farm/jobs.toml (the work) -- works
out the same division, and does the part that is its own, found by its
host name. Nothing is sent to a machine and no machine is in charge.

  scripts/farm.py check             this machine: Blender, the graphics card, the disk, its share of every job
  scripts/farm.py plan [JOB]        who does what
  scripts/farm.py run JOB           this machine's share of a job (again: only what is not yet done)
  scripts/farm.py status [JOB]      how much of this machine's share is done
  scripts/farm.py collect JOB       fetch the other machines' results here (rsync), say what is missing
  scripts/farm.py encode JOB        a film's frames into its video (after collect)
  scripts/farm.py bench             time a standard picture: the number to put in hosts.toml as speed
  scripts/farm.py --self-test       the division's own checks (in the gate)

Options: --as HOST (do or show another machine's share: one that is off),
--only A-B (frames) or --only ITEM,ITEM (do just these, whoever's they
are), --hosts FILE, --jobs FILE (another repository's tables), --dry
(say what would run).

A film job is a scene script (Cycles.xtl's: the player and the scene's
calls, written by the job's prepare command) and a range of frames;
frames are divided into consecutive runs in proportion to speed. An
each job is a list of items and a command an item; items are dealt to
the machines in proportion to speed, only to machines with the video
memory the job asks for. Results are under work/farm/JOB/ (ignored by
git): frames/NNNN.png for a film, ITEM/ and ITEM/.done for an each job.
"""

import argparse
import os
import shlex
import shutil
import socket
import subprocess
import sys
import time
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RANGE = ROOT / "extensions" / "blender" / "tools" / "farm-range.py"
BENCH = ROOT / "extensions" / "blender" / "tools" / "farm-bench.py"


def fail(message):
    sys.exit(f"farm: {message}")


def load(path):
    try:
        with open(path, "rb") as f:
            return tomllib.load(f)
    except FileNotFoundError:
        fail(f"{path}: no such file")
    except tomllib.TOMLDecodeError as e:
        fail(f"{path}: {e}")


def hosts_of(table):
    """The machines that are on, in the table's order: name -> its row."""
    hosts = {}
    for name, row in table.get("hosts", {}).items():
        if row.get("on", True):
            speed = float(row.get("speed", 1.0))
            if speed <= 0:
                fail(f"host {name}: speed must be more than 0")
            hosts[name] = {"speed": speed, "vram": float(row.get("vram", 0)), **row}
    if not hosts:
        fail("no machine is on in the hosts table")
    return hosts


def shares(n, weights):
    """n things in proportion to the weights: whole numbers that sum to n
    (largest remainders first, the earlier machine on a tie)."""
    total = sum(weights)
    exact = [n * w / total for w in weights]
    whole = [int(x) for x in exact]
    order = sorted(range(len(weights)), key=lambda i: (-(exact[i] - whole[i]), i))
    for i in order[: n - sum(whole)]:
        whole[i] += 1
    return whole


def ranges(first, last, hosts):
    """A film's frames as consecutive runs, a machine each: name -> (first, last) or None."""
    names = list(hosts)
    counts = shares(last - first + 1, [hosts[h]["speed"] for h in names])
    out, at = {}, first
    for name, count in zip(names, counts):
        out[name] = (at, at + count - 1) if count else None
        at += count
    return out


def deal(items, hosts, vram=0.0):
    """An each job's items dealt in proportion to speed, to the machines
    with enough video memory: name -> its items. Each item goes to the
    machine that would be least loaded after taking it."""
    able = [h for h in hosts if hosts[h]["vram"] >= vram]
    if not able:
        fail(f"no machine on has {vram:g} GB of video memory")
    out = {h: [] for h in hosts}
    for item in items:
        best = min(able, key=lambda h: ((len(out[h]) + 1) / hosts[h]["speed"], able.index(h)))
        out[best].append(item)
    return out


def self_test():
    assert shares(450, [1.6, 1.0, 1.0]) == [200, 125, 125]
    assert shares(10, [1, 1, 1]) == [4, 3, 3]
    assert shares(2, [1, 1, 1]) == [1, 1, 0]
    assert shares(0, [1, 2]) == [0, 0]
    for n in range(0, 200, 7):
        for weights in ([1.6, 1.0, 1.0], [1.6, 1.0, 1.0, 2.3], [3.0], [0.1, 9.0]):
            got = shares(n, weights)
            assert sum(got) == n and all(g >= 0 for g in got), (n, weights, got)
    hosts = {"a": {"speed": 1.6, "vram": 16}, "b": {"speed": 1.0, "vram": 12}, "c": {"speed": 1.0, "vram": 12}}
    r = ranges(1, 450, hosts)
    assert r == {"a": (1, 200), "b": (201, 325), "c": (326, 450)}, r
    r = ranges(5, 6, hosts)
    assert r == {"a": (5, 5), "b": (6, 6), "c": None}, r
    # every frame once
    for first, last in ((1, 1), (1, 7), (10, 449)):
        seen = []
        for run in ranges(first, last, hosts).values():
            if run:
                seen.extend(range(run[0], run[1] + 1))
        assert seen == list(range(first, last + 1))
    d = deal(list("abcdefghi"), hosts)
    assert sorted(sum(d.values(), [])) == list("abcdefghi")
    assert len(d["a"]) >= len(d["b"]) >= len(d["c"])
    d = deal(list("abc"), hosts, vram=16)
    assert d == {"a": ["a", "b", "c"], "b": [], "c": []}
    assert parse_only("3-5,9") == [3, 4, 5, 9]
    print("farm: self-test ok")


def parse_only(text):
    frames = []
    for part in text.split(","):
        a, _, b = part.partition("-")
        frames.extend(range(int(a), int(b or a) + 1))
    return frames


def find_blender(row):
    for candidate in (os.environ.get("BLENDER"), row.get("blender"), "/Applications/Blender.app/Contents/MacOS/Blender", shutil.which("blender")):
        if candidate and Path(candidate).is_file():
            return candidate
    return None


def blender_run(blender, script, args, log, dry=False, timeout=None, fresh=False):
    """Blender in the background on a script, its output to a log; the exit status."""
    command = [blender, "--background", *shlex.split(os.environ.get("XETAL_BLENDER_ARGS", "")), "-noaudio", "--factory-startup",
               "--python-exit-code", "1", "--python", str(script), "--", *map(str, args)]
    if dry:
        print("  would run: " + shlex.join(command))
        return 0
    log.parent.mkdir(parents=True, exist_ok=True)
    with open(log, "wb" if fresh else "ab") as out:
        out.write(f"\n== {time.strftime('%Y-%m-%d %H:%M:%S')} {shlex.join(command)}\n".encode())
        out.flush()
        try:
            return subprocess.run(command, stdout=out, stderr=subprocess.STDOUT, cwd=ROOT, timeout=timeout).returncode
        except subprocess.TimeoutExpired:
            return 124


class Farm:
    def __init__(self, args):
        self.args = args
        self.hosts = hosts_of(load(args.hosts))
        self.jobs = load(args.jobs).get("jobs", {})
        self.me = args.as_host or socket.gethostname().split(".")[0]
        self.real = socket.gethostname().split(".")[0]

    def job(self, name):
        if name not in self.jobs:
            fail(f"no job {name}; the jobs: {', '.join(self.jobs) or 'none'}")
        job = self.jobs[name]
        kind = job.get("kind")
        if kind not in ("film", "each"):
            fail(f"job {name}: kind must be film or each")
        return job

    def mine(self):
        if self.me not in self.hosts:
            fail(f"this machine, {self.me}, is not on in {self.args.hosts} (add it, or say --as HOST)")
        return self.hosts[self.me]

    def out(self, name):
        return ROOT / "work" / "farm" / name

    def split(self, name):
        """Who does what of a job: name -> (first, last) or a list of items."""
        job = self.job(name)
        if job["kind"] == "film":
            return ranges(int(job["first"]), int(job["last"]), self.hosts)
        return deal(list(job["items"]), self.hosts, float(job.get("vram", 0)))

    def frame_file(self, name, n):
        return self.out(name) / "frames" / f"{n:04d}.png"

    def wanted(self, name, host):
        """What a machine is to do of a job: frame numbers, or items."""
        job = self.job(name)
        if self.args.only:
            return parse_only(self.args.only) if job["kind"] == "film" else self.args.only.split(",")
        part = self.split(name)[host]
        if job["kind"] == "film":
            return list(range(part[0], part[1] + 1)) if part else []
        return part

    def done(self, name, thing):
        if self.job(name)["kind"] == "film":
            return self.frame_file(name, thing).is_file()
        return (self.out(name) / str(thing) / ".done").is_file()

    def tables(self):
        """A short mark of the two tables: the same on machines that will divide the work the same way."""
        import hashlib
        mark = hashlib.sha256(Path(self.args.hosts).read_bytes() + b"\0" + Path(self.args.jobs).read_bytes()).hexdigest()[:8]
        return f"tables {mark} (every machine must say the same)"

    def plan(self, names):
        print(self.tables())
        for name in names:
            job = self.job(name)
            split = self.split(name)
            if job["kind"] == "film":
                print(f"{name}: a film, frames {job['first']} to {job['last']}")
                for host, part in split.items():
                    what = f"frames {part[0]} to {part[1]} ({part[1] - part[0] + 1})" if part else "nothing"
                    print(f"  {host:<12} {what}")
            else:
                print(f"{name}: {len(job['items'])} items" + (f", {job['vram']} GB of video memory or more" if job.get("vram") else ""))
                for host, items in split.items():
                    print(f"  {host:<12} {len(items)}: {' '.join(map(str, items)) or 'nothing'}")

    def prepare(self, name):
        command = self.job(name).get("prepare")
        if not command:
            return
        print(f"farm: {name}: prepare: {command}")
        if self.args.dry:
            return
        if subprocess.run(command, shell=True, cwd=ROOT).returncode != 0:
            fail(f"{name}: prepare failed")

    def run(self, name):
        job = self.job(name)
        row = self.mine()
        todo = [t for t in self.wanted(name, self.me) if not self.done(name, t)]
        if not todo:
            print(f"farm: {name}: nothing left for {self.me}")
            return
        blender = find_blender(row)
        if not blender and not self.args.dry:
            fail("no Blender found (BLENDER, the hosts table's blender, or blender on PATH)")
        self.prepare(name)
        out = self.out(name)
        log = out / f"{self.real}.log"
        began = time.time()
        if job["kind"] == "film":
            scene = ROOT / job["scene"]
            if not scene.is_file() and not self.args.dry:
                fail(f"{name}: {job['scene']} is not there (its prepare command writes it)")
            # consecutive runs of what is left, a Blender each (it skips frames that are there)
            runs, start = [], todo[0]
            for a, b in zip(todo, todo[1:] + [None]):
                if b != a + 1:
                    runs.append((start, a))
                    start = b
            for first, last in runs:
                print(f"farm: {name}: {self.me} renders frames {first} to {last} ({self.tables()}); log {log.relative_to(ROOT)}")
                code = blender_run(blender or "blender", RANGE, [out, scene, "frames", first, last], log, self.args.dry)
                if code != 0:
                    fail(f"{name}: Blender ended with {code}; see {log.relative_to(ROOT)}")
        else:
            for item in todo:
                place = out / str(item)
                command = job["command"].format(item=item, out=place.relative_to(ROOT))
                print(f"farm: {name}: {self.me} does {item}: {command}")
                if self.args.dry:
                    continue
                place.mkdir(parents=True, exist_ok=True)
                env = {**os.environ, "FARM_ITEM": str(item), "FARM_OUT": str(place), "FARM_HOST": self.me, **({"BLENDER": blender} if blender else {})}
                with open(log, "ab") as f:
                    f.write(f"\n== {time.strftime('%Y-%m-%d %H:%M:%S')} {item}: {command}\n".encode())
                    f.flush()
                    code = subprocess.run(command, shell=True, cwd=ROOT, env=env, stdout=f, stderr=subprocess.STDOUT).returncode
                if code != 0:
                    fail(f"{name}: {item} ended with {code}; see {log.relative_to(ROOT)}")
                (place / ".done").write_text(f"{self.real} {time.strftime('%Y-%m-%d %H:%M:%S')}\n")
        if not self.args.dry:
            left = [t for t in self.wanted(name, self.me) if not self.done(name, t)]
            took = time.time() - began
            print(f"farm: {name}: {self.me} did {len(todo) - len(left)} of {len(todo)} in {took:.0f} s" + (f"; NOT done: {left[:20]}" if left else ""))
            if left:
                sys.exit(1)

    def status(self, names):
        for name in names:
            mine = self.wanted(name, self.me) if self.me in self.hosts else []
            got = sum(1 for t in mine if self.done(name, t))
            every = [t for h in self.hosts for t in self.wanted_of(name, h)]
            here = sum(1 for t in every if self.done(name, t))
            print(f"{name}: {self.me} has done {got} of its {len(mine)}; {here} of all {len(every)} are on this machine")

    def wanted_of(self, name, host):
        part = self.split(name)[host]
        if self.job(name)["kind"] == "film":
            return list(range(part[0], part[1] + 1)) if part else []
        return part

    def collect(self, name):
        job = self.job(name)
        out = self.out(name)
        out.mkdir(parents=True, exist_ok=True)
        for host, row in self.hosts.items():
            if host == self.real:
                continue
            there = f"{row.get('ssh', host)}:{row.get('path', '~/github/softwarewrighter/X_eTaL-extensions')}/work/farm/{name}/"
            command = ["rsync", "-a", "--ignore-existing", "--exclude", "*.log", there, f"{out}/"]
            print("farm: " + shlex.join(command))
            if not self.args.dry and subprocess.run(command).returncode != 0:
                print(f"farm: could not fetch from {host} (is it on, and reachable as {row.get('ssh', host)}?)")
        missing = {}
        for host in self.hosts:
            lack = [t for t in self.wanted_of(name, host) if not self.done(name, t)]
            if lack:
                missing[host] = lack
        if not missing:
            print(f"farm: {name}: all here" + ("; now: scripts/farm.py encode " + name if job["kind"] == "film" else ""))
            return
        for host, lack in missing.items():
            print(f"farm: {name}: {len(lack)} of {host}'s not here: {' '.join(map(str, lack[:12]))}{' ...' if len(lack) > 12 else ''}")
        sys.exit(1)

    def encode(self, name):
        job = self.job(name)
        if job["kind"] != "film":
            fail(f"{name} is not a film")
        lack = [n for n in range(int(job["first"]), int(job["last"]) + 1) if not self.frame_file(name, n).is_file()]
        if lack and not self.args.dry:
            fail(f"{name}: {len(lack)} frames are not here (collect first): {lack[:12]}")
        blender = find_blender(self.hosts.get(self.real, {}))
        if not blender and not self.args.dry:
            fail("no Blender found")
        out = self.out(name)
        log = out / f"{self.real}.log"
        code = blender_run(blender or "blender", RANGE, [out, ROOT / job["scene"], "encode", job.get("video", name + ".mp4")], log, self.args.dry)
        if code != 0:
            fail(f"{name}: Blender ended with {code}; see {log.relative_to(ROOT)}")
        if not self.args.dry:
            print(f"farm: {name}: {(out / job.get('video', name + '.mp4')).relative_to(ROOT)}")

    def check(self):
        name = self.real
        print(f"machine: {name}" + ("" if name in self.hosts else f"  (NOT on in {self.args.hosts})"))
        row = self.hosts.get(name, {})
        blender = find_blender(row)
        ok = bool(blender)
        if blender:
            version = subprocess.run([blender, "--version"], capture_output=True, text=True).stdout.splitlines()[:1]
            print(f"Blender: {blender} ({version[0] if version else '?'})")
            log = ROOT / "work" / "farm" / f"check-{name}.log"
            code = blender_run(blender, BENCH, ["devices"], log, fresh=True)
            found = [line[8:] for line in log.read_text(errors="replace").splitlines() if line.startswith("device: ")]
            for line in found:
                print(f"Cycles:  {line}")
            if code != 0 or not found:
                print(f"Cycles:  no device listed (see {log.relative_to(ROOT)})")
                ok = False
            elif all(line.startswith("CPU") or " CPU " in line for line in found):
                print("Cycles:  only the processor: renders will be slow")
        else:
            print("Blender: NOT FOUND (set BLENDER, or blender in the hosts table, or put blender on PATH)")
        smi = shutil.which("nvidia-smi")
        if smi:
            cards = subprocess.run([smi, "--query-gpu=name,memory.total,driver_version", "--format=csv,noheader"], capture_output=True, text=True).stdout.strip()
            print(f"card:    {cards or 'none reported'}")
        usage = shutil.disk_usage(ROOT)
        print(f"disk:    {usage.free / 2**30:.0f} GB free under {ROOT}")
        for tool in ("rsync", "ssh"):
            print(f"{tool}:   {shutil.which(tool) or 'NOT FOUND'}")
        if name in self.hosts:
            print(f"table:   speed {self.hosts[name]['speed']:g}, {self.hosts[name]['vram']:g} GB")
            print("this machine's share:")
            for job in self.jobs:
                mine = self.wanted_of(job, name)
                got = sum(1 for t in mine if self.done(job, t))
                print(f"  {job:<16} {len(mine)} ({got} done)")
        if not ok:
            sys.exit(1)

    def bench(self):
        blender = find_blender(self.hosts.get(self.real, {}))
        if not blender:
            fail("no Blender found")
        log = ROOT / "work" / "farm" / f"bench-{self.real}.log"
        code = blender_run(blender, BENCH, ["bench", ROOT / "work" / "farm" / f"bench-{self.real}.png"], log, fresh=True)
        lines = [line for line in log.read_text(errors="replace").splitlines() if line.startswith(("device: ", "bench: "))]
        print("\n".join(lines))
        if code != 0 or not any(line.startswith("bench: ") for line in lines):
            fail(f"the benchmark did not finish; see {log.relative_to(ROOT)}")
        print("speed in hosts.toml is in proportion to 1 / seconds: divide the slowest machine's seconds by each machine's.")


def main():
    p = argparse.ArgumentParser(description="Blender work divided between machines by their names.", epilog=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--self-test", action="store_true")
    p.add_argument("--hosts", default=str(ROOT / "farm" / "hosts.toml"))
    p.add_argument("--jobs", default=str(ROOT / "farm" / "jobs.toml"))
    p.add_argument("--as", dest="as_host", metavar="HOST")
    p.add_argument("--only")
    p.add_argument("--dry", action="store_true")
    p.add_argument("command", nargs="?", choices=["check", "plan", "run", "status", "collect", "encode", "bench"])
    p.add_argument("job", nargs="?")
    args = p.parse_args()
    if args.self_test:
        self_test()
        return
    if not args.command:
        p.print_help()
        sys.exit(2)
    farm = Farm(args)
    if args.command == "check":
        farm.check()
    elif args.command == "bench":
        farm.bench()
    elif args.command in ("plan", "status"):
        names = [args.job] if args.job else list(farm.jobs)
        (farm.plan if args.command == "plan" else farm.status)(names)
    else:
        if not args.job:
            fail(f"{args.command} needs a job; the jobs: {', '.join(farm.jobs) or 'none'}")
        getattr(farm, args.command)(args.job)


if __name__ == "__main__":
    main()
