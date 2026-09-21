// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Integration tests that exercise query builders across submodule boundaries.

use crate::models::{ElementState, SearchCriteria, SmilesSearchType};
use crate::queries::consts::SUBSCRIPT_DIGIT_MAPPINGS;
use crate::queries::formula::{normalize_digits_expr, normalize_formula_digits};
use crate::queries::{
    StructureKind, classify_structure, query_all_compounds, query_compounds_by_taxon,
    query_construct_from_select, query_counts_from_base, query_sachem, query_with_limit,
    query_with_server_filters,
};

#[test]
fn server_filter_query_includes_formula_and_halogen_clauses() {
    let mut crit = SearchCriteria {
        taxon: "*".into(),
        ..SearchCriteria::default()
    };
    crit.formula_enabled = true;
    crit.c_min = 1;
    crit.c_max = 10;
    crit.f_state = ElementState::Required;

    let q = query_with_server_filters(&query_all_compounds(), &crit);
    assert!(q.contains("?_formula_tokens"));
    assert!(q.contains("?_count_c >= 1 && ?_count_c <= 10"));
    assert!(q.contains("?_count_f > 0"));
}

#[test]
fn server_filter_inserts_required_mass_when_mass_filtering() {
    let crit = SearchCriteria {
        mass_min: 100.0,
        mass_max: 500.0,
        ..SearchCriteria::default()
    };

    let q = query_with_server_filters(&query_all_compounds(), &crit);
    assert!(q.contains("?c wdt:P2067 ?compound_mass"));
    assert!(q.contains("FILTER(?compound_mass >= 100"));
    assert!(q.contains("?compound_mass <= 500"));
}

#[test]
fn server_filter_inserts_required_date_when_year_filtering() {
    let crit = SearchCriteria {
        year_min: 2000,
        year_max: 2024,
        ..SearchCriteria::default()
    };

    let q = query_with_server_filters(&query_all_compounds(), &crit);
    assert!(q.contains("?r wdt:P577 ?ref_date"));
    assert!(q.contains("FILTER(YEAR(?ref_date) >= 2000"));
    assert!(q.contains("YEAR(?ref_date) <= 2024)"));
}

#[test]
fn construct_query_switches_select_to_construct() {
    let q = query_construct_from_select(&query_compounds_by_taxon("Q2382443"));
    assert!(q.contains("CONSTRUCT"));
    assert!(q.contains("?c p:P703 ?statement"));
    assert!(!q.contains("SELECT\n  (xsd:integer"));
}

#[test]
fn sachem_query_uses_combined_prefixes() {
    let q = query_sachem("c1ccccc1", SmilesSearchType::Substructure, 0.8, None);
    // Should have all standard prefixes
    assert!(q.contains("PREFIX xsd:"));
    assert!(q.contains("PREFIX wd:"));
    assert!(q.contains("PREFIX wdt:"));
    // Should have structure-specific prefixes
    assert!(q.contains("PREFIX sachem:"));
    assert!(q.contains("PREFIX idsm:"));
    // Should NOT duplicate prefixes
    assert_eq!(q.matches("PREFIX xsd:").count(), 1);
    assert_eq!(q.matches("PREFIX sachem:").count(), 1);
}

#[test]
fn sachem_taxon_query_applies_ancestry_filter() {
    let q = query_sachem(
        "c1ccccc1",
        SmilesSearchType::Substructure,
        0.8,
        Some("Q158572"),
    );
    assert!(q.contains("?t (wdt:P171*) wd:Q158572"));
    // Should bind taxon_name and reference metadata
    assert!(q.contains("?t wdt:P225 ?taxon_name"));
    assert!(q.contains("?ref pr:P248 ?r"));
}

#[test]
fn sachem_no_taxon_query_makes_taxa_optional() {
    let q = query_sachem("c1ccccc1", SmilesSearchType::Substructure, 0.8, None);
    // Taxa block should be OPTIONAL when no taxon specified
    assert!(q.contains("OPTIONAL {"));
    assert!(q.contains("?c p:P703 ?statement"));
    // Should not have ancestry filter
    assert!(!q.contains("?t (wdt:P171*)"));
}

#[test]
fn count_query_uses_distinct_entry_triples_not_raw_rows() {
    let q = query_counts_from_base(&query_sachem(
        "CCO",
        SmilesSearchType::Substructure,
        0.8,
        Some("Q158572"),
    ));
    assert!(q.contains("COUNT(DISTINCT CONCAT("));
    assert!(q.contains("COUNT(DISTINCT CONCAT("));
    assert!(q.contains("AS ?n_entries_unique"));
    assert!(q.contains("STR(?compound)"));
    assert!(q.contains("COALESCE(STR(?taxon), \"\")"));
    assert!(q.contains("COALESCE(STR(?ref_qid), \"\")"));
}

/// The count query must NOT carry the display-only `OPTIONAL` blocks
/// (`REFERENCE_METADATA_OPTIONAL` / `PROPERTIES_OPTIONAL`) — they only supply
/// titles / DOIs / dates / ISO-SMILES / mass / formula / labels, none of which
/// feed the `COUNT(DISTINCT …)` metrics.  Carrying them forces `QLever` to
/// materialize the `rdfs:label` scans just to derive a count, which is what
/// made the WASM count "SUPER SLOW" / RAM-heavy on mobile.
#[test]
fn count_query_strips_display_optionals_but_keeps_core_triples() {
    // Compound browser (three-level SELECT, REQUIRED p:P703 association).
    let compound_count = query_counts_from_base(&query_compounds_by_taxon("Q158572"));
    assert!(
        !compound_count.contains("OPTIONAL {"),
        "count query must strip OPTIONAL blocks: {compound_count}"
    );
    // ...while keeping the counted compound-taxon-reference triples.
    assert!(compound_count.contains("?c p:P703 ?statement"));

    // Structure search without a taxon (p:P703 lives inside an OPTIONAL).
    let sachem_count = query_counts_from_base(&query_sachem(
        "CCO",
        SmilesSearchType::Substructure,
        1.0,
        None,
    ));
    assert!(
        sachem_count.contains("OPTIONAL {"),
        "sachem no-taxon count must KEEP the p:P703 OPTIONAL (only display metadata stripped)"
    );
    assert!(!sachem_count.contains("rdfs:label"));
    assert!(sachem_count.contains("?c p:P703 ?statement"));
}

#[test]
fn subscript_digit_normalizers_stay_in_sync() {
    assert_eq!(normalize_formula_digits("C₆H₁₂O₆"), "C6H12O6");

    let expr = normalize_digits_expr("?_formula_nospace");
    for (from, to) in SUBSCRIPT_DIGIT_MAPPINGS {
        assert!(expr.contains(&format!("\"{from}\"")));
        assert!(expr.contains(&format!("\"{to}\"")));
    }
}

#[test]
fn compound_queries_use_distinct_outer_select() {
    let q1 = query_all_compounds();
    let q2 = query_compounds_by_taxon("Q2382443");
    let q3 = query_sachem("c1ccccc1", SmilesSearchType::Substructure, 0.8, None);

    for q in [q1, q2, q3] {
        assert!(
            q.contains("SELECT DISTINCT"),
            "missing DISTINCT on outer select: {q}"
        );
    }
}

#[test]
fn compound_queries_keep_ref_uri_for_rdf_construct_compat() {
    let q = query_all_compounds();
    assert!(q.contains("\n  ?ref\n"));
    assert!(q.contains("?ref_qid"));
}

#[test]
fn compound_queries_project_raw_formula_and_normalize_at_display_layer() {
    let q = query_compounds_by_taxon("Q2382443");
    assert!(q.contains("?compound_formula_raw"));
    assert!(q.contains("AS ?compound_formula"));
}

#[test]
fn server_filters_bind_formula_from_raw_column() {
    let crit = SearchCriteria {
        formula_enabled: true,
        formula_exact: "C6H12O6".into(),
        ..SearchCriteria::default()
    };
    let q = query_with_server_filters(&query_all_compounds(), &crit);
    assert!(q.contains("BOUND(?compound_formula_raw)"));
    assert!(q.contains("STR(?compound_formula_raw)"));
    assert!(q.contains("?_formula_norm"));
}

#[test]
fn construct_query_rebinds_formula_from_raw_column() {
    let q = query_construct_from_select(&query_compounds_by_taxon("Q2382443"));
    assert!(q.contains("?compound_formula_raw"));
    assert!(q.contains("AS ?compound_formula"));
    assert!(q.contains("?c wdt:P274 ?compound_formula ."));
    assert_eq!(q.matches("PREFIX xsd:").count(), 1);
    assert_eq!(q.matches("PREFIX wdt:").count(), 1);
}

#[test]
fn construct_query_normalizes_formula_subscript_digits() {
    let q = query_construct_from_select(&query_compounds_by_taxon("Q2382443"));
    assert!(q.contains("BIND("));
    assert!(q.contains("STR(?compound_formula_raw)"));
    // Regression guard: keep subscript-digit normalization in RDF export.
    assert!(q.contains("\"₆\""));
    assert!(q.contains("\"6\""));
}

#[test]
fn sachem_query_projects_formula_from_raw_column() {
    let q = query_sachem("c1ccccc1", SmilesSearchType::Substructure, 0.8, None);
    // The formula column must be derived from ?compound_formula_raw, not a bare ?compound_formula
    assert!(q.contains("?compound_formula_raw"));
    assert!(q.contains("AS ?compound_formula"));
}

#[test]
fn prefixes_once_per_query_not_duplicated() {
    let q1 = query_all_compounds();
    assert_eq!(q1.matches("PREFIX xsd:").count(), 1);
    assert_eq!(q1.matches("PREFIX rdfs:").count(), 1);
    assert_eq!(q1.matches("PREFIX wd:").count(), 1);

    let q2 = query_compounds_by_taxon("Q2382443");
    assert_eq!(q2.matches("PREFIX xsd:").count(), 1);
    assert_eq!(q2.matches("PREFIX wdt:").count(), 1);

    let q3 = query_sachem("c1ccccc1", SmilesSearchType::Substructure, 0.8, None);
    assert_eq!(q3.matches("PREFIX sachem:").count(), 1);
    assert_eq!(q3.matches("PREFIX idsm:").count(), 1);
}

#[test]
fn taxon_search_query_format_is_valid() {
    let q = crate::queries::query_taxon_search("Test Taxon");
    // Should contain exactly one PREFIX
    assert_eq!(q.matches("PREFIX").count(), 1);
    // Should have wdt: prefix
    assert!(q.contains("PREFIX wdt:"));
    // Should have SELECT
    assert!(q.contains("SELECT"));
    // Should have VALUES
    assert!(q.contains("VALUES"));
    // Should reference the taxon name
    assert!(q.contains("Test Taxon"));
    // Should start with PREFIX (no leading whitespace)
    assert!(
        q.starts_with("PREFIX"),
        "query should start with PREFIX immediately, got: {}",
        &q[..q.len().min(50)]
    );
}

#[test]
fn limit_query_appends_limit_clause() {
    let base = "SELECT ?s WHERE { ?s ?p ?o }";
    let q = query_with_limit(base, 100);
    assert!(q.ends_with("LIMIT 100"));
}

#[test]
fn classify_structure_detects_formats() {
    assert_eq!(classify_structure(""), StructureKind::Empty);
    assert_eq!(classify_structure("   "), StructureKind::Empty);
    assert_eq!(classify_structure("c1ccccc1"), StructureKind::Smiles);
    assert_eq!(
        classify_structure("CHEMBL123     2.5V2000\n  12 34 0 0 0 0 0 0 0 0 0 0\nM  END"),
        StructureKind::MolfileV2000
    );
}
