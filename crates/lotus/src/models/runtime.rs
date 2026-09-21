// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Runtime configuration: row limits and current-year caching.
//!
//! These values depend on the target architecture (WASM vs native) and are
//! therefore not pure constants — they're computed once and cached.

use std::sync::OnceLock;

use super::TABLE_ROW_LIMIT;

/// Cached current year — computed once at first call, then reused so that
/// `SystemTime::now()` / `js_sys::Date` is only hit a single time.
pub static CURRENT_YEAR_CACHE: OnceLock<u16> = OnceLock::new();

/// Returns the current calendar year as a `u16`.
///
/// On native targets this uses `SystemTime::now()`; on WASM, it falls back to
/// `js_sys::Date`.  The result is memoized in [`CURRENT_YEAR_CACHE`] so the
/// syscall / JS interop only happens once per process.
///
/// The year is clamped to `[1800, u16::MAX]`.  `1800` matches
/// [`DEFAULT_YEAR_MIN`] — any earlier year indicates a clock skew issue.
///
/// [`DEFAULT_YEAR_MIN`]: super::DEFAULT_YEAR_MIN
#[must_use]
pub fn current_year() -> u16 {
    *CURRENT_YEAR_CACHE.get_or_init(|| {
        #[cfg(target_arch = "wasm32")]
        {
            js_sys::Date::new_0().get_full_year().min(u16::MAX as u32) as u16
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            let secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0_i64, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX));
            let year = (1970 + secs / 31_556_952).clamp(0, i64::from(u16::MAX));
            u16::try_from(year).unwrap_or(u16::MAX)
        }
    })
}

/// Returns the maximum number of table rows to display at runtime.
///
/// On WASM, this scales based on `navigator.deviceMemory` and mobile UA
/// detection.  On native, returns [`TABLE_ROW_LIMIT`].
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn runtime_table_row_limit() -> usize {
    // Keep wasm conservative by default while still scaling on capable devices.
    let mut limit = 500usize;
    if let Some(win) = web_sys::window() {
        let win_js = wasm_bindgen::JsValue::from(win);
        if let Ok(nav) =
            js_sys::Reflect::get(&win_js, &wasm_bindgen::JsValue::from_str("navigator"))
        {
            if let Ok(mem) =
                js_sys::Reflect::get(&nav, &wasm_bindgen::JsValue::from_str("deviceMemory"))
                && let Some(gb) = mem.as_f64()
            {
                if gb <= 2.0 {
                    limit = 220;
                } else if gb <= 4.0 {
                    limit = 360;
                } else if gb >= 8.0 {
                    limit = 800;
                }
            }

            if let Ok(ua) =
                js_sys::Reflect::get(&nav, &wasm_bindgen::JsValue::from_str("userAgent"))
                && let Some(ua) = ua.as_string()
            {
                let ua = ua.to_ascii_lowercase();
                let mobile = ua.contains("iphone")
                    || ua.contains("ipad")
                    || ua.contains("android")
                    || ua.contains("mobile");
                if mobile {
                    limit = limit.min(280);
                }
            }
        }
    }
    limit.clamp(180, TABLE_ROW_LIMIT)
}

#[cfg(not(target_arch = "wasm32"))]
/// Returns the maximum number of table rows to display at runtime.
///
/// On native, returns [`TABLE_ROW_LIMIT`].
#[must_use]
pub const fn runtime_table_row_limit() -> usize {
    TABLE_ROW_LIMIT
}
