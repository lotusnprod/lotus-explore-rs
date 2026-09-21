// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Language-switcher button group.
//!
//! Reads and writes the `Signal<Locale>` from `LocaleProvider` context via
//! [`use_locale_signal`] — zero props required.

use crate::hooks::{use_locale, use_locale_signal};
use crate::i18n::{Locale, TextKey, t};
use crate::state::use_app_state_context;
use crate::ui::prelude::{SegmentedControl, SegmentedControlItem};
use dioxus::prelude::*;

/// Four-button language switcher (EN / FR / DE / IT).
#[component]
pub fn LangSwitch() -> Element {
    let mut locale_sig = use_locale_signal();
    let locale = use_locale();
    let dark_mode = use_app_state_context().state.read().dark_mode;

    rsx! {
        nav {
            class: "lang-switch inline-flex items-center rounded-full overflow-hidden border border-border bg-surface shadow-xs",
            aria_label: t(locale, TextKey::Language).to_string(),
            SegmentedControl {
                aria_label: t(locale, TextKey::Language).to_string(),
                selected_value: locale_code(locale).to_string(),
                dark: dark_mode,
                wrap: false,
                items: vec![
                    SegmentedControlItem { label: "EN".to_string(), value: "en".to_string() },
                    SegmentedControlItem { label: "FR".to_string(), value: "fr".to_string() },
                    SegmentedControlItem { label: "DE".to_string(), value: "de".to_string() },
                    SegmentedControlItem { label: "IT".to_string(), value: "it".to_string() },
                ],
                on_select: move |value: String| {
                    let next = match value.as_str() {
                        "en" => Locale::En,
                        "fr" => Locale::Fr,
                        "de" => Locale::De,
                        "it" => Locale::It,
                        _ => Locale::En,
                    };
                    if *locale_sig.peek() != next {
                        *locale_sig.write() = next;
                    }
                },
            }
        }
    }
}

fn locale_code(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "en",
        Locale::Fr => "fr",
        Locale::De => "de",
        Locale::It => "it",
    }
}
