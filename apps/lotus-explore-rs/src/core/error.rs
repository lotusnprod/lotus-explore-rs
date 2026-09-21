// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Runtime error model.
//!
//! ## Design History
//!
//! This module previously defined `AppError`, `ErrorKind`, and `ValidationError`
//! as an intermediate error layer between `ApiClientError` and `RepositoryError`.
//! That layer was removed in favor of a direct `From<ApiClientError>` implementation
//! on `RepositoryError`, which reduces the conversion chain from 4 hops to 2:
//!
//! **Before:** `ApiClientError → AppError → RepositoryError → DomainError`
//! **After:**  `ApiClientError → RepositoryError → DomainError`
//!
//! The authoritative error types for each layer are:
//!
//! | Layer          | Type                                      |
//! |----------------|-------------------------------------------|
//! | Transport      | `crate::api::ApiClientError`              |
//! | Repository     | `crate::repositories::RepositoryError`    |
//! | Domain/Feature | `crate::features::explore::types::DomainError` |
//!
//! ## Future Extension
//!
//! If a genuine cross-cutting error concern arises (e.g. structured logging,
//! telemetry correlation IDs), introduce it here rather than adding a new
//! conversion layer.
