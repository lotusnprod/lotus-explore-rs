// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! SPARQL query-string builders for the `LOTUS`/`Wikidata`/`QLever` ecosystem.
//!
//! This module is the single source of truth for all SPARQL query construction
//! in the workspace.  `lotus-explore-rs` imports from here rather than
//! building query strings inline.
//!
//! ## Submodule layout
//!
//! | Submodule   | Responsibility                                              |
//! |-------------|-------------------------------------------------------------|
//! | `consts`     | Shared SPARQL prefix blocks and Wikidata property fragments |
//! | `compound`   | Compound / taxon lookup queries (`query_all_compounds`, etc.)|
//! | `sachem`     | Structure similarity & substructure search via IDSM        |
//! | `structure`  | Structure-string classification & escaping               |
//! | `pagination` | Pagination helpers (LIMIT, COUNT)                       |
//! | `filters`    | Server-side filter injection (mass, year, formula)          |
//! | `rdf`        | CONSTRUCT query generation for Turtle/RDF export            |
//! | `formula`    | Subscript-digit normalization and formula BIND expressions  |

mod compound;
mod consts;
mod filters;
mod formula;
mod pagination;
mod rdf;
mod sachem;
mod structure;

// ── Re-exports (public API unchanged) ────────────────────────────────────────

pub use compound::{query_all_compounds, query_compounds_by_taxon, query_taxon_search};
pub use consts::transform_query_for_wdqs;
pub use filters::query_with_server_filters;
pub use pagination::{query_counts_from_base, query_with_limit};
pub use rdf::query_construct_from_select;
pub use sachem::{query_sachem, query_sachem_batch};
pub use structure::{StructureKind, classify_structure, escape_structure_literal};

// ── Test module ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests;
