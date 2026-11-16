# The render farm

Blender work -- a film's frames, texture bakes, exports of pieces and
clips -- divided between several machines, each with this repository
and a graphics card. No machine is in charge and nothing is sent to a
machine: every machine reads the same two tracked tables, works out
the same division, and does the share that is its own, found by its
host name.

- `farm/hosts.toml`: the machines, how fast each is against the
  others (`speed`), its video memory (`vram`), and whether it is
  taking part (`on`).
- `farm/jobs.toml`: the work, a table a job.
- `scripts/farm.py`: the one command (`just farm ...`).
- `extensions/blender/tools/farm-range.py` and `farm-bench.py`: what
  Blender runs.

The machines: large12 (RTX 5060, 16 GB), beast48 and big72 (RTX 3060,
12 GB each), queenbee (RTX 3090, 24 GB) and max (the Mac, an M1 Max). Off in the table as of
2026-11-16: big72 (it does not boot) and queenbee (powered down).
Blender is 5.3.0 on all of them (Arch's package); the scripts were
written against 5.1.2.

## On each machine, once

```sh
git clone https://github.com/softwarewrighter/X_eTaL-extensions.git ~/github/softwarewrighter/X_eTaL-extensions
cd ~/github/softwarewrighter/X_eTaL-extensions
just venv         # the Python environment (needs uv)
just farm check   # Blender, the card Cycles will use, the disk, this machine's share
just farm bench   # about a minute: the seconds a standard picture takes
```

Needed: Blender (5.3.0, Arch's `blender` package: the same on all, or
pictures differ), `uv`, `just`, a Rust toolchain (the
scene's script is written by an X_eTaL program, which `just farm run`
builds the first time: some minutes), `rsync` and `ssh`.

`just farm check` fails when the machine's name is not in
`farm/hosts.toml`, when Blender is not found, or when Cycles lists no
device. If Cycles lists only the processor, the card's driver or
Blender's build is the thing to look at before rendering anything.

Put each machine's benchmark seconds in `farm/hosts.toml` as `speed`:
the slowest machine's seconds divided by this machine's. Commit and
push the table, and pull it everywhere: **every machine must have the
same tables**, or two machines will render the same frames and some
frames will be nobody's. `just farm plan` on two machines must print
the same thing.

## A job

```sh
git pull
just farm plan city-film     # who does what
just farm run city-film      # this machine's share
just farm status city-film
```

`run` does only what is not yet done: a frame whose picture is there
and an item whose `.done` is there are skipped, so a run that stopped
is simply run again. Results are under `work/farm/JOB/` (ignored by
git): `frames/NNNN.png` for a film, `ITEM/` for an each-job, and a log
a machine.

When the machines have finished, on the machine that is to have the
whole (any one of them, or the Mac):

```sh
just farm collect city-film  # rsync from each other machine; says what is missing
just farm encode city-film   # the frames into work/farm/city-film/city.mp4
```

`collect` reaches a machine as `ssh NAME` at
`~/github/softwarewrighter/X_eTaL-extensions`; `ssh` and `path` in the
hosts table say otherwise.

## When a machine is off or slow

- A machine that will not take part: `on = false` in the table,
  committed and pulled everywhere before anyone starts. The division
  changes for everyone, so do it before a job, not during.
- During a job, to take over what another machine has not done:
  `just farm run city-film --as big72` does big72's share here, and
  `--only 326-360` (frames) or `--only ITEM,ITEM` does just those,
  whoever's they are. Then `collect` as usual.

## The two kinds of job

A film is a scene's script and a range of frames; the frames are
divided into consecutive runs in proportion to speed.

```toml
[jobs.city-film]
kind = "film"
prepare = "scripts/farm-scene.sh farm/city-film.xtl"
scene = "work/farm/city/scene.py"
first = 1
last = 450
video = "city.mp4"
```

`prepare` runs on each machine before its share and writes the scene:
an X_eTaL program (`farm/city-film.xtl`) writing Cycles.xtl's player
and the scene's calls, with nothing in it that renders. A scene's own
`still`, `frames` and `encode` lines are left out when the farm runs
it, so an existing demo's script can be used as it is.

An each-job is a list of items and a command an item; items are dealt
out in proportion to speed, only to machines with the video memory
the job asks for (`vram`, in GB): the way to keep a large bake off the
12 GB cards.

```toml
[jobs.city-stills]
kind = "each"
items = ["0.5", "6.0", "10.5", "14.5"]
command = '"$BLENDER" --background ... --python extensions/blender/tools/farm-range.py -- {out} work/farm/city/scene.py still at-{item}.png {item}'
```

The command runs from the repository's root with `{item}` and `{out}`
(the item's own directory) filled in, and `BLENDER`, `FARM_ITEM`,
`FARM_OUT` and `FARM_HOST` set. It may be anything: a bake, an export
of one piece, a clip.

Another repository uses the same command with its own tables:
`scripts/farm.py --hosts FILE --jobs FILE ...`.

## Small results

Frames stay out of git. A small result that belongs in the repository
(a `.glb`, a baked texture) is committed on that machine's own branch,
named for the machine (CLAUDE.md, "Working a branch in parallel with
another agent": `agentrail rename prefix large12`), and merged from
there.

## For Claude on a farm machine

Your machine's name is `hostname -s`; your share of every job is what
`just farm check` prints. Do: `git pull`, `just farm check`, then
`just farm run JOB` for the job you were asked for, and report what it
printed, the log's last lines if it failed
(`work/farm/JOB/NAME.log`), and `just farm status JOB`. Do not edit
`farm/hosts.toml` or `farm/jobs.toml` during a job, do not render
another machine's share unless asked (`--as`), and do not run the
repository's gate beside a render.

## What has and has not been tried

The division, the Blender helpers and the whole of a small film (three
frames by three pretended machines, then encoded) and a still were
run on one Mac. The Arch machines' first `just farm check` and `just
farm bench` are the first runs on Linux, and `collect` between
machines has not been run at all: expect to correct paths, the speeds
and perhaps Blender's options (`XETAL_BLENDER_ARGS`) there.
