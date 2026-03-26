use sqlx::Connection;
use sqlx::postgres::PgConnectOptions;
use tracing::info;

/// Ping the database to verify connectivity before creating a pool
pub async fn ping_database(options: &PgConnectOptions, label: &str) -> anyhow::Result<()> {
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

/// Test database validation by executing a simple query
/// Returns:
/// * `anyhow::Result<bool>` - True if the query returns 1, false otherwise.
/// Errors:
/// * `anyhow::Error` - If the query fails.
pub async fn test_db_connection(pool: &sqlx::PgPool) -> anyhow::Result<bool> {
    let row: (i32,) = sqlx::query_as("SELECT 1").fetch_one(pool).await?;

    Ok(row.0 == 1)
}

#[cfg(test)]
mod tests {
    use crate::database::start_conn_pool;

    use super::*;

    #[tokio::test]
    async fn test_connection() {
        let pool = start_conn_pool().await.unwrap();
        assert!(test_db_connection(&pool).await.is_err_and(|e| {
            eprintln!("Database connection error: {}", e);
            false
        }));
    }
}
