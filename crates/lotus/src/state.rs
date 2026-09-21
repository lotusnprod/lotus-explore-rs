// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Cache-key computation shared by the native server and the WASM client.
//!
//! Both the `--features server` HTTP layer and the in-browser result cache
//! derive cache keys from the same functions here, so a key produced by one is
//! valid for the other.

use sha2::{Digest, Sha256};

/// Cache key for a search result page, derived from the canonical SPARQL
/// `query`, display `limit`, and whether per-page counts are included.
#[must_use]
pub fn build_search_cache_key(query: &str, limit: usize, include_counts: bool) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"search");
    hasher.update(limit.to_le_bytes());
    hasher.update([u8::from(include_counts)]);
    hasher.update(query.as_bytes());
    format!("search:{}", sha256_hex(hasher.finalize()))
}

/// Cache key for an export request, derived from the SPARQL `query` (the
/// format/action is cheap to recompute relative to re-executing the query).
#[must_use]
pub fn build_export_cache_key(query: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"export");
    hasher.update(query.as_bytes());
    format!("export:{}", sha256_hex(hasher.finalize()))
}

/// Lowercase hex encoding of a finalized digest.
fn sha256_hex(bytes: impl AsRef<[u8]>) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let bytes = bytes.as_ref();
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_inputs_produce_same_key() {
        let q = "SELECT ?s WHERE { ?s ?p ?o }";
        assert_eq!(
            build_search_cache_key(q, 100, true),
            build_search_cache_key(q, 100, true)
        );
    }

    #[test]
    fn keys_are_distinct_by_query_limit_and_kind() {
        let q = "SELECT ?s WHERE { ?s ?p ?o }";
        assert_ne!(
            build_search_cache_key(q, 100, true),
            build_export_cache_key(q)
        );
        assert_ne!(
            build_search_cache_key(q, 100, true),
            build_search_cache_key(q, 200, true)
        );
        assert!(build_search_cache_key(q, 100, true).starts_with("search:"));
        assert!(build_export_cache_key(q).starts_with("export:"));
    }
}
