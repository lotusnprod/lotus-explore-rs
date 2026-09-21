// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Card component for grouped content.

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct CardProps {
    #[props(default)]
    pub children: Element,
    #[props(default)]
    pub href: Option<String>,
    #[props(default = "")]
    pub class: &'static str,
}

#[component]
pub fn Card(props: CardProps) -> Element {
    let custom_class = props.class;

    if let Some(href) = &props.href {
        rsx! {
            article {
                class: "flex flex-col gap-4 rounded-xl border border-panel-border bg-panel-soft p-4 shadow-xs transition-shadow duration-200 hover:shadow-md {custom_class}",
                a {
                    href: href,
                    class: "block",
                    {props.children}
                }
            }
        }
    } else {
        rsx! {
            article {
                class: "flex flex-col gap-4 rounded-xl border border-panel-border bg-panel-soft p-4 shadow-xs {custom_class}",
                {props.children}
            }
        }
    }
}
