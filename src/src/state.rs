// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

pub mod form_context;

use crate::app_state::AppState;
use crate::features::explore::ExploreState;
use dioxus::prelude::*;

pub use form_context::{FormCriteriaContext, use_form_criteria_context};

// ── App State Context ─────────────────────────────────────────────────────

/// Root context: access to the unified `AppState` (view, download, metrics).
///
/// Use this to read or mutate view-selection and download-orchestration state.
/// For search form state use [`FormCriteriaContext`]; for results and lifecycle
/// state use [`ResultsContext`].
#[derive(Clone, Copy)]
pub struct AppStateContext {
    pub state: Signal<AppState>,
}

impl AppStateContext {
    pub const fn new(state: Signal<AppState>) -> Self {
        Self { state }
    }
}

// ── Results Context ───────────────────────────────────────────────────────────

/// Context for results-area components.
///
/// Contains only the live `explore` signal — previously it also held a
/// `app_state` reference that was used to read a stale mirror copy of
/// `ExploreState`.  That mirror copy has been removed; components now read the
/// live signal directly.
#[derive(Clone, Copy)]
pub struct ResultsContext {
    /// Live explore signal — results, lifecycle, UI chrome.
    pub explore: Signal<ExploreState>,
}

impl ResultsContext {
    pub const fn new(explore: Signal<ExploreState>) -> Self {
        Self { explore }
    }
}

// ── Hook Helpers ──────────────────────────────────────────────────────────────

/// Hook to read the root AppStateContext from any descendant component.
pub fn use_app_state_context() -> AppStateContext {
    use_context::<AppStateContext>()
}

pub fn use_app_selector<T: PartialEq + Clone + 'static>(
    app_state: Signal<AppState>,
    f: impl Fn(&AppState) -> T + 'static,
) -> Memo<T> {
    use_memo(move || f(&app_state.read()))
}

pub fn use_results_context() -> ResultsContext {
    use_context::<ResultsContext>()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use crate::features::explore::{FormAction, apply_form_action};
    use crate::models::SearchCriteria;

    #[test]
    fn apply_form_action_taxon_round_trips() {
        let base = SearchCriteria::default();
        let updated = apply_form_action(base, FormAction::Taxon("Rosa".into()));
        assert_eq!(updated.taxon, "Rosa");
    }

    #[test]
    fn apply_form_action_mass_range_round_trips() {
        let base = SearchCriteria::default();
        let updated = apply_form_action(base, FormAction::MassMin(50.0));
        assert_eq!(updated.mass_min, 50.0);
    }

    #[test]
    fn form_action_all_element_bounds_round_trip() {
        use crate::models::ElementState;
        let base = SearchCriteria::default();
        let updated = apply_form_action(base.clone(), FormAction::CMin(6));
        assert_eq!(updated.c_min, 6);

        let updated = apply_form_action(base.clone(), FormAction::HMax(20));
        assert_eq!(updated.h_max, 20);

        let updated = apply_form_action(base.clone(), FormAction::FState(ElementState::Required));
        assert_eq!(updated.f_state, ElementState::Required);
    }

    #[test]
    fn form_action_formula_enabled_round_trips() {
        let base = SearchCriteria::default();
        assert!(!base.formula_enabled);
        let updated = apply_form_action(base, FormAction::FormulaEnabled(true));
        assert!(updated.formula_enabled);
    }

    #[test]
    fn form_action_smiles_search_type_round_trips() {
        use crate::models::SmilesSearchType;
        let base = SearchCriteria::default();
        let updated = apply_form_action(
            base,
            FormAction::SmilesSearchType(SmilesSearchType::Similarity),
        );
        assert_eq!(updated.smiles_search_type, SmilesSearchType::Similarity);
    }

    #[test]
    fn form_action_year_range_round_trips() {
        let base = SearchCriteria::default();
        let updated = apply_form_action(base.clone(), FormAction::YearMin(1990));
        assert_eq!(updated.year_min, 1990);
        let updated = apply_form_action(base, FormAction::YearMax(2025));
        assert_eq!(updated.year_max, 2025);
    }
}
