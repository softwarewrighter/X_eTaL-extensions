# launch

Saga 6 of X_eTaL-extensions (docs/plan.md): this repo's part of the
ecosystem's wider launch, as ../X_eTaL/docs/research4.txt reprioritizes
it -- stabilize, synchronize, explain; no new feature sagas. Extensions
are not the critical path: the point (ordinary X_eTaL -> typed facade
-> stable ABI -> native capability) is made by sqlite. Audio and 3D
(the media saga, paused) are post-launch.

Rules as before (CLAUDE.md). Every step gated, documented, committed
with .agentrail/, completed, pushed.

## Steps

1. reprioritize -- plan and README per research4; media paused
   (scene WIP on wip/media-scene); the extension ABI and the bridge
   labelled experimental; promotion-blocker list for this repo.
2. blockers -- fix this repo's blockers (xetal-x version provenance,
   anything a first user trips on); upstream blockers (E5, E6)
   tracked.
3. audit -- cross-repo ask and status audit: our asks against upstream
   (E3-E6 now filed), vendor refreshed if anything landed, status and
   docs reconciled, walkthrough re-run.
4. xtlm-ready -- Ffi.xtlm designed against MC10-MC13 so the binding
   macro ships the day .xtlm lands (blocked until then).
5. snapshot -- a version for the six-repo compatible snapshot
   (tag with the user's yes).
