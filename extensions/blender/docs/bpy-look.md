# Bpy.xtl: the look

Textures, bevels, figures and cloth for the pieces `Bpy.xtl` describes:
what a game needs for a piece to look like what it is. Everything here
is written as Python text, as the rest of `Bpy.xtl` is, and run by
Blender in batch; the result is a `.glb` whose materials, skin, shapes
and clips Three.js's `GLTFLoader` and `AnimationMixer` use as they are.

A script that uses any of these joins `bp:look` after `bp:head`:

```
script := bp:head c_at bp:look c_at pieces c_at bp:report
```

`bp:head` is unchanged, so every script written before is the same
text as it was.

Written for and run with Blender 5.1.2 (macOS, `--background`).
`tests/LookPieces.xtl` describes three pieces with it -- a wall, a
walking figure, a banner -- and `tests/look-run.xtl` makes them.

## Textures and bevels

A texture is a named material, made once and shared by every part
painted with it. Its pattern is built from Blender's shader nodes so
that it repeats over a square, baked with Cycles into images, and the
images are packed in the exported `.glb` (each once, however many
parts use it).

| Function | What it writes |
| -------- | -------------- |
| `(name, kind, rgb, rgb2, meters, pixels, maps) bp:t_exture 0` | the texture `name`: `kind` one of `stone` `wood` `weave` `grass` `thatch` `snow`; its two colors; `meters` the width of the square it repeats over; `pixels` the images' side; `maps` 1 (base color), 2 (and roughness) or 3 (and normals) |
| `(part, width, segments) bp:b_evel 0` | the part's sharp edges (over 40 degrees) beveled `width` meters in `segments` faces, then shaded smooth with its large faces kept flat |
| `(part, texture, turn) bp:p_aint 0` | the part painted with the texture, laid along the part's own axes at the texture's meters; `turn` 1 lays it a quarter turn round |
| `(name, quality) bp:e_xportLook 0` | `out/NAME.glb` with its textures as JPEG of that quality (0: PNG) |
| `bp:c_ontents name` | `out/NAME-contents.txt`: the file read back (bytes, textures, skins, shapes, clips) |

```
("look stone", "stone", 0.6 0.63 0.67, 0.3 0.3 0.32, 1.0, 256, 3) bp:t_exture 0
("wall block", 0.0 0.0 1.0, 1.0 1.0 2.0, 0.0 0.0 0.0, 0.6 0.63 0.67, 0.0) bp:b_ox 0
("wall block", 0.03, 2) bp:b_evel 0
("wall block", "look stone", 0) bp:p_aint 0
("look-wall", 85) bp:e_xportLook 0
```

The kinds, over one square: stone is 4 courses of 2 blocks, every
other course moved half a block, its joints `rgb2`; wood is 4 planks
along x with the grain `rgb2`; weave is 16 threads each way; thatch is
3 layers of reeds; grass and snow have no direction. So with `meters`
1.0 a stone course is 0.25 m high and a block 0.5 m long.

How it is made: the pattern is a function of the square's two
coordinates; its noise is Blender's 4D noise read round two circles, so
it meets itself at every edge. Three values are baked from a flat
square with Cycles (emission: no light is involved): the color, the
roughness, and a height. The normals are worked out from the height,
round the edges, so they repeat too. A part's faces get coordinates in
meters along its own axes (a face toward x shows y and z, and so on),
so a texture keeps one scale on every part and a part's rotation
carries its texture with it. Bevel before painting.

### What it costs

Measured, JPEG at quality 85 (PNG, quality 0, is 4 to 10 times
larger):

| Texture | Color | Roughness | Normals |
| ------- | ----- | --------- | ------- |
| stone, 256 pixels, 1.0 m | 6.8 KB | 4.5 KB | 14.6 KB |
| wood, 256 pixels, 0.5 m | 9.2 KB | 7.6 KB | 10.4 KB |
| weave, 256 pixels, 0.5 m | 11.7 KB | 4.7 KB | 15.2 KB |
| grass, 256 pixels, 0.5 m | 14.1 KB | 5.3 KB | 45.2 KB |
| thatch, 256 pixels, 0.5 m | 13.4 KB | 6.3 KB | 20.8 KB |
| snow, 256 pixels, 0.5 m | 6.9 KB | 8.8 KB | 21.1 KB |
| wood, 128 pixels | 3.6 KB | 2.7 KB | - |
| weave, 128 pixels | 3.8 KB | - | - |

A beveled box (2 segments) is 54 faces, 108 triangles, 88 to 96
corners in the file. The wall of `LookPieces.xtl` -- five beveled
boxes, a stone texture with all three maps at 256 pixels and a wood
texture with two at 128 -- is 54 KB: 22 KB without its textures, 32 KB
of textures.

### Limits

- A texture is shared inside one `.glb`. Two files that use the same
  texture each carry their own copy.
- The images' names in the file are the texture's name and the map
  (`look-stone-color`); they are also written to `out/textures/` as
  PNG.
- JPEG is lossy: normals at quality 85 show no blocks at a board's
  distance, but use 0 (PNG) where it matters.
- Opposite faces of a part show the texture mirrored.
- `bp:m_eshToml` writes a painted part in the texture's first color.

## Figures

A figure is parts skinned to an armature, with clips keyed on its
bones.

| Function | What it writes |
| -------- | -------------- |
| `(name, bones, heads, tails, parents) bp:a_rmature 0` | the armature `name` in the piece (or group) being made: `bones` their names in one text, a space between; `heads` and `tails` n by 3, x y z in the piece; `parents` each bone's parent by number from 1, 0 for none |
| `(part, rig, bones, reach) bp:b_ind 0` | the part skinned to the armature: one bone binds it rigidly; with several each corner follows the nearest bone and shares with the others within `reach` meters more |
| `(rig, bone, clip, what, times, values) bp:p_ose 0` | the bone keyed in the clip: `what` is `rotation` (x y z degrees about the piece's axes, from its rest), `location` (meters from its rest) or `scale`; a row of values for each time |

```
("rig", "hips thigh shin", 3 3 r_eshape 0.0 0.0 0.45  0.0 0.0 0.45  0.0 0.0 0.24, 3 3 r_eshape 0.0 0.0 0.55  0.0 0.0 0.24  0.0 0.0 0.0, 0 1 2) bp:a_rmature 0
("leg", "rig", "thigh shin", 0.08) bp:b_ind 0
("rig", "thigh", "walk", "rotation", 0.0 0.5 1.0, 3 3 r_eshape 0.0 0.0 0.0  28.0 0.0 0.0  0.0 0.0 0.0) bp:p_ose 0
```

A rotation is given about the piece's axes, not the bone's own: a leg
that hangs down swings about x whichever way its bone was drawn. A
bone's keys carry its children, as a group's do. The clips are made as
`bp:k_eys` makes them (an action for each clip, a track of its name at
export), so a piece may have clips on bones and on parts under one
name. In the file: one skin, each bone a node, each clip an animation
of the bones' translation, rotation (quaternions) and scale, sampled a
frame at a time (30 a second unless `bp:s_etup` changed it).

The figure of `LookPieces.xtl` -- nine bones, seven parts, 828
triangles, two clips (walk 1 s, idle 2 s) -- is 96 KB, 92 KB without
its one texture.

### Limits

- Rotation keys must be less than half a turn apart (they are joined
  the short way round).
- The weights are worked out from distances to the bones, at most as
  many bones as were named for the part; glTF keeps four a corner.
- A part needs corners where it is to bend: a box bends only at its
  ends, a turned limb (`bp:l_athe`) at each ring.
- Key a skinned part's bones, not the part: it hangs from the
  armature once bound.
- Every bone is written in every clip (the exporter samples all of
  them), which is most of a small figure's bytes.
