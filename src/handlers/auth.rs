// Authentication handlers (register, login)

use axum::{extract::State, http::StatusCode, response::Json};
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::api_models::{ApiResponse, LoginRequest, LoginResponse, RegisterRequest, UserResponse};
use crate::auth::{hash_password, verify_password, AuthError, Claims};
use crate::middleware::AppState;
use crate::models::User;

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<UserResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Validate input
    if let Err(errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(format!("Validation error: {:?}", errors))),
        ));
    }

    // Check if user already exists
    let existing_user = sqlx::query!(
        "SELECT id FROM users WHERE email = $1 OR username = $2",
        payload.email,
        payload.username
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!("Database error: {}", e))),
        )
    })?;

    if existing_user.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ApiResponse::error("User already exists with this email or username".to_string())),
        ));
    }

    // Hash password
    let password_hash = hash_password(&payload.password).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!("Password hashing error: {}", e))),
        )
    })?;

    // Create user
    let user_id = Uuid::new_v4();
    let now = Utc::now();
    
    sqlx::query!(
        r#"
        INSERT INTO users (id, email, username, password_hash, first_name, last_name, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        user_id,
        payload.email,
        payload.username,
        password_hash,
        payload.first_name,
        payload.last_name,
        now,
        now
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!("Failed to create user: {}", e))),
        )
    })?;

    // Fetch created user
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(format!("Failed to fetch created user: {}", e))),
            )
        })?;

    let user_response = UserResponse {
        id: user.id,
        email: user.email,
        username: user.username,
        first_name: user.first_name,
        last_name: user.last_name,
        is_active: user.is_active.unwrap_or(false),
        is_verified: user.is_verified.unwrap_or(false),
        created_at: user.created_at.unwrap_or(Utc::now()),
    };

    Ok(Json(ApiResponse::success(user_response)))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Validate input
    if let Err(errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(format!("Validation error: {:?}", errors))),
        ));
    }

    // Find user by email
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE email = $1",
        payload.email
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
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::error("Invalid credentials".to_string())),
        )
    })?;

    // Check if user is active
    if !user.is_active.unwrap_or(false) {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::error("User account is not active".to_string())),
        ));
    }

    // Verify password
    let password_valid = verify_password(&payload.password, &user.password_hash).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!("Password verification error: {}", e))),
        )
    })?;

    if !password_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::error("Invalid credentials".to_string())),
        ));
    }

    // Get user's primary organization (if any)
    let org_id = sqlx::query!(
        "SELECT organization_id FROM organization_memberships WHERE user_id = $1 AND is_active = true ORDER BY created_at ASC LIMIT 1",
        user.id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!("Database error: {}", e))),
        )
    })?
    .map(|row| row.organization_id);

    // Create JWT token
    let claims = Claims::new(user.id, user.email.clone(), org_id);
    let token = state.jwt_manager.create_token(&claims).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!("Token creation failed: {:?}", e))),
        )
    })?;

    let user_response = UserResponse {
        id: user.id,
        email: user.email,
        username: user.username,
        first_name: user.first_name,
        last_name: user.last_name,
        is_active: user.is_active.unwrap_or(false),
        is_verified: user.is_verified.unwrap_or(false),
        created_at: user.created_at.unwrap_or(Utc::now()),
    };

    let login_response = LoginResponse {
        token,
        user: user_response,
    };

    Ok(Json(ApiResponse::success(login_response)))
}