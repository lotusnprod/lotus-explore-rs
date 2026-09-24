// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Repository layer — a thin boundary between the search orchestration and the
//! concrete data-access backends (REST API and direct SPARQL).
//!
//! # Design rationale
//!
//! `orchestrator.rs` previously called `api::search` and SPARQL transport
//! directly, mixing I/O concerns with business logic. Introducing a trait here
//! gives us:
//!
//! * **Clean boundaries** — orchestration code does not import transport details
//! * **Testability** — unit tests can supply a `MockRepository` without network
//!
//! # Trait object vs generics
//!
//! We use `impl LotusRepository` (generics, monomorphised) rather than
//! `dyn LotusRepository` (dynamic dispatch) because:
//!
//! * `async fn` in trait currently requires `dyn`-unsafe workarounds on stable
//! * WASM futures are `!Send`, which would require boxing the returned futures
//! * Monomorphisation gives zero-overhead abstraction at compile time
//!
//! Concrete production code uses [`HybridRepository`], which tries the REST API
//! first (if `api_base` is configured) and falls back to direct SPARQL.

pub mod hybrid;
#[cfg(test)]
pub mod mock;

// Re-export WDQS fallback tracking functions from hybrid.rs
pub use hybrid::{get_wdqs_transformed_query, is_wdqs_fallback_used, reset_wdqs_fallback_flag};

pub use hybrid::HybridRepository;

use crate::api::SearchResponse;
use crate::models::SearchCriteria;
#[cfg(not(target_arch = "wasm32"))]
use std::io::{Seek, Write};
use std::sync::Arc;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum RepositoryError {
    #[error("LOTUS API not configured")]
    NotConfigured,

    #[error("network error: {0}")]
    Network(Arc<str>),

    #[error("HTTP {status}: {body}")]
    Http { status: u16, body: String },

    #[error("parse error: {0}")]
    Parse(Arc<str>),
}

impl RepositoryError {
    pub fn network(message: impl Into<Arc<str>>) -> Self {
        Self::Network(message.into())
    }

    pub fn parse(message: impl Into<Arc<str>>) -> Self {
        Self::Parse(message.into())
    }
}

impl From<crate::api::ApiClientError> for RepositoryError {
    fn from(value: crate::api::ApiClientError) -> Self {
        match value {
            crate::api::ApiClientError::Network(msg) => Self::network(msg),
            crate::api::ApiClientError::Http(status, body) => Self::Http { status, body },
            crate::api::ApiClientError::Parse(msg) => Self::parse(msg),
        }
    }
}

/// Boundary trait for data-access operations used by the search orchestrator.
///
/// Implementations may delegate to the REST API, SPARQL, or a test stub.
pub trait LotusRepository: Clone + 'static {
    /// Try the REST API fast path.  Returns:
    /// - `None` — API path unavailable without an attempted request (e.g. test stub)
    /// - `Some(Ok(resp))` — successful API response
    /// - `Some(Err(RepositoryError::NotConfigured))` — API not configured; caller should fall back
    /// - `Some(Err(reason))` — API call failed; caller should fall back
    async fn api_search(
        &self,
        criteria: &SearchCriteria,
        limit: usize,
        include_counts: bool,
    ) -> Option<Result<SearchResponse, RepositoryError>>;

    /// Execute a SPARQL query and return the raw response body.
    async fn sparql_body(
        &self,
        query: &str,
    ) -> Result<lotus::transport::ResponseBody, RepositoryError>;

    #[cfg(not(target_arch = "wasm32"))]
    async fn sparql_tempfile(
        &self,
        query: &str,
    ) -> Result<tempfile::NamedTempFile, RepositoryError> {
        let body = self.sparql_body(query).await?;
        let mut file = tempfile::NamedTempFile::new()
            .map_err(|e| RepositoryError::parse(format!("tempfile create failed: {e}")))?;
        file.write_all(&body)
            .map_err(|e| RepositoryError::parse(format!("tempfile write failed: {e}")))?;
        file.as_file_mut()
            .rewind()
            .map_err(|e| RepositoryError::parse(format!("tempfile rewind failed: {e}")))?;
        Ok(file)
    }
}
