use pyo3::{Bound, PyAny, PyResult, Python, pyfunction};
use pyo3_async_runtimes::tokio::future_into_py;

use crate::database::start_conn_pool;

use super::utils::test_db_connection;

#[pyfunction]
/// Test database connection from Python
pub fn test_db_connection_py<'p>(py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
    future_into_py(py, async move {
        let pool = start_conn_pool().await.map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Failed to start database connection pool: {}",
                e
            ))
        })?;
        test_db_connection(&pool).await.map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Failed to test database connection: {}",
                e
            ))
        })
    })
}
