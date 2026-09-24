// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Export helpers split by concern:
//! - `filters`: criteria -> structured JSON
//! - `metadata`: schema.org dataset metadata generation
//! - `filename`: deterministic download filenames and timestamp helpers

mod filename;
mod filters;
mod metadata;

pub use filename::generate_filename;
pub use metadata::{APP_NAME, MetadataInputs, SparqlEndpoint, build_metadata_json};
