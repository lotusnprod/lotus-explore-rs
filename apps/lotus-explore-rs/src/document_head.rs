// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Programmatic document `<head>` management for lotus-explore-rs.

// Dioxus's `asset!` macro resolves to `&[u8]`, which clippy's
// `volatile_composites` flags in every `asset!` call site; the value type is
// fixed by the framework and cannot be made volatile-compatible at the call
// site. The lint is suppressed for the whole module because every flagged
// expression is an `asset!` invocation of exactly this shape.
#![allow(clippy::volatile_composites)]

use crate::ui::document::DocumentHead;
use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
use dioxus::document::document as dioxus_document;

/// Build the absolute base URL (origin + pathname) from the browser's
/// `location` so that every canonical / hreflang link is an absolute URL.
#[cfg(target_arch = "wasm32")]
fn base_url() -> String {
    let Some(win) = web_sys::window() else {
        return String::new();
    };
    let loc = win.location();
    let origin = loc.origin().unwrap_or_default();
    let pathname = loc.pathname().unwrap_or_default();
    format!("{origin}{pathname}")
}

#[cfg(not(target_arch = "wasm32"))]
fn base_url() -> String {
    String::new()
}

/// Build a root-relative URL for a static asset under the served `public/` tree.
///
/// Uses the `<script src>` tag (set by `dx build --base-path`) to detect the base
/// path at runtime, so the Ketcher iframe works on both GitHub Pages (`/<repo>/`)
/// and root domains (`/`).
#[cfg(target_arch = "wasm32")]
pub fn asset_url(path: &str) -> String {
    let path = path.trim_start_matches('/');
    let base = web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.query_selector("script[src]").ok())
        .flatten()
        .and_then(|el| el.get_attribute("src"))
        .and_then(|src| src.find("assets/").map(|pos| src[..pos].to_string()))
        .unwrap_or_else(|| String::from("/"));
    format!("{base}{path}")
}

#[cfg(not(target_arch = "wasm32"))]
pub fn asset_url(_path: &str) -> String {
    String::new()
}

const DESCRIPTION: &str =
    "Explore LOTUS with taxon filters, structure search, and Wikidata curation workflows.";

/// Build `application/ld+json` structured data (schema.org `WebApplication`).
fn json_ld(canonical: &str) -> String {
    let desc = serde_json::to_string(DESCRIPTION).unwrap_or_else(|_| "\"\"".to_string());
    let url = serde_json::to_string(canonical).unwrap_or_else(|_| "\"\"".to_string());
    format!(
        "{{\"@context\":\"https://schema.org\",\"@type\":\"WebApplication\",\"name\":\"LOTUS Knowledge Explorer\",\"description\":{desc},\"url\":{url},\"applicationCategory\":\"ScienceApplication\",\"operatingSystem\":\"Web\",\"inLanguage\":[\"en\",\"fr\",\"de\",\"it\"],\"offers\":{{\"@type\":\"Offer\",\"price\":\"0\",\"priceCurrency\":\"EUR\"}}}}"
    )
}

/// Alternating-language hreflang map: `"en"` → path suffix.
/// English has no suffix because it is the default language.
#[cfg(target_arch = "wasm32")]
const HREF_LANGS: &[(&str, &str)] = &[("en", ""), ("fr", "fr"), ("de", "de"), ("it", "it")];

#[component]
pub fn LotusDocumentHead(lang: String) -> Element {
    let canonical = match lang.as_str() {
        "en" => base_url(),
        other => {
            let base = base_url();
            if base.contains('?') {
                format!("{base}&lang={other}")
            } else {
                format!("{base}?lang={other}")
            }
        }
    };

    // Inject hreflang `<link rel="alternate">` tags with absolute URLs.
    #[cfg(target_arch = "wasm32")]
    use_hook(move || {
        let doc = dioxus_document();
        let base = base_url();
        for (hreflang, suffix) in HREF_LANGS {
            let href = if suffix.is_empty() {
                base.clone()
            } else if base.contains('?') {
                format!("{base}&lang={suffix}")
            } else {
                format!("{base}?lang={suffix}")
            };
            let attrs: Vec<(&str, String)> = vec![
                ("rel", "alternate".to_string()),
                ("href", href),
                ("hreflang", hreflang.to_string()),
            ];
            doc.create_head_element("link", &attrs, None);
        }
    });
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(clippy::redundant_clone)]
    let _ = canonical.clone(); // suppress unused warning in tests

    rsx! {
        DocumentHead {
            title: "LOTUS Explore-rs".to_string(),
            lang,
            description: Some(DESCRIPTION.to_string()),
            og_type: Some("website".to_string()),
            og_url: Some(canonical.clone()),
            og_site_name: Some("LOTUS Explore-rs".to_string()),
            theme_colors: Some(("#f6f8fb", "#10141b")),
            json_ld: Some(json_ld(&canonical)),
            canonical: Some(canonical),
        }

        document::Script { src: asset!("/public/assets/js/bootstrap.js"), defer: true, "type": "text/javascript" }
    }
}

/// Lazily inject the curation bridge JS files into the document `<head>`.
#[component]
pub fn CurationScripts() -> Element {
    rsx! {
        document::Script { src: asset!("/public/assets/js/curation/rdkit-bridge.js"), defer: true, "type": "text/javascript" }
        document::Script { src: asset!("/public/assets/js/curation/citation-bridge.js"), defer: true, "type": "text/javascript" }
    }
}
