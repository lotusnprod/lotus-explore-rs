// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! SPARQL over HTTP transport — the platform-agnostic layer beneath
//! [`crate::sparql`] (LOTUS wrappers) and [`crate::models`] (domain types).
//!
//! Provides a thin HTTP client that POSTs a query string to any SPARQL /
//! `QLever` endpoint, handles retries with exponential backoff,
//! content-negotiated format selection, and gateway-error detection.  It knows
//! nothing about LOTUS, Wikidata, or CSV schema — callers supply the endpoint
//! URL and interpret the returned bytes.
//!
//! # `QLever` CSV export URL format
//!   `https://qlever.dev/api/wikidata?query=<encoded>&action=csv_export`

pub use execute::{
    execute_query, execute_sparql_body, execute_sparql_bytes, execute_sparql_with_format,
    execute_sparql_with_format_body, execute_sparql_with_format_bytes, fetch_export_url_bytes,
    fetch_url_bytes,
};

#[cfg(not(target_arch = "wasm32"))]
pub use execute::{execute_sparql_tempfile, execute_sparql_with_format_tempfile};
pub use types::{
    FetchError, QLEVER_WIKIDATA, ResponseBody, ResponseFormat, WDQS_SCHOLARLY, WDQS_WIKIDATA,
};

// CSV / string helpers are re-exported through `csv::*` since they were all
// public in the original transport module.
pub use csv::{clean_doi, coalesce, col_idx, extract_qid, field, non_empty, parse_year};

mod client;
mod csv;
mod error;
mod execute;
mod types;

#[cfg(test)]
mod tests;
