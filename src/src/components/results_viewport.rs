// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Top-level results area component using phase-driven rendering.

use crate::components::loading::{DownloadDispatchState, DownloadOnlyState, LoadingState};
use crate::components::results_table::ResultsTable;
use crate::state::use_results_context;
use crate::ui::ContentPhase;
use dioxus::prelude::*;

#[component]
pub fn ResultsViewport() -> Element {
    use crate::features::explore::ExploreUiState;
    use crate::features::explore::selectors::use_result_selector;

    let state = use_results_context();
    let explore = state.explore;
    // Hoisted to component top-level — hooks must be called unconditionally.
    let _locale = crate::hooks::use_locale();

    let ui_state = use_memo(move || ExploreUiState::from_explore(explore));

    let phase = use_memo(move || {
        let s = *ui_state.read();
        ContentPhase::from_lifecycle(
            s.loading,
            s.has_error,
            s.searched_once,
            s.download_only_mode,
            s.has_entries,
        )
    });

    // Get the SPARQL query to show even on error
    let sparql_query = use_result_selector(explore, |result| result.sparql_query.clone());
    let searched_once = use_result_selector(explore, |result| result.sparql_query.is_some());

    match *phase.read() {
        ContentPhase::Welcome => rsx! {},
        ContentPhase::Loading => rsx! {
            LoadingState {}
        },
        // Error state: show the SPARQL query that was attempted
        ContentPhase::Error => {
            let query = sparql_query.read();
            query.as_ref().map_or_else(
                || rsx! {},
                |q| {
                    rsx! { QueryDisplay { query: (*q).to_string() } }
                },
            )
        }
        ContentPhase::Empty => {
            if *searched_once.read() {
                rsx! { ResultsTable {} }
            } else {
                rsx! {}
            }
        }
        ContentPhase::Loaded => rsx! {
            ResultsTable {}
        },
        ContentPhase::DownloadOnly => {
            if ui_state.read().download_dispatching {
                rsx! {
                    DownloadDispatchState {}
                }
            } else {
                rsx! {
                    DownloadOnlyState {}
                }
            }
        }
    }
}

#[component]
fn QueryDisplay(query: String) -> Element {
    rsx! {
        section {
            id: "query-display",
            class: "page-section w-full max-w-none px-0",
            h2 { class: "text-title font-semibold text-text mb-4", "SPARQL Query" }
            pre {
                class: "m-0 max-h-96 overflow-auto font-mono text-ui text-muted whitespace-pre-wrap break-all",
                code { class: "block p-0", "{query}" }
            }
        }
    }
}
