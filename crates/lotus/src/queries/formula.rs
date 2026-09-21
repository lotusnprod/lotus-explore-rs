// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Subscript-digit normalization and formula BIND-expression generation.
//!
//! Wikidata sometimes stores chemical formulas using Unicode subscript digits
//! (₀₁₂₃…₉).  These helpers build SPARQL `REPLACE` chains that normalize them
//! to ASCII so downstream filters can compare formula strings reliably.

use crate::queries::consts::SUBSCRIPT_DIGIT_MAPPINGS;

/// Build a nested `REPLACE(…, "₀", "0")`-style SPARQL expression that
/// normalizes subscript digits in a variable to ASCII.
///
/// The expression is built left-to-right:
/// ```text
/// REPLACE(REPLACE(STR(?var), "₀", "0"), "₁", "1")…
/// ```
/// This is more efficient than deeply nested SELECT clauses or multiple BINDs.
#[must_use]
pub(super) fn normalize_digits_expr(var: &str) -> String {
    SUBSCRIPT_DIGIT_MAPPINGS.iter().fold(
        format!("STR({var})"),
        |acc, &(subscript_char, ascii_digit)| {
            format!(r#"REPLACE({acc}, "{subscript_char}", "{ascii_digit}")"#)
        },
    )
}

/// Build a SPARQL BIND expression that extracts the count of a given element
/// symbol from a tokenized formula string.
///
/// The regex pattern `\\|symbol([0-9]*)(\\||$)` captures the number following
/// the element symbol.  If the element is absent, the BIND defaults to `0`;
/// if present without an explicit count, it defaults to `1`.
#[must_use]
pub(super) fn element_count_bind(symbol: &str, out_var: &str) -> String {
    let escaped = symbol.replace('"', "\\\"");
    let pattern = format!(r"\\|{escaped}([0-9]*)(\\||$)");
    let capture_expr = format!(r#"REPLACE(?_formula_tokens, ".*{pattern}.*", "$1")"#);
    format!(
        "BIND(IF(REGEX(?_formula_tokens, \"{pattern}\"), IF(STRLEN({capture_expr}) = 0, 1, xsd:integer({capture_expr})), 0) AS {out_var})"
    )
}

/// Normalize subscript digits in a Rust string (₀ → 0, ₁ → 1, etc.).
pub(super) fn normalize_formula_digits(s: &str) -> String {
    s.chars().map(normalize_formula_digit).collect()
}

/// Normalize a single subscript digit character, identity for non-subscript.
pub(super) fn normalize_formula_digit(c: char) -> char {
    SUBSCRIPT_DIGIT_MAPPINGS
        .iter()
        .find_map(|(from, to)| (*from == c).then_some(*to))
        .unwrap_or(c)
}
