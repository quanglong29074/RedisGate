// API key management handlers

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::api_models::{
    ApiKeyCreationResponse, ApiKeyResponse, ApiResponse, CreateApiKeyRequest, PaginatedResponse,
    PaginationParams,
};
use crate::auth::{ApiKeyClaims};
use crate::middleware::{AppState, CurrentUser};
use crate::models::ApiKey;

type ErrorResponse = (StatusCode, Json<ApiResponse<()>>);

// Helper function to convert ApiKey to ApiKeyResponse
fn api_key_to_response(api_key: ApiKey) -> ApiKeyResponse {
    ApiKeyResponse {
        id: api_key.id,
        name: api_key.name,
        key_prefix: api_key.key_prefix,
        organization_id: api_key.organization_id,
        scopes: api_key.scopes.unwrap_or_else(|| vec!["read".to_string()]),
        last_used_at: api_key.last_used_at,
        is_active: api_key.is_active.unwrap_or(true),
        expires_at: api_key.expires_at,
        created_at: api_key.created_at.unwrap_or_else(|| Utc::now()),
    }
}

// Generate a JWT-based API key
fn generate_api_key_jwt(
    state: &AppState, 
    api_key_id: Uuid,
    user_id: Uuid,
    organization_id: Uuid,
    scopes: Vec<String>,
    expires_at: Option<DateTime<Utc>>
) -> Result<(String, String), String> {
    // Generate a key prefix for identification (still useful for display)
    let key_prefix = format!("rg_{}", &api_key_id.to_string()[..8]);
    
    // Create JWT claims for the API key
    let claims = ApiKeyClaims::new(
        api_key_id,
        user_id,
        organization_id,
        scopes,
        key_prefix.clone(),
        expires_at,
    );
    
    // Generate JWT token
    let jwt_token = state.jwt_manager.create_api_key_token(&claims)
        .map_err(|e| format!("Failed to create JWT token: {:?}", e))?;
    
    Ok((jwt_token, key_prefix))
}

pub async fn create_api_key(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<Json<ApiResponse<ApiKeyCreationResponse>>, ErrorResponse> {
    // Validate input
    if let Err(errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(format!("Validation error: {:?}", errors))),
        ));
    }

    // Check if user has access to the organization
    let _org_membership = sqlx::query!(
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
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Organization not found or access denied".to_string())),
        )
    })?;

    // Check if organization has reached API key limit
    let api_key_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM api_keys WHERE organization_id = $1 AND is_active = true",
        payload.organization_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?
    .count
    .unwrap_or(0);

    let org_limits = sqlx::query!(
        "SELECT max_api_keys FROM organizations WHERE id = $1",
        payload.organization_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?;

    if api_key_count >= org_limits.max_api_keys.unwrap_or(10) as i64 {
        return Err((
            StatusCode::CONFLICT,
            Json(ApiResponse::<()>::error("Organization has reached the maximum number of API keys".to_string())),
        ));
    }

    // Generate API key JWT token
    let api_key_id = Uuid::new_v4();
    let (api_key_token, key_prefix) = generate_api_key_jwt(
        &state,
        api_key_id,
        current_user.id,
        payload.organization_id,
        payload.scopes.clone(),
        payload.expires_at,
    ).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Key generation error: {}", e))),
        )
    })?;

    let now = Utc::now();

    // Create API key record with JWT token
    sqlx::query!(
        r#"
        INSERT INTO api_keys (id, name, key_token, key_prefix, user_id, organization_id, scopes, expires_at, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
        api_key_id,
        payload.name,
        api_key_token,
        key_prefix,
        current_user.id,
        payload.organization_id,
        &payload.scopes,
        payload.expires_at,
        now,
        now
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Failed to create API key: {}", e))),
        )
    })?;

    // Fetch created API key
    let created_key = sqlx::query_as!(
        ApiKey,
        r#"SELECT id, name, key_token, key_prefix, user_id, organization_id, scopes, 
                  last_used_at, last_used_ip, is_active, expires_at, created_at, updated_at 
           FROM api_keys WHERE id = $1"#,
        api_key_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Failed to fetch created API key: {}", e))),
        )
    })?;

    let api_key_response = api_key_to_response(created_key);

    let creation_response = ApiKeyCreationResponse {
        api_key: api_key_response,
        key: api_key_token, // Return the JWT token (only on creation)
    };

    Ok(Json(ApiResponse::success(creation_response)))
}

pub async fn list_api_keys(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<PaginationParams>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<ApiResponse<PaginatedResponse<ApiKeyResponse>>>, ErrorResponse> {
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
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Organization not found or access denied".to_string())),
        )
    })?;

    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = (page - 1) * limit;

    // Get API keys for the organization
    let api_keys = sqlx::query_as!(
        ApiKey,
        r#"
        SELECT id, name, key_token, key_prefix, user_id, organization_id, scopes, 
               last_used_at, last_used_ip, is_active, expires_at, created_at, updated_at
        FROM api_keys 
        WHERE organization_id = $1 AND is_active = true
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
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?;

    // Get total count
    let total_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM api_keys WHERE organization_id = $1 AND is_active = true",
        org_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?
    .count
    .unwrap_or(0);

    let api_key_responses: Vec<ApiKeyResponse> = api_keys
        .into_iter()
        .map(api_key_to_response)
        .collect();

    let total_pages = ((total_count as f64) / (limit as f64)).ceil() as u32;

    let paginated_response = PaginatedResponse {
        items: api_key_responses,
        total_count,
        page,
        limit,
        total_pages,
    };

    Ok(Json(ApiResponse::success(paginated_response)))
}

pub async fn get_api_key(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Path((org_id, key_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<ApiKeyResponse>>, ErrorResponse> {
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
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Organization not found or access denied".to_string())),
        )
    })?;

    // Get API key
    let api_key = sqlx::query_as!(
        ApiKey,
        r#"SELECT id, name, key_token, key_prefix, user_id, organization_id, scopes, 
                  last_used_at, last_used_ip, is_active, expires_at, created_at, updated_at
           FROM api_keys WHERE id = $1 AND organization_id = $2 AND is_active = true"#,
        key_id,
        org_id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("API key not found".to_string())),
        )
    })?;

    let api_key_response = api_key_to_response(api_key);

    Ok(Json(ApiResponse::success(api_key_response)))
}

pub async fn revoke_api_key(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Path((org_id, key_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<()>>, ErrorResponse> {
    // Check if user has access to the organization
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
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Organization not found or access denied".to_string())),
        )
    })?;

    // Get API key to check ownership
    let api_key = sqlx::query!(
        "SELECT user_id FROM api_keys WHERE id = $1 AND organization_id = $2 AND is_active = true",
        key_id,
        org_id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("API key not found".to_string())),
        )
    })?;

    // Only key owner or org admin/owner can revoke
    if api_key.user_id != current_user.id && !["admin", "owner"].contains(&org_membership.role.as_str()) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("Insufficient permissions to revoke this API key".to_string())),
        ));
    }

    let now = Utc::now();

    // Revoke API key (soft delete)
    sqlx::query!(
        "UPDATE api_keys SET is_active = false, updated_at = $1 WHERE id = $2",
        now,
        key_id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Failed to revoke API key: {}", e))),
        )
    })?;

    Ok(Json(ApiResponse {
        success: true,
        data: None,
        message: Some("API key revoked successfully".to_string()),
        timestamp: Utc::now(),
    }))
}