// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Page header: title, language switcher, view switcher, subtitle, archive note.
//!
//! Zero props -- all data comes from context (`use_locale`, `AppStateContext`).

use crate::app::routes::Route;
use crate::components::layout::dark_mode_toggle::DarkModeToggle;
use crate::components::layout::lang_switch::LangSwitch;
use crate::components::layout::view_switch::ViewSwitch;
use crate::hooks::use_locale;
use crate::i18n::{TextKey, t};
use crate::ui::a11y_contract::PAGE_TITLE_ID;
use dioxus::prelude::*;

const LOTUS_LOGO_SVG: &str = include_str!("../../../public/favicon.svg");

/// Full page header section.
///
/// Composes `LangSwitch` (EN/FR/DE/IT), `DarkModeToggle` (light/dark), and
/// `ViewSwitch` (Search / Curation / Structure editor) as context-aware
/// children. Zero props -- only re-renders when locale or route changes.
#[component]
pub fn PageHeader() -> Element {
    let locale = use_locale();
    let route: Route = use_route();
    let home = route.with_view("explore");

    rsx! {
        header {
            class: "sticky top-0 z-3 min-h-[46px] bg-shell-chrome border-b border-shell-border rounded-t-xl px-4 sm:px-8",
            div {
                class: "flex flex-wrap items-start justify-between gap-3 sm:gap-4",
                div {
                    class: "w-16 shrink-0",
                    aria_hidden: "true",
                    dangerous_inner_html: LOTUS_LOGO_SVG,
                }
                div {
                    class: "min-w-0 max-w-full",
                h1 { id: PAGE_TITLE_ID,
                    class: "text-display font-bold min-w-0 break-words overflow-hidden",
                    Link {
                         to: home.navigation_string(),

                        class: "break-words text-text no-underline hover:no-underline",
                        "{t(locale, TextKey::PageTitle)}"
                    }
                }
                }
                div {
                    class: "flex flex-wrap items-center gap-2 min-w-0 max-w-full",
                    ViewSwitch {}
                    LangSwitch {}
                    DarkModeToggle {}
                }
            }
            p {
                class: "max-w-[72ch] text-sm leading-6 text-critical-muted mt-3 pb-2 break-words",
                "{t(locale, TextKey::PageSubtitle)}"
            }
        }
    }
}
