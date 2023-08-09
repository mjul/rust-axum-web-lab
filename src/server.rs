//! HTTP-based application server with routes
use std::net::SocketAddr;

use askama::Template;
use axum::extract::State;
use axum::handler::Handler;
use axum::{routing::get, Router};
use axum_macros::debug_handler;
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
    languages: Vec<Language>, // the field name should match the variable name in the template
}

async fn languages() -> LanguagesTemplate {
    LanguagesTemplate {
        languages: LANGUAGES.into(),
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
        languages: app_state.old_languages.clone(),
    }
}

async fn stateful_new_languages(State(app_state): State<AppState>) -> LanguagesTemplate {
    LanguagesTemplate {
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
