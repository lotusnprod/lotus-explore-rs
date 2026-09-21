// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

use super::Locale;

mod de;
mod en;
mod fr;
mod it;

pub fn err_invalid_search_input(locale: Locale) -> String {
    match locale {
        Locale::En => en::err_invalid_search_input(),
        Locale::Fr => fr::err_invalid_search_input(),
        Locale::De => de::err_invalid_search_input(),
        Locale::It => it::err_invalid_search_input(),
    }
}

pub fn err_api_not_configured(locale: Locale) -> String {
    match locale {
        Locale::En => en::err_api_not_configured(),
        Locale::Fr => fr::err_api_not_configured(),
        Locale::De => de::err_api_not_configured(),
        Locale::It => it::err_api_not_configured(),
    }
}

pub fn err_taxon_too_long(locale: Locale) -> String {
    match locale {
        Locale::En => en::err_taxon_too_long(),
        Locale::Fr => fr::err_taxon_too_long(),
        Locale::De => de::err_taxon_too_long(),
        Locale::It => it::err_taxon_too_long(),
    }
}

pub fn err_structure_too_long(locale: Locale) -> String {
    match locale {
        Locale::En => en::err_structure_too_long(),
        Locale::Fr => fr::err_structure_too_long(),
        Locale::De => de::err_structure_too_long(),
        Locale::It => it::err_structure_too_long(),
    }
}

pub fn err_mass_out_of_range(locale: Locale) -> String {
    match locale {
        Locale::En => en::err_mass_out_of_range(),
        Locale::Fr => fr::err_mass_out_of_range(),
        Locale::De => de::err_mass_out_of_range(),
        Locale::It => it::err_mass_out_of_range(),
    }
}

pub fn err_mass_range_invalid(locale: Locale) -> String {
    match locale {
        Locale::En => en::err_mass_range_invalid(),
        Locale::Fr => fr::err_mass_range_invalid(),
        Locale::De => de::err_mass_range_invalid(),
        Locale::It => it::err_mass_range_invalid(),
    }
}

pub fn err_year_out_of_range(locale: Locale) -> String {
    match locale {
        Locale::En => en::err_year_out_of_range(),
        Locale::Fr => fr::err_year_out_of_range(),
        Locale::De => de::err_year_out_of_range(),
        Locale::It => it::err_year_out_of_range(),
    }
}

pub fn err_year_range_invalid(locale: Locale) -> String {
    match locale {
        Locale::En => en::err_year_range_invalid(),
        Locale::Fr => fr::err_year_range_invalid(),
        Locale::De => de::err_year_range_invalid(),
        Locale::It => it::err_year_range_invalid(),
    }
}

pub fn err_element_count_too_high(locale: Locale) -> String {
    match locale {
        Locale::En => en::err_element_count_too_high(),
        Locale::Fr => fr::err_element_count_too_high(),
        Locale::De => de::err_element_count_too_high(),
        Locale::It => it::err_element_count_too_high(),
    }
}

pub fn err_similarity_threshold_invalid(locale: Locale) -> String {
    match locale {
        Locale::En => en::err_similarity_threshold_invalid(),
        Locale::Fr => fr::err_similarity_threshold_invalid(),
        Locale::De => de::err_similarity_threshold_invalid(),
        Locale::It => it::err_similarity_threshold_invalid(),
    }
}

pub fn err_unsupported_format(locale: Locale, fmt: &str) -> String {
    match locale {
        Locale::En => en::err_unsupported_format(fmt),
        Locale::Fr => fr::err_unsupported_format(fmt),
        Locale::De => de::err_unsupported_format(fmt),
        Locale::It => it::err_unsupported_format(fmt),
    }
}

pub fn err_taxon_parse_failed(locale: Locale, detail: &str) -> String {
    match locale {
        Locale::En => en::err_taxon_parse_failed(detail),
        Locale::Fr => fr::err_taxon_parse_failed(detail),
        Locale::De => de::err_taxon_parse_failed(detail),
        Locale::It => it::err_taxon_parse_failed(detail),
    }
}

pub fn err_query_stage_failed(locale: Locale, stage: &str, detail: &str) -> String {
    match locale {
        Locale::En => en::err_query_stage_failed(stage, detail),
        Locale::Fr => fr::err_query_stage_failed(stage, detail),
        Locale::De => de::err_query_stage_failed(stage, detail),
        Locale::It => it::err_query_stage_failed(stage, detail),
    }
}

pub fn err_taxon_not_found(locale: Locale, taxon: &str) -> String {
    match locale {
        Locale::En => en::err_taxon_not_found(taxon),
        Locale::Fr => fr::err_taxon_not_found(taxon),
        Locale::De => de::err_taxon_not_found(taxon),
        Locale::It => it::err_taxon_not_found(taxon),
    }
}

pub fn warn_input_standardized(locale: Locale, original: &str, normalized: &str) -> String {
    match locale {
        Locale::En => en::warn_input_standardized(original, normalized),
        Locale::Fr => fr::warn_input_standardized(original, normalized),
        Locale::De => de::warn_input_standardized(original, normalized),
        Locale::It => it::warn_input_standardized(original, normalized),
    }
}

pub fn warn_ambiguous_taxon(
    locale: Locale,
    best_name: &str,
    best_qid: &str,
    names: &str,
) -> String {
    match locale {
        Locale::En => en::warn_ambiguous_taxon(best_name, best_qid, names),
        Locale::Fr => fr::warn_ambiguous_taxon(best_name, best_qid, names),
        Locale::De => de::warn_ambiguous_taxon(best_name, best_qid, names),
        Locale::It => it::warn_ambiguous_taxon(best_name, best_qid, names),
    }
}

pub fn warn_qlever_bad_gateway(locale: Locale) -> String {
    match locale {
        Locale::En => en::warn_qlever_bad_gateway(),
        Locale::Fr => fr::warn_qlever_bad_gateway(),
        Locale::De => de::warn_qlever_bad_gateway(),
        Locale::It => it::warn_qlever_bad_gateway(),
    }
}

pub fn warn_wdqs_fallback(locale: Locale) -> String {
    match locale {
        Locale::En => en::warn_wdqs_fallback(),
        Locale::Fr => fr::warn_wdqs_fallback(),
        Locale::De => de::warn_wdqs_fallback(),
        Locale::It => it::warn_wdqs_fallback(),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn error_hint_memory(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::error_hint_memory(),
        Locale::Fr => fr::error_hint_memory(),
        Locale::De => de::error_hint_memory(),
        Locale::It => it::error_hint_memory(),
    }
}
