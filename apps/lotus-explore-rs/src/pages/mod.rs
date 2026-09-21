// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Page-level components: top-level views for each application section.
//!
//! Page components represent full-page views in the application, typically
//! managed by the router or main shell component. They compose together
//! smaller, reusable UI components from the `components` module.
//!
//! ## Pages
//! - `draw_page` - Structure editor tab with Ketcher molecule editor
//! - `ketcher_panel` - Panel for Ketcher molecule editor

pub mod draw_page;
pub mod ketcher_panel;

pub use draw_page::DrawPage;
