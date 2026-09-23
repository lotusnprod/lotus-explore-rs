// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Unit tests for transport helpers.
//!
//! Only pure (non-async) helpers are tested here — `looks_like_gateway_error`,
//! `compact_http_error_text`, `extract_qid`, and `clean_doi`.

use super::error::{
    compact_http_error_text, is_client_error, is_rate_limit, is_success, looks_like_gateway_error,
};
use crate::transport::{clean_doi, coalesce, col_idx, extract_qid, field, non_empty, parse_year};

// ── Gateway error detection ─────────────────────────────────────────────────

#[test]
fn detects_html_gateway_payloads() {
    let html = "<html><head><title>502 Bad Gateway</title></head><body>nginx</body></html>";
    assert!(looks_like_gateway_error(html));
}

#[test]
fn does_not_flag_regular_csv_as_gateway_error() {
    let csv = "compound,taxon\nQ1,Q2\n";
    assert!(!looks_like_gateway_error(csv));
}

#[test]
fn does_not_flag_json_as_gateway_error() {
    let json = r#"{"head":{"vars":["label"]},"results":{"bindings":[]}}"#;
    assert!(!looks_like_gateway_error(json));
}

#[test]
fn detects_cloudflare_gateway_error() {
    let html = "<!DOCTYPE html><html><head><title>504 Gateway Timeout</title></head><body>cloudflare</body></html>";
    assert!(looks_like_gateway_error(html));
}

// ── extract_qid ─────────────────────────────────────────────────────────────

#[test]
fn extract_qid_handles_uri_and_plain_qid() {
    assert_eq!(
        extract_qid("http://www.wikidata.org/entity/Q12345"),
        "Q12345"
    );
    assert_eq!(
        extract_qid("https://www.wikidata.org/entity/Q12345"),
        "Q12345"
    );
    assert_eq!(extract_qid("Q999"), "Q999");
    assert_eq!(extract_qid("not-a-qid"), "");
    assert_eq!(extract_qid(""), "");
}

#[test]
fn extract_qid_rejects_non_qid_prefixes() {
    assert_eq!(extract_qid("http://www.wikidata.org/entity/P123"), "");
    assert_eq!(extract_qid("Qabc"), "");
    assert_eq!(extract_qid("Q"), "");
}

// ── clean_doi ──────────────────────────────────────────────────────────────

#[test]
fn clean_doi_normalizes_prefixed_urls() {
    assert_eq!(
        clean_doi("https://doi.org/10.1000/xyz"),
        Some("10.1000/xyz".to_string())
    );
    assert_eq!(clean_doi("  "), None);
}

#[test]
fn clean_doi_passes_through_bare_dois() {
    assert_eq!(clean_doi("10.1000/xyz"), Some("10.1000/xyz".to_string()));
    assert_eq!(
        clean_doi("  10.1000/xyz  "),
        Some("10.1000/xyz".to_string())
    );
}

#[test]
fn clean_doi_returns_none_for_empty_input() {
    assert_eq!(clean_doi(""), None);
    assert_eq!(clean_doi("   "), None);
}

// ── coalesce / non_empty ────────────────────────────────────────────────────

#[test]
fn coalesce_prefers_first_non_empty() {
    assert_eq!(coalesce("first", "second"), Some("first"));
    assert_eq!(coalesce("  first  ", "second"), Some("first"));
}

#[test]
fn coalesce_falls_back_to_second() {
    assert_eq!(coalesce("", "second"), Some("second"));
    assert_eq!(coalesce("   ", "second"), Some("second"));
}

#[test]
fn coalesce_returns_none_when_both_empty() {
    assert_eq!(coalesce("", ""), None);
    assert_eq!(coalesce("   ", "   "), None);
}

#[test]
fn non_empty_strips_and_checks() {
    assert_eq!(non_empty("hello"), Some("hello"));
    assert_eq!(non_empty("  hello  "), Some("hello"));
    assert_eq!(non_empty(""), None);
    assert_eq!(non_empty("   "), None);
}

// ── parse_year ─────────────────────────────────────────────────────────────

#[test]
fn parse_year_extracts_year_from_iso_date() {
    assert_eq!(parse_year("2021-04-23T00:00:00Z"), Some(2021));
    assert_eq!(parse_year("2021-04-23"), Some(2021));
    assert_eq!(parse_year("2021"), Some(2021));
}

#[test]
fn parse_year_returns_none_for_invalid_input() {
    assert_eq!(parse_year("not-a-date"), None);
    assert_eq!(parse_year(""), None);
    assert_eq!(parse_year("abc-def-ghi"), None);
}

// ── col_idx / field ────────────────────────────────────────────────────────

#[test]
fn col_idx_finds_named_column() {
    let headers = csv::StringRecord::from(vec!["compound", "taxon", "mass"]);
    assert_eq!(col_idx(&headers, "taxon"), Some(1));
    assert_eq!(col_idx(&headers, "mass"), Some(2));
    assert_eq!(col_idx(&headers, "missing"), None);
}

#[test]
fn field_returns_trimmed_value_or_empty() {
    let headers = csv::StringRecord::from(vec!["name", "value"]);
    let record = csv::StringRecord::from(vec!["  compound_1  ", "42.0"]);
    assert_eq!(field(&record, col_idx(&headers, "name")), "compound_1");
    assert_eq!(field(&record, col_idx(&headers, "value")), "42.0");
    assert_eq!(field(&record, None), "");
}

// ── compact_http_error_text ────────────────────────────────────────────────

#[test]
fn compact_http_error_text_prefers_json_exception_field() {
    let body = r#"{
  "exception": "Trying to insert a cache key which was already present",
  "query": "SELECT ..."
}"#;
    assert_eq!(
        compact_http_error_text(body),
        "Trying to insert a cache key which was already present"
    );
}

#[test]
fn compact_http_error_text_truncates_long_fallback_line() {
    let body = format!("{{\n  \"detail\": \"{}\"\n}}", "x".repeat(400));
    let compact = compact_http_error_text(&body);
    assert!(compact.chars().count() <= 241);
    assert!(compact.ends_with('…'));
}

#[test]
fn compact_http_error_text_empty_body() {
    assert_eq!(compact_http_error_text(""), "empty response body");
    assert_eq!(compact_http_error_text("   \n  "), "empty response body");
}

// ── parse_json_exception_field ─────────────────────────────────────────────

#[test]
fn parse_json_exception_field_extracts_value() {
    // Test via compact_http_error_text, since parse_json_exception_field is private.
    let body = r#"{"exception":"timeout"}"#;
    assert_eq!(compact_http_error_text(body), "timeout");
}

#[test]
fn parse_json_exception_field_handles_escapes() {
    let body = r#"{"exception":"hello\nworld"}"#;
    assert_eq!(compact_http_error_text(body), "hello\nworld");
}

#[test]
fn parse_json_exception_field_returns_none_when_absent() {
    // No "exception" field → falls through to line-based parsing.
    let body = r#"{"ok":true}"#;
    let compact = compact_http_error_text(body);
    assert!(compact.starts_with('{'));
    let body = "not json";
    assert_eq!(compact_http_error_text(body), "not json");
}

// ── HTTP status classification ────────────────────────────────────────────────
//
// These predicates drive the retry / fail-fast decision in `execute.rs` and are
// used by both the native and (cfg-gated) wasm transport paths. Testing them
// directly pins the boundary semantics independent of any HTTP client.

#[test]
fn success_range_includes_only_2xx() {
    assert!(is_success(200));
    assert!(is_success(204));
    assert!(is_success(299));
    assert!(!is_success(199));
    assert!(!is_success(300));
    assert!(!is_success(429));
    assert!(!is_success(502));
}

#[test]
fn client_error_range_includes_429() {
    assert!(is_client_error(400));
    assert!(is_client_error(404));
    assert!(is_client_error(429));
    assert!(is_client_error(499));
    assert!(!is_client_error(399));
    assert!(!is_client_error(500));
    assert!(!is_client_error(503));
}

#[test]
fn rate_limit_matches_only_429() {
    assert!(is_rate_limit(429));
    assert!(!is_rate_limit(400));
    assert!(!is_rate_limit(503));
}

// ── HTTP transport retry / classification (mock-driven) ─────────────────────
//
// These exercise the retry, rate-limit, gateway and fail-fast logic in
// `execute.rs` via a scripted [`MockClient`] — no network required.
//
// `backoff_sleep` is a no-op under `cfg(test)`, so the 429 backoff paths are
// instant.

#[cfg(not(target_arch = "wasm32"))]
mod transport_async {
    use crate::transport::client::{HttpClient, HttpResponse};
    use crate::transport::execute::{
        execute_sparql_with_format_body_c, execute_sparql_with_format_tempfile_c,
        fetch_url_bytes_with_accept_c,
    };
    use crate::transport::types::{FetchError, ResponseFormat};
    use bytes::Bytes;
    use std::collections::VecDeque;
    use std::io::Read;
    use std::sync::Arc;
    use std::sync::Mutex;

    // ── Scripted HTTP doubles ─────────────────────────────────────────────────

    /// A single scripted response step fed to [`MockClient`].
    #[derive(Debug)]
    enum MockStep {
        /// A successful HTTP response with the given status and body.
        Ok(u16, Bytes),
        /// A transport-level network failure (maps to `FetchError::Network`).
        NetworkErr,
    }

    /// A scripted [`HttpClient`] for unit-testing the retry / classification
    /// logic in [`crate::transport::execute`] without hitting the network.
    ///
    /// Steps are consumed in order across attempts/calls — so a sequence like
    /// `[Ok(502, …), Ok(200, body)]` exercises a retry-then-success path.
    #[derive(Debug)]
    struct MockClient {
        steps: Arc<Mutex<VecDeque<MockStep>>>,
    }

    impl MockClient {
        fn new(steps: impl IntoIterator<Item = MockStep>) -> Self {
            Self {
                steps: Arc::new(Mutex::new(VecDeque::from_iter(steps))),
            }
        }

        /// Number of scripted steps not yet consumed — lets tests assert that a
        /// fail-fast path did not perform an extra attempt.
        fn remaining(&self) -> usize {
            self.steps.lock().unwrap().len()
        }

        fn next(&self) -> Result<MockResponse, FetchError> {
            let mut guard = self.steps.lock().unwrap();
            match guard.pop_front() {
                Some(MockStep::Ok(status, body)) => Ok(MockResponse::new(status, body)),
                Some(MockStep::NetworkErr) => Err(FetchError::Network("mock network error".into())),
                None => Err(FetchError::Network("mock client exhausted".into())),
            }
        }
    }

    impl Clone for MockClient {
        fn clone(&self) -> Self {
            Self {
                steps: Arc::clone(&self.steps),
            }
        }
    }

    impl HttpClient for MockClient {
        type Response = MockResponse;

        async fn post(
            &self,
            _endpoint: &str,
            _accept: &str,
            _body: String,
        ) -> Result<MockResponse, FetchError> {
            self.next()
        }

        async fn get(&self, _url: &str, _accept: &str) -> Result<MockResponse, FetchError> {
            self.next()
        }
    }

    /// A lightweight [`HttpResponse`] backed by a single `Bytes` body.
    #[derive(Debug)]
    struct MockResponse {
        status: u16,
        body: Bytes,
        pos: usize,
    }

    impl MockResponse {
        fn new(status: u16, body: Bytes) -> Self {
            Self {
                status,
                body,
                pos: 0,
            }
        }
    }

    impl HttpResponse for MockResponse {
        #[inline]
        fn status(&self) -> u16 {
            self.status
        }

        async fn bytes(self) -> Result<Bytes, FetchError> {
            Ok(self.body)
        }

        async fn text(self) -> Result<String, FetchError> {
            Ok(String::from_utf8_lossy(&self.body).into_owned())
        }

        async fn chunk(&mut self) -> Result<Option<Bytes>, FetchError> {
            let len = self.body.len();
            if self.pos >= len {
                return Ok(None);
            }
            let end = (self.pos + 8192).min(len);
            let chunk = self.body.slice(self.pos..end);
            self.pos = end;
            Ok(Some(chunk))
        }
    }

    fn body(s: &str) -> Bytes {
        Bytes::from(s.to_string())
    }

    fn gateway_html() -> Bytes {
        body("<html><head><title>502 Bad Gateway</title></head><body>nginx</body></html>")
    }

    fn client(steps: impl IntoIterator<Item = MockStep>) -> MockClient {
        MockClient::new(steps)
    }

    // ── POST / body driver: execute_sparql_with_format_body_c ────────────────

    #[tokio::test]
    async fn post_body_succeeds_on_first_attempt() {
        let csv = body("compound,taxon\nQ1,Q2\n");
        let client = client([MockStep::Ok(200, csv.clone())]);
        let out = execute_sparql_with_format_body_c(
            "SELECT * WHERE {}",
            "https://qlever.dev/api/wikidata",
            ResponseFormat::Csv,
            &client,
        )
        .await;
        assert_eq!(out.unwrap(), csv);
    }

    #[tokio::test]
    async fn post_body_retries_on_5xx_then_succeeds() {
        let csv = body("compound,taxon\nQ1,Q2\n");
        let client = client([
            MockStep::Ok(500, body("internal server error")),
            MockStep::Ok(200, csv.clone()),
        ]);
        let out = execute_sparql_with_format_body_c(
            "SELECT * WHERE {}",
            "https://qlever.dev/api/wikidata",
            ResponseFormat::Csv,
            &client,
        )
        .await;
        assert_eq!(out.unwrap(), csv);
    }

    #[tokio::test]
    async fn post_body_retries_on_network_error_then_succeeds() {
        let csv = body("compound,taxon\nQ1,Q2\n");
        let client = client([MockStep::NetworkErr, MockStep::Ok(200, csv.clone())]);
        let out = execute_sparql_with_format_body_c(
            "SELECT * WHERE {}",
            "https://qlever.dev/api/wikidata",
            ResponseFormat::Csv,
            &client,
        )
        .await;
        assert_eq!(out.unwrap(), csv);
    }

    #[tokio::test]
    async fn post_body_retries_on_html_gateway_disguised_as_2xx() {
        let csv = body("compound,taxon\nQ1,Q2\n");
        let client = client([
            MockStep::Ok(200, gateway_html()),
            MockStep::Ok(200, csv.clone()),
        ]);
        let out = execute_sparql_with_format_body_c(
            "SELECT * WHERE {}",
            "https://qlever.dev/api/wikidata",
            ResponseFormat::Csv,
            &client,
        )
        .await;
        assert_eq!(out.unwrap(), csv);
    }

    #[tokio::test]
    async fn post_body_429_rate_limit_retries_then_succeeds() {
        // 429 triggers backoff + retry; on the second attempt the body arrives.
        // `backoff_sleep` is a no-op under cfg(test), so this is instant.
        let csv = body("compound,taxon\nQ1,Q2\n");
        let client = client([
            MockStep::Ok(429, body("rate limited")),
            MockStep::Ok(200, csv.clone()),
        ]);
        let out = execute_sparql_with_format_body_c(
            "SELECT * WHERE {}",
            "https://qlever.dev/api/wikidata",
            ResponseFormat::Csv,
            &client,
        )
        .await;
        assert_eq!(out.unwrap(), csv);
    }

    #[tokio::test]
    async fn post_body_429_exhausted_returns_http_429() {
        // Both attempts return 429 → final error is Http(429), not the 429's body.
        let client = client([
            MockStep::Ok(429, body("rate limited")),
            MockStep::Ok(429, body("rate limited")),
        ]);
        let err = execute_sparql_with_format_body_c(
            "SELECT * WHERE {}",
            "https://qlever.dev/api/wikidata",
            ResponseFormat::Csv,
            &client,
        )
        .await;
        match err.unwrap_err() {
            FetchError::Http(429, _) => {}
            other => panic!("expected Http(429), got {other:?}"),
        }
    }

    #[tokio::test]
    async fn post_body_4xx_fails_fast_without_retry() {
        // 404 is a client error → fail immediately; the second step (if any) is
        // never consumed.
        let client = client([MockStep::Ok(404, body("not found"))]);
        let err = execute_sparql_with_format_body_c(
            "SELECT * WHERE {}",
            "https://qlever.dev/api/wikidata",
            ResponseFormat::Csv,
            &client,
        )
        .await;
        match err.unwrap_err() {
            FetchError::Http(404, detail) => assert!(detail.contains("not found")),
            other => panic!("expected Http(404), got {other:?}"),
        }
        // Fail-fast consumed exactly one step (the 404); no retry occurred.
        assert_eq!(client.remaining(), 0);
    }

    #[tokio::test]
    async fn post_body_5xx_exhausted_returns_last_http_error() {
        let client = client([
            MockStep::Ok(500, body("internal server error")),
            MockStep::Ok(500, body("internal server error")),
        ]);
        let err = execute_sparql_with_format_body_c(
            "SELECT * WHERE {}",
            "https://qlever.dev/api/wikidata",
            ResponseFormat::Csv,
            &client,
        )
        .await;
        match err.unwrap_err() {
            FetchError::Http(500, _) => {}
            other => panic!("expected Http(500), got {other:?}"),
        }
    }

    #[tokio::test]
    async fn post_body_network_errors_exhausted_returns_network_error() {
        let client = client([MockStep::NetworkErr, MockStep::NetworkErr]);
        let err = execute_sparql_with_format_body_c(
            "SELECT * WHERE {}",
            "https://qlever.dev/api/wikidata",
            ResponseFormat::Csv,
            &client,
        )
        .await;
        match err.unwrap_err() {
            FetchError::Network(_) => {}
            other => panic!("expected Network, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn post_body_empty_2xx_returns_empty_error() {
        let client = client([MockStep::Ok(200, Bytes::new())]);
        let err = execute_sparql_with_format_body_c(
            "SELECT * WHERE {}",
            "https://qlever.dev/api/wikidata",
            ResponseFormat::Csv,
            &client,
        )
        .await;
        assert_eq!(err.unwrap_err(), FetchError::Empty);
    }

    // ── Tempfile / streaming driver: execute_sparql_with_format_tempfile_c ────

    #[tokio::test]
    async fn tempfile_streams_body_successfully() {
        let csv = body("compound,taxon\nQ1,Q2\n");
        let client = client([MockStep::Ok(200, csv.clone())]);
        let file = execute_sparql_with_format_tempfile_c(
            "SELECT * WHERE {}",
            "https://qlever.dev/api/wikidata",
            ResponseFormat::Csv,
            &client,
        )
        .await
        .expect("tempfile success");
        // The driver rewinds the file before returning.
        let mut got = String::new();
        file.as_file().read_to_string(&mut got).unwrap();
        assert_eq!(got, std::str::from_utf8(&csv).unwrap());
    }

    #[tokio::test]
    async fn tempfile_streams_large_body_in_chunks() {
        // > 8KB to exercise multiple `chunk()` iterations.
        let big = body(&"x".repeat(20_000));
        let client = client([MockStep::Ok(200, big.clone())]);
        let file = execute_sparql_with_format_tempfile_c(
            "SELECT * WHERE {}",
            "https://qlever.dev/api/wikidata",
            ResponseFormat::Csv,
            &client,
        )
        .await
        .expect("tempfile success");
        let mut got = Vec::new();
        file.as_file().read_to_end(&mut got).unwrap();
        assert_eq!(got.len(), 20_000);
    }

    #[tokio::test]
    async fn tempfile_retries_on_html_gateway_then_succeeds() {
        let csv = body("compound,taxon\nQ1,Q2\n");
        let client = client([MockStep::Ok(200, gateway_html()), MockStep::Ok(200, csv)]);
        let file = execute_sparql_with_format_tempfile_c(
            "SELECT * WHERE {}",
            "https://qlever.dev/api/wikidata",
            ResponseFormat::Csv,
            &client,
        )
        .await
        .expect("tempfile success after gateway retry");
        let mut got = String::new();
        file.as_file().read_to_string(&mut got).unwrap();
        assert_eq!(got, "compound,taxon\nQ1,Q2\n");
    }

    #[tokio::test]
    async fn tempfile_4xx_fails_fast_without_retry() {
        let client = client([MockStep::Ok(404, body("not found"))]);
        let err = execute_sparql_with_format_tempfile_c(
            "SELECT * WHERE {}",
            "https://qlever.dev/api/wikidata",
            ResponseFormat::Csv,
            &client,
        )
        .await;
        match err.unwrap_err() {
            FetchError::Http(404, _) => {}
            other => panic!("expected Http(404), got {other:?}"),
        }
        assert_eq!(client.remaining(), 0);
    }

    #[tokio::test]
    async fn tempfile_empty_2xx_returns_empty_error() {
        let client = client([MockStep::Ok(200, Bytes::new())]);
        let err = execute_sparql_with_format_tempfile_c(
            "SELECT * WHERE {}",
            "https://qlever.dev/api/wikidata",
            ResponseFormat::Csv,
            &client,
        )
        .await;
        assert_eq!(err.unwrap_err(), FetchError::Empty);
    }

    // ── GET driver: fetch_url_bytes_with_accept_c ────────────────────────────

    #[tokio::test]
    async fn get_succeeds_on_first_attempt() {
        let payload = body("application/gzip blob");
        let client = client([MockStep::Ok(200, payload.clone())]);
        let out = fetch_url_bytes_with_accept_c(
            "https://example.org/export.csv?q=1&action=csv_export",
            "text/csv",
            &client,
        )
        .await;
        assert_eq!(out.unwrap(), payload.to_vec());
    }

    #[tokio::test]
    async fn get_502_retried_then_succeeds() {
        let payload = body("compound,taxon\nQ1,Q2\n");
        let client = client([
            MockStep::Ok(502, body("502 Bad Gateway")),
            MockStep::Ok(200, payload.clone()),
        ]);
        let out = fetch_url_bytes_with_accept_c(
            "https://example.org/export.csv?q=1&action=csv_export",
            "text/csv",
            &client,
        )
        .await;
        assert_eq!(out.unwrap(), payload.to_vec());
    }

    #[tokio::test]
    async fn get_429_fails_fast_without_backoff() {
        // Unlike the POST path, GET fails fast on 429 (no backoff, no retry).
        let client = client([MockStep::Ok(429, body("rate limited"))]);
        let err = fetch_url_bytes_with_accept_c(
            "https://example.org/export.csv?q=1&action=csv_export",
            "text/csv",
            &client,
        )
        .await;
        match err.unwrap_err() {
            FetchError::Http(429, _) => {}
            other => panic!("expected Http(429), got {other:?}"),
        }
        assert_eq!(client.remaining(), 0);
    }

    #[tokio::test]
    async fn get_4xx_fails_fast_without_retry() {
        let client = client([MockStep::Ok(404, body("not found"))]);
        let err = fetch_url_bytes_with_accept_c(
            "https://example.org/export.csv?q=1&action=csv_export",
            "text/csv",
            &client,
        )
        .await;
        match err.unwrap_err() {
            FetchError::Http(404, detail) => assert!(detail.contains("not found")),
            other => panic!("expected Http(404), got {other:?}"),
        }
        assert_eq!(client.remaining(), 0);
    }

    #[tokio::test]
    async fn get_html_gateway_on_2xx_retries_then_succeeds() {
        let payload = body("compound,taxon\nQ1,Q2\n");
        let client = client([
            MockStep::Ok(200, gateway_html()),
            MockStep::Ok(200, payload.clone()),
        ]);
        let out = fetch_url_bytes_with_accept_c(
            "https://example.org/export.csv?q=1&action=csv_export",
            "text/csv",
            &client,
        )
        .await;
        assert_eq!(out.unwrap(), payload.to_vec());
    }

    #[tokio::test]
    async fn get_empty_2xx_returns_empty_error() {
        let client = client([MockStep::Ok(200, Bytes::new())]);
        let err = fetch_url_bytes_with_accept_c(
            "https://example.org/export.csv?q=1&action=csv_export",
            "text/csv",
            &client,
        )
        .await;
        assert_eq!(err.unwrap_err(), FetchError::Empty);
    }
}
