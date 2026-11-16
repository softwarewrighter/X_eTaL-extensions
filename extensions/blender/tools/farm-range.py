# Run by Blender for the farm (scripts/farm.py): a scene script written
# by Cycles.xtl (the player and the scene's calls), with what it would
# render left out, and then one thing rendered:
#   blender --background --python farm-range.py -- OUT SCENE frames FIRST LAST
#   blender --background --python farm-range.py -- OUT SCENE still FILE SECONDS
#   blender --background --python farm-range.py -- OUT SCENE encode FILE
# The scene's own lines that render (still, frames, encode) are not
# run: the farm says what to render. OUT is where the scene writes, as
# the scene itself reads it (the first argument after "--"); frames go
# to OUT/frames/NNNN.png and a frame that is there is not rendered
# again, so a run that stopped can be run again.
import sys

args = sys.argv[sys.argv.index("--") + 1:]
out, scene, what = args[0], args[1], args[2]
with open(scene, encoding="utf-8") as f:
    lines = f.read().split("\n")
renders = ("still(", "frames(", "encode(")
kept = [line for line in lines if not line.startswith(renders)]
left = len(lines) - len(kept)
space = {"__name__": "__main__", "__file__": scene}
exec(compile("\n".join(kept), scene, "exec"), space)
print("farm: the scene is made (%d rendering lines of its own left out)" % left)
if what == "frames":
    first, last = int(args[3]), int(args[4])
    print("farm: frames %d to %d" % (first, last))
    space["frames"]("frames", first, last)
elif what == "still":
    space["still"](args[3], float(args[4]))
elif what == "encode":
    space["encode"]("frames", args[3])
else:
    sys.exit("farm-range: frames, still or encode, not %r" % what)
print("farm: done")
