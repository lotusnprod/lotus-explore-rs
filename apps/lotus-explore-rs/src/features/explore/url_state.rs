// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

#[cfg(target_arch = "wasm32")]
use super::url_codec::build_query_string;
use crate::app::view::AppView;
use crate::i18n::Locale;
use std::collections::BTreeMap;

pub use super::url_codec::{
    InitialUrlState, build_shareable_url, is_true_flag, parse_criteria_from_params,
    parse_startup_action_from_params,
};

#[cfg(test)]
pub use super::url_codec::InitialDownloadState;

pub fn initial_url_state() -> InitialUrlState {
    let params = read_url_query_params();
    InitialUrlState {
        criteria: parse_criteria_from_params(&params),
        view: AppView::from_query_value(params.get("view").map(String::as_str)),
        locale: Locale::detect(params.get("lang").map_or("", String::as_str)),
        download: parse_startup_action_from_params(&params),
        dark_mode: params.get("dark_mode").is_some_and(|v| is_true_flag(v)),
    }
}

pub fn absolute_share_url(share: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some((origin, pathname)) = origin_and_pathname() {
            return format!("{origin}{pathname}{share}");
        }
    }
    share.into()
}

pub fn absolute_current_url_with_query(query: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some((origin, pathname)) = origin_and_pathname() {
            return if query.is_empty() {
                format!("{origin}{pathname}")
            } else {
                format!("{origin}{pathname}?{query}")
            };
        }
    }
    if query.is_empty() {
        String::new()
    } else {
        format!("?{query}")
    }
}

/// The browser's `location.origin` + `location.pathname` (no query), when
/// available. Centralizes the `web_sys::window()` lookup shared by the
/// `absolute_*` URL builders so they don't each re-open the window.
#[cfg(target_arch = "wasm32")]
fn origin_and_pathname() -> Option<(String, String)> {
    let win = web_sys::window()?;
    let loc = win.location();
    Some((loc.origin().ok()?, loc.pathname().ok()?))
}

/// Replace the current history entry with `query` (the serialized query string)
/// reflected in the address bar — shared by every `persist_*` caller so they
/// don't each repeat the build→absolute-url→`replace_state` sequence.
#[cfg(target_arch = "wasm32")]
fn replace_history_state(query: &str) {
    let url = absolute_current_url_with_query(query);
    if let Some(win) = web_sys::window()
        && let Ok(history) = win.history()
    {
        let _ = history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&url));
    }
}

pub fn persist_locale_query_param(locale: Locale) {
    // The default English locale is the site's canonical home, so it must NOT
    // be forced into the address bar as `?lang=en` — doing so rewrites every
    // English visit to a non-canonical query string that conflicts with
    // `hreflang` and `rel=canonical`. Only non-default locales are reflected in
    // the URL, so language switches and shared/bookmark links are still
    // preserved across navigation and refresh.
    #[cfg(target_arch = "wasm32")]
    {
        let mut params = read_url_query_params();
        if locale == Locale::En {
            params.remove("lang");
        } else {
            params.insert("lang".into(), locale.lang_code().into());
        }
        replace_history_state(&build_query_string(&params));
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = locale;
    }
}

pub fn persist_view_query_param(view: AppView) {
    #[cfg(target_arch = "wasm32")]
    {
        let mut params = read_url_query_params();
        if let Some(view_param) = view.query_value() {
            params.insert("view".into(), view_param.into());
        } else {
            params.remove("view");
        }
        replace_history_state(&build_query_string(&params));
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = view.query_value();
    }
}

pub fn persist_dark_mode_query_param(dark_mode: bool) {
    #[cfg(target_arch = "wasm32")]
    {
        let mut params = read_url_query_params();
        if dark_mode {
            params.insert("dark_mode".into(), "true".into());
        } else {
            params.remove("dark_mode");
        }
        replace_history_state(&build_query_string(&params));
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = dark_mode;
    }
}

pub fn read_url_query_params() -> BTreeMap<String, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let mut out = BTreeMap::new();
        let Some(window) = web_sys::window() else {
            return out;
        };
        let Ok(search) = window.location().search() else {
            return out;
        };
        let query = search.trim_start_matches('?');
        for pair in query.split('&') {
            if pair.is_empty() {
                continue;
            }
            let (key, val) = pair.split_once('=').unwrap_or((pair, ""));
            let key_decoded =
                urlencoding::decode(key).map_or_else(|_| key.into(), std::borrow::Cow::into_owned);
            let val_decoded =
                urlencoding::decode(val).map_or_else(|_| val.into(), std::borrow::Cow::into_owned);
            out.insert(key_decoded, val_decoded);
        }
        out
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        BTreeMap::new()
    }
}
