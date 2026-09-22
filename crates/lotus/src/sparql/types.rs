// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! LOTUS domain types, string interners, FNV-1a hashing, and the shared value
//! extraction/normalization helpers (`parse_entity_id`, `normalize_statement_value`,
//! `normalize_doi_value`, `fill_qid`, `byte_field_str`).
//!
//! These cluster around entry construction: `CompoundInterners::build_entry`
//! normalizes/dedupes field values while assembling a `CompoundEntry`. They are
//! `pub` rather than `pub(crate)` because `types` is a private submodule, so
//! `pub` means "visible to the `sparql` subtree only".

use crate::models::CompoundEntry;
use crate::transport::{extract_qid, parse_year};
use std::collections::HashMap;
use std::num::Wrapping;
use std::sync::Arc;

const WIKIDATA_STATEMENT_PREFIX: &str = "http://www.wikidata.org/entity/statement/";
pub struct CompoundColumns {
    pub compound: Option<usize>,
    pub label: Option<usize>,
    pub inchikey: Option<usize>,
    pub smiles_iso: Option<usize>,
    pub smiles_con: Option<usize>,
    pub mass: Option<usize>,
    pub formula: Option<usize>,
    pub taxon: Option<usize>,
    pub taxon_name: Option<usize>,
    pub ref_qid: Option<usize>,
    pub ref_title: Option<usize>,
    pub ref_doi: Option<usize>,
    pub ref_date: Option<usize>,
    pub statement: Option<usize>,
}

impl CompoundColumns {
    /// Scan a CSV header row and return the column index for each known field.
    ///
    /// Column presence is optional — absent columns map to `None`.
    pub fn detect(headers: &csv::ByteRecord) -> Self {
        let find =
            |name: &str| -> Option<usize> { headers.iter().position(|h| h == name.as_bytes()) };
        Self {
            compound: find("compound"),
            label: find("compoundLabel"),
            inchikey: find("compound_inchikey"),
            smiles_iso: find("compound_smiles_iso"),
            smiles_con: find("compound_smiles_conn"),
            mass: find("compound_mass"),
            formula: find("compound_formula"),
            taxon: find("taxon"),
            taxon_name: find("taxon_name"),
            ref_qid: find("ref_qid"),
            ref_title: find("ref_title"),
            ref_doi: find("ref_doi"),
            ref_date: find("ref_date"),
            statement: find("statement"),
        }
    }
}

/// String interners for all `CompoundEntry` fields.
///
/// Interning avoids duplicate `Arc<str>` allocations for repeated values
/// (e.g. the same taxon name appearing in many rows).  Each field has its
/// own interner to maximize hit rates — taxon names are far less unique than
/// compound QIDs, so they get a smaller initial capacity.
pub struct CompoundInterners {
    qid: StrInterner,
    label: StrInterner,
    taxon_name: StrInterner,
    ref_title: StrInterner,
    doi: StrInterner,
    inchikey: StrInterner,
    smiles: StrInterner,
    formula: StrInterner,
    statement: StrInterner,
}

impl CompoundInterners {
    pub fn new(cap: usize) -> Self {
        Self {
            qid: StrInterner::with_capacity(cap),
            label: StrInterner::with_capacity(cap),
            taxon_name: StrInterner::with_capacity(64),
            ref_title: StrInterner::with_capacity(128),
            doi: StrInterner::with_capacity(cap / 2),
            inchikey: StrInterner::with_capacity(cap),
            smiles: StrInterner::with_capacity(cap * 2),
            formula: StrInterner::with_capacity(cap),
            statement: StrInterner::with_capacity(cap),
        }
    }

    /// Interpolate all fields from a CSV record into a [`CompoundEntry`],
    /// using the column map and string interners to avoid redundant allocation.
    pub fn build_entry(
        &mut self,
        cols: &CompoundColumns,
        rec: &csv::ByteRecord,
        compound_qid: &str,
        taxon_qid: &str,
        reference_qid: &str,
    ) -> CompoundEntry {
        let label = byte_field_str(rec, cols.label);
        let inchikey = byte_field_str(rec, cols.inchikey);
        let smiles_iso = byte_field_str(rec, cols.smiles_iso);
        let smiles_con = byte_field_str(rec, cols.smiles_con);
        let mass_str = byte_field_str(rec, cols.mass);
        let formula = byte_field_str(rec, cols.formula);
        let taxon_name = byte_field_str(rec, cols.taxon_name);
        let ref_title = byte_field_str(rec, cols.ref_title);
        let ref_doi = byte_field_str(rec, cols.ref_doi);
        let ref_date = byte_field_str(rec, cols.ref_date);
        let statement = byte_field_str(rec, cols.statement);

        CompoundEntry {
            compound_qid: self.qid.intern_or_empty(compound_qid),
            name: self.label.intern_or_empty(label),
            inchikey: self.inchikey.intern_optional(inchikey),
            smiles: self.smiles.intern_optional(if smiles_iso.is_empty() {
                smiles_con
            } else {
                smiles_iso
            }),
            mass: mass_str.parse::<f64>().ok(),
            formula: self.formula.intern_optional(formula),
            taxon_qid: self.qid.intern_or_empty(taxon_qid),
            taxon_name: self.taxon_name.intern_or_empty(taxon_name),
            reference_qid: self.qid.intern_or_empty(reference_qid),
            ref_title: self.ref_title.intern_optional(ref_title),
            ref_doi: normalize_doi_value(ref_doi).and_then(|d| self.doi.intern_optional(d)),
            pub_year: parse_year(ref_date).and_then(|y| i16::try_from(y).ok()),
            statement: normalize_statement_value(statement)
                .and_then(|s| self.statement.intern_optional(s)),
        }
    }
}

/// A simple FNV-1a string interner — maps `&str` → `Arc<str>`, reusing the
/// same allocation for identical values.
#[derive(Default)]
pub struct StrInterner {
    map: HashMap<Box<str>, Arc<str>>,
}

impl StrInterner {
    /// Construct an `StrInterner` pre-sized for `cap` unique strings.
    ///
    /// Sub-field capacities are tuned: taxon names get 64 slots (many rows share
    /// a few taxa), reference titles get 128 (shared across references), smiles
    /// get `cap * 2` (ISO + connection variants).
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
        }
    }

    pub fn intern_or_empty(&mut self, value: &str) -> Arc<str> {
        let v = value.trim();
        if v.is_empty() {
            return Arc::<str>::from("");
        }
        if let Some(existing) = self.map.get(v) {
            return existing.clone();
        }
        let arc = Arc::<str>::from(v);
        self.map.insert(v.to_owned().into_boxed_str(), arc.clone());
        arc
    }

    pub fn intern_optional(&mut self, value: &str) -> Option<Arc<str>> {
        let v = value.trim();
        if v.is_empty() {
            None
        } else {
            Some(self.intern_or_empty(v))
        }
    }
}

/// FNV-1a hash extension: XOR each byte with the accumulator, then multiply.
#[inline]
fn fnv1a_extend(mut h: Wrapping<u64>, bytes: &[u8]) -> Wrapping<u64> {
    const FNV_PRIME: Wrapping<u64> = Wrapping(1_099_511_628_211);
    for b in bytes {
        h ^= Wrapping(u64::from(*b));
        h *= FNV_PRIME;
    }
    h
}

/// FNV-1a hash of a single byte slice (uses the standard offset basis).
#[inline]
pub fn fnv1a_one(bytes: &[u8]) -> u64 {
    fnv1a_extend(Wrapping(14_695_981_039_346_656_037_u64), bytes).0
}

/// Compute a FNV-1a hash of a (compound, taxon, reference) QID triple.
///
/// Used as a deduplication key in [`parse_compounds_csv_display_bytes`] and
/// [`parse_compounds_csv_capped_reader`].
pub fn entry_key_fingerprint(compound_qid: &[u8], taxon_qid: &[u8], reference_qid: &[u8]) -> u64 {
    let mut h = Wrapping(14_695_981_039_346_656_037_u64);
    h = fnv1a_extend(h, compound_qid);
    h = fnv1a_extend(h, &[0x1f]);
    h = fnv1a_extend(h, taxon_qid);
    h = fnv1a_extend(h, &[0x1f]);
    h = fnv1a_extend(h, reference_qid);
    h.0
}

/// Parse a Wikidata entity ID from a CSV cell value.
///
/// Handles three formats:
/// 1. Full URI: `http://www.wikidata.org/entity/Q123` → `Q123`
/// 2. Typed literal: `"456"^^<...integer>` → `Q456`
/// 3. Bare numeric: `456` → `Q456`
///
/// Returns an empty string for non-QID values (e.g. `P123` properties).
pub fn parse_entity_id(value: &str) -> String {
    let qid = extract_qid(value);
    if !qid.is_empty() {
        return qid;
    }

    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let lexical = trimmed
        .split("^^")
        .next()
        .unwrap_or(trimmed)
        .trim()
        .trim_matches('"');

    if let Some(rest) = lexical.strip_prefix('Q')
        && !rest.is_empty()
        && rest.bytes().all(|b| b.is_ascii_digit())
    {
        return lexical.to_string();
    }

    if !lexical.is_empty() && lexical.bytes().all(|b| b.is_ascii_digit()) {
        return format!("Q{lexical}");
    }

    String::new()
}

/// Strip the Wikidata statement prefix from an optional value.
///
/// Returns `None` for empty/whitespace input, otherwise the value with
/// `http://www.wikidata.org/entity/statement/` stripped if present.
#[inline]
pub fn normalize_statement_value(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(
        trimmed
            .strip_prefix(WIKIDATA_STATEMENT_PREFIX)
            .unwrap_or(trimmed),
    )
}

/// Normalise a DOI from a CSV cell: strip the `doi.org/` prefix if present.
///
/// Returns `None` for empty/whitespace input.  This is the borrowed-string
/// equivalent of [`crate::transport::clean_doi`], used internally by the
/// interning layer to avoid allocation before calling [`StrInterner`].
#[inline]
pub fn normalize_doi_value(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let normalized = trimmed.split("doi.org/").last().unwrap_or(trimmed).trim();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

/// Read a trimmed UTF-8 string from a CSV byte record at an optional column index.
///
/// Returns `""` (not `None`) when the column is absent or non-UTF-8, so
/// callers can treat all values uniformly via `intern_optional`.
#[inline]
fn byte_field_str(rec: &csv::ByteRecord, idx: Option<usize>) -> &str {
    idx.and_then(|i| rec.get(i))
        .map_or("", |bytes| std::str::from_utf8(bytes).unwrap_or("").trim())
}

/// Write a QID into `out`, parsing from a Wikidata URI, typed literal, or bare number.
///
/// Clears nothing — callers are expected to clear `out` before calling.
/// On invalid/empty input, `out` is left unchanged (not emptied).
pub fn fill_qid(out: &mut String, bytes: &[u8]) {
    let s = match std::str::from_utf8(bytes) {
        Ok(s) => s.trim(),
        Err(_) => return,
    };
    if s.is_empty() {
        return;
    }

    if let Some(idx) = s.rfind("wikidata.org/entity/") {
        let rest = &s[idx + "wikidata.org/entity/".len()..];
        if rest.len() >= 2
            && rest.as_bytes()[0] == b'Q'
            && rest.bytes().skip(1).all(|b| b.is_ascii_digit())
        {
            out.push_str(rest);
            return;
        }
    }

    let lexical = s.split("^^").next().unwrap_or(s).trim().trim_matches('"');

    if lexical.is_empty() {
        return;
    }

    if lexical.as_bytes().first() == Some(&b'Q')
        && lexical.len() >= 2
        && lexical[1..].bytes().all(|b| b.is_ascii_digit())
    {
        out.push_str(lexical);
        return;
    }

    if lexical.bytes().all(|b| b.is_ascii_digit()) {
        out.push('Q');
        out.push_str(lexical);
    }
}
