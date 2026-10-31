const CACHE = "bcc-images";

self.addEventListener("install", () => self.skipWaiting());
self.addEventListener("activate", (event) => event.waitUntil(self.clients.claim()));
self.addEventListener("fetch", (event) => {
  const url = new URL(event.request.url);

  if (url.origin !== self.location.origin || !url.pathname.startsWith("/images/")) {
    return;
  }

  event.respondWith(
    caches.open(CACHE).then((cache) => cache.match(event.request).then((hit) => hit || new Response("", { status: 404 })))
  );
});
