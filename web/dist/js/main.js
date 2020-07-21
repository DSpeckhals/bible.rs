// Search box
(function () {
    function getResults(q, cb) {
        if (!q) {
            cb([]);
            return;
        }

        fetch("/api/search?q=" + encodeURIComponent(q)).then(function (resp) {
            return resp.json();
        }).then(function (data) {
            cb(data.matches);
        });
    }

    autocomplete("#q", { autoselect: true, debounce: 350, hint: false }, [
        {
            source: getResults,
            templates: {
                suggestion: function (result) {
                    return "<p><i>" + result.link.label + "</i> | " + result.text + "</p>";
                }
            }
        }
    ]).on("autocomplete:selected", function (e, suggestion) {
        document.location.href = suggestion.link.url;
    });
    document.getElementById("q").setAttribute("aria-label", "Search the Bible");

    document.onkeypress = function (e) {
        if (e.ctrlKey || e.altKey || e.metaKey) {
            return;
        }
        var key = String.fromCharCode(e.charCode || e.keyCode);
        if (document.activeElement.tagName !== "INPUT" && key.toLowerCase() === "s") {
            e.preventDefault();
            document.getElementById("q").focus();
        }
    };

    document.getElementById("search-form").onsubmit = function(e) {
        e.preventDefault();
        return false;
    };
})();

// Swipe navigation
(function () {
    function detectSwipe(el, callback) {
        var touchSurface = el;
        var swipeDir;
        var startX;
        var startY;
        var distX;
        var distY;
        var threshold = 50;
        var restraint = 50;
        var allowedTime = 300;
        var elapsedTime;
        var startTime;
        var handleSwipe = callback || function () {};

        touchSurface.addEventListener("touchstart", function (e) {
            var touchObj = e.changedTouches[0];
            swipeDir = "none";
            distX = 0;
            distY = 0;
            startX = touchObj.pageX;
            startY = touchObj.pageY;
            startTime = new Date().getTime();
        }, false);

        touchSurface.addEventListener("touchend", function (e) {
            var touchObj = e.changedTouches[0];
            distX = touchObj.pageX - startX;
            distY = touchObj.pageY - startY;
            elapsedTime = new Date().getTime() - startTime;

            if (elapsedTime <= allowedTime) {
                if (Math.abs(distX) >= threshold && Math.abs(distY) <= restraint) {
                    swipeDir = (distX < 0)? "left" : "right";
                }
                else if (Math.abs(distY) >= threshold && Math.abs(distX) <= restraint) {
                    swipeDir = (distY < 0)? "up" : "down";
                }
            }
            handleSwipe(swipeDir);
        }, false);
    }

    var el = document.getElementsByTagName("main")[0];
    var prevEl = document.getElementById("link-prev");
    var nextEl = document.getElementById("link-next");
    if (prevEl || nextEl) {
        detectSwipe(el, function (swipeDir) {
            if (swipeDir == "right" && prevEl) {
                window.location.href = prevEl.getAttribute("href");
            } else if (swipeDir == "left" && nextEl) {
                window.location.href = nextEl.getAttribute("href");
            }
        });
    }
})();

// Service worker registration
if (navigator.serviceWorker) {
    navigator.serviceWorker.register("/static/js/sw.js", { scope: "/" });
}
