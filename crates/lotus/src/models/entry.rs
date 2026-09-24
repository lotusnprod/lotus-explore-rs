// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! LOTUS compound entries, taxon matches, and the shared `Arc<str>` row type.
//!
//! `CompoundEntry` is the primary result-row type returned by the SPARQL
//! parsing layer ([`crate::sparql`]).  All string fields use `Arc<str>` to
//! enable cheap cloning and string interning via [`crate::sparql::StrInterner`].

#![allow(missing_docs)] // fields are self-documenting from struct member names

use std::sync::Arc;

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
