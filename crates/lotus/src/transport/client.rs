// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! HTTP client construction, the transport trait abstraction, and test doubles.
//!
//! [`http_client`] lazily builds and caches a single [`reqwest::Client`] so
//! that connection pools are reused across queries.
//!
//! [`HttpClient`] / [`HttpResponse`] are a thin abstracting over `reqwest` so
//! that the retry / rate-limit / gateway logic in [`super::execute`] can be
//! unit-tested with a scripted [`MockClient`] instead of hitting the network.

use bytes::Bytes;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

use std::sync::OnceLock;

use super::types::FetchError;

// ── Cached reqwest client ─────────────────────────────────────────────────────

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
        // request and decodes the response body with `flate2` (pure-Rust
        // `miniz_oxide` backend — no native `libz`) before handing bytes to the
        // caller — substantially reducing transfer size for large result sets
        // without any changes to callers.
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

// ── Transport abstraction ─────────────────────────────────────────────────────

/// An HTTP response consumed by the transport layer.
///
/// Methods mirror the subset of `reqwest::Response` used by [`super::execute`]:
/// the immediate status code, the full body (`bytes`/`text`), and streaming
/// access via `chunk`. `bytes`/`text` consume the response; `chunk` borrows it
/// for the streaming (tempfile) path.
pub(super) trait HttpResponse {
    /// HTTP status code, available immediately after headers arrive.
    fn status(&self) -> u16;

    /// Read the full response body into bytes (consuming the response).
    async fn bytes(self) -> Result<Bytes, FetchError>;

    /// Read the full response body as UTF-8 text (consuming the response).
    async fn text(self) -> Result<String, FetchError>;

    /// Read the next chunk of the streaming body. Returns `None` at EOF.
    async fn chunk(&mut self) -> Result<Option<Bytes>, FetchError>;
}

/// An HTTP client that can issue POST/GET requests and return a typed response.
///
/// Abstracting over `reqwest` lets the retry / rate-limit / gateway logic in
/// [`super::execute`] be exercised by a mock in unit tests.
pub(super) trait HttpClient: Clone + Send + Sync + 'static {
    /// Concrete response type produced by this client.
    type Response: HttpResponse;

    /// Issue a SPARQL POST: form-encode `body` and send with `Accept: accept`.
    async fn post(
        &self,
        endpoint: &str,
        accept: &str,
        body: String,
    ) -> Result<Self::Response, FetchError>;

    /// Issue a GET for an arbitrary URL with `Accept: accept`.
    async fn get(&self, url: &str, accept: &str) -> Result<Self::Response, FetchError>;
}

// ── Production impl ───────────────────────────────────────────────────────────

/// Default HTTP client backed by the cached [`reqwest::Client`]. A ZST — passing
/// it around is free, so [`super::execute`] accepts it by value.
#[derive(Clone, Copy, Default)]
pub(super) struct DefaultHttp;

impl HttpClient for DefaultHttp {
    type Response = reqwest::Response;

    async fn post(
        &self,
        endpoint: &str,
        accept: &str,
        body: String,
    ) -> Result<reqwest::Response, FetchError> {
        let client = http_client()?;
        client
            .post(endpoint)
            .header("Accept", accept)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .map_err(|e| FetchError::Network(e.to_string()))
    }

    async fn get(&self, url: &str, accept: &str) -> Result<reqwest::Response, FetchError> {
        let client = http_client()?;
        client
            .get(url)
            .header("Accept", accept)
            .send()
            .await
            .map_err(|e| FetchError::Network(e.to_string()))
    }
}

impl HttpResponse for reqwest::Response {
    #[inline]
    fn status(&self) -> u16 {
        self.status().as_u16()
    }

    async fn bytes(self) -> Result<Bytes, FetchError> {
        self.bytes()
            .await
            .map_err(|e| FetchError::Network(e.to_string()))
    }

    async fn text(self) -> Result<String, FetchError> {
        self.text()
            .await
            .map_err(|e| FetchError::Network(e.to_string()))
    }

    // On native, `reqwest::Response` exposes an inherent `chunk` (used by the
    // tempfile streaming path). Method-call syntax resolves to the inherent
    // method, never the trait method being defined here, so there is no
    // recursion.
    #[cfg(not(target_arch = "wasm32"))]
    async fn chunk(&mut self) -> Result<Option<Bytes>, FetchError> {
        self.chunk()
            .await
            .map_err(|e| FetchError::Network(e.to_string()))
    }

    // Reqwest's wasm `Response` has no inherent `chunk`, and the streaming
    // (tempfile) path that calls `chunk` is native-only anyway — so this method
    // is unreachable on WASM. It exists solely to satisfy the trait.
    #[cfg(target_arch = "wasm32")]
    #[allow(clippy::needless_pass_by_ref_mut)]
    async fn chunk(&mut self) -> Result<Option<Bytes>, FetchError> {
        let _ = self;
        Err(FetchError::Network(
            "streaming not supported on WASM".into(),
        ))
    }
}
