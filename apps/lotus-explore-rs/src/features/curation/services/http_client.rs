// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project
#![allow(clippy::wildcard_imports)]

use super::*;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::OnceLock;

#[cfg(target_arch = "wasm32")]
use js_sys::{Function, JSON, Promise, Reflect};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn natprod_client() -> Result<&'static reqwest::Client, CurationError> {
    static CLIENT: OnceLock<Result<reqwest::Client, String>> = OnceLock::new();
    match CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .build()
            .map_err(|e| e.to_string())
    }) {
        Ok(client) => Ok(client),
        Err(message) => Err(CurationError::Http(format!(
            "failed to initialize curation HTTP client: {message}"
        ))),
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
pub(super) struct BatchConvertResponse {
    pub(super) results: Vec<BatchConvertItem>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
pub(super) struct BatchConvertItem {
    pub(super) output: String,
    pub(super) success: bool,
    pub(super) error: String,
}

#[cfg(target_arch = "wasm32")]
pub(super) async fn rdkit_bridge_call(
    method: &str,
    smiles: &str,
) -> Result<JsValue, CurationError> {
    let window = web_sys::window().ok_or_else(|| {
        CurationError::Http("window is unavailable; rdkit.js bridge cannot be used".into())
    })?;
    let window_value = JsValue::from(window);
    let bridge = Reflect::get(&window_value, &JsValue::from_str("__lotusRdkit"))
        .map_err(|_| CurationError::Http("rdkit.js bridge lookup failed".into()))?;
    if bridge.is_null() || bridge.is_undefined() {
        return Err(CurationError::Http(
            "rdkit.js bridge is unavailable; ensure RDKit assets are loaded".into(),
        ));
    }

    let ready = Reflect::get(&bridge, &JsValue::from_str("ready"))
        .map_err(|_| CurationError::Http("rdkit.js readiness hook missing".into()))?;
    let ready = match ready.dyn_into::<Function>() {
        Ok(function) => function.call0(&bridge).map_err(|err| {
            CurationError::Http(format!("rdkit.js readiness call failed: {err:?}"))
        })?,
        Err(ready) => ready,
    };
    if let Ok(promise) = ready.dyn_into::<Promise>() {
        JsFuture::from(promise).await.map_err(|err| {
            CurationError::Http(format!("rdkit.js failed to initialize: {err:?}"))
        })?;
    }

    let function = Reflect::get(&bridge, &JsValue::from_str(method))
        .map_err(|_| CurationError::Http(format!("rdkit.js method '{method}' not found")))?
        .dyn_into::<Function>()
        .map_err(|_| CurationError::Http(format!("rdkit.js method '{method}' is not callable")))?;

    let result = function
        .call1(&bridge, &JsValue::from_str(smiles))
        .map_err(|err| CurationError::Http(format!("rdkit.js {method} call failed: {err:?}")))?;

    // `dyn_into` consumes `result` and returns `Err(original)` on type mismatch,
    // so we can avoid cloning by matching on the returned value.
    match result.dyn_into::<Promise>() {
        Ok(promise) => JsFuture::from(promise)
            .await
            .map_err(|err| CurationError::Http(format!("rdkit.js {method} failed: {err:?}"))),
        Err(val) => Ok(val),
    }
}

#[cfg(target_arch = "wasm32")]
pub(super) fn js_value_to_json(value: &JsValue) -> Result<Value, CurationError> {
    let text = JSON::stringify(value)
        .ok()
        .and_then(|s| s.as_string())
        .ok_or_else(|| CurationError::Parse("rdkit.js returned a non-serializable value".into()))?;
    serde_json::from_str(&text)
        .map_err(|e| CurationError::Parse(format!("rdkit.js JSON parse error: {e}")))
}
