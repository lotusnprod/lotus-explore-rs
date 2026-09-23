// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Thin wrappers that execute a SPARQL query on the default Wikidata `QLever`
//! endpoint. Each delegates to `crate::transport` with the standard endpoint
//! (`QLEVER_WIKIDATA`) and response-format defaults.

#[cfg(not(target_arch = "wasm32"))]
use crate::transport::execute_sparql_tempfile as shared_execute_tempfile;
use crate::transport::{
    FetchError, QLEVER_WIKIDATA, ResponseFormat, execute_query as shared_execute,
    execute_sparql_body as shared_execute_body, execute_sparql_bytes as shared_execute_bytes,
    execute_sparql_with_format as shared_execute_with_format,
};

/// Execute a LOTUS query on the default Wikidata `QLever` endpoint.
///
/// # Errors
/// Returns [`FetchError`] for transport/HTTP failures, empty responses, or
/// invalid UTF-8 payloads.
pub async fn execute_query(sparql: &str) -> Result<String, FetchError> {
    shared_execute(sparql, QLEVER_WIKIDATA).await
}

/// Execute a LOTUS query and return raw response bytes.
///
/// # Errors
/// Returns [`FetchError`] for transport/HTTP failures or empty responses.
pub async fn execute_sparql_bytes(sparql: &str) -> Result<Vec<u8>, FetchError> {
    shared_execute_bytes(sparql, QLEVER_WIKIDATA).await
}

/// Execute a LOTUS query and return the raw response body.
///
/// # Errors
/// Returns [`FetchError`] for transport/HTTP failures or empty responses.
pub async fn execute_sparql_body(
    sparql: &str,
) -> Result<crate::transport::ResponseBody, FetchError> {
    shared_execute_body(sparql, QLEVER_WIKIDATA).await
}

#[cfg(not(target_arch = "wasm32"))]
/// Execute a LOTUS query and stream the response into a temporary file.
///
/// # Errors
/// Returns [`FetchError`] when request/streaming/tempfile I/O fails, or when
/// the upstream response is empty / an HTTP error.
pub async fn execute_sparql_tempfile(sparql: &str) -> Result<tempfile::NamedTempFile, FetchError> {
    shared_execute_tempfile(sparql, QLEVER_WIKIDATA).await
}

/// Execute a LOTUS query with an explicit response format.
///
/// # Errors
/// Returns [`FetchError`] for transport/HTTP failures, empty responses, or
/// invalid UTF-8 payloads.
pub async fn execute_sparql_format(
    sparql: &str,
    format: ResponseFormat,
) -> Result<String, FetchError> {
    shared_execute_with_format(sparql, QLEVER_WIKIDATA, format).await
}
