use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use tokio::sync::OnceCell;

pub mod auth_user;
pub mod auth_user_py;
pub mod utils;
pub mod utils_py;

static DB_POOL: OnceCell<sqlx::PgPool> = OnceCell::const_new();

/// Create a PostgreSQL connection pool.
/// # Arguments
/// * `options` - The connection options.
/// * `label` - The label for logging.
/// Returns:
/// * `sqlx::PgPool` - The connection pool.
/// Errors:
/// * `anyhow::Error` - If the connection pool creation fails.
async fn return_pg_pool(options: PgConnectOptions, label: &str) -> anyhow::Result<sqlx::PgPool> {
    utils::ping_database(&options, label).await?;
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
pub async fn start_conn_pool() -> anyhow::Result<&'static sqlx::PgPool> {
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
