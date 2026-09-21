// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Domain models for the LOTUS explorer/API shared core.
//!
//! ## Linked Open Data / Wikidata
//!
//! All entity identifiers in the LOTUS dataset follow the Wikidata entity URI
//! scheme.  The canonical prefix is `WIKIDATA_ENTITY_BASE`.  Statement
//! identifiers use `WIKIDATA_STATEMENT_BASE`.  These constants are
//! re-exported here so every layer (DTO deserialization, SPARQL parsing, UI
//! display) uses a single authoritative value.

#![allow(missing_docs)] // data-heavy module; fields are self-documenting from struct names

mod entry;
mod runtime;
mod search;
mod sort;
mod stats;

#[cfg(test)]
mod tests;

pub use entry::{CompoundEntry, Rows, TaxonMatch};
pub use runtime::{CURRENT_YEAR_CACHE, current_year, runtime_table_row_limit};
pub use search::SearchCriteria;
pub use sort::{SortColumn, SortDir, SortState};
pub use stats::{DatasetStats, ElementState, SmilesSearchType};

// ── Constants ────────────────────────────────────────────────────────────────

/// Base URI for Wikidata entities (e.g. `Q12345` → `<BASE>Q12345`).
pub const WIKIDATA_ENTITY_BASE: &str = "http://www.wikidata.org/entity/";

/// Base URI for Wikidata reification statements.
pub const WIKIDATA_STATEMENT_BASE: &str = "http://www.wikidata.org/entity/statement/";

/// Maximum table rows on WASM (conservative) vs native (large).
#[cfg(target_arch = "wasm32")]
pub const TABLE_ROW_LIMIT: usize = 1_000;
#[cfg(not(target_arch = "wasm32"))]
pub const TABLE_ROW_LIMIT: usize = 2_000_000;

/// Maximum element counts for molecular formula validation.
pub const DEFAULT_C_MAX: u16 = 512;
pub const DEFAULT_H_MAX: u16 = 1_024;
pub const DEFAULT_N_MAX: u16 = 256;
pub const DEFAULT_O_MAX: u16 = 256;
pub const DEFAULT_P_MAX: u16 = 128;
pub const DEFAULT_S_MAX: u16 = 64;

/// Minimum plausible publication year (prevents clock-skew / parsing garbage).
pub const DEFAULT_YEAR_MIN: u16 = 1800;
