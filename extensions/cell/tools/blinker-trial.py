#!/usr/bin/env python3
"""The blinker built by hand rules, a trial for the planner (step 27): it
prints a command program for circuits/blinker.toml. What works here is
what CellPlan.xtl must learn: jumpers under the chip first, a lead of
each part straight to a pin, the LED onto R3, the rest by enameled
jumpers sized to their span. So far 5 of the 7 nets are made (GND and
the battery are not).
    .venv/bin/python3 extensions/cell/tools/blinker-trial.py > /tmp/b.cell"""
import sys
TIP = {1: (3.81, -3.81), 2: (3.81, -1.27), 3: (3.81, 1.27), 4: (3.81, 3.81),
       5: (-3.81, 3.81), 6: (-3.81, 1.27), 7: (-3.81, -1.27), 8: (-3.81, -3.81)}
def n(x): return ("%.3f" % x).rstrip("0").rstrip(".")
def out(pin, fan=0.3):
    x, y = TIP[pin]; dx = 1 if x > 0 else -1; dy = fan * y
    m = (dx * dx + dy * dy) ** 0.5
    return dx / m, dy / m
def side(pin): return "P1" if TIP[pin][0] > 0 else "P2"
L = ["reset circuit=blinker", "tool A tweezers", "pick A U1 grasp=ends",
     "place A U1 at=0,0,-24 rpy=0,0,90", "vise close", "release A U1"]
def solder(joint, pz, feed=2, temp=None, dwell=2):
    t = f" temp={temp}" if temp else ""
    return [f"present {joint} at=S1", f"touch S1 at={joint} dwell={dwell}{t}", f"feed S1 mm={feed} rate=1",
            "back S1", f"wait until={joint} below=100", f"inspect {joint}"]
def direct(part, lead, pin, kind, pinch_at=3, roll=0):
    ux, uy = out(pin)
    if kind == "axial":
        al = f"{n(ux)},{n(uy)},0"
    else:  # radial: diagonal, from above and outside
        al = f"{n(0.7 * ux)},{n(0.7 * uy)},0.7"
    pz = side(pin)
    j = f"U1.{pin}+{part}.{lead}"
    return [f"pick A {part} grasp=body", "lift A mm=60",
            f"bring A {part}.{lead} to=U1.{pin}.tip along={al}" + (f" roll={roll}" if roll else ""),
            f"pinch {pz} on={part}.{lead} at={pinch_at}", f"release A {part}", "lift A mm=60"] + \
        solder(j, pz) + [f"unpinch {pz}", f"park {pz}", "stage to=0 tilt=0 at=0,0,0"]
STEPS = sys.argv[1] if len(sys.argv) > 1 else "all"
# the LED's anode onto R3's free end, from above, beyond it
def onto(part, lead, target, along, pz, pinch_at=6):
    j = target.replace(".tip", "") + f"+{part}.{lead}"
    return [f"pick A {part} grasp=body", "lift A mm=60",
            f"bring A {part}.{lead} to={target} along={along}",
            f"pinch {pz} on={part}.{lead} at={pinch_at}", f"release A {part}", "lift A mm=60"] + \
        solder(j, pz) + [f"unpinch {pz}", f"park {pz}", "stage to=0 tilt=0 at=0,0,0"]
SIDE = {"U1.1": 1, "U1.2": 1, "U1.3": 1, "U1.4": 1, "U1.5": -1, "U1.6": -1, "U1.7": -1, "U1.8": -1,
        "R1": -1, "R2": -1, "R3": 1, "C1": 1, "C2": -1, "D1": 1, "BT1": 1}
W = [0]
def jumper(a, b, awg=30):
    W[0] += 1; w = f"W{W[0]}"
    pa = ".".join(a.split(".")[:2]); pb = ".".join(b.split(".")[:2])
    sx = SIDE.get(pa, SIDE.get(pa.split(".")[0], 1))
    pz = "P1" if sx > 0 else "P2"
    return [f"wire S5 awg={awg} span={a}+{b} enamel=yes", f"pick A {w} grasp=middle", "cut S5", "lift A mm=60",
            f"bring A {w}.1 to={a} end={b} from={sx},0,-0.15",
            f"pinch {pz} on={w}.1 at=1.5", f"release A {w}", "lift A mm=60"] + \
        solder(f"{pa}+{w}.1", pz, feed=1.5, temp=400, dwell=3) + solder(f"{pb}+{w}.2", pz, feed=1.5, temp=400, dwell=3) + \
        [f"unpinch {pz}", f"park {pz}", "stage to=0 tilt=0 at=0,0,0"]

# 1. the jumpers under the chip, while it is bare
L += jumper("U1.4.tip", "U1.8.tip")
# 1.2 mm up the legs: over the first, still under the body
L += jumper("U1.2.4", "U1.6.4")
# 2. a lead of each part straight to a pin
L += direct("R1", 1, 8, "axial")
L += direct("R2", 2, 6, "axial")
L += direct("R3", 1, 3, "axial")
L += direct("C1", 1, 2, "radial", pinch_at=6)
L += direct("C2", 1, 5, "radial", pinch_at=3)
# 3. the LED onto R3's free end
ux, uy = out(3)
L += onto("D1", 1, "R3.2.tip", f"{n(0.7*ux)},{n(0.7*uy)},0.7", "P1")
# 4. the rest by jumpers
for a, b in [("R1.2.tip", "U1.7.tip"), ("R2.1.tip", "U1.7.tip"), ("C1.2.tip", "U1.1.tip"),
             ("C2.2.tip", "U1.1.tip"), ("D1.2.tip", "U1.1.tip")]:
    L += jumper(a, b)
L += ["inspect shorts", "check"]
print("\n".join(L))
