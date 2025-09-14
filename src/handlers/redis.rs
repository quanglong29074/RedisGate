// Redis HTTP API handlers

use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, HeaderMap, Method},
    response::Json,
};
use redis::{Commands, Connection, Client};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, warn, error};

use crate::middleware::AppState;
use crate::models::RedisInstance;

type ErrorResponse = (StatusCode, Json<Value>);

/// Redis command response format
#[derive(serde::Serialize)]
pub struct RedisResponse {
    result: Value,
}

/// Redis error response format
#[derive(serde::Serialize)]
struct RedisErrorResponse {
    error: String,
}

/// Extract API key from headers or query parameters
fn extract_api_key(headers: &HeaderMap, query: &Query<HashMap<String, String>>) -> Option<String> {
    // First try Authorization header
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                info!("Found API key in Authorization header: {}...", &token[..8.min(token.len())]);
                return Some(token.to_string());
            }
        }
    }
    
    // Then try _token query parameter
    if let Some(token) = query.get("_token") {
        info!("Found API key in _token query parameter: {}...", &token[..8.min(token.len())]);
        return Some(token.clone());
    }
    
    warn!("No API key found in headers or query parameters");
    None
}

/// Authenticate API key and get Redis instance
async fn authenticate_and_get_instance(
    state: &AppState,
    api_key: &str,
    instance_id: Uuid,
) -> Result<RedisInstance, ErrorResponse> {
    // Get API key from database
    let api_key_record = sqlx::query!(
        "SELECT id, organization_id, is_active FROM api_keys WHERE key_hash = $1",
        crate::auth::hash_password(api_key).unwrap()
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Database error checking API key: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Internal server error"})),
        )
    })?;

    let api_key_record = api_key_record.ok_or_else(|| {
        warn!("Invalid API key provided");
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid API key"})),
        )
    })?;

    if !api_key_record.is_active.unwrap_or(false) {
        warn!("Inactive API key used");
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "API key is not active"})),
        ));
    }

    // Get Redis instance and verify access
    let instance = sqlx::query_as!(
        RedisInstance,
        r#"
        SELECT id, name, slug, organization_id, api_key_id,
               port, private_ip_address, public_ip_address, domain,
               max_memory, current_memory, password_hash, redis_version,
               namespace, pod_name, service_name,
               status, last_health_check_at, health_status,
               cpu_usage_percent, memory_usage_percent, connections_count, max_connections,
               persistence_enabled, backup_enabled, last_backup_at,
               created_at, updated_at, deleted_at
        FROM redis_instances 
        WHERE id = $1 AND organization_id = $2 AND deleted_at IS NULL
        "#,
        instance_id,
        api_key_record.organization_id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Database error getting Redis instance: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Internal server error"})),
        )
    })?;

    instance.ok_or_else(|| {
        warn!("Redis instance not found or access denied");
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Redis instance not found"})),
        )
    })
}

/// Get Redis connection for an instance
async fn get_redis_connection(_instance: &RedisInstance) -> Result<Connection, ErrorResponse> {
    // For development, we'll connect to localhost:6379
    // In production, this would connect to the actual Redis instance
    let redis_url = "redis://127.0.0.1:6379/";
    
    let client = Client::open(redis_url).map_err(|e| {
        error!("Failed to create Redis client: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to connect to Redis"})),
        )
    })?;

    let connection = client.get_connection().map_err(|e| {
        error!("Failed to get Redis connection: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to connect to Redis"})),
        )
    })?;

    Ok(connection)
}

/// Convert Redis value to JSON
fn redis_value_to_json(value: redis::Value) -> Value {
    match value {
        redis::Value::Nil => Value::Null,
        redis::Value::Int(i) => Value::Number(serde_json::Number::from(i)),
        redis::Value::Data(bytes) => {
            if let Ok(s) = String::from_utf8(bytes) {
                Value::String(s)
            } else {
                Value::Null
            }
        }
        redis::Value::Bulk(values) => {
            let json_values: Vec<Value> = values
                .into_iter()
                .map(redis_value_to_json)
                .collect();
            Value::Array(json_values)
        }
        redis::Value::Status(s) => Value::String(s),
        redis::Value::Okay => Value::String("OK".to_string()),
    }
}

/// Handle PING command
pub async fn handle_ping(
    State(state): State<Arc<AppState>>,
    Path(instance_id): Path<Uuid>,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Result<Json<RedisResponse>, ErrorResponse> {
    info!("PING request for instance_id: {}", instance_id);
    
    let api_key = extract_api_key(&headers, &Query(query)).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Missing API key"})),
        )
    })?;

    let instance = authenticate_and_get_instance(&state, &api_key, instance_id).await?;
    let mut conn = get_redis_connection(&instance).await?;

    let result: String = redis::cmd("PING").query(&mut conn).map_err(|e| {
        error!("Redis PING failed: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Redis command failed"})),
        )
    })?;

    Ok(Json(RedisResponse {
        result: Value::String(result),
    }))
}

/// Handle SET command
pub async fn handle_set(
    State(state): State<Arc<AppState>>,
    Path((instance_id, key, value)): Path<(Uuid, String, String)>,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Result<Json<RedisResponse>, ErrorResponse> {
    let api_key = extract_api_key(&headers, &Query(query.clone())).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Missing API key"})),
        )
    })?;

    let instance = authenticate_and_get_instance(&state, &api_key, instance_id).await?;
    let mut conn = get_redis_connection(&instance).await?;

    // Handle optional parameters from query string
    let result = if let Some(ex) = query.get("EX") {
        let expire_seconds: u64 = ex.parse().map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid EX parameter"})),
            )
        })?;
        conn.set_ex(&key, &value, expire_seconds)
    } else {
        conn.set(&key, &value)
    }
    .map_err(|e| {
        error!("Redis SET failed: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Redis command failed"})),
        )
    })?;

    Ok(Json(RedisResponse {
        result: redis_value_to_json(result),
    }))
}

/// Handle GET command
pub async fn handle_get(
    State(state): State<Arc<AppState>>,
    Path((instance_id, key)): Path<(Uuid, String)>,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Result<Json<RedisResponse>, ErrorResponse> {
    let api_key = extract_api_key(&headers, &Query(query)).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Missing API key"})),
        )
    })?;

    let instance = authenticate_and_get_instance(&state, &api_key, instance_id).await?;
    let mut conn = get_redis_connection(&instance).await?;

    let result: redis::Value = conn.get(&key).map_err(|e| {
        error!("Redis GET failed: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Redis command failed"})),
        )
    })?;

    Ok(Json(RedisResponse {
        result: redis_value_to_json(result),
    }))
}

/// Handle DEL command
pub async fn handle_del(
    State(state): State<Arc<AppState>>,
    Path((instance_id, key)): Path<(Uuid, String)>,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Result<Json<RedisResponse>, ErrorResponse> {
    let api_key = extract_api_key(&headers, &Query(query)).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Missing API key"})),
        )
    })?;

    let instance = authenticate_and_get_instance(&state, &api_key, instance_id).await?;
    let mut conn = get_redis_connection(&instance).await?;

    let result: i32 = conn.del(&key).map_err(|e| {
        error!("Redis DEL failed: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Redis command failed"})),
        )
    })?;

    Ok(Json(RedisResponse {
        result: Value::Number(serde_json::Number::from(result)),
    }))
}

/// Handle generic Redis command via POST with JSON body
pub async fn handle_generic_command(
    State(state): State<Arc<AppState>>,
    Path(instance_id): Path<Uuid>,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
    Json(payload): Json<Vec<Value>>,
) -> Result<Json<RedisResponse>, ErrorResponse> {
    let api_key = extract_api_key(&headers, &Query(query)).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Missing API key"})),
        )
    })?;

    let instance = authenticate_and_get_instance(&state, &api_key, instance_id).await?;
    let mut conn = get_redis_connection(&instance).await?;

    if payload.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Empty command"})),
        ));
    }

    // Extract command and arguments
    let command = payload[0].as_str().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid command format"})),
        )
    })?;

    let args: Vec<String> = payload[1..]
        .iter()
        .map(|v| match v {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            _ => v.to_string(),
        })
        .collect();

    info!("Executing Redis command: {} with args: {:?}", command, args);

    let result = match command.to_uppercase().as_str() {
        "PING" => {
            let result: String = redis::cmd("PING").query(&mut conn).map_err(|e| {
                error!("Redis PING failed: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Redis command failed"})),
                )
            })?;
            redis::Value::Status(result)
        }
        "SET" => {
            if args.len() < 2 {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "SET requires key and value"})),
                ));
            }
            conn.set(&args[0], &args[1]).map_err(|e| {
                error!("Redis SET failed: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Redis command failed"})),
                )
            })?
        }
        "GET" => {
            if args.is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "GET requires key"})),
                ));
            }
            conn.get(&args[0]).map_err(|e| {
                error!("Redis GET failed: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Redis command failed"})),
                )
            })?
        }
        "DEL" => {
            if args.is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "DEL requires key"})),
                ));
            }
            let count: i32 = conn.del(&args[0]).map_err(|e| {
                error!("Redis DEL failed: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Redis command failed"})),
                )
            })?;
            redis::Value::Int(count as i64)
        }
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": format!("Unsupported command: {}", command)})),
            ));
        }
    };

    Ok(Json(RedisResponse {
        result: redis_value_to_json(result),
    }))
}

/// Debug handler to see what requests are coming in
pub async fn handle_debug_request(
    State(_state): State<Arc<AppState>>,
    Path((instance_id, path)): Path<(Uuid, String)>,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
    method: axum::http::Method,
) -> Result<Json<Value>, ErrorResponse> {
    info!("DEBUG: {} request to /redis/{}/{} with query: {:?}", method, instance_id, path, query);
    
    // Log authorization header
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            info!("DEBUG: Authorization header: {}", auth_str);
        }
    }
    
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": format!("Debug: {} to /redis/{}/{} not implemented", method, instance_id, path)})),
    ))
}