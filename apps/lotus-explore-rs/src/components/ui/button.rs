// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Shared Button component using inline Tailwind classes.

use dioxus::prelude::*;

/// Props for the Button component.
#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    #[props(default)]
    pub label: Option<String>,
    #[props(default = "button")]
    pub r#type: &'static str,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub loading: bool,
    #[props(default)]
    pub title: Option<String>,
    #[props(default)]
    pub aria_label: Option<String>,
    #[props(default)]
    pub aria_controls: Option<String>,
    #[props(default)]
    pub aria_expanded: Option<String>,
    #[props(default)]
    pub aria_pressed: Option<String>,
    #[props(into, default)]
    pub class: Option<String>,
    #[props(default)]
    pub onclick: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn Button(props: ButtonProps) -> Element {
    let default_class = "inline-flex items-center justify-center font-sans select-none transition-transform duration-150 ease-[cubic-bezier(.4,0,.2,1)] focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-accent/28 focus-visible:ring-offset-2 rounded-xl bg-accent text-bg font-semibold shadow-xs hover:bg-accent-2 active:bg-accent-2 min-h-[40px] gap-2 px-3.5 py-2 text-ui active:scale-[0.98] cursor-pointer";
    let class = props.class.as_deref().unwrap_or(default_class);

    rsx! {
        button {
            r#type: props.r#type,
            disabled: props.disabled || props.loading,
            title: props.title.as_deref().unwrap_or_default(),
            aria_label: props.aria_label.as_deref().unwrap_or_default(),
            aria_controls: props.aria_controls.as_deref().unwrap_or_default(),
            aria_expanded: props.aria_expanded.as_deref().unwrap_or_default(),
            aria_pressed: props.aria_pressed.as_deref().unwrap_or_default(),
            class: class,
            onclick: move |evt| {
                if !props.disabled && !props.loading
                    && let Some(handler) = props.onclick.as_ref() {
                        handler.call(evt);
                    }
            },
            if props.loading {
                span {
                    class: "inline-block size-3.5 rounded-full border-2 border-current border-t-transparent animate-spin",
                    "aria-hidden": "true",
                }
            }
            if let Some(ref text) = props.label {
                span { "{text}" }
            }
            {props.children}
        }
    }
}
