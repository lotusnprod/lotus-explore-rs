// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

#[cfg(not(target_arch = "wasm32"))]
pub(super) use crate::features::curation::domain::NATPROD_API_BASE;
pub(super) use crate::features::curation::domain::{
    CURATION_SPARQL_PREFIXES, CurationError, CurationInputRow, CurationResultRow, CurationStatus,
    DependencyResolution, MassResolution, WD_CHEMICAL_COMPOUND_QID, WD_OCCURS_IN_TAXON_PROP,
    WD_STEREOISOMER_GROUP_QID, WD_TAXON_QID, WD_TYPE_CHEMICAL_ENTITY_QID, WikidataCompound,
};
use crate::i18n::{
    Locale, curation_note_dependencies_pending, curation_note_existing_complete,
    curation_note_existing_updates, curation_note_new_compound, curation_pending_reference,
    curation_pending_taxon,
};
use lotus::queries::transform_query_for_wdqs;
use lotus::transport::{QLEVER_WIKIDATA, ResponseFormat, WDQS_SCHOLARLY, WDQS_WIKIDATA};
use serde::Deserialize;
use serde_json::Value;

mod chemical;
mod enrichment;
mod helpers;
mod http_client;
mod occurrence_cache;
mod reference_metadata;
pub mod wikidata;

use chemical::{convert_smiles, has_undefined_stereo, resolve_exact_mass};
use helpers::{
    QS_REF_INFERRED_FROM_SMILES, binding_value, escape_qs_string, escape_sparql_string,
    extract_qid_from_uri, has_isomeric_smiles, has_stereo_marks, normalize_doi,
    qs_canonical_smiles_statement, qs_inchi_statement, qs_inchikey_statement,
    qs_isomeric_smiles_statement, qs_statement_with_refs,
};
#[cfg(not(target_arch = "wasm32"))]
use http_client::{BatchConvertResponse, natprod_client};
#[cfg(target_arch = "wasm32")]
use http_client::{js_value_to_json, rdkit_bridge_call};
use reference_metadata::fetch_reference_quickstatements;
use wikidata::normalize_taxon_lookup;

pub mod inputs;
pub mod pipeline;
pub mod quickstatements;

#[cfg(test)]
pub(crate) use chemical::extract_exact_mass_from_json;
pub use enrichment::curate_single_row;
#[cfg(test)]
pub use helpers::{extract_formula_from_inchi, normalize_formula_for_wikidata, qs_mass_statement};

/// Single toggle to force WDQS fallback for testing.
/// Set this to `true` to force all queries to use WDQS (bypass QLever entirely).
/// Set this to `false` to use QLever with WDQS fallback on 502.
const FORCE_WDQS_FALLBACK: bool = false;

/// Execute a SPARQL query against Wikidata, preferring WDQS when
/// `FORCE_WDQS_FALLBACK` is enabled, otherwise falls back to WDQS on 502.
///
/// - Reference lookups (queries containing `SELECT ?ref WHERE {` and `wdt:P356`)
///   use the WDQS scholarly subgraph endpoint directly.
/// - All other queries are transformed via `transform_query_for_wdqs` and sent
///   to the regular WDQS endpoint.
pub async fn execute_sparql_with_wdqs_fallback(
    query: &str,
    format: ResponseFormat,
) -> Result<String, lotus::transport::FetchError> {
    if FORCE_WDQS_FALLBACK {
        log::warn!("event=curation_sparql phase=forced_wdqs_fallback");
        // For simple reference lookups, use scholarly endpoint directly
        if query.contains("SELECT ?ref WHERE {") && query.contains("wdt:P356") {
            return lotus::transport::execute_sparql_with_format(query, WDQS_SCHOLARLY, format)
                .await;
        }
        // For complex queries, apply transformation and use regular WDQS
        let wdqs_query = transform_query_for_wdqs(query);
        return lotus::transport::execute_sparql_with_format(&wdqs_query, WDQS_WIKIDATA, format)
            .await;
    }

    let result = lotus::transport::execute_sparql_with_format(query, QLEVER_WIKIDATA, format).await;

    match result {
        Ok(response) => Ok(response),
        Err(lotus::transport::FetchError::Http(502, _)) => {
            log::warn!("event=curation_sparql phase=fallback reason=qlever_502");
            // For simple reference lookups, use scholarly endpoint directly
            if query.contains("SELECT ?ref WHERE {") && query.contains("wdt:P356") {
                lotus::transport::execute_sparql_with_format(query, WDQS_SCHOLARLY, format).await
            } else {
                // For complex queries, apply transformation and use regular WDQS
                let wdqs_query = transform_query_for_wdqs(query);
                lotus::transport::execute_sparql_with_format(&wdqs_query, WDQS_WIKIDATA, format)
                    .await
            }
        }
        Err(e) => Err(e),
    }
}
