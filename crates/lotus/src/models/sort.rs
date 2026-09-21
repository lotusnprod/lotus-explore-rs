// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Sort state: column selection, direction, and their serialization.
//!
//! [`SortState`] is used by `lotus-explore-rs`'s results table and its
//! `/v1/search` endpoint to order result rows.

/// Column by which results can be sorted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Name,
    Mass,
    Formula,
    TaxonName,
    PubYear,
    RefTitle,
}

/// Sort direction: ascending or descending.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDir {
    Asc,
    Desc,
}

/// Combined sort state: which column and which direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SortState {
    pub col: SortColumn,
    pub dir: SortDir,
}

impl Default for SortState {
    fn default() -> Self {
        Self {
            col: SortColumn::Name,
            dir: SortDir::Asc,
        }
    }
}
