// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(clippy::fn_params_excessive_bools)]

//! Common UI utilities and phase models.

use dioxus::prelude::*;

/// High-level lifecycle phase for the results area viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentPhase {
    Welcome,
    Loading,
    Error,
    DownloadOnly,
    Empty,
    Loaded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LifecycleBooleans {
    pub loading: bool,
    pub has_error: bool,
    pub searched_once: bool,
    pub download_only_mode: bool,
    pub has_entries: bool,
}

impl ContentPhase {
    #[allow(clippy::too_many_arguments)]
    pub fn from_lifecycle(
        loading: bool,
        has_error: bool,
        searched_once: bool,
        download_only_mode: bool,
        has_entries: bool,
    ) -> Self {
        Self::from(LifecycleBooleans {
            loading,
            has_error,
            searched_once,
            download_only_mode,
            has_entries,
        })
    }
}

impl From<LifecycleBooleans> for ContentPhase {
    fn from(state: LifecycleBooleans) -> Self {
        let LifecycleBooleans {
            loading,
            has_error,
            searched_once,
            download_only_mode,
            has_entries,
        } = state;
        if loading {
            Self::Loading
        } else if has_error {
            Self::Error
        } else if download_only_mode {
            Self::DownloadOnly
        } else if !searched_once {
            Self::Welcome
        } else if !has_entries {
            Self::Empty
        } else {
            Self::Loaded
        }
    }
}

/// Default skip link style for keyboard navigation to main content.
pub const SKIP_LINK_STYLE: &str = "position:absolute;top:-100%;left:0.5rem;z-index:9999;padding:0.5rem 1rem;background:transparent;color:#0b5cab;font-size:0.875rem;font-weight:600;border-radius:0 0 4px 4px;text-decoration:underline;";

/// Skip navigation link for keyboard accessibility.
#[component]
pub fn skip_link() -> Element {
    rsx! {
        a {
            href: "#main-content",
            class: "skip-link",
            style: SKIP_LINK_STYLE,
            "Skip to main content"
        }
    }
}

/// Alternative skip link component for apps using `id="main"`.
#[component]
pub fn skip_link_main() -> Element {
    rsx! {
        a {
            href: "#main",
            class: "skip-link",
            style: SKIP_LINK_STYLE,
            "Skip to main content"
        }
    }
}
