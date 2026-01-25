use pyo3::{Bound, PyAny, PyResult, Python, pyfunction};
use pyo3_async_runtimes::tokio::future_into_py;
use sqlx::postgres::PgConnectOptions;

pub mod auth_user;

/// Start a connection pool to the PostgreSQL database
async fn start_conn_pool() -> sqlx::PgPool {
    let options = PgConnectOptions::new()
        .username("testdbuser")
        .password("testdbpass")
        .host("127.0.0.1")
        .port(5432)
        .database("testdb");

    sqlx::PgPool::connect_with(options).await.unwrap()
}

/// Test database connection by executing a simple query
async fn test_db_connection() {
    let pool = start_conn_pool().await;
    let mut connection = pool.try_acquire().unwrap();

    let row: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(&mut *connection)
        .await
        .unwrap();

    assert_eq!(row.0, 1);
}

#[pyfunction]
/// Test database connection from Python
pub fn test_db_connection_py<'p>(py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
    future_into_py(py, async move {
        test_db_connection().await;
        Ok(())
    })
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[tokio::test]
    async fn test_connection() {
        test_db_connection().await;
    }
}
