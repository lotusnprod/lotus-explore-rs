// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Core transport types: response body, content-negotiation format, and errors.
//!
//! These are re-exported at the [`crate::transport`] module root.

/// Type alias for the raw response body bytes returned by the transport layer.
pub type ResponseBody = bytes::Bytes;

/// Default `QLever` endpoint for Wikidata (used by lotus-explore-rs).
pub const QLEVER_WIKIDATA: &str = "https://qlever.dev/api/wikidata";
/// Wikidata Query Service endpoint, used as a fallback when `QLever` returns a
/// gateway error (502) — see [`crate::transport`] module docs.
pub const WDQS_WIKIDATA: &str = "https://query.wikidata.org/sparql";
/// Scholarly subgraph endpoint for reference metadata (P1476, P356, P577).
pub const WDQS_SCHOLARLY: &str = "https://query-scholarly.wikidata.org/sparql";
/// Maximum number of HTTP retry attempts before giving up.
pub(super) const MAX_HTTP_ATTEMPTS: u32 = 2;

/// Content-negotiation format for SPARQL responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseFormat {
    /// `text/csv` — used for bulk result-set download.
    Csv,
    /// `application/sparql-results+json` — structured JSON results.
    SparqlJson,
    /// `text/turtle` — RDF triples.
    Turtle,
    /// `application/n-triples` — RDF triples, one per line.
    NTriples,
}

impl ResponseFormat {
    /// Returns the `Accept` header value for this format.
    pub(super) const fn accept(self) -> &'static str {
        match self {
            Self::Csv => "text/csv",
            Self::SparqlJson => "application/sparql-results+json",
            Self::Turtle => "text/turtle",
            Self::NTriples => "application/n-triples",
        }
    }

    /// Returns the `QLever` `action=` parameter for this format, if any.
    pub(super) const fn action(self) -> Option<&'static str> {
        match self {
            Self::Csv => Some("csv_export"),
            Self::SparqlJson => Some("sparql_json_export"),
            Self::Turtle => Some("turtle_export"),
            Self::NTriples => None,
        }
    }
}

/// Error type for SPARQL-over-HTTP fetch operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchError {
    /// Network-level failure (DNS, timeout, connection refused).
    Network(String),
    /// HTTP response with a non-2xx status code.
    Http(u16, String),
    /// Response body could not be parsed.
    Parse(String),
    /// Response body was empty.
    Empty,
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(e) => write!(f, "Network error: {e}"),
            Self::Http(s, msg) => write!(f, "HTTP {s}: {msg}"),
            Self::Parse(e) => write!(f, "Parse error: {e}"),
            Self::Empty => write!(f, "Query returned no results"),
        }
    }
}
