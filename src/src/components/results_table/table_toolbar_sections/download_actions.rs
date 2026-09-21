// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Download actions toolbar group — buttons to trigger query/metadata downloads
//! and links to open the query in the QLever UI.

use super::super::download_model::{
    DOWNLOAD_METADATA_SPEC, DOWNLOAD_QUERY_CSV_SPEC, DOWNLOAD_QUERY_JSON_SPEC,
    DOWNLOAD_QUERY_RDF_SPEC, DownloadQuerySpec, build_download_toolbar_model_with_endpoint,
};
use crate::components::ui::Button;
use crate::download::{DownloadFormat, execute_download, trigger_download};
use crate::features::explore::use_toolbar_result_snapshot;
use crate::i18n::{TextKey, t};
use crate::models::SearchCriteria;
use crate::perf;
use crate::state::use_results_context;
use dioxus::prelude::*;
use std::sync::Arc;

// ── private helpers ─────────────────────────────────────────────────────────

fn spawn_query_download(
    format: DownloadFormat,
    status_message: String,
    _criteria_snapshot: Option<Arc<SearchCriteria>>,
    filename: String,
    query: Arc<str>,
    mut download_busy: Signal<bool>,
    mut download_status: Signal<Option<String>>,
) {
    *download_busy.write() = true;
    *download_status.write() = Some(status_message);
    spawn(async move {
        log::info!(
            "event=download phase=table_dispatch state=started format={}",
            format.log_name()
        );
        log::info!(
            "event=download phase=table_query state=check format={} has_SERVICE={} query_bytes={}",
            format.log_name(),
            query.contains("SERVICE"),
            query.len()
        );
        if let Err(err) = execute_download(
            format,
            #[cfg(target_arch = "wasm32")]
            _criteria_snapshot.expect("wasm download requires criteria snapshot"),
            query,
            filename,
        )
        .await
        {
            log::warn!(
                "event=download phase=table_fetch state=error format={} reason={err}",
                format.log_name()
            );
        }
        *download_busy.write() = false;
        *download_status.write() = None;
    });
}

fn dispatch_query_download_spec(
    spec: DownloadQuerySpec,
    locale: crate::i18n::Locale,
    criteria_snapshot: Option<Arc<SearchCriteria>>,
    filename: String,
    query: Arc<str>,
    download_busy: Signal<bool>,
    download_status: Signal<Option<String>>,
) {
    spawn_query_download(
        spec.format,
        t(locale, spec.status_key).to_string(),
        criteria_snapshot,
        filename,
        query,
        download_busy,
        download_status,
    );
}

fn dispatch_metadata_download_blob(filename: &str, body: &str) {
    log::info!(
        "event=download phase=table_dispatch state=started format=metadata filename={} size={}",
        filename,
        body.len()
    );
    let trigger_timer = perf::start_timer("LOTUS:table_download_meta_trigger");
    if body.is_empty() {
        log::error!(
            "event=download phase=table_dispatch state=error format=metadata reason=empty_body"
        );
        return;
    }
    trigger_download(filename, "application/ld+json", body);
    let elapsed_ms =
        perf::end_timer("LOTUS:table_download_meta_trigger", trigger_timer).as_secs_f64() * 1000.0;
    log::info!(
        "event=download phase=table_trigger state=success format=metadata elapsed_ms={elapsed_ms:.1}"
    );
}

// ── components ─────────────────────────────────────────────────────────────

/// Displays download status with spinning indicator.
#[component]
fn DownloadStatusSpinner(
    download_status: ReadSignal<Option<String>>,
    locale: crate::i18n::Locale,
) -> Element {
    let status_msg = download_status.read().clone();
    let text = status_msg
        .as_deref()
        .unwrap_or_else(|| t(locale, TextKey::PreparingDownload));

    rsx! {
        span {
            role: "status",
            aria_live: "polite",
            class: "inline-flex items-center gap-2 rounded-xl border border-border bg-surface px-3 py-1.5 text-ui font-semibold text-muted shadow-xs",
            span { class: "spinner-sm", "aria-hidden": "true" }
            {text}
        }
    }
}

/// Download button for query results (CSV, JSON, RDF formats).
#[component]
fn DownloadQueryButton(
    spec: DownloadQuerySpec,
    sparql_query: Arc<str>,
    locale: crate::i18n::Locale,
    disabled: bool,
    download_busy: Signal<bool>,
    download_status: Signal<Option<String>>,
    criteria: ReadSignal<SearchCriteria>,
    filename: String,
) -> Element {
    let title = t(locale, spec.title_key);
    let label = t(locale, spec.label_key);

    rsx! {
        Button {
            r#type: "button",
            disabled,
            class: "inline-flex shrink-0 items-center justify-center font-sans select-none transition-transform duration-150 ease-[cubic-bezier(.4,0,.2,1)] focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-accent/28 focus-visible:ring-offset-2 rounded-xl border border-border bg-surface text-text font-semibold shadow-xs hover:bg-bg active:bg-bg min-h-9 px-4 py-1.5 text-ui active:scale-[0.98]",
            title: Some(title.to_string()),
            aria_label: Some(title.to_string()),
            label: Some(label.to_string()),
            onclick: {
                let q = sparql_query.clone();
                let fname = filename.clone();
                #[cfg(target_arch = "wasm32")]
                let criteria_snapshot = Some(Arc::new(criteria.read().clone()));
                #[cfg(not(target_arch = "wasm32"))]
                let criteria_snapshot = None;
                move |_| {
                    dispatch_query_download_spec(
                        spec,
                        locale,
                        criteria_snapshot.clone(),
                        fname.clone(),
                        q.clone(),
                        download_busy,
                        download_status,
                    );
                }
            },
        }
    }
}

/// Download button for metadata JSON file.
#[component]
fn DownloadMetadataButton(
    metadata_json: Arc<str>,
    toolbar_model: ReadSignal<
        crate::components::results_table::download_model::DownloadToolbarModel,
    >,
    locale: crate::i18n::Locale,
    disabled: bool,
) -> Element {
    let title = t(locale, DOWNLOAD_METADATA_SPEC.title_key);
    let label = t(locale, DOWNLOAD_METADATA_SPEC.label_key);

    rsx! {
        Button {
            r#type: "button",
            disabled,
            class: "inline-flex shrink-0 items-center justify-center font-sans select-none transition-transform duration-150 ease-[cubic-bezier(.4,0,.2,1)] focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-accent/28 focus-visible:ring-offset-2 rounded-xl border border-border bg-surface text-text font-semibold shadow-xs hover:bg-bg active:bg-bg min-h-9 px-4 py-1.5 text-ui active:scale-[0.98]",
            aria_label: Some(title.to_string()),
            label: Some(label.to_string()),
            onclick: {
                let body = metadata_json.clone();
                let filename = toolbar_model.read().metadata_filename.clone();
                move |_| {
                    dispatch_metadata_download_blob(&filename, body.as_ref());
                }
            },
        }
    }
}

#[component]
pub fn DownloadActionsGroup() -> Element {
    let locale = crate::hooks::use_locale();
    let explore = use_results_context().explore;

    // Each selector subscribes to exactly one field; the component only
    // re-renders when any of these specific fields change.
    let criteria = crate::features::explore::selectors::use_ui_selector(explore, |ui| {
        ui.executed_criteria.clone()
    });
    let toolbar_snapshot = use_toolbar_result_snapshot(explore);

    let snapshot = toolbar_snapshot.read();
    let toolbar_model = use_signal(|| {
        build_download_toolbar_model_with_endpoint(
            &criteria.read(),
            snapshot.sparql_query.as_deref(),
            snapshot.metadata_json.as_deref(),
            snapshot.query_hash.as_deref(),
            snapshot.result_hash.as_deref(),
            snapshot.endpoint.into(),
        )
    });

    let download_results_label = t(locale, TextKey::DownloadResults);
    let open_in_title = t(locale, TextKey::OpenInEndpointTitle);
    let _open_in_label = t(locale, TextKey::OpenInEndpoint);

    // Local download state — busy flag and status text.
    let download_busy = use_signal(|| false);
    let download_status: Signal<Option<String>> = use_signal(|| None);

    let sparql_query_value = snapshot.sparql_query.clone();
    let metadata_json_value = snapshot.metadata_json.clone();
    let export_available = toolbar_model.read().export_available;
    let ui_url = toolbar_model.read().ui_url.clone();
    let endpoint_name = toolbar_model.read().sparql_endpoint_ui.to_string();
    let ui_url_for_click = ui_url.clone();
    drop(snapshot);

    rsx! {
        nav { class: "flex w-full min-w-0 flex-wrap items-center justify-center gap-3 py-1 mb-3", aria_label: "{download_results_label}",
            if *download_busy.read() {
                DownloadStatusSpinner {
                    download_status,
                    locale,
                }
            }
            if export_available {
                ul {
                    class: "flex min-w-0 flex-wrap items-center justify-center gap-3",
                    if let Some(query) = sparql_query_value.as_ref() {
                        li {
                            DownloadQueryButton {
                                spec: DOWNLOAD_QUERY_CSV_SPEC,
                                sparql_query: query.clone(),
                                locale,
                                disabled: *download_busy.read(),
                                download_busy,
                                download_status,
                                criteria,
                                filename: toolbar_model.read().csv_filename.clone(),
                            }
                        }
                        li {
                            DownloadQueryButton {
                                spec: DOWNLOAD_QUERY_JSON_SPEC,
                                sparql_query: query.clone(),
                                locale,
                                disabled: *download_busy.read(),
                                download_busy,
                                download_status,
                                criteria,
                                filename: toolbar_model.read().json_filename.clone(),
                            }
                        }
                        li {
                            DownloadQueryButton {
                                spec: DOWNLOAD_QUERY_RDF_SPEC,
                                sparql_query: query.clone(),
                                locale,
                                disabled: *download_busy.read(),
                                download_busy,
                                download_status,
                                criteria,
                                filename: toolbar_model.read().rdf_filename.clone(),
                            }
                        }
                    }
                    if let Some(body) = metadata_json_value.as_ref() {
                        li {
                            DownloadMetadataButton {
                                metadata_json: body.clone(),
                                toolbar_model,
                                locale,
                                disabled: *download_busy.read(),
                            }
                        }
                    }
                    if let Some(_url) = ui_url_for_click.as_ref() {
                        li {
                            Button {
                                r#type: "button",
                                class: "inline-flex shrink-0 items-center justify-center font-sans select-none transition-transform duration-150 ease-[cubic-bezier(.4,0,.2,1)] focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-accent/28 focus-visible:ring-offset-2 rounded-xl border border-border bg-surface text-text font-semibold shadow-xs hover:bg-bg active:bg-bg min-h-9 px-4 py-1.5 text-ui active:scale-[0.98]",
                                title: Some(format!("{open_in_title} ({endpoint_name})")),
                                aria_label: Some(format!("{open_in_title} ({endpoint_name})")),
                                label: Some(format!("Open in {endpoint_name}")),
                                onclick: move |_| {
                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        if let Some(win) = web_sys::window() {
                                            let _ = win.open_with_url_and_target(
                                                ui_url_for_click.as_ref().unwrap(),
                                                "_blank",
                                            );
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}
