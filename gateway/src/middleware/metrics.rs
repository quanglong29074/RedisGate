// src/middleware/metrics.rs
use axum::{
    http::Request,
    middleware::Next,
    response::Response,
};
use metrics::{counter, histogram, Label};
use std::time::Instant;

/// Metrics middleware to record request count and duration
pub async fn metrics_middleware(req: Request<axum::body::Body>, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().to_string();

    let response = next.run(req).await;
    let elapsed = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    let labels = vec![
        Label::new("method", method),
        Label::new("status", status),
    ];

    // Record total requests
    counter!("http_requests_total", labels.clone());

    // Record request duration
    let req_duration = histogram!("http_request_duration_seconds", labels);
    req_duration.record(elapsed);

    response
}
