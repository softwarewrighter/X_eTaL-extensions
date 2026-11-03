# HEX1000-keys-leds

Queued Saga 16 in X_eTaL-extensions, requested 2026-11-03.
Source: hardwarewrighter/hex1000, Tang Nano 1K with a physical 4x4
hex keypad, four operator buttons and an eight-digit MAX7219 display.
Candidate pin: b7148fea99121e60f4f58151682dce96cbbfa5be (clean local
checkout when planned). Confirm and pin at the evidence step.
This is a separate project from the existing Tang Nano 9K UART design.

Public names: HEX1000-UART for the existing 9K execution/build videos;
HEX1000-keys-leds for the new 1K execution/build videos. Keep existing
demo paths and published anchors working. The shared HEX1000 program
counter explainer remains shared. New slugs: hex-keys-leds-run and
hex-keys-leds-build.

## Steps

1. uart-names: distinguish the existing UART videos in titles, captions,
   docs and site; retain old commands, asset paths and anchors.
2. keys-leds-evidence: pin and clone hardwarewrighter/hex1000 separately,
   identify full calculator sources and board variant, extract evidence.
3. keys-leds-run: a waveform-driven execution video, actual key contacts,
   scanner, debounce, machine and MAX7219 traffic checked against X_eTaL.
4. keys-leds-build: the full 1K calculator synthesis/placement/routing
   and bitstream, actual counts and BELs, schematic panel model.
5. keys-leds-release: landscape masters and Shorts, cue sheets/subtitles,
   headless tests, site recordings, documentation, gate and live check.

Every implementation step uses X_eTaL to compute scene arrays and write
Blender scripts. Reuse Video/Bpy and evidence readers where suitable.
Use the new project's full calculator top, not its peripheral-free
self-test or the 9K UART wrapper. No hardware programming is required.
Distinguish simulation checks from historical user-observed hardware
results. Do not invent geometry dimensions or physical wire routes.
