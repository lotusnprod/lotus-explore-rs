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
    // Check if this is a simple reference lookup query (just SELECT ?ref)
    let is_simple_ref_query = query.contains("SELECT ?ref WHERE {")
        && query.contains("wdt:P356")
        && !query.contains("SERVICE")
        && !query.contains("OPTIONAL");

    // Try ?ref pattern first, then ?r pattern
    if is_simple_ref_query {
        // For simple reference queries, wrap SELECT in SERVICE while keeping PREFIXES outside
        let query_without_prefix_placeholder = query.replace("{CURATION_SPARQL_PREFIXES}\n", "");
        let query_body = query_without_prefix_placeholder.replace(" LIMIT 1", "");
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
