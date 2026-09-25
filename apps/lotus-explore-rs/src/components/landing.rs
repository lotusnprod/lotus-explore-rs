// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

use crate::features::explore::url_state::href_with_current_query;
use crate::hooks::use_locale;
use crate::i18n::{TextKey, t};
use dioxus::prelude::*;

#[component]
pub fn LandingPage() -> Element {
    let locale = use_locale();
    let search_href = href_with_current_query("/search");
    rsx! {
        section {
            class: "page-section w-full max-w-none px-4 sm:px-6 lg:px-8",
            aria_labelledby: "landing-welcome-heading",
            div {
                class: "mx-auto min-w-0 w-full max-w-6xl",
                div {
                    class: "min-w-0 w-full max-w-full overflow-hidden rounded-xl border border-shell-border bg-shell-raised p-6 shadow-xs sm:p-8",
                h2 {
                    id: "landing-welcome-heading",
                    class: "text-title font-semibold text-text",
                    "{t(locale, TextKey::LandingTitle)}"
                }
                p {
                    class: "mt-3 w-full min-w-0 break-words text-body leading-relaxed text-muted",
                    "{t(locale, TextKey::WelcomeLeadA)}"
                    "{t(locale, TextKey::WelcomeLeadB)}"
                    a {
                        href: "https://www.wikidata.org/wiki/Q104225190",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "mx-1 font-medium text-accent hover:underline",
                        "LOTUS initiative"
                    }
                    "{t(locale, TextKey::WelcomeLeadC)}"
                    a {
                        href: "https://www.wikidata.org/",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "mx-1 font-medium text-accent hover:underline",
                        "Wikidata"
                    }
                    "{t(locale, TextKey::WelcomeLeadD)}"
                    a {
                        href: "https://qlever.dev/wikidata",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "mx-1 font-medium text-accent hover:underline",
                        "QLever"
                    }
                    "{t(locale, TextKey::WelcomeLeadE)}"
                    " "
                    span {
                        class: "text-ui italic text-subtle",
                        "{t(locale, TextKey::LabelLanguagePolicy)}"
                    }
                }
                }
                div {
                    class: "mt-6 flex justify-center",
                    a {
                        href: "{search_href}",
                        class: "inline-flex min-h-[40px] items-center justify-center gap-2 rounded-xl bg-accent px-3.5 py-2 text-ui font-semibold text-bg no-underline shadow-xs transition-transform duration-150 ease-[cubic-bezier(.4,0,.2,1)] hover:bg-accent-2 active:bg-accent-2 active:scale-[0.98] focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-accent/28 focus-visible:ring-offset-2 cursor-pointer",
                        "{t(locale, TextKey::OpenSearch)}"
                    }
                }
            }
        }
    }
}

#[component]
pub fn NotFoundPage() -> Element {
    let locale = use_locale();
    let home_href = href_with_current_query("/");
    rsx! {
        section {
            class: "page-section w-full max-w-none px-4 sm:px-6 lg:px-8",
            aria_labelledby: "not-found-heading",
            div {
                class: "w-full max-w-3xl rounded-xl border border-shell-border bg-shell-raised p-6 shadow-xs sm:p-8",
                h2 {
                    id: "not-found-heading",
                    class: "text-title font-semibold text-text",
                    "{t(locale, TextKey::PageNotFound)}"
                }
                p {
                    class: "mt-3 text-body leading-relaxed text-muted",
                    "{t(locale, TextKey::PageNotFoundDescription)}"
                }
                a {
                    href: "{home_href}",
                    class: "inline-flex min-h-[40px] items-center justify-center gap-2 rounded-xl bg-accent px-3.5 py-2 text-ui font-semibold text-bg no-underline shadow-xs transition-transform duration-150 ease-[cubic-bezier(.4,0,.2,1)] hover:bg-accent-2 active:bg-accent-2 active:scale-[0.98] focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-accent/28 focus-visible:ring-offset-2 cursor-pointer",
                    "{t(locale, TextKey::ReturnHome)}"
                }
            }
        }
    }
}
