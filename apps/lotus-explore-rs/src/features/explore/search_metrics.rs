// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Execution-time metrics for a single search pipeline run.
//!
//! [`SearchMetrics`] accumulates wall-clock timings for the network and
//! parsing phases of one search request.  It is created fresh per request,
//! passed by `&mut` through the pipeline, then consumed by
//! [`emit_search_summary`] to produce a single structured log line.
//!
//! ## Why a separate module?
//!
//! Previously this lived in `search_state.rs` alongside the Dioxus application
//! state.  Execution metrics are ephemeral data that exist only during a single
//! async search task; they have no business being adjacent to the persistent
//! `ExploreState` reducer.  Separating them makes both modules leaner and
//! keeps platform-specific `#[cfg]` gating (the parallel-network variant is
//! WASM-only) from contaminating the main state module.

use crate::services::search_telemetry as telemetry;

/// Wall-clock timings accumulated during a single search execution.
///
/// All durations are summed; for WASM parallel fetches
/// `add_parallel_network` accepts an already-overlapped elapsed value so
/// the total reflects wall time, not summed sequential time.
#[derive(Default, Clone, Copy)]
pub struct SearchMetrics {
    /// Total wall time spent waiting for network responses (ms).
    pub network_ms: f64,
    /// Total wall time spent parsing CSV/JSON payloads (ms).
    pub parse_ms: f64,
    /// Number of distinct SPARQL/REST calls issued.
    pub sparql_calls: usize,
}

impl SearchMetrics {
    /// Record a completed sequential network call.
    pub fn add_network(&mut self, elapsed: std::time::Duration) {
        self.network_ms = elapsed.as_secs_f64().mul_add(1000.0, self.network_ms);
        self.sparql_calls += 1;
    }

    /// Record a completed parse phase.
    pub fn add_parse(&mut self, elapsed: std::time::Duration) {
        self.parse_ms = elapsed.as_secs_f64().mul_add(1000.0, self.parse_ms);
    }

    /// Record an already-overlapped (parallel) network batch: add the batch's
    /// wall-clock duration once, and the number of calls bundled in it.
    /// WASM-only per the parallel-fetch timing design (see struct docs).
    #[cfg(target_arch = "wasm32")]
    pub fn add_parallel_network(&mut self, elapsed: std::time::Duration, n_calls: usize) {
        self.network_ms = elapsed.as_secs_f64().mul_add(1000.0, self.network_ms);
        self.sparql_calls += n_calls;
    }
}

/// Searches taking longer than this threshold are flagged as slow queries in
/// the telemetry log.
const SLOW_QUERY_THRESHOLD_MS: f64 = 5_000.0;

/// Emit a structured summary log line for a completed search.
///
/// If `total_elapsed` exceeds [`SLOW_QUERY_THRESHOLD_MS`] an additional
/// `slow_query` warning is emitted so dashboards can alert on regressions.
pub fn emit_search_summary(total_elapsed: std::time::Duration, metrics: SearchMetrics) {
    let total_ms = total_elapsed.as_secs_f64() * 1000.0;
    let details = format!(
        "total_ms={total_ms:.1} network_ms={:.1} parse_ms={:.1} sparql_calls={}",
        metrics.network_ms, metrics.parse_ms, metrics.sparql_calls
    );
    telemetry::search_summary_done(&details);
    if total_ms >= SLOW_QUERY_THRESHOLD_MS {
        telemetry::search_summary_slow_query(&details);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn add_network_accumulates_and_increments_call_count() {
        let mut m = SearchMetrics::default();
        m.add_network(Duration::from_millis(100));
        m.add_network(Duration::from_millis(200));
        assert!((m.network_ms - 300.0).abs() < 1.0);
        assert_eq!(m.sparql_calls, 2);
    }

    #[test]
    fn add_parse_accumulates_independently() {
        let mut m = SearchMetrics::default();
        m.add_parse(Duration::from_millis(50));
        m.add_parse(Duration::from_millis(75));
        assert!((m.parse_ms - 125.0).abs() < 1.0);
        assert_eq!(m.sparql_calls, 0); // parse doesn't count calls
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn add_parallel_network_counts_by_calls_arg() {
        let mut m = SearchMetrics::default();
        m.add_parallel_network(Duration::from_millis(200), 2);
        assert!((m.network_ms - 200.0).abs() < 1.0);
        assert_eq!(m.sparql_calls, 2);
    }
}
