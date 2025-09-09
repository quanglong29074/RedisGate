// handlers/health.rs
use axum::{response::IntoResponse, Json};
use serde_json::json;

/// GET /healthz
pub async fn health_check() -> impl IntoResponse {
    // Return simple status for health check
    Json(json!({
        "status": "ok"
    }))
}
