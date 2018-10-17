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
        },
    ]).on("autocomplete:selected", function (e, suggestion) {
        document.location.href = suggestion.link.url;
    });

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

    navigator.serviceWorker && navigator.serviceWorker.register("/static/js/sw.js", { scope: "/" });
})();
