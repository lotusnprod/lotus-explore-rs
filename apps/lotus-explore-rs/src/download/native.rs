// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

use crate::download::DownloadFormat;
use crate::perf;
use crate::sparql;
use lotus::queries::transform_query_for_wdqs;
use lotus::transport::ResponseFormat as LotusResponseFormat;
use lotus::transport::WDQS_SCHOLARLY;
use lotus::transport::WDQS_WIKIDATA;
use std::sync::Arc;

/// Single toggle to force WDQS fallback for testing.
/// Set this to `true` to force all queries to use WDQS.
/// Set this to `false` to use QLever (default).
const FORCE_WDQS_FALLBACK: bool = false;

pub(super) async fn execute_download_native(
    format: DownloadFormat,
    query: Arc<str>,
    filename: String,
    dl_timer: perf::TimerHandle,
) -> Result<(), String> {
    execute_download_with_fallback(format, query, filename, dl_timer).await
}

async fn execute_download_with_fallback(
    format: DownloadFormat,
    query: Arc<str>,
    filename: String,
    dl_timer: perf::TimerHandle,
) -> Result<(), String> {
    // Check if we should force WDQS fallback for testing
    if FORCE_WDQS_FALLBACK {
        log::warn!(
            "event=download format={} phase=fetch state=fallback reason=force_env",
            format.log_name()
        );
        return execute_download_wdqs(format, query, filename, dl_timer).await;
    }

    // First try QLever, fallback to WDQS on 502
    let result = execute_download_direct(format, query.as_ref()).await;

    match result {
        Ok(body) => {
            let fetch_elapsed = perf::end_timer(format.timer_label(), dl_timer);
            perf::log_timing(
                "download",
                &format!(
                    "event=download format={} phase=fetch state=success source=direct body_bytes={}",
                    format.log_name(),
                    body.len()
                ),
                Some(fetch_elapsed),
            );

            let trigger_timer = perf::start_timer(&format.trigger_timer_label());
            trigger_download(&filename, format.content_type(), &body);
            let trigger_elapsed = perf::end_timer(&format.trigger_timer_label(), trigger_timer);
            perf::log_timing(
                "download",
                &format!(
                    "event=download format={} phase=trigger state=success source=direct",
                    format.log_name()
                ),
                Some(trigger_elapsed),
            );
            Ok(())
        }
        Err(e) => {
            // Check if it's a gateway error (502) indicating QLever is down
            if e.contains("502") || e.contains("Bad Gateway") || e.contains("gateway") {
                log::warn!(
                    "event=download format={} phase=fetch state=fallback reason=qlever_502",
                    format.log_name()
                );
                // Retry with WDQS using scholarly subgraph query
                execute_download_wdqs(format, query, filename, dl_timer).await
            } else {
                handle_download_error(format, &e, dl_timer)
            }
        }
    }
}

async fn execute_download_wdqs(
    format: DownloadFormat,
    query: Arc<str>,
    filename: String,
    dl_timer: perf::TimerHandle,
) -> Result<(), String> {
    // For simple reference queries, use scholarly endpoint directly without transformation
    if query.contains("SELECT ?ref WHERE {") && query.contains("wdt:P356") {
        log::warn!(
            "event=download format={} phase=fetch state=wdqs_scholarly_endpoint",
            format.log_name()
        );
        let query_without_prefix = query.replace("{CURATION_SPARQL_PREFIXES}\n", "");
        return match format {
            DownloadFormat::Csv => {
                execute_sparql_with_format_download(
                    format,
                    &query_without_prefix,
                    WDQS_SCHOLARLY,
                    LotusResponseFormat::Csv,
                    &filename,
                    dl_timer,
                )
                .await
            }
            DownloadFormat::Json => {
                execute_sparql_with_format_download(
                    format,
                    &query_without_prefix,
                    WDQS_SCHOLARLY,
                    LotusResponseFormat::SparqlJson,
                    &filename,
                    dl_timer,
                )
                .await
            }
            DownloadFormat::Rdf => {
                // For Turtle format, SELECT must be wrapped in CONSTRUCT
                let construct_query = format.prepared_query(&query_without_prefix);
                execute_sparql_with_format_download(
                    format,
                    &construct_query,
                    WDQS_SCHOLARLY,
                    LotusResponseFormat::Turtle,
                    &filename,
                    dl_timer,
                )
                .await
            }
        };
    }

    // For complex queries, apply transformation and use regular WDQS
    let wdqs_query = transform_query_for_wdqs(&query);

    match format {
        DownloadFormat::Csv => {
            execute_sparql_with_format_download(
                format,
                &wdqs_query,
                WDQS_WIKIDATA,
                LotusResponseFormat::Csv,
                &filename,
                dl_timer,
            )
            .await
        }
        DownloadFormat::Json => {
            execute_sparql_with_format_download(
                format,
                &wdqs_query,
                WDQS_WIKIDATA,
                LotusResponseFormat::SparqlJson,
                &filename,
                dl_timer,
            )
            .await
        }
        DownloadFormat::Rdf => {
            // For Turtle format, SELECT queries must be wrapped in CONSTRUCT
            // (WDQS can't return Turtle for SELECT queries)
            let construct_query = format.prepared_query(&wdqs_query);
            execute_sparql_with_format_download(
                format,
                &construct_query,
                WDQS_WIKIDATA,
                LotusResponseFormat::Turtle,
                &filename,
                dl_timer,
            )
            .await
        }
    }
}

async fn execute_sparql_with_format_download(
    format: DownloadFormat,
    query: &str,
    endpoint: &str,
    response_format: LotusResponseFormat,
    filename: &str,
    dl_timer: perf::TimerHandle,
) -> Result<(), String> {
    use lotus::transport::execute_sparql_with_format as shared_execute;

    let body = shared_execute(query, endpoint, response_format)
        .await
        .map_err(|e| e.to_string())?;

    let fetch_elapsed = perf::end_timer(format.timer_label(), dl_timer);
    perf::log_timing(
        "download",
        &format!(
            "event=download format={} phase=fetch state=success source=wdqs body_bytes={}",
            format.log_name(),
            body.len()
        ),
        Some(fetch_elapsed),
    );

    let trigger_timer = perf::start_timer(&format.trigger_timer_label());
    trigger_download(filename, format.content_type(), &body);
    let trigger_elapsed = perf::end_timer(&format.trigger_timer_label(), trigger_timer);
    perf::log_timing(
        "download",
        &format!(
            "event=download format={} phase=trigger state=success source=wdqs",
            format.log_name()
        ),
        Some(trigger_elapsed),
    );
    Ok(())
}

fn handle_download_error(
    format: DownloadFormat,
    error: &str,
    dl_timer: perf::TimerHandle,
) -> Result<(), String> {
    let elapsed = perf::end_timer(format.timer_label(), dl_timer);
    perf::log_timing(
        "download",
        &format!(
            "event=download format={} phase=fetch state=error source=direct reason={}",
            format.log_name(),
            error
        ),
        Some(elapsed),
    );
    log::warn!(
        "event=download format={} phase=fetch state=error source=direct reason={}",
        format.log_name(),
        error
    );
    Err(error.to_string())
}

async fn execute_download_direct(format: DownloadFormat, query: &str) -> Result<String, String> {
    let prepared_query = format.prepared_query(query);
    match format {
        DownloadFormat::Csv => sparql::execute_query(&prepared_query)
            .await
            .map_err(|e| e.to_string()),
        DownloadFormat::Json => {
            sparql::execute_sparql_format(&prepared_query, LotusResponseFormat::SparqlJson)
                .await
                .map_err(|e| e.to_string())
        }
        DownloadFormat::Rdf => {
            sparql::execute_sparql_format(&prepared_query, LotusResponseFormat::Turtle)
                .await
                .map_err(|e| e.to_string())
        }
    }
}

pub(super) fn trigger_download(filename: &str, mime: &str, content: &str) {
    let _ = crate::upload::download_text(content, filename);
    let _ = mime;
}
