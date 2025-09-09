use axum::{extract::{Path, State}, response::IntoResponse, Json};
use deadpool_redis::{Pool, redis::AsyncCommands};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;



/// Alias cho pool
type RedisPool = Pool;

/// POST /set/:key/:value
pub async fn set_key(
    State(pool): State<RedisPool>,
    Path((key, value)): Path<(String, String)>,
) -> impl IntoResponse {
    let mut conn = pool.get().await.unwrap();
    let _: () = conn.set(&key, &value).await.unwrap();

    Json(json!({ "result": "OK" }))
}

/// GET /get/:key
pub async fn get_key(
    State(pool): State<RedisPool>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    let mut conn = pool.get().await.unwrap();
    let result: Option<String> = conn.get(&key).await.unwrap();

    match result {
        Some(value) => Json(json!({ "result": value })),
        None => Json(json!({ "error": "Key not found" })),
    }
}
