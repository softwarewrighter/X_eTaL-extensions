#!/usr/bin/env python3
"""The multivibrator by hand rules, a trial for the planner (as
blinker-trial.py was): prints a command program for
circuits/multivibrator.toml, Manhattan style on the ground-plane tile
(decision C24), every joint a lap.
    .venv/bin/python3 extensions/cell/tools/multivibrator-trial.py [STAGE] > /tmp/m.cell"""
import sys

STAGE = int(sys.argv[1]) if len(sys.argv) > 1 else 99
L = []
wires = [0]


def solder(joint):
    return [f"present {joint} at=S1", f"touch S1 at={joint} dwell=1.5 temp=400",
            "feed S1 mm=2 rate=1", "back S1", f"wait until={joint} below=100", f"inspect {joint}"]


def away(to="0,60,60"):
    return ["lift A mm=60", f"move A to={to}", "joints A 0 0 0 0 90 0"]


def pin(frame):
    return ".".join(frame.split(".")[:2])


def jumper(a, b, via=None, pz="P1", at=1.5, frm=None):
    wires[0] += 1
    w = f"W{wires[0]}"
    v = f" via={via}" if via else ""
    f = f" from={frm}" if frm else ""
    out = [f"wire S5 awg=30 span={a}+{b} lap=3.2{v} enamel=yes", f"pick A {w} grasp=middle",
           "cut S5", "lift A mm=60", f"lay A {w}{f}", f"pinch {pz} on={w}.1 at={at}", f"release A {w}"]
    out += away()
    out += solder(f"{pin(a)}+{w}.1") + solder(f"{pin(b)}+{w}.2")
    out += [f"unpinch {pz}", f"park {pz}", "stage to=0 tilt=0 at=0,0,0"]
    return out


def lap(part, lead, to, along, bend=True, pz="P1", at=3, frm=None, short=None, after=(), out_to="0,60,60"):
    out = [f"pick A {part} grasp=body", "lift A mm=60"]
    if bend:
        out += [f"insert A on={lead} at=12", "form angle=90", "lift A mm=30",
                f"shear A on={lead} at=17.2", "lift A mm=60"]
    f_ = f" from={frm}" if frm else ""
    out += [f"bring A {lead} to={to} along={along} lap=3.2{f_}", f"pinch {pz} on={lead} at={at}",
            f"release A {part}"]
    out += away(out_to)
    out += solder(f"{pin(to)}+{lead}")
    if short:
        lead2, at2 = short.split(" at=")
        out += [f"present {lead2}.{at2} at=S4", f"trim S4 on={short}", "back S4"]
    out += list(after)
    # back from a station first (down and in), then turned home: a turn
    # and a move at once can sweep a pincer through the station
    out += [f"unpinch {pz}", f"park {pz}", "stage tilt=0 at=0,0,0", "stage to=0"]
    return out


# the tile in the vise, its posts up: rows at x = -5.08 (posts 1 to 5:
# GND, LED1, C1N, B2, VCC) and x = +5.08 (6 to 10: GND, LED2, C2N, B1,
# VCC), y from -15.24 to +15.24, tips at z = -18.2
L += ["reset circuit=multivibrator", "tool A tweezers", "pick A P1 grasp=body",
      "place A P1 at=0,0,-24 rpy=0,0,90", "vise close", "release A P1", "lift A mm=60"]
# 1. VCC across the far end
if STAGE >= 1:
    L += jumper("P1.5.tip", "P1.10.tip", via="0,15.24,-16.5")
# 2. the resistors, each onto its signal post from outside its row (every
#    lap first), the VCC lead trimmed and jumped to the row's VCC post
if STAGE >= 2:
    L += lap("R1", "R1.2", "P1.2.tip", "-0.95,-0.3,0", pz="P2", short="R1.1 at=10")
    L += lap("R2", "R2.2", "P1.4.tip", "-0.95,0.3,0", pz="P2", short="R2.1 at=10")
    L += lap("R4", "R4.2", "P1.7.tip", "0.95,-0.3,0", pz="P2", short="R4.1 at=10")
    L += lap("R3", "R3.2", "P1.9.tip", "0.95,0.3,0", pz="P2", short="R3.1 at=10")
    L += jumper("R1.1.tip", "P1.5.tip", via="-24,4,-12", pz="P2")
    L += jumper("R2.1.tip", "P1.5.tip", pz="P2")
    L += jumper("R4.1.tip", "P1.10.tip", via="24,4,-12", pz="P1")
    L += jumper("R3.1.tip", "P1.10.tip", pz="P1")
L += ["inspect shorts", "check"]
print("\n".join(L))
