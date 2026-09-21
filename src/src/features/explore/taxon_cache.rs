// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! In-process taxon name → QID cache.
//!
//! Avoids re-querying Wikidata for the same taxon name within a single
//! browser session. The cache is intentionally simple: it is never evicted
//! and is bounded by the number of distinct taxon names searched during the
//! session, which is expected to be small.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

type TaxonCache = HashMap<String, String>;

fn taxon_cache() -> &'static Mutex<TaxonCache> {
    static CACHE: OnceLock<Mutex<TaxonCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Returns the cached QID for the given taxon `name`, or `None` if not cached.
pub fn lookup(name: &str) -> Option<String> {
    let key = name.trim().to_lowercase();
    if key.is_empty() {
        return None;
    }
    let guard = taxon_cache().lock().ok()?;
    guard.get(&key).cloned()
}

/// Stores `qid` in the cache under the normalised form of `name`.
pub fn store(name: &str, qid: &str) {
    let key = name.trim().to_lowercase();
    if key.is_empty() || qid.trim().is_empty() {
        return;
    }
    if let Ok(mut guard) = taxon_cache().lock() {
        guard.insert(key, qid.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_returns_none_for_empty_key() {
        assert!(lookup("").is_none());
        assert!(lookup("   ").is_none());
    }

    #[test]
    fn store_and_lookup_roundtrip() {
        store("Gentiana lutea", "Q2598745");
        let result = lookup("gentiana lutea");
        assert_eq!(result.as_deref(), Some("Q2598745"));
    }

    #[test]
    fn store_ignores_empty_qid() {
        store("Somespecies", "");
        assert!(lookup("somespecies").is_none());
    }
}
