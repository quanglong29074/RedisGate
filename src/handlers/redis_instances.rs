// Redis instance management handlers

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::api_models::{
    ApiResponse, CreateRedisInstanceRequest, PaginatedResponse, PaginationParams,
    RedisInstanceResponse,
};
use crate::auth::hash_password;
use crate::middleware::{AppState, CurrentUser};
use crate::models::{ApiKey, RedisInstance};

// Generate a secure Redis password
fn generate_redis_password() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*";
    let mut rng = rand::thread_rng();
    
    (0..24)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub async fn create_redis_instance(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateRedisInstanceRequest>,
) -> Result<Json<ApiResponse<RedisInstanceResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Validate input
    if let Err(errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(format!("Validation error: {:?}", errors))),
        ));
    }

    // Check if user has access to the organization
    let org_membership = sqlx::query!(
        r#"
        SELECT role FROM organization_memberships 
        WHERE organization_id = $1 AND user_id = $2 AND is_active = true
        "#,
        payload.organization_id,
        current_user.id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!("Database error: {}", e))),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Organization not found or access denied".to_string())),
        )
    })?;

    // Check if organization has reached Redis instance limit
    let instance_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM redis_instances WHERE organization_id = $1 AND deleted_at IS NULL",
        payload.organization_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!("Database error: {}", e))),
        )
    })?
    .count
    .unwrap_or(0);

    let org_limits = sqlx::query!(
        "SELECT max_redis_instances FROM organizations WHERE id = $1",
        payload.organization_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!("Database error: {}", e))),
        )
    })?;

    if instance_count >= org_limits.max_redis_instances as i64 {
        return Err((
            StatusCode::CONFLICT,
            Json(ApiResponse::error("Organization has reached the maximum number of Redis instances".to_string())),
        ));
    }

    // Check if slug is unique within organization
    let existing_instance = sqlx::query!(
        "SELECT id FROM redis_instances WHERE organization_id = $1 AND slug = $2 AND deleted_at IS NULL",
        payload.organization_id,
        payload.slug
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!("Database error: {}", e))),
        )
    })?;

    if existing_instance.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ApiResponse::error("Redis instance with this slug already exists in the organization".to_string())),
        ));
    }

    // Create dedicated API key for this Redis instance
    let api_key_id = Uuid::new_v4();
    let api_key_name = format!("{}-redis-key", payload.name);
    let api_key = format!("rg_redis_{}", Uuid::new_v4().simple());
    let key_prefix = api_key.chars().take(8).collect::<String>();
    let key_hash = hash_password(&api_key).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!("Key hashing error: {}", e))),
        )
    })?;

    let now = Utc::now();

    // Create API key
    sqlx::query!(
        r#"
        INSERT INTO api_keys (id, name, key_hash, key_prefix, user_id, organization_id, scopes, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
        api_key_id,
        api_key_name,
        key_hash,
        key_prefix,
        current_user.id,
        payload.organization_id,
        &vec!["redis:*".to_string()], // Full Redis access for this instance
        now,
        now
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!("Failed to create API key: {}", e))),
        )
    })?;

    // Generate Redis password and hash it
    let redis_password = generate_redis_password();
    let redis_password_hash = hash_password(&redis_password).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!("Password hashing error: {}", e))),
        )
    })?;

    // Create Redis instance
    let instance_id = Uuid::new_v4();
    let redis_version = payload.redis_version.unwrap_or_else(|| "7.2".to_string());
    let persistence_enabled = payload.persistence_enabled.unwrap_or(true);
    let backup_enabled = payload.backup_enabled.unwrap_or(false);
    let namespace = format!("redis-{}", payload.organization_id.simple());
    
    // Use a default port range (Redis typically uses 6379, but we'll assign dynamically)
    let port = 6379;
    let domain = format!("{}.{}.redis.local", payload.slug, payload.organization_id.simple());

    sqlx::query!(
        r#"
        INSERT INTO redis_instances (
            id, name, slug, organization_id, api_key_id, port, domain,
            max_memory, current_memory, password_hash, redis_version, namespace,
            status, health_status, cpu_usage_percent, memory_usage_percent,
            connections_count, max_connections, persistence_enabled, backup_enabled,
            created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)
        "#,
        instance_id,
        payload.name,
        payload.slug,
        payload.organization_id,
        api_key_id,
        port,
        domain,
        payload.max_memory,
        0i64, // current_memory starts at 0
        Some(redis_password_hash),
        redis_version,
        namespace,
        "creating", // status
        "unknown", // health_status
        Decimal::new(0, 2), // cpu_usage_percent
        Decimal::new(0, 2), // memory_usage_percent
        0i32, // connections_count
        100i32, // max_connections (default)
        persistence_enabled,
        backup_enabled,
        now,
        now
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!("Failed to create Redis instance: {}", e))),
        )
    })?;

    // Fetch created instance
    let redis_instance = sqlx::query_as!(
        RedisInstance,
        "SELECT * FROM redis_instances WHERE id = $1",
        instance_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!("Failed to fetch created Redis instance: {}", e))),
        )
    })?;

    let instance_response = RedisInstanceResponse {
        id: redis_instance.id,
        name: redis_instance.name,
        slug: redis_instance.slug,
        organization_id: redis_instance.organization_id,
        api_key_id: redis_instance.api_key_id,
        port: redis_instance.port,
        domain: redis_instance.domain,
        max_memory: redis_instance.max_memory,
        current_memory: redis_instance.current_memory,
        redis_version: redis_instance.redis_version,
        namespace: redis_instance.namespace,
        status: redis_instance.status,
        health_status: redis_instance.health_status,
        cpu_usage_percent: redis_instance.cpu_usage_percent,
        memory_usage_percent: redis_instance.memory_usage_percent,
        connections_count: redis_instance.connections_count,
        max_connections: redis_instance.max_connections,
        persistence_enabled: redis_instance.persistence_enabled,
        backup_enabled: redis_instance.backup_enabled,
        last_backup_at: redis_instance.last_backup_at,
        created_at: redis_instance.created_at,
        updated_at: redis_instance.updated_at,
    };

    Ok(Json(ApiResponse::success(instance_response)))
}

pub async fn list_redis_instances(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<PaginationParams>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<ApiResponse<PaginatedResponse<RedisInstanceResponse>>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Check if user has access to the organization
    let _org_membership = sqlx::query!(
        r#"
        SELECT role FROM organization_memberships 
        WHERE organization_id = $1 AND user_id = $2 AND is_active = true
        "#,
        org_id,
        current_user.id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!("Database error: {}", e))),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Organization not found or access denied".to_string())),
        )
    })?;

    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = (page - 1) * limit;

    // Get Redis instances for the organization
    let redis_instances = sqlx::query_as!(
        RedisInstance,
        r#"
        SELECT * FROM redis_instances 
        WHERE organization_id = $1 AND deleted_at IS NULL
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        org_id,
        limit as i64,
        offset as i64
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!("Database error: {}", e))),
        )
    })?;

    // Get total count
    let total_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM redis_instances WHERE organization_id = $1 AND deleted_at IS NULL",
        org_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!("Database error: {}", e))),
        )
    })?
    .count
    .unwrap_or(0);

    let instance_responses: Vec<RedisInstanceResponse> = redis_instances
        .into_iter()
        .map(|instance| RedisInstanceResponse {
            id: instance.id,
            name: instance.name,
            slug: instance.slug,
            organization_id: instance.organization_id,
            api_key_id: instance.api_key_id,
            port: instance.port,
            domain: instance.domain,
            max_memory: instance.max_memory,
            current_memory: instance.current_memory,
            redis_version: instance.redis_version,
            namespace: instance.namespace,
            status: instance.status,
            health_status: instance.health_status,
            cpu_usage_percent: instance.cpu_usage_percent,
            memory_usage_percent: instance.memory_usage_percent,
            connections_count: instance.connections_count,
            max_connections: instance.max_connections,
            persistence_enabled: instance.persistence_enabled,
            backup_enabled: instance.backup_enabled,
            last_backup_at: instance.last_backup_at,
            created_at: instance.created_at,
            updated_at: instance.updated_at,
        })
        .collect();

    let total_pages = ((total_count as f64) / (limit as f64)).ceil() as u32;

    let paginated_response = PaginatedResponse {
        items: instance_responses,
        total_count,
        page,
        limit,
        total_pages,
    };

    Ok(Json(ApiResponse::success(paginated_response)))
}

pub async fn get_redis_instance(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Path((org_id, instance_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<RedisInstanceResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Check if user has access to the organization
    let _org_membership = sqlx::query!(
        r#"
        SELECT role FROM organization_memberships 
        WHERE organization_id = $1 AND user_id = $2 AND is_active = true
        "#,
        org_id,
        current_user.id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!("Database error: {}", e))),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Organization not found or access denied".to_string())),
        )
    })?;

    // Get Redis instance
    let redis_instance = sqlx::query_as!(
        RedisInstance,
        "SELECT * FROM redis_instances WHERE id = $1 AND organization_id = $2 AND deleted_at IS NULL",
        instance_id,
        org_id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!("Database error: {}", e))),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Redis instance not found".to_string())),
        )
    })?;

    let instance_response = RedisInstanceResponse {
        id: redis_instance.id,
        name: redis_instance.name,
        slug: redis_instance.slug,
        organization_id: redis_instance.organization_id,
        api_key_id: redis_instance.api_key_id,
        port: redis_instance.port,
        domain: redis_instance.domain,
        max_memory: redis_instance.max_memory,
        current_memory: redis_instance.current_memory,
        redis_version: redis_instance.redis_version,
        namespace: redis_instance.namespace,
        status: redis_instance.status,
        health_status: redis_instance.health_status,
        cpu_usage_percent: redis_instance.cpu_usage_percent,
        memory_usage_percent: redis_instance.memory_usage_percent,
        connections_count: redis_instance.connections_count,
        max_connections: redis_instance.max_connections,
        persistence_enabled: redis_instance.persistence_enabled,
        backup_enabled: redis_instance.backup_enabled,
        last_backup_at: redis_instance.last_backup_at,
        created_at: redis_instance.created_at,
        updated_at: redis_instance.updated_at,
    };

    Ok(Json(ApiResponse::success(instance_response)))
}

pub async fn delete_redis_instance(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Path((org_id, instance_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Check if user has admin access to the organization
    let org_membership = sqlx::query!(
        r#"
        SELECT role FROM organization_memberships 
        WHERE organization_id = $1 AND user_id = $2 AND is_active = true
        "#,
        org_id,
        current_user.id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!("Database error: {}", e))),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Organization not found or access denied".to_string())),
        )
    })?;

    if !["admin", "owner"].contains(&org_membership.role.as_str()) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Insufficient permissions to delete Redis instances".to_string())),
        ));
    }

    // Check if Redis instance exists
    let redis_instance = sqlx::query!(
        "SELECT api_key_id FROM redis_instances WHERE id = $1 AND organization_id = $2 AND deleted_at IS NULL",
        instance_id,
        org_id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!("Database error: {}", e))),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Redis instance not found".to_string())),
        )
    })?;

    let now = Utc::now();

    // Soft delete Redis instance
    sqlx::query!(
        "UPDATE redis_instances SET deleted_at = $1, updated_at = $2 WHERE id = $3",
        now,
        now,
        instance_id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!("Failed to delete Redis instance: {}", e))),
        )
    })?;

    // Deactivate associated API key
    sqlx::query!(
        "UPDATE api_keys SET is_active = false, updated_at = $1 WHERE id = $2",
        now,
        redis_instance.api_key_id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!("Failed to deactivate API key: {}", e))),
        )
    })?;

    Ok(Json(ApiResponse {
        success: true,
        data: None,
        message: Some("Redis instance deleted successfully".to_string()),
        timestamp: Utc::now(),
    }))
}