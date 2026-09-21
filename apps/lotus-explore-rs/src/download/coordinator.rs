// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! The consolidated [`lotus::export::ExportFormat`] from the `lotus` crate is re-exported
//! in [`crate::download`] as `DownloadFormat`.
//!
//! The action strings (`"csv_export"`, `"qlever_json_export"`, `"turtle_export"`)
//! and the CONSTRUCT-wrapping for RDF are now defined once in
//! `lotus::export::ExportFormat`.
