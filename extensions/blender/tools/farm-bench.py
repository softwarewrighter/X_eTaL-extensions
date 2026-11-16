# Run by Blender for the farm (scripts/farm.py check and bench):
#   ... -- devices        the devices Cycles would render with, a line each
#   ... -- bench FILE     the time one standard picture takes on them
# The devices are chosen as Cycles.xtl's player chooses them: the first
# of OptiX, CUDA, HIP, oneAPI and Metal that has a device, else the
# processor.
import sys
import time

import bpy

args = sys.argv[sys.argv.index("--") + 1:]
scene = bpy.context.scene
scene.render.engine = "CYCLES"
prefs = bpy.context.preferences.addons["cycles"].preferences
chosen = "CPU"
for kind in ("OPTIX", "CUDA", "HIP", "ONEAPI", "METAL"):
    try:
        prefs.compute_device_type = kind
    except TypeError:
        continue
    prefs.get_devices()
    if any(d.type == kind for d in prefs.devices):
        for d in prefs.devices:
            d.use = d.type == kind
        chosen = kind
        break
if chosen == "CPU":
    print("device: CPU (no graphics card Cycles can use)")
else:
    scene.cycles.device = "GPU"
    for d in prefs.devices:
        if d.use:
            print("device: %s %s" % (chosen, d.name))
if args[0] == "bench":
    # a picture that takes a graphics card some seconds: many spheres, glossy, lit by the sky
    import math
    for o in list(bpy.data.objects):
        bpy.data.objects.remove(o)
    world = bpy.data.worlds.new("sky")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs[1].default_value = 1.0
    scene.world = world
    mat = bpy.data.materials.new("m")
    mat.use_nodes = True
    mat.node_tree.nodes["Principled BSDF"].inputs["Roughness"].default_value = 0.2
    mat.node_tree.nodes["Principled BSDF"].inputs["Metallic"].default_value = 0.6
    bpy.ops.mesh.primitive_plane_add(size=60)
    for i in range(12):
        for j in range(12):
            bpy.ops.mesh.primitive_uv_sphere_add(radius=0.45, location=(i - 5.5, j - 5.5, 0.45 + 0.3 * math.sin(i * 1.7 + j)), segments=48, ring_count=24)
            bpy.context.object.data.materials.append(mat)
    cam = bpy.data.objects.new("camera", bpy.data.cameras.new("camera"))
    scene.collection.objects.link(cam)
    cam.location = (0.0, -15.0, 9.0)
    cam.rotation_euler = (math.radians(60), 0.0, 0.0)
    scene.camera = cam
    scene.render.resolution_x, scene.render.resolution_y, scene.render.resolution_percentage = 1280, 720, 100
    scene.cycles.samples = 256
    scene.cycles.use_adaptive_sampling = False
    scene.cycles.use_denoising = False
    scene.render.filepath = args[1]
    scene.render.image_settings.file_format = "PNG"
    began = time.time()
    bpy.ops.render.render(write_still=True)
    print("bench: %.1f seconds (1280 by 720, 256 samples, %s)" % (time.time() - began, chosen))
