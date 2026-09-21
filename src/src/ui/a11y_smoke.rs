// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Lightweight accessibility smoke tests.
//!
//! These tests guard critical ARIA/landmark contracts in source markup so
//! regressions are detected early during refactors.

#[cfg(test)]
mod tests {
    #[test]
    fn main_landmark_is_labelled_and_skip_link_targets_it() {
        let shell_src = include_str!("../app/shell.rs");
        assert!(shell_src.contains("href: SKIP_TO_RESULTS_HREF"));
        assert!(shell_src.contains("id: MAIN_PANEL_ID"));
        assert!(shell_src.contains("aria_labelledby: PAGE_TITLE_ID"));
    }

    #[test]
    fn search_panel_exposes_heading_and_body_landmarks() {
        let search_panel_src = include_str!("../components/search_panel.rs");
        assert!(search_panel_src.contains("id: SEARCH_PANEL_BODY_ID"));
    }

    #[test]
    fn sortable_headers_expose_action_oriented_aria_label() {
        let header_src = include_str!("../components/results_table/table_header.rs");
        assert!(header_src.contains("aria_sort_toggle"));
        assert!(header_src.contains("aria_label: \"{sort_aria}\""));
    }

    #[test]
    fn page_header_exposes_single_home_link_and_heading_id() {
        let header_src = include_str!("../components/layout/page_header.rs");
        assert!(header_src.contains("h1 { id: PAGE_TITLE_ID"));
        assert!(header_src.contains("class: \"text-inherit no-underline hover:no-underline\""));
        // Home link uses visible text as accessible name (no redundant aria_label)
        assert!(header_src.contains("\"{t(locale, TextKey::PageTitle)}\""));
    }
}
