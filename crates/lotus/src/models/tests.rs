// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Unit tests for LOTUS domain models.

#![allow(missing_docs)]

use super::entry::CompoundEntry;
use super::search::SearchCriteria;
use super::sort::{SortColumn, SortDir, SortState};
use super::stats::{DatasetStats, ElementState, SmilesSearchType};
use std::collections::BTreeMap;
use std::sync::Arc;

fn make_entry(compound: &str, taxon: &str, reference: &str) -> CompoundEntry {
    CompoundEntry {
        compound_qid: Arc::from(compound),
        name: Arc::from(""),
        taxon_qid: Arc::from(taxon),
        taxon_name: Arc::from(""),
        reference_qid: Arc::from(reference),
        ..CompoundEntry::default()
    }
}

#[test]
fn dataset_stats_from_entries_counts_unique_triples() {
    let entries = vec![
        make_entry("Q1", "Q10", "Q100"),
        make_entry("Q1", "Q10", "Q100"), // duplicate triple
        make_entry("Q1", "Q11", "Q101"), // same compound, different taxon+ref
        make_entry("Q2", "Q10", "Q100"),
    ];
    let stats = DatasetStats::from_entries(&entries);
    assert_eq!(stats.n_entries, 4);
    assert_eq!(stats.n_entries_unique, 3); // 3 distinct (compound, taxon, ref) triples
    assert_eq!(stats.n_compounds, 2);
    assert_eq!(stats.n_taxa, 2);
    assert_eq!(stats.n_references, 2);
}

#[test]
fn dataset_stats_from_entries_empty_slice() {
    let stats = DatasetStats::from_entries(&[]);
    assert_eq!(stats.n_entries, 0);
    assert_eq!(stats.n_entries_unique, 0);
    assert_eq!(stats.n_compounds, 0);
}

#[test]
fn dataset_stats_from_entries_all_identical() {
    let entries = vec![make_entry("Q1", "Q1", "Q1"); 5];
    let stats = DatasetStats::from_entries(&entries);
    assert_eq!(stats.n_entries, 5);
    assert_eq!(stats.n_entries_unique, 1);
}

#[test]
fn shareable_query_params_omit_default_formula_values_when_only_toggle_is_enabled() {
    let criteria = SearchCriteria {
        taxon: "Fungi".into(),
        formula_enabled: true,
        ..SearchCriteria::default()
    };

    let params: BTreeMap<String, String> = criteria.shareable_query_params().into_iter().collect();

    assert_eq!(params.get("taxon").map(String::as_str), Some("Fungi"));
    assert_eq!(
        params.get("formula_filter").map(String::as_str),
        Some("true")
    );
    for key in [
        "formula_exact",
        "c_min",
        "c_max",
        "h_min",
        "h_max",
        "n_min",
        "n_max",
        "o_min",
        "o_max",
        "p_min",
        "p_max",
        "s_min",
        "s_max",
        "f_state",
        "cl_state",
        "br_state",
        "i_state",
    ] {
        assert!(
            !params.contains_key(key),
            "unexpected default formula param: {key}"
        );
    }
}

#[test]
fn shareable_query_params_keep_only_non_default_formula_overrides() {
    let criteria = SearchCriteria {
        taxon: "Fungi".into(),
        formula_enabled: true,
        c_min: 1,
        c_max: 10,
        o_max: 32,
        cl_state: ElementState::Required,
        br_state: ElementState::Excluded,
        ..SearchCriteria::default()
    };

    let params: BTreeMap<String, String> = criteria.shareable_query_params().into_iter().collect();

    assert_eq!(
        params.get("formula_filter").map(String::as_str),
        Some("true")
    );
    assert_eq!(params.get("c_min").map(String::as_str), Some("1"));
    assert_eq!(params.get("c_max").map(String::as_str), Some("10"));
    assert_eq!(params.get("o_max").map(String::as_str), Some("32"));
    assert_eq!(params.get("cl_state").map(String::as_str), Some("required"));
    assert_eq!(params.get("br_state").map(String::as_str), Some("excluded"));
    assert!(!params.contains_key("o_min"));
    assert!(!params.contains_key("f_state"));
    assert!(!params.contains_key("i_state"));
}

#[test]
fn shareable_query_params_use_single_structure_param_namespace() {
    let criteria = SearchCriteria {
        taxon: "Gentiana lutea".into(),
        smiles: "CCCC".into(),
        smiles_search_type: SmilesSearchType::Substructure,
        ..SearchCriteria::default()
    };

    let params: BTreeMap<String, String> = criteria.shareable_query_params().into_iter().collect();

    assert_eq!(params.get("structure").map(String::as_str), Some("CCCC"));
    assert_eq!(
        params.get("structure_search_type").map(String::as_str),
        Some("substructure")
    );
    assert!(!params.contains_key("smiles"));
    assert!(!params.contains_key("smiles_search_type"));
}

#[test]
fn sort_state_defaults_to_name_asc() {
    let sort = SortState::default();
    assert_eq!(sort.col, SortColumn::Name);
    assert_eq!(sort.dir, SortDir::Asc);
}
