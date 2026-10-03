# media

Saga 5 of X_eTaL-extensions (docs/plan.md, decisions M1-M5;
docs/parity.md): demos like sw-MLPL demo-extensions' MP3 player
visualizer. Rust decodes and plays audio (Symphonia, CPAL) and renders
a retained 3D scene (winit, wgpu; adapted from demo-extensions'
native3d crates, copied, never depended on); X_eTaL analyses each
chunk and builds the scene as arrays. xetal-x gives its main thread to
the window (macOS) and runs the program on another; the program pulls
events and pushes changes.

Rules as before (CLAUDE.md). Every step gated, documented, committed
with .agentrail/, completed, pushed.

## Steps

1. parity -- docs/parity.md; this saga.
2. ui-host -- crates/xetal-ext-ui, xetal-x off the main thread, a
   minimal pixel window proving the loop.
3. scene3d -- retained 3D lines and points by id, camera, events; a
   wireframe cube.
4. audio -- open, chunks, play, pause, seek, position; fixtures.
5. spectrum -- the visualizer in X_eTaL.
6. media-release -- docs, parity, status, retrospective.
