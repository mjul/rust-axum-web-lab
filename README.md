# Rust Axum Web Application

A web application in Rust built with Tokio Axum and the Hyper server.

## Structure

- [`src/main.rs`](src/main.rs): The main entry point of the application.
- [`assets`](assets): static assets such as images, stylesheets, *etc*.
- [`templates`](templates): templates for HTML pages. This is the default location for templates
  when using the `askama` crate.

## Features

### Axum Routing

Routing in Axum similar to routing in low-level HTTP libraries like Sinatra, Flask and Express.
See [`src/server.rs`](src/main.rs) for the routing.

It is worth to mention that the concept of route and nested routes.
A route is a single path, like `/foo` or `/bar`. A nested route is a the root route
and everything "under" it. For example, `/root/x` and `/root/y/z` are nested routes under `/root`.

When declaring the routing, keep this in mind:

We serve a single file on a single (non-nested) route like this:

```rust 
    let router = axum::routing::Router::new()
        .route_service("/assets/foo.html", ServeFile::new("assets/foo.html"));
```

We can serve a whole directory like this. Note that we use `.nest_service` since it nests all the routes,
`.route_service` would route the root path to the service only.

```rust 
    let router = axum::routing::Router::new()
        .nest_service("/assets", ServeDir::new("assets"));
```

#### Path Parameters
You can use path parameters to turn path segments into parameters to the handler function:
```rust
    let router = axum::routing::Router::new()
        .route("/languages/years/:year", get(languages_from_year));
```

The handler function would look like this, using the `Path` extractor. It contains a 
single value or a tuple if you match multiple path segments:

```rust
    // Path is an Axum Extract to get the matched value from the path (see below in the route configuration)
    async fn languages_from_year(Path(year): Path<u32>) -> LanguagesTemplate { /* ... */ }
```

#### Query Parameters
The query parameters are extracted using the `Query` extractor. It can be used to extract the
query parameters into an stringly typed HashMap of key-value pairs or a typed struct:

```rust
    // Stringly typed
    async fn languages_from_year_query(Query(params): Query<HashMap<String, String>>) 
                -> LanguagesTemplate { /* ... */ }
```

```rust
    // Typed struct

    /// Axum can use `serde` to deserialize the query parameters into a struct
    #[derive(Deserialize)]
    pub(crate) struct LanguagesFilter {
      year_from_inclusive: Option<u32>,
      year_to_exclusive: Option<u32>,
    }

    async fn languages_by_struct_query(filter: Query<LanguagesFilter>) 
                -> LanguagesTemplate { /* ... */ }
```

### Debugging Axum Handlers

The error messages are terrible when the handler signatures are not correct.

    Unfortunately Rust gives poor error messages if you try to use a function that doesn’t quite match what’s required by Handler.

https://docs.rs/axum/latest/axum/handler/index.html#debugging-handler-type-errors

Use `axum-macros` crate and its `debug_handler` macro to get better error messages. 
Just apply it to the handler function:

```rust
#[debug_handler]
async fn foo( /* ... */ ) -> impl IntoResponse {
    // ...
}
```

### Tracing

Tracing is enabled, see the use of the tracing macros like `info!`.
See [`src/main.rs`](src/main.rs) for the configuration of the tracing library.

Note that it is not enough to configure the tracing library, in many cases
the libraries that are used also need to be configured to use tracing by enabling
the `tracing` feature. See [cargo.toml](cargo.toml).

For example, for `tower-http` the following is needed:

```toml
tower-http = { version = "0.4.3", features = ["fs", "trace"] }
```

### Templating

The application uses the [askama](https://github.com/djc/askama) template library to render HTML pages.

It appears that `askama` and `tera` are popular choices for templating in Rust. However, `askama`
comes with `axum` bindings, so we will use that for now.

#### Template Inheritance
We can use template inheritance to improve consistency and reduce duplication in our templates.
See https://djc.github.io/askama/template_syntax.html

For example, see the `/languages` route in [`src/server.rs`](src/server.rs) and the corresponding
templates in `templates/languages`, *e.g.* [`templates/languages/base.html`](templates/languages/base.html)
and [`templates/languages/index.html`](templates/languages/index.html).

## License

Published under the MIT License, see [LICENSE](LICENSE).
