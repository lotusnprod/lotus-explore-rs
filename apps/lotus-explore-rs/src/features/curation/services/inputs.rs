#![allow(clippy::manual_let_else)]
#![allow(clippy::redundant_closure_for_method_calls)]
// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

use super::helpers::find_ascii_ci;
use crate::features::curation::domain::{CurationError, CurationInputRow};

pub fn example_rows() -> Vec<CurationInputRow> {
    vec![
        CurationInputRow {
            name: "Voatriafricanine A".into(),
            smiles: "OC12N3C4=C(O)C([C@H](C[C@H]/5[C@@H]6C(OC)=O)C(N([H])C7=C8C=CC=C7)=C8CC6N(C)CC5=C\\C)=CC=C4[C@@]19CCN%10C9[C@@]%11(C[C@H]2C[C@H]%12[C@H]%13[C@@]%14(CC(C(OC)=O)=C%15NC%16=CC=CC=C%16[C@@]%15%17CCN([C@@H]%123)C%14%17)CCO%13)CCO[C@H]%11CC%10".into(),
            taxon: Some("Voacanga africana".into()),
            doi: Some("10.1021/acs.jnatprod.1c00812".into()),
        },
        CurationInputRow {
            name: "Voatriafricanine B (taxon and DOI wrong but new)".into(),
            smiles: "OC12N3C4=C(O)C([C@H](C[C@H]/5[C@@H]6C(OC)=O)C(N([H])C7=C8C=CC=C7)=C8CC6N(C)CC5=C\\C)=CC=C4[C@@]19CCN%10C9[C@@]%11(C[C@H]2C[C@H]%12[C@H]%13[C@@]%14(CC(C(OC)=O)=C%15NC%16=C(OC)C=CC=C%16[C@@]%15%17CCN([C@@H]%123)C%14%17)CCO%13)CCO[C@H]%11CC%10".into(),
            taxon: Some("Gentiana lutea".into()),
            doi: Some("10.1068/P080363".into()),
        },
        CurationInputRow {
            name: "[HYPOTHETICAL - non-real test case]".into(),
            smiles: "CCN(CC)C(=O)N1C=NC2=C1N=CN2C(F)(F)F".into(),
            taxon: Some("Ficticia imaginaria".into()),
            doi: Some("10.59350/sk00y-3gh44".into()),
        },
    ]
}

pub fn parse_tsv_rows(tsv: &str) -> Result<Vec<CurationInputRow>, CurationError> {
    let mut lines = tsv.lines().map(str::trim).filter(|line| !line.is_empty());

    let header = match lines.next() {
        Some(h) => h,
        None => return Ok(Vec::new()),
    };

    let columns = header.split('\t').map(normalize_header).collect::<Vec<_>>();
    let name_idx = columns
        .iter()
        .position(|c| c == "name")
        .ok_or_else(|| CurationError::InvalidInput("TSV is missing a 'name' column".into()))?;
    let smiles_idx = columns
        .iter()
        .position(|c| c == "smiles")
        .ok_or_else(|| CurationError::InvalidInput("TSV is missing a 'smiles' column".into()))?;
    let taxon_idx = columns
        .iter()
        .position(|c| matches!(c.as_str(), "taxon" | "organism"));
    let doi_idx = columns.iter().position(|c| c == "doi");
    let max_needed_idx = [Some(name_idx), Some(smiles_idx), taxon_idx, doi_idx]
        .into_iter()
        .flatten()
        .max()
        .unwrap_or(0);

    let mut out = Vec::new();
    for line in lines {
        let mut name: Option<&str> = None;
        let mut smiles: Option<&str> = None;
        let mut taxon_raw: Option<&str> = None;
        let mut doi_raw: Option<&str> = None;

        for (idx, field) in line.split('\t').enumerate() {
            if idx > max_needed_idx {
                break;
            }
            let field = field.trim();
            if idx == name_idx {
                name = Some(field);
            }
            if idx == smiles_idx {
                smiles = Some(field);
            }
            if taxon_idx == Some(idx) {
                taxon_raw = Some(field);
            }
            if doi_idx == Some(idx) {
                doi_raw = Some(field);
            }
        }

        let Some(name) = name else {
            continue;
        };
        let Some(smiles) = smiles else {
            continue;
        };
        if name.is_empty() || smiles.is_empty() {
            continue;
        }
        let taxon = taxon_raw.and_then(|v| non_empty(v).map(ToOwned::to_owned));
        let doi = doi_raw.and_then(normalize_doi);
        out.push(CurationInputRow {
            name: name.into(),
            smiles: smiles.into(),
            taxon,
            doi,
        });
    }
    Ok(out)
}

pub fn row_uniqueness_key(row: &CurationInputRow) -> String {
    let smiles = row.smiles.trim();
    let taxon = row
        .taxon
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| v.to_ascii_lowercase())
        .unwrap_or_default();
    let doi = row
        .doi
        .as_deref()
        .and_then(normalize_doi)
        .unwrap_or_default();
    format!("{smiles}\t{taxon}\t{doi}")
}

fn normalize_header(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace(' ', "_")
}

fn normalize_doi(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let canonical =
        find_ascii_ci(trimmed, b"doi.org/").map_or(trimmed, |idx| &trimmed[(idx + 8)..]);
    if canonical.is_empty() {
        return None;
    }
    Some(canonical.to_ascii_uppercase())
}

fn non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tsv_supports_expected_headers() {
        let tsv = "name\tsmiles\torganism\tdoi\nA\tCCO\tTaxon\thttps://doi.org/10.1/x\n";
        let rows = parse_tsv_rows(tsv).expect("tsv parse");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "A");
        assert_eq!(rows[0].smiles, "CCO");
        assert_eq!(rows[0].taxon.as_deref(), Some("Taxon"));
        assert_eq!(rows[0].doi.as_deref(), Some("10.1/X"));
    }

    #[test]
    fn row_key_normalizes_taxon_and_doi() {
        let row = CurationInputRow {
            name: "compound A".into(),
            smiles: " CCO ".into(),
            taxon: Some("  Voacanga africana ".into()),
            doi: Some("https://doi.org/10.1000/abc".into()),
        };
        assert_eq!(
            row_uniqueness_key(&row),
            "CCO\tvoacanga africana\t10.1000/ABC"
        );
    }
}

#[test]
fn test_user_example_multiline() {
    let tsv = "name\tsmiles\ttaxon\tdoi\n2'-deoxyguanosine\tC1[C@@H]([C@H](O[C@H]1N2C=NC3=C2N=C(NC3=O)N)CO)O\tIsaria cicadae\t10.1177/1934578X1501001233\nthymidine\tCC1=CN(C(=O)NC1=O)[C@H]2C[C@@H]([C@H](O2)CO)O\tIsaria cicadae\t10.1177/1934578X1501001233\nadenosine\tC1=NC(=C2C(=N1)N(C=N2)[C@H]3[C@@H]([C@@H]([C@H](O3)CO)O)O)N\tIsaria cicadae\t10.1177/1934578X1501001233\n2'-deoxyadenosine\tC1[C@@H]([C@H](O[C@H]1N2C=NC3=C(N=CN=C32)N)CO)O\tIsaria cicadae\t10.1177/1934578X1501001233\ntryptophan\tC1=CC=C2C(=C1)C(=CN2)C[C@@H](C(=O)O)N\tIsaria cicadae\t10.1177/1934578X1501001233\nphenylalanine\tC1=CC=C(C=C1)C[C@@H](C(=O)O)N\tIsaria cicadae\t10.1177/1934578X1501001233\ntyrosine\tC1=CC(=CC=C1C[C@@H](C(=O)O)N)O\tIsaria cicadae\t10.1177/1934578X1501001233\nN-acetylnoradrenaline\tCC(=O)NCC(C1=CC(=C(C=C1)O)O)O\tIsaria cicadae\t10.1177/1934578X1501001233";

    let rows = parse_tsv_rows(tsv).expect("tsv parse");
    assert_eq!(rows.len(), 8, "Expected 8 rows, got {}", rows.len());

    for row in rows {
        assert!(!row.name.is_empty(), "Name should not be empty");
        assert!(!row.smiles.is_empty(), "SMILES should not be empty");
        assert_eq!(
            row.taxon.as_deref(),
            Some("Isaria cicadae"),
            "Taxon should be Isaria cicadae"
        );
        assert_eq!(
            row.doi.as_deref(),
            Some("10.1177/1934578X1501001233"),
            "DOI should match"
        );
    }
}
