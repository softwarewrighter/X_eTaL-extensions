#!/usr/bin/env python3
"""The 555 blinker by hand rules -- the trial the planner learned from
(programs/blinker.cell is now planned from nets/blinker.net by
CellPlan.xtl's search, cp:b_uild); kept as the record. It prints a
command program for circuits/blinker.toml. The chip in the vise legs up
(dead-bug); every joint a lap joint (decision C22): a lead's end
alongside a leg or lead, 3.2 mm. Enameled jumpers between legs first,
then one lead of each part lapped onto its pin's leg from outside, then
jumpers with lapped ends for the rest. What works here is what
CellPlan.xtl must learn.
    .venv/bin/python3 extensions/cell/tools/blinker-trial.py [STAGE] > /tmp/b.cell"""
import sys

STAGE = int(sys.argv[1]) if len(sys.argv) > 1 else 99
L = []
wires = [0]


def solder(joint):
    return [f"present {joint} at=S1", f"touch S1 at={joint} dwell=1.5 temp=400",
            "feed S1 mm=2 rate=1", "back S1", f"wait until={joint} below=100", f"inspect {joint}"]


def away(to="0,60,60"):
    # out from over the work, home through joint space
    return ["lift A mm=60", f"move A to={to}", "joints A 0 0 0 0 90 0"]


def pin(frame):
    return ".".join(frame.split(".")[:2])


def jumper(a, b, via=None, pz="P1", at=1.5, frm=None):
    wires[0] += 1
    w = f"W{wires[0]}"
    v = f" via={via}" if via else ""
    out = [f"wire S5 awg=30 span={a}+{b} lap=3.2{v} enamel=yes", f"pick A {w} grasp=middle",
           "cut S5", "lift A mm=60", f"lay A {w}" + (f" from={frm}" if frm else ""), f"pinch {pz} on={w}.1 at={at}", f"release A {w}"]
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
        # the other lead trimmed short at the cutter station, soldered on
        lead2, at2 = short.split(" at=")
        out += [f"present {lead2}.{at2} at=S4", f"trim S4 on={short}"]
    out += list(after)
    out += [f"unpinch {pz}", f"park {pz}", "stage to=0 tilt=0 at=0,0,0"]
    return out


L += ["reset circuit=blinker", "tool A tweezers", "pick A U1 grasp=ends",
      "place A U1 at=0,0,-24 rpy=180,0,90", "vise close", "release A U1", "lift A mm=60"]
# 1. jumpers between legs, while the legs are bare: VCC 4-8, then THR
#    2-6 over it
if STAGE >= 1:
    L += jumper("U1.4.tip", "U1.8.tip")
    L += jumper("U1.2.tip", "U1.6.tip", via="0,0,-17.3")
# 2. a lead of each part onto its pin's leg from outside
if STAGE >= 2:
    L += lap("R1", "R1.1", "U1.8.tip", "0.934,-0.356,0", pz="P1", short="R1.2 at=10")
    L += lap("R2", "R2.2", "U1.6.tip", "0.934,0.356,0", pz="P1", short="R2.1 at=10")
    L += lap("R3", "R3.1", "U1.3.tip", "-0.934,0.356,0", pz="P2", short="R3.2 at=10",
             after=["bend P1 on=R3.2 at=6 angle=90 toward=0,0,1"])
    L += lap("C2", "C2.1", "U1.5.tip", "0,1,0", bend=False, pz="P1", frm="0,0,1")
    # GND from C2 over the chip to pin 1, before C1 stands over pin 2
    L += jumper("C2.2.tip", "U1.1.tip", via="0,0,-15", pz="P1")
    L += lap("C1", "C1.1", "U1.2.tip", "-1,0,0", bend=False, pz="P2", frm="0,0,1", out_to="-40,80,80")
# 3. DIS by jumpers from the free ends; the LED onto R3's upturned end
if STAGE >= 3:
    L += jumper("R1.2.tip", "U1.7.tip", pz="P1")
    L += jumper("R2.1.tip", "U1.7.tip", pz="P1")
    L += lap("D1", "D1.1", "R3.2.tip", "0.356,0.934,0", bend=False, pz="P2", frm="-1,0,1")
# 4. GND: C1.2 to pin 1, the LED's cathode to C1.2 higher up it
if STAGE >= 4:
    L += jumper("C1.2.tip", "U1.1.tip", via="-8,-6,-19", pz="P2", frm="-0.5,-1,0.8")
    L += jumper("D1.2.tip", "C1.2.10", via="-17,6,-13", pz="P2", frm="0,1,0.5")
# 5. the battery snap (its leads kitted cut to 9 mm) set flat on the
#    turntable beside the vise, its leads toward the chip; a jumper from
#    each lead up past the jaw to pin 8 (VCC) and pin 1 (GND)
if STAGE >= 5:
    L += ["pick A BT1 grasp=body", "lift A mm=60", "move A to=0,-38,20",
          "place A BT1 at=0,-38,-56.4 rpy=0,0,180", "release A BT1"]
    L += away()
    L += jumper("BT1.1.tip", "U1.8.tip", via="4,-22,-28", pz="P1")
    L += jumper("BT1.2.tip", "U1.1.tip", via="-1,-22,-28", pz="P2")
L += ["inspect shorts", "check"]
print("\n".join(L))
