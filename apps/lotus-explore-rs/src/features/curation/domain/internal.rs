// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

#[derive(Debug)]
pub struct WikidataCompound {
    pub(crate) qid: String,
    pub(crate) canonical_smiles: Option<String>,
    pub(crate) isomeric_smiles: Option<String>,
    pub(crate) inchi: Option<String>,
    pub(crate) formula: Option<String>,
    pub(crate) mass: Option<f64>,
}

#[derive(Debug, Default)]
pub struct DependencyResolution {
    pub(crate) taxon_qid: Option<String>,
    pub(crate) reference_qid: Option<String>,
    pub(crate) dependency_blocks: Vec<String>,
    pub(crate) pending_messages: Vec<String>,
}

#[derive(Debug, Default)]
pub struct MassResolution {
    pub(crate) exact_mass: Option<f64>,
    pub(crate) warning: Option<String>,
}
