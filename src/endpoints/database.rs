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
pub async fn handle_db_health(pg_pool: State<PgPool>) -> impl IntoResponse {
    match test_db_connection(&pg_pool).await {
        Ok(_) => "Database connection is healthy".into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database connection error: {}", e),
        )
            .into_response(),
    }
}
