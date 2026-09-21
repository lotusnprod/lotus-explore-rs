// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Search criteria: taxon, structure, mass, year, and molecular-formula filters.
//!
//! [`SearchCriteria`] is the canonical filter set used by `lotus-explore-rs`'s
//! server (`/v1/search`) and client (UI form).  It serializes to URL
//! query parameters via [`SearchCriteria::shareable_query_params`].

use super::runtime::current_year;
use super::stats::{ElementState, SmilesSearchType};
use super::{
    DEFAULT_C_MAX, DEFAULT_H_MAX, DEFAULT_N_MAX, DEFAULT_O_MAX, DEFAULT_P_MAX, DEFAULT_S_MAX,
    DEFAULT_YEAR_MIN,
};

/// Search criteria for LOTUS compound queries.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchCriteria {
    pub taxon: String,
    pub smiles: String,
    pub smiles_search_type: SmilesSearchType,
    pub smiles_threshold: f64,
    pub mass_min: f64,
    pub mass_max: f64,
    pub year_min: u16,
    pub year_max: u16,
    pub formula_enabled: bool,
    pub formula_exact: String,
    pub c_min: u16,
    pub c_max: u16,
    pub h_min: u16,
    pub h_max: u16,
    pub n_min: u16,
    pub n_max: u16,
    pub o_min: u16,
    pub o_max: u16,
    pub p_min: u16,
    pub p_max: u16,
    pub s_min: u16,
    pub s_max: u16,
    pub f_state: ElementState,
    pub cl_state: ElementState,
    pub br_state: ElementState,
    pub i_state: ElementState,
}

impl Default for SearchCriteria {
    fn default() -> Self {
        Self {
            taxon: "Gentiana lutea".into(),
            smiles: String::new(),
            smiles_search_type: SmilesSearchType::Substructure,
            smiles_threshold: 0.8,
            mass_min: 0.0,
            mass_max: 10000.0,
            year_min: DEFAULT_YEAR_MIN,
            year_max: current_year(),
            formula_enabled: false,
            formula_exact: String::new(),
            c_min: 0,
            c_max: DEFAULT_C_MAX,
            h_min: 0,
            h_max: DEFAULT_H_MAX,
            n_min: 0,
            n_max: DEFAULT_N_MAX,
            o_min: 0,
            o_max: DEFAULT_O_MAX,
            p_min: 0,
            p_max: DEFAULT_P_MAX,
            s_min: 0,
            s_max: DEFAULT_S_MAX,
            f_state: ElementState::Allowed,
            cl_state: ElementState::Allowed,
            br_state: ElementState::Allowed,
            i_state: ElementState::Allowed,
        }
    }
}

impl SearchCriteria {
    /// Returns `true` if any mass filter (non-default min/max) is active.
    #[must_use]
    pub fn has_mass_filter(&self) -> bool {
        self.mass_min > 0.0 || self.mass_max < 10000.0
    }

    /// Returns `true` if any year filter (non-default min/max) is active.
    #[must_use]
    pub fn has_year_filter(&self) -> bool {
        self.year_min > DEFAULT_YEAR_MIN || self.year_max < current_year()
    }

    /// Returns the six element-range tuples `(label, min, max, default_max)`.
    #[must_use]
    pub const fn element_ranges(&self) -> [(&'static str, u16, u16, u16); 6] {
        [
            ("C", self.c_min, self.c_max, DEFAULT_C_MAX),
            ("H", self.h_min, self.h_max, DEFAULT_H_MAX),
            ("N", self.n_min, self.n_max, DEFAULT_N_MAX),
            ("O", self.o_min, self.o_max, DEFAULT_O_MAX),
            ("P", self.p_min, self.p_max, DEFAULT_P_MAX),
            ("S", self.s_min, self.s_max, DEFAULT_S_MAX),
        ]
    }

    /// Returns `true` if formula filtering is enabled AND any formula sub-filter
    /// is non-default.
    #[must_use]
    pub fn has_formula_filter(&self) -> bool {
        self.formula_enabled
            && (!self.formula_exact.trim().is_empty()
                || self
                    .element_ranges()
                    .iter()
                    .any(|(_, min, max, default_max)| *min > 0 || *max < *default_max)
                || self.f_state != ElementState::Allowed
                || self.cl_state != ElementState::Allowed
                || self.br_state != ElementState::Allowed
                || self.i_state != ElementState::Allowed)
    }

    /// Returns `true` if any filter (structure, mass, year, formula) is active.
    #[must_use]
    pub fn has_effective_filters(&self) -> bool {
        !self.smiles.trim().is_empty()
            || self.has_mass_filter()
            || self.has_year_filter()
            || self.has_formula_filter()
    }

    /// Returns `true` if the criteria has a non-empty taxon or structure.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.taxon.trim().is_empty() || !self.smiles.trim().is_empty()
    }

    /// Serialise to URL query parameters, omitting default-valued fields so
    /// the URL stays compact and shareable.
    #[must_use]
    pub fn shareable_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();
        if !self.taxon.trim().is_empty() {
            params.push(("taxon".to_string(), self.taxon.clone()));
        }
        if !self.smiles.trim().is_empty() {
            params.push(("structure".to_string(), self.smiles.clone()));
            params.push((
                "structure_search_type".to_string(),
                self.smiles_search_type.as_str().to_string(),
            ));
            if self.smiles_search_type == SmilesSearchType::Similarity {
                params.push((
                    "smiles_threshold".to_string(),
                    format!("{:.2}", self.smiles_threshold),
                ));
            }
        }
        if self.has_mass_filter() {
            params.push(("mass_filter".to_string(), "true".to_string()));
            params.push(("mass_min".to_string(), format!("{}", self.mass_min)));
            params.push(("mass_max".to_string(), format!("{}", self.mass_max)));
        }
        if self.has_year_filter() {
            params.push(("year_filter".to_string(), "true".to_string()));
            params.push(("year_start".to_string(), format!("{}", self.year_min)));
            params.push(("year_end".to_string(), format!("{}", self.year_max)));
        }
        if self.formula_enabled {
            params.push(("formula_filter".to_string(), "true".to_string()));
            if !self.formula_exact.trim().is_empty() {
                params.push(("formula_exact".to_string(), self.formula_exact.clone()));
            }
            for (label, min, max, default_max) in self.element_ranges() {
                let key = label.to_ascii_lowercase();
                if min > 0 {
                    params.push((format!("{key}_min"), min.to_string()));
                }
                if max < default_max {
                    params.push((format!("{key}_max"), max.to_string()));
                }
            }
            for (label, state) in [
                ("f", self.f_state),
                ("cl", self.cl_state),
                ("br", self.br_state),
                ("i", self.i_state),
            ] {
                if state != ElementState::Allowed {
                    params.push((format!("{label}_state"), state.as_str().to_string()));
                }
            }
        }
        params
    }
}
