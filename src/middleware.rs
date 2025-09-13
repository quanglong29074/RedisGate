// Authentication middleware for protecting routes

use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;
use std::sync::Arc;

use crate::auth::{AuthError, Claims, JwtManager};
use crate::models::User;

// Middleware for JWT authentication
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "));

    let token = auth_header.ok_or(AuthError::MissingToken)?;

    let claims = state.jwt_manager.verify_token(token)?;
    
    // Verify user still exists and is active
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE id = $1 AND is_active = true",
        claims.claims.user_id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| AuthError::UserNotFound)?
    .ok_or(AuthError::UserNotFound)?;

    if !user.is_active {
        return Err(AuthError::UserNotActive);
    }

    // Store user info in request extensions for handlers to use
    request.extensions_mut().insert(CurrentUser {
        id: user.id,
        email: user.email,
        username: user.username,
        org_id: claims.claims.org_id,
    });

    Ok(next.run(request).await)
}

// Current user info extracted from JWT
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: uuid::Uuid,
    pub email: String,
    pub username: String,
    pub org_id: Option<uuid::Uuid>,
}

// Application state
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub jwt_manager: JwtManager,
}

impl AppState {
    pub fn new(db_pool: PgPool, jwt_secret: &str) -> Self {
        Self {
            db_pool,
            jwt_manager: JwtManager::new(jwt_secret),
        }
    }
}