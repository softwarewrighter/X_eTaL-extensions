# assembly-cell

A programmable robot assembly cell, virtual first (the user,
2026-11-24): articulated arms hold through-hole parts (later SMD) lead
to lead while others solder, spot weld, clip heatsinks and run fine
wire, watched by a simulated stereo USB microscope; a semi-authentic,
physics-respecting 3D playground an LLM agent drives command by
command over MCP, previewed step by step like a slicer. Design:
docs/cell-design.md. Plan and decisions C1-C9: docs/plan.md, Saga 17.

Every step: just gate (or check-quick for docs only), docs, CHANGES.md,
commit including both saga records, complete with
--saga .agentrail-queue/assembly-cell, push.
