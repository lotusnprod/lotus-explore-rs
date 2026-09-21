// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! HTTP client construction and caching.
//!
//! [`http_client`] lazily builds and caches a single [`reqwest::Client`] so
//! that connection pools are reused across queries.

#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

use std::sync::OnceLock;

use super::types::FetchError;

/// Returns a cached `reqwest::Client`, lazily initialised.
///
/// The client is built with platform-specific settings: on native, gzip
/// decompression, timeouts, and connection pooling are configured; on WASM,
/// the browser manages these automatically.
///
/// # Errors
/// Returns [`FetchError::Network`] if the reqwest client builder fails.
pub(super) fn http_client() -> Result<&'static reqwest::Client, FetchError> {
    static CLIENT: OnceLock<Result<reqwest::Client, String>> = OnceLock::new();
    match CLIENT.get_or_init(build_http_client) {
        Ok(client) => Ok(client),
        Err(msg) => Err(FetchError::Network(format!(
            "failed to initialize SPARQL HTTP client: {msg}"
        ))),
    }
}

/// Build a new [`reqwest::Client`] with transport-layer settings.
pub(super) fn build_http_client() -> Result<reqwest::Client, String> {
    #[cfg(target_arch = "wasm32")]
    {
        // In the browser, fetch automatically sends `Accept-Encoding: gzip,
        // deflate, br` and decompresses transparently — no extra configuration
        // is required.
        reqwest::Client::builder()
            .build()
            .map_err(|e| e.to_string())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Enable automatic gzip decompression so `QLever` can return compressed
        // CSV/JSON/Turtle payloads. This adds `Accept-Encoding: gzip` to every
        // request and decodes the response body with flate2 before handing bytes
        // to the caller — substantially reducing transfer size for large result
        // sets without any changes to callers.
        //
        // Timeouts: 8s connect, 120s total per request.
        // Pool: 90s idle, max 32 idle per host.
        reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(8))
            .timeout(Duration::from_mins(2))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(32)
            .tcp_keepalive(Duration::from_secs(30))
            .gzip(true)
            .build()
            .map_err(|e| e.to_string())
    }
}
