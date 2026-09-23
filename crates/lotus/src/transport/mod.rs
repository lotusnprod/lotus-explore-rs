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
    fetch_url_bytes,
};

#[cfg(not(target_arch = "wasm32"))]
pub use execute::execute_sparql_tempfile;
pub use types::{
    FetchError, QLEVER_WIKIDATA, ResponseBody, ResponseFormat, WDQS_SCHOLARLY, WDQS_WIKIDATA,
};

// CSV / string helpers: `field`/`non_empty` are used downstream;
// `col_idx`/`extract_qid`/`parse_year` are crate-internal parsing details.
pub(crate) use csv::{col_idx, extract_qid, parse_year};
pub use csv::{field, non_empty};

mod client;
mod csv;
mod error;
mod execute;
mod types;

#[cfg(test)]
mod tests;
