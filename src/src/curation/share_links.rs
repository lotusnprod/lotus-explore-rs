// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

use super::*;
use std::collections::BTreeMap;

pub const CURATION_ROWS_PARAM: &str = "curation_rows";
const CURATION_RUN_PARAM: &str = "curation_run";

pub fn initial_curation_rows_from_url() -> Vec<CurationInputRow> {
    let params = read_curation_url_query_params();
    curation_rows_from_query_params(&params)
}

pub fn initial_curation_autorun_from_url() -> bool {
    let params = read_curation_url_query_params();
    params
        .get(CURATION_RUN_PARAM)
        .is_some_and(|value| is_true_flag(value))
}

pub fn build_curation_share_url(
    rows: &[CurationInputRow],
    locale: Locale,
    autorun: bool,
) -> Option<String> {
    if rows.is_empty() {
        return None;
    }
    let params = curation_query_params(rows, locale, autorun);
    let query = params
        .into_iter()
        .map(|(key, value)| {
            format!(
                "{}={}",
                urlencoding::encode(&key),
                urlencoding::encode(&value)
            )
        })
        .collect::<Vec<_>>()
        .join("&");
    Some(format!("?{query}"))
}

fn curation_query_params(
    rows: &[CurationInputRow],
    locale: Locale,
    autorun: bool,
) -> BTreeMap<String, String> {
    let mut params = BTreeMap::new();
    params.insert("view".into(), "curation-explorer".into());
    params.insert(
        "lang".into(),
        match locale {
            Locale::En => "en",
            Locale::Fr => "fr",
            Locale::De => "de",
            Locale::It => "it",
        }
        .into(),
    );
    params.insert(
        CURATION_ROWS_PARAM.into(),
        serde_json::to_string(rows).unwrap_or_else(|_| String::from("[]")),
    );
    if autorun {
        params.insert(CURATION_RUN_PARAM.into(), "true".into());
    }
    params
}

pub fn curation_rows_from_query_params(params: &BTreeMap<String, String>) -> Vec<CurationInputRow> {
    params
        .get(CURATION_ROWS_PARAM)
        .and_then(|raw| serde_json::from_str::<Vec<CurationInputRow>>(raw).ok())
        .unwrap_or_default()
}

fn is_true_flag(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn read_curation_url_query_params() -> BTreeMap<String, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let mut out = BTreeMap::new();
        let Some(window) = web_sys::window() else {
            return out;
        };
        let Ok(search) = window.location().search() else {
            return out;
        };
        for pair in search.trim_start_matches('?').split('&') {
            if pair.is_empty() {
                continue;
            }
            let mut parts = pair.splitn(2, '=');
            let key = parts.next().unwrap_or_default();
            let value = parts.next().unwrap_or_default();
            let key = urlencoding::decode(key)
                .map_or_else(|_| key.into(), |decoded| decoded.into_owned());
            let value = urlencoding::decode(value)
                .map_or_else(|_| value.into(), |decoded| decoded.into_owned());
            out.insert(key, value);
        }
        out
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        BTreeMap::new()
    }
}
