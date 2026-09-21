// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

use super::Locale;

mod de;
mod en;
mod fr;
mod it;

pub fn heading_add_one_row(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::heading_add_one_row(),
        Locale::Fr => fr::heading_add_one_row(),
        Locale::De => de::heading_add_one_row(),
        Locale::It => it::heading_add_one_row(),
    }
}

pub fn heading_tsv_import(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::heading_tsv_import(),
        Locale::Fr => fr::heading_tsv_import(),
        Locale::De => de::heading_tsv_import(),
        Locale::It => it::heading_tsv_import(),
    }
}

pub fn heading_queued_rows(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::heading_queued_rows(),
        Locale::Fr => fr::heading_queued_rows(),
        Locale::De => de::heading_queued_rows(),
        Locale::It => it::heading_queued_rows(),
    }
}

pub fn heading_results(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::heading_results(),
        Locale::Fr => fr::heading_results(),
        Locale::De => de::heading_results(),
        Locale::It => it::heading_results(),
    }
}

pub fn heading_quickstatements(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::heading_quickstatements(),
        Locale::Fr => fr::heading_quickstatements(),
        Locale::De => de::heading_quickstatements(),
        Locale::It => it::heading_quickstatements(),
    }
}

pub fn heading_quickstatements_dependencies(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::heading_quickstatements_dependencies(),
        Locale::Fr => fr::heading_quickstatements_dependencies(),
        Locale::De => de::heading_quickstatements_dependencies(),
        Locale::It => it::heading_quickstatements_dependencies(),
    }
}

pub fn placeholder_molecule_name(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::placeholder_molecule_name(),
        Locale::Fr => fr::placeholder_molecule_name(),
        Locale::De => de::placeholder_molecule_name(),
        Locale::It => it::placeholder_molecule_name(),
    }
}

pub fn placeholder_taxon_optional(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::placeholder_taxon_optional(),
        Locale::Fr => fr::placeholder_taxon_optional(),
        Locale::De => de::placeholder_taxon_optional(),
        Locale::It => it::placeholder_taxon_optional(),
    }
}

pub fn placeholder_doi_optional(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::placeholder_doi_optional(),
        Locale::Fr => fr::placeholder_doi_optional(),
        Locale::De => de::placeholder_doi_optional(),
        Locale::It => it::placeholder_doi_optional(),
    }
}

pub fn button_add_row(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::button_add_row(),
        Locale::Fr => fr::button_add_row(),
        Locale::De => de::button_add_row(),
        Locale::It => it::button_add_row(),
    }
}

pub fn button_load_example_rows(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::button_load_example_rows(),
        Locale::Fr => fr::button_load_example_rows(),
        Locale::De => de::button_load_example_rows(),
        Locale::It => it::button_load_example_rows(),
    }
}

pub fn button_append_tsv_rows(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::button_append_tsv_rows(),
        Locale::Fr => fr::button_append_tsv_rows(),
        Locale::De => de::button_append_tsv_rows(),
        Locale::It => it::button_append_tsv_rows(),
    }
}

pub fn button_generate_quickstatements(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::button_generate_quickstatements(),
        Locale::Fr => fr::button_generate_quickstatements(),
        Locale::De => de::button_generate_quickstatements(),
        Locale::It => it::button_generate_quickstatements(),
    }
}

pub fn button_generating(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::button_generating(),
        Locale::Fr => fr::button_generating(),
        Locale::De => de::button_generating(),
        Locale::It => it::button_generating(),
    }
}

pub fn button_remove(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::button_remove(),
        Locale::Fr => fr::button_remove(),
        Locale::De => de::button_remove(),
        Locale::It => it::button_remove(),
    }
}

pub fn col_name(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::col_name(),
        Locale::Fr => fr::col_name(),
        Locale::De => de::col_name(),
        Locale::It => it::col_name(),
    }
}

pub fn col_action(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::col_action(),
        Locale::Fr => fr::col_action(),
        Locale::De => de::col_action(),
        Locale::It => it::col_action(),
    }
}

pub fn col_original_smiles(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::col_original_smiles(),
        Locale::Fr => fr::col_original_smiles(),
        Locale::De => de::col_original_smiles(),
        Locale::It => it::col_original_smiles(),
    }
}

pub fn col_canonical_smiles(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::col_canonical_smiles(),
        Locale::Fr => fr::col_canonical_smiles(),
        Locale::De => de::col_canonical_smiles(),
        Locale::It => it::col_canonical_smiles(),
    }
}

pub fn col_exact_mass(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::col_exact_mass(),
        Locale::Fr => fr::col_exact_mass(),
        Locale::De => de::col_exact_mass(),
        Locale::It => it::col_exact_mass(),
    }
}

pub fn col_status(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::col_status(),
        Locale::Fr => fr::col_status(),
        Locale::De => de::col_status(),
        Locale::It => it::col_status(),
    }
}

pub fn label_new_item(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::label_new_item(),
        Locale::Fr => fr::label_new_item(),
        Locale::De => de::label_new_item(),
        Locale::It => it::label_new_item(),
    }
}

pub fn hint_expected_tsv_headers(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::hint_expected_tsv_headers(),
        Locale::Fr => fr::hint_expected_tsv_headers(),
        Locale::De => de::hint_expected_tsv_headers(),
        Locale::It => it::hint_expected_tsv_headers(),
    }
}

pub fn hint_scroll_curation_results(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::hint_scroll_curation_results(),
        Locale::Fr => fr::hint_scroll_curation_results(),
        Locale::De => de::hint_scroll_curation_results(),
        Locale::It => it::hint_scroll_curation_results(),
    }
}

pub fn msg_name_smiles_required(locale: Locale) -> String {
    match locale {
        Locale::En => en::msg_name_smiles_required(),
        Locale::Fr => fr::msg_name_smiles_required(),
        Locale::De => de::msg_name_smiles_required(),
        Locale::It => it::msg_name_smiles_required(),
    }
}

pub fn msg_duplicate_row_skipped(locale: Locale) -> String {
    match locale {
        Locale::En => en::msg_duplicate_row_skipped(),
        Locale::Fr => fr::msg_duplicate_row_skipped(),
        Locale::De => de::msg_duplicate_row_skipped(),
        Locale::It => it::msg_duplicate_row_skipped(),
    }
}

pub fn msg_no_valid_tsv_rows(locale: Locale) -> String {
    match locale {
        Locale::En => en::msg_no_valid_tsv_rows(),
        Locale::Fr => fr::msg_no_valid_tsv_rows(),
        Locale::De => de::msg_no_valid_tsv_rows(),
        Locale::It => it::msg_no_valid_tsv_rows(),
    }
}

pub fn msg_tsv_import_complete(locale: Locale, added: usize, skipped: usize) -> String {
    match locale {
        Locale::En => en::msg_tsv_import_complete(added, skipped),
        Locale::Fr => fr::msg_tsv_import_complete(added, skipped),
        Locale::De => de::msg_tsv_import_complete(added, skipped),
        Locale::It => it::msg_tsv_import_complete(added, skipped),
    }
}

pub fn msg_examples_loaded(locale: Locale, added: usize, skipped: usize) -> String {
    match locale {
        Locale::En => en::msg_examples_loaded(added, skipped),
        Locale::Fr => fr::msg_examples_loaded(added, skipped),
        Locale::De => de::msg_examples_loaded(added, skipped),
        Locale::It => it::msg_examples_loaded(added, skipped),
    }
}

pub fn msg_add_row_before_generate(locale: Locale) -> String {
    match locale {
        Locale::En => en::msg_add_row_before_generate(),
        Locale::Fr => fr::msg_add_row_before_generate(),
        Locale::De => de::msg_add_row_before_generate(),
        Locale::It => it::msg_add_row_before_generate(),
    }
}

pub fn msg_running_checks(locale: Locale) -> String {
    match locale {
        Locale::En => en::msg_running_checks(),
        Locale::Fr => fr::msg_running_checks(),
        Locale::De => de::msg_running_checks(),
        Locale::It => it::msg_running_checks(),
    }
}

pub fn msg_done_review_copy(locale: Locale) -> String {
    match locale {
        Locale::En => en::msg_done_review_copy(),
        Locale::Fr => fr::msg_done_review_copy(),
        Locale::De => de::msg_done_review_copy(),
        Locale::It => it::msg_done_review_copy(),
    }
}

pub fn msg_curation_failed(locale: Locale, detail: &str) -> String {
    match locale {
        Locale::En => en::msg_curation_failed(detail),
        Locale::Fr => fr::msg_curation_failed(detail),
        Locale::De => de::msg_curation_failed(detail),
        Locale::It => it::msg_curation_failed(detail),
    }
}

pub fn msg_curation_rate_limited(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::msg_curation_rate_limited(),
        Locale::Fr => fr::msg_curation_rate_limited(),
        Locale::De => de::msg_curation_rate_limited(),
        Locale::It => it::msg_curation_rate_limited(),
    }
}

pub fn msg_prerequisites_pending(locale: Locale, count: usize) -> String {
    match locale {
        Locale::En => en::msg_prerequisites_pending(count),
        Locale::Fr => fr::msg_prerequisites_pending(count),
        Locale::De => de::msg_prerequisites_pending(count),
        Locale::It => it::msg_prerequisites_pending(count),
    }
}

pub fn msg_two_step_hint(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::msg_two_step_hint(),
        Locale::Fr => fr::msg_two_step_hint(),
        Locale::De => de::msg_two_step_hint(),
        Locale::It => it::msg_two_step_hint(),
    }
}

pub fn button_second_pass(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::button_second_pass(),
        Locale::Fr => fr::button_second_pass(),
        Locale::De => de::button_second_pass(),
        Locale::It => it::button_second_pass(),
    }
}

pub fn msg_second_pass_running(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::msg_second_pass_running(),
        Locale::Fr => fr::msg_second_pass_running(),
        Locale::De => de::msg_second_pass_running(),
        Locale::It => it::msg_second_pass_running(),
    }
}

pub fn msg_second_pass_done(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::msg_second_pass_done(),
        Locale::Fr => fr::msg_second_pass_done(),
        Locale::De => de::msg_second_pass_done(),
        Locale::It => it::msg_second_pass_done(),
    }
}

pub fn msg_second_pass_still_pending_count(locale: Locale, count: usize) -> String {
    match locale {
        Locale::En => en::msg_second_pass_still_pending_count(count),
        Locale::Fr => fr::msg_second_pass_still_pending_count(count),
        Locale::De => de::msg_second_pass_still_pending_count(count),
        Locale::It => it::msg_second_pass_still_pending_count(count),
    }
}

pub fn curation_badge_prerequisite_pending(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::curation_badge_prerequisite_pending(),
        Locale::Fr => fr::curation_badge_prerequisite_pending(),
        Locale::De => de::curation_badge_prerequisite_pending(),
        Locale::It => it::curation_badge_prerequisite_pending(),
    }
}

pub fn curation_badge_mass_missing(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::curation_badge_mass_missing(),
        Locale::Fr => fr::curation_badge_mass_missing(),
        Locale::De => de::curation_badge_mass_missing(),
        Locale::It => it::curation_badge_mass_missing(),
    }
}

pub fn curation_badge_second_pass_required(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::curation_badge_second_pass_required(),
        Locale::Fr => fr::curation_badge_second_pass_required(),
        Locale::De => de::curation_badge_second_pass_required(),
        Locale::It => it::curation_badge_second_pass_required(),
    }
}

pub fn curation_mass_warning_title(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::curation_mass_warning_title(),
        Locale::Fr => fr::curation_mass_warning_title(),
        Locale::De => de::curation_mass_warning_title(),
        Locale::It => it::curation_mass_warning_title(),
    }
}

pub fn msg_delay_advice(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::msg_delay_advice(),
        Locale::Fr => fr::msg_delay_advice(),
        Locale::De => de::msg_delay_advice(),
        Locale::It => it::msg_delay_advice(),
    }
}

pub fn curation_qs_dev_label(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::curation_qs_dev_label(),
        Locale::Fr => fr::curation_qs_dev_label(),
        Locale::De => de::curation_qs_dev_label(),
        Locale::It => it::curation_qs_dev_label(),
    }
}

pub fn curation_qs_dev_prereq_hint(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::curation_qs_dev_prereq_hint(),
        Locale::Fr => fr::curation_qs_dev_prereq_hint(),
        Locale::De => de::curation_qs_dev_prereq_hint(),
        Locale::It => it::curation_qs_dev_prereq_hint(),
    }
}

pub fn curation_qs_dev_main_hint(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::curation_qs_dev_main_hint(),
        Locale::Fr => fr::curation_qs_dev_main_hint(),
        Locale::De => de::curation_qs_dev_main_hint(),
        Locale::It => it::curation_qs_dev_main_hint(),
    }
}

pub fn curation_note_existing_complete(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::curation_note_existing_complete(),
        Locale::Fr => fr::curation_note_existing_complete(),
        Locale::De => de::curation_note_existing_complete(),
        Locale::It => it::curation_note_existing_complete(),
    }
}

pub fn curation_note_existing_updates(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::curation_note_existing_updates(),
        Locale::Fr => fr::curation_note_existing_updates(),
        Locale::De => de::curation_note_existing_updates(),
        Locale::It => it::curation_note_existing_updates(),
    }
}

pub fn curation_note_new_compound(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::curation_note_new_compound(),
        Locale::Fr => fr::curation_note_new_compound(),
        Locale::De => de::curation_note_new_compound(),
        Locale::It => it::curation_note_new_compound(),
    }
}

pub fn curation_note_dependencies_pending(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::curation_note_dependencies_pending(),
        Locale::Fr => fr::curation_note_dependencies_pending(),
        Locale::De => de::curation_note_dependencies_pending(),
        Locale::It => it::curation_note_dependencies_pending(),
    }
}

pub fn curation_pending_taxon(locale: Locale, taxon: &str) -> String {
    match locale {
        Locale::En => en::curation_pending_taxon(taxon),
        Locale::Fr => fr::curation_pending_taxon(taxon),
        Locale::De => de::curation_pending_taxon(taxon),
        Locale::It => it::curation_pending_taxon(taxon),
    }
}

pub fn curation_pending_reference(locale: Locale, doi: &str) -> String {
    match locale {
        Locale::En => en::curation_pending_reference(doi),
        Locale::Fr => fr::curation_pending_reference(doi),
        Locale::De => de::curation_pending_reference(doi),
        Locale::It => it::curation_pending_reference(doi),
    }
}

pub fn view_switch_aria(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::view_switch_aria(),
        Locale::Fr => fr::view_switch_aria(),
        Locale::De => de::view_switch_aria(),
        Locale::It => it::view_switch_aria(),
    }
}

pub fn view_label_explorer(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::view_label_explorer(),
        Locale::Fr => fr::view_label_explorer(),
        Locale::De => de::view_label_explorer(),
        Locale::It => it::view_label_explorer(),
    }
}

pub fn view_label_curation_explorer(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::view_label_curation_explorer(),
        Locale::Fr => fr::view_label_curation_explorer(),
        Locale::De => de::view_label_curation_explorer(),
        Locale::It => it::view_label_curation_explorer(),
    }
}

pub fn view_label_draw(locale: Locale) -> &'static str {
    match locale {
        Locale::En => en::view_label_draw(),
        Locale::Fr => fr::view_label_draw(),
        Locale::De => de::view_label_draw(),
        Locale::It => it::view_label_draw(),
    }
}

pub fn curation_status_label(locale: Locale, status_key: &str) -> &'static str {
    match locale {
        Locale::En => en::curation_status_label(status_key),
        Locale::Fr => fr::curation_status_label(status_key),
        Locale::De => de::curation_status_label(status_key),
        Locale::It => it::curation_status_label(status_key),
    }
}
