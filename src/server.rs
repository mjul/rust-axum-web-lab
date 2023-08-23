//! HTTP-based application server with routes
use std::collections::HashMap;
use std::fmt;
use std::net::SocketAddr;
use std::str::FromStr;

use askama::Template;
use axum::extract::{Path, Query, State};
use axum::handler::Handler;
use axum::{routing::get, Router};
use axum_macros::debug_handler;
use serde::{de, Deserialize, Deserializer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::{instrument, Instrument};

#[derive(Template)] // this will generate the code...
#[template(path = "index.html")] // using the template in this path, relative to the `templates` dir in the crate root
struct IndexTemplate<'a> {
    name: &'a str, // the field name should match the variable name in the template
}

// Note how the IndexTemplate implements IntoResponse for Axum, so we can return it directly from a handler:
async fn index() -> IndexTemplate<'static> {
    IndexTemplate { name: "world" }
}

#[derive(Clone)]
struct Language {
    name: &'static str,
    year: u32,
}

const LANGUAGES: [Language; 6] = [
    Language {
        name: "FORTRAN",
        year: 1954,
    },
    Language {
        name: "LISP",
        year: 1958,
    },
    Language {
        name: "COBOL",
        year: 1959,
    },
    Language {
        name: "ALGOL 60",
        year: 1960,
    },
    Language {
        name: "Prolog",
        year: 1972,
    },
    Language {
        name: "ML",
        year: 1973,
    },
];

// This is an example of a template using template inheritance for consistency
// It also shows how to use loops in the template
#[derive(Template)]
#[template(path = "languages/index.html")]
struct LanguagesTemplate {
    headline: String,
    languages: Vec<Language>, // the field name should match the variable name in the template
}

async fn languages() -> LanguagesTemplate {
    LanguagesTemplate {
        headline: "Languages".to_string(),
        languages: LANGUAGES.into(),
    }
}

// Use debug_handler to get better error messages in case the handler is not correctly defined
#[debug_handler]
// Path is an Axum Extract to get the matched value from the path (see below in the route configuration)
async fn languages_from_year_path(Path(year): Path<u32>) -> LanguagesTemplate {
    let matches = LANGUAGES
        .iter()
        .filter(|l| l.year == year)
        .map(|l| l.clone())
        .collect();
    let headline = format!("Languages from {}", year);
    LanguagesTemplate {
        headline,
        languages: matches,
    }
}

async fn languages_from_year_query(
    Query(params): Query<HashMap<String, String>>,
) -> LanguagesTemplate {
    // No error handling since this fn is a demonstration of Query extraction
    let year = params
        .get("year")
        .expect("expected query parameter years ")
        .parse::<u32>()
        .expect("expected a valid number for year");
    let matches = LANGUAGES
        .iter()
        .filter(|l| l.year == year)
        .map(|l| l.clone())
        .collect();
    let headline = format!("Languages from {}", year);
    LanguagesTemplate {
        headline,
        languages: matches,
    }
}

/// Axum can use `serde` to deserialize the query parameters into a struct
#[derive(Deserialize)]
pub(crate) struct LanguagesFilter {
    year_from_inclusive: Option<u32>,
    year_to_exclusive: Option<u32>,
}

impl LanguagesFilter {
    /// Check if a language is accepted through the filter
    fn accepts(&self, language: &Language) -> bool {
        let year = language.year;
        let matches_from = self.year_from_inclusive.map_or(true, |from| from <= year);
        let matches_to = self.year_to_exclusive.map_or(true, |to| year < to);
        matches_from && matches_to
    }
}

/// We can define handlers with a typed struct instead of the raw query parameters
async fn languages_by_struct_query(filter: Query<LanguagesFilter>) -> LanguagesTemplate {
    // No error handling since this fn is a demonstration of Query extraction
    let matches = LANGUAGES
        .into_iter()
        .filter(|l| filter.accepts(l))
        .map(|l| l.clone())
        .collect();
    let headline = match (&filter.year_from_inclusive, &filter.year_to_exclusive) {
        (Some(from), Some(to)) => {
            format!(
                "Languages from year {} (inclusive) to {} (exclusive)",
                from, to
            )
        }
        (Some(from), None) => {
            format!("Languages from year {} and onwards", from)
        }
        (None, Some(to)) => {
            format!("Languages before year {}", to)
        }
        (None, None) => {
            format!("Languages from any year")
        }
    };
    LanguagesTemplate {
        headline,
        languages: matches,
    }
}

#[derive(Clone)]
struct AppState {
    old_languages: Vec<Language>,
    new_languages: Vec<Language>,
}

impl AppState {
    pub fn new() -> Self {
        let (old_languages, new_languages) =
            LANGUAGES.into_iter().clone().partition(|l| l.year < 1970);
        Self {
            old_languages,
            new_languages,
        }
    }
}

// To access the state in the we need to use the `State` extractor like this.
// Note the `#[debug_handler]` macro - it makes the compiler errors more readable
// in case the handler is not correctly defined.
#[debug_handler]
async fn stateful_old_languages(State(app_state): State<AppState>) -> LanguagesTemplate {
    LanguagesTemplate {
        headline: "Old Languages".to_string(),
        languages: app_state.old_languages.clone(),
    }
}

async fn stateful_new_languages(State(app_state): State<AppState>) -> LanguagesTemplate {
    LanguagesTemplate {
        headline: "New Languages".to_string(),
        languages: app_state.new_languages.clone(),
    }
}

fn stateful_router() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/old", get(stateful_old_languages))
        .route("/new", get(stateful_new_languages))
}

/// Get the application router
// When combining routers, it is simpler to use a generic return type (see `.nest` below)
pub(crate) fn router<T>() -> Router<T>
where
    T: Clone + Send + Sync + 'static,
{
    Router::new()
        // Route the root to the index fn above
        .route("/", get(index))
        // We can serve a single file like this:
        //.route_service("/assets/foo.html", ServeFile::new("assets/foo.html"))
        // Or we can serve a whole directory like this:
        // note that we use .nest_service since it nests a lot our routes,
        // .route_service would route on the root path to the service only
        .nest_service("/assets", ServeDir::new("assets"))
        // Route the /languages path to the languages fn above
        // This is an example of a using templates with inheritance
        .route("/languages/", get(languages))
        // We can capture a part of the path as a parameter and pass it to the handler
        .route("/languages/years/:year", get(languages_from_year_path))
        // We can also capture the query parameters and get the year from the query string:
        .route("/languages/year", get(languages_from_year_query))
        // We can also use a struct to capture the query parameters
        .route("/languages/filter", get(languages_by_struct_query))
        // We can have state in the application and pass it to the handlers
        // This changes the signature of the Router and the handler functions
        .nest("/stateful/", stateful_router())
        .with_state(AppState::new())
        // Add tracing to the router (i.e. trace all of the above)
        .layer(TraceLayer::new_for_http())
}

/// Start the server
// #[instrument] adds tracing to the function
#[instrument]
pub(crate) async fn serve(socket_addr: &SocketAddr) {
    tracing::info!("Starting server at http://{}", socket_addr);
    let app = router();
    axum::Server::bind(socket_addr)
        .serve(app.into_make_service())
        .await
        .unwrap()
}

/// Axum can use `serde` to deserialize the query parameters into a struct
/// If we want to accept empty query string parameters as `None` we need to decorate the struct
/// with `#[serde(deserialize_with = "empty_string_as_none")]` to use a bespoke
/// deserializer that accepts empty strings as `None`
/// Note that we do not use this in the application, but see the tests below.
#[derive(Deserialize)]
pub(crate) struct LanguagesFilterThatAcceptsEmptyQueryParameterValuesAsNone {
    #[serde(deserialize_with = "empty_string_as_none")]
    year_from_inclusive: Option<u32>,
    #[serde(deserialize_with = "empty_string_as_none")]
    year_to_exclusive: Option<u32>,
}

// Taken from documentation:
// https://github.com/tokio-rs/axum/blob/main/examples/query-params-with-empty-strings/src/main.rs
fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}

#[cfg(test)]
mod tests {
    use axum::http::Uri;

    use super::*;

    #[test]
    fn languages_filter_can_deserialize_when_all_query_string_params_are_present_and_valid() {
        let uri = Uri::builder()
            .path_and_query("/?year_from_inclusive=1950&year_to_exclusive=1970")
            .build()
            .unwrap();
        let q = Query::<LanguagesFilter>::try_from_uri(&uri).unwrap();
        assert_eq!(Some(1950), q.year_from_inclusive);
        assert_eq!(Some(1970), q.year_to_exclusive);
    }

    #[test]
    fn languages_filter_can_deserialize_when_all_query_string_params_are_missing() {
        let uri = Uri::builder().path_and_query("/?").build().unwrap();
        let q = Query::<LanguagesFilter>::try_from_uri(&uri).unwrap();
        assert_eq!(None, q.year_from_inclusive);
        assert_eq!(None, q.year_to_exclusive);
    }

    // Serde normally requires that all query strings parameters are valid when present (not present with empty value)
    #[test]
    fn languages_filter_cannot_deserialize_when_all_query_string_params_are_present_but_empty() {
        let uri = Uri::builder()
            .path_and_query("/?year_from_inclusive=&year_to_exclusive=")
            .build()
            .unwrap();
        let actual = Query::<LanguagesFilter>::try_from_uri(&uri);
        assert!(actual.is_err());
    }

    // The modified struct with the `#[serde(deserialize_with = "empty_string_as_none")]` attribute can
    // deserialize in this case, too
    #[test]
    fn languages_filter_with_decoration_can_deserialize_when_all_query_string_params_are_present_but_empty(
    ) {
        let uri = Uri::builder()
            .path_and_query("/?year_from_inclusive=&year_to_exclusive=")
            .build()
            .unwrap();
        let q =
            Query::<LanguagesFilterThatAcceptsEmptyQueryParameterValuesAsNone>::try_from_uri(&uri)
                .unwrap();
        assert_eq!(None, q.year_from_inclusive);
        assert_eq!(None, q.year_to_exclusive);
    }
}
