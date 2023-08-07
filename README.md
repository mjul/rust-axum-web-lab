# Rust Axum Web Application
A web application in Rust built with Tokio Axum and the Hyper server.

## Structure

- [`src/main.rs`](src/main.rs): The main entry point of the application.
- [`assets`](assets): static assets such as images, stylesheets, *etc*. 
- [`templates`](templates): templates for HTML pages. This is the default location for templates
  when using the `askama` crate. 

## Features

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


## License
Published under the MIT License, see [LICENSE](LICENSE).
