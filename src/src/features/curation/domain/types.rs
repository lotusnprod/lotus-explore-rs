// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurationInputRow {
    pub name: String,
    pub smiles: String,
    pub taxon: Option<String>,
    pub doi: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CurationStatus {
    ExistingComplete,
    ExistingNeedsUpdates,
    NewCompound,
    PendingDependencies,
    Error,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CurationResultRow {
    pub input: CurationInputRow,
    pub canonical_smiles: Option<String>,
    pub inchikey: Option<String>,
    pub inchi: Option<String>,
    pub formula: Option<String>,
    pub exact_mass: Option<f64>,
    pub mass_warning: Option<String>,
    pub wikidata_qid: Option<String>,
    pub status: CurationStatus,
    pub note: String,
    pub dependency_blocks: Vec<String>,
    pub quickstatements: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuickStatementsBundle {
    pub dependencies: std::sync::Arc<str>,
    pub main: std::sync::Arc<str>,
}

impl Default for QuickStatementsBundle {
    fn default() -> Self {
        Self {
            dependencies: std::sync::Arc::<str>::from(""),
            main: std::sync::Arc::<str>::from(""),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurationErrorKind {
    InvalidInput,
    Transport,
    Parse,
}

#[derive(Debug, Error)]
pub enum CurationError {
    #[error("{0}")]
    InvalidInput(String),
    #[error("{0}")]
    Http(String),
    #[error("{0}")]
    Parse(String),
}

impl CurationError {
    pub const fn kind(&self) -> CurationErrorKind {
        match self {
            Self::InvalidInput(_) => CurationErrorKind::InvalidInput,
            Self::Http(_) => CurationErrorKind::Transport,
            Self::Parse(_) => CurationErrorKind::Parse,
        }
    }

    pub const fn is_recoverable(&self) -> bool {
        !matches!(self, Self::Parse(_))
    }
}
