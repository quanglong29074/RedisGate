use axum::{
    extract::{Query, Path, State},
    response::{IntoResponse},
    Json,
};

use deadpool_redis::{
    redis::{AsyncCommands, FromRedisValue},
    Pool,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::json;

use crate::{
    error::{GatewayError, Result},
    server::AppState,
};


#[derive(Deserialize)]
pub struct SetKeyRequest {
    value: String,
    ttl_seconds: Option<u64>,
}

#[derive(Deserialize)]
pub struct MethodOverride {
    method: Option<String>,
    value: Option<String>,
    ttl_seconds: Option<u64>,
}

#[derive(Serialize)]
pub struct SetKeyResponse {
    status: String,
}

#[derive(Serialize)]
pub struct GetKeyResponse {
    key: String,
    value: String,
}

#[derive(Serialize)]
pub struct DeleteKeyResponse {
    deleted: u32,
}

//
// /// Alias cho pool
// type RedisPool = Pool;
//
// /// POST /set/:key/:value
// pub async fn set_key(
//     State(pool): State<RedisPool>,
//     Path((key, value)): Path<(String, String)>,
// ) -> Result<impl IntoResponse> { // Trả về Result
//     let mut conn = pool.get().await?; // Xử lý lỗi từ pool.get()
//     let _: () = conn.set(&key, &value).await?; // Xử lý lỗi từ conn.set()
//
//     Ok(Json(json!({ "result": "OK" }))) // Trả về Ok nếu thành công
// }
//
// /// GET /get/:key
// pub async fn get_key(
//     State(pool): State<RedisPool>,
//     Path(key): Path<String>,
// ) -> Result<impl IntoResponse> { // Trả về Result
//     let mut conn = pool.get().await?; // Xử lý lỗi từ pool.get()
//     let result: Option<String> = conn.get(&key).await?; // Xử lý lỗi từ conn.get()
//
//     match result {
//         Some(value) => Ok(Json(json!({ "result": value }))), // Trả về Ok nếu tìm thấy
//         None => Err(GatewayError::InstanceNotFound(format!("Key '{}' not found", key))), // Trả về lỗi nếu không tìm thấy
//     }
// }


pub async fn get_key(
    Path((instance_name, key)): Path<(String, String)>,
    Query(params): Query<MethodOverride>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    println!("🔑 Đang thực hiện GET key '{}' trên instance '{}'", key, instance_name);
    // Handle method override
    if let Some(method) = &params.method {
        match method.to_uppercase().as_str() {
            "POST" => return set_key_override(instance_name, key, params, state).await,
            "DELETE" => return delete_key_method(instance_name, key, state).await,
            _ => {}
        }
    }

    // Get Redis client for instance
    let mut client = state
        .redis_pool
        .get_client(&instance_name)
        .await
        .ok_or_else(|| GatewayError::InstanceNotFound(instance_name.clone()))?;

    // Execute Redis GET command
    let value: Option<String> = redis::cmd("GET")
        .arg(&key)
        .query_async(&mut client)
        .await
        .map_err(GatewayError::Redis)?;

    match value {
        Some(v) => Ok(Json(serde_json::json!({
            "key": key,
            "value": v
        }))),
        None => Err(GatewayError::BadRequest(format!("Key '{}' not found", key))),
    }
}

pub async fn set_key(
    Path((instance_name, key)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(payload): Json<SetKeyRequest>,
) -> Result<Json<SetKeyResponse>> {
    // Get Redis client for instance
    let mut client = state
        .redis_pool
        .get_client(&instance_name)
        .await
        .ok_or_else(|| GatewayError::InstanceNotFound(instance_name))?;

    // Execute Redis SET command with optional TTL
    if let Some(ttl) = payload.ttl_seconds {
        redis::cmd("SETEX")
            .arg(&key)
            .arg(ttl)
            .arg(&payload.value)
            .query_async::<_, ()>(&mut client)
            .await
            .map_err(GatewayError::Redis)?;
    } else {
        redis::cmd("SET")
            .arg(&key)
            .arg(&payload.value)
            .query_async(&mut client)
            .await
            .map_err(GatewayError::Redis)?;
    }

    Ok(Json(SetKeyResponse {
        status: "OK".to_string(),
    }))
}

async fn set_key_override(
    instance_name: String,
    key: String,
    params: MethodOverride,
    state: AppState,
) -> Result<Json<serde_json::Value>> {
    let value = params.value.ok_or_else(|| {
        GatewayError::BadRequest("Missing 'value' parameter for SET operation".to_string())
    })?;

    let request = SetKeyRequest {
        value,
        ttl_seconds: params.ttl_seconds,
    };

    let response = set_key(
        Path((instance_name, key)),
        State(state),
        Json(request),
    ).await?;

    Ok(Json(serde_json::to_value(response.0)?))
}

pub async fn delete_key(
    Path((instance_name, key)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    delete_key_method(instance_name, key, state).await
}

async fn delete_key_method(
    instance_name: String,
    key: String,
    state: AppState,
) -> Result<Json<serde_json::Value>> {
    // Get Redis client for instance
    let mut client = state
        .redis_pool
        .get_client(&instance_name)
        .await
        .ok_or_else(|| GatewayError::InstanceNotFound(instance_name))?;

    // Execute Redis DEL command
    let deleted: u32 = redis::cmd("DEL")
        .arg(&key)
        .query_async(&mut client)
        .await
        .map_err(GatewayError::Redis)?;

    Ok(Json(serde_json::json!({
        "deleted": deleted
    })))
}