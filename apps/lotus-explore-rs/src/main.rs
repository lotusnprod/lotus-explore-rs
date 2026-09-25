// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

// This crate compiles Dioxus WASM-client code alongside native-server code in
// a single compilation unit.  On native targets (no `server` feature) main()
// just prints a hint and never launches the Dioxus renderer, so all UI/i18n
// modules are technically unreachable. The following lints are allowed for this
// Dioxus cross-cfg situation: dead_code, unreachable_pub.
// `missing_const_for_fn` (nursery) is allowed crate-wide: it fires ~64× across
// UI/i18n locale-dispatch code where const-ness has no material benefit (the
// dispatchers cannot be `const` without const-cascading into all four locale
// table files, and UI helpers run at runtime only).
// NOTE: `clippy::module_name_repetitions` is deliberately NOT allowed here;
// the few `App*`/`Export*` names that need it carry item-level allows instead.
#![cfg_attr(target_arch = "wasm32", allow(clippy::future_not_send))]
// `tower` serves `server`-feature tests only (`ServiceExt::oneshot`), but Cargo
// has no feature-gated dev-dependencies, so `unused_crate_dependencies`
// false-positives on default-features builds (the canonical `just clippy`
// invocation). The lint stays enabled workspace-wide and remains effective
// for the feature-less `lotus`/`lotus-deploy` crates.
#![allow(unused_crate_dependencies)]
#![allow(dead_code, unreachable_pub, clippy::missing_const_for_fn)]

//! `lotus-explore-rs` — LOTUS Explorer.
//!
//! A linked open data (LOD) explorer for the LOTUS compound-taxon-reference
//! knowledge graph from Wikidata, queried via SPARQL.  Powered by the `lotus`
//! shared crate and the `QLever` SPARQL endpoint.
//!
//! # Quick start
//!
//! ```bash
//! just serve
//! ```
//!
//! The LOTUS API server is built into this package. Run it natively and serve
//! the WASM client in parallel:
//!
//! ```bash
//! cargo run --locked --features server -p lotus-explore-rs     # API on :8787
//! just serve                                           # client on :8080
//! ```
//!
//! The dev server proxies `/v1` requests to `http://127.0.0.1:8787`.
//! Without the server running, the explorer falls back to direct QLever/SPARQL.
//!
//! # Architecture
//!
//! See [`docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md) for the full architectural
//! overview.
//!
//! # Curation share links
//!
//! - [`docs/CURATION_SHARE_LINKS.md`](../docs/CURATION_SHARE_LINKS.md)
//!
//! # Setup: external assets
//!
//! RDKit.js and the Scholia Citation.js bundle are fetched into
//! `public/assets/vendor` before serving or deploying. Their source refs are
//! configurable with `RDKIT_VERSION` and `CITATION_JS_REF`; both default to
//! the latest available ref. The curation bridges load the files on demand
//! when their respective operations first need them. The
//! `@citation-js/plugin-quickstatements` output formatter is registered inline
//! with citation.js after it loads.
//! The initial document metadata is defined in `index.html`; route-specific
//! curation scripts are added from `src/document_head.rs`.
//!
//! The `just` recipes fetch these assets automatically. To fetch them directly:
//!
//! ```bash
//! cd apps/lotus-explore-rs && cargo run --release -p lotus-deploy --bin fetch-assets
//! ```
//!
//! # Citation
//!
//! - Paper (DOI): <https://doi.org/10.7554/eLife.70780>
//! - BibTeX: [`public/docs/references.bib`](../public/docs/references.bib)
//!
//! # Site metadata
//!
//! `public/llms.txt`, `public/humans.txt`, `public/robots.txt`,
//! `public/.well-known/security.txt`, `public/_headers`, and
//! `public/site.webmanifest` are generated from
//! [`metadata/site-metadata.json`](../metadata/site-metadata.json).
//!
//! # Explorer ⇄ API integration
//!
//! | Scenario                | `api_base` source                     | API used            |
//! | ----------------------- | ------------------------------------- | ------------------- |
//! | Codeberg Pages (public) | none                                  | ✗ direct SPARQL     |
//! | Local dev               | auto-detected `http://127.0.0.1:8787` | ✓ if server running |
//! | Build-time              | `LOTUS_API_BASE` env var              | ✓                   |
//! | Runtime override        | `?api_base=…` query param             | ✓                   |
//!
//! # URL automation
//!
//! URL-driven execution and exports:
//!
//! - `?execute=true` --- run query on load
//! - `?download=true&format=csv` --- download CSV
//! - `?download=true&format=json` --- download SPARQL Results JSON
//! - `?download=true&format=rdf` --- download RDF (Turtle)
//!
//! When both `download` and `execute` are present, `download` takes priority.
//!
//! # Archive
//!
//! A frozen version is archived on Zenodo: <https://doi.org/10.5281/zenodo.5794106>

#![allow(non_snake_case)] // Dioxus PascalCase component naming convention

mod api;
mod app;
mod app_state;
/// In-browser result cache (mirrors the native server's result cache).
#[cfg(any(test, target_arch = "wasm32"))]
mod cache;
mod components;
mod curation;
mod document_head;
mod download;
mod export;
mod features;
mod hooks;
mod i18n;
mod models;
mod pages;
mod perf;
mod queries;
mod repositories;
mod services;
mod sparql;
mod state;
mod ui;
mod upload;
mod utils;

#[cfg(all(feature = "server", not(target_arch = "wasm32")))]
mod server;

#[cfg(target_arch = "wasm32")]
use dioxus::prelude::*;

#[cfg(test)]
mod tests;

#[cfg(all(not(target_arch = "wasm32"), feature = "server"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    server::run().await
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "server")))]
fn main() {
    eprintln!(
        "lotus-explore-rs (native): enable the `server` feature to host the API \
         (cargo run --features server -p lotus-explore-rs), \
         or build the WASM client with `just serve`."
    );
}

#[cfg(target_arch = "wasm32")]
fn main() {
    let level = if cfg!(debug_assertions) {
        log::Level::Debug
    } else {
        log::Level::Info
    };
    console_log::init_with_level(level).ok();
    launch(app::shell::AppRoot);
}
