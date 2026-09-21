// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Pagination and dataset-statistics query builders.
//!
//! These operate on already-constructed query strings — they don't know about
//! the internal structure of the base query beyond finding the `SELECT`
//! keyword.

use crate::queries::consts::{PROPERTIES_OPTIONAL, REFERENCE_METADATA_OPTIONAL};

/// Generate a dataset statistics query from a base compound query.
///
/// **Metrics:**
/// - `n_entries`: total result triples (including duplicates)
/// - `n_entries_unique`: unique compound-taxon-reference combinations
/// - `n_compounds`: distinct compounds (Wikidata entities)
/// - `n_taxa`: distinct organisms
/// - `n_references`: distinct evidence sources
///
/// Uses `COUNT(DISTINCT …)` to compute cardinality without materializing
/// full result sets.  The base query's `SELECT`/`WHERE` is otherwise preserved,
/// so all filtering / search logic is retained.
///
/// # Why the display `OPTIONAL`s are stripped
///
/// Only the **core** compound-taxon-reference triples are counted.  The
/// display-only `OPTIONAL` blocks — `REFERENCE_METADATA_OPTIONAL` (titles /
/// DOIs / dates) and `PROPERTIES_OPTIONAL` (ISO SMILES, monoisotopic mass,
/// formula, `rdfs:label` lookup) — are removed before the count is wrapped,
/// because:
///
/// 1. They never change the distinct-entry cardinality the metrics above
///    report — the `COUNT(DISTINCT CONCAT(STR(?compound), COALESCE(STR(?taxon),
///    ""), COALESCE(STR(?ref_qid), "")))`, `COUNT(DISTINCT ?compound)`, etc.
///    all key off the core binding triples, not the display metadata.
/// 2. `PROPERTIES_OPTIONAL` alone forces `QLever` to scan **every** `rdfs:label`
///    per compound (two `FILTER(LANG(…))` passes) just to derive a count —
///    making this single `POST` the slowest and most RAM-hungry request in a
///    mobile-phone WASM search, and (when it raced the display query) the
///    trigger for the Qlever 429 storm.
///
/// Keeping the count sequential + best-effort (returns `None` on 429, falling
/// back to the display row count — see `fetch_results/wasm.rs` and
/// `CURATION_CONCURRENCY = 1`) and stripping these `OPTIONAL`s keeps it cheap,
/// accurate and 429-free.  Unbound display variables project as `NULL`/`""`
/// (`STR(unbound)` is `""` in `QLever`), so the `BIND`s in the base SELECT degrade
/// gracefully rather than erroring.
#[must_use]
pub fn query_counts_from_base(base_query: &str) -> String {
    let Some(select_pos) = base_query.find("SELECT") else {
        return base_query.to_string();
    };
    let prefixes = &base_query[..select_pos];
    // Strip the display-only `OPTIONAL` blocks so the count only walks the
    // cheap core compound-taxon-reference triples (see the doc comment above).
    // The const text is embedded verbatim by the query builders, so a literal
    // `replace` matches exactly and is a no-op when the blocks are absent.
    let stripped = base_query[select_pos..]
        .replace(REFERENCE_METADATA_OPTIONAL, "")
        .replace(PROPERTIES_OPTIONAL, "");
    let inner_select = stripped.trim();

    format!(
        r#"{prefixes}
SELECT
  (COUNT(*) AS ?n_entries)
  (COUNT(DISTINCT CONCAT(
    STR(?compound), "\u001F", COALESCE(STR(?taxon), ""), "\u001F", COALESCE(STR(?ref_qid), "")
  )) AS ?n_entries_unique)
  (COUNT(DISTINCT ?compound) AS ?n_compounds)
  (COUNT(DISTINCT ?taxon) AS ?n_taxa)
  (COUNT(DISTINCT ?ref_qid) AS ?n_references)
WHERE {{
  {{
    {inner_select}
  }}
}}"#
    )
}

/// Append a `LIMIT` clause to a base query for pagination or sampling.
///
/// # Use Cases
///
/// - Pagination: fetch first N results, then apply `OFFSET` for next page
/// - Sampling: `LIMIT 100` for quick exploratory queries
/// - UI constraints: avoid overwhelming clients with massive result sets
#[must_use]
pub fn query_with_limit(base_query: &str, limit: usize) -> String {
    let trimmed = base_query.trim_end();
    format!("{trimmed}\nLIMIT {limit}")
}
