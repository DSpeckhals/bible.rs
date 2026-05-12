// Publish the header's measured height as --header-h so the search
// dropdown (which uses position: fixed to break out of the input box)
// can anchor itself just below the sticky header at any viewport size.
(function () {
    var header = document.querySelector(".site-header");
    if (!header) return;
    var publish = function () {
        document.documentElement.style.setProperty(
            "--header-h", header.offsetHeight + "px"
        );
    };
    publish();
    if (typeof ResizeObserver === "function") {
        new ResizeObserver(publish).observe(header);
    } else {
        window.addEventListener("resize", publish);
    }
})();

// Search box with Algolia autocomplete.
// The form posts natively to /search?q=... when no suggestion is chosen,
// so the page works fully with JavaScript disabled.
(function () {
    var input = document.getElementById("q");
    if (!input) return;

    function getResults(q, cb) {
        if (!q) { cb([]); return; }
        fetch("/api/search?q=" + encodeURIComponent(q))
            .then(function (r) { return r.json(); })
            .then(function (data) { cb(data.matches || []); })
            .catch(function () { cb([]); });
    }

    if (typeof autocomplete === "function") {
        autocomplete(input, { autoselect: true, debounce: 350, hint: false }, [{
            source: getResults,
            templates: {
                suggestion: function (result) {
                    return "<p><i>" + result.link.label + "</i> | " + result.text + "</p>";
                }
            }
        }]).on("autocomplete:selected", function (_e, suggestion) {
            window.location.assign(suggestion.link.url);
        });
    }

    // Keyboard shortcut: press "s" anywhere to focus the search box.
    document.addEventListener("keydown", function (e) {
        if (e.ctrlKey || e.altKey || e.metaKey) return;
        if (document.activeElement && document.activeElement.tagName === "INPUT") return;
        if (e.key && e.key.toLowerCase() === "s") {
            e.preventDefault();
            input.focus();
        }
    });
})();

// Theme toggle: cycles auto → light → dark, persists in localStorage,
// updates aria state and the visible label. The FOUC-prevention bootstrap
// already ran inline in <head>; this module only handles user interaction.
(function () {
    var btn = document.getElementById("theme-toggle");
    if (!btn) return;
    var icon = btn.querySelector(".theme-toggle__icon");
    var label = btn.querySelector(".theme-toggle__label");

    var ICONS = { auto: "☼", light: "☀", dark: "☾" };
    var LABELS = { auto: "Auto", light: "Light", dark: "Dark" };
    var ORDER = ["auto", "light", "dark"];

    function read() {
        try {
            var v = localStorage.getItem("theme");
            return (v === "light" || v === "dark") ? v : "auto";
        } catch (e) { return "auto"; }
    }

    function apply(state) {
        if (state === "auto") {
            document.documentElement.removeAttribute("data-theme");
        } else {
            document.documentElement.setAttribute("data-theme", state);
        }
        try {
            if (state === "auto") localStorage.removeItem("theme");
            else localStorage.setItem("theme", state);
        } catch (e) { /* ignore */ }
        if (icon) icon.textContent = ICONS[state];
        if (label) label.textContent = LABELS[state];
        btn.setAttribute("aria-pressed", state === "dark" ? "true" : "false");
        btn.setAttribute("aria-label", "Theme: " + LABELS[state] + ". Click to change.");
    }

    apply(read());

    btn.addEventListener("click", function () {
        var cur = read();
        var next = ORDER[(ORDER.indexOf(cur) + 1) % ORDER.length];
        apply(next);
    });
})();

// Service worker registration.
if ("serviceWorker" in navigator) {
    window.addEventListener("load", function () {
        navigator.serviceWorker.register("/static/js/sw.js", { scope: "/" });
    });
}
