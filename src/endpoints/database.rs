use axum::{extract::State, response::IntoResponse};
use sqlx::PgPool;

use crate::database::utils::test_db_connection;

#[utoipa::path(
get,
path = "/db_health",
responses(
    (status = 200, description = "Database connection is healthy", body = String),
)
)]
pub async fn handle_db_health(pg_pool: State<Option<PgPool>>) -> impl IntoResponse {
    let Some(pool) = pg_pool.0 else {
        return (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "Database not configured".to_string(),
        )
            .into_response();
    };
    match test_db_connection(&pool).await {
        Ok(_) => "Database connection is healthy".into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database connection error: {}", e),
        )
            .into_response(),
    }
}
