# Ideas, not planned

Extensions suggested along the way and kept here as notes only: no
saga, no steps, no promise of order (the user's decision, 2026-10-04).
Each would follow the same pattern as the others -- a Rust crate behind
an ordinary typed X_eTaL facade, tested headlessly.

## Camera input: live image arrays

Webcam frames as n by m by 3 arrays, read a frame at a time, so an
X_eTaL program can filter them and show the result in a canvas window:
a live edge detector (a few rotations and a sum, as X_eTaL-demos'
image pipeline does to still images), motion as the difference of two
frames, a colour mask.

- Crate: `nokhwa` (cross-platform capture); a frame scaled down in Rust
  (an n by m by 3 Int array crossing the text bridge costs about
  2.6 million numbers a second, so 160 by 120 is comfortable).
- Shape of the facade: `cam:o_pen @` (an id), `cam:f_rame id` (the
  newest frame as an n by m by 3 array), `cam:c_lose! id`.
- Care: the camera is a device on the user's machine, like the screen
  and the speakers -- never opened without asking; headless tests read
  frames from a file instead (`XETAL_HEADLESS=1` plays a recorded
  sequence).

## Running ONNX models

A pretrained model run on X_eTaL arrays: X_eTaL prepares the input (a
digit drawn in a canvas window, scaled to 28 by 28) and reads the
output (ten scores), the model itself evaluated by Rust. It would tie
this repository to X_eTaL-ML, whose networks are written in X_eTaL:
the same digit through an X_eTaL network and an ONNX one, compared.

- Crate: `tract` (pure Rust ONNX inference; no native runtime).
- Shape of the facade: `nn:l_oad path` (an id), `id nn:r_un input` (the
  output array), `nn:s_hapes id` (input and output shapes).
- Care: models are files the user supplies, or a tiny one trained in
  X_eTaL-ML and exported; nothing downloaded.
