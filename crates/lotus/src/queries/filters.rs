// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Server-side filter injection for SPARQL queries.
//!
//! When the user applies mass-range, publication-year, or molecular-formula
//! filters, these functions inject the appropriate `FILTER()` and
//! `required_inserts` triples into the query's WHERE block.  The mass and
//! year filters use "required inserts" (non-OPTIONAL triples) so that `QLever`
//! can plan joins more efficiently — the filter is applied early, before
//! OPTIONAL enrichment.

use crate::models::{
    DEFAULT_C_MAX, DEFAULT_H_MAX, DEFAULT_N_MAX, DEFAULT_O_MAX, DEFAULT_P_MAX, DEFAULT_S_MAX,
    ElementState, SearchCriteria,
};
use crate::queries::formula::{
    element_count_bind, normalize_digits_expr, normalize_formula_digits,
};

/// Apply mass-range filters (P2067 — molecular weight).
///
/// Inserts `?c wdt:P2067 ?compound_mass .` as a **required** triple so the
/// `FILTER()` runs before OPTIONAL clauses.
fn apply_mass_filters(
    criteria: &SearchCriteria,
    filters: &mut Vec<String>,
    required_inserts: &mut Vec<String>,
) {
    if !criteria.has_mass_filter() {
        return;
    }
    filters.push(format!(
        "FILTER(?compound_mass >= {:.6} && ?compound_mass <= {:.6})",
        criteria.mass_min, criteria.mass_max
    ));
    required_inserts.push("?c wdt:P2067 ?compound_mass .".to_string());
}

/// Apply publication-year filters (`YEAR(?ref_date)` on P577).
///
/// Inserts `?r wdt:P577 ?ref_date .` as a **required** triple.
fn apply_year_filters(
    criteria: &SearchCriteria,
    filters: &mut Vec<String>,
    required_inserts: &mut Vec<String>,
) {
    if !criteria.has_year_filter() {
        return;
    }
    filters.push(format!(
        "FILTER(YEAR(?ref_date) >= {} && YEAR(?ref_date) <= {})",
        criteria.year_min, criteria.year_max
    ));
    required_inserts.push("?r wdt:P577 ?ref_date .".to_string());
}

/// Apply molecular-formula filters (element counts, halogens, exact formula).
///
/// Uses pre-computed element-count BINDs with subscript-digit normalization
/// to avoid repeated regex evaluation per row.
fn apply_formula_filters(
    criteria: &SearchCriteria,
    prelude: &mut Vec<String>,
    filters: &mut Vec<String>,
) {
    if !criteria.has_formula_filter() {
        return;
    }

    prelude.push("FILTER(BOUND(?compound_formula_raw))".to_string());
    prelude.push("BIND(STR(?compound_formula_raw) AS ?_formula_raw)".to_string());
    prelude.push(r#"BIND(REPLACE(?_formula_raw, " ", "") AS ?_formula_nospace)"#.to_string());
    prelude.push(format!(
        "BIND({} AS ?_formula_norm)",
        normalize_digits_expr("?_formula_nospace")
    ));
    prelude.push(
        "BIND(REPLACE(?_formula_norm, \"([A-Z])\", \"|$1\") AS ?_formula_tokens)".to_string(),
    );

    for (symbol, min, max, default_max) in [
        ("C", criteria.c_min, criteria.c_max, DEFAULT_C_MAX),
        ("H", criteria.h_min, criteria.h_max, DEFAULT_H_MAX),
        ("N", criteria.n_min, criteria.n_max, DEFAULT_N_MAX),
        ("O", criteria.o_min, criteria.o_max, DEFAULT_O_MAX),
        ("P", criteria.p_min, criteria.p_max, DEFAULT_P_MAX),
        ("S", criteria.s_min, criteria.s_max, DEFAULT_S_MAX),
    ] {
        if min > 0 || max < default_max {
            let var = format!("?_count_{}", symbol.to_ascii_lowercase());
            prelude.push(element_count_bind(symbol, &var));
            filters.push(format!("FILTER({var} >= {min} && {var} <= {max})"));
        }
    }

    for (symbol, state) in [
        ("F", criteria.f_state),
        ("Cl", criteria.cl_state),
        ("Br", criteria.br_state),
        ("I", criteria.i_state),
    ] {
        if state != ElementState::Allowed {
            let var = format!("?_count_{}", symbol.to_ascii_lowercase());
            prelude.push(element_count_bind(symbol, &var));
            match state {
                ElementState::Allowed => {}
                ElementState::Required => filters.push(format!("FILTER({var} > 0)")),
                ElementState::Excluded => filters.push(format!("FILTER({var} = 0)")),
            }
        }
    }

    if let Some(exact) = criteria
        .formula_enabled
        .then_some(criteria.formula_exact.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let exact_norm: String = normalize_formula_digits(exact)
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        let exact_escaped = exact_norm.replace('\\', r"\\").replace('"', r#"\""#);
        filters.push(format!("FILTER(?_formula_norm = \"{exact_escaped}\")"));
    }
}

/// Inject filter fragments (required inserts, prelude, and filters) into a
/// SPARQL query's WHERE block.
///
/// Inserts `required_inserts` and `prelude` just inside the last `}` before
/// `filters`, preserving the query's structural integrity.
fn inject_filter_fragments(
    base_query: &str,
    required_inserts: &[String],
    prelude: &[String],
    filters: &[String],
) -> String {
    let trimmed = base_query.trim_end();
    let Some(last_close) = trimmed.rfind('}') else {
        let mut out = String::with_capacity((filters.len() + required_inserts.len()) * 100);
        out.push_str(trimmed);
        for insert in required_inserts {
            out.push('\n');
            out.push_str(insert);
        }
        for filter in filters {
            out.push('\n');
            out.push_str(filter);
        }
        return out;
    };

    let mut out = String::with_capacity(
        trimmed.len() + (filters.len() + prelude.len() + required_inserts.len()) * 100,
    );
    out.push_str(&trimmed[..last_close]);
    out.push('\n');

    for insert in required_inserts {
        out.push_str(insert);
        out.push('\n');
    }
    if !prelude.is_empty() {
        out.push_str(&prelude.join("\n"));
        out.push('\n');
    }
    out.push_str(&filters.join("\n"));
    out.push('\n');
    out.push_str(&trimmed[last_close..]);
    out
}

/// Apply server-side filtering conditions (mass, year, molecular formula) to
/// a base query.
///
/// Combines mass, year, and molecular-formula filters from [`SearchCriteria`]
/// and injects them into the query's WHERE block.  When no filters are active,
/// the base query is returned unchanged.
#[must_use]
pub fn query_with_server_filters(base_query: &str, criteria: &SearchCriteria) -> String {
    let mut filters = Vec::new();
    let mut prelude = Vec::new();
    let mut required_inserts = Vec::new();

    apply_mass_filters(criteria, &mut filters, &mut required_inserts);
    apply_year_filters(criteria, &mut filters, &mut required_inserts);
    apply_formula_filters(criteria, &mut prelude, &mut filters);

    if prelude.is_empty() && filters.is_empty() && required_inserts.is_empty() {
        return base_query.to_string();
    }

    inject_filter_fragments(base_query, &required_inserts, &prelude, &filters)
}
