const CITATION_JS_SRC = "https://tools-static.wmflabs.org/scholia/js/citation.js";
let citationJsLoadPromise = null;

function loadCitationJs() {
    if (typeof globalThis.__Cite === "function") {
        return Promise.resolve(globalThis.__Cite);
    }
    if (citationJsLoadPromise) {
        return citationJsLoadPromise;
    }

    citationJsLoadPromise = new Promise((resolve, reject) => {
        const existing = document.querySelector(`script[src="${CITATION_JS_SRC}"]`);
        const complete = () => {
            try {
                let CiteCtor = null;
                if (typeof globalThis.Cite === "function") {
                    CiteCtor = globalThis.Cite;
                } else if (typeof require === "function") {
                    CiteCtor = require("citation-js").Cite;
                }

                if (typeof CiteCtor === "function") {
                    globalThis.__Cite = CiteCtor;
                    resolve(CiteCtor);
                } else {
                    reject(new Error("citation.js loaded but did not expose a Cite constructor"));
                }
            } catch (_error) {
                reject(new Error("Scholia citation.js bundle failed to expose Cite"));
            }
        };

        if (existing) {
            existing.addEventListener("load", complete, { once: true });
            existing.addEventListener(
                "error",
                () => reject(new Error("citation.js failed to load")),
                { once: true }
            );
            return;
        }

        const script = document.createElement("script");
        script.src = CITATION_JS_SRC;
        script.async = true;
        script.crossOrigin = "anonymous";
        script.addEventListener("load", complete, { once: true });
        script.addEventListener(
            "error",
            () => reject(new Error("citation.js failed to load")),
            { once: true }
        );
        document.head.appendChild(script);
    }).catch((error) => {
        citationJsLoadPromise = null;
        throw error;
    });

    return citationJsLoadPromise;
}

async function resolveCitationBridge() {
    if (typeof globalThis.__Cite === "function") {
        return globalThis.__Cite;
    }
    return await loadCitationJs();
}

async function fetchDoiCslJson(doi) {
    const response = await fetch(`https://doi.org/${encodeURIComponent(doi)}`, {
        headers: {
            Accept: "application/vnd.citationstyles.csl+json, application/json;q=0.9",
        },
    });
    if (!response.ok) {
        throw new Error(`DOI metadata fetch failed: HTTP ${response.status}`);
    }
    return await response.json();
}

window.__lotusCitation = {
    async quickStatements(doi) {
        const trimmed = String(doi || "").trim();
        if (!trimmed) {
            return "";
        }
        const CiteCtor = await resolveCitationBridge();
        const cslJson = await fetchDoiCslJson(trimmed);
        let cite;
        try {
            cite = new CiteCtor(cslJson);
        } catch (_ctorError) {
            cite = await CiteCtor.async(cslJson);
        }
        const output = cite.format('quickstatements');
        return typeof output === "string" ? output : "";
    },
};