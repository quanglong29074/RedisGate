use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GatewayError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Redis error: {0}")]
    DeadpoolRedis(#[from] deadpool_redis::redis::RedisError),

    #[error("Pool error: {0}")]
    Pool(#[from] deadpool_redis::PoolError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Instance not found: {0}")]
    InstanceNotFound(String),

    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Internal server error")]
    Internal,
}

impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            GatewayError::Redis(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Redis operation error: {}", e)),
            GatewayError::DeadpoolRedis(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Deadpool Redis error: {}", e)),
            GatewayError::Pool(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Redis pool error: {}", e)),
            GatewayError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            GatewayError::InstanceNotFound(id) => (StatusCode::NOT_FOUND, format!("Instance not found: {}", id)),
            GatewayError::BadRequest(msg) => (StatusCode::BAD_REQUEST, format!("Bad request: {}", msg)),
            GatewayError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, GatewayError>;