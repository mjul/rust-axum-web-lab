//! A small Axum-based web application
//! The `main` module is the entry point for the application,
//! the [server] module contains the server code.
use std::net::SocketAddr;

// We need this for tracing (logging)
use tracing::info;
use tracing_subscriber;

mod server;

#[tokio::main]
async fn main() {
    // Initialize tracing
    let default_collector = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        // build but do not install the subscriber.
        .finish();

    tracing::subscriber::set_global_default(default_collector)
        .expect("setting default subscriber failed");

    let socket_addr: &SocketAddr = &"127.0.0.1:3000".parse().unwrap();
    server::serve(socket_addr).await;
}
