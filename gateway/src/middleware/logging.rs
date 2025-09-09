// src/middleware/logging.rs
use axum::{
    http::Request,
    middleware::Next,
    response::Response,
};
use tracing::info;
use uuid::Uuid;

/// Request logging middleware with correlation ID
pub async fn request_logger(req: Request<axum::body::Body>, next: Next) -> Response {
    let request_id = Uuid::new_v4();
    info!("➡️ [{}] {} {}", request_id, req.method(), req.uri());

    let response = next.run(req).await;

    info!("⬅️ [{}] status: {}", request_id, response.status());
    response
}
