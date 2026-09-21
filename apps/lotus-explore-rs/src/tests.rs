// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

use crate::download::DownloadFormat;
use crate::features::curation::state::page_controller::rows_to_tsv;
use crate::features::explore::search_state::ExploreState;
use crate::features::explore::selectors::toolbar_snapshot_from_result;
use crate::ui::ContentPhase;

fn is_supported_download_format(fmt: &str) -> bool {
    DownloadFormat::parse(fmt).is_some()
}

#[test]
fn supported_download_formats_include_documented_values() {
    assert!(is_supported_download_format("csv"));
    assert!(is_supported_download_format("json"));
    assert!(is_supported_download_format("ndjson"));
    assert!(is_supported_download_format("rdf"));
    assert!(!is_supported_download_format("ttl"));
}

#[test]
fn supported_download_formats_allow_case_and_whitespace_variants() {
    assert!(is_supported_download_format(" CSV "));
    assert!(is_supported_download_format("Json"));
    assert!(is_supported_download_format("RDF"));
}

#[test]
fn integration_explore_snapshot_drives_loaded_phase_and_toolbar_data() {
    let mut explore = ExploreState::default();
    explore.lifecycle.searched_once = true;
    explore.result.sparql_query = Some("SELECT * WHERE { ?s ?p ?o }".into());
    explore.result.total_matches = Some(3);

    let snapshot = toolbar_snapshot_from_result(&explore.result);
    let phase = ContentPhase::from_lifecycle(
        explore.lifecycle.loading,
        explore.lifecycle.error.is_some(),
        explore.lifecycle.searched_once,
        explore.lifecycle.download_only_mode,
        true,
    );

    assert_eq!(snapshot.total_matches, Some(3));
    assert!(snapshot.sparql_query.is_some());
    assert_eq!(phase, ContentPhase::Loaded);
}

#[test]
fn integration_curation_rows_tsv_round_trip_keeps_expected_header() {
    let rows = vec![crate::curation::CurationInputRow {
        name: "A name".to_string(),
        smiles: "CCO".to_string(),
        taxon: Some("Rosa canina".to_string()),
        doi: Some("10.1000/ABC".to_string()),
    }];

    let tsv = rows_to_tsv(&rows);
    let parsed = crate::curation::parse_tsv_rows(&tsv).expect("tsv should parse");

    assert!(tsv.starts_with("name\tsmiles\ttaxon\tdoi\n"));
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].name, "A name");
    assert_eq!(parsed[0].smiles, "CCO");
}
