// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Export-format and download-URL helpers used by `lotus-explore-rs`
//! (serves both the native `/v1` API and the WASM client).
//!
//! This module is the single source of truth for the CSV/JSON/RDF export
//! enum and its mapping to `action=` strings ("`csv_export`",
//! "`qlever_json_export`", "`turtle_export`").  Apps import `ExportFormat`
//! and call `qlever_export_url` / `build_upstream_export_url` instead of
//! re-implementing the mapping.

#![allow(clippy::module_name_repetitions)]

/// The three archive formats supported by the LOTUS/QLever export pipeline.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExportFormat {
    /// Comma-separated values — the compact, lossless default for bulk export.
    Csv,
    /// JSON in the [SPARQL Query Results JSON Format](https://www.w3.org/TR/sparql11-results-json/),
    /// also used for `ndjson` (one JSON object per line).
    Json,
    /// RDF/Turtle triples via a `CONSTRUCT` query, suitable for ingestion
    /// into triple stores.
    Rdf,
}

impl ExportFormat {
    /// Parse from a CLI / URL fragment ("csv", "json", "ndjson", "rdf").
    ///
    /// Returns `None` for unrecognized formats.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        let normalized = s.trim();
        if normalized.eq_ignore_ascii_case("csv") {
            Some(Self::Csv)
        } else if normalized.eq_ignore_ascii_case("json")
            || normalized.eq_ignore_ascii_case("ndjson")
        {
            Some(Self::Json)
        } else if normalized.eq_ignore_ascii_case("rdf") {
            Some(Self::Rdf)
        } else {
            None
        }
    }

    /// File extension without leading dot (e.g. `"csv"`).
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Json => "json",
            Self::Rdf => "rdf",
        }
    }

    /// `QLever` `action=` parameter value for this format.
    #[must_use]
    pub const fn qlever_action(self) -> &'static str {
        match self {
            Self::Csv => "csv_export",
            Self::Json => "qlever_json_export",
            Self::Rdf => "turtle_export",
        }
    }

    /// Prepares the SPARQL query for export, wrapping it in a `CONSTRUCT`
    /// template when the format is RDF.
    #[must_use]
    pub fn prepared_query(self, query: &str) -> String {
        match self {
            Self::Rdf => crate::queries::query_construct_from_select(query),
            _ => query.to_string(),
        }
    }

    /// MIME type associated with this export format.
    #[must_use]
    pub const fn content_type(self) -> &'static str {
        match self {
            Self::Csv => "text/csv;charset=utf-8",
            Self::Json => "application/sparql-results+json;charset=utf-8",
            Self::Rdf => "text/turtle;charset=utf-8",
        }
    }

    /// `perf::start_timer` label fragment for this format.
    #[must_use]
    pub const fn log_name(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Json => "json",
            Self::Rdf => "rdf",
        }
    }

    /// `perf::start_timer` full label (e.g. `"LOTUS:download_csv"`).
    #[must_use]
    pub const fn timer_label(self) -> &'static str {
        match self {
            Self::Csv => "LOTUS:download_csv",
            Self::Json => "LOTUS:download_json",
            Self::Rdf => "LOTUS:download_rdf",
        }
    }

    /// Trigger timer label (full timer label + `"_trigger"`).
    #[must_use]
    pub fn trigger_timer_label(self) -> String {
        format!("{}_trigger", self.timer_label())
    }
}

/// Builds a `QLever` export URL for the given query and format.
///
/// RDF exports use a `CONSTRUCT` query so `QLever` emits triples rather than
/// `SELECT` rows; CSV and JSON go straight to `QLever`'s native export actions.
#[must_use]
pub fn qlever_export_url(query: &str, format: ExportFormat) -> String {
    let prepared_query = if format == ExportFormat::Rdf {
        crate::queries::query_construct_from_select(query)
    } else {
        query.to_string()
    };
    format!(
        "{}?query={}&action={}",
        crate::transport::QLEVER_WIKIDATA,
        urlencoding::encode(&prepared_query),
        format.qlever_action()
    )
}

/// Builds a `QLever` export URL for the given query and format action string.
///
/// This is kept for callers that need a raw action string (e.g. the
/// `lotus-explore-rs` `/v1/search` endpoint, which lists direct `QLever` URLs
/// alongside its own gzip-cached export URLs).
#[must_use]
pub fn qlever_export_url_with_action(query: &str, action: &str) -> String {
    format!(
        "{}?query={}&action={action}",
        crate::transport::QLEVER_WIKIDATA,
        urlencoding::encode(query)
    )
}

/// Builds a `lotus-explore-rs` `/v1/export-file/{cache_key}/{format}` URL.
#[must_use]
pub fn api_export_file_url(cache_key: &str, format: ExportFormat) -> String {
    format!("/v1/export-file/{cache_key}/{}", format.extension())
}

/// Builds the upstream `QLever` export URL for a given query and format,
/// choosing the appropriate action string.
///
/// For RDF, the query is first wrapped in a `CONSTRUCT` template via
/// [`crate::queries::query_construct_from_select`].
#[must_use]
pub fn build_upstream_export_url(query: &str, format: ExportFormat) -> String {
    qlever_export_url(query, format)
}

/// Sanitizes a filename for safe browser download.
///
/// Removes control characters and replaces path separators and quotes with
/// underscores. No external crate required.
#[inline]
#[must_use]
pub fn sanitize_download_filename(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.trim().chars() {
        if c.is_control() {
            continue;
        }
        match c {
            '/' | '\\' | '"' | '\'' | '\n' | '\r' => out.push('_'),
            _ => out.push(c),
        }
    }
    out.trim_matches('.').trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_known_formats() {
        assert_eq!(ExportFormat::parse("csv"), Some(ExportFormat::Csv));
        assert_eq!(ExportFormat::parse("json"), Some(ExportFormat::Json));
        assert_eq!(ExportFormat::parse("ndjson"), Some(ExportFormat::Json));
        assert_eq!(ExportFormat::parse("rdf"), Some(ExportFormat::Rdf));
        assert_eq!(ExportFormat::parse(" JSON "), Some(ExportFormat::Json));
        assert_eq!(ExportFormat::parse("RDF"), Some(ExportFormat::Rdf));
        assert_eq!(ExportFormat::parse("ttl"), None);
    }

    #[test]
    fn extensions_match_variants() {
        assert_eq!(ExportFormat::Csv.extension(), "csv");
        assert_eq!(ExportFormat::Json.extension(), "json");
        assert_eq!(ExportFormat::Rdf.extension(), "rdf");
    }

    #[test]
    fn qlever_actions_are_stable() {
        assert_eq!(ExportFormat::Csv.qlever_action(), "csv_export");
        assert_eq!(ExportFormat::Json.qlever_action(), "qlever_json_export");
        assert_eq!(ExportFormat::Rdf.qlever_action(), "turtle_export");
    }

    #[test]
    fn sanitize_download_filename_strips_path_separators() {
        assert!(!sanitize_download_filename("../../etc/passwd").contains('/'));
        assert!(!sanitize_download_filename("../../etc/passwd").contains('\\'));
    }

    #[test]
    fn sanitize_download_filename_preserves_safe_names() {
        assert_eq!(
            sanitize_download_filename("lotus_results.csv"),
            "lotus_results.csv"
        );
        assert_eq!(
            sanitize_download_filename("natural-products.json"),
            "natural-products.json"
        );
    }

    #[test]
    fn parse_round_trips_through_extension() {
        for fmt in [ExportFormat::Csv, ExportFormat::Json, ExportFormat::Rdf] {
            let ext = fmt.extension();
            assert_eq!(ExportFormat::parse(ext), Some(fmt));
        }
    }

    #[test]
    fn content_types_are_valid_mime() {
        assert!(ExportFormat::Csv.content_type().starts_with("text/csv"));
        assert!(
            ExportFormat::Json
                .content_type()
                .starts_with("application/")
        );
        assert!(ExportFormat::Rdf.content_type().starts_with("text/turtle"));
    }

    #[test]
    fn parse_empty_and_whitespace_returns_none() {
        assert_eq!(ExportFormat::parse(""), None);
        assert_eq!(ExportFormat::parse("   "), None);
        assert_eq!(ExportFormat::parse("xyz"), None);
        assert_eq!(ExportFormat::parse("csv\n"), Some(ExportFormat::Csv));
    }

    #[test]
    fn prepared_query_wraps_rdf_in_construct() {
        let select = "PREFIX wd: <http://www.wikidata.org/entity/>\nSELECT ?s WHERE { ?s ?p ?o }";
        let csv_q = ExportFormat::Csv.prepared_query(select);
        assert_eq!(csv_q, select);

        let rdf_q = ExportFormat::Rdf.prepared_query(select);
        assert!(
            rdf_q.contains("CONSTRUCT"),
            "RDF query should be wrapped in CONSTRUCT"
        );
        assert!(
            rdf_q.contains("WHERE"),
            "RDF query should preserve WHERE block"
        );
    }

    #[test]
    fn api_export_file_url_includes_extension() {
        assert_eq!(
            api_export_file_url("abc123", ExportFormat::Json),
            "/v1/export-file/abc123/json"
        );
        assert_eq!(
            api_export_file_url("abc123", ExportFormat::Csv),
            "/v1/export-file/abc123/csv"
        );
    }

    #[test]
    fn timer_labels_are_consistent() {
        assert_eq!(ExportFormat::Csv.timer_label(), "LOTUS:download_csv");
        assert_eq!(ExportFormat::Json.timer_label(), "LOTUS:download_json");
        assert_eq!(ExportFormat::Rdf.timer_label(), "LOTUS:download_rdf");
    }

    #[test]
    fn trigger_timer_label_has_suffix() {
        assert_eq!(
            ExportFormat::Csv.trigger_timer_label(),
            "LOTUS:download_csv_trigger"
        );
        assert_eq!(
            ExportFormat::Json.trigger_timer_label(),
            "LOTUS:download_json_trigger"
        );
        assert_eq!(
            ExportFormat::Rdf.trigger_timer_label(),
            "LOTUS:download_rdf_trigger"
        );
    }

    #[test]
    fn qlever_export_url_encodes_query_and_appends_action() {
        let url = qlever_export_url("SELECT ?s WHERE { ?s ?p ?o }", ExportFormat::Csv);
        assert!(url.starts_with(crate::transport::QLEVER_WIKIDATA));
        assert!(url.contains("action=csv_export"));
        assert!(url.contains("query="));
    }

    #[test]
    fn qlever_export_url_rdf_uses_construct() {
        let select = "SELECT ?s WHERE { ?s ?p ?o }";
        let url = qlever_export_url(select, ExportFormat::Rdf);
        // RDF should wrap in CONSTRUCT before encoding
        assert!(url.contains("action=turtle_export"));
    }

    #[test]
    fn build_upstream_export_url_delegates_to_qlever_export_url() {
        let query = "SELECT ?s WHERE { ?s ?p ?o }";
        assert_eq!(
            build_upstream_export_url(query, ExportFormat::Csv),
            qlever_export_url(query, ExportFormat::Csv)
        );
    }

    #[test]
    fn qlever_export_url_with_action_uses_custom_action() {
        let url = qlever_export_url_with_action("SELECT ?s WHERE { ?s ?p ?o }", "custom_action");
        assert!(url.starts_with(crate::transport::QLEVER_WIKIDATA));
        assert!(url.contains("action=custom_action"));
    }

    #[test]
    fn api_export_file_url_all_formats() {
        assert_eq!(
            api_export_file_url("key123", ExportFormat::Rdf),
            "/v1/export-file/key123/rdf"
        );
    }
}
