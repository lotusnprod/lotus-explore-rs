// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Application service layer for the Explore feature.
//!
//! Each module is a pure use-case function with no dependency on the Dioxus
//! runtime, making every unit independently testable without a virtual DOM.
//!
//! | Module            | Responsibility                                      |
//! |-------------------|-----------------------------------------------------|
//! | `api_pipeline`    | Execute the REST API fast path and map it to domain outcome |
//! | `strategy`        | Select execution path (API-first / SPARQL / DL-only)|
//! | `resolve_taxon`   | Resolve a free-text taxon name to a Wikidata QID    |
//! | `build_query`     | Build the SPARQL query from criteria + QID          |
//! | `fetch_results`   | Fetch full results once and derive capped table rows |
//! | `results_pipeline`| Orchestrate taxon resolution, query build, and results fetch |
//! | `finalize`        | Assemble hashes, metadata JSON, and stats           |

pub mod api_pipeline;
pub mod build_query;
pub mod fetch_results;
pub mod finalize;
pub mod resolve_taxon;
pub mod results_pipeline;
pub mod strategy;
