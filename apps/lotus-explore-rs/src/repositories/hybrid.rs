// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! [`HybridRepository`] — the production `LotusRepository` implementation.
//!
//! Strategy:
//! 1. If a REST API base URL is configured, try `api::search` first
//!    (gives exact counts + query in one round-trip).
//! 2. On API error or when not configured, return `None` / `Some(Err(…))`
//!    so the caller falls back to direct SPARQL execution.
//! 3. Direct SPARQL execution targets `QLever` (`QLEVER_WIKIDATA`) by
//!    default. If `QLever` answers with a 502 Bad Gateway — a known
//!    transient failure mode on the public instance — the same query is
//!    immediately re-sent to the Wikidata Query Service (`WDQS_WIKIDATA`)
//!    as a fallback, using the scholarly subgraph for reference metadata,
//!    rather than surfacing the gateway error to the user.

use crate::api;
use crate::api::SearchResponse;
use crate::models::SearchCriteria;
use crate::repositories::{LotusRepository, RepositoryError};
use crate::sparql;
use lotus::queries::transform_query_for_wdqs;
use lotus::transport::{self, FetchError, WDQS_WIKIDATA};
use std::cell::RefCell;

/// Single toggle to force WDQS fallback for testing.
/// Set this to `true` to force all queries to use WDQS.
/// Set this to `false` to use QLever (default).
const FORCE_WDQS_FALLBACK: bool = false;

thread_local! {
    /// Tracks whether WDQS fallback was used for the current search.
    /// Reset at the start of each search operation.
    static WDQS_FALLBACK_USED: RefCell<bool> = const { RefCell::new(false) };
    /// Stores the WDQS-transformed query when fallback occurs.
    static WDQS_TRANSFORMED_QUERY: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Check if WDQS fallback was used in the current operation.
pub fn is_wdqs_fallback_used() -> bool {
    WDQS_FALLBACK_USED.with(|b| *b.borrow())
}

/// Get the WDQS-transformed query (if fallback occurred).
/// Strips any trailing LIMIT clause so the stored query is suitable for
/// downloads (which should not be capped at the interactive display limit).
pub fn get_wdqs_transformed_query() -> Option<String> {
    WDQS_TRANSFORMED_QUERY.with(|b| b.borrow().as_ref().map(|q| strip_limit_clause(q)))
}

/// Remove a trailing `LIMIT nnn` (case-insensitive) clause from a query string.
///
/// This is used when retrieving the WDQS-transformed query for downloads —
/// the interactive search adds `LIMIT 500` (from `runtime_table_row_limit()`),
/// but downloads should fetch all results without that display-time cap.
fn strip_limit_clause(query: &str) -> String {
    // Strip a trailing `LIMIT nnn` clause so downloads aren't capped at
    // the interactive display limit (e.g. 500 from runtime_table_row_limit()).
    let trimmed = query.trim_end();
    let lower = trimmed.to_ascii_lowercase();
    // Find the last occurrence of "limit" preceded by whitespace or start of string
    let mut best_pos = None;
    for (i, _) in lower.match_indices("limit") {
        let preceded_ok = i == 0
            || trimmed[..i]
                .chars()
                .last()
                .is_some_and(|c| c.is_whitespace());
        if !preceded_ok {
            continue;
        }
        let after = trimmed[i + 5..].trim_start();
        let num = after.split_whitespace().next().unwrap_or("");
        if !num.is_empty()
            && num.chars().all(|c| c.is_ascii_digit())
            && after[num.len()..].trim().is_empty()
        {
            best_pos = Some(i);
        }
    }
    best_pos.map_or_else(
        || query.to_string(),
        |pos| trimmed[..pos].trim_end().to_string(),
    )
}

/// Reset the WDQS fallback flag at the start of a new operation.
pub fn reset_wdqs_fallback_flag() {
    WDQS_FALLBACK_USED.with(|b| *b.borrow_mut() = false);
    WDQS_TRANSFORMED_QUERY.with(|b| *b.borrow_mut() = None);
}

/// Mark that WDQS fallback was used and store the transformed query.
/// Only stores the first transformed query (results query priority over count query).
fn mark_wdqs_fallback_used(query: String) {
    // Only store the first transformed query (results query priority over count query)
    WDQS_TRANSFORMED_QUERY.with(|transformed| {
        if transformed.borrow().is_none() {
            *transformed.borrow_mut() = Some(query);
        }
    });
    WDQS_FALLBACK_USED.with(|flag| {
        *flag.borrow_mut() = true;
    });
}

/// Zero-size, `Copy` production repository.
///
/// Holds no state of its own; all configuration is read from environment and
/// runtime globals (`api_base_url`, `sparql::execute_sparql_bytes`, etc.).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HybridRepository;

impl LotusRepository for HybridRepository {
    async fn api_search(
        &self,
        criteria: &SearchCriteria,
        limit: usize,
        include_counts: bool,
    ) -> Option<Result<SearchResponse, RepositoryError>> {
        // The API fast-path is opt-in: treat an empty (auto-detected dev) base URL
        // the same as "not configured" so we never build a malformed relative URL.
        if api::api_base_url().is_none_or(|b| b.is_empty()) {
            return Some(Err(RepositoryError::NotConfigured));
        }
        // Call the transport client directly, mapping ApiClientError → RepositoryError
        // via the existing `From` implementation.  Bypassing the ApiLayer / AppError
        // intermediary eliminates a 4-hop conversion chain with no semantic benefit.
        Some(
            api::search(criteria, limit, include_counts)
                .await
                .map_err(RepositoryError::from),
        )
    }

    async fn sparql_bytes(&self, query: &str) -> Result<Vec<u8>, RepositoryError> {
        // Force WDQS fallback for testing if enabled
        if FORCE_WDQS_FALLBACK {
            log::warn!("event=qlever_bad_gateway action=fallback_wdqs_scholarly (FORCED)");
            let wdqs_query = transform_query_for_wdqs(query);
            mark_wdqs_fallback_used(wdqs_query.clone());
            return transport::execute_sparql_bytes(&wdqs_query, WDQS_WIKIDATA)
                .await
                .map_err(map_fetch_error);
        }

        match sparql::execute_sparql_bytes(query).await {
            Err(err) if is_bad_gateway(&err) => {
                log::warn!("event=qlever_bad_gateway action=fallback_wdqs_scholarly");
                let wdqs_query = transform_query_for_wdqs(query);
                mark_wdqs_fallback_used(wdqs_query.clone());
                transport::execute_sparql_bytes(&wdqs_query, WDQS_WIKIDATA)
                    .await
                    .map_err(map_fetch_error)
            }
            result => result.map_err(map_fetch_error),
        }
    }

    async fn sparql_body(
        &self,
        query: &str,
    ) -> Result<lotus::transport::ResponseBody, RepositoryError> {
        // Force WDQS fallback for testing if enabled
        if FORCE_WDQS_FALLBACK {
            log::warn!("event=qlever_bad_gateway action=fallback_wdqs_scholarly (FORCED)");
            let wdqs_query = transform_query_for_wdqs(query);
            mark_wdqs_fallback_used(wdqs_query.clone());
            return transport::execute_sparql_body(&wdqs_query, WDQS_WIKIDATA)
                .await
                .map_err(map_fetch_error);
        }

        match sparql::execute_sparql_body(query).await {
            Err(err) if is_bad_gateway(&err) => {
                log::warn!("event=qlever_bad_gateway action=fallback_wdqs_scholarly");
                let wdqs_query = transform_query_for_wdqs(query);
                mark_wdqs_fallback_used(wdqs_query.clone());
                transport::execute_sparql_body(&wdqs_query, WDQS_WIKIDATA)
                    .await
                    .map_err(map_fetch_error)
            }
            result => result.map_err(map_fetch_error),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn sparql_tempfile(
        &self,
        query: &str,
    ) -> Result<tempfile::NamedTempFile, RepositoryError> {
        // Force WDQS fallback for testing if enabled
        if FORCE_WDQS_FALLBACK {
            log::warn!("event=qlever_bad_gateway action=fallback_wdqs_scholarly (FORCED)");
            let wdqs_query = transform_query_for_wdqs(query);
            mark_wdqs_fallback_used(wdqs_query.clone());
            return transport::execute_sparql_tempfile(&wdqs_query, WDQS_WIKIDATA)
                .await
                .map_err(map_fetch_error);
        }

        match sparql::execute_sparql_tempfile(query).await {
            Err(err) if is_bad_gateway(&err) => {
                log::warn!("event=qlever_bad_gateway action=fallback_wdqs_scholarly");
                let wdqs_query = transform_query_for_wdqs(query);
                mark_wdqs_fallback_used(wdqs_query.clone());
                transport::execute_sparql_tempfile(&wdqs_query, WDQS_WIKIDATA)
                    .await
                    .map_err(map_fetch_error)
            }
            result => result.map_err(map_fetch_error),
        }
    }
}

/// True when `QLever` failed with a 502 Bad Gateway — the signal to retry the
/// same query against the WDQS fallback endpoint instead of surfacing the
/// error. `QLever`'s own transport layer already retries transient network
/// failures and gateway errors internally (see `MAX_HTTP_ATTEMPTS`), so a 502
/// reaching this layer means those in-endpoint retries were exhausted.
fn is_bad_gateway(err: &FetchError) -> bool {
    matches!(err, FetchError::Http(502, _))
}

fn map_fetch_error(err: FetchError) -> RepositoryError {
    match err {
        FetchError::Http(status, body) => RepositoryError::Http { status, body },
        FetchError::Network(msg) => RepositoryError::network(msg),
        FetchError::Parse(msg) => RepositoryError::parse(msg),
        FetchError::Empty => RepositoryError::parse("query returned no results"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_limit_removes_trailing_limit() {
        let q = "SELECT ?s WHERE { ?s ?p ?o } LIMIT 500";
        assert_eq!(strip_limit_clause(q), "SELECT ?s WHERE { ?s ?p ?o }");
    }

    #[test]
    fn strip_limit_preserves_query_without_limit() {
        let q = "SELECT ?s WHERE { ?s ?p ?o }";
        assert_eq!(strip_limit_clause(q), "SELECT ?s WHERE { ?s ?p ?o }");
    }

    #[test]
    fn strip_limit_preserves_inner_limit() {
        let q = "SELECT ?s WHERE { { SELECT ?s WHERE { ?s ?p ?o } LIMIT 10 } LIMIT 500";
        // Should only strip the last LIMIT
        assert_eq!(
            strip_limit_clause(q),
            "SELECT ?s WHERE { { SELECT ?s WHERE { ?s ?p ?o } LIMIT 10 }"
        );
    }

    #[test]
    fn strip_limit_handles_case_insensitive() {
        let q = "SELECT ?s WHERE { ?s ?p ?o } limit 500";
        assert_eq!(strip_limit_clause(q), "SELECT ?s WHERE { ?s ?p ?o }");
    }

    #[test]
    fn strip_limit_handles_newline() {
        let q = "SELECT ?s WHERE { ?s ?p ?o }\nLIMIT 500";
        assert_eq!(strip_limit_clause(q), "SELECT ?s WHERE { ?s ?p ?o }");
    }

    #[test]
    fn map_fetch_error_preserves_http_status_and_body() {
        let mapped = map_fetch_error(FetchError::Http(400, "invalid query".to_string()));
        assert_eq!(
            mapped,
            RepositoryError::Http {
                status: 400,
                body: "invalid query".to_string(),
            }
        );
    }

    #[test]
    fn map_fetch_error_keeps_network_as_network() {
        let mapped = map_fetch_error(FetchError::Network("timeout".to_string()));
        assert!(matches!(mapped, RepositoryError::Network(_)));
    }

    #[test]
    fn is_bad_gateway_matches_only_502() {
        assert!(is_bad_gateway(&FetchError::Http(
            502,
            "upstream gateway error (HTML payload)".into()
        )));
        assert!(!is_bad_gateway(&FetchError::Http(500, "boom".into())));
        assert!(!is_bad_gateway(&FetchError::Http(400, "bad query".into())));
        assert!(!is_bad_gateway(&FetchError::Network("timeout".into())));
        assert!(!is_bad_gateway(&FetchError::Empty));
    }
}
