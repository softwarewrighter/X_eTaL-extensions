# image

Pictures as X_eTaL arrays: a PNG or JPEG read into a Float array of
values from 0 to 1 -- height by width for gray, height by width by 3
for color -- and an array written back as a picture, through the
`image` crate. Once a photo is an array, X_eTaL does the rest:
filters, edges, statistics, compression.

- Package: `extensions/image/` (`extension.toml`; library
  `xetal_ext_image`: `libxetal_ext_image.dylib` on macOS, `.so` on
  Linux)
- Facade: `lib/Image.xtl`, recommended alias `im:`
- Native crates: `image` 0.25 (PNG and JPEG only); `tempfile` for tests

## From X_eTaL

```
"im:" u_se< "Image"
p := im:g_ray "demos/data/photo.jpg"        # h by w, 0 (black) to 1 (white)
im:s_ize "demos/data/photo.jpg"              # height width
e := 0.0 m_ax p - 1 o_-_2 p                  # a vertical-edge filter, in X_eTaL
"work/edges.png" im:w_rite! 4.0 * e          # back to a PNG
```

| Export | Type | What |
| ------ | ---- | ---- |
| `im:r_ead path` | `Char -> Float` | the picture as it is: h by w (gray) or h by w by 3 (color); alpha dropped |
| `im:g_ray path` | `Char -> Float` | the picture in gray (its luminance), h by w |
| `im:s_ize path` | `Char -> Int` | height and width, without reading the pixels |
| `path im:w_rite! a` | `Num a => Char -> a -> Int` | the array as a PNG (or a JPEG, by the name's extension), values clamped to 0..1 (NaN is 0); how many pixels |
| `(h c_at w) im:r_esize a` | `(Num a, Num b) => a -> b -> Float` | the picture resampled to h by w (Lanczos); gray stays gray |

Native functions: `read`, `gray`, `size`, `write`, `resize` (`just
list`).

Each 8-bit level k reads as k / 255, so a picture written and read
back is the same array. Files are named by paths under the working
directory (or `XETAL_IMAGE_ROOT`): relative, with no `..`. Pictures are
limited to 64 megapixels. Arrays cross the `ext:` bridge as text
(about 0.35 microseconds a number), so a 400 by 300 color photo takes
a few tenths of a second each way: read once, compute, write once.

## Build and test

```sh
just build      # the native library and xetal-x
just test       # Rust tests (rust/tests) and reg-rs tests (tests/)
just list       # the functions as xetal-x sees them
```

The tests make their pictures (nothing is downloaded): gray and color
written and read back exactly, values clamped, JPEG by the name,
resizing, the errors. The reg-rs tests pin the facade's types, a round
trip from X_eTaL, and the error for a path outside.

## Demos

Coming next: the photo lab.
