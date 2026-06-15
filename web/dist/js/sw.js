const VERSION = "v9";
const PRECACHE = "biblers-precache-" + VERSION;
const RUNTIME_HTML = "biblers-html-" + VERSION;
const RUNTIME_STATIC = "biblers-static-" + VERSION;

const PRECACHE_URLS = [
    "/",
    "/about",
    "/static/manifest.json",
    "/static/css/layers.css",
    "/static/css/variables.css",
    "/static/css/reset.css",
    "/static/css/global.css",
    "/static/css/autocomplete.css",
    "/static/css/pages/about.css",
    "/static/css/pages/bible.css",
    "/static/css/pages/book.css",
    "/static/css/pages/chapter.css",
    "/static/css/pages/error.css",
    "/static/css/pages/search-results.css",
    "/static/fonts/literata-latin.woff2",
    "/static/img/arrow-back.svg",
    "/static/img/arrow-forward.svg",
    "/static/img/bible.rs-32x32.png",
    "/static/img/bible.rs-192x192.png",
    "/static/img/bible.rs-512x512.png",
    "/static/img/bible.rs-maskable.svg",
    "/static/img/bible.rs.svg",
    "/static/img/book.svg",
    "/static/img/info.svg",
    "/static/img/unfold-more.svg",
    "/static/js/autocomplete.min.js",
    "/static/js/main.js",
];

const SWR_HTML_PATHS = new Set(["/", "/about"]);
const HTML_CACHE_LIMIT = 50;

self.addEventListener("install", (event) => {
    event.waitUntil(
        caches.open(PRECACHE)
            .then((cache) => cache.addAll(PRECACHE_URLS))
            .then(() => self.skipWaiting())
    );
});

self.addEventListener("activate", (event) => {
    event.waitUntil(
        caches.keys().then((names) =>
            Promise.all(
                names
                    .filter((n) => ![PRECACHE, RUNTIME_HTML, RUNTIME_STATIC].includes(n))
                    .map((n) => caches.delete(n))
            )
        ).then(() => self.clients.claim())
    );
});

function isHtmlRequest(request) {
    if (request.mode === "navigate") return true;
    const accept = request.headers.get("accept") || "";
    return accept.includes("text/html");
}

function trimCache(cacheName, max) {
    caches.open(cacheName).then((cache) =>
        cache.keys().then((keys) => {
            if (keys.length <= max) return;
            cache.delete(keys[0]).then(() => trimCache(cacheName, max));
        })
    );
}

async function staleWhileRevalidate(request) {
    const cache = await caches.open(RUNTIME_HTML);
    const cached = await cache.match(request);
    const network = fetch(request).then((response) => {
        if (response && response.ok) {
            cache.put(request, response.clone());
        }
        return response;
    }).catch(() => null);
    return cached || network;
}

async function networkFirstHtml(request) {
    try {
        const response = await fetch(request);
        if (response && response.ok) {
            const cache = await caches.open(RUNTIME_HTML);
            cache.put(request, response.clone());
            trimCache(RUNTIME_HTML, HTML_CACHE_LIMIT);
        }
        return response;
    } catch (e) {
        const cache = await caches.open(RUNTIME_HTML);
        const cached = await cache.match(request);
        if (cached) return cached;
        const precached = await caches.match("/");
        if (precached) return precached;
        throw e;
    }
}

async function cacheFirstStatic(request) {
    const cached = await caches.match(request);
    if (cached) return cached;
    try {
        const response = await fetch(request);
        if (response && response.ok) {
            const cache = await caches.open(RUNTIME_STATIC);
            cache.put(request, response.clone());
        }
        return response;
    } catch (e) {
        return Response.error();
    }
}

self.addEventListener("fetch", (event) => {
    const request = event.request;
    if (request.method !== "GET") return;

    const url = new URL(request.url);
    if (url.origin !== self.location.origin) return;

    // Search API: always live, never cached.
    if (url.pathname.startsWith("/api/")) {
        event.respondWith(fetch(request));
        return;
    }

    // Static assets — cache-first, busted by VERSION-bumped cache names.
    if (url.pathname.startsWith("/static/")) {
        event.respondWith(cacheFirstStatic(request));
        return;
    }

    // HTML navigation.
    if (isHtmlRequest(request)) {
        if (SWR_HTML_PATHS.has(url.pathname)) {
            event.respondWith(staleWhileRevalidate(request));
        } else {
            event.respondWith(networkFirstHtml(request));
        }
        return;
    }
});
