// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Dataset statistics and the `SmilesSearchType` / `ElementState` enums.
//!
//! `DatasetStats` aggregates deduplicated counts computed from result entries.
//! `SmilesSearchType` selects substructure vs similarity search.
//! `ElementState` controls whether an element is allowed, required, or excluded.

use super::CompoundEntry;

/// Aggregated counts for a dataset, computed from CSV query results.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DatasetStats {
    pub n_compounds: usize,
    pub n_taxa: usize,
    pub n_references: usize,
    pub n_entries: usize,
    pub n_entries_unique: usize,
}

impl DatasetStats {
    /// Compute dataset statistics from a slice of result entries.
    ///
    /// Uses `&str` slices (borrowed from `Arc<str>` fields) so no strings are
    /// copied or re-allocated.  A single pass over the entries populates all
    /// deduplicated ID sets simultaneously, including unique
    /// compound-taxon-reference triples (matching the `COUNT(DISTINCT …)`
    /// computed by `query_counts_from_base`).
    #[must_use]
    pub fn from_entries(entries: &[CompoundEntry]) -> Self {
        use std::collections::HashSet;
        let mut c: HashSet<&str> = HashSet::with_capacity(entries.len());
        let mut t: HashSet<&str> = HashSet::with_capacity(entries.len());
        let mut r: HashSet<&str> = HashSet::with_capacity(entries.len());
        let mut unique_triples: HashSet<(&str, &str, &str)> = HashSet::with_capacity(entries.len());
        for e in entries {
            c.insert(e.compound_qid.as_ref());
            if !e.taxon_qid.is_empty() {
                t.insert(e.taxon_qid.as_ref());
            }
            if !e.reference_qid.is_empty() {
                r.insert(e.reference_qid.as_ref());
            }
            unique_triples.insert((
                e.compound_qid.as_ref(),
                e.taxon_qid.as_ref(),
                e.reference_qid.as_ref(),
            ));
        }
        Self {
            n_compounds: c.len(),
            n_taxa: t.len(),
            n_references: r.len(),
            n_entries: entries.len(),
            n_entries_unique: unique_triples.len(),
        }
    }
}

/// Enum controlling how structure search is performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SmilesSearchType {
    #[default]
    Substructure,
    Similarity,
}

impl SmilesSearchType {
    /// Returns the string representation used in URL parameters.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Substructure => "substructure",
            Self::Similarity => "similarity",
        }
    }
}

impl std::fmt::Display for SmilesSearchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// State of an optional element filter (F, Cl, Br, I).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ElementState {
    #[default]
    Allowed,
    Required,
    Excluded,
}

impl ElementState {
    /// Returns the string representation used in URL parameters.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Allowed => "allowed",
            Self::Required => "required",
            Self::Excluded => "excluded",
        }
    }
}

impl std::fmt::Display for ElementState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for ElementState {
    type Err = std::convert::Infallible;

    /// Parse a case-sensitive element-state string.
    ///
    /// Accepts `"required"` and `"excluded"`; all other values (including
    /// `"allowed"` and unrecognized strings) map to [`ElementState::Allowed`].
    /// This is intentionally infallible so URL parameters can always be decoded
    /// without propagating parse errors through the call stack.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "required" => Self::Required,
            "excluded" => Self::Excluded,
            _ => Self::Allowed,
        })
    }
}
