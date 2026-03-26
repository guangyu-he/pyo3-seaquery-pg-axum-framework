use axum::response::IntoResponse;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::{PyResult, Python};

#[utoipa::path(
    get,
    path = "/handle_py_example_cls",
    responses(
        (status = 200, description = "Example of Python class binding", body = String)
    )
)]
/// Example endpoint demonstrating Python integration
pub async fn handle_py_example_cls() -> impl IntoResponse {
    let result: PyResult<String> = Python::attach(|py| {
        let sys = py.import("sys")?;
        let path = sys.getattr("path")?;
        path.call_method1("append", (".venv/lib/python3.13/site-packages",))?;
        path.call_method1("append", ("./python",))?;

        let module = py.import("call_py_from_rust")?;
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

#[utoipa::path(
    get,
    path = "/handle_py_example_cls",
    responses(
        (status = 200, description = "Example of Python function binding", body = String)
    )
)]
pub async fn handle_py_example_func() -> impl IntoResponse {
    let result: PyResult<String> = Python::attach(|py| {
        let sys = py.import("sys")?;
        let path = sys.getattr("path")?;
        path.call_method1("append", (".venv/lib/python3.13/site-packages",))?;
        path.call_method1("append", ("./python",))?;

        let module = py.import("call_py_from_rust")?;
        let hello = module.getattr("hello")?;

        let hello_string = hello.call1(("world",))?.extract::<String>()?;

        Ok(hello_string)
    });

    match result {
        Ok(s) => s.into_response(),
        Err(e) => format!("python error: {e}").into_response(),
    }
}
