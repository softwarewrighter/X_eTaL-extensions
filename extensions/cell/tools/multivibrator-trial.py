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
L += ["reset circuit=multivibrator", "tool A tweezers", "pick A T1 grasp=body",
      "place A T1 at=0,0,-24 rpy=0,0,90", "vise close", "release A T1", "lift A mm=60"]
# 1. VCC across the far end
if STAGE >= 1:
    L += jumper("T1.5.tip", "T1.10.tip", via="0,15.24,-16.5")
# 2. the resistors, each onto its signal post from outside its row (every
#    lap first), the VCC lead trimmed and jumped to the row's VCC post
if STAGE >= 2:
    L += lap("R1", "R1.2", "T1.2.tip", "-0.95,-0.3,0", pz="P2", short="R1.1 at=10")
    L += lap("R2", "R2.2", "T1.4.tip", "-0.95,0.3,0", pz="P2", short="R2.1 at=10")
    L += lap("R4", "R4.2", "T1.7.tip", "0.95,-0.3,0", pz="P2", short="R4.1 at=10")
    L += lap("R3", "R3.2", "T1.9.tip", "0.95,0.3,0", pz="P2", short="R3.1 at=10")
    L += jumper("R1.1.tip", "T1.5.tip", via="-24,4,-12", pz="P2")
    L += jumper("R2.1.tip", "T1.5.tip", pz="P2")
    L += jumper("R4.1.tip", "T1.10.tip", via="24,4,-12", pz="P1")
    L += jumper("R3.1.tip", "T1.10.tip", pz="P1")
# 3. the transistors, the collector onto its post from outside its row;
#    the emitter bent toward the ground end and jumped to that row's ground
#    post, the base bent toward the far end and jumped across the rows (at
#    two heights: the crossings clear each other)
if STAGE >= 3:
    L += lap("Q1", "Q1.3", "T1.3.tip", "-1,0,0", bend=False, pz="P2", frm="0,0,1",
             after=["bend P1 on=Q1.1 at=8 angle=90 toward=-0.6,-1,0",
                    "bend P1 on=Q1.2 at=6 angle=90 toward=-0.6,1,0"])
    L += jumper("Q1.1.tip", "T1.1.tip", via="-9,-8,-12", pz="P2")
    L += jumper("Q1.2.tip", "T1.9.tip", via="0,6,-12", pz="P2")
    L += lap("Q2", "Q2.3", "T1.8.tip", "1,0,0", bend=False, pz="P1", frm="0,0,1",
             after=["bend P2 on=Q2.1 at=8 angle=90 toward=0.6,-1,0",
                    "bend P2 on=Q2.2 at=6 angle=90 toward=0.6,1,0"])
    L += jumper("Q2.1.tip", "T1.6.tip", via="9,-8,-12", pz="P1")
    L += jumper("Q2.2.tip", "T1.4.tip", via="0,9,-8", pz="P1")
# 4. the capacitors between neighboring posts, on the one away from the
#    transistor (room): lapped onto it on the side facing the other, the
#    other lead bent toward the transistor's post and jumped to it
if STAGE >= 4:
    L += lap("C1", "C1.2", "T1.4.tip", "0,-1,0", bend=False, pz="P2", frm="0,0,1", out_to="-40,80,80",
             after=["bend P1 on=C1.1 at=8 angle=90 toward=0,-1,0"])
    L += jumper("C1.1.tip", "T1.3.tip", pz="P2")
    L += lap("C2", "C2.2", "T1.9.tip", "0,-1,0", bend=False, pz="P2", frm="0,0,1", out_to="-40,80,80",
             after=["bend P1 on=C2.1 at=8 angle=90 toward=0,-1,0"])
    L += jumper("C2.1.tip", "T1.8.tip", pz="P1")
# 5. the LEDs: the anode onto its post on the side facing the collector's
#    post. The cathode (to the collector's net) is not joined yet: under
#    the LED's tall body nothing reaches it from above or the side, and
#    the rig cannot bend an LED over (still to do: two pincer bends, out
#    and down, laying the cathode's end alongside the collector's post)
if STAGE >= 5:
    L += lap("D1", "D1.1", "T1.2.tip", "0,1,0", bend=False, pz="P2", frm="0,0,1", out_to="-40,80,80")
    L += lap("D2", "D2.1", "T1.7.tip", "0,1,0", bend=False, pz="P1", frm="0,0,1", out_to="-40,80,80")
# 6. the battery snap (its leads kitted cut to 9 mm) set flat on the
#    turntable at the VCC end, its leads toward the tile; a jumper from each
#    up past the jaw to its post
if STAGE >= 6:
    L += ["pick A BT1 grasp=body", "lift A mm=60", "move A to=0,38,20",
          "place A BT1 at=0,38,-56.4 rpy=0,0,0", "release A BT1"]
    L += away()
    L += jumper("BT1.1.tip", "T1.5.tip", via="-4,22,-28", pz="P2")
    L += jumper("BT1.2.tip", "T1.6.tip", via="24,0,-24", pz="P1")
L += ["inspect shorts", "check"]
print("\n".join(L))
