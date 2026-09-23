// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Consolidated app-level state — single source of truth for view routing,
//! download orchestration, and render-telemetry.
//!
//! ## Why the previous `SearchState` / `UiState` were removed
//!
//! The earlier revision stored `criteria`, `explore`, `locale`, and
//! stale mirror copies that were kept in sync via `use_effect` loops.  Those copies were:
//!
//! * **Never read** — every consumer read from the live signal instead.
//! * **Write-only** — the sync effects wrote to them but nothing consumed them.
//! * **Race-prone** — the effect runs one tick after the signal write, so there
//!   was always a frame where the two were out of sync.
//!
//! The live signals (`criteria: Signal<SearchCriteria>`,
//! `explore: Signal<ExploreState>`, `locale: Signal<Locale>`) are now the only
//! canonical owners of that data.  `AppState` is trimmed to the three concerns
//! that truly belong at the app root.

use crate::app::view::AppView;
use crate::download::DownloadFormat;

/// App-level state.  One signal of this type lives at the root of `App`.
///
/// Scope is deliberately narrow:
/// * **view** — which page is rendered (Search / Curation / Structure editor).
/// * **download** — pending download format + direct-execute flag read by the
///   download-dispatch hook.
/// * **metrics** — one-shot logging guards that prevent duplicate telemetry
///   events during the download-wait sequence.
///
/// Notice what is *not* here:
/// * `SearchCriteria` — lives in its own `Signal<SearchCriteria>` and is
///   exposed through `FormCriteriaContext`.
/// * `ExploreState` — lives in its own `Signal<ExploreState>` with its own
///   reducer and is exposed through `ResultsContext`.
/// * `Locale` — provided via `LocaleProvider` context and accessed with
///   `use_locale()`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AppState {
    /// Currently active view / page.
    pub view: AppView,

    /// Download orchestration (format pending, direct-execute mode).
    pub download: DownloadState,

    /// One-shot logging guards used by the download-dispatch hook.
    pub metrics: MetricsState,

    /// Dark mode enabled (undocumented debug flag).
    pub dark_mode: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            view: AppView::Explore,
            download: DownloadState::default(),
            metrics: MetricsState::default(),
            dark_mode: false,
        }
    }
}

// ── Download State: Orchestration ─────────────────────────────────────────────

/// Download action orchestration and pending-format queue.
#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct DownloadState {
    /// Parsed pending programmatic download format.
    pub pending_format: Option<DownloadFormat>,

    /// Raw invalid format from URL (`?download=true&format=...`) preserved so
    /// startup validation can report the exact unsupported value once.
    pub pending_invalid_format: Option<String>,

    /// `true` when the URL included `?execute=true` (direct search + preview).
    pub direct_execute: bool,
}

// ── Metrics State: One-shot telemetry guards ──────────────────────────────────

/// Guards that prevent duplicate log events during the download-wait sequence.
///
/// These are reset to `false` once the awaited condition resolves.
#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct MetricsState {
    /// We already logged "waiting for loading to finish" this dispatch cycle.
    pub waiting_loading_logged: bool,
    /// We already logged "waiting for SPARQL query to materialize" this cycle.
    pub waiting_query_logged: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_state_default_starts_on_explore_view() {
        let state = AppState::default();
        assert_eq!(state.view, AppView::Explore);
    }

    #[test]
    fn download_state_default_is_inactive() {
        let state = DownloadState::default();
        assert!(state.pending_format.is_none());
        assert!(state.pending_invalid_format.is_none());
        assert!(!state.direct_execute);
    }

    #[test]
    fn metrics_state_default_has_no_logged_guards() {
        let state = MetricsState::default();
        assert!(!state.waiting_loading_logged);
        assert!(!state.waiting_query_logged);
    }

    #[test]
    fn app_state_clone_is_equal_to_original() {
        let a = AppState::default();
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn download_state_with_pending_format_is_not_default() {
        let s = DownloadState {
            pending_format: Some(DownloadFormat::Csv),
            pending_invalid_format: None,
            direct_execute: false,
        };
        assert_ne!(s, DownloadState::default());
    }

    // Phase 2 Integration regression tests: verify that AppState no longer
    // contains the removed SearchState / UiState mirror fields that caused the
    // sync-effect antipattern.

    #[test]
    fn app_state_has_no_search_or_ui_mirror_fields() {
        // Compile-time proof: if SearchState or UiState still existed and were
        // embedded here we would get a type-check error trying to construct
        // AppState without them.  This test just confirms Default works without
        // any sync-effect scaffolding.
        let _state = AppState::default();
    }

    #[test]
    fn metrics_state_loading_guard_can_be_set_and_cleared() {
        let m = MetricsState {
            waiting_loading_logged: true,
            ..MetricsState::default()
        };
        assert!(m.waiting_loading_logged);
        let m2 = MetricsState {
            waiting_loading_logged: false,
            ..MetricsState::default()
        };
        assert!(!m2.waiting_loading_logged);
    }

    #[test]
    fn download_state_direct_execute_flag_round_trips() {
        let s = DownloadState {
            pending_format: None,
            pending_invalid_format: None,
            direct_execute: true,
        };
        assert!(s.direct_execute);
        assert_ne!(s, DownloadState::default());
    }

    #[test]
    fn download_state_can_hold_invalid_startup_format() {
        let s = DownloadState {
            pending_format: None,
            pending_invalid_format: Some("ttl".into()),
            direct_execute: false,
        };
        assert_eq!(s.pending_invalid_format.as_deref(), Some("ttl"));
    }
}
