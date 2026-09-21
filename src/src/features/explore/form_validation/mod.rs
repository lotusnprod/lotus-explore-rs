// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Form input validation framework.
//!
//! Centralizes validation logic for search form inputs to enable:
//! * Consistent validation rules across components
//! * Reusable validators for common patterns (ranges, numbers, strings)
//! * Type-safe validation results
//! * Clear error messages
//!
//! ## Pattern: Validator Functions
//!
//! Each validator is a pure function that takes raw input and returns a `Result`.
//! Validators can be composed to build complex validation pipelines.

mod dispatch;
mod rules;
mod types;

pub use dispatch::validate_dispatch_criteria;

#[cfg(test)]
mod tests;
