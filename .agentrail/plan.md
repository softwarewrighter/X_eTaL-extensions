# visualizer

The media saga resumed (the user, 2026-10-03: while .xtlm is not ready
upstream). Demos like sw-MLPL demo-extensions' MP3 player visualizer:
Rust decodes and plays audio (Symphonia, CPAL; Ogg Vorbis first) and
draws a retained 3D scene (CPU, softbuffer, in xetal-x's UI host);
X_eTaL analyses each chunk and builds the scene as arrays. Plan
decisions M1-M7 (docs/plan.md, Saga 5). Verified headlessly (saved
frames); the screen only with the user's go (screen protocol).

## Steps

1. scene -- the scene extension (from branch wip/media-scene): objects
   by id, orbit camera, CPU rendering, headless frames; a cube demo.
2. audio -- open, info, chunks, play, pause, seek, position, close;
   generated Ogg/MP3/WAV fixtures.
3. spectrum -- the visualizer demo in X_eTaL; headless golden; video.
4. visualizer-release -- docs, parity, status, videos, retrospective.
