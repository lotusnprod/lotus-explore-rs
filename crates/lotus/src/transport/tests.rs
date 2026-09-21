// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Unit tests for transport helpers.
//!
//! Only pure (non-async) helpers are tested here — `looks_like_gateway_error`,
//! `compact_http_error_text`, `extract_qid`, and `clean_doi`.

use super::error::{compact_http_error_text, looks_like_gateway_error};
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
