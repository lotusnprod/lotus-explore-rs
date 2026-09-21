// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! SPARQL result parsing and LOTUS-domain HTTP wrappers.
//!
//! Re-exports thin wrappers around `crate::transport` that target the default
//! `QLever` Wikidata endpoint, plus the CSV-parsing helpers that turn raw
//! `QLever` CSV into strongly-typed LOTUS domain objects (`CompoundEntry`,
//! `DatasetStats`, `TaxonMatch`).
//!
//! Split by responsibility:
//! - `types`: domain types, string interners, FNV-1a hashing, and value
//!   extraction/normalization helpers.
//! - `parsing`: CSV to domain parsers.
//! - `execution`: thin wrappers that execute queries on the default endpoint.
//!
//! Public items are re-exported below so that `lotus::sparql::*` paths are
//! unchanged for downstream crates (`lotus-explore-rs`).

mod execution;
mod parsing;
mod types;

#[cfg(not(target_arch = "wasm32"))]
pub use execution::execute_sparql_tempfile;
pub use execution::{
    execute_query, execute_sparql_body, execute_sparql_bytes, execute_sparql_format,
    fetch_export_url_format,
};
pub use parsing::{
    parse_compounds_csv_capped_bytes, parse_compounds_csv_capped_reader,
    parse_compounds_csv_display_bytes, parse_counts_csv_bytes, parse_taxon_csv_bytes,
};

#[cfg(test)]
mod tests {
    use super::parsing::*;
    use super::types::*;
    use std::sync::Arc;

    #[test]
    fn parse_taxon_csv_handles_uri_and_numeric_qids() {
        let csv = b"taxon,taxon_name\nhttp://www.wikidata.org/entity/Q123,Alpha\n\"456\"^^<http://www.w3.org/2001/XMLSchema#integer>,Beta\n";
        let parsed = parse_taxon_csv_bytes(csv).expect("taxon parse");
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].qid, "Q123");
        assert_eq!(parsed[0].name, "Alpha");
        assert_eq!(parsed[1].qid, "Q456");
        assert_eq!(parsed[1].name, "Beta");
    }

    #[test]
    fn parse_counts_csv_prefers_unique_when_available() {
        let csv = b"n_entries,n_entries_unique,n_compounds,n_taxa,n_references\n10,7,3,2,4\n";
        let stats = parse_counts_csv_bytes(csv).expect("count parse");
        assert_eq!(stats.n_entries, 10);
        assert_eq!(stats.n_entries_unique, 7);
        assert_eq!(stats.n_compounds, 3);
        assert_eq!(stats.n_taxa, 2);
        assert_eq!(stats.n_references, 4);
    }

    #[test]
    fn parse_compounds_display_dedups_by_entry_triple() {
        let csv = b"compound,compoundLabel,compound_inchikey,compound_smiles_conn,compound_mass,compound_formula,taxon,taxon_name,ref_qid,ref_title,ref_doi,ref_date,statement\nQ1,cmpd,IK1,C,123.4,C1H2,Q10,TaxonA,Q100,TitleA,10.1/a,2022-01-01,http://www.wikidata.org/entity/statement/S1\nQ1,cmpd,IK1,C,123.4,C1H2,Q10,TaxonA,Q100,TitleA,10.1/a,2022-01-01,http://www.wikidata.org/entity/statement/S1\nQ2,cmpd2,IK2,CC,111.1,C2H4,Q11,TaxonB,Q101,TitleB,10.1/b,2021-01-01,http://www.wikidata.org/entity/statement/S2\n";
        let rows = parse_compounds_csv_display_bytes(csv, 50).expect("display parse");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].compound_qid.as_ref(), "Q1");
        assert_eq!(rows[0].taxon_qid.as_ref(), "Q10");
        assert_eq!(rows[0].statement.as_deref(), Some("S1"));
        assert_eq!(rows[1].compound_qid.as_ref(), "Q2");
        assert_eq!(rows[1].statement.as_deref(), Some("S2"));
    }

    #[test]
    fn parse_compounds_capped_reports_cap_and_stats() {
        let csv = b"compound,compoundLabel,compound_inchikey,compound_smiles_conn,compound_mass,compound_formula,taxon,taxon_name,ref_qid,ref_title,ref_doi,ref_date,statement\nQ1,cmpd,IK1,C,123.4,C1H2,Q10,TaxonA,Q100,TitleA,10.1/a,2022-01-01,http://www.wikidata.org/entity/statement/S1\nQ2,cmpd2,IK2,CC,111.1,C2H4,Q11,TaxonB,Q101,TitleB,10.1/b,2021-01-01,http://www.wikidata.org/entity/statement/S2\nQ3,cmpd3,IK3,CCC,99.1,C3H6,Q12,TaxonC,Q102,TitleC,10.1/c,2020-01-01,http://www.wikidata.org/entity/statement/S3\n";
        let (rows, stats, capped) = parse_compounds_csv_capped_bytes(csv, 2).expect("capped parse");
        assert_eq!(rows.len(), 2);
        assert!(capped);
        assert_eq!(stats.n_entries, 3);
        assert_eq!(stats.n_entries_unique, 3);
        assert_eq!(stats.n_compounds, 3);
    }

    #[test]
    fn parse_compounds_capped_reader_matches_bytes_path() {
        let csv = b"compound,compoundLabel,taxon,ref_qid\nQ1,cmpd,Q10,Q100\nQ2,cmpd2,Q11,Q101\nQ3,cmpd3,Q12,Q102\n";
        let (rows_bytes, stats_bytes, capped_bytes) =
            parse_compounds_csv_capped_bytes(csv, 2).expect("bytes parse");
        let (rows_reader, stats_reader, capped_reader) =
            parse_compounds_csv_capped_reader(std::io::Cursor::new(csv), 2).expect("reader parse");

        assert_eq!(rows_reader.len(), rows_bytes.len());
        assert_eq!(stats_reader, stats_bytes);
        assert_eq!(capped_reader, capped_bytes);
    }

    #[test]
    fn fill_qid_handles_various_formats() {
        let mut out = String::new();
        fill_qid(&mut out, b"http://www.wikidata.org/entity/Q12345");
        assert_eq!(out, "Q12345");

        out.clear();
        fill_qid(
            &mut out,
            b"\"456\"^^<http://www.w3.org/2001/XMLSchema#integer>",
        );
        assert_eq!(out, "Q456");

        out.clear();
        fill_qid(&mut out, b"Q789");
        assert_eq!(out, "Q789");

        out.clear();
        fill_qid(&mut out, b"");
        assert_eq!(out, "");
    }

    #[test]
    fn parse_entity_id_extracts_bare_or_uri_qids() {
        assert_eq!(
            parse_entity_id("http://www.wikidata.org/entity/Q123"),
            "Q123"
        );
        assert_eq!(parse_entity_id("Q456"), "Q456");
        assert_eq!(
            parse_entity_id("\"789\"^^<http://www.w3.org/2001/XMLSchema#integer>"),
            "Q789"
        );
        assert_eq!(parse_entity_id("P123"), "");
        assert_eq!(parse_entity_id(""), "");
    }

    #[test]
    fn interner_deduplicates_strings() {
        let mut interner = StrInterner::with_capacity(4);
        let a = interner.intern_or_empty("hello");
        let b = interner.intern_or_empty("hello");
        assert!(Arc::ptr_eq(&a, &b));
    }

    #[test]
    fn interner_returns_empty_for_blank_input() {
        let mut interner = StrInterner::with_capacity(4);
        let result = interner.intern_or_empty("   ");
        assert_eq!(result.as_ref(), "");
    }

    #[test]
    fn interner_optional_returns_none_for_empty() {
        let mut interner = StrInterner::with_capacity(4);
        assert_eq!(interner.intern_optional("   "), None);
        assert!(interner.intern_optional("hello").is_some());
    }

    #[test]
    fn normalize_statement_strips_prefix() {
        let with_prefix = "http://www.wikidata.org/entity/statement/S1";
        assert_eq!(normalize_statement_value(with_prefix), Some("S1"));
        assert_eq!(normalize_statement_value("S2"), Some("S2"));
        assert_eq!(normalize_statement_value(""), None);
    }

    #[test]
    fn normalize_doi_strips_prefix() {
        assert_eq!(
            normalize_doi_value("https://doi.org/10.1/a"),
            Some("10.1/a")
        );
        assert_eq!(normalize_doi_value("10.1/b"), Some("10.1/b"));
        assert_eq!(normalize_doi_value(""), None);
        assert_eq!(normalize_doi_value("   "), None);
    }

    #[test]
    fn entry_key_fingerprint_is_deterministic() {
        let key1 = entry_key_fingerprint(b"Q1", b"Q10", b"Q100");
        let key2 = entry_key_fingerprint(b"Q1", b"Q10", b"Q100");
        let key3 = entry_key_fingerprint(b"Q2", b"Q10", b"Q100");
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn parse_compounds_display_handles_empty_compound_id() {
        let csv = b"compound,compoundLabel,taxon,taxon_name,ref_qid\n,Q1,cmpd,,Q100\n";
        let rows = parse_compounds_csv_display_bytes(csv, 50).expect("display parse");
        // Rows with empty compound_qid should be skipped
        assert_eq!(rows.len(), 0);
    }

    #[test]
    fn parse_counts_csv_falls_back_to_entries_when_unique_is_zero() {
        let csv = b"n_entries,n_entries_unique,n_compounds,n_taxa,n_references\n10,0,3,2,4\n";
        let stats = parse_counts_csv_bytes(csv).expect("count parse");
        assert_eq!(stats.n_entries, 10);
        assert_eq!(stats.n_entries_unique, 10);
    }
}
