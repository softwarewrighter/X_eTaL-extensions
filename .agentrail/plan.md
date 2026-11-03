# blender

X_eTaL drives Blender 3D in batch (the user, 2026-10-11): X_eTaL writes
the Python -- an outer launcher that runs Blender in the background, and
the full bpy script Blender runs inside -- and an extension runs the
launcher and reports what happened. Short term: animations (rendered
video) and exportable 3D files (glTF, OBJ); later more advanced meshes,
materials and effects. Renderer: EEVEE. docs/plan.md, Saga 15.

## Steps

1. blender-ext -- the extension: run a launcher with a timeout, the
   Blender it is configured with, logs and outputs back; write files.
2. bpy-gen -- an X_eTaL library that writes bpy code from arrays:
   scene reset, meshes (many cubes as one mesh), materials, lights,
   camera, keyframed orbit, EEVEE settings, video and file output.
3. blender-cubes -- the demo: one cube, a 10 by 10 slab, a 10 by 10 by
   10 block, labeled, orbited 360 degrees; a video and .glb/.obj.
4. blender-release -- recordings on the site, docs, the site checked.

## Additional planning, 2026-11-03

Step 17 keys-leds-plan queues independent Saga 16 at
.agentrail-queue/hex1000-keys-leds/.agentrail/ (docs/plan.md).
The existing GPU steps are now 18 gpu-build, 19 gpu-run and
20 explain-release. The queued saga distinguishes HEX1000-UART
(9K) and HEX1000-keys-leds (1K physical keypad/MAX7219), retaining
existing public links. Use agentrail --saga .agentrail-queue/hex1000-keys-leds
for that saga; do not replace or discard this active record.
