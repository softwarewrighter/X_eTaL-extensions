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
| a host that owns a window: the viewer embeds the interpreter and calls it per frame | `xetal-x` gives its main thread to windows (the UI host); the program, on its own thread, pulls events and pushes scene patches | done |

## Extensions and demos

| demo-extensions | Here | Plan |
| --------------- | ---- | ---- |
| hello | hello (live) | done |
| digest | digest: SHA-256 and CRC-32 of text and files | done (2026-10-05) |
| sqlite | sqlite + the data notebook | done |
| http-server, TodoMVC | web (axum on loopback; the program takes each request and replies) + the live Life page and TodoMVC kept in SQLite | done (2026-10-05) |
| http-client, experiment dashboard | -- | roadmap: fetch saga (http) |
| canvas: a blocking native window presenting an array | canvas: arrays as pixels, keys and clicks back; Life | done |
| native3d: retained line/point scenes with stable ids, camera, picking; wireframe cube, Life plane and torus, tic-tac-toe, point cloud, weight distribution, model atlas, disk usage, system layouts | scene: retained lines, segments and dots by id, orbit camera, drawn on the CPU (frames pinned headless); the cube | done for lines and points; picking, boxes and the larger demos not planned now |
| audio: Symphonia decode of MP3 and Ogg/Vorbis in bounded chunks, CPAL playback with decode-ahead, pause, seek | audio: the same (plus WAV), the window under the playhead, a virtual clock for exact tests | done |
| audio-spectrum: the MP3 player visualizer | `just demo audio spectrum`: X_eTaL analyzes (a spectrum by one inner product), Rust plays and draws; recorded with its music | done; checked live 2026-10-04 |
| hftok (Hugging Face tokenizer), verified downloads | -- | not planned (closer to the ML line) |
| Yew/WebAssembly ML microscope | hello and clock live, the shell | done for this repo's needs |

## Division of labor in the visualizer

Rust does what an array interpreter is bad at or cannot do: decoding
MP3 (a sequential bitstream with Huffman codes), feeding the sound
device in real time, and drawing with the GPU. X_eTaL does what it is
good at: the analysis and the picture's geometry as whole-array
arithmetic. The array-shaped parts of decoding (MP3's IMDCT and
synthesis filterbank are matrix products; WAV is a reshape) could be a
teaching demo in X_eTaL once it can read binary files (ask E7).
