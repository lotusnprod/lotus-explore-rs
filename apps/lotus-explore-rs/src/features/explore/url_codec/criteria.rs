// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

use super::{QueryParams, is_true_flag};
use crate::models::{ElementState, SearchCriteria, SmilesSearchType};
use std::str::FromStr;

#[derive(Clone, Debug, Default, PartialEq)]
struct CriteriaQueryDto {
    taxon: Option<String>,
    structure: Option<String>,
    structure_search_type: Option<SmilesSearchType>,
    smiles_threshold: Option<f64>,
    mass_filter: Option<RangeF64Dto>,
    year_filter: Option<RangeU16Dto>,
    formula_filter: Option<FormulaQueryDto>,
    has_explicit_taxon: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct FormulaQueryDto {
    exact: Option<String>,
    c_min: Option<u16>,
    c_max: Option<u16>,
    h_min: Option<u16>,
    h_max: Option<u16>,
    n_min: Option<u16>,
    n_max: Option<u16>,
    o_min: Option<u16>,
    o_max: Option<u16>,
    p_min: Option<u16>,
    p_max: Option<u16>,
    s_min: Option<u16>,
    s_max: Option<u16>,
    f_state: Option<ElementState>,
    cl_state: Option<ElementState>,
    br_state: Option<ElementState>,
    i_state: Option<ElementState>,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct RangeF64Dto {
    min: Option<f64>,
    max: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct RangeU16Dto {
    min: Option<u16>,
    max: Option<u16>,
}

pub fn parse_criteria_from_params(params: &QueryParams) -> SearchCriteria {
    CriteriaQueryDto::parse(params).into_criteria()
}

impl CriteriaQueryDto {
    fn parse(params: &QueryParams) -> Self {
        let taxon = params.get("taxon").cloned();
        Self {
            has_explicit_taxon: taxon.is_some(),
            taxon,
            structure: params
                .get("structure")
                .cloned()
                .or_else(|| params.get("smiles").cloned()),
            structure_search_type: params
                .get("structure_search_type")
                .map(String::as_str)
                .or_else(|| params.get("smiles_search_type").map(String::as_str))
                .map(parse_search_type),
            smiles_threshold: parse_positive_threshold(
                params.get("smiles_threshold").map(String::as_str),
            ),
            mass_filter: RangeF64Dto::parse_when_enabled(
                params,
                "mass_filter",
                "mass_min",
                "mass_max",
            ),
            year_filter: RangeU16Dto::parse_when_enabled(
                params,
                "year_filter",
                "year_start",
                "year_end",
            ),
            formula_filter: FormulaQueryDto::parse(params),
        }
    }

    fn into_criteria(self) -> SearchCriteria {
        let mut criteria = SearchCriteria::default();
        if let Some(taxon) = self.taxon {
            criteria.taxon = taxon;
        }
        if let Some(structure) = self.structure {
            criteria.smiles = structure;
        }
        if let Some(search_type) = self.structure_search_type {
            criteria.smiles_search_type = search_type;
        }
        if let Some(threshold) = self.smiles_threshold {
            criteria.smiles_threshold = threshold;
        }
        if let Some(range) = self.mass_filter {
            range.apply(
                &mut criteria,
                |c, value| c.mass_min = value,
                |c, value| c.mass_max = value,
            );
        }
        if let Some(range) = self.year_filter {
            range.apply(
                &mut criteria,
                |c, value| c.year_min = value,
                |c, value| c.year_max = value,
            );
        }
        if let Some(formula) = self.formula_filter {
            formula.apply(&mut criteria);
        }

        if criteria.smiles.trim().is_empty() || self.has_explicit_taxon {
            return criteria;
        }

        criteria.taxon.clear();
        criteria
    }
}

impl FormulaQueryDto {
    fn parse(params: &QueryParams) -> Option<Self> {
        params
            .get("formula_filter")
            .is_some_and(|v| is_true_flag(v))
            .then(|| Self {
                exact: params.get("formula_exact").cloned(),
                c_min: parse_param(params, "c_min"),
                c_max: parse_param(params, "c_max"),
                h_min: parse_param(params, "h_min"),
                h_max: parse_param(params, "h_max"),
                n_min: parse_param(params, "n_min"),
                n_max: parse_param(params, "n_max"),
                o_min: parse_param(params, "o_min"),
                o_max: parse_param(params, "o_max"),
                p_min: parse_param(params, "p_min"),
                p_max: parse_param(params, "p_max"),
                s_min: parse_param(params, "s_min"),
                s_max: parse_param(params, "s_max"),
                f_state: parse_element_state(params, "f_state"),
                cl_state: parse_element_state(params, "cl_state"),
                br_state: parse_element_state(params, "br_state"),
                i_state: parse_element_state(params, "i_state"),
            })
    }

    fn apply(self, criteria: &mut SearchCriteria) {
        // Declarative field assignment: moves value from Option<T> into criteria field when Some.
        macro_rules! set_if_some {
            ($src:expr => $dst:expr) => {
                if let Some(v) = $src {
                    $dst = v;
                }
            };
        }

        criteria.formula_enabled = true;
        set_if_some!(self.exact    => criteria.formula_exact);
        set_if_some!(self.c_min   => criteria.c_min);
        set_if_some!(self.c_max   => criteria.c_max);
        set_if_some!(self.h_min   => criteria.h_min);
        set_if_some!(self.h_max   => criteria.h_max);
        set_if_some!(self.n_min   => criteria.n_min);
        set_if_some!(self.n_max   => criteria.n_max);
        set_if_some!(self.o_min   => criteria.o_min);
        set_if_some!(self.o_max   => criteria.o_max);
        set_if_some!(self.p_min   => criteria.p_min);
        set_if_some!(self.p_max   => criteria.p_max);
        set_if_some!(self.s_min   => criteria.s_min);
        set_if_some!(self.s_max   => criteria.s_max);
        set_if_some!(self.f_state  => criteria.f_state);
        set_if_some!(self.cl_state => criteria.cl_state);
        set_if_some!(self.br_state => criteria.br_state);
        set_if_some!(self.i_state  => criteria.i_state);
    }
}

impl RangeF64Dto {
    fn parse_when_enabled(
        params: &QueryParams,
        enabled_key: &str,
        min_key: &str,
        max_key: &str,
    ) -> Option<Self> {
        params
            .get(enabled_key)
            .is_some_and(|v| is_true_flag(v))
            .then(|| Self {
                min: parse_param(params, min_key),
                max: parse_param(params, max_key),
            })
    }

    fn apply(
        self,
        criteria: &mut SearchCriteria,
        set_min: impl FnOnce(&mut SearchCriteria, f64),
        set_max: impl FnOnce(&mut SearchCriteria, f64),
    ) {
        if let Some(value) = self.min {
            set_min(criteria, value);
        }
        if let Some(value) = self.max {
            set_max(criteria, value);
        }
    }
}

impl RangeU16Dto {
    fn parse_when_enabled(
        params: &QueryParams,
        enabled_key: &str,
        min_key: &str,
        max_key: &str,
    ) -> Option<Self> {
        params
            .get(enabled_key)
            .is_some_and(|v| is_true_flag(v))
            .then(|| Self {
                min: parse_param(params, min_key),
                max: parse_param(params, max_key),
            })
    }

    fn apply(
        self,
        criteria: &mut SearchCriteria,
        set_min: impl FnOnce(&mut SearchCriteria, u16),
        set_max: impl FnOnce(&mut SearchCriteria, u16),
    ) {
        if let Some(value) = self.min {
            set_min(criteria, value);
        }
        if let Some(value) = self.max {
            set_max(criteria, value);
        }
    }
}

fn parse_param<T: FromStr>(params: &QueryParams, name: &str) -> Option<T> {
    params.get(name).and_then(|value| value.parse::<T>().ok())
}

fn parse_element_state(params: &QueryParams, name: &str) -> Option<ElementState> {
    params
        .get(name)
        .map(|value| ElementState::from_str(value).unwrap_or_default())
}

fn parse_search_type(value: &str) -> SmilesSearchType {
    if value.eq_ignore_ascii_case("similarity") {
        SmilesSearchType::Similarity
    } else {
        SmilesSearchType::Substructure
    }
}

fn parse_positive_threshold(value: Option<&str>) -> Option<f64> {
    value
        .and_then(|raw| raw.parse::<f64>().ok())
        .filter(|v| v.is_finite() && *v > 0.0)
        .map(|v| v.clamp(0.05, 1.0))
}
