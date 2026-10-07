# fetch

Fetch and analyze (docs/plan.md, Saga 11): an http extension (bounded
GET with ureq), a digest extension (SHA-256, CRC-32), and the quakes
demo: the USGS earthquake feed into SQLite, analyzed and drawn by
X_eTaL. Network only on request: loopback tests, a saved copy for the
golden, `just live-quakes` for the live feed. Photo lab follows.

## Steps

1. http -- bounded GET; tests against web on loopback.
2. digest -- SHA-256 and CRC-32 of text and files.
3. quakes -- the demo, a saved copy for the golden, just live-quakes.
4. fetch-release -- recordings, docs, status, retrospective, site checked.
