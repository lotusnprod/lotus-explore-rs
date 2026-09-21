// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! LOTUS compound entries, taxon matches, and the shared `Arc<str>` row type.
//!
//! `CompoundEntry` is the primary result-row type returned by the SPARQL
//! parsing layer ([`crate::sparql`]).  All string fields use `Arc<str>` to
//! enable cheap cloning and string interning via [`crate::sparql::StrInterner`].

#![allow(missing_docs)] // fields are self-documenting from struct member names

use std::sync::Arc;

use super::WIKIDATA_STATEMENT_BASE;

/// A single deduplicated compound-taxon-reference result row.
///
/// Every field is an `Arc<str>` (or `Option<Arc<str>>`) so rows can be cloned
/// cheaply for UI rendering, sorting, and pagination without re-allocating
/// string data.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CompoundEntry {
    pub compound_qid: Arc<str>,
    pub name: Arc<str>,
    pub inchikey: Option<Arc<str>>,
    pub smiles: Option<Arc<str>>,
    pub mass: Option<f64>,
    pub formula: Option<Arc<str>>,
    pub taxon_qid: Arc<str>,
    pub taxon_name: Arc<str>,
    pub reference_qid: Arc<str>,
    pub ref_title: Option<Arc<str>>,
    pub ref_doi: Option<Arc<str>>,
    pub pub_year: Option<i16>,
    pub statement: Option<Arc<str>>,
}

/// Shared, cheaply-cloneable slice type for compound result rows.
pub type Rows = Arc<[CompoundEntry]>;

/// A single taxon-search match (QID + display name).
#[derive(Debug, Clone)]
pub struct TaxonMatch {
    pub qid: String,
    pub name: String,
}

impl CompoundEntry {
    /// Returns the DOI string (trimmed), if present and non-empty.
    pub fn doi(&self) -> Option<&str> {
        self.ref_doi
            .as_deref()
            .map(str::trim)
            .filter(|d| !d.is_empty())
    }

    /// Returns a full `https://doi.org/{doi}` URL, if a DOI is present.
    #[must_use]
    pub fn doi_url(&self) -> Option<String> {
        self.doi().map(|d| format!("https://doi.org/{d}"))
    }

    /// Returns a `CDKDEPict` SVG URL for the entry's `SMILES`, if available and
    /// single-line (multi-line SMILES would break the URL).
    #[must_use]
    pub fn depict_url(&self) -> Option<String> {
        let smiles = self.smiles.as_deref()?.trim();
        if smiles.is_empty() || smiles.contains('\n') {
            return None;
        }
        Some(format!(
            "https://www.simolecule.com/cdkdepict/depict/cow/svg?smi={}&annotate=cip",
            urlencoding::encode(smiles)
        ))
    }

    /// Returns the bare statement ID (e.g. `S1`), stripping the Wikidata
    /// statement URI prefix if present.
    pub fn statement_id_str(&self) -> Option<&str> {
        let raw = self.statement.as_deref().map(str::trim)?;
        if raw.is_empty() {
            return None;
        }
        Some(raw.strip_prefix(WIKIDATA_STATEMENT_BASE).unwrap_or(raw))
    }

    /// Returns the bare statement ID as an owned `String`.
    #[must_use]
    pub fn statement_id(&self) -> Option<String> {
        self.statement_id_str().map(str::to_owned)
    }
}
