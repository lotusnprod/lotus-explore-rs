// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project
#![allow(clippy::future_not_send)]

use super::ResultsPipelineOutcome;
use crate::export::SparqlEndpoint;
use crate::features::explore::request::SearchRequest;
use crate::features::explore::search_metrics::SearchMetrics;
use crate::features::explore::service::{
    build_query::{apply_server_filters, build_sparql_query},
    fetch_results::FetchResult,
    resolve_taxon::{self, TaxonResolution},
};
use crate::features::explore::types::{DomainError, QueryPhase};
use crate::repositories::{LotusRepository, is_wdqs_fallback_used};

pub(super) struct ResultsExecutionPlan {
    taxon_resolution: TaxonResolution,
    execution_query: String,
}

impl ResultsExecutionPlan {
    pub(super) fn execution_query(&self) -> &str {
        &self.execution_query
    }

    pub(super) fn into_download_only_outcome(self) -> ResultsPipelineOutcome {
        let endpoint = if is_wdqs_fallback_used() {
            SparqlEndpoint::Wdqs
        } else {
            SparqlEndpoint::Qlever
        };
        let warning = if is_wdqs_fallback_used() {
            Some(crate::features::explore::types::TaxonWarning::WdqsFallback)
        } else {
            self.taxon_resolution.warning
        };
        // Store the display query (WDQS-transformed if fallback occurred)
        let query = crate::repositories::get_wdqs_transformed_query()
            .unwrap_or_else(|| self.execution_query.clone());

        ResultsPipelineOutcome {
            rows: Vec::new(),
            qid: self.taxon_resolution.qid,
            warning,
            query,
            total_matches: None,
            total_stats: None,
            display_capped_rows: false,
            endpoint,
        }
    }

    pub(super) fn into_interactive_outcome(
        self,
        fetch_result: FetchResult,
    ) -> ResultsPipelineOutcome {
        let endpoint = if is_wdqs_fallback_used() {
            SparqlEndpoint::Wdqs
        } else {
            SparqlEndpoint::Qlever
        };
        let warning = if is_wdqs_fallback_used() {
            Some(crate::features::explore::types::TaxonWarning::WdqsFallback)
        } else {
            self.taxon_resolution.warning
        };
        // Store the display query (WDQS-transformed if fallback occurred)
        let query = crate::repositories::get_wdqs_transformed_query()
            .unwrap_or_else(|| self.execution_query.clone());

        ResultsPipelineOutcome {
            rows: fetch_result.rows,
            qid: self.taxon_resolution.qid,
            warning,
            query,
            total_matches: fetch_result.total_matches,
            total_stats: fetch_result.total_stats,
            display_capped_rows: fetch_result.display_capped_rows,
            endpoint,
        }
    }
}

pub(super) async fn build_execution_plan<R: LotusRepository>(
    request: &SearchRequest,
    normalized_smiles: &str,
    repo: &R,
    metrics: &mut SearchMetrics,
    on_phase: &impl Fn(QueryPhase),
) -> Result<ResultsExecutionPlan, DomainError> {
    let taxon = request.criteria().taxon.trim();
    if resolve_taxon::requires_remote_lookup(taxon) {
        on_phase(QueryPhase::ResolvingTaxon);
    }

    let taxon_resolution = resolve_taxon::resolve(taxon, repo, metrics).await?;
    let sparql_query = build_sparql_query(
        normalized_smiles,
        request.criteria(),
        taxon_resolution.qid.as_deref(),
    );
    let execution_query = apply_server_filters(&sparql_query, request.criteria());

    Ok(ResultsExecutionPlan {
        taxon_resolution,
        execution_query,
    })
}
