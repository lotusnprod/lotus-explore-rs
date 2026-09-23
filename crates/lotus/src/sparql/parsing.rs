// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! CSV to domain parsers: turn raw `QLever` CSV payloads into strongly-typed
//! LOTUS results (`CompoundEntry` rows, `DatasetStats`, `TaxonMatch` vectors).

use super::types::{
    CompoundColumns, CompoundInterners, entry_key_fingerprint, fill_qid, fnv1a_one, parse_entity_id,
};
use crate::models::{CompoundEntry, DatasetStats, TaxonMatch};
use crate::transport::{FetchError, col_idx, field};
use std::collections::HashSet;
use std::io::Read;

/// Parse CSV bytes into deduplicated `CompoundEntry` display rows.
///
/// # Errors
/// Returns [`FetchError::Parse`] when CSV decoding fails.
pub fn parse_compounds_csv_display_bytes(
    csv_bytes: &[u8],
    max_rows: usize,
) -> Result<Vec<CompoundEntry>, FetchError> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(csv_bytes);

    let headers = rdr
        .byte_headers()
        .map_err(|e| FetchError::Parse(e.to_string()))?
        .clone();
    let cols = CompoundColumns::detect(&headers);

    let initial_cap = max_rows.min(1024);
    let mut entries: Vec<CompoundEntry> = Vec::with_capacity(initial_cap);
    let mut seen: HashSet<u64> = HashSet::with_capacity(initial_cap.saturating_mul(2));
    let mut interners = CompoundInterners::new(initial_cap);
    let mut compound_qid = String::new();
    let mut taxon_qid = String::new();
    let mut reference_qid = String::new();

    let mut rec = csv::ByteRecord::new();
    while entries.len() < max_rows
        && rdr
            .read_byte_record(&mut rec)
            .map_err(|e| FetchError::Parse(e.to_string()))?
    {
        compound_qid.clear();
        if let Some(i) = cols.compound
            && let Some(b) = rec.get(i)
        {
            fill_qid(&mut compound_qid, b);
        }
        if compound_qid.is_empty() {
            continue;
        }
        taxon_qid.clear();
        if let Some(i) = cols.taxon
            && let Some(b) = rec.get(i)
        {
            fill_qid(&mut taxon_qid, b);
        }
        reference_qid.clear();
        if let Some(i) = cols.ref_qid
            && let Some(b) = rec.get(i)
        {
            fill_qid(&mut reference_qid, b);
        }

        let key = entry_key_fingerprint(
            compound_qid.as_bytes(),
            taxon_qid.as_bytes(),
            reference_qid.as_bytes(),
        );
        if !seen.insert(key) {
            continue;
        }

        entries.push(interners.build_entry(&cols, &rec, &compound_qid, &taxon_qid, &reference_qid));
    }

    Ok(entries)
}

/// Parse a single-row counts CSV into [`DatasetStats`].
///
/// # Errors
/// Returns [`FetchError::Parse`] when CSV decoding fails or no row is present.
pub fn parse_counts_csv_bytes(csv_bytes: &[u8]) -> Result<DatasetStats, FetchError> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(csv_bytes);

    let headers = rdr
        .headers()
        .map_err(|e| FetchError::Parse(e.to_string()))?
        .clone();
    let c_entries = col_idx(&headers, "n_entries");
    let c_entries_unique = col_idx(&headers, "n_entries_unique");
    let c_compounds = col_idx(&headers, "n_compounds");
    let c_taxa = col_idx(&headers, "n_taxa");
    let c_refs = col_idx(&headers, "n_references");

    let mut records = rdr.records();
    let rec = match records.next() {
        Some(Ok(r)) => r,
        Some(Err(e)) => return Err(FetchError::Parse(e.to_string())),
        None => return Err(FetchError::Parse("Missing count row".to_string())),
    };

    let parse_num =
        |idx: Option<usize>| -> usize { field(&rec, idx).parse::<usize>().unwrap_or(0) };

    let n_entries = parse_num(c_entries);
    let n_entries_unique = parse_num(c_entries_unique);

    Ok(DatasetStats {
        n_entries,
        n_entries_unique: if n_entries_unique == 0 {
            n_entries
        } else {
            n_entries_unique
        },
        n_compounds: parse_num(c_compounds),
        n_taxa: parse_num(c_taxa),
        n_references: parse_num(c_refs),
    })
}

/// Parse capped compound rows and aggregate stats from CSV bytes.
///
/// # Errors
/// Returns [`FetchError::Parse`] when CSV decoding fails.
// Test-only helper: exercised by `super::tests`, dead in non-test builds.
#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn parse_compounds_csv_capped_bytes(
    csv_bytes: &[u8],
    max_rows: usize,
) -> Result<(Vec<CompoundEntry>, DatasetStats, bool), FetchError> {
    parse_compounds_csv_capped_reader(csv_bytes, max_rows)
}

/// Parse capped compound rows and aggregate stats from any CSV reader.
///
/// # Errors
/// Returns [`FetchError::Parse`] when CSV decoding fails.
pub fn parse_compounds_csv_capped_reader<R: Read>(
    reader: R,
    max_rows: usize,
) -> Result<(Vec<CompoundEntry>, DatasetStats, bool), FetchError> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(reader);

    let headers = rdr
        .byte_headers()
        .map_err(|e| FetchError::Parse(e.to_string()))?
        .clone();
    let cols = CompoundColumns::detect(&headers);

    let initial_cap = max_rows.min(2048);
    let mut entries: Vec<CompoundEntry> = Vec::with_capacity(initial_cap);
    let mut seen: HashSet<u64> = HashSet::with_capacity(initial_cap.saturating_mul(2));
    let mut compound_fps: HashSet<u64> = HashSet::with_capacity(initial_cap);
    let mut taxon_fps: HashSet<u64> = HashSet::with_capacity(initial_cap);
    let mut ref_fps: HashSet<u64> = HashSet::with_capacity(initial_cap);
    let mut total_raw = 0usize;
    let mut total_distinct = 0usize;
    let mut interners = CompoundInterners::new(initial_cap);

    let mut compound_qid = String::new();
    let mut taxon_qid = String::new();
    let mut reference_qid = String::new();

    let mut rec = csv::ByteRecord::new();
    while rdr
        .read_byte_record(&mut rec)
        .map_err(|e| FetchError::Parse(e.to_string()))?
    {
        compound_qid.clear();
        if let Some(i) = cols.compound
            && let Some(b) = rec.get(i)
        {
            fill_qid(&mut compound_qid, b);
        }
        if compound_qid.is_empty() {
            continue;
        }
        total_raw += 1;
        taxon_qid.clear();
        if let Some(i) = cols.taxon
            && let Some(b) = rec.get(i)
        {
            fill_qid(&mut taxon_qid, b);
        }
        reference_qid.clear();
        if let Some(i) = cols.ref_qid
            && let Some(b) = rec.get(i)
        {
            fill_qid(&mut reference_qid, b);
        }

        let key = entry_key_fingerprint(
            compound_qid.as_bytes(),
            taxon_qid.as_bytes(),
            reference_qid.as_bytes(),
        );
        if !seen.insert(key) {
            continue;
        }

        total_distinct += 1;
        compound_fps.insert(fnv1a_one(compound_qid.as_bytes()));
        if !taxon_qid.is_empty() {
            taxon_fps.insert(fnv1a_one(taxon_qid.as_bytes()));
        }
        if !reference_qid.is_empty() {
            ref_fps.insert(fnv1a_one(reference_qid.as_bytes()));
        }

        if entries.len() < max_rows {
            entries.push(interners.build_entry(
                &cols,
                &rec,
                &compound_qid,
                &taxon_qid,
                &reference_qid,
            ));
        }
    }

    let stats = DatasetStats {
        n_compounds: compound_fps.len(),
        n_taxa: taxon_fps.len(),
        n_references: ref_fps.len(),
        n_entries: total_raw,
        n_entries_unique: total_distinct,
    };
    let was_capped = total_distinct > entries.len();
    Ok((entries, stats, was_capped))
}

/// Parse taxon search CSV rows into `(qid, name)` matches.
///
/// # Errors
/// Returns [`FetchError::Parse`] when CSV decoding fails.
pub fn parse_taxon_csv_bytes(csv_bytes: &[u8]) -> Result<Vec<TaxonMatch>, FetchError> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(csv_bytes);

    let headers = rdr
        .headers()
        .map_err(|e| FetchError::Parse(e.to_string()))?
        .clone();

    let c_taxon = col_idx(&headers, "taxon");
    let c_name = col_idx(&headers, "taxon_name");

    let mut matches = Vec::new();
    for result in rdr.records() {
        let rec = result.map_err(|e| FetchError::Parse(e.to_string()))?;
        let qid = parse_entity_id(field(&rec, c_taxon));
        let name = field(&rec, c_name).to_string();
        if !qid.is_empty() && !name.is_empty() {
            matches.push(TaxonMatch { qid, name });
        }
    }
    Ok(matches)
}
