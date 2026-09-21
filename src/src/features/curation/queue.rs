// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

use crate::curation::{CurationInputRow, row_uniqueness_key};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppendOutcome {
    pub added: usize,
    pub skipped: usize,
}

pub fn non_empty_trimmed(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.into())
    }
}

pub fn append_unique_rows(
    queue: &mut Vec<CurationInputRow>,
    candidates: impl IntoIterator<Item = CurationInputRow>,
) -> AppendOutcome {
    let mut seen = queue
        .iter()
        .map(row_uniqueness_key)
        .collect::<HashSet<String>>();

    let mut added = 0usize;
    let mut skipped = 0usize;

    for row in candidates {
        let key = row_uniqueness_key(&row);
        if seen.insert(key) {
            queue.push(row);
            added += 1;
        } else {
            skipped += 1;
        }
    }

    AppendOutcome { added, skipped }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_empty_trimmed_returns_none_for_blank_values() {
        assert_eq!(non_empty_trimmed("   \n\t"), None);
    }

    #[test]
    fn non_empty_trimmed_keeps_meaningful_text() {
        assert_eq!(
            non_empty_trimmed("  Gentiana lutea  "),
            Some("Gentiana lutea".into())
        );
    }

    #[test]
    fn append_unique_rows_skips_existing_and_duplicate_candidates() {
        let mut queue = vec![CurationInputRow {
            name: "A".into(),
            smiles: "CCO".into(),
            taxon: Some("Taxon".into()),
            doi: Some("10.1/ABC".into()),
        }];

        let outcome = append_unique_rows(
            &mut queue,
            vec![
                CurationInputRow {
                    name: "A-duplicate-1".into(),
                    smiles: "CCO".into(),
                    taxon: Some("taxon".into()),
                    doi: Some("https://doi.org/10.1/abc".into()),
                },
                CurationInputRow {
                    name: "B".into(),
                    smiles: "CCN".into(),
                    taxon: None,
                    doi: None,
                },
                CurationInputRow {
                    name: "B-duplicate-2".into(),
                    smiles: "CCN".into(),
                    taxon: None,
                    doi: None,
                },
            ],
        );

        assert_eq!(
            outcome,
            AppendOutcome {
                added: 1,
                skipped: 2
            }
        );
        assert_eq!(queue.len(), 2);
        assert_eq!(queue[1].name, "B");
    }
}
