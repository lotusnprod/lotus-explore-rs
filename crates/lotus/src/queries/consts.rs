// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Shared SPARQL prefix blocks and Wikidata property fragments.
//!
//! These string constants are the structural scaffolding for all compound /
//! taxon / reference queries.  Keeping them in one place ensures every query
//! builder uses consistent prefixes and property identifiers.

#![allow(missing_docs)] // raw SPARQL strings, documented at the `queries` module level

/// Subscript digit mappings (₀ → 0, ₁ → 1, … ₉ → 9) for formula normalization.
pub(super) const SUBSCRIPT_DIGIT_MAPPINGS: [(char, char); 10] = [
    ('₀', '0'),
    ('₁', '1'),
    ('₂', '2'),
    ('₃', '3'),
    ('₄', '4'),
    ('₅', '5'),
    ('₆', '6'),
    ('₇', '7'),
    ('₈', '8'),
    ('₉', '9'),
];

/// Standard `Wikidata`/`QLever` SPARQL PREFIX declarations.
pub(super) const PREFIXES: &str = r"PREFIX xsd:    <http://www.w3.org/2001/XMLSchema#>
PREFIX rdfs:   <http://www.w3.org/2000/01/rdf-schema#>
PREFIX prov:   <http://www.w3.org/ns/prov#>
PREFIX wd:     <http://www.wikidata.org/entity/>
PREFIX wdt:    <http://www.wikidata.org/prop/direct/>
PREFIX p:      <http://www.wikidata.org/prop/>
PREFIX ps:     <http://www.wikidata.org/prop/statement/>
PREFIX pq:     <http://www.wikidata.org/prop/qualifier/>
PREFIX pr:     <http://www.wikidata.org/prop/reference/>
PREFIX wikibase: <http://wikiba.se/ontology#>
PREFIX schema: <http://schema.org/>
";

/// Extended PREFIXES for structure search queries (IDSM/Sachem service).
pub(super) const PREFIXES_WITH_STRUCTURE: &str = r"PREFIX xsd:    <http://www.w3.org/2001/XMLSchema#>
PREFIX rdfs:   <http://www.w3.org/2000/01/rdf-schema#>
PREFIX prov:   <http://www.w3.org/ns/prov#>
PREFIX wd:     <http://www.wikidata.org/entity/>
PREFIX wdt:    <http://www.wikidata.org/prop/direct/>
PREFIX p:      <http://www.wikidata.org/prop/>
PREFIX ps:     <http://www.wikidata.org/prop/statement/>
PREFIX pq:     <http://www.wikidata.org/prop/qualifier/>
PREFIX pr:     <http://www.wikidata.org/prop/reference/>
PREFIX wikibase: <http://wikiba.se/ontology#>
PREFIX schema: <http://schema.org/>
PREFIX sachem: <http://bioinfo.uochb.cas.cz/rdf/v1.0/sachem#>
PREFIX idsm:   <https://idsm.elixir-czech.cz/sparql/endpoint/>
";

/// Compound identifier retrieval via Wikidata direct properties.
pub(super) const COMPOUND_IDENTIFIERS: &str = r"
  ?c wdt:P235 ?compound_inchikey;
     wdt:P233 ?compound_smiles_conn.
";

/// Taxon-reference association via Wikidata statement structure.
pub(super) const TAXON_REFERENCE_ASSOCIATION: &str = r"
  ?c p:P703 ?statement.
  ?statement ps:P703 ?t;
             prov:wasDerivedFrom ?ref.
  ?ref pr:P248 ?r.
  ?t wdt:P225 ?taxon_name.
";

/// Reference metadata: title (P1476), DOI (P356), publication date (P577).
/// Uses ` ?r` variable — matched by [`TAXON_REFERENCE_ASSOCIATION`].
pub(super) const REFERENCE_METADATA_OPTIONAL: &str = r"
  OPTIONAL { ?r wdt:P1476 ?ref_title. }
  OPTIONAL { ?r wdt:P356 ?ref_doi. }
  OPTIONAL { ?r wdt:P577 ?ref_date. }
";

/// Reference metadata: title (P1476), DOI (P356), publication date (P577).
/// Uses ` ?ref` variable — matched by reference resolution queries.
pub(super) const REFERENCE_METADATA_OPTIONAL_REF: &str = r"
  OPTIONAL { ?ref wdt:P1476 ?ref_title. }
  OPTIONAL { ?ref wdt:P356 ?ref_doi. }
  OPTIONAL { ?ref wdt:P577 ?ref_date. }
";

/// Core variables projected from the innermost (Level-1) SELECT.
pub(super) const COMPOUND_CORE_VARS: &str =
    "?c ?compound_inchikey ?compound_smiles_conn ?t ?taxon_name ?r ?ref ?statement";

/// Full variable list projected by the middle (Level-2) SELECT.
pub(super) const COMPOUND_ENRICHED_VARS: &str = r"?c ?compound_inchikey ?compound_smiles_conn
      ?compound_smiles_iso ?compound_mass ?compound_formula_raw
      ?compoundLabel
      ?t ?taxon_name
      ?r ?ref
      ?ref_title ?ref_doi ?ref_date
      ?statement";

/// Compound properties with efficient subscript digit normalization.
pub(super) const PROPERTIES_OPTIONAL: &str = r#"
  OPTIONAL { ?c wdt:P2017 ?compound_smiles_iso. }
  OPTIONAL { ?c wdt:P2067 ?compound_mass. }
  OPTIONAL { ?c wdt:P274 ?compound_formula_raw. }
  OPTIONAL { ?c rdfs:label ?compoundLabelMul. FILTER(LANG(?compoundLabelMul) = "mul") }
  OPTIONAL { ?c rdfs:label ?compoundLabelEn. FILTER(LANG(?compoundLabelEn) = "en") }
  BIND(COALESCE(?compoundLabelMul, ?compoundLabelEn) AS ?compoundLabel)
"#;

/// Reference metadata service wrapper for WDQS scholarly subgraph.
///
/// Wraps the standard reference metadata OPTIONAL blocks with a SERVICE clause
/// that queries the scholarly subgraph endpoint for enhanced bibliographic data.
/// Used when executing queries directly against WDQS (fallback from `QLever` 502).
pub(super) const REFERENCE_METADATA_SERVICE: &str = r"
  SERVICE <https://query-scholarly.wikidata.org/sparql> {
    OPTIONAL { ?r wdt:P1476 ?ref_title. }
    OPTIONAL { ?r wdt:P356 ?ref_doi. }
    OPTIONAL { ?r wdt:P577 ?ref_date. }
  }
";

/// Reference metadata service wrapper for WDQS scholarly subgraph (using ?ref variable).
///
/// Same as `REFERENCE_METADATA_SERVICE` but uses ?ref instead of ?r as the variable name.
pub(super) const REFERENCE_METADATA_SERVICE_REF: &str = r"
  SERVICE <https://query-scholarly.wikidata.org/sparql> {
    OPTIONAL { ?ref wdt:P1476 ?ref_title. }
    OPTIONAL { ?ref wdt:P356 ?ref_doi. }
    OPTIONAL { ?ref wdt:P577 ?ref_date. }
  }
";

/// Transforms a query to use scholarly subgraph SERVICE for reference metadata
/// (replacing standalone OPTIONAL blocks with SERVICE-wrapped version).
///
/// This is used when falling back from `QLever` to WDQS, as the scholarly subgraph
/// provides enhanced access to bibliographic data.
#[must_use]
pub fn transform_query_for_wdqs(query: &str) -> String {
    // A simple reference lookup is a `?ref` query that also has no SERVICE or
    // OPTIONAL blocks — otherwise we'd wrap an already-SERVICE-wrapped query.
    let is_simple_ref_query = is_scholarly_reference_query(query)
        && !query.contains("SERVICE")
        && !query.contains("OPTIONAL");

    // Try ?ref pattern first, then ?r pattern
    if is_simple_ref_query {
        // For simple reference queries, wrap SELECT in SERVICE while keeping PREFIXES outside
        let query_body = strip_curation_prefixes(query).replace(" LIMIT 1", "");
        format!(
            "SERVICE <https://query-scholarly.wikidata.org/sparql> {{\n  {query_body}\n}}\nLIMIT 1"
        )
    } else if query.contains(REFERENCE_METADATA_OPTIONAL_REF) {
        // Replace ?ref variable with SERVICE
        query.replace(
            REFERENCE_METADATA_OPTIONAL_REF,
            REFERENCE_METADATA_SERVICE_REF,
        )
    } else if query.contains(REFERENCE_METADATA_OPTIONAL) {
        // Replace ?r variable with SERVICE
        query.replace(REFERENCE_METADATA_OPTIONAL, REFERENCE_METADATA_SERVICE)
    } else {
        query.to_string()
    }
}

/// True for curation reference-lookup queries (`SELECT ?ref WHERE { … wdt:P356`),
/// shared by the WDQS download/dispatch paths and `transform_query_for_wdqs`.
#[must_use]
pub fn is_scholarly_reference_query(query: &str) -> bool {
    query.contains("SELECT ?ref WHERE {") && query.contains("wdt:P356")
}

/// Remove the `{CURATION_SPARQL_PREFIXES}` placeholder (and its trailing newline)
/// that curation query templates embed before substitution.
#[must_use]
fn strip_curation_prefixes(query: &str) -> String {
    query.replace("{CURATION_SPARQL_PREFIXES}\n", "")
}

/// Choose the WDQS endpoint and the query to execute for a download/fallback path.
///
/// Reference-lookup queries ([`is_scholarly_reference_query`]) go straight to the
/// scholarly subgraph endpoint with the curation-prefix placeholder stripped; all
/// other queries are rewritten via [`transform_query_for_wdqs`] for the regular
/// WDQS endpoint. The returned query still needs
/// [`crate::export::ExportFormat::prepared_query`] before execution.
#[must_use]
pub fn wdqs_download_query(query: &str) -> (&'static str, String) {
    use crate::transport::{WDQS_SCHOLARLY, WDQS_WIKIDATA};
    if is_scholarly_reference_query(query) {
        (WDQS_SCHOLARLY, strip_curation_prefixes(query))
    } else {
        (WDQS_WIKIDATA, transform_query_for_wdqs(query))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::{WDQS_SCHOLARLY, WDQS_WIKIDATA};

    #[test]
    fn scholarly_predicate_detects_reference_queries() {
        assert!(is_scholarly_reference_query(
            "SELECT ?ref WHERE { ?r wdt:P356 \"x\" }"
        ));
        // Missing the P356 property → not a reference lookup.
        assert!(!is_scholarly_reference_query(
            "SELECT ?ref WHERE { ?r wdt:P1476 ?t }"
        ));
        // A different SELECT variable is not the reference lookup shape.
        assert!(!is_scholarly_reference_query(
            "SELECT ?compound WHERE { ?c wdt:P356 \"x\" }"
        ));
    }

    #[test]
    fn wdqs_download_query_routes_reference_lookups_to_scholarly() {
        let q = "{CURATION_SPARQL_PREFIXES}\nSELECT ?ref WHERE { ?r wdt:P356 \"10.1/x\" }";
        let (endpoint, prepared) = wdqs_download_query(q);
        assert_eq!(endpoint, WDQS_SCHOLARLY);
        // The curation-prefix placeholder is stripped for the scholarly endpoint.
        assert!(!prepared.contains("{CURATION_SPARQL_PREFIXES}"));
        assert!(prepared.contains("SELECT ?ref WHERE"));
        assert!(!prepared.contains("TRANSFORM"));
    }

    #[test]
    fn wdqs_download_query_transforms_ordinary_queries() {
        let q = "SELECT ?s WHERE { ?s ?p ?o }";
        let (endpoint, prepared) = wdqs_download_query(q);
        assert_eq!(endpoint, WDQS_WIKIDATA);
        // A plain query that `transform_query_for_wdqs` leaves untouched.
        assert_eq!(prepared, q);
    }

    #[test]
    fn transform_wraps_simple_ref_query_in_scholarly_service() {
        let q = "{CURATION_SPARQL_PREFIXES}\nSELECT ?ref WHERE { ?r wdt:P356 \"10.1/x\" } LIMIT 1";
        let out = transform_query_for_wdqs(q);
        assert!(out.starts_with("SERVICE <https://query-scholarly.wikidata.org/sparql>"));
        assert!(!out.contains("{CURATION_SPARQL_PREFIXES}"));
        assert!(out.contains("SELECT ?ref WHERE { ?r wdt:P356 \"10.1/x\" }"));
        assert!(out.ends_with("LIMIT 1"));
        // The trailing display LIMIT must be stripped from the inner body.
        assert!(!out.contains(" LIMIT 1"));
    }

    #[test]
    fn transform_replaces_ref_optional_with_service() {
        let q =
            format!("SELECT ?c WHERE {{ ?c wdt:P356 \"x\". {REFERENCE_METADATA_OPTIONAL_REF} }}");
        let out = transform_query_for_wdqs(&q);
        assert!(out.contains(REFERENCE_METADATA_SERVICE_REF));
        assert!(!out.contains(REFERENCE_METADATA_OPTIONAL_REF));
    }

    #[test]
    fn transform_replaces_r_optional_with_service() {
        let q = format!("SELECT ?c WHERE {{ ?c wdt:P356 \"x\". {REFERENCE_METADATA_OPTIONAL} }}");
        let out = transform_query_for_wdqs(&q);
        assert!(out.contains(REFERENCE_METADATA_SERVICE));
        assert!(!out.contains(REFERENCE_METADATA_OPTIONAL));
    }

    #[test]
    fn transform_passthrough_for_unrelated_query() {
        let q = "SELECT ?s WHERE { ?s ?p ?o }";
        assert_eq!(transform_query_for_wdqs(q), q);
    }
}
