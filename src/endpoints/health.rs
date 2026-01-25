#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Health check OK", body = String)
    )
)]
/// Health check endpoint
pub async fn health() -> &'static str {
    "OK"
}
