use pyo3::{Bound, PyAny, PyResult, Python, pyfunction};
use pyo3_async_runtimes::tokio::future_into_py;
use sqlx::Connection;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use tokio::sync::OnceCell;
use tracing::info;

pub mod auth_user;
pub mod auth_user_py;

static DB_POOL: OnceCell<sqlx::PgPool> = OnceCell::const_new();

/// Ping the database to verify connectivity before creating a pool
async fn ping_database(options: &PgConnectOptions, label: &str) -> anyhow::Result<()> {
    info!("[{}] Pinging database to verify connectivity...", label);
    let mut conn = sqlx::postgres::PgConnection::connect_with(options)
        .await
        .map_err(|e| anyhow::anyhow!("[{}] Database unreachable: {}", label, e))?;
    conn.ping()
        .await
        .map_err(|e| anyhow::anyhow!("[{}] Database ping failed: {}", label, e))?;
    conn.close()
        .await
        .map_err(|e| anyhow::anyhow!("[{}] Failed to close ping connection: {}", label, e))?;
    info!("[{}] Database ping successful", label);
    Ok(())
}

/// Create a PostgreSQL connection pool.
/// # Arguments
/// * `options` - The connection options.
/// * `label` - The label for logging.
/// Returns:
/// * `sqlx::PgPool` - The connection pool.
/// Errors:
/// * `anyhow::Error` - If the connection pool creation fails.
async fn return_pg_pool(options: PgConnectOptions, label: &str) -> anyhow::Result<sqlx::PgPool> {
    ping_database(&options, label).await?;
    PgPoolOptions::new()
        .max_connections(5)
        .min_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .idle_timeout(std::time::Duration::from_secs(300))
        .test_before_acquire(true)
        .connect_with(options)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to {} database: {}", label, e))
}

/// Start a connection pool to the PostgreSQL database
/// Returns:
/// * `anyhow::Result<&'static sqlx::PgPool>` - The connection pool.
/// Errors:
/// * `anyhow::Error` - If the connection pool creation fails.
async fn start_conn_pool() -> anyhow::Result<&'static sqlx::PgPool> {
    DB_POOL
        .get_or_try_init(|| async {
            let database = std::env::var("DB_NAME")
                .map_err(|e| anyhow::anyhow!("Error getting DB_NAME: {}", e))?;

            let options = PgConnectOptions::new()
                .username(
                    std::env::var("DB_USER")
                        .map_err(|e| anyhow::anyhow!("Error getting DB_USER: {}", e))?
                        .as_str(),
                )
                .password(
                    std::env::var("DB_PASSWORD")
                        .map_err(|e| anyhow::anyhow!("Error getting DB_PASSWORD: {}", e))?
                        .as_str(),
                )
                .host(
                    std::env::var("DB_HOST")
                        .map_err(|e| anyhow::anyhow!("Error getting DB_HOST: {}", e))?
                        .as_str(),
                )
                .port(
                    std::env::var("DB_PORT")
                        .unwrap_or_else(|_| "5432".to_string())
                        .parse()
                        .map_err(|e| anyhow::anyhow!("Error parsing DB_PORT: {}", e))?,
                )
                .database(database.as_str());

            return_pg_pool(options, "testdb").await
        })
        .await
}

/// Test database validation by executing a simple query
/// Returns:
/// * `anyhow::Result<bool>` - True if the query returns 1, false otherwise.
/// Errors:
/// * `anyhow::Error` - If the query fails.
async fn test_db_connection() -> anyhow::Result<bool> {
    let pool = start_conn_pool().await?;

    let row: (i32,) = sqlx::query_as("SELECT 1").fetch_one(pool).await?;

    Ok(row.0 == 1)
}

#[pyfunction]
/// Test database connection from Python
pub fn test_db_connection_py<'p>(py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
    future_into_py(py, async move {
        test_db_connection().await.map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Failed to test database connection: {}",
                e
            ))
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection() {
        assert!(test_db_connection().await.is_err_and(|e| {
            eprintln!("Database connection error: {}", e);
            false
        }));
    }
}
