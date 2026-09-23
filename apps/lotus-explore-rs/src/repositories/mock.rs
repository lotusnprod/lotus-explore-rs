// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! [`MockRepository`] — a test stub for demonstrating SPARQL-only queries without backend dependencies.
//!
//! Test-only mock repository used by explorer unit tests.

use crate::api::SearchResponse;
use crate::models::SearchCriteria;
use crate::repositories::{LotusRepository, RepositoryError};

/// Test-only mock repository for unit tests without network dependencies.
#[derive(Clone)]
pub struct MockRepository {
    /// Fixed CSV bytes returned for every `sparql_bytes` call.
    pub sparql_response: Result<Vec<u8>, RepositoryError>,
}

impl MockRepository {
    /// Returns fixed CSV bytes for all SPARQL calls; simulates no API.
    pub fn sparql_only(csv: Vec<u8>) -> Self {
        Self {
            sparql_response: Ok(csv),
        }
    }

    /// Always returns a SPARQL network error.
    pub fn sparql_error(msg: impl Into<String>) -> Self {
        Self {
            sparql_response: Err(RepositoryError::network(msg.into())),
        }
    }
}

impl LotusRepository for MockRepository {
    async fn api_search(
        &self,
        _criteria: &SearchCriteria,
        _limit: usize,
        _include_counts: bool,
    ) -> Option<Result<SearchResponse, RepositoryError>> {
        None // Simulate "API not configured" — fall through to SPARQL.
    }

    async fn sparql_bytes(&self, _query: &str) -> Result<Vec<u8>, RepositoryError> {
        self.sparql_response.clone()
    }
}
