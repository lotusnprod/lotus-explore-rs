// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! View-switcher nav component (Search / Curation / Structure editor).
//!
//! Reads `AppStateContext` for the current view and `use_locale()` for labels —
//! zero props required.

use crate::app::view::AppView;
use crate::hooks::use_locale;
use crate::i18n::{
    view_label_curation_explorer, view_label_draw, view_label_explorer, view_switch_aria,
};
use crate::state::{use_app_selector, use_app_state_context};
use crate::ui::prelude::{SegmentedControl, SegmentedControlItem};
use dioxus::prelude::*;

/// Three-button view switcher.
#[component]
pub fn ViewSwitch() -> Element {
    let ctx = use_app_state_context();
    let locale = use_locale();
    let mut app_state = ctx.state;
    let current_view = *use_app_selector(app_state, |state| state.view).read();
    let dark_mode = app_state.read().dark_mode;

    rsx! {
        nav { class: "view-switch flex flex-wrap items-center rounded-full overflow-hidden border border-border bg-surface shadow-xs", aria_label: "{view_switch_aria(locale)}",
            SegmentedControl {
                aria_label: view_switch_aria(locale),
                selected_value: view_key(current_view),
                dark: dark_mode,
                wrap: true,
                active_aria_current: "page",
                items: vec![
                    SegmentedControlItem {
                        label: view_label_explorer(locale),
                        value: "explore",
                    },
                    SegmentedControlItem {
                        label: view_label_curation_explorer(locale),
                        value: "curation",
                    },
                    SegmentedControlItem {
                        label: view_label_draw(locale),
                        value: "draw",
                    },
                ],
                on_select: move |value: String| {
                    let next = match value.as_str() {
                        "curation" => AppView::Curation,
                        "draw" => AppView::Draw,
                        _ => AppView::Explore,
                    };
                    app_state.with_mut(|s| s.view = next);
                },
            }
        }
    }
}

fn view_key(view: AppView) -> &'static str {
    match view {
        AppView::Explore => "explore",
        AppView::Curation => "curation",
        AppView::Draw => "draw",
    }
}
