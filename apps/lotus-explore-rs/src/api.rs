// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

mod client;
mod config;
mod dto;
mod error;

#[cfg(target_arch = "wasm32")]
pub use client::export_urls;
pub use client::search;
pub use config::api_base_url;
#[cfg(target_arch = "wasm32")]
pub use dto::ExportUrlResponse;
pub use dto::SearchResponse;
pub use error::ApiClientError;
