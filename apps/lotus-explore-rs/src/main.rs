// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

// This crate compiles Dioxus WASM-client code alongside native-server code in
// a single compilation unit.  On native targets (no `server` feature) main()
// just prints a hint and never launches the Dioxus renderer, so all UI/i18n
// modules are technically unreachable — hence `allow(dead_code)`. Without it
// rustc flags ~980 Dioxus RSX component/hook sites as dead (false positives the
// reachability analyzer can't trace through macro expansion).
#![allow(dead_code)]
#![cfg_attr(target_arch = "wasm32", allow(clippy::future_not_send))]

//! `lotus-explore-rs` — LOTUS Knowledge Explorer.
//!
//! A linked open data (LOAD) explorer for the LOTUS compound-taxon-reference
//! knowledge graph from Wikidata, queried via SPARQL.  Powered by the `lotus`
//! shared crate and the QLever SPARQL endpoint.
//!
//! # Quick start
//!
//! ```bash
//! dx serve --package lotus-explore-rs
//! ```
//!
//! The LOTUS API server is built into this package. Run it natively and serve
//! the WASM client in parallel:
//!
//! ```bash
//! cargo run --locked --features server -p lotus-explore-rs     # API on :8787
//! dx serve --package lotus-explore-rs --platform web           # client on :8080
//! ```
//!
//! The dev server proxies `/v1` requests to `http://127.0.0.1:8787`.
//! Without the server running, the explorer falls back to direct QLever/SPARQL.
//!
//! # Architecture
//!
//! See [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md) for the full architectural
//! overview.
//!
//! # Engineering skills
//!
//! - [`SKILLS.md`](./SKILLS.md)
//! - [`docs/skills/SUGGESTIONS.md`](./docs/skills/SUGGESTIONS.md)
//!
//! # Curation share links
//!
//! - [`docs/CURATION_SHARE_LINKS.md`](./docs/CURATION_SHARE_LINKS.md)
//!
//! # Development testing
//!
//! Run logging format tests during telemetry work:
//!
//! ```bash
//! cargo test --locked -p lotus-explore-rs utils::logging::tests
//! ```
//!
//! # Setup: external assets
//!
//! RDKit.js and citation.js are loaded from CDN on demand by the curation
//! workflow when their respective operations first need them (no local
//! download needed). The @citation-js/plugin-quickstatements output formatter
//! is registered inline with citation.js after it loads.
//! All document `<head>` metadata, scripts, and styles are managed in Rust
//! via `ui::document::DocumentHead` — see `src/document_head.rs`.
//!
//! Ketcher (115 MB) must be fetched before serving or deploying:
//!
//! ```bash
//! cargo run --release -p lotus-deploy --bin fetch-ketcher
//! ```
//!
//! # Citation
//!
//! - Paper (DOI): <https://doi.org/10.7554/eLife.70780>
//! - BibTeX: [`public/docs/references.bib`](./public/docs/references.bib)
//!
//! # Site metadata
//!
//! `public/llms.txt`, `public/humans.txt`, `public/robots.txt`,
//! `public/.well-known/security.txt`, `public/_headers`, and
//! `public/site.webmanifest` are generated from
//! [`metadata/site-metadata.json`](./metadata/site-metadata.json).
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
mod core;
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
         or build the WASM client with `dx serve --platform web`."
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
