// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! `CONSTRUCT` query generation for Turtle/RDF export.
//!
//! When exporting to RDF, the `SELECT` query is rewritten into a `CONSTRUCT`
//! that emits the full compound–taxon–reference provenance graph as triples.

use crate::queries::compound::compound_formula_expr;
/// Transform a SELECT query into a CONSTRUCT query for RDF export.
///
/// # Output Format
///
/// Turtle-serializable RDF triples representing the full compound-taxon-reference
/// relationship graph:
/// - Compound identifiers (`InChIKey`, `SMILES`, mass, formula, label)
/// - `P703` (found in taxon) statement with provenance (`prov:wasDerivedFrom`)
/// - Reference metadata (title, `DOI`, publication date)
///
/// **Pattern:**
/// Maps the `SELECT` variables to RDF triples using `Wikidata` vocabulary:
/// compound properties (P235, P233, etc.), taxon info, references, and metadata.
#[must_use]
pub fn query_construct_from_select(select_query: &str) -> String {
    let Some(select_pos) = select_query.find("SELECT") else {
        return select_query.to_string();
    };
    let Some(where_pos) = select_query[select_pos..].find("WHERE") else {
        return select_query.to_string();
    };
    let where_abs = select_pos + where_pos;
    let prefixes = &select_query[..select_pos];
    let where_block = select_query[where_abs..].trim();
    let normalized_where_block = construct_where_with_formula_bind(where_block);

    format!(
        r"{prefixes}
CONSTRUCT {{
  ?c wdt:P235 ?compound_inchikey .
  ?c wdt:P233 ?compound_smiles_conn .
  ?c wdt:P2017 ?compound_smiles_iso .
  ?c wdt:P2067 ?compound_mass .
  ?c wdt:P274 ?compound_formula .
  ?c rdfs:label ?compoundLabel .
  ?c p:P703 ?statement .
  ?statement ps:P703 ?t ;
             prov:wasDerivedFrom ?ref .
  ?ref pr:P248 ?r .
  ?t wdt:P225 ?taxon_name .
  ?r wdt:P1476 ?ref_title .
  ?r wdt:P356 ?ref_doi .
  ?r wdt:P577 ?ref_date .
}}
{normalized_where_block}"
    )
}

/// Inject a `BIND(… AS ?compound_formula)` clause into the WHERE block so
/// that the `CONSTRUCT` template can reference a normalized formula variable.
///
/// This is needed because the original `SELECT` projects `?compound_formula_raw`
/// but the `CONSTRUCT` template binds `?c wdt:P274 ?compound_formula` — the
/// formula must be derived from the raw column via subscript-digit normalization.
fn construct_where_with_formula_bind(where_block: &str) -> String {
    let Some(open_brace) = where_block.find('{') else {
        return where_block.to_string();
    };
    let Some(close_brace) = where_block.rfind('}') else {
        return where_block.to_string();
    };
    if close_brace <= open_brace {
        return where_block.to_string();
    }

    let inner = &where_block[(open_brace + 1)..close_brace];
    let formula_bind = format!(
        "  BIND({} AS ?compound_formula)",
        compound_formula_expr("?compound_formula_raw")
    );

    let mut out = String::with_capacity(where_block.len() + formula_bind.len() + 16);
    out.push_str("WHERE {");
    out.push_str(inner);
    out.push('\n');
    out.push_str(&formula_bind);
    out.push_str("\n}");
    out
}
