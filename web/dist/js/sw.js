const CACHE_NAME = "biblers-cache-v4";

// Use cache before fetching from the network.
self.addEventListener("fetch", (e) => {
    // Search API requests are not locally cached.
    if (e.request.url.indexOf("/api/search") > -1) {
        e.respondWith(fetch(e.request));

    // Standard pages are cached.
    } else {
        e.respondWith(
            caches.match(e.request).then(initialResp =>
                initialResp || fetch(e.request).then(resp =>
                    caches.open(CACHE_NAME).then(cache => {
                        cache.put(e.request, resp.clone());
                        return resp;
                    })
                )
            ),
        )
    }
});

// Install static files.
self.addEventListener("install", (e) => {
    e.waitUntil(
        caches.open(CACHE_NAME).then(cache => cache.addAll([
            "/",
            "/static/manifest.json",
            "/static/css/style.css",
            "/static/img/arrow-back.svg",
            "/static/img/arrow-forward.svg",
            "/static/img/bible.rs.svg",
            "/static/img/book.svg",
            "/static/img/info.svg",
            "/static/img/unfold-more.svg",
            "/static/js/autocomplete.min.js",
            "/static/js/main.js",
        ])),
    );
});

// Clear the old caches.
self.addEventListener("activate", (e) => {
    e.waitUntil(
        caches.keys().then((keyList) =>
            Promise.all(keyList.map((k) => {
                if (CACHE_NAME.indexOf(k) === -1) {
                    return caches.delete(k);
                }
            }))
        ),
    );
});
