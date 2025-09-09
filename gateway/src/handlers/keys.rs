use axum::{extract::{Path, State}, response::IntoResponse, Json};
use redis::AsyncCommands;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

// Alias cho Redis connection (dùng lại từ main)
type RedisConn = Arc<Mutex<redis::aio::MultiplexedConnection>>;

/// POST /set/:key/:value
pub async fn set_key(
    State(conn): State<RedisConn>,
    Path((key, value)): Path<(String, String)>,
) -> impl IntoResponse {
    let mut conn = conn.lock().await;
    let _: () = conn.set(&key, &value).await.unwrap();

    Json(json!({ "result": "OK" }))
}

/// GET /get/:key
pub async fn get_key(
    State(conn): State<RedisConn>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    let mut conn = conn.lock().await;
    let result: Option<String> = conn.get(&key).await.unwrap();

    match result {
        Some(value) => Json(json!({ "result": value })),
        None => Json(json!({ "error": "Key not found" })),
    }
}
