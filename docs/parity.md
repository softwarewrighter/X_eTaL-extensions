# Parity with sw-MLPL's demo-extensions

`../../sw-ml-study/demo-extensions` did for sw-MLPL what this
repository does for X_eTaL, and went further. This page compares the
two (2026-10-03) and says what is planned here. Code and design may be
copied from it (plan A14), never depended on.

## The boundary

| demo-extensions | Here | Plan |
| --------------- | ---- | ---- |
| ABI V1: one entry symbol, versioned `#[repr(C)]` descriptor, validation, panic containment | the same, reduced to X_eTaL's types, plus X_eTaL signatures per function | done |
| values: nil, bool, i64, f64, UTF-8, bytes, dense arrays (u8, i64, f32, f64), native handles, nested records | Bool, Int, Float, Text, dense Bool/Int/Float/Char arrays | handles as Int ids issued by an extension (no ABI change) when the audio extension needs them; bytes when X_eTaL can read binary files (ask E7); records not needed (X_eTaL has no records) |
| SDK, loader, manifests, dynamic and static providers | the same | done |
| the language calls natively: sw-MLPL gained a host registry and a C-descriptor adapter upstream | the `ext:` bridge (`xetal-x`) and its browser counterpart | ask E1, X_eTaL Saga 23 |
| a host that owns a window: the viewer embeds the interpreter and calls it per frame | `xetal-x` runs the program on X_eTaL's worker thread, its main thread waits | media saga: the main thread runs the window; the program pulls events and pushes scene patches |

## Extensions and demos

| demo-extensions | Here | Plan |
| --------------- | ---- | ---- |
| hello | hello (live) | done |
| digest | -- | roadmap (cheap) |
| sqlite | sqlite + the data notebook | done |
| http-client, http-server, TodoMVC, experiment dashboard | -- | roadmap: web saga (axum), fetch saga |
| canvas: a blocking native window presenting an array | -- | media saga, as the first use of the window host |
| native3d: retained line/point scenes with stable ids, camera, picking; wireframe cube, Life plane and torus, tic-tac-toe, point cloud, weight distribution, model atlas, disk usage, system layouts | -- | media saga: the scene extension (adapted from `mlpl-native3d-scene` and `-window`); the visualizer first, then a cube and Life in 3D as smaller demos |
| audio: Symphonia decode of MP3 and Ogg/Vorbis in bounded chunks, CPAL playback with decode-ahead, pause, seek | -- | media saga: the audio extension |
| audio-spectrum: the MP3 player visualizer | -- | media saga: X_eTaL analyses each chunk (a spectrum by one inner product with a cosine/sine table, bass/mid/high), Rust renders |
| hftok (Hugging Face tokenizer), verified downloads | -- | not planned (closer to the ML line) |
| Yew/WebAssembly ML microscope | hello and clock live, the shell | done for this repo's needs |

## Division of labour in the visualizer

Rust does what an array interpreter is bad at or cannot do: decoding
MP3 (a sequential bitstream with Huffman codes), feeding the sound
device in real time, and drawing with the GPU. X_eTaL does what it is
good at: the analysis and the picture's geometry as whole-array
arithmetic. The array-shaped parts of decoding (MP3's IMDCT and
synthesis filterbank are matrix products; WAV is a reshape) could be a
teaching demo in X_eTaL once it can read binary files (ask E7).
