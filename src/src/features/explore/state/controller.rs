// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

#[cfg(test)]
use crate::app::view::AppView;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg(test)]
pub struct AppLayoutClasses {
    pub app_layout: &'static str,
    pub main: &'static str,
}

#[cfg(test)]
pub fn classes_for_view(view: AppView) -> AppLayoutClasses {
    if view == AppView::Explore {
        AppLayoutClasses {
            app_layout: "app-layout",
            main: "main-content",
        }
    } else {
        AppLayoutClasses {
            app_layout: "app-layout single-pane",
            main: "main-content single-pane",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes_for_non_explore_view_uses_single_pane_layout() {
        let classes = classes_for_view(AppView::Curation);
        assert_eq!(classes.app_layout, "app-layout single-pane");
        assert_eq!(classes.main, "main-content single-pane");
    }
}
