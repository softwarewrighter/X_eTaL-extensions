# cell_models.py -- the Python the assembly cell's models are built
# with inside Blender, after Bpy.xtl's head (bp:head: bpy, scene,
# material, piece, export_piece). lib/CellModels.xtl writes the calls:
# what each model is made of comes from X_eTaL as numbers; the arm's
# pivots and a part's pins come from the simulator as JSON.
#
# Units: millimeters and degrees here (the simulator's); Blender's
# scene is meters, so every length is scaled by MM. A model's root is
# an empty at its origin; its named nodes are empties a player or a
# check finds by name.
import json

import mathutils

MM = 0.001
MATS = {}


def mat(name, rgb, emit=0.0):
    # one material a name, made once
    if name not in MATS:
        MATS[name] = material(name, tuple(rgb), emit)
    return MATS[name]


def fresh():
    # an empty scene for the next model: node names repeat between
    # models ("1", "tool_tip") and Blender would rename the second
    global scene
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = bpy.context.scene
    MATS.clear()


def save(root, name):
    export_piece(root, os.path.join(out, name + ".glb"))


def vec(text):
    return tuple(float(t) for t in text.split())


def empty(name, owner, at=(0, 0, 0), turn=(0, 0, 0)):
    e = bpy.data.objects.new(name, None)
    scene.collection.objects.link(e)
    e.parent = owner
    e.location = tuple(v * MM for v in at)
    e.rotation_euler = tuple(math.radians(a) for a in turn)
    e.empty_display_size = 0.01
    return e


def finish(o, name, owner, m):
    o.name = name
    o.data.name = name
    o.parent = owner
    o.data.materials.clear()
    o.data.materials.append(m)
    return o


def between(name, owner, p0, p1, r, m, sides=12):
    # a cylinder of radius r from p0 to p1 (mm, in the owner's frame)
    a = mathutils.Vector(p0) * MM
    b = mathutils.Vector(p1) * MM
    d = b - a
    if d.length < 1e-9:
        return None
    bpy.ops.mesh.primitive_cylinder_add(vertices=sides, radius=r * MM, depth=d.length)
    o = finish(bpy.context.active_object, name, owner, m)
    o.rotation_mode = "QUATERNION"
    o.rotation_quaternion = d.to_track_quat("Z", "Y")
    o.location = (a + b) / 2
    return o


def box(name, owner, at, size, m, turn=(0, 0, 0)):
    bpy.ops.mesh.primitive_cube_add(size=1.0)
    o = finish(bpy.context.active_object, name, owner, m)
    o.scale = tuple(v * MM for v in size)
    o.location = tuple(v * MM for v in at)
    o.rotation_euler = tuple(math.radians(a) for a in turn)
    return o


def ball(name, owner, at, radii, m):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=16, ring_count=8, radius=1.0)
    o = finish(bpy.context.active_object, name, owner, m)
    o.scale = tuple(v * MM for v in radii)
    o.location = tuple(v * MM for v in at)
    return o


AXES = {"x": (0, 1, 0, 90), "y": (1, 0, 0, -90), "z": (0, 0, 1, 0)}


def along(name, owner, center, axis, r, length, m, sides=16):
    # a cylinder of radius r and that length along axis x, y or z
    c = mathutils.Vector(center)
    u = {"x": (1, 0, 0), "y": (0, 1, 0), "z": (0, 0, 1)}[axis]
    h = mathutils.Vector(u) * (length / 2)
    return between(name, owner, tuple(c - h), tuple(c + h), r, m, sides)


def d_shape(name, owner, r, length, flat, m):
    # a cylinder along z with one side cut flat, flat mm from the axis
    # toward -y (TO-92's body)
    bm = bmesh.new()
    bmesh.ops.create_cone(bm, cap_ends=True, segments=24, radius1=r * MM, radius2=r * MM, depth=length * MM)
    geom = bm.verts[:] + bm.edges[:] + bm.faces[:]
    cut = bmesh.ops.bisect_plane(bm, geom=geom, plane_co=(0, -flat * MM, 0), plane_no=(0, -1, 0), clear_outer=True)
    edges = [e for e in cut["geom_cut"] if isinstance(e, bmesh.types.BMEdge)]
    bmesh.ops.holes_fill(bm, edges=edges, sides=0)
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    o = bpy.data.objects.new(name, me)
    scene.collection.objects.link(o)
    return finish(o, name, owner, m)


def write_json(name, data):
    with open(os.path.join(out, name), "w") as f:
        json.dump(data, f, indent=1, sort_keys=False)
        f.write("\n")


# -- the arm ------------------------------------------------------------


def build_arm(spec_path, housing, links, colors):
    # housing: per joint (j1..j6) a radius and a length along its axis;
    # links: per joint a radius; colors: body, joints, flange (rgb)
    with open(spec_path) as f:
        spec = json.load(f)
    housing = [housing[i:i + 2] for i in range(0, 12, 2)]
    colors = [colors[i:i + 3] for i in range(0, 9, 3)]
    body = mat("arm_body", colors[0])
    joint = mat("arm_joint", colors[1])
    flange = mat("arm_flange", colors[2])
    made = {}
    for n in spec["nodes"]:
        owner = made.get(n["parent"]) if n["parent"] else None
        made[n["name"]] = empty(n["name"], owner, n["at"], n["turn"])
    base = made["arm_base"]
    box("arm_plate", base, (0, 0, 8), (220, 220, 16), joint)
    for i in range(6):
        g = made["arm_j%d" % (i + 1)]
        r, length = housing[i]
        along("arm_j%d_housing" % (i + 1), g, (0, 0, 0), "z", r, length, joint, 24)
    for k, link in enumerate(spec["links"]):
        i = int(link["group"][-1]) - 1
        between("arm_link%d_%d" % (i + 1, k % 2), made[link["group"]], link["from"], link["to"], links[i], body, 16)
    along("arm_flange_disc", made["arm_flange"], (0, 0, -3), "z", 32, 6, flange, 24)
    write_json("arm.rig.json", spec)
    return base


# -- parts ----------------------------------------------------------------


def build_part(kind_path, colors, bands):
    # colors: body, lead, mark (rgb); bands: rgb triples for a
    # resistor's bands, left to right (none for other kinds)
    with open(kind_path) as f:
        kind = json.load(f)
    colors = [colors[i:i + 3] for i in range(0, 9, 3)]
    bands = [bands[i:i + 3] for i in range(0, len(bands), 3)]
    name = kind["kind"]
    root = empty(name, None)
    body = mat(name + "_body", colors[0])
    lead = mat("lead", colors[1])
    mark = mat(name + "_mark", colors[2])
    b = kind["body"]
    if b["shape"] == "box":
        box(name + "_body", root, (0, 0, 0), b["size"], body)
    elif b["shape"] == "cylinder":
        along(name + "_body", root, (0, 0, 0), b["axis"], b["diameter"] / 2, b["length"], body, 24)
    else:
        d_shape(name + "_body", root, b["diameter"] / 2, b["length"], b["flat"], body)
    for p in kind["pins"]:
        pts = p["lead"]
        for k in range(len(pts) - 1):
            between("%s_lead%s_%d" % (name, p["name"], k), root, pts[k], pts[k + 1], p["diameter"] / 2, lead, 8)
        empty(p["name"], root, pts[0])
        empty(p["name"] + ".tip", root, pts[-1])
    for g in kind["grasps"]:
        empty("grasp." + g["name"], root, g["at"], g["rpy"])
    marks(name, root, kind, body, mark, bands)
    nodes = [{"name": p["name"], "at": p["lead"][0]} for p in kind["pins"]]
    nodes += [{"name": p["name"] + ".tip", "at": p["lead"][-1]} for p in kind["pins"]]
    nodes += [{"name": "grasp." + g["name"], "at": g["at"]} for g in kind["grasps"]]
    write_json(name + ".rig.json", {"name": name, "units": "mm, degrees", "nodes": nodes, "part": kind})
    return root


def marks(name, root, kind, body, mark, bands):
    # what tells a part's value or way round: bands, a notch, a stripe
    b = kind["body"]
    if name == "axial-resistor":
        r = b["diameter"] / 2 + 0.05
        for k, rgb in enumerate(bands):
            x = -2.0 + 1.0 * k + (0.6 if k == len(bands) - 1 else 0.0)
            along("%s_band%d" % (name, k + 1), root, (x, 0, 0), "x", r, 0.5, mat("band%d" % k, rgb), 24)
    elif name == "axial-diode":
        along(name + "_band", root, (1.3, 0, 0), "x", b["diameter"] / 2 + 0.05, 0.5, mark, 24)
    elif name.startswith("dip"):
        along(name + "_notch", root, (-b["size"][0] / 2 + 0.8, 0, b["size"][2] / 2), "z", 0.6, 0.2, mark, 12)
    elif name == "radial-electrolytic":
        box(name + "_stripe", root, (1.2, -b["diameter"] / 2, 0), (1.6, 0.2, b["length"] * 0.9), mark)
    elif name == "led-5mm":
        ball(name + "_dome", root, (0, 0, b["length"] / 2), (2.5, 2.5, 2.5), body)
        along(name + "_rim", root, (0, 0, -b["length"] / 2 + 0.5), "z", 2.9, 1.0, mark, 24)
    elif name == "battery-snap":
        for k, x in enumerate((-6.35, 6.35)):
            along("%s_stud%d" % (name, k + 1), root, (x, 0, b["size"][2] / 2 + 1.5), "z", 3.0, 3.0, mark, 16)


# -- tools and fixtures: shapes from X_eTaL ------------------------------


def build(name, rows, nodes):
    # rows: (kind, part name, group, at, size, turn, rgb) -- kind box,
    # cyl (size r r h, along z), ball (radii) or seg (at to size, radius
    # in turn's first number); nodes: (name, parent, at, turn) empties,
    # made first, the model's own groups and named points
    root = empty(name, None)
    made = {name: root}
    for n, parent, at, turn in nodes:
        made[n] = empty(n, made[parent], at, turn)
    for kind, part, group, at, size, turn, rgb in rows:
        owner = made[group]
        m = mat(part, rgb)
        if kind == "box":
            box(part, owner, at, size, m, turn)
        elif kind == "cyl":
            o = along(part, owner, (0, 0, 0), "z", size[0], size[2], m, 20)
            o.rotation_mode = "XYZ"
            o.location = tuple(v * MM for v in at)
            o.rotation_euler = tuple(math.radians(a) for a in turn)
        elif kind == "ball":
            ball(part, owner, at, size, m)
        else:
            between(part, owner, at, size, turn[0], m, 12)
    world = {}
    bpy.context.view_layer.update()
    for n, _, _, _ in nodes:
        w = made[n].matrix_world
        world[n] = [round(v / MM, 6) for v in w.translation]
    rig = {"name": name, "units": "mm, degrees",
           "nodes": [{"name": n, "parent": parent, "at": list(at), "turn": list(turn), "world": world[n]}
                     for n, parent, at, turn in nodes]}
    write_json(name + ".rig.json", rig)
    return root


# -- the check: each .glb read back against its rig ----------------------


def node_frames():
    bpy.context.view_layer.update()
    return {o.name: o for o in bpy.data.objects}


def check(names):
    # import each NAME.glb into an empty scene; every node of its rig is
    # there, where the rig says (0.001 mm); the arm, posed at each check
    # pose, puts its flange where the simulator says (0.01 mm)
    lines = []
    for name in names:
        with open(os.path.join(out, name + ".rig.json")) as f:
            rig = json.load(f)
        bpy.ops.wm.read_factory_settings(use_empty=True)
        bpy.ops.import_scene.gltf(filepath=os.path.join(out, name + ".glb"))
        objs = node_frames()
        bad = []
        for n in rig["nodes"]:
            o = objs.get(n["name"])
            want = n.get("world", n["at"])
            if o is None:
                bad.append("no node " + n["name"])
                continue
            got = [v / MM for v in o.matrix_world.translation]
            if max(abs(g - w) for g, w in zip(got, want)) > 0.001:
                bad.append("%s at %s, not %s" % (n["name"], fmt(got), fmt(want)))
            if "axis" in n:
                z = o.matrix_world.to_3x3() @ mathutils.Vector((0, 0, 1))
                if max(abs(a - b) for a, b in zip(z.normalized(), n["axis"])) > 1e-6:
                    bad.append("%s axis %s, not %s" % (n["name"], fmt(z), fmt(n["axis"])))
        posed = 0
        groups = [objs.get("arm_j%d" % (i + 1)) for i in range(6)] if rig.get("poses") else []
        rest = [g.matrix_basis.to_3x3().normalized() for g in groups if g]
        for p in rig.get("poses", []):
            for i, q in enumerate(p["q"]):
                g = groups[i]
                g.rotation_mode = "QUATERNION"
                g.rotation_quaternion = (rest[i] @ mathutils.Matrix.Rotation(math.radians(q), 3, "Z")).to_quaternion()
            bpy.context.view_layer.update()
            f = objs["arm_flange"].matrix_world
            got = [v / MM for v in f.translation]
            z = (f.to_3x3() @ mathutils.Vector((0, 0, 1))).normalized()
            if max(abs(g - w) for g, w in zip(got, p["flange"])) > 0.01 or max(abs(a - b) for a, b in zip(z, p["z"])) > 1e-5:
                bad.append("posed %s: flange at %s, not %s" % (fmt(p["q"]), fmt(got), fmt(p["flange"])))
            posed += 1
        what = "%d nodes" % len(rig["nodes"]) + (", %d poses" % posed if posed else "")
        lines.append("%s: %s %s" % (name, what, "ok" if not bad else "WRONG: " + "; ".join(bad)))
    with open(os.path.join(out, "check.txt"), "w") as f:
        f.write("".join(line + "\n" for line in lines))


def fmt(v):
    return " ".join("%.3f" % x for x in v)


# -- turntable stills ----------------------------------------------------


def stills(names, size, angles):
    # each model alone on a gray floor, from each angle (degrees about
    # z, 25 degrees above), framed to its size: out/stills/NAME-A.png
    for name in names:
        bpy.ops.wm.read_factory_settings(use_empty=True)
        sc = bpy.context.scene
        bpy.ops.import_scene.gltf(filepath=os.path.join(out, name + ".glb"))
        objs = [o for o in sc.objects if o.type == "MESH"]
        lo = mathutils.Vector((1e9, 1e9, 1e9))
        hi = -lo
        for o in objs:
            for c in o.bound_box:
                w = o.matrix_world @ mathutils.Vector(c)
                lo = mathutils.Vector(map(min, lo, w))
                hi = mathutils.Vector(map(max, hi, w))
        mid = (lo + hi) / 2
        radius = max((hi - lo).length / 2, 0.005)
        sc.render.engine = "BLENDER_EEVEE"
        sc.render.resolution_x, sc.render.resolution_y = size
        sc.view_settings.view_transform = "Standard"
        world = bpy.data.worlds.new("sky")
        world.color = (0.18, 0.19, 0.21)
        sc.world = world
        sun = bpy.data.objects.new("sun", bpy.data.lights.new("sun", "SUN"))
        sun.data.energy = 3.0
        sun.rotation_euler = (math.radians(40), math.radians(10), math.radians(30))
        sc.collection.objects.link(sun)
        cam = bpy.data.objects.new("cam", bpy.data.cameras.new("cam"))
        cam.data.lens = 50
        sc.collection.objects.link(cam)
        sc.camera = cam
        for a in angles:
            t = math.radians(a)
            dist = radius / math.tan(math.radians(18)) * 1.15
            eye = mid + mathutils.Vector((math.sin(t) * math.cos(math.radians(25)), -math.cos(t) * math.cos(math.radians(25)), math.sin(math.radians(25)))) * dist
            cam.location = eye
            cam.rotation_euler = (mid - eye).to_track_quat("-Z", "Y").to_euler()
            cam.data.clip_start = dist / 100
            cam.data.clip_end = dist * 10
            sc.render.filepath = os.path.join(out, "stills", "%s-%03d.png" % (name, a))
            bpy.ops.render.render(write_still=True)
