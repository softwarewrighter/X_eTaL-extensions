# The cell player for Blender (decision C11): a scene the simulator wrote
# (its calls, the same the three extension's cell.js draws: cell, fixture,
# rig, track, part, attach, place, joint, lead, station, pincer, ...)
# shown with the cell's models (NAME.glb), each frame posed as the page
# poses it -- the arm's joints from its track, the turntable from its
# own, each part where it starts, carried while held (by the arm's
# flange or the turntable's top), put where a fall ended; a re-formed
# lead a tube from its time on, the model's lead hidden then; a joint's
# solder as it grows. It follows Cycles.xtl's player (scene, frame(),
# still, frames, encode) in one script; the calls run in a namespace of
# their own (`rig` is the player's). Read by a render node with Blender
# alone.
#
# cell_load(calls_file, models_folder) builds the scene; a frame-change
# handler poses it at each frame's time.
import math
import os
import re

import bpy
from mathutils import Euler, Matrix, Quaternion, Vector

CELL = {}


def _cell_calls():
    """The scene's calls, recorded."""
    s = {
        "fixtures": [], "rigs": [], "tracks": {}, "parts": [], "attaches": [],
        "places": [], "joints": [], "stations": [], "pincers": [], "leads": [],
    }

    def nums(text, k):
        v = [float(t) for t in text.split()]
        return [v[i:i + k] for i in range(0, len(v), k)]

    def ignore(*_a, **_k):
        return None

    space = {
        "nums": nums, "Infinity": float("inf"), "null": None, "true": True, "false": False,
        "cell": ignore,
        "fixture": lambda i, m, p: s["fixtures"].append((i, m, p)),
        "rig": lambda i, m, b: s["rigs"].append((i, m, b)),
        "track": lambda i, rows: s["tracks"].__setitem__(i, rows),
        "part": lambda i, k, p: s["parts"].append((i, k, p)),
        "attach": lambda i, a, t0, t1: s["attaches"].append((i, a, t0, t1)),
        "place": lambda i, t, p: s["places"].append((i, t, p)),
        "joint": lambda n, p, xyz, rows, r, t, lap=None: s["joints"].append((n, p, xyz, rows, r, t, lap)),
        "station": lambda i, tool, h, rows: s["stations"].append((i, tool, h, rows)),
        "pincer": lambda i, tool, rows: s["pincers"].append((i, tool, rows)),
        "lead": lambda i, pin, t, d, rows: s["leads"].append((i, pin, t, d, rows)),
    }
    for name in ("expect", "step", "feeding", "heat", "mark", "scope", "feeder", "bender",
                 "bender_shape", "ghost", "snap", "sleeve"):
        space[name] = ignore
    return s, space


def _at(rows, t):
    """A track's row at time t, linear between rows (without the time)."""
    if not rows:
        return []
    if t <= rows[0][0]:
        return list(rows[0][1:])
    for a, b in zip(rows, rows[1:]):
        if t <= b[0]:
            f = 0.0 if b[0] == a[0] else (t - a[0]) / (b[0] - a[0])
            return [x + (y - x) * f for x, y in zip(a[1:], b[1:])]
    return list(rows[-1][1:])


def _pose(p):
    """x y z roll pitch yaw (mm, degrees) as a matrix."""
    r = Euler((math.radians(p[3]), math.radians(p[4]), math.radians(p[5])), "XYZ")
    return Matrix.Translation((p[0], p[1], p[2])) @ r.to_matrix().to_4x4()


def _material(name, rgb, metal=0.0, rough=0.5, alpha=1.0):
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    b = m.node_tree.nodes.get("Principled BSDF")
    b.inputs["Base Color"].default_value = (*rgb, 1.0)
    b.inputs["Metallic"].default_value = metal
    b.inputs["Roughness"].default_value = rough
    if alpha < 1.0:
        b.inputs["Alpha"].default_value = alpha
    return m


def _holder(name):
    o = bpy.data.objects.new(name, None)
    bpy.context.scene.collection.objects.link(o)
    return o


def _load(folder, model, name):
    """A model (folder/model.glb) under a new empty: (the empty, its objects)."""
    path = os.path.join(folder, model + ".glb")
    if not os.path.exists(path):
        return None, []
    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=path)
    new = [o for o in bpy.data.objects if o not in before]
    # the models are in meters (cell_models.py): a thousand to the mm;
    # the importer has already turned glTF's y-up to Blender's z-up
    h = _holder(name)
    inner = _holder(name + " (mm)")
    inner.parent = h
    inner.scale = (1000.0, 1000.0, 1000.0)
    for o in new:
        if o.parent is None:
            o.parent = inner
    return h, new


def _node(objs, name):
    for o in objs:
        if o.name.split(".")[0] == name:
            return o
    return None


def _world(o):
    """An object's matrix in the scene, from its parents' own (no update)."""
    m = o.matrix_basis.copy()
    while o.parent is not None:
        m = o.parent.matrix_basis @ o.matrix_parent_inverse @ m
        o = o.parent
    return m


def _turn(node, rest, deg):
    # the models' joints turn about their own y (glTF's); imported to
    # Blender's z-up, that is the node's own z
    node.rotation_mode = "QUATERNION"
    node.rotation_quaternion = rest @ Quaternion((0.0, 0.0, 1.0), math.radians(deg))


def _tube(name, pts, d, mat, parent):
    c = bpy.data.curves.new(name, "CURVE")
    c.dimensions = "3D"
    c.bevel_depth = max(d, 0.1) / 2.0
    c.bevel_resolution = 2
    sp = c.splines.new("POLY")
    sp.points.add(len(pts) - 1)
    for k, q in enumerate(pts):
        sp.points[k].co = (q[0], q[1], q[2], 1.0)
    o = bpy.data.objects.new(name, c)
    o.data.materials.append(mat)
    bpy.context.scene.collection.objects.link(o)
    o.parent = parent
    return o


def cell_load(calls_file, folder):
    """The scene's calls read and its objects made."""
    s, space = _cell_calls()
    with open(calls_file, encoding="utf-8") as f:
        exec(compile(f.read(), calls_file, "exec"), space)
    CELL.clear()
    CELL["s"] = s
    copper = _material("lead", (0.72, 0.45, 0.2), metal=0.9, rough=0.3)
    solder = _material("solder", (0.85, 0.87, 0.9), metal=0.8, rough=0.25)
    steel = _material("housing", (0.29, 0.32, 0.38), rough=0.7)
    # the fixtures: the stage's turning top, the rest still
    fixtures = {}
    for fid, model, pose in s["fixtures"]:
        h, objs = _load(folder, model, fid)
        if h is None:
            continue
        h.matrix_basis = _pose(pose)
        turn = _node(objs, "stage_turn")
        rest = turn.rotation_quaternion.copy() if turn else None
        if turn:
            turn.rotation_mode = "QUATERNION"
        fixtures[fid] = {"h": h, "base": _pose(pose), "turn": turn, "rest": rest}
    # the arms: their joints and flange
    arms = {}
    for aid, model, base in s["rigs"]:
        h, objs = _load(folder, model, aid)
        h.matrix_basis = _pose(base)
        joints = [_node(objs, "arm_j%d" % n) for n in range(1, 7)]
        for j in joints:
            j.rotation_mode = "QUATERNION"
        arms[aid] = {"h": h, "joints": joints, "rest": [j.rotation_quaternion.copy() for j in joints],
                     "flange": _node(objs, "arm_flange")}
    # the parts and tools: their models (a wire has none: its leads only)
    parts = {}
    for pid, kind, pose in s["parts"]:
        h, objs = _load(folder, kind, pid)
        if h is None:
            h, objs = _holder(pid), []
        meshes = {}
        for o in objs:
            m = re.search(r"_lead([^_]+)_\d+$", o.name.split(".")[0])
            if m and o.type == "MESH":
                meshes.setdefault(m.group(1), []).append(o)
        parts[pid] = {"h": h, "start": _pose(pose), "meshes": meshes, "forms": {}}
    # re-formed leads: a tube a form, in the part's frame
    for n, (pid, pin, t, d, rows) in enumerate(s["leads"]):
        p = parts.get(pid)
        if p is None or len(rows) < 2:
            continue
        o = _tube("%s_%s_%d" % (pid, pin, n), rows, d, copper, p["h"])
        p["forms"].setdefault(pin, []).append((t, o))
    # joints: the solder on its part, a ball or a lap's fillet
    joints = []
    for name, pid, xyz, rows, result, t, lap in s["joints"]:
        p = parts.get(pid)
        if p is None or not rows:
            continue
        bpy.ops.mesh.primitive_uv_sphere_add(radius=1.0, segments=16, ring_count=8)
        o = bpy.context.active_object
        o.name = "joint " + name
        o.data.materials.append(solder)
        o.parent = p["h"]
        if lap:
            o.location = Vector(lap[0:3])
            o.rotation_mode = "QUATERNION"
            o.rotation_quaternion = Vector((0, 0, 1)).rotation_difference(Vector(lap[3:6]).normalized())
        else:
            o.location = Vector(xyz)
        joints.append({"o": o, "rows": rows, "lap": lap})
    # stations and pincers: a housing, the tool at its mount
    movers = []
    for sid, tool, housing, rows in s["stations"]:
        h, _ = _load(folder, tool, sid + " tool")
        bpy.ops.mesh.primitive_cube_add(size=1.0)
        box = bpy.context.active_object
        box.name = sid + " housing"
        box.scale = (40, 40, 60)
        box.matrix_basis = _pose(housing) @ Matrix.Diagonal((40, 40, 60, 1))
        box.data.materials.append(steel)
        if h is not None:
            movers.append({"tool": h, "rows": rows, "housing": None})
    for pid, tool, rows in s["pincers"]:
        h, _ = _load(folder, tool, pid + " tool")
        bpy.ops.mesh.primitive_cube_add(size=1.0)
        box = bpy.context.active_object
        box.name = pid + " housing"
        box.data.materials.append(steel)
        movers.append({"tool": h, "rows": [[r[0]] + r[7:13] for r in rows],
                       "housing": box, "hrows": [r[0:7] for r in rows]})
    CELL.update(fixtures=fixtures, arms=arms, parts=parts, joints=joints, movers=movers)
    cell_frame(bpy.context.scene)
    bpy.app.handlers.frame_change_pre.append(cell_frame)


def _set_arm(aid, t):
    a = CELL["arms"][aid]
    q = _at(CELL["s"]["tracks"].get(aid, []), t) or [0, 0, 0, 0, 90, 0]
    for j, rest, deg in zip(a["joints"], a["rest"], q):
        _turn(j, rest, deg)


def _set_stage(t):
    st = CELL["fixtures"].get("stage")
    if st is None:
        return
    row = _at(CELL["s"]["tracks"].get("stage", []), t) + [0.0] * 5
    turn, tilt, x, y, z = row[0:5]
    if st["turn"] is not None:
        _turn(st["turn"], st["rest"], turn)
    st["h"].matrix_basis = Matrix.Translation((x, y, z)) @ st["base"] @ Matrix.Rotation(math.radians(tilt), 4, "X")


def _carrier(by, t):
    """What carries a part at t: an arm's flange, or the stage's top."""
    if by == "stage":
        _set_stage(t)
        st = CELL["fixtures"]["stage"]
        return _world(st["turn"]) if st["turn"] is not None else _world(st["h"])
    _set_arm(by, t)
    return _world(CELL["arms"][by]["flange"])


def _part_at(pid, t):
    s = CELL["s"]
    m = CELL["parts"][pid]["start"].copy()
    events = [(a[2], "a", a) for a in s["attaches"] if a[0] == pid and a[2] <= t]
    events += [(p[1], "p", p) for p in s["places"] if p[0] == pid and p[1] <= t]
    events.sort(key=lambda e: e[0])
    for _, kind, e in events:
        if kind == "p":
            m = _pose(e[2])
            continue
        _, by, t0, t1 = e
        if by != "stage" and by not in CELL["arms"]:
            continue
        if by == "stage" and "stage" not in CELL["fixtures"]:
            continue
        offset = _carrier(by, t0).inverted() @ m
        m = _carrier(by, min(t, t1)) @ offset
    return m


def cell_frame(scene, *_):
    """The cell posed at the frame's time."""
    if "s" not in CELL:
        return
    t = (scene.frame_current - 1) / scene.render.fps
    mats = {pid: _part_at(pid, t) for pid in CELL["parts"]}
    for pid, p in CELL["parts"].items():
        p["h"].matrix_basis = mats[pid]
        for pin, forms in p["forms"].items():
            shown = None
            for ft, o in forms:
                o.hide_render = o.hide_viewport = True
                if ft <= t:
                    shown = o
            if shown is not None:
                shown.hide_render = shown.hide_viewport = False
            for m in p["meshes"].get(pin, []):
                m.hide_render = m.hide_viewport = shown is not None
    for j in CELL["joints"]:
        row = _at(j["rows"], t) if t >= j["rows"][0][0] else None
        mm3 = row[1] if row else 0.0
        j["o"].hide_render = j["o"].hide_viewport = mm3 <= 0.0
        if mm3 > 0.0:
            if j["lap"]:
                ln = j["lap"][6]
                r = max(0.4, math.sqrt(mm3 / (math.pi * ln)) * 1.6)
                j["o"].scale = (r, r, ln / 2.0)
            else:
                k = max(1.0, (3.0 * mm3 / (4.0 * math.pi)) ** (1.0 / 3.0) * 2.2)
                j["o"].scale = (k, k, k)
    for mv in CELL["movers"]:
        if mv["tool"] is not None:
            mv["tool"].matrix_basis = _pose(_at(mv["rows"], t))
        if mv["housing"] is not None:
            mv["housing"].matrix_basis = _pose(_at(mv["hrows"], t)) @ Matrix.Diagonal((20, 20, 30, 1))
    # the arms and the stage last, at t (a carrier asked about other times)
    for aid in CELL["arms"]:
        _set_arm(aid, t)
    _set_stage(t)


def cell_view(close=True):
    """A camera as the page's (its close-up or its whole view) and light."""
    yaw, pitch = math.radians(35), math.radians(22 if close else 28)
    dist, target = (200.0, Vector((0, 15, 0))) if close else (1700.0, Vector((0, 0, 60)))
    c = math.cos(pitch)
    # the page's y-up offset (x, up, z) is the cell's (x, -z, up)
    eye = target + Vector((dist * c * math.sin(yaw), -dist * c * math.cos(yaw), dist * math.sin(pitch)))
    cam = bpy.data.cameras.new("view")
    cam.lens = 35
    cam.clip_start, cam.clip_end = 1.0, 20000.0
    o = bpy.data.objects.new("view", cam)
    bpy.context.scene.collection.objects.link(o)
    o.location = eye
    o.rotation_mode = "QUATERNION"
    o.rotation_quaternion = (target - eye).to_track_quat("-Z", "Y")
    bpy.context.scene.camera = o
    for n, (loc, power) in enumerate((((300, -300, 600), 4.0e6), ((-400, 200, 400), 2.0e6))):
        light = bpy.data.lights.new("light%d" % n, "AREA")
        light.energy = power
        light.size = 300
        lo = bpy.data.objects.new("light%d" % n, light)
        bpy.context.scene.collection.objects.link(lo)
        lo.location = loc
        lo.rotation_mode = "QUATERNION"
        lo.rotation_quaternion = (target - Vector(loc)).to_track_quat("-Z", "Y")
    w = bpy.data.worlds.new("cell")
    w.use_nodes = True
    w.node_tree.nodes["Background"].inputs["Color"].default_value = (0.05, 0.06, 0.08, 1)
    bpy.context.scene.world = w
