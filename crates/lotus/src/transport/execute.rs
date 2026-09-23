// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! SPARQL query execution: POST, retry, content-negotiation, streaming.
//!
//! Every public function follows the same retry pattern: up to
//! [`MAX_HTTP_ATTEMPTS`] attempts, retrying on transient network failures and
//! 5xx server errors, failing fast on 4xx client errors.
//!
//! The HTTP calls are routed through the [`HttpClient`] trait (see
//! [`super::client`]) so the retry / rate-limit / gateway classification logic
//! can be unit-tested with a [`MockClient`] rather than the live network. The
//! public API is unchanged — each entry point builds a [`DefaultHttp`] and
//! delegates to a generic inner driver.

#[cfg(not(target_arch = "wasm32"))]
use std::io::{Seek, Write};

use super::client::{DefaultHttp, HttpClient, HttpResponse};
use super::error::{
    compact_http_error_text, is_client_error, is_rate_limit, is_success, looks_like_gateway_error,
};
use super::types::{FetchError, MAX_HTTP_ATTEMPTS, ResponseBody, ResponseFormat};

// ── Rate-limit backoff ────────────────────────────────────────────────────────

/// Sleep for the rate-limit backoff duration.
///
/// Production uses a blocking `std::thread::sleep` (matching the original
/// transport behaviour — a deliberate, in-scope-preserving choice). Under
/// `cfg(test)` the sleep is elided so retry tests are instant and
/// deterministic; on WASM the 429 branch returns before reaching this call.
#[allow(clippy::missing_const_for_fn, unused_variables)]
fn backoff_sleep(ms: u64) {
    #[cfg(not(any(target_arch = "wasm32", test)))]
    std::thread::sleep(std::time::Duration::from_millis(ms));
}

// ── Convenience wrappers ──────────────────────────────────────────────────────

/// Execute a SPARQL query against `endpoint` and return the raw CSV body.
///
/// Up to two attempts, with `Accept: text/csv` so the endpoint can honor
/// content negotiation even when the `action=csv_export` form parameter is
/// ignored. Retries transient network / 5xx errors; 4xx errors fail fast.
///
/// # Errors
/// Returns [`FetchError`] when the request fails, the upstream responds with an
/// HTTP error, or the body is empty / invalid UTF-8.
pub async fn execute_query(sparql: &str, endpoint: &str) -> Result<String, FetchError> {
    execute_sparql_with_format(sparql, endpoint, ResponseFormat::Csv).await
}

/// Execute a SPARQL query and return raw response bytes.
///
/// Useful for memory-sensitive paths where callers parse CSV directly from bytes
/// without first materializing an intermediate UTF-8 `String`.
///
/// # Errors
/// Returns [`FetchError`] for network/HTTP failures or empty upstream payloads.
pub async fn execute_sparql_bytes(sparql: &str, endpoint: &str) -> Result<Vec<u8>, FetchError> {
    let body = execute_sparql_body(sparql, endpoint).await?;
    Ok(body.to_vec())
}

/// Execute a SPARQL query and return the raw response body.
///
/// This avoids an extra `Bytes -> Vec<u8>` copy for callers that can parse from
/// borrowed byte slices or readers.
///
/// # Errors
/// Returns [`FetchError`] for network/HTTP failures or empty upstream payloads.
pub async fn execute_sparql_body(sparql: &str, endpoint: &str) -> Result<ResponseBody, FetchError> {
    execute_sparql_with_format_body(sparql, endpoint, ResponseFormat::Csv).await
}

#[cfg(not(target_arch = "wasm32"))]
/// Execute a SPARQL query and stream the response into a temporary file.
///
/// # Errors
/// Returns [`FetchError`] when request/streaming/tempfile I/O fails, or when
/// the upstream response is empty / an HTTP error.
pub async fn execute_sparql_tempfile(
    sparql: &str,
    endpoint: &str,
) -> Result<tempfile::NamedTempFile, FetchError> {
    execute_sparql_with_format_tempfile(sparql, endpoint, ResponseFormat::Csv).await
}

// ── Format-specific wrappers ──────────────────────────────────────────────────

/// Execute a SPARQL query and decode response bytes as UTF-8 text.
///
/// # Errors
/// Returns [`FetchError`] for transport/HTTP failures, empty responses, or
/// invalid UTF-8 payloads.
pub async fn execute_sparql_with_format(
    sparql: &str,
    endpoint: &str,
    format: ResponseFormat,
) -> Result<String, FetchError> {
    let bytes = execute_sparql_with_format_bytes(sparql, endpoint, format).await?;
    String::from_utf8(bytes).map_err(|e| FetchError::Parse(e.to_string()))
}

/// Execute a SPARQL query and return response bytes in a chosen representation.
///
/// # Errors
/// Returns [`FetchError`] for transport/HTTP failures or empty responses.
pub async fn execute_sparql_with_format_bytes(
    sparql: &str,
    endpoint: &str,
    format: ResponseFormat,
) -> Result<Vec<u8>, FetchError> {
    let body = execute_sparql_with_format_body(sparql, endpoint, format).await?;
    Ok(body.to_vec())
}

// ── Generic inner drivers ─────────────────────────────────────────────────────

/// POST a SPARQL query and return the raw response body.
///
/// Up to [`MAX_HTTP_ATTEMPTS`] attempts. Retries on transient network errors,
/// `5xx` responses, HTML gateway pages masquerading as `2xx`, and `429` rate
/// limits (with [`backoff_sleep`]). Fails fast on other `4xx` codes. On WASM,
/// `429` fails immediately (no backoff is possible).
pub(super) async fn execute_sparql_with_format_body_c<C: HttpClient>(
    sparql: &str,
    endpoint: &str,
    format: ResponseFormat,
    client: &C,
) -> Result<ResponseBody, FetchError> {
    log::debug!("SPARQL POST endpoint: {endpoint}");

    let mut last_err: Option<FetchError> = None;

    for attempt in 0..MAX_HTTP_ATTEMPTS {
        // `Accept` and `Content-Type: application/x-www-form-urlencoded` are
        // both CORS-safelisted, so the request stays simple (no preflight).
        // Do not add `User-Agent` or other custom headers — browsers refuse to
        // let WASM set them, which causes `QLever` to reject the preflight.
        let result = client
            .post(
                endpoint,
                format.accept(),
                build_sparql_form_body(sparql, format),
            )
            .await;

        match result {
            Ok(resp) => {
                let code = resp.status();
                if is_success(code) {
                    return match resp.bytes().await {
                        Ok(bytes) if bytes.is_empty() => Err(FetchError::Empty),
                        Ok(bytes) => {
                            // HTML gateway pages are text, so inspect a lossy preview.
                            let preview = String::from_utf8_lossy(&bytes);
                            if looks_like_gateway_error(&preview) {
                                let err = FetchError::Http(
                                    502,
                                    "upstream gateway error (HTML payload)".into(),
                                );
                                if attempt + 1 < MAX_HTTP_ATTEMPTS {
                                    last_err = Some(err);
                                    continue;
                                }
                                return Err(err);
                            }
                            Ok(bytes)
                        }
                        Err(e) => {
                            let err = FetchError::Network(e.to_string());
                            if attempt + 1 < MAX_HTTP_ATTEMPTS {
                                last_err = Some(err);
                                continue;
                            }
                            Err(err)
                        }
                    };
                }

                let body = resp.text().await.unwrap_or_default();
                let detail = compact_http_error_text(&body);
                log::error!("event=sparql_http_error status={code} detail={detail}");
                // Retry on rate limiting (429) with backoff; fail fast on other 4xx.
                if is_rate_limit(code) {
                    let backoff_ms: u64 = 1000 * u64::from(attempt + 1); // 1s, 2s, 3s...
                    log::warn!(
                        "event=sparql_rate_limit status={code} attempt={} backoff_ms={}",
                        attempt + 1,
                        backoff_ms
                    );
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        backoff_sleep(backoff_ms);
                        last_err = Some(FetchError::Http(code, detail.clone()));
                        continue;
                    }
                    #[cfg(target_arch = "wasm32")]
                    {
                        return Err(FetchError::Http(code, detail));
                    }
                }
                if is_client_error(code) {
                    return Err(FetchError::Http(code, detail));
                }
                last_err = Some(FetchError::Http(code, detail));
            }
            Err(e) => {
                last_err = Some(e);
            }
        }
    }

    Err(last_err.unwrap_or_else(|| FetchError::Network("unknown error".into())))
}

/// Execute a SPARQL query and return the raw response body for a representation.
///
/// # Errors
/// Returns [`FetchError`] for transport/HTTP failures or empty responses.
pub async fn execute_sparql_with_format_body(
    sparql: &str,
    endpoint: &str,
    format: ResponseFormat,
) -> Result<ResponseBody, FetchError> {
    execute_sparql_with_format_body_c(sparql, endpoint, format, &DefaultHttp).await
}

#[cfg(not(target_arch = "wasm32"))]
/// POST a SPARQL query and stream the selected representation into a tempfile.
///
/// Retries on transient network errors, `5xx` responses, HTML gateway pages,
/// and `429` rate limits (with [`backoff_sleep`]); fails fast on `4xx`.
pub(super) async fn execute_sparql_with_format_tempfile_c<C: HttpClient>(
    sparql: &str,
    endpoint: &str,
    format: ResponseFormat,
    client: &C,
) -> Result<tempfile::NamedTempFile, FetchError> {
    log::debug!("SPARQL POST endpoint: {endpoint}");

    let mut last_err: Option<FetchError> = None;

    'attempts: for attempt in 0..MAX_HTTP_ATTEMPTS {
        let result = client
            .post(
                endpoint,
                format.accept(),
                build_sparql_form_body(sparql, format),
            )
            .await;

        match result {
            Ok(mut resp) => {
                let code = resp.status();
                if is_success(code) {
                    let mut file = tempfile::NamedTempFile::new()
                        .map_err(|e| FetchError::Parse(format!("tempfile create failed: {e}")))?;
                    let mut preview = Vec::with_capacity(2048);
                    let mut wrote_any = false;

                    loop {
                        match resp.chunk().await {
                            Ok(Some(chunk)) => {
                                wrote_any = true;
                                if preview.len() < 2048 {
                                    let take = (2048 - preview.len()).min(chunk.len());
                                    preview
                                        .extend_from_slice(chunk.get(..take).unwrap_or_default());
                                }
                                file.write_all(&chunk).map_err(|e| {
                                    FetchError::Parse(format!("tempfile write failed: {e}"))
                                })?;
                            }
                            Ok(None) => break,
                            Err(e) => {
                                let err = FetchError::Network(e.to_string());
                                if attempt + 1 < MAX_HTTP_ATTEMPTS {
                                    last_err = Some(err);
                                    continue 'attempts;
                                }
                                return Err(err);
                            }
                        }
                    }

                    if !wrote_any {
                        return Err(FetchError::Empty);
                    }

                    let preview_text = String::from_utf8_lossy(&preview);
                    if looks_like_gateway_error(&preview_text) {
                        let err =
                            FetchError::Http(502, "upstream gateway error (HTML payload)".into());
                        if attempt + 1 < MAX_HTTP_ATTEMPTS {
                            last_err = Some(err);
                            continue;
                        }
                        return Err(err);
                    }

                    file.as_file_mut()
                        .rewind()
                        .map_err(|e| FetchError::Parse(format!("tempfile rewind failed: {e}")))?;
                    return Ok(file);
                }

                let body = resp.text().await.unwrap_or_default();
                let detail = compact_http_error_text(&body);
                log::error!("event=sparql_http_error status={code} detail={detail}");
                // Retry on rate limiting (429) with backoff; fail fast on other 4xx.
                if is_rate_limit(code) {
                    let backoff_ms: u64 = 1000 * u64::from(attempt + 1); // 1s, 2s, 3s...
                    log::warn!(
                        "event=sparql_rate_limit status={code} attempt={} backoff_ms={}",
                        attempt + 1,
                        backoff_ms
                    );
                    backoff_sleep(backoff_ms);
                    last_err = Some(FetchError::Http(code, detail.clone()));
                    continue;
                }
                if is_client_error(code) {
                    return Err(FetchError::Http(code, detail));
                }
                last_err = Some(FetchError::Http(code, detail));
            }
            Err(e) => {
                last_err = Some(e);
            }
        }
    }

    Err(last_err.unwrap_or_else(|| FetchError::Network("unknown error".into())))
}

/// Execute a SPARQL query and stream the response into a temporary file.
///
/// # Errors
/// Returns [`FetchError`] when request/streaming/tempfile I/O fails, or when
/// the upstream response is empty / an HTTP error.
#[cfg(not(target_arch = "wasm32"))]
pub async fn execute_sparql_with_format_tempfile(
    sparql: &str,
    endpoint: &str,
    format: ResponseFormat,
) -> Result<tempfile::NamedTempFile, FetchError> {
    execute_sparql_with_format_tempfile_c(sparql, endpoint, format, &DefaultHttp).await
}

// ── URL-based fetch ─────────────────────────────────────────────────────────

/// Fetch a fully-formed export URL (for example with `action=csv_export`) and
/// return raw response bytes.
///
/// This is useful for clients that want direct `QLever` export representations
/// while still using HTTP content negotiation (`Accept` / `Accept-Encoding`).
///
/// # Errors
/// Returns [`FetchError`] for transport/HTTP failures or empty responses.
pub async fn fetch_export_url_bytes(
    url: &str,
    format: ResponseFormat,
) -> Result<Vec<u8>, FetchError> {
    fetch_url_bytes_with_accept(url, format.accept()).await
}

/// Fetch an arbitrary URL and return raw response bytes.
///
/// Unlike [`fetch_export_url_bytes`], this does not constrain the `Accept`
/// header to a specific SPARQL representation. It is used for API-managed
/// download artifacts such as `application/gzip` attachments.
///
/// # Errors
/// Returns [`FetchError`] for transport/HTTP failures or empty responses.
pub async fn fetch_url_bytes(url: &str) -> Result<Vec<u8>, FetchError> {
    fetch_url_bytes_with_accept(url, "*/*").await
}

/// Fetch a URL with a specific `Accept` header and return the response body.
pub(super) async fn fetch_url_bytes_with_accept_c<C: HttpClient>(
    url: &str,
    accept: &str,
    client: &C,
) -> Result<Vec<u8>, FetchError> {
    log::debug!("GET endpoint: {url}");

    let mut last_err: Option<FetchError> = None;

    for attempt in 0..MAX_HTTP_ATTEMPTS {
        let result = client.get(url, accept).await;

        match result {
            Ok(resp) => {
                let code = resp.status();
                if is_success(code) {
                    return match resp.bytes().await {
                        Ok(bytes) if bytes.is_empty() => Err(FetchError::Empty),
                        Ok(bytes) => {
                            let preview = String::from_utf8_lossy(&bytes);
                            if looks_like_gateway_error(&preview) {
                                let err = FetchError::Http(
                                    502,
                                    "upstream gateway error (HTML payload)".into(),
                                );
                                if attempt + 1 < MAX_HTTP_ATTEMPTS {
                                    last_err = Some(err);
                                    continue;
                                }
                                return Err(err);
                            }
                            Ok(bytes.to_vec())
                        }
                        Err(e) => {
                            let err = FetchError::Network(e.to_string());
                            if attempt + 1 < MAX_HTTP_ATTEMPTS {
                                last_err = Some(err);
                                continue;
                            }
                            Err(err)
                        }
                    };
                }

                let body = resp.text().await.unwrap_or_default();
                let detail = compact_http_error_text(&body);
                log::error!("event=http_error status={code} detail={detail}");
                // 4xx (including 429) fail fast — no backoff on this path.
                if is_client_error(code) {
                    return Err(FetchError::Http(code, detail));
                }
                last_err = Some(FetchError::Http(code, detail));
            }
            Err(e) => {
                last_err = Some(e);
            }
        }
    }

    Err(last_err.unwrap_or_else(|| FetchError::Network("unknown error".into())))
}

async fn fetch_url_bytes_with_accept(url: &str, accept: &str) -> Result<Vec<u8>, FetchError> {
    fetch_url_bytes_with_accept_c(url, accept, &DefaultHttp).await
}

// ── Form body construction ────────────────────────────────────────────────────

/// Build the `query=<encoded>&action=<name>` form body for a SPARQL POST.
fn build_sparql_form_body(sparql: &str, format: ResponseFormat) -> String {
    let encoded = urlencoding::encode(sparql);
    // "query=" + encoded + optional "&action=<name>"
    let action = format.action();
    let capacity = 6 + encoded.len() + action.map_or(0, |a| 8 + a.len());
    let mut body = String::with_capacity(capacity);
    body.push_str("query=");
    body.push_str(&encoded);
    if let Some(action) = action {
        body.push_str("&action=");
        body.push_str(action);
    }
    body
}
