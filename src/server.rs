//! HTTP-based application server with routes
use axum::{routing::get, Router};
use std::net::SocketAddr;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing::{info, instrument};
use askama::Template;

#[derive(Template)] // this will generate the code...
#[template(path = "index.html")] // using the template in this path, relative to the `templates` dir in the crate root
struct IndexTemplate<'a> {
    name: &'a str, // the field name should match the variable name in the template
}

// Note how the IndexTemplate implements IntoResponse for Axum, so we can return it directly from a handler:
async fn index() -> IndexTemplate<'static> {
    IndexTemplate { name: "world" }
}

/// Get the application router
pub(crate) fn router() -> Router {
    Router::new()
        .route("/", get(index))
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
