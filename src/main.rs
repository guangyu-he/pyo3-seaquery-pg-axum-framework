use std::env;

use axum::{Router, routing::get};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::{Level, error};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use pyo3_seaquery_pg_axum_framework::database::start_conn_pool;
use pyo3_seaquery_pg_axum_framework::endpoints::ApiDoc;
use pyo3_seaquery_pg_axum_framework::endpoints::database::handle_db_health;
use pyo3_seaquery_pg_axum_framework::endpoints::health::health;
use pyo3_seaquery_pg_axum_framework::endpoints::py_example::{
    handle_py_example_cls, handle_py_example_func,
};
use pyo3_seaquery_pg_axum_framework::middleware::log::init_tracing;

#[tokio::main]
async fn main() {
    // Set PYTHONHOME so the embedded Python interpreter can find its standard library.
    // PY_BASE_PREFIX is baked in at compile time by build.rs (points to the real Python
    // installation, not the venv — the venv only has site-packages).
    // Can be overridden at runtime via the PYTHONHOME env var.
    if env::var("PYTHONHOME").is_err() {
        // SAFETY: called once at startup before any other threads are spawned.
        unsafe { env::set_var("PYTHONHOME", env!("PY_BASE_PREFIX")) };
    }

    init_tracing();

    let pool = match start_conn_pool().await {
        Ok(pool) => pool.clone(),
        Err(e) => {
            error!("Failed to create database pool: {}", e);
            panic!("Failed to create database pool");
        }
    };

    let app = Router::new()
        .route("/py_example_cls", get(handle_py_example_cls))
        .route("/py_example_func", get(handle_py_example_func))
        .route("/health", get(health))
        .route("/db_health", get(handle_db_health))
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(pool)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
