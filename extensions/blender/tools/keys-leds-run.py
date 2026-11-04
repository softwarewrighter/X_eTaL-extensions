#!/usr/bin/env python3
"""Read physical-calculator waveforms and sample them at X_eTaL's times.

No scene geometry or animation timing is chosen here. The model replay
and sampling map are X_eTaL; this adapter reads VCD and decodes real SPI.
"""
import argparse
import bisect
import hashlib
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
EXT = ROOT / "extensions/blender"
OUT = EXT / "work/keysrun"
CLONE = ROOT / "work/hex1000"


def run(args, name, timeout=1200):
    result = subprocess.run(list(map(str, args)), cwd=EXT, env=environment(), capture_output=True, text=True, timeout=timeout)
    (OUT / (name + ".log")).write_text(result.stdout + result.stderr)
    if result.returncode:
        raise RuntimeError(f"{name} failed; see {OUT / (name + '.log')}")
    return result.stdout


def environment():
    paths = [CLONE / f"libs/{name}/src" for name in ("Front", "Panel", "Hex1000", "Radix")]
    paths += [ROOT / "work/hex1000-libraries/libs/Strings/src", EXT / "lib", EXT / "demos", ROOT / "lib"]
    env = dict(os.environ, XETAL_PATH=os.pathsep.join(map(str, paths)))
    env.setdefault("XETAL_BLENDER_TIMEOUT", "1800")
    return env


def fingerprints():
    paths = [OUT / name for name in ("trace.txt", "run.vcd", "events.txt", "model-check.log")]
    paths += [EXT / "tools/keys-leds-replay.xtl", EXT / "demos/hex-keys-leds-run.tb.v", ROOT / "HEX1000_COMMIT", ROOT / "XETAL_COMMIT"]
    return {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest() for path in paths}


def checked_record():
    subprocess.run(["bash", ROOT / "scripts/hex1000.sh", "--check"], check=True, stdout=subprocess.DEVNULL)
    record = json.loads((OUT / "run-evidence.json").read_text())
    if record.get("status") != "checked" or record.get("model_inputs") != fingerprints():
        raise ValueError("simulation/model inputs changed; run prepare again")
    for name, digest in record["sha256"].items():
        if hashlib.sha256((OUT / name).read_bytes()).hexdigest() != digest:
            raise ValueError(f"checked evidence changed: {name}")
    if int((OUT / "spi-word.txt").read_text()) != record["selected_spi"]["word"]:
        raise ValueError("selected serial word changed")
    return record


def read_wave(path, font, setup):
    waves = {}
    ids = {}
    values = {}
    transactions = []
    screens = []
    display = [0] * 8
    bits = []
    start = 0
    packet = {}

    def commit(at):
        nonlocal bits, start
        previous = dict(values)
        values.update(packet)
        for name, value in packet.items():
            waves.setdefault(name, []).append((at, value))
        if previous.get("load") == 1 and values.get("load") == 0:
            start = at
            if bits:
                raise ValueError("incomplete SPI word before LOAD falls")
            waves.setdefault("spi_bits", []).append((at, 0))
        if previous.get("sclk") == 0 and values.get("sclk") == 1:
            if values.get("load") != 0 or values.get("din") not in (0, 1):
                raise ValueError("invalid SPI clock/data")
            bits.append(values["din"])
            waves.setdefault("spi_bits", []).append((at, len(bits)))
        if previous.get("load") == 0 and values.get("load") == 1 and bits:
            if len(bits) != 16:
                raise ValueError("SPI word is not 16 bits")
            word = 0
            for bit in bits:
                word = word * 2 + bit
            bits = []
            address, data = word >> 8, word & 255
            if 1 <= address <= 8:
                if data and data not in font:
                    raise ValueError("unknown MAX7219 glyph")
                display[address - 1] = data
            elif word not in setup:
                raise ValueError(f"unexpected initialization word {word:04x}")
            transactions.append({"start": start, "end": at, "word": word})
            screens.append((at, "".join(" " if v == 0 else "0123456789ABCDEF"[font.index(v)] for v in reversed(display))))
            for i, value in enumerate(display):
                name = f"display{i}"
                if not waves.get(name) or waves[name][-1][1] != value:
                    waves.setdefault(name, []).append((at, value))

    with path.open() as source:
        header = True
        at = 0
        scale = False
        for raw in source:
            line = raw.strip()
            if header:
                if line == "1ps":
                    scale = True
                if line.startswith("$var"):
                    parts = line.split()
                    ids.setdefault(parts[3], []).append(parts[4])
                if line.startswith("$enddefinitions"):
                    if not scale:
                        raise ValueError("expected VCD timescale 1 ps")
                    header = False
                continue
            if not line or line.startswith("$"):
                continue
            if line.startswith("#"):
                commit(at)
                packet = {}
                at = int(line[1:])
            else:
                if line.startswith("b"):
                    binary, key = line[1:].split()
                else:
                    binary, key = line[0], line[1:]
                value = None if any(c in binary.lower() for c in "xz") else int(binary, 2)
                for name in ids[key]:
                    packet[name] = value
        commit(at)
    return waves, transactions, screens


def at(wave, time):
    index = bisect.bisect_right(wave, time, key=lambda item: item[0]) - 1
    return wave[index][1] if index >= 0 else 0


def prepare(reuse):
    OUT.mkdir(parents=True, exist_ok=True)
    subprocess.run(["bash", ROOT / "scripts/hex1000.sh", "--check"], check=True, stdout=subprocess.DEVNULL)
    if reuse:
        checked_record()
    if not reuse:
        (OUT / "run-evidence.json").write_text('{"status":"in-progress"}\n')
        run(["iverilog", "-g2012", "-s", "tb", "-o", OUT / "run.vvp", CLONE / "boards/tangnano1k/calculator.v",
             CLONE / "libs/Front/rtl/front.v", CLONE / "libs/Hex1000/rtl/hex1000.v", EXT / "demos/hex-keys-leds-run.tb.v"], "compile")
        # The testbench owns relative output paths in this directory.
        result = subprocess.run(["vvp", "run.vvp"], cwd=OUT, capture_output=True, text=True, timeout=300)
        (OUT / "simulation.log").write_text(result.stdout + result.stderr)
        if result.returncode:
            raise ValueError("board simulation failed")
        run([ROOT / "bin/xetal", "run", EXT / "tools/keys-leds-replay.xtl"], "model-check")
    checked = (OUT / "model-check.log").read_text()
    if "PASS Hex1000 and Front replay:" not in checked:
        raise ValueError("missing model check")
    tables = run([ROOT / "bin/xetal", "eval", "-e",
                  '"fr:" u_se< "Front"\nfr:s_eg 16 + o_ffsets 16\n\'{ i -> fr:w_ord (i, fr:BootDigits) } e_ach o_ffsets 5'], "model-font").splitlines()
    font, setup = [list(map(int, line.split())) for line in tables]
    waves, transactions, screens = read_wave(OUT / "run.vcd", font, setup)
    events = [list(map(int, row.split())) for row in (OUT / "events.txt").read_text().splitlines()]
    if [event[2] for event in events] != [12, 0, 13, 14, 16, 11, 14, 14, 15, 19, 19]:
        raise ValueError("physical key events differ")
    stage_times = {value: t for t, value in reversed(waves["stage"]) if value is not None}
    first = next(t for t, value in waves["reset"] if value == 0)
    first_accept = next(t for t, value in waves["accepted"] if value)
    presses = [event[0] * 1000 for event in events]
    bounce_end = next(t for t, value in waves["pressed"] if t > stage_times[1] and value == 0)
    if first_accept <= presses[0]:
        raise ValueError("bounce was accepted")
    if not any(text == "DEADBEEF" and t < presses[0] for t, text in screens):
        raise ValueError("SPI did not show the startup banner")
    before_clear = [text for t, text in screens if t < presses[-1]][-1]
    if before_clear.strip() != "17FCD" or screens[-1][1].strip() != "0":
        raise ValueError("decoded display does not show 17FCD then zero")
    accepted_codes = [(t, at(waves["key_code"], t)) for t, phase in waves["phase"] if phase == 1]
    if [code for _, code in accepted_codes] != [12, 0, 13, 14, 16, 11, 14, 14, 15, 19, 20]:
        raise ValueError("held key repeated or second-equals translation differs")
    selected = next(word for word in transactions if word["start"] >= stage_times[25] and 1 <= word["word"] >> 8 <= 8)
    landmarks = [first, stage_times[1], bounce_end, presses[0], first_accept, presses[1], presses[9], stage_times[25],
                 selected["start"], selected["end"], presses[-1], stage_times[31], waves["machine_clk"][-1][0]]
    if any(b <= a for a, b in zip(landmarks, landmarks[1:])):
        raise ValueError("non-increasing timing landmarks")
    (OUT / "landmarks.txt").write_text(" ".join(map(str, landmarks)) + "\n")
    (OUT / "spi-word.txt").write_text(str(selected["word"]) + "\n")
    (OUT / "waveforms.json").write_text(json.dumps(waves))
    record = {"status": "checked", "project_commit": (ROOT / "HEX1000_COMMIT").read_text().strip(),
              "model_inputs": fingerprints(),
              "model_replay": checked.strip(), "spi_words": len(transactions), "font": font,
              "selected_spi": selected, "accepted_codes": accepted_codes, "nominal_clock_hz": 27000000,
              "simulated_clock_period_ns": 37.038, "hardware_actions": [],
              "sha256": {name: hashlib.sha256((OUT / name).read_bytes()).hexdigest()
                         for name in ("trace.txt", "run.vcd", "events.txt", "model-check.log", "landmarks.txt", "waveforms.json")}}
    (OUT / "run-evidence.json").write_text(json.dumps(record, indent=2) + "\n")
    print(f"PASS VCD: {len(transactions)} SPI words; DEADBEEF -> 17FCD -> 0; 11 accepted keys")


def samples():
    checked_record()
    waves = json.loads((OUT / "waveforms.json").read_text())
    names = ["pressed", "ops_n", "scan_rows", "cols_sync", "accepted", "stable", "phase", "key_code", "a", "pc",
             "status", "din", "sclk", "load", "oe"] + [f"display{i}" for i in range(8)] + ["stage", "spi_bits"]
    times = list(map(int, (OUT / "sample-times.txt").read_text().split()))
    first = int((OUT / "landmarks.txt").read_text().split()[0])
    rows = []
    for time in times:
        row = [at(waves[name], time) for name in names]
        if time >= first and any(value is None for value in row[:15]):
            raise ValueError("unknown signal after startup")
        rows.append(" ".join(map(str, [time] + [v or 0 for v in row])))
    (OUT / "samples.txt").write_text("\n".join(rows) + "\n")
    print(f"sampled {len(rows)} video frames from checked VCD")


def render(preview):
    checked_record()
    run([ROOT / "bin/xetal", "run", EXT / "tools/keys-leds-sampling.xtl"], "sampling")
    samples()
    script = "tests/keys-leds-quick.xtl" if preview else "demos/hex-keys-leds-run.xtl"
    print("rendering " + script, flush=True)
    print(run([ROOT / "target/release/xetal-x", "--ext", ROOT / "extensions", "run", script],
              "preview" if preview else "render", timeout=7200), flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("prepare", "samples", "preview", "render"))
    parser.add_argument("--reuse-checked", action="store_true", help="reuse the local simulation/model replay after checking its files")
    args = parser.parse_args()
    if args.action == "prepare":
        prepare(args.reuse_checked)
    elif args.action == "samples":
        samples()
    else:
        render(args.action == "preview")
