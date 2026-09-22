// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! SPARQL query builders for compound / taxon lookup.
//!
//! These functions build the "core" compound queries shared by both the
//! simple compound browser and the Sachem structure search.  They all follow
//! a consistent three-level SELECT / subquery pattern that lets `QLever` plan
//! joins efficiently.

use crate::queries::consts::{
    COMPOUND_CORE_VARS, COMPOUND_ENRICHED_VARS, COMPOUND_IDENTIFIERS, PREFIXES,
    PROPERTIES_OPTIONAL, REFERENCE_METADATA_OPTIONAL, TAXON_REFERENCE_ASSOCIATION,
};
use crate::queries::formula::normalize_digits_expr;

/// Builds the `SELECT DISTINCT` clause with subscript-digit-normalized formula.
pub(super) fn compound_select_clause() -> String {
    format!(
        r#"
SELECT DISTINCT
  (xsd:integer(STRAFTER(STR(?c), "Q")) AS ?compound)
  ?compoundLabel
  ?compound_inchikey
  ?compound_smiles_conn
  ?compound_smiles_iso
  ?compound_mass
  ({formula} AS ?compound_formula)
  (xsd:integer(STRAFTER(STR(?t), "Q")) AS ?taxon)
  ?taxon_name
  (xsd:integer(STRAFTER(STR(?r), "Q")) AS ?ref_qid)
  ?ref
  ?ref_title
  ?ref_doi
  ?ref_date
  ?statement
"#,
        formula = normalize_digits_expr("?compound_formula_raw")
    )
}

/// Convenience wrapper used by [`query_construct_from_select`](crate::queries::rdf::query_construct_from_select)
/// to produce the formula BIND expression.
pub(super) fn compound_formula_expr(raw_var: &str) -> String {
    normalize_digits_expr(raw_var)
}

/// Search for taxa by scientific name.
///
/// Uses Wikidata's P225 (taxon name, scientific nomenclature).
/// Returns all matching Wikidata entities where the scientific name equals the query.
///
/// # Use Cases
///
/// - Autocomplete/suggestions for taxon filtering
/// - Validation that a taxon exists before querying compounds
#[must_use]
pub fn query_taxon_search(name: &str) -> String {
    let e = name.replace('\\', r"\\").replace('"', r#"\""#);
    format!(
        r#"PREFIX wdt: <http://www.wikidata.org/prop/direct/>
SELECT
  ?taxon
  ?taxon_name
WHERE {{
  VALUES ?taxon_name {{ "{e}" }}
  ?taxon wdt:P225 ?taxon_name .
}}"#
    )
}

/// Build the compound query with an optional taxon ancestry filter.
/// When `taxon_qid` is `Some`, adds the `P171*` transitive-closure filter.
fn query_compounds_inner(taxon_qid: Option<&str>) -> String {
    let ancestry = taxon_qid
        .map(|qid| format!("\n          ?t (wdt:P171*) wd:{qid}."))
        .unwrap_or_default();
    let compound_select = compound_select_clause();
    format!(
        r"{PREFIXES}
{compound_select}
WHERE {{
  {{
    SELECT
      {COMPOUND_ENRICHED_VARS}
    WHERE {{
      {{
        SELECT {COMPOUND_CORE_VARS}
        WHERE {{
          {COMPOUND_IDENTIFIERS}
          {TAXON_REFERENCE_ASSOCIATION}
          {ancestry}
        }}
      }}
      {REFERENCE_METADATA_OPTIONAL}
      {PROPERTIES_OPTIONAL}
    }}
  }}
}}"
    )
}

/// Query compounds found in a specific taxon and all descendants.
///
/// Uses a three-level SELECT/subquery pattern:
/// 1. Innermost SELECT: core compound-taxon-reference triples + ancestry filter
/// 2. Middle SELECT: OPTIONAL enrichment (reference metadata, properties)
/// 3. Outer SELECT: `xsd:integer` projections on the small, enriched result set
///
/// The ancestry filter (`P171*` transitive closure) is applied *inside* the
/// innermost subquery so `QLever` only enriches matching rows.
#[must_use]
pub fn query_compounds_by_taxon(taxon_qid: &str) -> String {
    query_compounds_inner(Some(taxon_qid))
}

/// Query all compounds from all organisms/taxa in LOTUS.
///
/// Same three-level scaffolding as [`query_compounds_by_taxon`] but without the
/// ancestry filter. Large result sets should be paginated via LIMIT.
#[must_use]
pub fn query_all_compounds() -> String {
    query_compounds_inner(None)
}
