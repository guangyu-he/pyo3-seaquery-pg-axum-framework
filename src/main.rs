use axum::{Router, routing::get};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use pyo3_seaquery_pg_axum_framework::endpoints::ApiDoc;
use pyo3_seaquery_pg_axum_framework::endpoints::health::health;
use pyo3_seaquery_pg_axum_framework::endpoints::py_example::py_example;
use pyo3_seaquery_pg_axum_framework::server::init_tracing;

#[tokio::main]
async fn main() {
    init_tracing();

    let app = Router::new()
        .route("/py_example", get(py_example))
        .route("/health", get(health))
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
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
