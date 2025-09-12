use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use deadpool_redis::CreatePoolError;
use serde_json::Error as SerdeJsonError;

#[derive(Error, Debug)]
pub enum GatewayError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Pool error: {0}")]
    Pool(#[from] deadpool_redis::PoolError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Instance not found: {0}")]
    InstanceNotFound(String),

    #[error("Failed to connect to Redis instance: {0}")]
    RedisConnectionError(String),


    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Internal server error")]
    Internal,

    #[error("Kubernetes client error: {0}")]
    KubeError(#[from] kube::Error),

    #[error("Create pool error: {0}")]
    CreatePool(#[from] CreatePoolError),

    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] SerdeJsonError),
}

impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        let (status, message) = match & self {
            GatewayError::Redis(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Redis operation error: {}", e)),
            GatewayError::Pool(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Redis pool error: {}", e)),
            GatewayError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            GatewayError::InstanceNotFound(id) => (StatusCode::NOT_FOUND, format!("Instance not found: {}", id)),
            GatewayError::BadRequest(msg) => (StatusCode::BAD_REQUEST, format!("Bad request: {}", msg)),
            GatewayError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            GatewayError::KubeError(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Kubernetes client error: {}", e),),
            GatewayError::CreatePool(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Create pool error: {}", e)),
            GatewayError::SerdeJson(e) => (StatusCode::BAD_REQUEST, format!("JSON error: {}", e)),
            GatewayError::RedisConnectionError(e) => (StatusCode::BAD_REQUEST, format!("Connect to Redis instance error: {}", e)),
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, GatewayError>;