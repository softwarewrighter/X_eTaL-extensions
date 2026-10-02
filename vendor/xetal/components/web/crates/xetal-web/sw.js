// The live demo's service worker: the app keeps working offline.
// Network first: every request goes to the network, and what comes
// back is kept; with no network, the kept copy is used. So a new
// deploy is picked up at once when online (trunk names its wasm and
// css by their hash, and index.html is always fetched fresh), and the
// last version opened still runs offline.
const CACHE = "xetal-live";

self.addEventListener("install", () => self.skipWaiting());

self.addEventListener("activate", (event) => {
  event.waitUntil(self.clients.claim());
});

self.addEventListener("fetch", (event) => {
  if (event.request.method !== "GET") return;
  event.respondWith(
    fetch(event.request)
      .then((response) => {
        if (response.ok) {
          const copy = response.clone();
          caches.open(CACHE).then((cache) => cache.put(event.request, copy));
        }
        return response;
      })
      .catch(() => caches.match(event.request).then((kept) => kept || Response.error())),
  );
});
