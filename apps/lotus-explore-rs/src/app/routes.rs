use super::shell::{AppShell, ExplorePage};
use crate::components::data_curation_page::DataCurationPage;
use crate::i18n::Locale;
use crate::pages::DrawPage;
use dioxus::prelude::*;
use dioxus::router::routable::FromQuery;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RouteQuery {
    params: BTreeMap<String, String>,
}

impl RouteQuery {
    pub fn from_encoded(query: &str) -> Self {
        Self {
            params: parse_encoded_query(query),
        }
    }

    fn from_decoded(query: &str) -> Self {
        Self {
            params: parse_decoded_query(query),
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(String::as_str)
    }

    fn set(&mut self, key: &str, value: &str) {
        self.params.insert(key.to_string(), value.to_string());
    }

    fn remove(&mut self, key: &str) {
        self.params.remove(key);
    }
}

impl FromQuery for RouteQuery {
    fn from_query(query: &str) -> Self {
        Self::from_decoded(query)
    }
}

impl fmt::Display for RouteQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&encode_query(&self.params))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Routable)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AppShell)]
    #[route("/?:..query#:hash")]
    Explore { query: RouteQuery, hash: String },

    #[route("/curation?:..query#:hash")]
    Curation { query: RouteQuery, hash: String },

    #[route("/draw?:..query#:hash")]
    Draw { query: RouteQuery, hash: String },
}

impl Route {
    pub fn query_string(&self) -> String {
        self.query_value().to_string()
    }

    pub(crate) fn query_value(&self) -> RouteQuery {
        match self {
            Self::Explore { query, .. }
            | Self::Curation { query, .. }
            | Self::Draw { query, .. } => query.clone(),
        }
    }

    pub fn hash(&self) -> &str {
        match self {
            Self::Explore { hash, .. } | Self::Curation { hash, .. } | Self::Draw { hash, .. } => {
                hash
            }
        }
    }

    pub fn view_key(&self) -> &'static str {
        match self {
            Self::Explore { .. } => "explore",
            Self::Curation { .. } => "curation",
            Self::Draw { .. } => "draw",
        }
    }

    fn page_query(&self) -> RouteQuery {
        let current = self.query_value();
        let mut query = RouteQuery::default();
        for key in ["lang", "dark_mode", "api_base"] {
            if let Some(value) = current.get(key) {
                query.set(key, value);
            }
        }
        query
    }

    pub fn with_view(self, view: &str) -> Self {
        let query = self.page_query();
        let hash = String::new();
        match view {
            "curation" => Self::Curation { query, hash },
            "draw" => Self::Draw { query, hash },
            _ => Self::Explore { query, hash },
        }
    }

    pub fn with_locale(self, locale: Locale) -> Self {
        let mut query = self.query_value();
        if locale == Locale::En {
            query.remove("lang");
        } else {
            query.set("lang", locale.lang_code());
        }
        self.with_query_and_hash(query)
    }

    pub fn with_dark_mode(self, dark_mode: bool) -> Self {
        let mut query = self.query_value();
        if dark_mode {
            query.set("dark_mode", "true");
        } else {
            query.remove("dark_mode");
        }
        self.with_query_and_hash(query)
    }

    fn with_query_and_hash(self, query: RouteQuery) -> Self {
        let hash = self.hash().to_string();
        match self {
            Self::Explore { .. } => Self::Explore { query, hash },
            Self::Curation { .. } => Self::Curation { query, hash },
            Self::Draw { .. } => Self::Draw { query, hash },
        }
    }
}

#[component]
pub fn Explore(query: RouteQuery, hash: String) -> Element {
    let _ = (query, hash);
    rsx! { ExplorePage {} }
}

#[component]
pub fn Curation(query: RouteQuery, hash: String) -> Element {
    let _ = (query, hash);
    rsx! { DataCurationPage {} }
}

#[component]
pub fn Draw(query: RouteQuery, hash: String) -> Element {
    let _ = (query, hash);
    rsx! { DrawPage {} }
}

fn parse_encoded_query(query: &str) -> BTreeMap<String, String> {
    query
        .trim_start_matches('?')
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            (decode_component(key), decode_component(value))
        })
        .collect()
}

fn parse_decoded_query(query: &str) -> BTreeMap<String, String> {
    split_decoded_query(query)
        .into_iter()
        .filter_map(|pair| {
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            if key.is_empty() {
                return None;
            }
            Some((key.to_string(), value.to_string()))
        })
        .collect()
}

fn split_decoded_query(query: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (index, ch) in query.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '[' | '{' | '(' => depth = depth.saturating_add(1),
            ']' | '}' | ')' => depth = depth.saturating_sub(1),
            '&' if depth == 0 => {
                parts.push(&query[start..index]);
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(&query[start..]);
    parts
}

fn decode_component(value: &str) -> String {
    urlencoding::decode(value).map_or_else(|_| value.to_string(), std::borrow::Cow::into_owned)
}

fn encode_query(params: &BTreeMap<String, String>) -> String {
    params
        .iter()
        .map(|(key, value)| {
            format!(
                "{}={}",
                urlencoding::encode(key),
                urlencoding::encode(value)
            )
        })
        .collect::<Vec<_>>()
        .join("&")
}

#[cfg(test)]
mod tests {
    use super::{Route, RouteQuery};
    use crate::i18n::Locale;

    #[test]
    fn routes_round_trip_query_and_hash_segments() {
        let parsed = "/curation?lang=fr&dark_mode=true#main-panel".parse::<Route>();
        assert!(parsed.is_ok(), "route should parse");
        if let Ok(route) = parsed {
            assert_eq!(route.view_key(), "curation");
            assert_eq!(route.query_string(), "dark_mode=true&lang=fr");
            assert_eq!(route.hash(), "main-panel");
            assert_eq!(
                route.to_string(),
                "/curation?dark_mode=true&lang=fr#main-panel"
            );
        }
    }

    #[test]
    fn encoded_ampersands_in_json_query_are_preserved() {
        let route = Route::Curation {
            query: RouteQuery::from_encoded("curation_rows=%5B%7B%22name%22%3A%22A%26B%22%7D%5D"),
            hash: String::new(),
        };
        assert_eq!(
            route.to_string(),
            "/curation?curation_rows=%5B%7B%22name%22%3A%22A%26B%22%7D%5D"
        );
    }

    #[test]
    fn parsed_json_query_preserves_encoded_ampersands() {
        let parsed =
            "/curation?curation_rows=%5B%7B%22name%22%3A%22A%26B%22%7D%5D".parse::<Route>();
        assert!(parsed.is_ok(), "route should parse");
        if let Ok(route) = parsed {
            assert_eq!(
                route.query_string(),
                "curation_rows=%5B%7B%22name%22%3A%22A%26B%22%7D%5D"
            );
        }
    }

    #[test]
    fn changing_view_resets_page_query_and_hash() {
        let route = Route::Explore {
            query: RouteQuery::from_encoded("taxon=Rosa&lang=fr"),
            hash: "results".into(),
        };
        let route = route.with_view("draw");
        assert_eq!(route.view_key(), "draw");
        assert_eq!(route.query_string(), "lang=fr");
        assert_eq!(route.hash(), "");
    }

    #[test]
    fn preference_updates_preserve_route_metadata() {
        let route = Route::Draw {
            query: RouteQuery::from_encoded("api_base=https%3A%2F%2Fexample.org"),
            hash: "main-panel".into(),
        }
        .with_locale(Locale::Fr)
        .with_dark_mode(true);
        assert_eq!(
            route.query_string(),
            "api_base=https%3A%2F%2Fexample.org&dark_mode=true&lang=fr"
        );
        assert_eq!(route.hash(), "main-panel");
    }
}
