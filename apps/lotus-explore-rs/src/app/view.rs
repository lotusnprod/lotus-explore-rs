// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppView {
    Explore,
    Curation,
    Draw,
}

impl AppView {
    pub fn from_query_value(value: Option<&str>) -> Self {
        match value {
            Some("curation" | "curation-explorer") => Self::Curation,
            Some("draw") => Self::Draw,
            _ => Self::Explore,
        }
    }

    pub const fn query_value(self) -> Option<&'static str> {
        match self {
            Self::Explore => None,
            Self::Curation => Some("curation-explorer"),
            Self::Draw => Some("draw"),
        }
    }
}
