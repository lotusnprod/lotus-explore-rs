// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

use crate::features::curation::domain::{
    CurationError, CurationInputRow, CurationResultRow, QuickStatementsBundle,
    build_quickstatements_bundle,
};
use crate::i18n::Locale;
use futures::stream::{self, StreamExt};
use std::future::Future;

// Curation drives Qlever with several POSTs per row (compound fetch, ASK, ...).
// QLever's *anonymous* quota rejects bursts (the 15:00 storm was 5 POSTs in
// ~150ms → two 429s), so rows are curated strictly ONE AT A TIME: each row's
// Qlever POSTs are serialized, which is the only way to stay under the quota
// without an API key. Non-Qlever work per row (RDKit SMILES handling) still
// pipelines naturally within the single in-flight row.
const CURATION_CONCURRENCY: usize = 1;

pub async fn curate_rows<F, Fut>(
    locale: Locale,
    rows: Vec<CurationInputRow>,
    curate_single_row: F,
    row_uniqueness_key: fn(&CurationInputRow) -> String,
) -> Result<(Vec<CurationResultRow>, QuickStatementsBundle), CurationError>
where
    F: Fn(Locale, CurationInputRow) -> Fut + Clone,
    Fut: Future<Output = CurationResultRow>,
{
    let mut seen_keys = std::collections::HashSet::with_capacity(rows.len());
    let mut unique_rows = Vec::with_capacity(rows.len());
    for row in rows {
        if seen_keys.insert(row_uniqueness_key(&row)) {
            unique_rows.push(row);
        }
    }

    let mut indexed_results = stream::iter(unique_rows.into_iter().enumerate())
        .map(|(idx, row)| {
            let curate_single_row = curate_single_row.clone();
            async move { (idx, curate_single_row(locale, row).await) }
        })
        .buffer_unordered(CURATION_CONCURRENCY)
        .collect::<Vec<_>>()
        .await;
    indexed_results.sort_by_key(|(idx, _)| *idx);

    let results = indexed_results
        .into_iter()
        .map(|(_, row)| row)
        .collect::<Vec<_>>();
    let bundle = build_quickstatements_bundle(&results);
    Ok((results, bundle))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::curation::domain::{CurationInputRow, CurationStatus};
    use futures::executor::block_on;

    fn key(row: &CurationInputRow) -> String {
        row.smiles.to_ascii_lowercase()
    }

    async fn fake_curate(_locale: Locale, input: CurationInputRow) -> CurationResultRow {
        CurationResultRow {
            input,
            canonical_smiles: None,
            inchikey: None,
            inchi: None,
            formula: None,
            exact_mass: None,
            mass_warning: None,
            wikidata_qid: None,
            status: CurationStatus::NewCompound,
            note: String::new(),
            dependency_blocks: Vec::new(),
            quickstatements: vec!["CREATE".to_string()],
        }
    }

    #[test]
    fn curate_rows_deduplicates_and_preserves_input_order() {
        let rows = vec![
            CurationInputRow {
                name: "a".into(),
                smiles: "CCO".into(),
                taxon: None,
                doi: None,
            },
            CurationInputRow {
                name: "dup".into(),
                smiles: "cco".into(),
                taxon: None,
                doi: None,
            },
            CurationInputRow {
                name: "b".into(),
                smiles: "N".into(),
                taxon: None,
                doi: None,
            },
        ];

        let (result_rows, _bundle) =
            block_on(curate_rows(Locale::En, rows, fake_curate, key)).expect("pipeline result");
        let names = result_rows
            .into_iter()
            .map(|r| r.input.name)
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["a", "b"]);
    }
}
