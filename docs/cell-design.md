# The assembly cell -- design

A programmable robot cell that builds electronic circuits in free air,
in 3D, from the center outward: articulated arms hold two parts so
that a lead touches a lead, a third arm solders (or spot welds) them,
and the next part is brought in. It is virtual first: a simulation
that respects the physics that matter (reach, collision, gravity,
heat, solder, wire), driven one command at a time by a program or an
LLM agent, and drawn as it goes -- or previewed as a whole, the way a
slicer previews a 3D printer's moves before the printer runs them.

Status: being built (docs/plan.md, Saga 17; decisions accepted
2026-11-24). Built so far: the extension `cell` with the parts
library, the command table, text and JSON parsing, the state as JSON,
the arms, stage and vise, the models, the player with the preview,
grasping with collision, and the live page
(<https://softwarewrighter.github.io/X_eTaL-extensions/cell/live/>,
extensions/cell/docs/README.md); the rest below is the design,
its names proposals.

## What is built where

| Piece | Where | Made with | Does |
| ----- | ----- | --------- | ---- |
| The simulator | `extensions/cell/rust` (a library, the extension `cell`) | Rust; `parry3d` for collision | the one authority on the cell's state: runs a command, says what happened, keeps the timeline. Pure, deterministic, no clock of its own |
| The facade | `extensions/cell/lib/Cell.xtl`, alias `ce:` | X_eTaL | a program sends commands, reads state, timelines and checks |
| The planner | `extensions/cell/lib/CellPlan.xtl`, alias `cp:` | X_eTaL | a netlist to a placement in 3D to a command program: the slicer |
| The models | `extensions/cell/lib/CellModels.xtl`, alias `cm:` | X_eTaL writing bpy through Bpy.xtl; Blender | arms, tools, trays, stage, microscope, components as `.glb`, each with a rig file |
| The player | `extensions/three/web/static/cell.js` | Three.js, beside the city's and the board's players | draws a cell scene, plays a timeline with a scrubber, exposes `window.cell` |
| The agent's door | `extensions/cell/mcp` (a binary, `xetal-cell-mcp`) | Rust, `rmcp` (the MCP Rust SDK) | MCP tools over the simulator; serves the page on port 8470 and pushes each result to it |
| The live page | `pages/cell/live` | the simulator as WebAssembly, the player, X_eTaL in the page | the cell driven in a browser, nothing installed |
| The render bundle | `just ext cell bundle NAME` | Cycles.xtl's player, Blender | a directory and zip a render node runs with only Blender |

The split: X_eTaL decides what (the plan, the shapes, the scene); Rust
decides what is possible (the physics); Three.js and Blender only
draw. The same scene text drives both players, as the neon city does.

## The cell

Units are millimeters, degrees, seconds, grams, watts and degrees
Celsius. The simulator is Z up, right handed; glTF is Y up and the
player converts, as console-3D's rig does.

- **Work zone**: a sphere of 60 mm radius around the origin, above a
  rotary stage (one axis, Z) that can also hold a part or a
  subassembly in a vise. The circuit grows outward from the origin.
- **Four arms** on a ring of 450 mm radius at 0, 90, 180 and 270
  degrees, each a generic six-axis arm (base yaw, shoulder, elbow,
  three-axis spherical wrist) with a quick-change flange. Named A, B,
  C, D. Any arm can hold any tool; the first circuit uses A and B to
  hold, C to solder and D to feed solder and clip heatsinks.
- **Tool racks** (one beside each arm, decision C14, each with every
  tool): tweezer
  gripper (two fingers, force limited), soldering iron (tip size,
  set point), solder feeder (wire diameter, feed rate), heatsink clip
  (a copper jaw on a spring, a thermal mass), spot-weld electrodes
  (capacitor discharge), flush cutter (trims excess leads and cuts wire),
  wire feeder (a spool, a nozzle, a cutter: bare tinned or enameled
  wire), and a bender (round-nose jaws on mandrels: forms leads and
  wires, decision C15).
- **Trays**: pocketed trays of parts, each pocket a part in a known
  pose (leads down, body up); a spool rack of wire gauges.
- **Microscope**: a USB microscope on a pan-tilt-zoom head on a gantry,
  optionally stereo (two lenses with a baseline). Magnification sets
  the field of view and a shallow depth of field.

No arm or tool imitates a commercial product; the shapes are generic.

## Parts

A parts library, `extensions/cell/parts/*.toml`, one file a part kind:
the body (a box, cylinder or lathe profile), the pins (each a named
frame and a lead: a polyline from the body, diameter, material),
grasp frames (where tweezers may hold), and the thermal data below.
Through-hole first:

| Kind | Example | Pins | Notes |
| ---- | ------- | ---- | ----- |
| DIP-8, -14, -16 | NE555 | 8 to 16 | held dead-bug (legs up) as the core of a circuit |
| TO-92 | 2N3904 | 3 | heat sensitive: clip a heatsink between body and joint |
| axial resistor | 10 kilohm, 1/4 W | 2 | color bands drawn from the value |
| axial diode | 1N4148 | 2 | glass body |
| radial electrolytic | 10 uF | 2 | heat sensitive sleeve, polarity |
| ceramic disc | 100 nF | 2 | |
| 5 mm LED | red | 2 | polarity, lens |
| battery leads | 9 V snap | 2 | the circuit's power, held in the vise |

Later: SMD (0805, 0603, SOT-23, SOIC-8) joined by fine wire, and small
PCB tiles as parts (a ground plane, a high-current bus, a prebuilt
subassembly) with pads as pins.

Every pin has a name, `R1.1`, `U1.3`; every pin's tip is a frame a
command can name, `R1.1.tip`. A joint is named by its pins, `U1.3+R1.1`.

## The command language

One command a line, as text, and the same as JSON (`{"op": "move",
"arm": "A", ...}`) for MCP. The table of commands, their arguments,
units and limits is data in the simulator; the parser, the MCP tool
schemas and the docs are made from it, so a new command appears
everywhere at once.

```
reset circuit=blinker                      # trays filled, arms home
tool A tweezers                            # A changes tools at the rack
pick A U1 grasp=ends                       # approach, close, lift: one move each, checked
place A U1 at=0,0,0 rpy=180,0,0            # the DIP, dead-bug, at the origin
pick B R1 grasp=body
align B R1.1.tip to=U1.3.tip gap=0.1       # lead to lead, 0.1 mm apart
tool D clip
clip D on=U1.3 at=2                        # heatsink 2 mm from the body
tool C iron temp=350
touch C at=U1.3+R1.1 dwell=1.5             # tip on the joint
feed D mm=1.2 rate=2                       # solder into the joint
lift C
wait until=U1.3+R1.1 below=150
release B R1                               # B lets go: R1 now hangs on its joint
scope zoom=40 aim=U1.3+R1.1 ; snap
inspect U1.3+R1.1                          # gap, fillet, quality, temperatures
```

Lower-level commands always exist beside the conveniences: `joints A
j1..j6` (absolute joint angles), `move A to=x,y,z rpy=r,p,y speed=`
(a tool pose by inverse kinematics), `grip A close force=0.5`,
`stage turn=90`, `vise close`. `pick`, `place`, `align` and `touch`
are each a short list of these, expanded by the simulator and shown
as such in the timeline, so the agent and the user see every motion.

Every command returns a result:

```json
{"ok": false, "t0": 41.20, "t1": 41.85,
 "did": ["move A ...", "move A ..."],
 "stopped": "collision: arm B link 5 with R1 body at 41.85 s",
 "events": [], "warnings": ["U1.3 at 196 C, limit 260 C"]}
```

- A command that cannot start (out of reach, a joint limit, a tool not
  held) is refused and changes nothing.
- A command that starts and meets a collision stops there, as a real
  arm's protective stop does, and says where. `dry=1` on any command
  runs it on a copy and returns what would happen.
- `rewind t=` puts the cell back as it was (a simulation's privilege;
  the timeline keeps both branches' commands, marked).

## The physics

Semi-authentic: each model is the simplest one that makes the wrong
plan fail for the right reason. Every constant is in a TOML file with
its source or marked illustrative.

**Motion.** Arms are position controlled, as industrial arms are:
kinematic, not dynamic. Forward kinematics from DH parameters; inverse
kinematics in closed form for the spherical wrist, choosing the
solution nearest the current pose; joint limits; per-joint speed and
acceleration limits; trapezoidal profiles, all joints arriving
together; Cartesian moves sampled to stay straight. Payload is grams,
far under any limit, so it changes nothing.

**Collision.** Links are capsules, bodies boxes or cylinders, leads
and wires thin capsules (`parry3d`). Checked along every motion at a
step fine enough for the fastest point to move under 0.2 mm. Allowed
contacts: tweezers on their grasp frames, iron tip on a joint, a lead
on the lead it is being aligned to (that is the point).

**Grasp and gravity.** A part in closed tweezers moves with them; a
part joined to the assembly moves with the assembly; a part released
with neither falls (straight down, to the floor; no tumbling) and is
lost, a warning. The assembly is a tree of rigid parts joined at
joints; held by at least one arm or the vise, or it falls too.

**Sag.** A part hanging from leads or fine wire deflects. Each
unsupported span is a cantilever, deflection F L^3 / (3 E I) with E of
copper or steel and I of the wire's round section; a span that sags
past the joint tolerance (0.1 mm while soldering, 1 mm when done) is a
warning that names it: use a thicker wire, a shorter span, or support.

**Heat.** A thermal network: nodes for each component body, each lead
in segments of about 1 mm, each joint's solder, each wire segment, the
iron tip, each heatsink clip. Conduction between neighbors (k A / L),
convection to air from every surface (h A, h higher when the part
moves, about 10 W/m^2K still to 25 moving), contact conductance where
the iron or a clip touches. Explicit time steps of 1 ms or less,
stable by construction. The iron tip is held at its set point by a
heater of limited power, so a large joint pulls it down.

**Solder.** Alloys as data: SAC305 (lead-free, melts 217 to 220 C,
iron 330 to 370 C) by default, Sn63Pb37 (183 C) as the option. Solder
fed onto a joint melts only where the joint is above the liquidus;
latent heat is taken from the joint. A joint wets when both surfaces
are above the liquidus long enough (about 1 s). On cooling a joint is:

| Result | When |
| ------ | ---- |
| good | wetted both sides, enough solder, still while solidifying |
| cold | did not wet one side |
| disturbed | a held part moved while the joint was between solidus and liquidus |
| starved, blob | too little or too much solder for the gap |
| bridged | solder reached another pin's lead |

A good joint is a rigid connection and an electrical one.

**Damage.** Each part kind has a body limit (about 260 C for 10 s for
most, lower for electrolytic sleeves and LED lenses) and a sensitivity
flag; exceeding it marks the part damaged, and the netlist check fails
on it. Heat reaching a neighbor joint above its solidus melts it again
-- the problem heatsink clips solve.

**Spot welding.** A capacitor discharge (E = C V^2 / 2) through two
electrodes either side of a lap of two leads: a weld when the energy
is in the material's window (nickel or steel leads weld well, copper
poorly), a blow-out above it. The current's path is traced through the
assembly: a semiconductor in a path from electrode to electrode, or
with no grounding clip on its lead, marks a transient risk -- refused
by default (`weld ... risky=allow` to proceed and see the damage).

**Wire.** A spool of a gauge: diameter, resistance a meter, ampacity,
stiffness, bare or enameled (which must be stripped by heat before it
wets). The feeder pushes wire out of its nozzle; a gripper holds the
free end; the wire bends where the nozzle moves relative to the held
end (a polyline, with a minimum bend radius); the cutter cuts. A wire
is then a part with two ends, each a pin.

| AWG | Diameter (mm) | Use here |
| --- | ------------- | -------- |
| 22 | 0.64 | through-hole jumpers, the power rail |
| 26 | 0.40 | through-hole signals |
| 30 | 0.25 | SMD to SMD |
| 34 | 0.16 | 0402 and fine pitch, later |

**Electricity.** No circuit simulation. `probe` measures continuity
and resistance between two pins over good joints and wire. The finished
assembly's connection graph is compared with the netlist: the test of
a build is that they are the same, with no part damaged.

## Seeing: the microscope

`scope pan= tilt= zoom= focus= rotate=` moves the microscope; `snap`
returns a picture, `snap stereo=1` two (or one red-cyan). The field of
view comes from the magnification (at 40x about 6 mm across) and the
depth of field is shallow, so a joint out of the focal plane is blurred:
seeing the far side of an assembly means turning it (`stage turn=`) or
moving the scope. The player draws the scope's view as a second picture
beside the cell's.

`inspect` reports a joint's gap, fillet and result, and a pin's or
joint's temperature. Two modes:

- **oracle** (the default): the simulator's own numbers.
- **sensed**: positions with noise of about one pixel at the current
  magnification, temperatures only through a thermal camera's spot,
  results only as a picture shows them. The agent must look, as a real
  cell's controller must. This is the mode a physical cell would have.

## Reading a part, and testing it

A part is checked before it goes in, as at a bench (decisions C12,
C13 in docs/plan.md):

- **Markings.** Every part instance carries its markings as data and
  its model draws them: a resistor's color bands, a ceramic
  capacitor's code (104), an electrolytic's value and voltage, an IC's
  part number and pin-1 notch, a transistor's type, an LED's color.
  `identify PART` turns the part (or moves the microscope) until the
  marked side faces the lens and reads it. In oracle mode the answer is
  the data; in sensed mode it is decoded from the scope's picture --
  band colors sampled, printed text matched against the cell's own
  glyphs -- with a confidence, and it can fail (the far side, glare,
  blur), so the agent learns to turn the part.
- **A tester with probes.** A probe tool -- a sprung pin on a lead --
  in each of two arms, while a third holds the part in tweezers in
  free air: no socket. `test PART` touches the probes to lead pairs in
  turn and measures as a universal component tester does: resistance,
  capacitance and ESR, inductance, a diode's forward drop and polarity
  (or open, short), an LED's drop and color, a transistor's type, which
  lead is base, collector and emitter, and its gain. Each part
  instance has an electrical model as data, faults included (open,
  short, out of tolerance, marked wrong). `verify PART` compares the
  measurement with what the netlist wants and what the marking says.

## The timeline and the preview

The simulator records every motion, contact, temperature and event
on one timeline. A scene for the player is that timeline written as
calls (the players' text: one call a line, numbers as tables):

```
cell('blinker', 1280, 720)
rig('A', 'arm', nums('0 450 0 0 0 -90', 6)[0])
part('U1', 'dip8', nums('0 0 0 180 0 0', 6)[0])
track('A', nums('t j1 j2 j3 j4 j5 j6 ...', 7))
attach('U1', 'A', 3.20, 18.75)
heat('U1.3', nums('t celsius ...', 2))
joint('U1.3+R1.1', nums('x y z', 3)[0], 'good', 22.40)
wire('W1', 26, nums('x y z ...', 3), 31.0)
step(12, 'solder U1.3 to R1.1', 20.10, 24.80)
scope(nums('t pan tilt zoom ...', 4))
```

The page: the cell, the scope's picture, a heat color ramp on parts
and leads, the list of steps (click one to go there), a scrubber and
a speed, ghosts of the finished circuit in place before it is built,
collisions and warnings marked where they happened. As console-3D's
Selectric, the player has an API: `await cell.play(from, to)`,
`cell.seek(t)`, `cell.state`, `cell.stop()`.

Two ways to fill it:

- **Preview**: a whole program run by the simulator first, then
  played: `just ext cell preview blinker`. This is the slicer.
- **Live**: the MCP server runs commands as they come and pushes each
  result's piece of timeline to the open page, which plays it.

## Driving it in a browser

Interactive first: the live page runs the simulator, compiled to
WebAssembly, beside the player, with nothing installed. The core has
no threads, files or clocks (time is the simulation's own), so the
same crate is the extension, the MCP server's engine and the page's.
The page has a control for every arm and tool, a command box that
takes the command language above, each result or refusal as it
comes, dry runs, rewind and the scrubber. It is published at
`pages/cell/live`, the site's second live page beside the city.

Videos are recordings of the player from headless Chrome, not
ray-traced renders.

## A render bundle

When a ray-traced film is wanted, `just ext cell bundle NAME` writes
a scene as a job for a Blender render node, a directory and its zip:

```
NAME/
  README.txt     what it is and how to run it
  job.toml       frames, fps, size, samples, engine, the Blender it
                 was made for, a sha256 a file
  scene.py       Cycles.xtl's player and the timeline's calls
  models/*.glb   the cell's and the parts' models
  render.sh      render.sh FIRST LAST [DEVICE]: frames to frames/,
                 skipping those already there; a log
  encode.sh      frames/ to NAME.mp4 (ffmpeg)
```

The node needs Blender, and ffmpeg to encode; not this repository,
Rust or X_eTaL. Several nodes take a range each; the farm
(docs/farm.md) gets a job kind `bundle` that divides a bundle's frames
over its machines as it divides a film's.

## The asset contract

`CellModels.xtl` describes every model as X_eTaL arrays and Bpy.xtl
writes it; Blender exports `NAME.glb` and `NAME.rig.json`. An arm's
hierarchy, one group a joint's pivot (as console-3D's carrier, tilt
and spin):

```
arm_base
`- arm_j1            yaw about local Z
   `- arm_j2         shoulder
      `- arm_j3      elbow
         `- arm_j4   wrist roll
            `- arm_j5  wrist pitch
               `- arm_j6  flange roll
                  `- arm_flange   tools attach here
```

`NAME.rig.json` gives the node names, axes, zero poses, limits and the
DH parameters, and is the same file the simulator reads: one source of
truth for the shape and the kinematics. A check reads the `.glb` back
and compares its pivots with the rig. Parts are models too: each part
kind's `.glb` has a node a pin, named as the pin.

## MCP

`xetal-cell-mcp` speaks MCP on stdio and serves the page on port 8470.

| Tool | Does |
| ---- | ---- |
| `cell_reset` | a circuit's trays and netlist, arms home |
| `cell_do` | one command (text or JSON), its result |
| `cell_run` | a list of commands, stopping at the first failure |
| `cell_state` | arms, tools, parts, joints, temperatures (oracle mode) |
| `cell_snap` | the microscope's picture, as an image |
| `cell_inspect`, `cell_probe` | as the commands |
| `cell_rewind` | back to a time |
| `cell_plan` | run the X_eTaL planner on a netlist; its program, not run |
| `cell_check` | the assembly against the netlist |

Each command family also has its own tool with a typed schema
(`arm_move`, `iron_touch`, ...), made from the command table, for
agents that do better with typed tools than with text. A session's
commands are saved as a program, so an agent's build can be previewed
again, edited and rerun.

`cell_snap` needs a picture: the open page draws the scope's view
offscreen and returns a PNG; with no page open the server starts a
headless Chrome on it (as the recordings do) -- never a window on the
user's screen.

## A physical cell, later

Nothing here commits to hardware, but nothing should block it: the
command language says what, in the cell's frame, not how a motor
moves; sensed mode gives the controller only what cameras give; the
rig files hold the kinematics a real arm's controller would be given.
