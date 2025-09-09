// src/handlers/health.rs
use axum::response::IntoResponse;
use axum::Json;
use metrics_exporter_prometheus::PrometheusHandle;
use serde_json::json;

/// GET /healthz
pub async fn health_check() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

/// GET /metrics
pub async fn metrics_endpoint(handle: PrometheusHandle) -> impl IntoResponse {
    let metrics_text = handle.render(); // Render all recorded metrics
    (axum::http::StatusCode::OK, metrics_text) // Return to Prometheus
}
