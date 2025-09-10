use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use deadpool_redis::{
    redis::{AsyncCommands, FromRedisValue},
    Pool,
};
use serde_json::json;

use crate::error::{GatewayError, Result}; // Import Result và GatewayError

/// Alias cho pool
type RedisPool = Pool;

/// POST /set/:key/:value
pub async fn set_key(
    State(pool): State<RedisPool>,
    Path((key, value)): Path<(String, String)>,
) -> Result<impl IntoResponse> { // Trả về Result
    let mut conn = pool.get().await?; // Xử lý lỗi từ pool.get()
    let _: () = conn.set(&key, &value).await?; // Xử lý lỗi từ conn.set()

    Ok(Json(json!({ "result": "OK" }))) // Trả về Ok nếu thành công
}

/// GET /get/:key
pub async fn get_key(
    State(pool): State<RedisPool>,
    Path(key): Path<String>,
) -> Result<impl IntoResponse> { // Trả về Result
    let mut conn = pool.get().await?; // Xử lý lỗi từ pool.get()
    let result: Option<String> = conn.get(&key).await?; // Xử lý lỗi từ conn.get()

    match result {
        Some(value) => Ok(Json(json!({ "result": value }))), // Trả về Ok nếu tìm thấy
        None => Err(GatewayError::InstanceNotFound(format!("Key '{}' not found", key))), // Trả về lỗi nếu không tìm thấy
    }
}