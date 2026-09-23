// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! CSV / string utility helpers used by result-set parsing.
//!
//! These operate on the `csv` crate's [`csv::StringRecord`] type as well as
//! raw `&str` slices from Wikidata/Wikidata entity URIs.

/// Index of a named header column (None if absent).
#[must_use]
pub fn col_idx(headers: &csv::StringRecord, name: &str) -> Option<usize> {
    headers.iter().position(|h| h == name)
}

/// Get a trimmed field value by optional column index.
#[must_use]
pub fn field(record: &csv::StringRecord, idx: Option<usize>) -> &str {
    idx.and_then(|i| record.get(i)).unwrap_or("").trim()
}

/// Strip the Wikidata entity URI prefix to get a bare QID (e.g. `Q12345`).
///
/// Accepts:
/// * Full canonical URIs:  `http://www.wikidata.org/entity/Q12345`  ([`WIKIDATA_ENTITY_BASE`])
/// * HTTPS variant:        `https://www.wikidata.org/entity/Q12345`
/// * Bare QIDs:            `Q12345`
///
/// Returns an empty string for any unrecognized format.
///
/// [`WIKIDATA_ENTITY_BASE`]: crate::models::WIKIDATA_ENTITY_BASE
#[must_use]
pub fn extract_qid(s: &str) -> String {
    use crate::models::WIKIDATA_ENTITY_BASE;
    const WIKIDATA_ENTITY_BASE_HTTPS: &str = "https://www.wikidata.org/entity/";

    let candidate = s
        .strip_prefix(WIKIDATA_ENTITY_BASE)
        .or_else(|| s.strip_prefix(WIKIDATA_ENTITY_BASE_HTTPS))
        .unwrap_or(s)
        .trim();

    // All QID characters are ASCII — check bytes instead of chars to avoid
    // the full Unicode iterator overhead.
    let bytes = candidate.as_bytes();
    if bytes.first() == Some(&b'Q')
        && bytes.len() > 1
        && bytes
            .get(1..)
            .is_some_and(|rest| rest.iter().all(u8::is_ascii_digit))
    {
        candidate.to_string()
    } else {
        String::new()
    }
}

/// Return `Some(s)` only if `s` is non-empty after trimming.
#[must_use]
pub fn non_empty(s: &str) -> Option<&str> {
    let t = s.trim();
    if t.is_empty() { None } else { Some(t) }
}

/// Prefer `a`, fall back to `b`, return None if both empty.
#[must_use]
pub fn coalesce<'a>(a: &'a str, b: &'a str) -> Option<&'a str> {
    non_empty(a).or_else(|| non_empty(b))
}

/// Parse `2021-04-23T00:00:00Z` or `2021` → year as i32.
#[must_use]
pub fn parse_year(s: &str) -> Option<i32> {
    s.trim().split(['-', 'T']).next()?.trim().parse().ok()
}

/// Normalise a DOI: strip `https://doi.org/` prefix if present.
#[must_use]
pub fn clean_doi(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    if let Some(doi) = t.split("doi.org/").last() {
        let doi = doi.trim();
        if !doi.is_empty() {
            return Some(doi.to_string());
        }
    }
    Some(t.to_string())
}
