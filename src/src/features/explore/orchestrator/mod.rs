// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Search orchestration — thin dispatcher that sequences service calls.
//!
//! `start_search` validates input, dispatches `SearchRequested`, then spawns
//! `do_search`.  `do_search` delegates all I/O and business logic to the
//! `service::` modules so that those remain independently testable.

mod controller;
mod runtime;

pub use controller::SearchTaskController;
pub use runtime::start_search;

#[cfg(test)]
mod tests;
