// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Structure similarity and substructure search queries via the IDSM/Sachem service.
//!
//! `Sachem` is isolated in a subquery so `QLever` can pre-filter compounds before
//! expensive reference / property enrichment.

use crate::models::SmilesSearchType;
use crate::queries::compound::compound_select_clause;
use crate::queries::consts::{
    COMPOUND_IDENTIFIERS, PREFIXES_WITH_STRUCTURE, PROPERTIES_OPTIONAL, REFERENCE_METADATA_OPTIONAL,
};
use crate::queries::structure::escape_structure_literal;

/// SPARQL body for a Sachem search with no taxon filter: the reference/taxon
/// association is wrapped in `OPTIONAL` so compounds lacking occurrence data are
/// still returned.
fn sachem_body_no_taxon(sachem_clause: &str) -> String {
    format!(
        r"
  {sachem_clause}
  {COMPOUND_IDENTIFIERS}

  OPTIONAL {{
    ?c p:P703 ?statement .
    ?statement ps:P703 ?t ;
               prov:wasDerivedFrom ?ref .
    ?ref pr:P248 ?r .
    ?t wdt:P225 ?taxon_name .
    {REFERENCE_METADATA_OPTIONAL}
  }}

  {PROPERTIES_OPTIONAL}
"
    )
}

/// Structure similarity/substructure search query via IDSM/Sachem service.
///
/// # Arguments
///
/// - `smiles` — query SMILES string
/// - `search_type` — [`Similarity`](crate::models::SmilesSearchType::Similarity) or
///   [`Substructure`](crate::models::SmilesSearchType::Substructure)
/// - `threshold` — Tanimoto similarity cutoff (0.0–1.0) for similarity search
/// - `taxon_qid` — when `Some`, applies a `P171*` ancestry filter inside the
///   `Sachem` pass to ensure `QLever` only enriches matching rows
#[must_use]
pub fn query_sachem(
    smiles: &str,
    search_type: SmilesSearchType,
    threshold: f64,
    taxon_qid: Option<&str>,
) -> String {
    let structure_literal = escape_structure_literal(smiles);
    let is_multiline_literal =
        structure_literal.starts_with("'''") || structure_literal.starts_with(r#"\"\"\""#);

    let sachem_clause = match search_type {
        SmilesSearchType::Similarity => format!(
            r#"SERVICE idsm:wikidata {{
    ?c sachem:similarCompoundSearch [
      sachem:query {structure_literal};
      sachem:cutoff "{threshold}"^^xsd:double
    ].
  }}"#
        ),
        SmilesSearchType::Substructure if is_multiline_literal => format!(
            r#"SERVICE idsm:wikidata {{
    [ sachem:compound ?c; sachem:score ?_sachem_score ]
      sachem:scoredSubstructureSearch [
        sachem:query {structure_literal};
        sachem:searchMode sachem:substructureSearch;
        sachem:chargeMode sachem:defaultChargeAsAny;
        sachem:isotopeMode sachem:ignoreIsotopes;
        sachem:aromaticityMode sachem:aromaticityDetectIfMissing;
        sachem:stereoMode sachem:ignoreStereo;
        sachem:tautomerMode sachem:ignoreTautomers;
        sachem:radicalMode sachem:ignoreSpinMultiplicity;
        sachem:topn "-1"^^xsd:integer;
        sachem:internalMatchingLimit "1000000"^^xsd:integer
      ].
  }}"#
        ),
        SmilesSearchType::Substructure => format!(
            r"SERVICE idsm:wikidata {{
    ?c sachem:substructureSearch [
      sachem:query {structure_literal}
    ].
  }}"
        ),
    };

    let sachem_subquery = format!(
        r"{{
    SELECT DISTINCT ?c
    WHERE {{
      {sachem_clause}
    }}
  }}"
    );

    let body = taxon_qid.map_or_else(
        || sachem_body_no_taxon(&sachem_clause),
        |qid| {
            format!(
                r"
  {sachem_subquery}

  {COMPOUND_IDENTIFIERS}

  ?c p:P703 ?statement .
  ?statement ps:P703 ?t ;
             prov:wasDerivedFrom ?ref .
  ?ref pr:P248 ?r .
  ?t wdt:P225 ?taxon_name .
  ?t (wdt:P171*) wd:{qid} .

  {REFERENCE_METADATA_OPTIONAL}
  {PROPERTIES_OPTIONAL}
"
            )
        },
    );

    let compound_select = compound_select_clause();
    format!(
        r"{PREFIXES_WITH_STRUCTURE}
{compound_select}
WHERE {{
{body}
}}"
    )
}
