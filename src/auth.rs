// Authentication utilities for RedisGate

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: Uuid,
    pub email: String,
    pub org_id: Option<Uuid>,
    pub exp: i64,
    pub iat: i64,
}

impl Claims {
    pub fn new(user_id: Uuid, email: String, org_id: Option<Uuid>) -> Self {
        let now = Utc::now();
        let exp = now + Duration::hours(24); // Token expires in 24 hours

        Self {
            user_id,
            email,
            org_id,
            exp: exp.timestamp(),
            iat: now.timestamp(),
        }
    }
}

#[derive(Clone)]
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtManager {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    pub fn create_token(&self, claims: &Claims) -> Result<String, AuthError> {
        encode(&Header::default(), claims, &self.encoding_key)
            .map_err(|_| AuthError::TokenCreationFailed)
    }

    pub fn verify_token(&self, token: &str) -> Result<TokenData<Claims>, AuthError> {
        decode::<Claims>(token, &self.decoding_key, &Validation::default())
            .map_err(|_| AuthError::InvalidToken)
    }
}

#[derive(Debug)]
pub enum AuthError {
    TokenCreationFailed,
    InvalidToken,
    TokenExpired,
    MissingToken,
    InvalidCredentials,
    UserNotFound,
    UserNotActive,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::TokenCreationFailed => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create token"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "Token expired"),
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing authorization token"),
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            AuthError::UserNotFound => (StatusCode::UNAUTHORIZED, "User not found"),
            AuthError::UserNotActive => (StatusCode::UNAUTHORIZED, "User account is not active"),
        };

        (status, message).into_response()
    }
}

pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    bcrypt::verify(password, hash)
}