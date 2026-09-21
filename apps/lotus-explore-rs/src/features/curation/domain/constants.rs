// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

#[cfg(not(target_arch = "wasm32"))]
pub const NATPROD_API_BASE: &str = "https://api.naturalproducts.net/latest";

pub const CURATION_SPARQL_PREFIXES: &str = "\
PREFIX wd: <http://www.wikidata.org/entity/>\n\
PREFIX wdt: <http://www.wikidata.org/prop/direct/>\n\
PREFIX p: <http://www.wikidata.org/prop/>\n\
PREFIX ps: <http://www.wikidata.org/prop/statement/>\n\
PREFIX prov: <http://www.w3.org/ns/prov#>\n\
PREFIX pr: <http://www.wikidata.org/prop/reference/>\n\
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>";

pub const WD_CHEMICAL_COMPOUND_QID: &str = "Q11173";
pub const WD_TYPE_CHEMICAL_ENTITY_QID: &str = "Q113145171";
pub const WD_STEREOISOMER_GROUP_QID: &str = "Q59199015";
pub const WD_OCCURS_IN_TAXON_PROP: &str = "P703";
pub const WD_TAXON_QID: &str = "Q16521";
