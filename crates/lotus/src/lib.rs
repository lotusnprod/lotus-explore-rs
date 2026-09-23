// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! # lotus — LOTUS domain & SPARQL shared core
//!
//! The single source of truth for everything LOTUS/Wikidata/QLever across the
//! `lotus-explore-rs` repository.  The `lotus-explore-rs` app — serving both its native
//! `/v1` API (under `--features server`) and its WASM client — and any future app
//! consume this crate rather than constructing queries or parsing CSV results themselves.
//!
//! ## Module map
//!
//! | Module      | Responsibility                                                |
//! |-------------|--------------------------------------------------------------|
//! | transport   | Platform-agnostic SPARQL-over-HTTP: retries, content-negotiation, gateway-error detection. Accepts any endpoint URL. |
//! | models      | LOTUS domain types: `SearchCriteria`, `CompoundEntry`, `DatasetStats`, `TaxonMatch`, sort state, element constants. |
//! | queries     | SPARQL query-string builders — `query_all_compounds`, `query_sachem`, `query_with_server_filters`, etc. No I/O. |
//! | sparql      | LOTUS-specific wrappers combining transport + models: execute against the default QLever/Wikidata endpoint, parse CSV result sets into typed rows. |
//!
//! ## Design non-goals
//!
//! - File upload, blob streaming, or progress reporting → see the `upload` crate.
//! - UI components or styling → see the `ui` crate.
//! - Application routing, Dioxus state machines, or i18n → these live in each app.

#![warn(missing_docs)]
// reqwest-wasm futures are inherently `!Send` (single-threaded browser runtime);
// this is correct, not a defect. Suppress it on the wasm target only.
#![cfg_attr(target_arch = "wasm32", allow(clippy::future_not_send))]

/// Export-format and download-URL helpers used by `lotus-explore-rs`.
pub mod export;
/// LOTUS domain models: search criteria, compound entries, dataset stats.
pub mod models;
/// SPARQL query-string builders — no I/O.
pub mod queries;
/// LOTUS-specific wrappers combining transport + models: execute queries, parse CSV results.
pub mod sparql;
/// Cache-key computation shared by the native server and the WASM client.
pub mod state;
/// Platform-agnostic SPARQL-over-HTTP transport.
pub mod transport;
