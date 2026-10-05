# web

X_eTaL on the web (docs/plan.md, Saga 9; plan A15): a web extension
keeps axum on a background thread with a bounded request queue; the
X_eTaL program pulls the next request and posts its reply, so the
program is the request loop. Loopback only in tests; port 8470.

## Steps

1. pages-fresh -- the gate fails when pages/ is out of date.
2. web -- the extension, Rust tests with a real client on loopback.
3. live-page -- a page recomputing Life or Mandelbrot as SVG per request.
4. todomvc -- TodoMVC in X_eTaL with sqlite.
5. web-release -- recordings, docs, status, retrospective, site checked online.
