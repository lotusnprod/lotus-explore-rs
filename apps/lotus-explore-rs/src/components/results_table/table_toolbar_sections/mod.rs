// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Subcomponents for the `ResultsTable` toolbar sections.
//!
//! Each submodule owns exactly one visual concern:
//! * [`stat_bar`] — dataset statistics badges and the capped-rows notice
//! * [`download_actions`] — download button group (CSV, JSON, RDF, metadata) and `QLever` link
//! * [`query_panel`] — SPARQL query display panel with copy functionality

mod download_actions;
mod query_panel;
mod stat_bar;

pub use download_actions::DownloadActionsGroup;
pub use query_panel::QueryPanel;
pub use stat_bar::{CappedRowsNotice, StatBar};
