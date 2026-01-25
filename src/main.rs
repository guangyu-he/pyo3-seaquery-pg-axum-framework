use pyo3::prelude::*;
use pyo3::types::PyDict;

use axum::response::IntoResponse;
use axum::{Router, routing::get};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(paths(py_example))]
struct ApiDoc;

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "OK", body = String)
    )
)]
async fn py_example() -> impl IntoResponse {
    let result: PyResult<String> = Python::attach(|py| {
        let sys = py.import("sys")?;
        let path = sys.getattr("path")?;
        path.call_method1("append", (".venv/lib/python3.13/site-packages",))?;
        path.call_method1("append", ("./python",))?;

        let module = py.import("main")?;
        let user_cls = module.getattr("User")?;

        let kwargs = PyDict::new(py);
        kwargs.set_item("id", 1)?;
        kwargs.set_item("name", "Bob")?;
        kwargs.set_item("email", "test@example.de")?;

        let user = user_cls.call((), Some(&kwargs))?;

        let s: String = user.getattr("greet")?.call0()?.extract()?;
        Ok(s)
    });

    match result {
        Ok(s) => s.into_response(),
        Err(e) => format!("python error: {e}").into_response(),
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(py_example))
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
